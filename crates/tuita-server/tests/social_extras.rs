//! Lists, reports, the data export, account deletion and the RSS feed, through the public API.
#![allow(missing_docs)]

mod common;

use axum::body::{Body, to_bytes};
use axum::http::{HeaderMap, Method, Request, StatusCode};
use common::{App, Browser, PASSWORD};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

async fn count(db: &PgPool, sql: &'static str, id: Uuid) -> i64 {
    sqlx::query_scalar(sql)
        .bind(id)
        .fetch_one(db)
        .await
        .expect("count")
}

async fn insert_post(db: &PgPool, author: Uuid, text: &str, reply_to: Option<Uuid>) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query("insert into posts (id, author_id, text, reply_to_id) values ($1, $2, $3, $4)")
        .bind(id)
        .bind(author)
        .bind(text)
        .bind(reply_to)
        .execute(db)
        .await
        .expect("insert post");
    id
}

fn uuid_of(value: &Value) -> Uuid {
    value["id"]
        .as_str()
        .and_then(|id| id.parse().ok())
        .expect("id")
}

fn post_texts(page: &Value) -> Vec<&str> {
    page["items"]
        .as_array()
        .expect("items")
        .iter()
        .map(|post| post["text"].as_str().expect("text"))
        .collect()
}

// ------------------------------------------------------------------ lists

#[sqlx::test(migrations = "./migrations")]
async fn lists_are_created_changed_and_deleted_by_their_owner(db: PgPool) {
    common::block_words(&db, "gizmo").await;
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let rejected = |message: &str| (StatusCode::BAD_REQUEST, message.to_owned());

    for (body, (status, message)) in [
        (json!({"name": "  "}), rejected("List name is required.")),
        (
            json!({"name": "n".repeat(41)}),
            rejected("List name must be at most 40 characters."),
        ),
        (
            json!({"name": "ok", "description": "d".repeat(161)}),
            rejected("Description must be at most 160 characters."),
        ),
        (
            json!({"name": "g1zm0"}),
            rejected("That list name or description isn\u{2019}t allowed."),
        ),
        (
            json!({"name": "ok", "description": "g1zm0"}),
            rejected("That list name or description isn\u{2019}t allowed."),
        ),
    ] {
        ana.browser.post("/lists", body).await.err(status, &message);
    }

    let list = ana
        .browser
        .post(
            "/lists",
            json!({"name": " Friends ", "description": "close ones"}),
        )
        .await
        .ok();
    let id = list["id"].as_str().expect("id").to_owned();
    assert_eq!(list["name"], "Friends");
    assert_eq!(list["description"], "close ones");
    assert_eq!(list["private"], false);
    assert_eq!(list["isOwner"], true);
    assert_eq!(list["members"], 0);
    assert_eq!(list["hasMember"], Value::Null);
    assert_eq!(list["owner"]["handle"], "ana");

    let seen = bob.browser.get(&format!("/lists/{id}")).await.ok();
    assert_eq!(seen["isOwner"], false);
    assert_eq!(bob.browser.get("/lists").await.ok(), json!([]));
    let mine = ana.browser.get("/lists").await.ok();
    assert_eq!(mine.as_array().expect("lists").len(), 1);

    let owner_only = "Only the list owner can change it.";
    bob.browser
        .patch(&format!("/lists/{id}"), json!({"name": "mine now"}))
        .await
        .err(StatusCode::FORBIDDEN, owner_only);
    bob.browser
        .delete(&format!("/lists/{id}"))
        .await
        .err(StatusCode::FORBIDDEN, owner_only);
    bob.browser
        .post_empty(&format!("/lists/{id}/members/ana"))
        .await
        .err(StatusCode::FORBIDDEN, owner_only);

    let changed = ana
        .browser
        .patch(
            &format!("/lists/{id}"),
            json!({"name": "Pals", "private": true}),
        )
        .await
        .ok();
    assert_eq!(changed["name"], "Pals");
    assert_eq!(changed["description"], "close ones");
    assert_eq!(changed["private"], true);
    ana.browser
        .patch(&format!("/lists/{id}"), json!({"name": ""}))
        .await
        .err(StatusCode::BAD_REQUEST, "List name is required.");

    for path in [
        format!("/lists/{id}"),
        format!("/lists/{id}/timeline"),
        format!("/lists/{id}/members"),
    ] {
        bob.browser
            .get(&path)
            .await
            .err(StatusCode::NOT_FOUND, "List not found.");
    }
    bob.browser
        .patch(&format!("/lists/{id}"), json!({"name": "x"}))
        .await
        .err(StatusCode::NOT_FOUND, "List not found.");
    ana.browser.get(&format!("/lists/{id}/timeline")).await.ok();

    assert_eq!(
        ana.browser.delete(&format!("/lists/{id}")).await.ok(),
        json!({"ok": true})
    );
    ana.browser
        .get(&format!("/lists/{id}"))
        .await
        .err(StatusCode::NOT_FOUND, "List not found.");
    ana.browser
        .get("/lists/not-a-uuid")
        .await
        .err(StatusCode::NOT_FOUND, "List not found.");
}

#[sqlx::test(migrations = "./migrations")]
async fn list_members_are_added_listed_and_removed(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let bob = app.person(&db, "bob").await;
    let cat = app.person(&db, "cat").await;
    let dan = app.person(&db, "dan").await;
    let first = ana
        .browser
        .post("/lists", json!({"name": "one"}))
        .await
        .ok();
    let second = ana
        .browser
        .post("/lists", json!({"name": "two"}))
        .await
        .ok();
    let (first, second) = (
        first["id"].as_str().expect("id"),
        second["id"].as_str().expect("id"),
    );

    for handle in ["bob", "cat"] {
        ana.browser
            .post_empty(&format!("/lists/{first}/members/{handle}"))
            .await
            .ok();
    }
    // Twice is fine and counts once.
    ana.browser
        .post_empty(&format!("/lists/{first}/members/bob"))
        .await
        .ok();
    assert_eq!(
        ana.browser.get(&format!("/lists/{first}")).await.ok()["members"],
        2
    );

    let lists = ana.browser.get("/lists?member=bob").await.ok();
    let flags: Vec<(&str, &Value)> = lists
        .as_array()
        .expect("lists")
        .iter()
        .map(|list| (list["name"].as_str().expect("name"), &list["hasMember"]))
        .collect();
    assert_eq!(flags, [("two", &json!(false)), ("one", &json!(true))]);
    ana.browser
        .get("/lists?member=nobody")
        .await
        .err(StatusCode::NOT_FOUND, "User not found.");

    let page = ana
        .browser
        .get(&format!("/lists/{first}/members?limit=1"))
        .await
        .ok();
    assert_eq!(page["items"][0]["handle"], "cat");
    let next = page["next"].as_str().expect("cursor");
    let rest = ana
        .browser
        .get(&format!("/lists/{first}/members?limit=1&before={next}"))
        .await
        .ok();
    assert_eq!(rest["items"][0]["handle"], "bob");
    assert_eq!(rest["next"], Value::Null);

    // Blocked either way: refused.
    for (blocker, blocked) in [(ana.id, dan.id), (dan.id, ana.id)] {
        sqlx::query("insert into blocks (user_id, target_id) values ($1, $2)")
            .bind(blocker)
            .bind(blocked)
            .execute(&db)
            .await
            .expect("block");
        ana.browser
            .post_empty(&format!("/lists/{second}/members/dan"))
            .await
            .err(StatusCode::FORBIDDEN, "You can\u{2019}t add this account.");
        sqlx::query("delete from blocks")
            .execute(&db)
            .await
            .expect("unblock");
    }
    ana.browser
        .post_empty(&format!("/lists/{second}/members/nobody"))
        .await
        .err(StatusCode::NOT_FOUND, "User not found.");

    ana.browser
        .delete(&format!("/lists/{first}/members/bob"))
        .await
        .ok();
    ana.browser
        .delete(&format!("/lists/{first}/members/bob"))
        .await
        .ok();
    assert_eq!(
        ana.browser.get(&format!("/lists/{first}")).await.ok()["members"],
        1
    );
    assert_eq!(
        count(
            &db,
            "select count(*) from list_members where user_id = $1",
            cat.id
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &db,
            "select count(*) from list_members where user_id = $1",
            bob.id
        )
        .await,
        0
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_user_owns_at_most_fifty_lists_and_a_list_holds_at_most_500_members(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    sqlx::query(
        "insert into lists (id, owner_id, name)
         select gen_random_uuid(), $1, 'list ' || n from generate_series(1, 50) n",
    )
    .bind(ana.id)
    .execute(&db)
    .await
    .expect("fifty lists");
    ana.browser
        .post("/lists", json!({"name": "one too many"}))
        .await
        .err(StatusCode::BAD_REQUEST, "You can have up to 50 lists.");

    let mut bob = app.person(&db, "bob").await;
    let list = bob
        .browser
        .post("/lists", json!({"name": "crowd"}))
        .await
        .ok();
    let id = uuid_of(&list);
    sqlx::query(
        "insert into users (id, handle, display_name)
         select gen_random_uuid(), 'u' || n, 'u' || n from generate_series(1, 500) n",
    )
    .execute(&db)
    .await
    .expect("users");
    sqlx::query(
        "insert into list_members (list_id, user_id)
         select $1, id from users where handle ~ '^u[0-9]+$' and id <> $2",
    )
    .bind(id)
    .bind(bob.id)
    .execute(&db)
    .await
    .expect("members");
    assert_eq!(
        count(
            &db,
            "select count(*) from list_members where list_id = $1",
            id
        )
        .await,
        500
    );
    common::insert_user(&db, "extra").await;
    bob.browser
        .post_empty(&format!("/lists/{id}/members/extra"))
        .await
        .err(StatusCode::BAD_REQUEST, "Lists can have up to 500 members.");
    // Somebody already in is not an addition.
    bob.browser
        .post_empty(&format!("/lists/{id}/members/u1"))
        .await
        .ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn creating_lists_is_limited_to_thirty_an_hour(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    for n in 0..30 {
        ana.browser
            .post("/lists", json!({"name": format!("l{n}")}))
            .await
            .ok();
    }
    let reply = ana.browser.post("/lists", json!({"name": "l30"})).await;
    assert_eq!(
        reply.status,
        StatusCode::TOO_MANY_REQUESTS,
        "{}",
        reply.body
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn the_list_timeline_shows_members_posts_replies_among_members_and_reposts(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let mut cat = app.person(&db, "cat").await;
    let mut dan = app.person(&db, "dan").await;
    let list = ana
        .browser
        .post("/lists", json!({"name": "crew"}))
        .await
        .ok();
    let id = list["id"].as_str().expect("id").to_owned();
    for handle in ["bob", "cat"] {
        ana.browser
            .post_empty(&format!("/lists/{id}/members/{handle}"))
            .await
            .ok();
    }

    bob.browser.post_text("b1").await;
    let c1 = cat.browser.post_text("c1").await;
    let d1 = dan.browser.post_text("d1").await;
    ana.browser.post_text("a1").await;
    bob.browser
        .post("/posts", json!({"text": "r1", "replyTo": c1["id"]}))
        .await
        .ok();
    bob.browser
        .post("/posts", json!({"text": "r2", "replyTo": d1["id"]}))
        .await
        .ok();
    cat.browser
        .post_empty(&format!("/posts/{}/repost", d1["id"].as_str().expect("id")))
        .await
        .ok();
    bob.browser.post_text("a secret one").await;
    sqlx::query("update users set muted_words = '{secret}' where id = $1")
        .bind(ana.id)
        .execute(&db)
        .await
        .expect("mute word");

    let page = ana.browser.get(&format!("/lists/{id}/timeline")).await.ok();
    assert_eq!(post_texts(&page), ["d1", "r1", "c1", "b1"], "{page}");
    assert_eq!(page["items"][0]["repostedBy"]["handle"], "cat");
    assert_ne!(page["items"][0]["entryId"], d1["id"]);
    assert_eq!(page["items"][1]["entryId"], page["items"][1]["id"]);
    assert_eq!(page["next"], Value::Null);

    let first = ana
        .browser
        .get(&format!("/lists/{id}/timeline?limit=3"))
        .await
        .ok();
    assert_eq!(post_texts(&first), ["d1", "r1", "c1"]);
    let next = first["next"].as_str().expect("cursor");
    let second = ana
        .browser
        .get(&format!("/lists/{id}/timeline?limit=3&before={next}"))
        .await
        .ok();
    assert_eq!(post_texts(&second), ["b1"]);
    assert_eq!(second["next"], Value::Null);

    sqlx::query("insert into mutes (user_id, target_id) values ($1, $2)")
        .bind(ana.id)
        .bind(bob.id)
        .execute(&db)
        .await
        .expect("mute");
    let page = ana.browser.get(&format!("/lists/{id}/timeline")).await.ok();
    assert_eq!(post_texts(&page), ["d1", "c1"]);
}

// ------------------------------------------------------------------ reports

#[sqlx::test(migrations = "./migrations")]
async fn a_report_is_filed_once_while_it_is_open(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let mut moderator = app.person(&db, "mod").await;
    sqlx::query("update users set role = 'moderator' where id = $1")
        .bind(moderator.id)
        .execute(&db)
        .await
        .expect("promote");
    let post = ana.browser.post_text("report me").await;
    let post = post["id"].as_str().expect("id").to_owned();

    let filed = bob
        .browser
        .post(
            "/reports",
            json!({"postId": post, "reason": "spam", "note": " why "}),
        )
        .await
        .ok();
    assert_eq!(filed, json!({"ok": true, "duplicate": false}));
    let again = bob
        .browser
        .post("/reports", json!({"postId": post, "reason": "hate"}))
        .await
        .ok();
    assert_eq!(again, json!({"ok": true, "duplicate": true}));
    let (reason, note): (String, String) =
        sqlx::query_as("select reason, note from reports where reporter_id = $1")
            .bind(bob.id)
            .fetch_one(&db)
            .await
            .expect("one report");
    assert_eq!((reason.as_str(), note.as_str()), ("spam", "why"));

    // Reporting the account itself is a different report.
    let by_handle = bob
        .browser
        .post(
            "/reports",
            json!({"handle": "@ana", "reason": "harassment"}),
        )
        .await
        .ok();
    assert_eq!(by_handle["duplicate"], false);
    assert_eq!(
        count(
            &db,
            "select count(*) from reports where reporter_id = $1",
            bob.id
        )
        .await,
        2
    );

    assert_eq!(moderator.browser.get("/badges").await.ok()["reports"], 2);
    assert_eq!(
        bob.browser.get("/badges").await.ok()["reports"],
        Value::Null
    );
    sqlx::query("update reports set status = 'resolved'")
        .execute(&db)
        .await
        .expect("resolve");
    assert_eq!(moderator.browser.get("/badges").await.ok()["reports"], 0);
    let after = bob
        .browser
        .post("/reports", json!({"postId": post, "reason": "spam"}))
        .await
        .ok();
    assert_eq!(after["duplicate"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn bad_reports_are_refused(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    app.person(&db, "bob").await;
    let own = ana.browser.post_text("mine").await;
    let own = own["id"].as_str().expect("id").to_owned();

    for (body, status, message) in [
        (
            json!({"reason": "spam"}),
            StatusCode::BAD_REQUEST,
            "Nothing to report.",
        ),
        (
            json!({"handle": "ana", "reason": "spam"}),
            StatusCode::BAD_REQUEST,
            "You can\u{2019}t report yourself.",
        ),
        (
            json!({"postId": own, "reason": "spam"}),
            StatusCode::BAD_REQUEST,
            "You can\u{2019}t report yourself.",
        ),
        (
            json!({"handle": "bob", "reason": "spam", "note": "n".repeat(1001)}),
            StatusCode::BAD_REQUEST,
            "Details must be at most 1000 characters.",
        ),
        (
            json!({"handle": "nobody", "reason": "spam"}),
            StatusCode::NOT_FOUND,
            "User not found.",
        ),
        (
            json!({"postId": Uuid::now_v7(), "reason": "spam"}),
            StatusCode::NOT_FOUND,
            "Post not found.",
        ),
    ] {
        ana.browser
            .post("/reports", body)
            .await
            .err(status, message);
    }
    let reply = ana
        .browser
        .post("/reports", json!({"handle": "bob", "reason": "mean"}))
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.body);
}

#[sqlx::test(migrations = "./migrations")]
async fn reporting_is_limited_to_twenty_an_hour(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    app.person(&db, "bob").await;
    for _ in 0..20 {
        ana.browser
            .post("/reports", json!({"handle": "bob", "reason": "spam"}))
            .await
            .ok();
    }
    let reply = ana
        .browser
        .post("/reports", json!({"handle": "bob", "reason": "spam"}))
        .await;
    assert_eq!(
        reply.status,
        StatusCode::TOO_MANY_REQUESTS,
        "{}",
        reply.body
    );
}

// ------------------------------------------------------------------ export

#[sqlx::test(migrations = "./migrations")]
async fn the_export_holds_what_the_account_holds_and_is_limited_to_five_an_hour(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    common::insert_user(&db, "cat").await;
    common::insert_user(&db, "dan").await;

    let mine = ana.browser.post_text("hello export").await;
    let gone = ana.browser.post_text("deleted").await;
    ana.browser
        .delete(&format!("/posts/{}", gone["id"].as_str().expect("id")))
        .await
        .ok();
    let media = Uuid::now_v7();
    sqlx::query(
        "insert into media (id, owner_id, kind, mime, path, bytes) values ($1, $2, 'image', 'image/png', $3, 1)",
    )
    .bind(media)
    .bind(ana.id)
    .bind(format!("{media}.png"))
    .execute(&db)
    .await
    .expect("media");
    sqlx::query("insert into post_media (post_id, media_id, position) values ($1, $2, 0)")
        .bind(uuid_of(&mine))
        .bind(media)
        .execute(&db)
        .await
        .expect("attach");
    let theirs = bob.browser.post_text("liked by ana").await;
    let theirs = theirs["id"].as_str().expect("id").to_owned();
    ana.browser
        .post_empty(&format!("/posts/{theirs}/like"))
        .await
        .ok();
    ana.browser
        .post_empty(&format!("/posts/{theirs}/repost"))
        .await
        .ok();
    ana.browser
        .post_empty(&format!("/posts/{theirs}/bookmark"))
        .await
        .ok();
    ana.browser.post_empty("/users/bob/follow").await.ok();
    bob.browser.post_empty("/users/ana/follow").await.ok();
    ana.browser.post_empty("/users/cat/block").await.ok();
    ana.browser.post_empty("/users/dan/mute").await.ok();
    for text in ["to bob", "deleted message"] {
        let sent = ana
            .browser
            .post("/conversations/bob", json!({"text": text}))
            .await
            .ok();
        if text == "deleted message" {
            ana.browser
                .delete(&format!(
                    "/conversations/bob/messages/{}",
                    sent["id"].as_str().expect("id")
                ))
                .await
                .ok();
        }
    }
    bob.browser
        .post("/conversations/ana", json!({"text": "from bob"}))
        .await
        .ok();

    let reply = ana.browser.get("/me/export").await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    let disposition = reply.header("content-disposition");
    assert!(
        disposition.starts_with("attachment; filename=\"tuita-ana-")
            && disposition.ends_with(".json\""),
        "{disposition}"
    );
    assert_eq!(
        disposition.len(),
        "attachment; filename=\"tuita-ana-2026-11-09.json\"".len()
    );
    let export = reply.json();
    assert_eq!(export["instance"], "social.example");
    assert_eq!(export["profile"]["user"]["handle"], "ana");
    assert!(export["exportedAt"].is_string());
    let posts = export["posts"].as_array().expect("posts");
    assert_eq!(posts.len(), 1, "{export}");
    assert_eq!(posts[0]["text"], "hello export");
    assert_eq!(posts[0]["media"], json!([format!("/media/{media}.png")]));
    for edges in ["likes", "reposts", "bookmarks"] {
        let rows = export[edges].as_array().expect(edges);
        assert_eq!(rows.len(), 1, "{edges}");
        assert_eq!(rows[0]["postId"], theirs.as_str());
    }
    assert_eq!(export["following"], json!(["bob"]));
    assert_eq!(export["followers"], json!(["bob"]));
    assert_eq!(export["blocked"], json!(["cat"]));
    assert_eq!(export["muted"], json!(["dan"]));
    let messages = export["messages"].as_array().expect("messages");
    let summary: Vec<(&str, bool, &str)> = messages
        .iter()
        .map(|m| {
            (
                m["with"].as_str().expect("with"),
                m["sent"].as_bool().expect("sent"),
                m["text"].as_str().expect("text"),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [("bob", true, "to bob"), ("bob", false, "from bob")]
    );

    for _ in 0..4 {
        ana.browser.get("/me/export").await.ok();
    }
    let sixth = ana.browser.get("/me/export").await;
    assert_eq!(
        sixth.status,
        StatusCode::TOO_MANY_REQUESTS,
        "{}",
        sixth.body
    );
}

// ------------------------------------------------------------------ deletion

async fn delete_me(browser: &mut Browser, password: &str) -> common::Reply {
    browser
        .send(Method::DELETE, "/me", Some(json!({ "password": password })))
        .await
}

#[sqlx::test(migrations = "./migrations")]
async fn deletion_needs_the_password_and_never_removes_the_last_admin(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.browser_at([198, 51, 100, 10]);
    let ana_me = ana.signup("ana").await;
    assert_eq!(ana_me["role"], "admin");
    let mut ben = app.browser_at([198, 51, 100, 11]);
    ben.signup("ben").await;

    delete_me(&mut ana, "not the password")
        .await
        .err(StatusCode::FORBIDDEN, "Wrong password.");
    delete_me(&mut ana, PASSWORD).await.err(
        StatusCode::BAD_REQUEST,
        "You\u{2019}re the only admin. Make someone else an admin before deleting your account.",
    );
    assert_eq!(ana.get("/me").await.ok()["user"]["user"]["handle"], "ana");

    // A suspended second admin does not count.
    sqlx::query("update users set role = 'admin', suspended_at = now() where handle = 'ben'")
        .execute(&db)
        .await
        .expect("suspended admin");
    assert_eq!(
        delete_me(&mut ana, PASSWORD).await.status,
        StatusCode::BAD_REQUEST
    );
    sqlx::query("update users set suspended_at = null where handle = 'ben'")
        .execute(&db)
        .await
        .expect("unsuspend");
    delete_me(&mut ana, PASSWORD).await.ok();
    ben.get("/me").await.ok();
    // ben is the only admin left now.
    delete_me(&mut ben, PASSWORD).await.err(
        StatusCode::BAD_REQUEST,
        "You\u{2019}re the only admin. Make someone else an admin before deleting your account.",
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn deleting_an_account_anonymises_it_and_cleans_up_after_it(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.browser_at([198, 51, 100, 20]);
    let ana_me = ana.signup("ana").await;
    let mut bob = app.browser_at([198, 51, 100, 21]);
    let bob_me = bob.signup("bob").await;
    let (ana_id, bob_id) = (uuid_of(&ana_me["user"]), uuid_of(&bob_me["user"]));
    let mut cat = app.person(&db, "cat").await;

    let ana_post = ana.post_text("ana speaks").await;
    let ana_post_id = ana_post["id"].as_str().expect("id").to_owned();
    let bob_post = bob.post_text("bob speaks").await;
    let bob_post_id = bob_post["id"].as_str().expect("id").to_owned();
    bob.post(
        "/posts",
        json!({"text": "bob replies", "replyTo": ana_post_id}),
    )
    .await
    .ok();
    bob.post(
        "/posts",
        json!({"text": "bob quotes", "quoteOf": ana_post_id}),
    )
    .await
    .ok();
    for action in ["like", "repost", "bookmark"] {
        bob.post_empty(&format!("/posts/{ana_post_id}/{action}"))
            .await
            .ok();
    }
    ana.post_empty(&format!("/posts/{bob_post_id}/like"))
        .await
        .ok();
    ana.post_empty("/users/bob/follow").await.ok();
    bob.post_empty("/users/ana/follow").await.ok();
    bob.post("/conversations/ana", json!({"text": "hi ana"}))
        .await
        .ok();
    ana.post("/conversations/bob", json!({"text": "hi bob"}))
        .await
        .ok();
    let own_list = bob.post("/lists", json!({"name": "bobs"})).await.ok();
    bob.post_empty(&format!(
        "/lists/{}/members/cat",
        own_list["id"].as_str().expect("id")
    ))
    .await
    .ok();
    let ana_list = ana.post("/lists", json!({"name": "anas"})).await.ok();
    ana.post_empty(&format!(
        "/lists/{}/members/bob",
        ana_list["id"].as_str().expect("id")
    ))
    .await
    .ok();
    bob.post_empty("/users/cat/block").await.ok();
    bob.post_empty("/users/cat/mute").await.ok();
    sqlx::query(
        "insert into scheduled_posts (id, author_id, payload, publish_at)
         values ($1, $2, '{\"text\": \"later\"}', now() + interval '1 day')",
    )
    .bind(Uuid::now_v7())
    .bind(bob_id)
    .execute(&db)
    .await
    .expect("scheduled");
    cat.browser.post_text("cat is here").await;
    let token = bob
        .cookie("tuita_session")
        .expect("session cookie")
        .to_owned();

    assert_eq!(
        count(
            &db,
            "select like_count::int8 from posts where id = $1",
            Uuid::parse_str(&ana_post_id).expect("uuid")
        )
        .await,
        1
    );

    let reply = delete_me(&mut bob, PASSWORD).await;
    assert_eq!(reply.ok(), json!({"ok": true}));
    assert!(
        reply
            .set_cookies()
            .iter()
            .any(|(name, value)| name == "tuita_session" && value.is_empty()),
        "{:?}",
        reply.set_cookies()
    );

    let mut stale = app.browser_at([198, 51, 100, 22]);
    stale.set_cookie("tuita_session", &token);
    stale
        .get("/conversations")
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");

    let (handle, name, bio, role, deleted, hash, followers, following, posts): (
        String,
        String,
        String,
        String,
        bool,
        Option<String>,
        i32,
        i32,
        i32,
    ) = sqlx::query_as(
        "select handle, display_name, bio, role, deleted_at is not null, password_hash,
                followers_count, following_count, posts_count from users where id = $1",
    )
    .bind(bob_id)
    .fetch_one(&db)
    .await
    .expect("bob's row");
    assert!(
        handle.starts_with("deleted_") && handle.len() == "deleted_".len() + 12,
        "{handle}"
    );
    assert!(
        handle[8..].chars().all(|c| c.is_ascii_hexdigit()),
        "{handle}"
    );
    assert_eq!(
        (name.as_str(), bio.as_str(), role.as_str()),
        ("Deleted account", "", "user")
    );
    assert!(deleted && hash.is_none());
    assert_eq!((followers, following, posts), (0, 0, 0));

    let posts_left: i64 = sqlx::query_scalar(
        "select count(*) from posts where author_id = $1 and (deleted_at is null or text <> '')",
    )
    .bind(bob_id)
    .fetch_one(&db)
    .await
    .expect("count");
    assert_eq!(posts_left, 0);
    for sql in [
        "select count(*) from likes where user_id = $1",
        "select count(*) from reposts where user_id = $1",
        "select count(*) from bookmarks where user_id = $1",
        "select count(*) from follows where follower_id = $1 or followee_id = $1",
        "select count(*) from blocks where user_id = $1 or target_id = $1",
        "select count(*) from mutes where user_id = $1 or target_id = $1",
        "select count(*) from messages where sender_id = $1 or recipient_id = $1",
        "select count(*) from notifications where user_id = $1 or actor_id = $1",
        "select count(*) from sessions where user_id = $1",
        "select count(*) from list_members where user_id = $1",
        "select count(*) from lists where owner_id = $1",
        "select count(*) from scheduled_posts where author_id = $1",
    ] {
        assert_eq!(count(&db, sql, bob_id).await, 0, "{sql}");
    }
    let (likes, reposts, replies, quotes): (i32, i32, i32, i32) = sqlx::query_as(
        "select like_count, repost_count, reply_count, quote_count from posts where id = $1",
    )
    .bind(Uuid::parse_str(&ana_post_id).expect("uuid"))
    .fetch_one(&db)
    .await
    .expect("ana's post counters");
    assert_eq!((likes, reposts, replies, quotes), (0, 0, 0, 0));
    let (followers, following): (i32, i32) =
        sqlx::query_as("select followers_count, following_count from users where id = $1")
            .bind(ana_id)
            .fetch_one(&db)
            .await
            .expect("ana's row");
    assert_eq!((followers, following), (0, 0));
    assert_eq!(
        count(
            &db,
            "select count(*) from likes where post_id = $1",
            Uuid::parse_str(&bob_post_id).expect("uuid")
        )
        .await,
        0
    );

    ana.get("/users/bob")
        .await
        .err(StatusCode::NOT_FOUND, "User not found.");
    let timeline = ana.get("/timelines/following").await.ok();
    assert_eq!(post_texts(&timeline), ["ana speaks"]);
    let all = cat.browser.get("/timelines/foryou").await.ok();
    assert!(
        !post_texts(&all).iter().any(|text| text.starts_with("bob")),
        "{all}"
    );
    assert_eq!(ana.get("/notifications").await.ok()["items"], json!([]));
    assert_eq!(ana.get("/conversations").await.ok()["items"], json!([]));
    assert_eq!(ana.get("/badges").await.ok()["messages"], 0);

    // The handle is free again.
    let mut again = app.browser_at([198, 51, 100, 23]);
    assert_eq!(again.signup("bob").await["user"]["handle"], "bob");
}

// ------------------------------------------------------------------ rss

async fn rss(app: &App, path: &str) -> (StatusCode, HeaderMap, String) {
    let request = Request::builder()
        .uri(path)
        .body(Body::empty())
        .expect("request");
    let response = app
        .router
        .clone()
        .oneshot(request)
        .await
        .expect("router is infallible");
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.expect("body");
    (
        parts.status,
        parts.headers,
        String::from_utf8_lossy(&bytes).into_owned(),
    )
}

#[sqlx::test(migrations = "./migrations")]
async fn the_feed_lists_the_thirty_newest_top_level_posts(db: PgPool) {
    let app = common::app(db.clone());
    let ana = common::insert_user(&db, "ana").await;
    let mut ids = Vec::new();
    for n in 0..32 {
        ids.push(insert_post(&db, ana, &format!("post {n}"), None).await);
    }
    insert_post(&db, ana, "a reply", Some(ids[0])).await;
    let removed = insert_post(&db, ana, "removed", None).await;
    sqlx::query("update posts set deleted_at = now() where id = $1")
        .bind(removed)
        .execute(&db)
        .await
        .expect("delete");

    let (status, headers, body) = rss(&app, "/u/ana/rss.xml").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        headers["content-type"],
        "application/rss+xml; charset=utf-8"
    );
    assert_eq!(headers["cache-control"], "public, max-age=300");
    assert!(
        body.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"),
        "{body}"
    );
    assert_eq!(body.matches("<item>").count(), 30);
    assert!(body.contains("<rss version=\"2.0\">"));
    assert!(
        body.contains("<title>Name of ana (@ana) \u{b7} tuita</title>"),
        "{body}"
    );
    assert!(body.contains("<link>https://social.example/@ana</link>"));
    assert!(body.contains("<description>Posts by @ana on tuita</description>"));
    let newest = ids[31];
    assert!(
        body.contains(&format!(
            "<guid isPermaLink=\"true\">https://social.example/@ana/{newest}</guid>"
        )),
        "{body}"
    );
    assert!(body.find(&ids[31].to_string()) < body.find(&ids[30].to_string()));
    assert!(body.contains("<title>post 31</title>"));
    assert!(!body.contains("post 1<") && !body.contains("<title>post 0</title>"));
    assert!(!body.contains("a reply") && !body.contains("removed"));
    assert!(
        body.contains("<pubDate>") && body.contains(" +0000</pubDate>"),
        "{body}"
    );
    // The handle also works with the @.
    assert_eq!(rss(&app, "/u/@ana/rss.xml").await.0, StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_feed_escapes_every_interpolated_value(db: PgPool) {
    let app = common::app(db.clone());
    let ana = common::insert_user(&db, "ana").await;
    sqlx::query("update users set display_name = $2, bio = $3 where id = $1")
        .bind(ana)
        .bind("Tom & \"Jerry\" <b>")
        .bind("bio </description> & more")
        .execute(&db)
        .await
        .expect("profile");
    let post = insert_post(
        &db,
        ana,
        "<script>alert('x')</script> & \u{1}control\nsecond line",
        None,
    )
    .await;
    sqlx::query("update posts set cw = $2 where id = $1")
        .bind(post)
        .bind("<i>cw</i> & \"q\"")
        .execute(&db)
        .await
        .expect("cw");
    let media = Uuid::now_v7();
    sqlx::query(
        "insert into media (id, owner_id, kind, mime, path, bytes, alt) values ($1, $2, 'image', 'image/png', $3, 1, 'a & b')",
    )
    .bind(media)
    .bind(ana)
    .bind(format!("{media}.png"))
    .execute(&db)
    .await
    .expect("media");
    sqlx::query("insert into post_media (post_id, media_id, position) values ($1, $2, 0)")
        .bind(post)
        .bind(media)
        .execute(&db)
        .await
        .expect("attach");

    let (status, _, body) = rss(&app, "/u/ana/rss.xml").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        !body.contains("<script>") && !body.contains("<b>") && !body.contains("<i>"),
        "{body}"
    );
    assert!(!body.contains('\u{1}'));
    assert!(
        body.contains("<title>Tom &amp; &quot;Jerry&quot; &lt;b&gt; (@ana) \u{b7} tuita</title>"),
        "{body}"
    );
    assert!(
        body.contains("<description>bio &lt;/description&gt; &amp; more</description>"),
        "{body}"
    );
    assert!(
        body.contains("<title>&lt;i&gt;cw&lt;/i&gt; &amp; &quot;q&quot;</title>"),
        "{body}"
    );
    // The description is HTML inside XML, so it is escaped twice.
    assert!(
        body.contains("&lt;p&gt;&lt;strong&gt;CW: &amp;lt;i&amp;gt;cw&amp;lt;/i&amp;gt; &amp;amp; &amp;quot;q&amp;quot;&lt;/strong&gt;&lt;/p&gt;"),
        "{body}"
    );
    assert!(
        body.contains("&lt;p&gt;&amp;lt;script&amp;gt;alert(&amp;apos;x&amp;apos;)&amp;lt;/script&amp;gt; &amp;amp; control&lt;br&gt;second line&lt;/p&gt;"),
        "{body}"
    );
    assert!(
        body.contains(&format!(
            "&lt;img src=&quot;https://social.example/media/{media}.png&quot; alt=&quot;a &amp;amp; b&quot;&gt;"
        )),
        "{body}"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn the_feed_is_plain_text_404_for_unknown_deleted_and_suspended_accounts(db: PgPool) {
    let app = common::app(db.clone());
    let gone = common::insert_user(&db, "gone").await;
    let held = common::insert_user(&db, "held").await;
    sqlx::query("update users set deleted_at = now() where id = $1")
        .bind(gone)
        .execute(&db)
        .await
        .expect("delete");
    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(held)
        .execute(&db)
        .await
        .expect("suspend");
    for handle in ["nobody", "gone", "held", "someone@remote.example"] {
        let (status, headers, body) = rss(&app, &format!("/u/{handle}/rss.xml")).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{handle}");
        assert_eq!(headers["content-type"], "text/plain", "{handle}");
        assert_eq!(body, "Not found", "{handle}");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn deleting_an_account_orphans_its_media_so_cleanup_removes_the_files(db: PgPool) {
    let data = std::env::temp_dir().join(format!("tuita-delete-{}", Uuid::now_v7()));
    let dir = data.display().to_string();
    let app = common::app_with(db.clone(), |var| (var == "DATA_DIR").then(|| dir.clone()));
    let mut admin = app.browser_at([198, 51, 100, 30]);
    admin.signup("ada_first").await;
    let mut bob = app.browser_at([198, 51, 100, 31]);
    let me = bob.signup("bob").await;
    let bob_id = uuid_of(&me["user"]);

    let media = Uuid::now_v7();
    let name = format!("{media}.webp");
    std::fs::create_dir_all(data.join("media")).expect("mkdir");
    std::fs::write(data.join("media").join(&name), b"RIFF").expect("file");
    sqlx::query(
        "insert into media (id, owner_id, kind, mime, path, bytes, created_at)
         values ($1, $2, 'image', 'image/webp', $3, 4, now() - interval '48 hours')",
    )
    .bind(media)
    .bind(bob_id)
    .bind(&name)
    .execute(&db)
    .await
    .expect("media");
    bob.post(
        "/posts",
        json!({ "text": "pic", "mediaIds": [media.to_string()] }),
    )
    .await
    .ok();
    assert_eq!(
        count(
            &db,
            "select count(*) from media where post_id is not null and owner_id = $1",
            bob_id
        )
        .await,
        1
    );

    delete_me(&mut bob, PASSWORD).await.ok();
    assert_eq!(
        count(
            &db,
            "select count(*) from media where post_id is not null and owner_id = $1",
            bob_id
        )
        .await,
        0,
        "the upload still points at a post"
    );
    tuita_server::workers::cleanup::run_once(&app.state)
        .await
        .expect("cleanup");
    assert_eq!(
        count(
            &db,
            "select count(*) from media where owner_id = $1",
            bob_id
        )
        .await,
        0
    );
    assert!(!data.join("media").join(&name).exists());
    std::fs::remove_dir_all(&data).ok();
}
