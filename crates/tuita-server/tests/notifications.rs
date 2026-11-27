//! Notification list, grouping, read state and badges, with fixtures through SQL and
//! `db::notifications::notify`.
#![allow(missing_docs)]

mod common;

use axum::http::StatusCode;
use serde_json::Value;
use sqlx::PgPool;
use tuita_server::db::notifications;
use tuita_server::dto::{NotificationKindDto, StreamEvent};
use uuid::Uuid;

async fn post(db: &PgPool, author: Uuid, text: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query("insert into posts (id, author_id, text) values ($1, $2, $3)")
        .bind(id)
        .bind(author)
        .bind(text)
        .execute(db)
        .await
        .expect("insert post");
    id
}

async fn notify(
    db: &PgPool,
    user: Uuid,
    actor: Uuid,
    kind: NotificationKindDto,
    post: Option<Uuid>,
) {
    let mut conn = db.acquire().await.expect("connection");
    let stored = notifications::notify(&mut conn, user, actor, kind, post)
        .await
        .expect("notify");
    assert!(stored.is_some(), "{kind:?} by {actor} was skipped");
}

fn kinds(page: &Value) -> Vec<String> {
    page["items"]
        .as_array()
        .expect("items")
        .iter()
        .map(|item| item["kind"].as_str().expect("kind").to_owned())
        .collect()
}

#[sqlx::test(migrations = "./migrations")]
async fn four_likes_become_one_group(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let post_id = post(&db, ana.id, "liked").await;
    let mut actors = Vec::new();
    for handle in ["ben", "cat", "dan", "eve"] {
        let actor = common::insert_user(&db, handle).await;
        notify(&db, ana.id, actor, NotificationKindDto::Like, Some(post_id)).await;
        actors.push(actor);
    }
    let page = ana.browser.get("/notifications").await.ok();
    assert_eq!(kinds(&page), ["like"], "{page}");
    let group = &page["items"][0];
    assert_eq!(group["actorCount"], 4);
    let handles: Vec<&str> = group["actors"]
        .as_array()
        .expect("actors")
        .iter()
        .map(|actor| actor["handle"].as_str().expect("handle"))
        .collect();
    assert_eq!(handles, ["eve", "dan", "cat"]);
    assert_eq!(group["read"], false);
    assert_eq!(group["post"]["id"], post_id.to_string());
    assert_eq!(group["post"]["text"], "liked");
    assert_eq!(page["next"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn mentions_filter_keeps_replies_mentions_and_quotes(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let ben = common::insert_user(&db, "ben").await;
    let mine = post(&db, ana.id, "mine").await;
    let theirs = post(&db, ben, "theirs").await;
    notify(&db, ana.id, ben, NotificationKindDto::Like, Some(mine)).await;
    notify(&db, ana.id, ben, NotificationKindDto::Reply, Some(theirs)).await;
    notify(&db, ana.id, ben, NotificationKindDto::Follow, None).await;
    notify(&db, ana.id, ben, NotificationKindDto::Mention, Some(theirs)).await;
    notify(&db, ana.id, ben, NotificationKindDto::Quote, Some(theirs)).await;

    let all = ana.browser.get("/notifications").await.ok();
    assert_eq!(kinds(&all), ["quote", "mention", "follow", "reply", "like"]);
    let mentions = ana.browser.get("/notifications?filter=mentions").await.ok();
    assert_eq!(kinds(&mentions), ["quote", "mention", "reply"]);
    ana.browser
        .get("/notifications?filter=nope")
        .await
        .err(StatusCode::BAD_REQUEST, "Invalid filter.");
}

#[sqlx::test(migrations = "./migrations")]
async fn hidden_actors_and_gone_posts_drop_out(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let ben = common::insert_user(&db, "ben").await;
    let cat = common::insert_user(&db, "cat").await;
    let dan = common::insert_user(&db, "dan").await;
    let eve = common::insert_user(&db, "eve").await;
    let kept = post(&db, ana.id, "kept").await;
    let gone = post(&db, ana.id, "gone").await;
    notify(&db, ana.id, ben, NotificationKindDto::Like, Some(kept)).await;
    notify(&db, ana.id, cat, NotificationKindDto::Like, Some(kept)).await;
    notify(&db, ana.id, dan, NotificationKindDto::Like, Some(kept)).await;
    notify(&db, ana.id, eve, NotificationKindDto::Like, Some(gone)).await;
    notify(&db, ana.id, eve, NotificationKindDto::Follow, None).await;
    sqlx::query("update posts set deleted_at = now() where id = $1")
        .bind(gone)
        .execute(&db)
        .await
        .expect("delete post");
    sqlx::query("insert into blocks (user_id, target_id) values ($1, $2)")
        .bind(ana.id)
        .bind(ben)
        .execute(&db)
        .await
        .expect("block");
    sqlx::query("insert into mutes (user_id, target_id) values ($1, $2)")
        .bind(ana.id)
        .bind(cat)
        .execute(&db)
        .await
        .expect("mute");

    let page = ana.browser.get("/notifications").await.ok();
    assert_eq!(kinds(&page), ["follow", "like"], "{page}");
    assert_eq!(page["items"][1]["actorCount"], 1);
    assert_eq!(page["items"][1]["actors"][0]["handle"], "dan");
    assert_eq!(page["items"][0]["post"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn read_flag_and_mark_read(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let ben = common::insert_user(&db, "ben").await;
    let cat = common::insert_user(&db, "cat").await;
    let mine = post(&db, ana.id, "mine").await;
    notify(&db, ana.id, ben, NotificationKindDto::Like, Some(mine)).await;
    sqlx::query("update notifications set read_at = now()")
        .execute(&db)
        .await
        .expect("read the first");
    notify(&db, ana.id, cat, NotificationKindDto::Like, Some(mine)).await;
    notify(&db, ana.id, cat, NotificationKindDto::Follow, None).await;
    sqlx::query("update notifications set read_at = now() where kind = 'follow'")
        .execute(&db)
        .await
        .expect("read the follow");

    let page = ana.browser.get("/notifications").await.ok();
    assert_eq!(kinds(&page), ["follow", "like"]);
    assert_eq!(page["items"][0]["read"], true);
    assert_eq!(
        page["items"][1]["read"], false,
        "one unread row keeps the group unread"
    );
    assert_eq!(ana.browser.get("/badges").await.ok()["notifications"], 1);

    let done = ana.browser.post_empty("/notifications/read").await.ok();
    assert_eq!(done["ok"], true);
    let badges = ana.browser.get("/badges").await.ok();
    assert_eq!(badges["notifications"], 0);
    assert_eq!(badges["messages"], 0);
    assert_eq!(badges["reports"], Value::Null);
    let page = ana.browser.get("/notifications").await.ok();
    assert_eq!(page["items"][1]["read"], true);
}

#[sqlx::test(migrations = "./migrations")]
async fn pages_by_raw_row_with_a_cursor(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let ben = common::insert_user(&db, "ben").await;
    let theirs = post(&db, ben, "theirs").await;
    let mut ids = Vec::new();
    for _ in 0..5 {
        notify(&db, ana.id, ben, NotificationKindDto::Mention, Some(theirs)).await;
    }
    let rows: Vec<Uuid> = sqlx::query_scalar("select id from notifications order by id desc")
        .fetch_all(&db)
        .await
        .expect("ids");
    ids.extend(rows);

    // `all` reads two raw rows per requested entry.
    let first = ana.browser.get("/notifications?limit=1").await.ok();
    assert_eq!(first["items"].as_array().map(Vec::len), Some(2));
    assert_eq!(first["next"], ids[1].to_string());
    let second = ana
        .browser
        .get(&format!("/notifications?limit=1&before={}", ids[1]))
        .await
        .ok();
    assert_eq!(second["items"][0]["id"], ids[2].to_string());
    assert_eq!(second["items"][1]["id"], ids[3].to_string());
    assert_eq!(second["next"], ids[3].to_string());
    let third = ana
        .browser
        .get(&format!("/notifications?limit=1&before={}", ids[3]))
        .await
        .ok();
    assert_eq!(third["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(third["next"], Value::Null);

    let mentions = ana
        .browser
        .get("/notifications?filter=mentions&limit=2")
        .await
        .ok();
    assert_eq!(mentions["items"].as_array().map(Vec::len), Some(2));
    assert_eq!(mentions["next"], ids[1].to_string());
}

#[sqlx::test(migrations = "./migrations")]
async fn notifications_need_a_session(db: PgPool) {
    let app = common::app(db);
    let mut nobody = app.browser();
    nobody
        .get("/notifications")
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");
    nobody
        .get("/badges")
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");
}

/// The text of the first live `notification` event `rx` holds.
fn toast(
    rx: &mut tokio::sync::broadcast::Receiver<tuita_server::realtime::Event>,
) -> Option<String> {
    match &*rx.try_recv().expect("a notification event") {
        StreamEvent::Notification(event) => Some(
            event
                .post
                .as_ref()
                .map_or_else(String::new, |post| post.text.clone()),
        ),
        other => panic!("expected a notification, got {other:?}"),
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn the_live_preview_shows_only_what_the_recipient_may_open(db: PgPool) {
    let app = common::app(db.clone());
    let ana = app.person(&db, "ana").await;
    let ben = common::insert_user(&db, "ben").await;
    let parent = post(&db, ana.id, "parent").await;
    let mut rx = app.state.hub().subscribe(ana.id);

    for (visibility, text, expected) in [
        ("public", "open reply", "open reply"),
        ("followers", "followers only", ""),
        ("direct", "direct", ""),
    ] {
        let reply = Uuid::now_v7();
        sqlx::query(
            "insert into posts (id, author_id, text, visibility, reply_to_id)
             values ($1, $2, $3, $4, $5)",
        )
        .bind(reply)
        .bind(ben)
        .bind(text)
        .bind(visibility)
        .bind(parent)
        .execute(&db)
        .await
        .expect("reply");
        let mut conn = db.acquire().await.expect("connection");
        let pending = notifications::notify(
            &mut conn,
            ana.id,
            ben,
            NotificationKindDto::Reply,
            Some(reply),
        )
        .await
        .expect("notify");
        notifications::deliver(&app.state, pending).await;
        assert_eq!(toast(&mut rx).as_deref(), Some(expected), "{visibility}");
        while rx.try_recv().is_ok() {}
    }
}
