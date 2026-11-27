//! The cleanup worker: `run_once` removes what expired and keeps what is still used.
#![allow(missing_docs)]

mod common;

use std::path::Path;

use serde_json::json;
use sqlx::PgPool;
use tuita_server::workers::cleanup::{CleanupStats, run_once};
use uuid::Uuid;

async fn media(db: &PgPool, owner: Uuid, path: &str, age: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "insert into media (id, owner_id, kind, mime, path, bytes, created_at)
         values ($1, $2, 'image', 'image/png', $3, 3, now() - $4::interval)",
    )
    .bind(id)
    .bind(owner)
    .bind(path)
    .bind(age)
    .execute(db)
    .await
    .expect("insert media");
    id
}

fn touch(dir: &Path, name: &str) {
    let file = dir.join("media").join(name);
    std::fs::create_dir_all(file.parent().expect("parent")).expect("mkdir");
    std::fs::write(file, b"png").expect("write file");
}

async fn count(db: &PgPool, table: &str) -> i64 {
    sqlx::query_scalar(sqlx::AssertSqlSafe(format!("select count(*) from {table}")))
        .fetch_one(db)
        .await
        .expect("count")
}

#[sqlx::test(migrations = "./migrations")]
async fn removes_expired_state_and_keeps_the_rest(db: PgPool) {
    let data = std::env::temp_dir().join(format!("tuita-cleanup-{}", Uuid::now_v7()));
    let dir = data.display().to_string();
    let app = common::app_with(db.clone(), |var| (var == "DATA_DIR").then(|| dir.clone()));
    let ana = app.person(&db, "ana").await;

    sqlx::query(
        "insert into sessions (id, user_id, token_hash, expires_at)
         values ($1, $2, '\\x01', now() - interval '1 minute')",
    )
    .bind(Uuid::now_v7())
    .bind(ana.id)
    .execute(&db)
    .await
    .expect("expired session");
    assert_eq!(count(&db, "sessions").await, 2);

    let ben = common::insert_user(&db, "ben").await;
    sqlx::query("insert into feed_snapshots (user_id, ids, created_at) values ($1, '{}', now() - interval '21 minutes')")
        .bind(ben)
        .execute(&db)
        .await
        .expect("old snapshot");
    sqlx::query("insert into feed_snapshots (user_id, ids) values ($1, '{}')")
        .bind(ana.id)
        .execute(&db)
        .await
        .expect("fresh snapshot");

    let orphan = media(&db, ana.id, "aa/orphan.png", "25 hours").await;
    let missing_file = media(&db, ana.id, "aa/never-written.png", "30 hours").await;
    let fresh = media(&db, ana.id, "bb/fresh.png", "1 hour").await;
    let avatar = media(&db, ana.id, "cc/avatar.png", "48 hours").await;
    let poster = media(&db, ana.id, "dd/poster.png", "48 hours").await;
    let video = media(&db, ana.id, "dd/video.mp4", "48 hours").await;
    let attached = media(&db, ana.id, "ee/attached.png", "48 hours").await;
    let post = Uuid::now_v7();
    sqlx::query("insert into posts (id, author_id, text) values ($1, $2, 'p')")
        .bind(post)
        .bind(ana.id)
        .execute(&db)
        .await
        .expect("post");
    sqlx::query("update media set post_id = $2 where id = $1 or id = $3")
        .bind(attached)
        .bind(post)
        .bind(video)
        .execute(&db)
        .await
        .expect("attach");
    sqlx::query("update users set avatar_id = $2 where id = $1")
        .bind(ana.id)
        .bind(avatar)
        .execute(&db)
        .await
        .expect("avatar");
    sqlx::query("update media set poster_id = $2 where id = $1")
        .bind(video)
        .bind(poster)
        .execute(&db)
        .await
        .expect("poster");
    let card_image = media(&db, ana.id, "ff/card.webp", "48 hours").await;
    sqlx::query("insert into link_cards (url, image_id) values ('https://example.test/', $1)")
        .bind(card_image)
        .execute(&db)
        .await
        .expect("link card");
    let scheduled = media(&db, ana.id, "ff/scheduled.png", "48 hours").await;
    sqlx::query(
        "insert into scheduled_posts (id, author_id, payload, publish_at)
         values ($1, $2, jsonb_build_object('mediaIds', jsonb_build_array($3::text)), now() + interval '1 day')",
    )
    .bind(Uuid::now_v7())
    .bind(ana.id)
    .bind(scheduled.to_string())
    .execute(&db)
    .await
    .expect("scheduled post");
    for day in ["current_date - 3", "current_date"] {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "insert into post_views (post_id, viewer, day) values ($1, 'v', {day})"
        )))
        .bind(post)
        .execute(&db)
        .await
        .expect("view");
    }
    for name in [
        "aa/orphan.png",
        "bb/fresh.png",
        "cc/avatar.png",
        "dd/poster.png",
    ] {
        touch(&data, name);
    }

    let idle = Uuid::now_v7();
    drop(app.state.hub().subscribe(idle));
    for minutes in [-1_i32, 5] {
        sqlx::query(
            "insert into webauthn_challenges (id, state, expires_at)
             values ($1, '{}', now() + make_interval(mins => $2))",
        )
        .bind(Uuid::now_v7())
        .bind(minutes)
        .execute(&db)
        .await
        .expect("challenge");
    }

    let stats = run_once(&app.state).await.expect("cleanup");
    assert_eq!(
        stats,
        CleanupStats {
            sessions: 1,
            snapshots: 1,
            media_rows: 2,
            media_files: 1,
            post_views: 1,
            hub_channels: 1,
            challenges: 1,
            ..CleanupStats::default()
        }
    );

    assert_eq!(count(&db, "sessions").await, 1);
    let snapshots: Vec<Uuid> = sqlx::query_scalar("select user_id from feed_snapshots")
        .fetch_all(&db)
        .await
        .expect("snapshots");
    assert_eq!(snapshots, [ana.id]);
    let left: Vec<Uuid> = sqlx::query_scalar("select id from media order by id")
        .fetch_all(&db)
        .await
        .expect("media");
    let mut kept = vec![
        fresh, avatar, poster, video, attached, card_image, scheduled,
    ];
    kept.sort();
    assert_eq!(left, kept);
    assert!(!left.contains(&orphan) && !left.contains(&missing_file));
    assert!(!data.join("media/aa/orphan.png").exists());
    assert!(data.join("media/bb/fresh.png").exists());
    assert!(data.join("media/cc/avatar.png").exists());
    assert!(data.join("media/dd/poster.png").exists());
    assert_eq!(app.state.hub().channels(), 0);
    assert_eq!(count(&db, "post_views").await, 1);

    assert_eq!(
        run_once(&app.state).await.expect("again"),
        CleanupStats::default()
    );
    std::fs::remove_dir_all(&data).expect("remove temp dir");
}

#[sqlx::test(migrations = "./migrations")]
async fn forgets_inbox_activity_ids_after_seven_days(db: PgPool) {
    let app = common::app(db.clone());
    for (id, age) in [
        ("https://a.example/old", "8 days"),
        ("https://a.example/new", "6 days"),
    ] {
        sqlx::query(
            "insert into ap_seen (activity_id, received_at) values ($1, now() - $2::interval)",
        )
        .bind(id)
        .bind(age)
        .execute(&db)
        .await
        .expect("seen");
    }
    run_once(&app.state).await.expect("cleanup");
    let left: Vec<String> = sqlx::query_scalar("select activity_id from ap_seen")
        .fetch_all(&db)
        .await
        .expect("left");
    assert_eq!(left, ["https://a.example/new"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn tombstones_are_pruned_after_thirty_days(db: PgPool) {
    sqlx::query(
        "insert into ap_tombstones (ap_id, deleted_at) values
            ('https://a.example/notes/old', now() - interval '31 days'),
            ('https://a.example/notes/new', now() - interval '29 days')",
    )
    .execute(&db)
    .await
    .expect("tombstones");
    let app = common::app(db.clone());
    run_once(&app.state).await.expect("cleanup");
    let left: Vec<String> = sqlx::query_scalar("select ap_id from ap_tombstones")
        .fetch_all(&db)
        .await
        .expect("left");
    assert_eq!(left, ["https://a.example/notes/new"]);
}

async fn message(db: &PgPool, from: Uuid, to: Uuid, note: &str, deleted_days_ago: Option<i32>) {
    sqlx::query(
        "insert into messages (id, sender_id, recipient_id, text, ap_id, deleted_at)
         values ($1, $2, $3, '', $4, now() - make_interval(days => $5))",
    )
    .bind(Uuid::now_v7())
    .bind(from)
    .bind(to)
    .bind(note)
    .bind(deleted_days_ago)
    .execute(db)
    .await
    .expect("message");
}

async fn delivery(db: &PgPool, signer: Uuid, activity: serde_json::Value) {
    sqlx::query(
        "insert into ap_deliveries (id, inbox_url, activity, signer_id)
         values ($1, 'https://remote.example/inbox', $2, $3)",
    )
    .bind(Uuid::now_v7())
    .bind(activity)
    .bind(signer)
    .execute(db)
    .await
    .expect("delivery");
}

#[sqlx::test(migrations = "./migrations")]
async fn deleted_messages_go_after_a_week_with_their_queued_deliveries(db: PgPool) {
    let app = common::app(db.clone());
    let ana = common::insert_user(&db, "ana").await;
    let ben = common::insert_user(&db, "ben").await;
    let note = |name: &str| format!("https://social.example/users/ana/statuses/{name}");
    message(&db, ana, ben, &note("old"), Some(8)).await;
    message(&db, ana, ben, &note("old-delete"), Some(9)).await;
    message(&db, ana, ben, &note("recent"), Some(2)).await;
    message(&db, ana, ben, &note("live"), None).await;
    for activity in [
        json!({ "id": format!("{}/activity", note("old")), "object": { "id": note("old") } }),
        json!({ "id": "https://social.example/delete/1", "type": "Delete", "object": note("old-delete") }),
        json!({ "id": format!("{}/activity", note("recent")), "object": { "id": note("recent") } }),
        json!({ "id": format!("{}/activity", note("live")), "object": { "id": note("live") } }),
    ] {
        delivery(&db, ana, activity).await;
    }

    let stats = run_once(&app.state).await.expect("cleanup");
    assert_eq!((stats.messages, stats.deliveries), (2, 2), "{stats:?}");
    let left: Vec<String> = sqlx::query_scalar("select ap_id from messages order by ap_id")
        .fetch_all(&db)
        .await
        .expect("messages");
    assert_eq!(left, [note("live"), note("recent")]);
    assert_eq!(count(&db, "ap_deliveries").await, 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn finished_scheduled_posts_go_after_a_week(db: PgPool) {
    let app = common::app(db.clone());
    let ana = common::insert_user(&db, "ana").await;
    for (status, age) in [
        ("published", "8 days"),
        ("failed", "8 days"),
        ("published", "6 days"),
        ("pending", "-1 day"),
    ] {
        sqlx::query(
            "insert into scheduled_posts (id, author_id, payload, publish_at, status)
             values ($1, $2, '{\"text\":\"secret\",\"cw\":\"\"}', now() - $3::interval, $4)",
        )
        .bind(Uuid::now_v7())
        .bind(ana)
        .bind(age)
        .bind(status)
        .execute(&db)
        .await
        .expect("scheduled post");
    }
    let stats = run_once(&app.state).await.expect("cleanup");
    assert_eq!(stats.scheduled_posts, 2);
    let left: Vec<String> =
        sqlx::query_scalar("select status from scheduled_posts order by status, publish_at")
            .fetch_all(&db)
            .await
            .expect("left");
    assert_eq!(left, ["pending", "published"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn an_upload_is_removed_from_the_store_before_its_row(db: PgPool) {
    let data = std::env::temp_dir().join(format!("tuita-cleanup-{}", Uuid::now_v7()));
    let dir = data.display().to_string();
    let app = common::app_with(db.clone(), |var| (var == "DATA_DIR").then(|| dir.clone()));
    let ana = common::insert_user(&db, "ana").await;
    media(&db, ana, "aa/stuck.png", "48 hours").await;
    // A directory where the file should be: the store cannot delete it, but it is not "missing".
    std::fs::create_dir_all(data.join("media/aa/stuck.png")).expect("blocking directory");

    let stats = run_once(&app.state).await.expect("cleanup");
    assert_eq!((stats.media_rows, stats.media_files), (0, 0), "{stats:?}");
    assert_eq!(
        count(&db, "media").await,
        1,
        "the row stays while the object does"
    );

    std::fs::remove_dir_all(data.join("media/aa/stuck.png")).expect("unblock");
    touch(&data, "aa/stuck.png");
    let stats = run_once(&app.state).await.expect("cleanup again");
    assert_eq!((stats.media_rows, stats.media_files), (1, 1), "{stats:?}");
    assert_eq!(count(&db, "media").await, 0);
    assert!(!data.join("media/aa/stuck.png").exists());
    std::fs::remove_dir_all(&data).expect("remove temp dir");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_deleted_accounts_key_goes_once_no_delivery_needs_it(db: PgPool) {
    let app = common::app(db.clone());
    let ana = common::insert_user(&db, "ana").await;
    let ben = common::insert_user(&db, "ben").await;
    sqlx::query(
        "update users set deleted_at = now(), public_key_pem = 'pub', private_key_pem = 'priv'
          where id = any($1)",
    )
    .bind([ana, ben])
    .execute(&db)
    .await
    .expect("deleted accounts");
    delivery(
        &db,
        ana,
        json!({ "id": "https://social.example/delete/ana" }),
    )
    .await;

    assert_eq!(run_once(&app.state).await.expect("cleanup").keys, 1);
    let keys: Vec<(Uuid, Option<String>)> = sqlx::query_as(
        "select id, private_key_pem from users where handle in ('ana', 'ben') order by handle",
    )
    .fetch_all(&db)
    .await
    .expect("keys");
    assert_eq!(keys, [(ana, Some("priv".to_owned())), (ben, None)]);

    sqlx::query("delete from ap_deliveries")
        .execute(&db)
        .await
        .expect("delivered");
    assert_eq!(run_once(&app.state).await.expect("again").keys, 1);
}
