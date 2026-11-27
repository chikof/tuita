//! People and post search, hashtags, trends and tag pages.
#![allow(missing_docs)]

mod common;

use serde_json::Value;
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use common::{app, insert_user};

/// 2020-01-15 12:00:00 UTC.
const JAN_15_2020: i64 = 1_579_089_600;

async fn make_post(
    db: &PgPool,
    author: Uuid,
    text: &str,
    reply_to: Option<Uuid>,
    at: Option<OffsetDateTime>,
) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "insert into posts (id, author_id, text, reply_to_id, created_at)
         values ($1, $2, $3, $4, coalesce($5, now()))",
    )
    .bind(id)
    .bind(author)
    .bind(text)
    .bind(reply_to)
    .bind(at)
    .execute(db)
    .await
    .expect("insert post");
    id
}

async fn tag(db: &PgPool, post: Uuid, tag: &str) {
    sqlx::query("insert into post_tags (post_id, tag) values ($1, $2)")
        .bind(post)
        .bind(tag)
        .execute(db)
        .await
        .expect("insert tag");
}

async fn attach(db: &PgPool, owner: Uuid, post: Uuid, kind: &str) {
    let media = Uuid::now_v7();
    sqlx::query(
        "insert into media (id, owner_id, kind, mime, path, bytes) values ($1, $2, $3, 'x/y', 'p', 1)",
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

fn handles(users: &Value) -> Vec<&str> {
    users
        .as_array()
        .expect("array")
        .iter()
        .map(|user| user["handle"].as_str().expect("handle"))
        .collect()
}

fn texts(posts: &Value) -> Vec<&str> {
    posts
        .as_array()
        .expect("array")
        .iter()
        .map(|post| post["text"].as_str().expect("text"))
        .collect()
}

fn tags(trends: &Value) -> Vec<(&str, i64)> {
    trends
        .as_array()
        .expect("array")
        .iter()
        .map(|t| {
            (
                t["tag"].as_str().expect("tag"),
                t["posts"].as_i64().expect("posts"),
            )
        })
        .collect()
}

#[sqlx::test(migrations = "./migrations")]
async fn search_needs_a_session_and_empty_query_is_empty(db: PgPool) {
    let app = app(db.clone());
    let mut anon = app.browser();
    anon.get("/search?q=a")
        .await
        .err(axum::http::StatusCode::UNAUTHORIZED, "sign in required");
    anon.get("/trends").await.ok();
    let mut ana = app.person(&db, "ana").await;
    for path in ["/search", "/search?q=", "/search?q=%20%20&type=people"] {
        let result = ana.browser.get(path).await.ok();
        assert_eq!(
            result,
            serde_json::json!({"users": [], "posts": [], "next": null}),
            "{path}"
        );
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn people_search_ranks_filters_and_paginates(db: PgPool) {
    let app = app(db.clone());
    let mut me = app.person(&db, "me").await;
    insert_user(&db, "ann").await;
    let anna = insert_user(&db, "anna").await;
    let joanne = insert_user(&db, "joanne").await;
    let biog = insert_user(&db, "zed").await;
    let hidden = insert_user(&db, "annhidden").await;
    let blocked = insert_user(&db, "annblocked").await;
    let sus = insert_user(&db, "annsus").await;
    let quiet = insert_user(&db, "annquiet").await;
    sqlx::query("update users set display_name = 'Jo-Anne' where id = $1")
        .bind(joanne)
        .execute(&db)
        .await
        .expect("name");
    sqlx::query("update users set bio = 'I like ANN a lot' where id = $1")
        .bind(biog)
        .execute(&db)
        .await
        .expect("bio");
    sqlx::query("update users set verified = true where id = $1")
        .bind(anna)
        .execute(&db)
        .await
        .expect("verify");
    sqlx::query("update users set followers_count = 3 where id = $1")
        .bind(joanne)
        .execute(&db)
        .await
        .expect("followers");
    sqlx::query("update users set discoverable = false where id = any($1)")
        .bind([hidden, quiet])
        .execute(&db)
        .await
        .expect("private");
    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(sus)
        .execute(&db)
        .await
        .expect("suspend");
    sqlx::query("insert into blocks (user_id, target_id) values ($1, $2)")
        .bind(me.id)
        .bind(blocked)
        .execute(&db)
        .await
        .expect("block");
    let result = me.browser.get("/search?q=%40ann&type=people").await.ok();
    assert_eq!(handles(&result["users"]), ["ann", "anna", "joanne", "zed"]);
    assert_eq!(result["posts"], serde_json::json!([]));
    assert_eq!(result["next"], Value::Null);

    // Exact handle of an undiscoverable account still finds it.
    let exact = me.browser.get("/search?q=annquiet&type=people").await.ok();
    assert_eq!(handles(&exact["users"]), ["annquiet"]);

    let first = me
        .browser
        .get("/search?q=ann&type=people&limit=2")
        .await
        .ok();
    assert_eq!(handles(&first["users"]), ["ann", "anna"]);
    assert_eq!(first["next"], "2");
    let second = me
        .browser
        .get("/search?q=ann&type=people&limit=2&before=2")
        .await
        .ok();
    assert_eq!(handles(&second["users"]), ["joanne", "zed"]);
    assert_eq!(second["next"], Value::Null);
    me.browser
        .get("/search?q=ann&type=people&before=x")
        .await
        .err(axum::http::StatusCode::BAD_REQUEST, "Invalid cursor.");
}

#[sqlx::test(migrations = "./migrations")]
async fn people_search_matches_wildcards_literally(db: PgPool) {
    let app = app(db.clone());
    let mut me = app.person(&db, "me").await;
    insert_user(&db, "a_b").await;
    insert_user(&db, "axb").await;
    let result = me.browser.get("/search?q=a_b&type=people").await.ok();
    assert_eq!(handles(&result["users"]), ["a_b"]);
    let result = me.browser.get("/search?q=%25&type=people").await.ok();
    assert_eq!(result["users"], serde_json::json!([]));
}

struct Fixture {
    ana: Uuid,
    bob: Uuid,
    carol: Uuid,
}

/// Seven posts, oldest first: p1 .. p7.
async fn post_fixture(db: &PgPool) -> Fixture {
    let ana = insert_user(db, "ana").await;
    let bob = insert_user(db, "bob").await;
    let carol = insert_user(db, "carol").await;
    make_post(db, ana, "cats are great https://x.example", None, None).await;
    let p2 = make_post(db, bob, "dogs are great", None, None).await;
    make_post(db, ana, "cats and dogs", Some(p2), None).await;
    make_post(db, carol, "hello @bob cats", None, None).await;
    let old = OffsetDateTime::from_unix_timestamp(JAN_15_2020).expect("time");
    make_post(db, ana, "old cats", None, Some(old)).await;
    let p6 = make_post(db, ana, "media cats", None, None).await;
    attach(db, ana, p6, "image").await;
    let p7 = make_post(db, bob, "clip", None, None).await;
    attach(db, bob, p7, "video").await;
    Fixture { ana, bob, carol }
}

#[sqlx::test(migrations = "./migrations")]
async fn post_search_terms_phrases_and_exclusions(db: PgPool) {
    let app = app(db.clone());
    let mut viewer = app.person(&db, "viewer").await;
    post_fixture(&db).await;

    let r = viewer.browser.get("/search?q=CATS").await.ok();
    assert_eq!(
        texts(&r["posts"]),
        [
            "media cats",
            "old cats",
            "hello @bob cats",
            "cats and dogs",
            "cats are great https://x.example"
        ]
    );
    assert_eq!(r["next"], Value::Null);

    let r = viewer.browser.get("/search?q=cats+-dogs").await.ok();
    assert_eq!(
        texts(&r["posts"]),
        [
            "media cats",
            "old cats",
            "hello @bob cats",
            "cats are great https://x.example"
        ]
    );
    let r = viewer.browser.get("/search?q=%22cats%20are%22").await.ok();
    assert_eq!(texts(&r["posts"]), ["cats are great https://x.example"]);
    let r = viewer.browser.get("/search?q=cats+dogs").await.ok();
    assert_eq!(texts(&r["posts"]), ["cats and dogs"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn post_search_operators(db: PgPool) {
    let app = app(db.clone());
    let mut viewer = app.person(&db, "viewer").await;
    post_fixture(&db).await;
    let mut ask = async |q: &str| {
        let r = viewer.browser.get(&format!("/search?q={q}")).await.ok();
        assert_eq!(r["users"], serde_json::json!([]), "{q}");
        texts(&r["posts"])
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        ask("from:@Ana+cats").await,
        [
            "media cats",
            "old cats",
            "cats and dogs",
            "cats are great https://x.example"
        ]
    );
    assert!(ask("from:ana+from:bob").await.is_empty());
    assert_eq!(ask("to:bob").await, ["hello @bob cats", "cats and dogs"]);
    assert_eq!(ask("has:links").await, ["cats are great https://x.example"]);
    assert!(ask("has:poll").await.is_empty());
    assert_eq!(ask("has:image").await, ["media cats"]);
    assert_eq!(ask("has:video").await, ["clip"]);
    assert_eq!(ask("has:media").await, ["clip", "media cats"]);
    assert_eq!(
        ask("cats+since:2021-01-01").await,
        [
            "media cats",
            "hello @bob cats",
            "cats and dogs",
            "cats are great https://x.example"
        ]
    );
    assert_eq!(ask("cats+until:2020-01-15").await, ["old cats"]);
    assert!(ask("cats+until:2020-01-14").await.is_empty());
    assert_eq!(
        ask("cats+since:2020-01-15+until:2020-01-15").await,
        ["old cats"]
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn media_search_and_pagination(db: PgPool) {
    let app = app(db.clone());
    let mut viewer = app.person(&db, "viewer").await;
    post_fixture(&db).await;

    let r = viewer.browser.get("/search?q=cats&type=media").await.ok();
    assert_eq!(texts(&r["posts"]), ["media cats"]);
    assert_eq!(r["users"], serde_json::json!([]));

    let first = viewer.browser.get("/search?q=cats&limit=2").await.ok();
    assert_eq!(texts(&first["posts"]), ["media cats", "old cats"]);
    let c1 = first["next"].as_str().expect("next").to_owned();
    let second = viewer
        .browser
        .get(&format!("/search?q=cats&limit=2&before={c1}"))
        .await
        .ok();
    assert_eq!(
        texts(&second["posts"]),
        ["hello @bob cats", "cats and dogs"]
    );
    let c2 = second["next"].as_str().expect("next").to_owned();
    let third = viewer
        .browser
        .get(&format!("/search?q=cats&limit=2&before={c2}"))
        .await
        .ok();
    assert_eq!(texts(&third["posts"]), ["cats are great https://x.example"]);
    assert_eq!(third["next"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn post_search_hides_blocked_muted_and_deleted(db: PgPool) {
    let app = app(db.clone());
    let mut viewer = app.person(&db, "viewer").await;
    let fx = post_fixture(&db).await;
    for (table, target) in [("blocks", fx.carol), ("mutes", fx.bob)] {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "insert into {table} (user_id, target_id) values ($1, $2)"
        )))
        .bind(viewer.id)
        .bind(target)
        .execute(&db)
        .await
        .expect("edge");
    }
    sqlx::query("update posts set deleted_at = now() where text = 'old cats'")
        .execute(&db)
        .await
        .expect("delete");
    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(fx.ana)
        .execute(&db)
        .await
        .expect("suspend");
    let r = viewer.browser.get("/search?q=cats").await.ok();
    assert!(texts(&r["posts"]).is_empty());
    let r = viewer.browser.get("/search?q=dogs").await.ok();
    assert!(texts(&r["posts"]).is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn posts_search_adds_people_on_the_first_page_only(db: PgPool) {
    let app = app(db.clone());
    let mut viewer = app.person(&db, "viewer").await;
    post_fixture(&db).await;
    let r = viewer.browser.get("/search?q=ana").await.ok();
    assert_eq!(handles(&r["users"]), ["ana"]);
    assert_eq!(r["posts"], serde_json::json!([]));
    let r = viewer.browser.get("/search?q=%40bob").await.ok();
    assert_eq!(handles(&r["users"]), ["bob"]);
    let r = viewer.browser.get("/search?q=ana+-zzz").await.ok();
    assert_eq!(r["users"], serde_json::json!([]));
    let r = viewer.browser.get("/search?q=%23ana").await.ok();
    assert_eq!(r["users"], serde_json::json!([]));
    let cursor = Uuid::now_v7();
    let r = viewer
        .browser
        .get(&format!("/search?q=ana&before={cursor}"))
        .await
        .ok();
    assert_eq!(r["users"], serde_json::json!([]));
    let r = viewer.browser.get("/search?q=ana&type=media").await.ok();
    assert_eq!(r["users"], serde_json::json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn hashtag_suggestions_match_the_prefix(db: PgPool) {
    let app = app(db.clone());
    let mut me = app.person(&db, "me").await;
    let ana = insert_user(&db, "ana").await;
    for _ in 0..3 {
        let p = make_post(&db, ana, "#rust", None, None).await;
        tag(&db, p, "rust").await;
    }
    let p = make_post(&db, ana, "#rustacean", None, None).await;
    tag(&db, p, "rustacean").await;
    for _ in 0..2 {
        let p = make_post(&db, ana, "#ruby", None, None).await;
        tag(&db, p, "ruby").await;
    }
    let gone = make_post(&db, ana, "#rustgone", None, None).await;
    tag(&db, gone, "rustgone").await;
    sqlx::query("update posts set deleted_at = now() where id = $1")
        .bind(gone)
        .execute(&db)
        .await
        .expect("delete");
    let p = make_post(&db, ana, "#go", None, None).await;
    tag(&db, p, "go").await;

    let r = me.browser.get("/hashtags?prefix=ru").await.ok();
    assert_eq!(tags(&r), [("rust", 3), ("ruby", 2), ("rustacean", 1)]);
    let r = me.browser.get("/hashtags?prefix=%23RUS").await.ok();
    assert_eq!(tags(&r), [("rust", 3), ("rustacean", 1)]);
    let r = me.browser.get("/hashtags?prefix=r_").await.ok();
    assert!(tags(&r).is_empty());
    let r = me.browser.get("/hashtags").await.ok();
    assert_eq!(tags(&r).len(), 4);
}

#[sqlx::test(migrations = "./migrations")]
async fn trends_cover_seven_days_and_letter_tags(db: PgPool) {
    let app = app(db.clone());
    let mut me = app.person(&db, "me").await;
    let ana = insert_user(&db, "ana").await;
    let sus = insert_user(&db, "sus").await;
    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(sus)
        .execute(&db)
        .await
        .expect("suspend");
    // Ten tags; tag number i is on 10 - i posts.
    let names = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];
    for last in 0..names.len() {
        let p = make_post(&db, ana, "x", None, None).await;
        for name in &names[..=last] {
            tag(&db, p, name).await;
        }
    }
    for _ in 0..12 {
        let p = make_post(&db, ana, "year", None, None).await;
        tag(&db, p, "2024").await;
    }
    let week_ago = OffsetDateTime::now_utc() - Duration::days(8);
    for _ in 0..20 {
        let p = make_post(&db, ana, "old", None, Some(week_ago)).await;
        tag(&db, p, "stale").await;
    }
    let p = make_post(&db, sus, "x", None, None).await;
    tag(&db, p, "suspended").await;
    let p = make_post(&db, ana, "x", None, None).await;
    tag(&db, p, "removed").await;
    sqlx::query("update posts set deleted_at = now() where id = $1")
        .bind(p)
        .execute(&db)
        .await
        .expect("delete");

    let r = me.browser.get("/trends").await.ok();
    assert_eq!(
        tags(&r),
        [
            ("a", 10),
            ("b", 9),
            ("c", 8),
            ("d", 7),
            ("e", 6),
            ("f", 5),
            ("g", 4),
            ("h", 3)
        ]
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn tag_pages_hide_what_the_viewer_cannot_see(db: PgPool) {
    let app = app(db.clone());
    let mut me = app.person(&db, "me").await;
    let ana = insert_user(&db, "ana").await;
    let blocked = insert_user(&db, "blocked").await;
    let muted = insert_user(&db, "muted").await;
    let sus = insert_user(&db, "sus").await;
    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(sus)
        .execute(&db)
        .await
        .expect("suspend");
    for (table, target) in [("blocks", blocked), ("mutes", muted)] {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "insert into {table} (user_id, target_id) values ($1, $2)"
        )))
        .bind(me.id)
        .bind(target)
        .execute(&db)
        .await
        .expect("edge");
    }
    for (author, text) in [
        (ana, "first"),
        (blocked, "blocked one"),
        (ana, "second"),
        (muted, "muted one"),
        (sus, "suspended one"),
        (ana, "deleted one"),
        (ana, "third"),
    ] {
        let p = make_post(&db, author, text, None, None).await;
        tag(&db, p, "rust").await;
        if text == "deleted one" {
            sqlx::query("update posts set deleted_at = now() where id = $1")
                .bind(p)
                .execute(&db)
                .await
                .expect("delete");
        }
    }
    let other = make_post(&db, ana, "other tag", None, None).await;
    tag(&db, other, "go").await;

    let r = me.browser.get("/tags/Rust/posts").await.ok();
    assert_eq!(texts(&r["items"]), ["third", "second", "first"]);
    assert_eq!(r["next"], Value::Null);
    let first = me.browser.get("/tags/rust/posts?limit=2").await.ok();
    assert_eq!(texts(&first["items"]), ["third", "second"]);
    let next = first["next"].as_str().expect("next").to_owned();
    let second = me
        .browser
        .get(&format!("/tags/rust/posts?limit=2&before={next}"))
        .await
        .ok();
    assert_eq!(texts(&second["items"]), ["first"]);
    assert_eq!(second["next"], Value::Null);
    let none = me.browser.get("/tags/nothing/posts").await.ok();
    assert_eq!(none["items"], serde_json::json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn until_on_the_last_representable_day_does_not_panic(db: PgPool) {
    let app = app(db.clone());
    let mut viewer = app.person(&db, "viewer").await;
    let ana = common::insert_user(&db, "ana").await;
    make_post(&db, ana, "x marks the spot", None, None).await;
    for q in [
        "x%20until:9999-12-31",
        "x%20until:9999-12-30",
        "x%20since:9999-12-31",
    ] {
        let reply = viewer.browser.get(&format!("/search?q={q}")).await;
        assert_eq!(
            reply.status,
            axum::http::StatusCode::OK,
            "{q}: {}",
            reply.body
        );
    }
    let all = viewer
        .browser
        .get("/search?q=x%20until:9999-12-31")
        .await
        .ok();
    assert_eq!(texts(&all["posts"]), ["x marks the spot"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn huge_offsets_are_harmless(db: PgPool) {
    let app = app(db.clone());
    let mut viewer = app.person(&db, "viewer").await;
    for path in [
        "/search?q=v&type=people&before=9223372036854775807",
        "/timelines/foryou?cursor=9223372036854775807",
    ] {
        let reply = viewer.browser.get(path).await;
        assert_eq!(
            reply.status,
            axum::http::StatusCode::OK,
            "{path}: {}",
            reply.body
        );
    }
}
