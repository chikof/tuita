//! The M1 acceptance flows, driven through the public API only.
#![allow(missing_docs)]

mod common;

use std::net::SocketAddr;
use std::time::Duration;

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use common::{PASSWORD, app};
use futures_util::StreamExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

fn texts(page: &Value) -> Vec<&str> {
    page["items"]
        .as_array()
        .expect("items")
        .iter()
        .map(|post| post["text"].as_str().expect("text"))
        .collect()
}

#[sqlx::test(migrations = "./migrations")]
async fn signup_first_user_is_admin_second_is_user(db: PgPool) {
    let app = app(db);
    let mut ana = app.browser();
    let first = ana.signup("ana").await;
    assert_eq!(first["role"], "admin");
    let mut ben = app.browser_at([198, 51, 100, 2]);
    let second = ben.signup("ben").await;
    assert_eq!(second["role"], "user");
    assert_eq!(ben.get("/me").await.ok()["user"]["user"]["handle"], "ben");
}

#[sqlx::test(migrations = "./migrations")]
async fn login_is_rate_limited_on_the_21st_try_in_ten_minutes(db: PgPool) {
    let app = app(db);
    app.browser().signup("ana").await;
    let mut attacker = app.browser_at([203, 0, 113, 50]);
    for _ in 0..20 {
        let reply = attacker
            .post(
                "/auth/login",
                json!({ "handle": "ana", "password": "not the password" }),
            )
            .await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    }
    let reply = attacker
        .post(
            "/auth/login",
            json!({ "handle": "ana", "password": PASSWORD }),
        )
        .await;
    assert_eq!(reply.status, StatusCode::TOO_MANY_REQUESTS);
    assert!(reply.header("retry-after").parse::<u32>().expect("seconds") > 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn post_like_notifies_the_author_and_sets_badges(db: PgPool) {
    let app = app(db);
    let mut ana = app.browser();
    ana.signup("ana").await;
    let mut ben = app.browser_at([198, 51, 100, 2]);
    ben.signup("ben").await;

    let post = ana.post_text("hello #tuita").await;
    let id = post["id"].as_str().expect("id").to_owned();
    assert_eq!(post["counts"]["likes"], 0);
    assert_eq!(post["richText"][1]["type"], "hashtag");

    assert_eq!(ana.get("/badges").await.ok()["notifications"], 0);
    let liked = ben.post_empty(&format!("/posts/{id}/like")).await.ok();
    assert_eq!(liked["counts"]["likes"], 1);
    assert_eq!(liked["viewer"]["liked"], true);

    let list = ana.get("/notifications").await.ok();
    let rows = list["items"].as_array().expect("rows");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["kind"], "like");
    assert_eq!(rows[0]["actors"][0]["handle"], "ben");
    assert_eq!(rows[0]["actorCount"], 1);
    assert_eq!(rows[0]["post"]["id"], post["id"]);
    assert_eq!(rows[0]["read"], false);
    assert_eq!(ana.get("/badges").await.ok()["notifications"], 1);

    ana.post_empty("/notifications/read").await.ok();
    assert_eq!(ana.get("/badges").await.ok()["notifications"], 0);
    assert_eq!(
        ana.get("/notifications").await.ok()["items"][0]["read"],
        true
    );

    // Liking your own post notifies nobody.
    ana.post_empty(&format!("/posts/{id}/like")).await.ok();
    assert_eq!(ana.get("/badges").await.ok()["notifications"], 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_live_stream_receives_the_notification_when_someone_likes(db: PgPool) {
    let app = app(db);
    let mut ana = app.browser();
    ana.signup("ana").await;
    let mut ben = app.browser_at([198, 51, 100, 2]);
    ben.signup("ben").await;
    let id = ana.post_text("watch this").await["id"]
        .as_str()
        .expect("id")
        .to_owned();

    let mut request = Request::builder()
        .uri("/api/v1/stream")
        .header(
            "cookie",
            format!(
                "tuita_session={}",
                ana.cookie("tuita_session").expect("session")
            ),
        )
        .body(Body::empty())
        .expect("request");
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([198, 51, 100, 1], 1))));
    let response = app
        .router
        .clone()
        .oneshot(request)
        .await
        .expect("infallible");
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response.headers()["content-type"]
            .to_str()
            .expect("ct")
            .starts_with("text/event-stream")
    );
    let mut body = response.into_body().into_data_stream();

    async fn read_until(body: &mut axum::body::BodyDataStream, needle: &str) -> String {
        let mut seen = String::new();
        while !seen.contains(needle) {
            let chunk = tokio::time::timeout(Duration::from_secs(5), body.next())
                .await
                .unwrap_or_else(|_| panic!("timed out waiting for {needle:?}; saw {seen:?}"))
                .expect("stream open")
                .expect("chunk");
            seen.push_str(&String::from_utf8_lossy(&chunk));
        }
        seen
    }
    let first = read_until(&mut body, "event: badges").await;
    assert!(first.contains("retry: 5000"), "{first}");

    ben.post_empty(&format!("/posts/{id}/like")).await.ok();
    let frames = read_until(&mut body, "event: notification").await;
    let data = frames
        .lines()
        .skip_while(|line| *line != "event: notification")
        .find_map(|line| line.strip_prefix("data: "))
        .expect("notification data");
    let event: Value = serde_json::from_str(data).expect("json");
    assert_eq!(event["type"], "notification");
    assert_eq!(event["data"]["kind"], "like");
    assert_eq!(event["data"]["actor"]["handle"], "ben");
    assert_eq!(event["data"]["post"]["text"], "watch this");
}

#[sqlx::test(migrations = "./migrations")]
async fn follow_puts_the_post_on_the_following_timeline(db: PgPool) {
    let app = app(db);
    let mut ana = app.browser();
    ana.signup("ana").await;
    let mut ben = app.browser_at([198, 51, 100, 2]);
    ben.signup("ben").await;
    ana.post_text("before the follow").await;

    assert!(texts(&ben.get("/timelines/following").await.ok()).is_empty());
    let followed = ben.post_empty("/users/ana/follow").await.ok();
    assert_eq!(followed["relation"]["following"], true);
    assert_eq!(followed["counts"]["followers"], 1);
    ana.post_text("after the follow").await;
    ben.post_text("my own post").await;
    assert_eq!(
        texts(&ben.get("/timelines/following").await.ok()),
        ["my own post", "after the follow", "before the follow"]
    );
    assert_eq!(
        ben.get("/me").await.ok()["user"]["user"]["counts"]["following"],
        1
    );

    let notes = ana.get("/notifications").await.ok();
    assert_eq!(notes["items"][0]["kind"], "follow");

    ben.delete("/users/ana/follow").await.ok();
    assert_eq!(
        texts(&ben.get("/timelines/following").await.ok()),
        ["my own post"]
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn block_hides_both_ways(db: PgPool) {
    let app = app(db);
    let mut ana = app.browser();
    ana.signup("ana").await;
    let mut ben = app.browser_at([198, 51, 100, 2]);
    ben.signup("ben").await;
    let a_post = ana.post_text("from ana").await["id"]
        .as_str()
        .expect("id")
        .to_owned();
    let b_post = ben.post_text("from ben").await["id"]
        .as_str()
        .expect("id")
        .to_owned();
    ben.post_empty("/users/ana/follow").await.ok();
    ana.post_empty("/users/ben/follow").await.ok();

    let blocked = ana.post_empty("/users/ben/block").await.ok();
    assert_eq!(blocked["relation"]["blocked"], true);
    assert_eq!(blocked["counts"]["following"], 0);

    // Follows are gone both ways and each side's posts disappear for the other.
    assert_eq!(
        texts(&ana.get("/timelines/following").await.ok()),
        ["from ana"]
    );
    assert_eq!(
        texts(&ben.get("/timelines/following").await.ok()),
        ["from ben"]
    );
    ana.get(&format!("/posts/{b_post}"))
        .await
        .err(StatusCode::NOT_FOUND, "This post is unavailable.");
    ben.get(&format!("/posts/{a_post}"))
        .await
        .err(StatusCode::NOT_FOUND, "This post is unavailable.");
    assert!(texts(&ben.get("/users/ana/posts").await.ok()).is_empty());
    let found = ana.get("/search?q=from").await.ok();
    assert_eq!(found["posts"].as_array().expect("posts").len(), 1);
    assert_eq!(found["posts"][0]["text"], "from ana");
    ben.post_empty(&format!("/posts/{a_post}/like")).await.err(
        StatusCode::FORBIDDEN,
        "You can\u{2019}t interact with this post.",
    );
    ben.post_empty("/users/ana/follow").await.err(
        StatusCode::FORBIDDEN,
        "You can\u{2019}t follow this account.",
    );
    ben.post("/posts", json!({ "text": "hi", "replyTo": a_post }))
        .await
        .err(
            StatusCode::FORBIDDEN,
            "You can\u{2019}t interact with this post.",
        );
    let relation = ben.get("/users/ana").await.ok();
    assert_eq!(relation["relation"]["blocksYou"], true);

    ana.delete("/users/ben/block").await.ok();
    assert_eq!(texts(&ben.get("/users/ana/posts").await.ok()), ["from ana"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn deleting_a_post_decrements_every_counter(db: PgPool) {
    let app = app(db);
    let mut ana = app.browser();
    ana.signup("ana").await;
    let mut ben = app.browser_at([198, 51, 100, 2]);
    ben.signup("ben").await;

    let parent = ana.post_text("parent").await["id"]
        .as_str()
        .expect("id")
        .to_owned();
    let reply = ben
        .post("/posts", json!({ "text": "reply", "replyTo": parent }))
        .await
        .ok();
    let quote = ben
        .post("/posts", json!({ "text": "quote", "quoteOf": parent }))
        .await
        .ok();
    assert_eq!(quote["quote"]["text"], "parent");
    let me = ben.get("/me").await.ok();
    assert_eq!(me["user"]["user"]["counts"]["posts"], 2);
    let thread = ana.get(&format!("/posts/{parent}")).await.ok();
    assert_eq!(thread["post"]["counts"]["replies"], 1);
    assert_eq!(thread["post"]["counts"]["quotes"], 1);
    assert_eq!(reply["replyTo"]["handle"], "ana");

    ben.delete(&format!("/posts/{}", reply["id"].as_str().expect("id")))
        .await
        .ok();
    ben.delete(&format!("/posts/{}", quote["id"].as_str().expect("id")))
        .await
        .ok();
    let thread = ana.get(&format!("/posts/{parent}")).await.ok();
    assert_eq!(thread["post"]["counts"]["replies"], 0);
    assert_eq!(thread["post"]["counts"]["quotes"], 0);
    assert_eq!(
        ben.get("/me").await.ok()["user"]["user"]["counts"]["posts"],
        0
    );

    // The notifications those posts caused are gone with them.
    assert!(
        ana.get("/notifications").await.ok()["items"]
            .as_array()
            .expect("items")
            .is_empty()
    );

    ana.delete(&format!("/posts/{parent}")).await.ok();
    assert_eq!(
        ana.get("/me").await.ok()["user"]["user"]["counts"]["posts"],
        0
    );
    ana.get(&format!("/posts/{parent}"))
        .await
        .err(StatusCode::NOT_FOUND, "Post not found.");
}

#[sqlx::test(migrations = "./migrations")]
async fn cross_origin_post_with_a_cookie_is_403(db: PgPool) {
    let app = app(db);
    let mut ana = app.browser();
    ana.signup("ana").await;
    ana.origin = Some("https://evil.example".to_owned());
    ana.post("/posts", json!({ "text": "forged" }))
        .await
        .err(StatusCode::FORBIDDEN, "bad origin");
    ana.origin = Some(common::ORIGIN.to_owned());
    assert!(texts(&ana.get("/users/ana/posts").await.ok()).is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn the_account_switcher_holds_five_accounts_and_rejects_a_sixth(db: PgPool) {
    let app = app(db);
    let mut browser = app.browser();
    for n in 1..=5 {
        browser.signup_as(&format!("acct_{n}"), true).await;
    }
    assert_eq!(
        browser.get("/accounts").await.ok()["accounts"]
            .as_array()
            .expect("list")
            .len(),
        5
    );
    browser
        .post(
            "/auth/signup",
            json!({ "handle": "acct_6", "displayName": "Six", "password": PASSWORD, "acceptTerms": true, "add": true }),
        )
        .await
        .err(StatusCode::BAD_REQUEST, "You can be signed in to up to 5 accounts at once.");
    assert_eq!(
        browser.get("/accounts").await.ok()["accounts"]
            .as_array()
            .expect("list")
            .len(),
        5
    );
    // The refused sign-up must not have created the account.
    let mut other = app.browser_at([198, 51, 100, 9]);
    other.signup("acct_6").await;
}
