//! Profiles, follow, block, mute, the lists around them and suggestions.
#![allow(missing_docs)]

mod common;

use axum::http::StatusCode;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use common::{app, insert_user};

async fn make_post(db: &PgPool, author: Uuid, text: &str, reply_to: Option<Uuid>) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query("insert into posts (id, author_id, text, reply_to_id) values ($1, $2, $3, $4)")
        .bind(id)
        .bind(author)
        .bind(text)
        .bind(reply_to)
        .execute(db)
        .await
        .expect("insert post");
    sqlx::query("update users set posts_count = posts_count + 1 where id = $1")
        .bind(author)
        .execute(db)
        .await
        .expect("bump posts");
    id
}

async fn follows(db: &PgPool, follower: Uuid, followee: Uuid) {
    sqlx::query("insert into follows (follower_id, followee_id) values ($1, $2)")
        .bind(follower)
        .bind(followee)
        .execute(db)
        .await
        .expect("insert follow");
    sqlx::query("update users set followers_count = followers_count + 1 where id = $1")
        .bind(followee)
        .execute(db)
        .await
        .expect("bump followers");
    sqlx::query("update users set following_count = following_count + 1 where id = $1")
        .bind(follower)
        .execute(db)
        .await
        .expect("bump following");
}

async fn edge(db: &PgPool, table: &str, user: Uuid, target: Uuid) {
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "insert into {table} (user_id, target_id) values ($1, $2)"
    )))
    .bind(user)
    .bind(target)
    .execute(db)
    .await
    .expect("insert edge");
}

async fn counters(db: &PgPool, id: Uuid) -> (i32, i32) {
    sqlx::query_as("select followers_count, following_count from users where id = $1")
        .bind(id)
        .fetch_one(db)
        .await
        .expect("counters")
}

fn handles(items: &Value) -> Vec<&str> {
    items
        .as_array()
        .expect("array")
        .iter()
        .map(|item| item["handle"].as_str().expect("handle"))
        .collect()
}

fn texts(page: &Value) -> Vec<&str> {
    page["items"]
        .as_array()
        .expect("items")
        .iter()
        .map(|item| item["text"].as_str().expect("text"))
        .collect()
}

#[sqlx::test(migrations = "./migrations")]
async fn profile_card_shows_counters_and_relation(db: PgPool) {
    let app = app(db.clone());
    let ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    follows(&db, bob.id, ana.id).await;
    follows(&db, ana.id, bob.id).await;
    make_post(&db, ana.id, "hello", None).await;
    edge(&db, "mutes", bob.id, ana.id).await;

    let card = bob.browser.get("/users/@Ana").await.ok();
    assert_eq!(card["handle"], "ana");
    assert_eq!(card["displayName"], "Name of ana");
    assert_eq!(card["counts"]["followers"], 1);
    assert_eq!(card["counts"]["following"], 1);
    assert_eq!(card["counts"]["posts"], 1);
    assert_eq!(card["relation"]["following"], true);
    assert_eq!(card["relation"]["followsYou"], true);
    assert_eq!(card["relation"]["muted"], true);
    assert_eq!(card["relation"]["blocked"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn unknown_hidden_and_remote_accounts_are_404(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let gone = insert_user(&db, "gone").await;
    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(gone)
        .execute(&db)
        .await
        .expect("suspend");
    // `nobody@social.example` carries this instance's own domain: a local lookup, no network.
    for path in [
        "/users/nobody",
        "/users/nobody@social.example",
        "/users/gone",
    ] {
        ana.browser
            .get(path)
            .await
            .err(StatusCode::NOT_FOUND, "User not found.");
    }
    ana.browser
        .post_empty("/users/gone/follow")
        .await
        .err(StatusCode::NOT_FOUND, "User not found.");

    sqlx::query("update users set role = 'moderator' where id = $1")
        .bind(ana.id)
        .execute(&db)
        .await
        .expect("promote");
    let card = ana.browser.get("/users/gone").await.ok();
    assert_eq!(card["suspended"], true);
}

#[sqlx::test(migrations = "./migrations")]
async fn routes_need_a_session(db: PgPool) {
    let app = app(db.clone());
    app.person(&db, "ana").await;
    let mut anon = app.browser();
    for path in [
        "/users/ana",
        "/users/ana/posts",
        "/users/ana/followers",
        "/users/ana/following",
        "/users/ana/followed_by_known",
        "/me/blocks",
        "/me/mutes",
        "/suggestions",
    ] {
        anon.get(path)
            .await
            .err(StatusCode::UNAUTHORIZED, "sign in required");
    }
    anon.post_empty("/users/ana/follow")
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");
}

#[sqlx::test(migrations = "./migrations")]
async fn follow_and_unfollow_keep_counters_and_notify_once(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let bob = app.person(&db, "bob").await;
    let mut bob_browser = bob.browser;

    for _ in 0..2 {
        let card = bob_browser.post_empty("/users/ana/follow").await.ok();
        assert_eq!(card["counts"]["followers"], 1);
        assert_eq!(card["relation"]["following"], true);
    }
    assert_eq!(counters(&db, ana.id).await, (1, 0));
    assert_eq!(counters(&db, bob.id).await, (0, 1));
    let notes: i64 = sqlx::query_scalar(
        "select count(*) from notifications where user_id = $1 and actor_id = $2 and kind = 'follow'",
    )
    .bind(ana.id)
    .bind(bob.id)
    .fetch_one(&db)
    .await
    .expect("count");
    assert_eq!(notes, 1);
    assert_eq!(
        ana.browser.get("/users/bob").await.ok()["relation"]["followsYou"],
        true
    );

    for _ in 0..2 {
        let card = bob_browser.delete("/users/ana/follow").await.ok();
        assert_eq!(card["counts"]["followers"], 0);
        assert_eq!(card["relation"]["following"], false);
    }
    assert_eq!(counters(&db, ana.id).await, (0, 0));
    assert_eq!(counters(&db, bob.id).await, (0, 0));
}

#[sqlx::test(migrations = "./migrations")]
async fn follow_refuses_self_and_blocked_accounts(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    ana.browser
        .post_empty("/users/ana/follow")
        .await
        .err(StatusCode::BAD_REQUEST, "You can’t follow yourself.");
    ana.browser
        .post_empty("/users/ana/block")
        .await
        .err(StatusCode::BAD_REQUEST, "You can’t block yourself.");
    ana.browser
        .post_empty("/users/ana/mute")
        .await
        .err(StatusCode::BAD_REQUEST, "You can’t mute yourself.");

    edge(&db, "blocks", ana.id, bob.id).await;
    let msg = "You can’t follow this account.";
    ana.browser
        .post_empty("/users/bob/follow")
        .await
        .err(StatusCode::FORBIDDEN, msg);
    bob.browser
        .post_empty("/users/ana/follow")
        .await
        .err(StatusCode::FORBIDDEN, msg);
    assert_eq!(counters(&db, ana.id).await, (0, 0));
}

#[sqlx::test(migrations = "./migrations")]
async fn the_201st_follow_in_an_hour_is_rate_limited(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    insert_user(&db, "bob").await;
    for _ in 0..200 {
        ana.browser.post_empty("/users/bob/follow").await.ok();
    }
    let reply = ana.browser.post_empty("/users/bob/follow").await;
    reply.err(
        StatusCode::TOO_MANY_REQUESTS,
        "Too many requests. Try again in a few minutes.",
    );
    assert!(!reply.header("retry-after").is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn block_removes_follows_both_ways_and_unblock_restores_nothing(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let bob = insert_user(&db, "bob").await;
    let carol = insert_user(&db, "carol").await;
    follows(&db, ana.id, bob).await;
    follows(&db, bob, ana.id).await;
    follows(&db, carol, ana.id).await;
    assert_eq!(counters(&db, ana.id).await, (2, 1));

    let card = ana.browser.post_empty("/users/bob/block").await.ok();
    assert_eq!(card["relation"]["blocked"], true);
    assert_eq!(card["relation"]["following"], false);
    assert_eq!(card["relation"]["followsYou"], false);
    assert_eq!(card["counts"]["followers"], 0);
    assert_eq!(card["counts"]["following"], 0);
    assert_eq!(counters(&db, ana.id).await, (1, 0));
    assert_eq!(counters(&db, bob).await, (0, 0));
    assert_eq!(counters(&db, carol).await, (0, 1));

    let again = ana.browser.post_empty("/users/bob/block").await.ok();
    assert_eq!(again["relation"]["blocked"], true);
    assert_eq!(counters(&db, ana.id).await, (1, 0));

    let card = ana.browser.delete("/users/bob/block").await.ok();
    assert_eq!(card["relation"]["blocked"], false);
    assert_eq!(card["relation"]["following"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn mute_and_unmute_toggle_the_relation(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    insert_user(&db, "bob").await;
    for _ in 0..2 {
        let card = ana.browser.post_empty("/users/bob/mute").await.ok();
        assert_eq!(card["relation"]["muted"], true);
    }
    for _ in 0..2 {
        let card = ana.browser.delete("/users/bob/mute").await.ok();
        assert_eq!(card["relation"]["muted"], false);
    }
}

/// ana: one (pinned), two, pic (media), a reply to bob's post, and a repost of bob's post.
struct Profile {
    ana: Uuid,
    bob: Uuid,
}

async fn profile_fixture(db: &PgPool, app: &common::App) -> (Profile, common::Person) {
    let bob = app.person(db, "bob").await;
    let ana = insert_user(db, "ana").await;
    let one = make_post(db, ana, "one", None).await;
    make_post(db, ana, "two", None).await;
    let pic = make_post(db, ana, "pic", None).await;
    let media = Uuid::now_v7();
    sqlx::query(
        "insert into media (id, owner_id, kind, mime, path, bytes)
         values ($1, $2, 'image', 'image/png', 'x.png', 1)",
    )
    .bind(media)
    .bind(ana)
    .execute(db)
    .await
    .expect("media");
    sqlx::query("insert into post_media (post_id, media_id, position) values ($1, $2, 0)")
        .bind(pic)
        .bind(media)
        .execute(db)
        .await
        .expect("post_media");
    let bobs = make_post(db, bob.id, "bob post", None).await;
    make_post(db, ana, "reply", Some(bobs)).await;
    sqlx::query("insert into reposts (id, user_id, post_id) values ($1, $2, $3)")
        .bind(Uuid::now_v7())
        .bind(ana)
        .bind(bobs)
        .execute(db)
        .await
        .expect("repost");
    sqlx::query("update users set pinned_post_id = $2 where id = $1")
        .bind(ana)
        .bind(one)
        .execute(db)
        .await
        .expect("pin");
    sqlx::query("insert into likes (user_id, post_id) values ($1, $2)")
        .bind(ana)
        .bind(bobs)
        .execute(db)
        .await
        .expect("like");
    sqlx::query("insert into likes (user_id, post_id) values ($1, $2)")
        .bind(ana)
        .bind(pic)
        .execute(db)
        .await
        .expect("like");
    (Profile { ana, bob: bob.id }, bob)
}

#[sqlx::test(migrations = "./migrations")]
async fn posts_tab_lists_pinned_first_then_posts_and_reposts(db: PgPool) {
    let app = app(db.clone());
    let (_, mut bob) = profile_fixture(&db, &app).await;

    let page = bob.browser.get("/users/ana/posts").await.ok();
    assert_eq!(texts(&page), ["one", "bob post", "pic", "two"]);
    assert_eq!(page["next"], Value::Null);
    let items = page["items"].as_array().expect("items");
    assert_eq!(items[0]["pinned"], true);
    assert_eq!(items[3]["pinned"], false);
    assert_eq!(items[1]["repostedBy"]["handle"], "ana");
    assert_ne!(items[1]["entryId"], items[1]["id"]);
    assert_eq!(items[2]["entryId"], items[2]["id"]);
    assert_eq!(items[1]["author"]["handle"], "bob");
    assert_eq!(items[2]["repostedBy"], Value::Null);

    let first = bob.browser.get("/users/ana/posts?limit=2").await.ok();
    assert_eq!(texts(&first), ["one", "bob post", "pic"]);
    let next = first["next"].as_str().expect("next").to_owned();
    let second = bob
        .browser
        .get(&format!("/users/ana/posts?limit=2&before={next}"))
        .await
        .ok();
    assert_eq!(texts(&second), ["two"]);
    assert_eq!(second["next"], Value::Null);
}

async fn attach_media(db: &PgPool, owner: Uuid, post: Uuid, kind: &str) {
    let media = Uuid::now_v7();
    sqlx::query(
        "insert into media (id, owner_id, kind, mime, path, bytes)
         values ($1, $2, $3, 'application/octet-stream', 'x', 1)",
    )
    .bind(media)
    .bind(owner)
    .bind(kind)
    .execute(db)
    .await
    .expect("media");
    sqlx::query("insert into post_media (post_id, media_id, position) values ($1, $2, 0)")
        .bind(post)
        .bind(media)
        .execute(db)
        .await
        .expect("post_media");
}

#[sqlx::test(migrations = "./migrations")]
async fn pinned_post_shows_once_first_and_never_on_later_pages(db: PgPool) {
    let app = app(db.clone());
    let mut bob = app.person(&db, "bob").await;
    let ana = insert_user(&db, "ana").await;
    // The pinned post is the oldest, so an unfixed query would list it again on the last page.
    let pinned = make_post(&db, ana, "pinned", None).await;
    make_post(&db, ana, "a", None).await;
    make_post(&db, ana, "b", None).await;
    sqlx::query("insert into reposts (id, user_id, post_id) values ($1, $2, $3)")
        .bind(Uuid::now_v7())
        .bind(ana)
        .bind(pinned)
        .execute(&db)
        .await
        .expect("repost of the pinned post");
    sqlx::query("update users set pinned_post_id = $2 where id = $1")
        .bind(ana)
        .bind(pinned)
        .execute(&db)
        .await
        .expect("pin");

    let first = bob.browser.get("/users/ana/posts?limit=2").await.ok();
    assert_eq!(texts(&first), ["pinned", "pinned", "b"]);
    let items = first["items"].as_array().expect("items");
    assert_eq!(items[0]["pinned"], true);
    assert_eq!(items[0]["entryId"], items[0]["id"]);
    assert_eq!(items[1]["pinned"], false);
    assert_eq!(items[1]["repostedBy"]["handle"], "ana");
    assert_ne!(items[1]["entryId"], items[1]["id"]);
    let mut entry_ids: Vec<String> = items
        .iter()
        .map(|item| item["entryId"].as_str().expect("entryId").to_owned())
        .collect();

    let next = first["next"].as_str().expect("next").to_owned();
    let second = bob
        .browser
        .get(&format!("/users/ana/posts?limit=2&before={next}"))
        .await
        .ok();
    assert_eq!(texts(&second), ["a"]);
    assert_eq!(second["next"], Value::Null);
    entry_ids.extend(
        second["items"]
            .as_array()
            .expect("items")
            .iter()
            .map(|item| item["entryId"].as_str().expect("entryId").to_owned()),
    );
    let mut unique = entry_ids.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(
        unique.len(),
        entry_ids.len(),
        "duplicate entryId: {entry_ids:?}"
    );
    assert_eq!(entry_ids.len(), 4);

    // Unpaged, the pinned post is also listed once.
    let all = bob.browser.get("/users/ana/posts").await.ok();
    assert_eq!(texts(&all), ["pinned", "pinned", "b", "a"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn media_kind_filters_the_media_tab(db: PgPool) {
    let app = app(db.clone());
    let mut bob = app.person(&db, "bob").await;
    let ana = insert_user(&db, "ana").await;
    let photo = make_post(&db, ana, "photo", None).await;
    attach_media(&db, ana, photo, "image").await;
    let video = make_post(&db, ana, "video", None).await;
    attach_media(&db, ana, video, "video").await;
    make_post(&db, ana, "plain", None).await;

    let both = bob.browser.get("/users/ana/posts?tab=media").await.ok();
    assert_eq!(texts(&both), ["video", "photo"]);
    let photos = bob
        .browser
        .get("/users/ana/posts?tab=media&kind=photo")
        .await
        .ok();
    assert_eq!(texts(&photos), ["photo"]);
    let videos = bob
        .browser
        .get("/users/ana/posts?tab=media&kind=video")
        .await
        .ok();
    assert_eq!(texts(&videos), ["video"]);
    bob.browser
        .get("/users/ana/posts?tab=media&kind=x")
        .await
        .err(StatusCode::BAD_REQUEST, "Unknown kind.");

    // Other tabs ignore a valid kind.
    let posts = bob.browser.get("/users/ana/posts?kind=photo").await.ok();
    assert_eq!(texts(&posts), ["plain", "video", "photo"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn replies_media_and_likes_tabs(db: PgPool) {
    let app = app(db.clone());
    let (_, mut bob) = profile_fixture(&db, &app).await;

    let replies = bob.browser.get("/users/ana/posts?tab=replies").await.ok();
    assert_eq!(texts(&replies), ["reply"]);
    assert_eq!(replies["items"][0]["replyTo"]["handle"], "bob");
    let media = bob.browser.get("/users/ana/posts?tab=media").await.ok();
    assert_eq!(texts(&media), ["pic"]);

    let likes = bob.browser.get("/users/ana/posts?tab=likes").await.ok();
    assert_eq!(texts(&likes), ["pic", "bob post"]);
    let first = bob
        .browser
        .get("/users/ana/posts?tab=likes&limit=1")
        .await
        .ok();
    assert_eq!(texts(&first), ["pic"]);
    let next = first["next"].as_str().expect("next").to_owned();
    let second = bob
        .browser
        .get(&format!("/users/ana/posts?tab=likes&limit=1&before={next}"))
        .await
        .ok();
    assert_eq!(texts(&second), ["bob post"]);
    assert_eq!(second["next"], Value::Null);

    bob.browser
        .get("/users/ana/posts?tab=nope")
        .await
        .err(StatusCode::BAD_REQUEST, "Unknown tab.");
}

#[sqlx::test(migrations = "./migrations")]
async fn profile_posts_respect_blocks_mutes_and_deleted_posts(db: PgPool) {
    let app = app(db.clone());
    let (fx, mut bob) = profile_fixture(&db, &app).await;
    let mut carol = app.person(&db, "carol").await;

    // A muted profile still shows its own posts.
    edge(&db, "mutes", bob.id, fx.ana).await;
    let page = bob.browser.get("/users/ana/posts?tab=replies").await.ok();
    assert_eq!(texts(&page), ["reply"]);

    // Carol muted bob: the repost of bob's post drops out, the page keeps its shape.
    edge(&db, "mutes", carol.id, fx.bob).await;
    let page = carol.browser.get("/users/ana/posts").await.ok();
    assert_eq!(texts(&page), ["one", "pic", "two"]);

    sqlx::query("update posts set deleted_at = now() where text = 'two'")
        .execute(&db)
        .await
        .expect("delete");
    let page = carol.browser.get("/users/ana/posts").await.ok();
    assert_eq!(texts(&page), ["one", "pic"]);

    // A block either way empties the tab.
    edge(&db, "blocks", fx.ana, carol.id).await;
    let page = carol.browser.get("/users/ana/posts").await.ok();
    assert_eq!(page["items"], serde_json::json!([]));
    assert_eq!(page["next"], Value::Null);
    sqlx::query("delete from blocks")
        .execute(&db)
        .await
        .expect("unblock");
    edge(&db, "blocks", carol.id, fx.ana).await;
    let page = carol.browser.get("/users/ana/posts").await.ok();
    assert_eq!(page["items"], serde_json::json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn followers_and_following_paginate_newest_first(db: PgPool) {
    let app = app(db.clone());
    let mut viewer = app.person(&db, "viewer").await;
    let ana = insert_user(&db, "ana").await;
    for handle in ["b", "c", "d", "e"] {
        let id = insert_user(&db, handle).await;
        follows(&db, id, ana).await;
    }
    let hidden = insert_user(&db, "hidden").await;
    follows(&db, hidden, ana).await;
    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(hidden)
        .execute(&db)
        .await
        .expect("suspend");
    let x = insert_user(&db, "x").await;
    follows(&db, ana, x).await;
    let y = insert_user(&db, "y").await;
    follows(&db, ana, y).await;

    let first = viewer
        .browser
        .get("/users/ana/followers?limit=2")
        .await
        .ok();
    assert_eq!(handles(&first["items"]), ["e", "d"]);
    let next = first["next"].as_str().expect("next").to_owned();
    let second = viewer
        .browser
        .get(&format!("/users/ana/followers?limit=2&before={next}"))
        .await
        .ok();
    assert_eq!(handles(&second["items"]), ["c", "b"]);
    assert_eq!(second["next"], Value::Null);

    let following = viewer.browser.get("/users/ana/following").await.ok();
    assert_eq!(handles(&following["items"]), ["y", "x"]);
    assert_eq!(following["items"][0]["relation"]["following"], false);

    viewer
        .browser
        .get("/users/ana/followers?before=garbage")
        .await
        .err(StatusCode::BAD_REQUEST, "Invalid cursor.");
}

#[sqlx::test(migrations = "./migrations")]
async fn followed_by_known_lists_followers_the_viewer_follows(db: PgPool) {
    let app = app(db.clone());
    let mut vic = app.person(&db, "vic").await;
    let ana = insert_user(&db, "ana").await;
    let f1 = insert_user(&db, "f1").await;
    let f2 = insert_user(&db, "f2").await;
    let f3 = insert_user(&db, "f3").await;
    let gone = insert_user(&db, "gone").await;
    for id in [f1, f2, f3, gone] {
        follows(&db, id, ana).await;
    }
    for id in [f1, f2, gone] {
        follows(&db, vic.id, id).await;
    }
    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(gone)
        .execute(&db)
        .await
        .expect("suspend");

    let all = vic.browser.get("/users/ana/followed_by_known").await.ok();
    assert_eq!(all["count"], 2);
    assert_eq!(handles(&all["users"]), ["f2", "f1"]);
    let one = vic
        .browser
        .get("/users/ana/followed_by_known?limit=1")
        .await
        .ok();
    assert_eq!(one["count"], 2);
    assert_eq!(handles(&one["users"]), ["f2"]);
    let own = vic.browser.get("/users/vic/followed_by_known").await.ok();
    assert_eq!(own["count"], 0);
    assert_eq!(own["users"], serde_json::json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn blocks_and_mutes_list_newest_first_with_cursor(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut ids = Vec::new();
    for handle in ["b", "c", "d"] {
        ids.push(insert_user(&db, handle).await);
    }
    let suspended = insert_user(&db, "sus").await;
    let deleted = insert_user(&db, "del").await;
    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(suspended)
        .execute(&db)
        .await
        .expect("suspend");
    sqlx::query("update users set deleted_at = now() where id = $1")
        .bind(deleted)
        .execute(&db)
        .await
        .expect("delete");
    for table in ["blocks", "mutes"] {
        edge(&db, table, ana.id, ids[0]).await;
        edge(&db, table, ana.id, deleted).await;
        edge(&db, table, ana.id, suspended).await;
        edge(&db, table, ana.id, ids[1]).await;
        edge(&db, table, ana.id, ids[2]).await;
    }
    for path in ["/me/blocks", "/me/mutes"] {
        let all = ana.browser.get(path).await.ok();
        assert_eq!(handles(&all["items"]), ["d", "c", "sus", "b"], "{path}");
        let first = ana.browser.get(&format!("{path}?limit=2")).await.ok();
        assert_eq!(handles(&first["items"]), ["d", "c"]);
        let next = first["next"].as_str().expect("next").to_owned();
        let second = ana
            .browser
            .get(&format!("{path}?limit=2&before={next}"))
            .await
            .ok();
        assert_eq!(handles(&second["items"]), ["sus", "b"]);
        assert_eq!(second["next"], Value::Null);
    }
    let card = ana.browser.get("/me/blocks").await.ok();
    assert_eq!(card["items"][0]["relation"]["blocked"], true);
}

#[sqlx::test(migrations = "./migrations")]
async fn suggestions_rank_and_exclude(db: PgPool) {
    let app = app(db.clone());
    let mut me = app.person(&db, "me").await;
    let mut ids = std::collections::HashMap::new();
    for handle in [
        "a", "b", "c", "followed", "blocked", "blocker", "muted", "private", "sus",
    ] {
        ids.insert(handle, insert_user(&db, handle).await);
    }
    sqlx::query("update users set verified = true where id = $1")
        .bind(ids["a"])
        .execute(&db)
        .await
        .expect("verify");
    for (handle, count) in [("b", 5), ("c", 9)] {
        sqlx::query("update users set followers_count = $2 where id = $1")
            .bind(ids[handle])
            .bind(count)
            .execute(&db)
            .await
            .expect("count");
    }
    follows(&db, me.id, ids["followed"]).await;
    edge(&db, "blocks", me.id, ids["blocked"]).await;
    edge(&db, "blocks", ids["blocker"], me.id).await;
    edge(&db, "mutes", me.id, ids["muted"]).await;
    sqlx::query("update users set discoverable = false where id = $1")
        .bind(ids["private"])
        .execute(&db)
        .await
        .expect("private");
    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(ids["sus"])
        .execute(&db)
        .await
        .expect("suspend");

    let default = me.browser.get("/suggestions").await.ok();
    assert_eq!(handles(&default), ["a", "c", "b"]);
    let two = me.browser.get("/suggestions?limit=2").await.ok();
    assert_eq!(handles(&two), ["a", "c"]);
    assert_eq!(two[0]["relation"]["following"], false);
    let many = me.browser.get("/suggestions?limit=500").await.ok();
    assert_eq!(handles(&many), ["a", "c", "b"]);
}
