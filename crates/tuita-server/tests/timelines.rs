//! Timelines, new-post counts and bookmarks, with fixtures inserted through SQL.
#![allow(missing_docs)]

mod common;

use axum::http::StatusCode;
use serde_json::Value;
use sqlx::PgPool;
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
    sqlx::query("update users set posts_count = posts_count + 1 where id = $1")
        .bind(author)
        .execute(db)
        .await
        .expect("bump posts_count");
    id
}

async fn reply(db: &PgPool, author: Uuid, parent: Uuid) -> Uuid {
    let id = post(db, author, "a reply").await;
    sqlx::query("update posts set reply_to_id = $2, root_id = $2 where id = $1")
        .bind(id)
        .bind(parent)
        .execute(db)
        .await
        .expect("make reply");
    sqlx::query("update posts set reply_count = reply_count + 1 where id = $1")
        .bind(parent)
        .execute(db)
        .await
        .expect("bump reply_count");
    id
}

async fn follow(db: &PgPool, follower: Uuid, followee: Uuid) {
    sqlx::query("insert into follows (follower_id, followee_id) values ($1, $2)")
        .bind(follower)
        .bind(followee)
        .execute(db)
        .await
        .expect("insert follow");
    sqlx::query("update users set following_count = following_count + 1 where id = $1")
        .bind(follower)
        .execute(db)
        .await
        .expect("bump following");
    sqlx::query("update users set followers_count = followers_count + 1 where id = $1")
        .bind(followee)
        .execute(db)
        .await
        .expect("bump followers");
}

async fn repost(db: &PgPool, user: Uuid, post: Uuid) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query("insert into reposts (id, user_id, post_id) values ($1, $2, $3)")
        .bind(id)
        .bind(user)
        .bind(post)
        .execute(db)
        .await
        .expect("insert repost");
    sqlx::query("update posts set repost_count = repost_count + 1 where id = $1")
        .bind(post)
        .execute(db)
        .await
        .expect("bump repost_count");
    id
}

async fn block(db: &PgPool, user: Uuid, target: Uuid) {
    sqlx::query("insert into blocks (user_id, target_id) values ($1, $2)")
        .bind(user)
        .bind(target)
        .execute(db)
        .await
        .expect("insert block");
}

fn ids(page: &Value) -> Vec<String> {
    page["items"]
        .as_array()
        .expect("items array")
        .iter()
        .map(|item| item["id"].as_str().expect("id").to_owned())
        .collect()
}

fn texts(page: &Value) -> Vec<String> {
    page["items"]
        .as_array()
        .expect("items array")
        .iter()
        .map(|item| item["text"].as_str().expect("text").to_owned())
        .collect()
}

#[sqlx::test(migrations = "./migrations")]
async fn following_mixes_own_followed_and_reposts(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let ben = common::insert_user(&db, "ben").await;
    let cat = common::insert_user(&db, "cat").await;
    let dan = common::insert_user(&db, "dan").await;
    follow(&db, ana.id, ben).await;
    follow(&db, ana.id, dan).await;

    post(&db, ana.id, "ana one").await;
    post(&db, ben, "ben one").await;
    let cat_one = post(&db, cat, "cat one").await;
    let ben_two = post(&db, ben, "ben two").await;
    reply(&db, ben, ben_two).await;
    repost(&db, ben, cat_one).await;
    let cat_two = post(&db, cat, "cat two").await;
    repost(&db, ana.id, cat_two).await;
    post(&db, dan, "dan hidden").await;
    block(&db, ana.id, dan).await;

    let page = ana.browser.get("/timelines/following").await.ok();
    assert_eq!(
        texts(&page),
        ["cat one", "ben two", "ben one", "ana one"],
        "{page}"
    );
    assert_eq!(page["next"], Value::Null);
    assert_eq!(page["items"][0]["repostedBy"]["handle"], "ben");
    assert_ne!(page["items"][0]["entryId"], page["items"][0]["id"]);
    assert_eq!(page["items"][1]["entryId"], page["items"][1]["id"]);
    assert_eq!(page["items"][1]["repostedBy"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn following_drops_reposts_of_hidden_authors_and_deleted_posts(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let ben = common::insert_user(&db, "ben").await;
    let cat = common::insert_user(&db, "cat").await;
    follow(&db, ana.id, ben).await;
    let hidden = post(&db, cat, "from a blocked author").await;
    let gone = post(&db, cat, "deleted later").await;
    let shown = post(&db, cat, "visible").await;
    repost(&db, ben, hidden).await;
    repost(&db, ben, gone).await;
    repost(&db, ben, shown).await;
    block(&db, cat, ana.id).await;
    let page = ana.browser.get("/timelines/following").await.ok();
    assert!(ids(&page).is_empty(), "{page}");

    sqlx::query("delete from blocks")
        .execute(&db)
        .await
        .expect("unblock");
    sqlx::query("update posts set deleted_at = now() where id = $1")
        .bind(gone)
        .execute(&db)
        .await
        .expect("delete post");
    let page = ana.browser.get("/timelines/following").await.ok();
    assert_eq!(texts(&page), ["visible", "from a blocked author"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn following_pages_with_a_cursor(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let ben = common::insert_user(&db, "ben").await;
    let cat = common::insert_user(&db, "cat").await;
    follow(&db, ana.id, ben).await;
    follow(&db, ana.id, cat).await;
    post(&db, ben, "p1").await;
    let p2 = post(&db, cat, "p2").await;
    post(&db, ben, "p3").await;
    repost(&db, ben, p2).await;
    post(&db, ana.id, "p5").await;

    let first = ana.browser.get("/timelines/following?limit=2").await.ok();
    assert_eq!(texts(&first), ["p5", "p2"]);
    let cursor = first["next"].as_str().expect("next").to_owned();
    let second = ana
        .browser
        .get(&format!("/timelines/following?limit=2&before={cursor}"))
        .await
        .ok();
    assert_eq!(texts(&second), ["p3", "p2"]);
    assert_eq!(second["items"][1]["repostedBy"], Value::Null);
    let cursor = second["next"].as_str().expect("next").to_owned();
    let third = ana
        .browser
        .get(&format!("/timelines/following?limit=2&before={cursor}"))
        .await
        .ok();
    assert_eq!(texts(&third), ["p1"]);
    assert_eq!(third["next"], Value::Null);

    ana.browser
        .get("/timelines/following?before=nope")
        .await
        .err(StatusCode::BAD_REQUEST, "Invalid cursor.");
    ana.browser
        .get("/timelines/following?limit=0")
        .await
        .err(StatusCode::BAD_REQUEST, "Invalid limit.");
}

#[sqlx::test(migrations = "./migrations")]
async fn timelines_need_a_session(db: PgPool) {
    let app = common::app(db);
    let mut nobody = app.browser();
    nobody
        .get("/timelines/following")
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");
}

#[sqlx::test(migrations = "./migrations")]
async fn foryou_ranks_and_hides(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let ben = common::insert_user(&db, "ben").await;
    let dan = common::insert_user(&db, "dan").await;
    follow(&db, ana.id, ben).await;
    post(&db, ben, "followed").await;
    post(&db, dan, "blocked").await;
    let root = post(&db, ben, "root").await;
    reply(&db, ben, root).await;
    block(&db, ana.id, dan).await;

    let page = ana.browser.get("/timelines/foryou").await.ok();
    let mut got = texts(&page);
    got.sort();
    assert_eq!(got, ["followed", "root"]);
    assert_eq!(page["next"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn foryou_reuses_its_snapshot_until_it_expires(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let ben = common::insert_user(&db, "ben").await;
    let cat = common::insert_user(&db, "cat").await;
    let mut old = Vec::new();
    for n in 0..5 {
        let author = if n % 2 == 0 { ben } else { cat };
        old.push(post(&db, author, &format!("old {n}")).await.to_string());
    }

    let first = ana.browser.get("/timelines/foryou?limit=2").await.ok();
    assert_eq!(first["items"].as_array().map(Vec::len), Some(2));
    assert_eq!(first["next"], "2");
    let fresh = post(&db, ben, "fresh").await.to_string();

    let second = ana
        .browser
        .get("/timelines/foryou?limit=2&cursor=2")
        .await
        .ok();
    assert_eq!(second["next"], "4");
    let third = ana
        .browser
        .get("/timelines/foryou?limit=2&cursor=4")
        .await
        .ok();
    assert_eq!(third["next"], Value::Null);

    let mut seen: Vec<String> = [&first, &second, &third]
        .into_iter()
        .flat_map(ids)
        .collect();
    assert_eq!(seen.len(), 5, "no duplicates and no gaps");
    seen.sort();
    old.sort();
    assert_eq!(
        seen, old,
        "the post written after page one stays out of its snapshot"
    );
    assert!(!seen.contains(&fresh));

    let rebuilt = ana.browser.get("/timelines/foryou?limit=40").await.ok();
    assert!(ids(&rebuilt).contains(&fresh), "no cursor ranks afresh");
    assert_eq!(ids(&rebuilt).len(), 6);

    sqlx::query("update feed_snapshots set ids = '{}'")
        .execute(&db)
        .await
        .expect("empty snapshot");
    let reused = ana
        .browser
        .get("/timelines/foryou?limit=2&cursor=2")
        .await
        .ok();
    assert!(
        ids(&reused).is_empty(),
        "a fresh snapshot is used as stored"
    );

    sqlx::query("update feed_snapshots set created_at = now() - interval '21 minutes'")
        .execute(&db)
        .await
        .expect("age snapshot");
    let again = ana
        .browser
        .get("/timelines/foryou?limit=2&cursor=2")
        .await
        .ok();
    assert_eq!(ids(&again).len(), 2, "a stale snapshot is rebuilt");

    ana.browser
        .get("/timelines/foryou?cursor=x")
        .await
        .err(StatusCode::BAD_REQUEST, "Invalid cursor.");
}

#[sqlx::test(migrations = "./migrations")]
async fn new_counts_follow_the_feed_rules(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let ben = common::insert_user(&db, "ben").await;
    let cat = common::insert_user(&db, "cat").await;
    let dan = common::insert_user(&db, "dan").await;
    follow(&db, ana.id, ben).await;

    let marker = post(&db, cat, "before the marker").await;
    post(&db, ana.id, "mine").await;
    let ben_one = post(&db, ben, "ben one").await;
    post(&db, ben, "ben two").await;
    reply(&db, ben, ben_one).await;
    let cat_two = post(&db, cat, "cat two").await;
    repost(&db, ben, cat_two).await;
    post(&db, dan, "dan").await;
    block(&db, ana.id, dan).await;

    let count = |body: Value| body["count"].as_i64().expect("count");
    assert_eq!(
        count(ana.browser.get("/timelines/following/new").await.ok()),
        3
    );
    assert_eq!(
        count(ana.browser.get("/timelines/foryou/new").await.ok()),
        4
    );
    let since = format!("/timelines/foryou/new?since={marker}");
    assert_eq!(count(ana.browser.get(&since).await.ok()), 3);
    let since = format!("/timelines/following/new?since={ben_one}");
    assert_eq!(count(ana.browser.get(&since).await.ok()), 2);

    ana.browser
        .get("/timelines/bogus/new")
        .await
        .err(StatusCode::BAD_REQUEST, "Unknown timeline.");
    ana.browser
        .get("/timelines/foryou/new?since=zzz")
        .await
        .err(StatusCode::BAD_REQUEST, "Invalid since.");
}

#[sqlx::test(migrations = "./migrations")]
async fn new_counts_stop_at_200(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let ben = common::insert_user(&db, "ben").await;
    follow(&db, ana.id, ben).await;
    sqlx::query("insert into posts (id, author_id) select gen_random_uuid(), $1 from generate_series(1, 205)")
        .bind(ben)
        .execute(&db)
        .await
        .expect("bulk posts");
    for feed in ["following", "foryou"] {
        let body = ana
            .browser
            .get(&format!("/timelines/{feed}/new"))
            .await
            .ok();
        assert_eq!(body["count"], 200, "{feed}");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn bookmarks_list_newest_bookmark_first(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let ben = common::insert_user(&db, "ben").await;
    let dan = common::insert_user(&db, "dan").await;
    let mut marked = Vec::new();
    for text in ["one", "two", "three"] {
        marked.push(post(&db, ben, text).await);
    }
    let deleted = post(&db, ben, "deleted").await;
    let blocked = post(&db, dan, "blocked").await;
    // Bookmarks inserted in reverse post order: the list follows the bookmark time.
    for (n, id) in [marked[2], deleted, marked[0], blocked, marked[1]]
        .into_iter()
        .enumerate()
    {
        sqlx::query(
            "insert into bookmarks (user_id, post_id, created_at)
             values ($1, $2, now() - make_interval(mins => 10 - $3))",
        )
        .bind(ana.id)
        .bind(id)
        .bind(i32::try_from(n).expect("small"))
        .execute(&db)
        .await
        .expect("insert bookmark");
    }
    sqlx::query("update posts set deleted_at = now() where id = $1")
        .bind(deleted)
        .execute(&db)
        .await
        .expect("delete");
    block(&db, ana.id, dan).await;

    let first = ana.browser.get("/bookmarks?limit=2").await.ok();
    assert_eq!(texts(&first), ["two"]);
    let cursor = first["next"].as_str().expect("next").to_owned();
    let second = ana
        .browser
        .get(&format!("/bookmarks?limit=2&before={cursor}"))
        .await
        .ok();
    assert_eq!(texts(&second), ["one", "three"]);
    assert_eq!(second["next"], Value::Null);
}
