//! Data lifecycle and visibility: what a delete removes, what an erase wipes, and which calls
//! must not tell a reader that a post they cannot see exists.
#![allow(missing_docs)]

mod common;

use axum::http::StatusCode;
use common::{Person, app};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

async fn insert_post(
    db: &PgPool,
    author: Uuid,
    text: &str,
    visibility: &str,
    reply_to: Option<Uuid>,
) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "insert into posts (id, author_id, text, visibility, reply_to_id) values ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(author)
    .bind(text)
    .bind(visibility)
    .bind(reply_to)
    .execute(db)
    .await
    .expect("insert post");
    id
}

async fn follow(db: &PgPool, follower: Uuid, followee: Uuid) {
    sqlx::query(
        "insert into follows (follower_id, followee_id, state) values ($1, $2, 'accepted')",
    )
    .bind(follower)
    .bind(followee)
    .execute(db)
    .await
    .expect("follow");
}

const MISSING: &str = "Post not found.";

/// The status and `error` of the five calls that used to tell a hidden post from a missing one.
async fn probe(who: &mut Person, post: Uuid) -> Vec<(StatusCode, Value)> {
    let id = post.to_string();
    let replies = [
        who.browser.delete(&format!("/posts/{id}")).await,
        who.browser.post_empty(&format!("/posts/{id}/pin")).await,
        who.browser.post_empty(&format!("/posts/{id}/hide")).await,
        who.browser
            .post(
                "/reports",
                json!({ "postId": id, "reason": "spam", "note": "" }),
            )
            .await,
        who.browser.get(&format!("/posts/{id}")).await,
    ];
    replies
        .into_iter()
        .map(|reply| (reply.status, reply.json()["error"].clone()))
        .collect()
}

#[sqlx::test(migrations = "./migrations")]
async fn oracle_hidden_post_answers_like_a_missing_one(db: PgPool) {
    let app = app(db.clone());
    let ana = app.person(&db, "ana").await;
    let mut stranger = app.person(&db, "stranger").await;
    let mut ben = app.person(&db, "ben").await;
    let mut cat = app.person(&db, "cat").await;
    follow(&db, ben.id, ana.id).await;
    let blocked = insert_post(&db, ana.id, "blocked author", "public", None).await;
    sqlx::query("insert into blocks (user_id, target_id) values ($1, $2)")
        .bind(ana.id)
        .bind(stranger.id)
        .execute(&db)
        .await
        .expect("block");
    let followers = insert_post(&db, ana.id, "followers only", "followers", None).await;
    let direct = insert_post(&db, ana.id, "direct", "direct", None).await;

    let missing = probe(&mut stranger, Uuid::now_v7()).await;
    assert!(
        missing
            .iter()
            .all(|(status, error)| *status == StatusCode::NOT_FOUND && *error == MISSING),
        "{missing:?}"
    );
    for hidden in [followers, direct] {
        assert_eq!(probe(&mut stranger, hidden).await, missing, "{hidden}");
        assert_eq!(probe(&mut cat, hidden).await, missing, "{hidden}");
    }
    // A follower sees the followers-only post, so the calls answer for what they are.
    let seen = probe(&mut ben, followers).await;
    assert_eq!(seen[0].0, StatusCode::FORBIDDEN, "{seen:?}");
    assert_eq!(seen[0].1, "You can only delete your own posts.");
    assert_eq!(seen[1].0, StatusCode::FORBIDDEN, "{seen:?}");
    assert_eq!(seen[2].0, StatusCode::FORBIDDEN, "{seen:?}");
    assert_eq!(seen[3].0, StatusCode::OK, "{seen:?}");
    assert_eq!(seen[4].0, StatusCode::OK, "{seen:?}");
    // Blocked either way, a public post still gets its own answer, not the missing one.
    let blocked_view = probe(&mut stranger, blocked).await;
    assert_eq!(blocked_view[0].0, StatusCode::FORBIDDEN, "{blocked_view:?}");
    assert_eq!(blocked_view[4].1, "This post is unavailable.");
}

#[sqlx::test(migrations = "./migrations")]
async fn oracle_owner_and_staff_keep_their_answers(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut moderator = app.person(&db, "mod").await;
    sqlx::query("update users set role = 'moderator' where id = $1")
        .bind(moderator.id)
        .execute(&db)
        .await
        .expect("promote");
    let direct = insert_post(&db, ana.id, "direct", "direct", None).await;
    let followers = insert_post(&db, ana.id, "followers", "followers", None).await;

    ana.browser
        .post_empty(&format!("/posts/{direct}/pin"))
        .await
        .ok();
    ana.browser.delete(&format!("/posts/{direct}")).await.ok();
    moderator
        .browser
        .delete(&format!("/posts/{followers}"))
        .await
        .ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn oracle_reply_handle_needs_a_visible_parent(db: PgPool) {
    let app = app(db.clone());
    let ana = app.person(&db, "ana").await;
    let ben = app.person(&db, "ben").await;
    let mut cat = app.person(&db, "cat").await;
    let mut dan = app.person(&db, "dan").await;
    follow(&db, ben.id, ana.id).await;
    follow(&db, dan.id, ana.id).await;
    let public_parent = insert_post(&db, ana.id, "public parent", "public", None).await;
    let hidden_parent = insert_post(&db, ana.id, "followers parent", "followers", None).await;
    let reply_public = insert_post(&db, ben.id, "re public", "public", Some(public_parent)).await;
    let reply_hidden = insert_post(&db, ben.id, "re hidden", "public", Some(hidden_parent)).await;

    let seen = cat
        .browser
        .get(&format!("/posts/{reply_public}"))
        .await
        .ok();
    assert_eq!(seen["post"]["replyTo"]["handle"], "ana", "{seen}");
    assert_eq!(seen["post"]["replyTo"]["id"], public_parent.to_string());

    let hidden = cat
        .browser
        .get(&format!("/posts/{reply_hidden}"))
        .await
        .ok();
    assert_eq!(hidden["post"]["replyTo"], Value::Null, "{hidden}");
    let allowed = dan
        .browser
        .get(&format!("/posts/{reply_hidden}"))
        .await
        .ok();
    assert_eq!(allowed["post"]["replyTo"]["handle"], "ana", "{allowed}");

    sqlx::query("update posts set deleted_at = now(), text = '' where id = $1")
        .bind(public_parent)
        .execute(&db)
        .await
        .expect("delete parent");
    let gone = cat
        .browser
        .get(&format!("/posts/{reply_public}"))
        .await
        .ok();
    assert_eq!(gone["post"]["replyTo"], Value::Null, "{gone}");
}

async fn count(db: &PgPool, sql: &'static str, bind: Uuid) -> i64 {
    sqlx::query_scalar(sql)
        .bind(bind)
        .fetch_one(db)
        .await
        .expect("count")
}

#[sqlx::test(migrations = "./migrations")]
async fn lifecycle_deleting_a_post_removes_what_hangs_on_it(db: PgPool) {
    let data = std::env::temp_dir().join(format!("tuita-privacy-{}", Uuid::now_v7()));
    let dir = data.display().to_string();
    let app = common::app_with(db.clone(), |var| (var == "DATA_DIR").then(|| dir.clone()));
    let mut ana = app.person(&db, "ana").await;
    let ben = common::insert_user(&db, "ben").await;
    let post = insert_post(&db, ana.id, "regrettable", "public", None).await;
    let other = insert_post(&db, ana.id, "keeper", "public", None).await;

    let mut media = Vec::new();
    for (name, post) in [("aa/gone.png", post), ("bb/kept.png", other)] {
        let id = Uuid::now_v7();
        sqlx::query(
            "insert into media (id, owner_id, kind, mime, path, bytes, post_id)
             values ($1, $2, 'image', 'image/png', $3, 3, $4)",
        )
        .bind(id)
        .bind(ana.id)
        .bind(name)
        .bind(post)
        .execute(&db)
        .await
        .expect("media");
        sqlx::query("insert into post_media (post_id, media_id, position) values ($1, $2, 0)")
            .bind(post)
            .bind(id)
            .execute(&db)
            .await
            .expect("post_media");
        let file = data.join("media").join(name);
        std::fs::create_dir_all(file.parent().expect("parent")).expect("mkdir");
        std::fs::write(file, b"png").expect("file");
        media.push(id);
    }
    sqlx::query("insert into post_edits (id, post_id, text) values ($1, $2, 'first draft')")
        .bind(Uuid::now_v7())
        .bind(post)
        .execute(&db)
        .await
        .expect("edit");
    sqlx::query(
        "insert into polls (post_id, options, multiple, ends_at, counts)
         values ($1, '{a,b}', false, now() + interval '1 day', '{1,0}')",
    )
    .bind(post)
    .execute(&db)
    .await
    .expect("poll");
    sqlx::query("insert into poll_votes (post_id, user_id, choice) values ($1, $2, 0)")
        .bind(post)
        .bind(ben)
        .execute(&db)
        .await
        .expect("vote");
    sqlx::query("insert into post_views (post_id, viewer, day) values ($1, 'a1', current_date)")
        .bind(post)
        .execute(&db)
        .await
        .expect("view");
    sqlx::query(
        "insert into scheduled_posts (id, author_id, payload, publish_at, status, post_id)
         values ($1, $2, '{\"text\":\"regrettable\",\"cw\":\"cw\",\"sensitive\":false,\"mediaIds\":[],
                           \"poll\":{\"options\":[\"a\",\"b\"],\"durationHours\":1}}',
                 now() - interval '1 hour', 'published', $3)",
    )
    .bind(Uuid::now_v7())
    .bind(ana.id)
    .bind(post)
    .execute(&db)
    .await
    .expect("scheduled");

    let shown = ana.browser.get("/scheduled").await.ok();
    assert_eq!(shown[0]["text"], "regrettable", "{shown}");

    ana.browser.delete(&format!("/posts/{post}")).await.ok();

    for table in ["post_edits", "polls", "poll_votes", "post_views"] {
        let sql = format!("select count(*) from {table} where post_id = $1");
        let n: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .bind(post)
            .fetch_one(&db)
            .await
            .expect("count");
        assert_eq!(n, 0, "{table}");
    }
    let rows: Vec<Uuid> = sqlx::query_scalar("select id from media")
        .fetch_all(&db)
        .await
        .expect("media rows");
    assert_eq!(rows, [media[1]], "only the kept post's upload is left");
    assert!(!data.join("media/aa/gone.png").exists());
    assert!(data.join("media/bb/kept.png").exists());
    let shown = ana.browser.get("/scheduled").await.ok();
    assert_eq!(shown[0]["text"], "", "{shown}");
    assert_eq!(shown[0]["cw"], "", "{shown}");
    assert_eq!(shown[0]["poll"], Value::Null, "{shown}");
    assert_eq!(
        count(
            &db,
            "select count(*) from post_edits where post_id = $1",
            other
        )
        .await,
        0
    );
    std::fs::remove_dir_all(&data).expect("remove temp dir");
}

#[sqlx::test(migrations = "./migrations")]
async fn client_turning_show_client_off_clears_it_on_old_posts(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.browser();
    ana.signup("ana").await;
    let mut ben = app.browser_at([198, 51, 100, 2]);
    ben.signup("ben").await;
    let mine = ana.post_text("from my laptop").await;
    let theirs = ben.post_text("from theirs").await;
    let client = |db: &PgPool, id: &Value| {
        let id: Uuid = id.as_str().expect("id").parse().expect("uuid");
        let db = db.clone();
        async move {
            sqlx::query_scalar::<_, Option<String>>("select client from posts where id = $1")
                .bind(id)
                .fetch_one(&db)
                .await
                .expect("client")
        }
    };
    assert_eq!(client(&db, &mine["id"]).await.as_deref(), Some("linux"));

    ana.patch("/me", json!({ "showClient": false })).await.ok();
    assert_eq!(client(&db, &mine["id"]).await, None);
    assert_eq!(client(&db, &theirs["id"]).await.as_deref(), Some("linux"));
    let again = ana.post_text("posted later").await;
    assert_eq!(client(&db, &again["id"]).await, None);
}

async fn erase(db: &PgPool, browser: &mut common::Browser) {
    browser
        .send(
            axum::http::Method::DELETE,
            "/me",
            Some(json!({ "password": common::PASSWORD })),
        )
        .await
        .ok();
    let _ = db;
}

#[sqlx::test(migrations = "./migrations")]
async fn erase_wipes_keys_and_per_user_rows(db: PgPool) {
    let app = app(db.clone());
    app.browser().signup("boss").await;
    let mut ana = app.browser_at([198, 51, 100, 7]);
    let me = ana.signup("ana").await;
    let ana_id: Uuid = me["user"]["id"]
        .as_str()
        .expect("id")
        .parse()
        .expect("uuid");
    let ben = common::insert_user(&db, "ben").await;
    let ben_post = insert_post(&db, ben, "poll", "public", None).await;
    sqlx::query(
        "insert into polls (post_id, options, multiple, ends_at, counts)
         values ($1, '{a,b}', false, now() + interval '1 day', '{1,0}')",
    )
    .bind(ben_post)
    .execute(&db)
    .await
    .expect("poll");
    sqlx::query("insert into poll_votes (post_id, user_id, choice) values ($1, $2, 0)")
        .bind(ben_post)
        .bind(ana_id)
        .execute(&db)
        .await
        .expect("vote");
    sqlx::query("insert into status_idempotency (user_id, key) values ($1, 'k')")
        .bind(ana_id)
        .execute(&db)
        .await
        .expect("idempotency");
    sqlx::query(
        "insert into post_views (post_id, viewer, day) values ($1, 'u' || $2::text, current_date)",
    )
    .bind(ben_post)
    .bind(ana_id)
    .execute(&db)
    .await
    .expect("view");
    sqlx::query(
        "insert into post_views (post_id, viewer, day) values ($1, 'u' || $2::text, current_date)",
    )
    .bind(ben_post)
    .bind(ben)
    .execute(&db)
    .await
    .expect("other view");
    for (reporter, target) in [(ana_id, ben), (ben, ana_id)] {
        sqlx::query(
            "insert into reports (id, reporter_id, target_id, reason) values ($1, $2, $3, 'spam')",
        )
        .bind(Uuid::now_v7())
        .bind(reporter)
        .bind(target)
        .execute(&db)
        .await
        .expect("report");
    }
    let note = "https://social.example/users/ana/statuses/dm";
    sqlx::query(
        "insert into messages (id, sender_id, recipient_id, text, ap_id) values ($1, $2, $3, 'hi', $4)",
    )
    .bind(Uuid::now_v7())
    .bind(ana_id)
    .bind(ben)
    .bind(note)
    .execute(&db)
    .await
    .expect("message");
    for activity in [
        json!({ "id": format!("{note}/activity") }),
        json!({ "id": "x" }),
    ] {
        sqlx::query(
            "insert into ap_deliveries (id, inbox_url, activity) values ($1, 'https://r.example/i', $2)",
        )
        .bind(Uuid::now_v7())
        .bind(activity)
        .execute(&db)
        .await
        .expect("delivery");
    }
    sqlx::query(
        "update users set invited_with = 'CODE', notif_prefs = '{\"like\": false}' where id = $1",
    )
    .bind(ana_id)
    .execute(&db)
    .await
    .expect("personal columns");
    let (keys, webauthn): (Option<String>, Uuid) =
        sqlx::query_as("select private_key_pem, webauthn_id from users where id = $1")
            .bind(ana_id)
            .fetch_one(&db)
            .await
            .expect("before");
    assert!(keys.is_some(), "signup makes a key");

    erase(&db, &mut ana).await;

    let row: (Option<String>, Option<String>, Option<String>, Value, Uuid) = sqlx::query_as(
        "select private_key_pem, public_key_pem, invited_with, notif_prefs, webauthn_id
           from users where id = $1",
    )
    .bind(ana_id)
    .fetch_one(&db)
    .await
    .expect("after");
    assert_eq!((row.0, row.1, row.2), (None, None, None));
    assert_eq!(row.3, json!({}));
    assert_ne!(row.4, webauthn, "the passkey user handle is replaced");
    for (table, sql) in [
        (
            "poll_votes",
            "select count(*) from poll_votes where user_id = $1",
        ),
        (
            "status_idempotency",
            "select count(*) from status_idempotency where user_id = $1",
        ),
        (
            "post_views",
            "select count(*) from post_views where viewer = 'u' || $1::text",
        ),
        (
            "reports filed",
            "select count(*) from reports where reporter_id = $1",
        ),
    ] {
        assert_eq!(count(&db, sql, ana_id).await, 0, "{table}");
    }
    assert_eq!(
        count(
            &db,
            "select count(*) from post_views where viewer = 'u' || $1::text",
            ben
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &db,
            "select count(*) from reports where target_id = $1",
            ana_id
        )
        .await,
        1
    );
    let queued: Vec<String> = sqlx::query_scalar("select activity->>'id' from ap_deliveries")
        .fetch_all(&db)
        .await
        .expect("deliveries");
    assert_eq!(queued, ["x"], "the queued copy of the message is gone");
}

#[sqlx::test(migrations = "./migrations")]
async fn erase_keeps_the_key_until_the_account_delete_is_sent(db: PgPool) {
    let app = app(db.clone());
    app.browser().signup("boss").await;
    sqlx::query(
        "insert into users (id, handle, domain, ap_id, inbox_url)
         values ($1, 'far', 'remote.example', 'https://remote.example/users/far', 'https://remote.example/inbox')",
    )
    .bind(Uuid::now_v7())
    .execute(&db)
    .await
    .expect("remote account");
    let mut ana = app.browser_at([198, 51, 100, 8]);
    let me = ana.signup("ana").await;
    let ana_id: Uuid = me["user"]["id"]
        .as_str()
        .expect("id")
        .parse()
        .expect("uuid");

    erase(&db, &mut ana).await;
    let queued = count(
        &db,
        "select count(*) from ap_deliveries where signer_id = $1",
        ana_id,
    )
    .await;
    assert_eq!(queued, 1, "the actor Delete is queued");
    let key: Option<String> = sqlx::query_scalar("select private_key_pem from users where id = $1")
        .bind(ana_id)
        .fetch_one(&db)
        .await
        .expect("key");
    assert!(key.is_some(), "the Delete still has to be signed");

    sqlx::query("delete from ap_deliveries")
        .execute(&db)
        .await
        .expect("sent");
    let stats = tuita_server::workers::cleanup::run_once(&app.state)
        .await
        .expect("cleanup");
    assert_eq!(stats.keys, 1);
    let key: (Option<String>, Option<String>) =
        sqlx::query_as("select private_key_pem, public_key_pem from users where id = $1")
            .bind(ana_id)
            .fetch_one(&db)
            .await
            .expect("key");
    assert_eq!(key, (None, None));
}
