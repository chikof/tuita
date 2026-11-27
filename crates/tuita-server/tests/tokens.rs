//! API tokens and bots: bearer auth, scopes, the cookie-only paths, CSRF with and without a
//! cookie, rate limits and the token cap.

mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

use common::{App, Browser};

/// Makes a token for the signed-in `browser` and returns its secret.
async fn make_token(browser: &mut Browser, scopes: &[&str]) -> (String, Value) {
    let created = browser
        .post("/tokens", json!({ "name": "script", "scopes": scopes }))
        .await
        .ok();
    let secret = created["token"].as_str().expect("secret").to_owned();
    (secret, created["info"].clone())
}

/// A client that sends only `Authorization: Bearer`, like a script: no cookies, no `Origin`.
fn script(app: &App, secret: &str) -> Browser {
    let mut client = app.browser();
    client.origin = None;
    client
        .headers
        .push(("authorization".to_owned(), format!("Bearer {secret}")));
    client
}

#[sqlx::test(migrations = "./migrations")]
async fn a_read_token_cannot_post_but_a_write_token_can(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let (read, _) = make_token(&mut ana.browser, &["read"]).await;
    let (write, _) = make_token(&mut ana.browser, &["write"]).await;

    let mut reader = script(&app, &read);
    assert_eq!(
        reader.get("/me").await.ok()["user"]["user"]["handle"],
        "ana"
    );
    reader
        .post("/posts", json!({ "text": "from a bot" }))
        .await
        .err(
            StatusCode::FORBIDDEN,
            "This token doesn\u{2019}t have the \u{201c}write\u{201d} scope.",
        );
    reader.get("/conversations").await.err(
        StatusCode::FORBIDDEN,
        "This token doesn\u{2019}t have the \u{201c}messages\u{201d} scope.",
    );

    let mut writer = script(&app, &write);
    let post = writer
        .post("/posts", json!({ "text": "from a bot" }))
        .await
        .ok();
    assert_eq!(post["author"]["handle"], "ana");
    writer.get("/me").await.err(
        StatusCode::FORBIDDEN,
        "This token doesn\u{2019}t have the \u{201c}read\u{201d} scope.",
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn tokens_are_refused_on_the_cookie_only_paths(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    sqlx::query("update users set role = 'admin' where id = $1")
        .bind(ana.id)
        .execute(&db)
        .await
        .expect("admin");
    let (secret, _) = make_token(&mut ana.browser, &["read", "write", "messages"]).await;
    let mut client = script(&app, &secret);
    let refused = "API tokens can\u{2019}t be used for this. Sign in on the website instead.";
    client
        .post(
            "/me/password",
            json!({ "current": common::PASSWORD, "next": "another long password" }),
        )
        .await
        .err(StatusCode::FORBIDDEN, refused);
    client
        .get("/tokens")
        .await
        .err(StatusCode::FORBIDDEN, refused);
    client
        .post("/tokens", json!({ "name": "x" }))
        .await
        .err(StatusCode::FORBIDDEN, refused);
    client
        .get("/admin/stats")
        .await
        .err(StatusCode::FORBIDDEN, refused);
    client
        .get("/me/sessions")
        .await
        .err(StatusCode::FORBIDDEN, refused);
    client
        .get("/me/export")
        .await
        .err(StatusCode::FORBIDDEN, refused);
    client
        .get("/accounts")
        .await
        .err(StatusCode::FORBIDDEN, refused);
    client
        .get("/passkeys")
        .await
        .err(StatusCode::FORBIDDEN, refused);
    client
        .post("/auth/logout", json!({}))
        .await
        .err(StatusCode::FORBIDDEN, refused);
    client
        .delete("/me")
        .await
        .err(StatusCode::FORBIDDEN, refused);
    // The account is untouched, and a token may still change its own profile and bot flag.
    let me = client.patch("/me/bot", json!({ "bot": true })).await.ok();
    assert_eq!(me["user"]["bot"], true);
    assert!(client.patch("/me", json!({ "bio": "hello" })).await.ok()["user"]["bio"] == "hello");
}

#[sqlx::test(migrations = "./migrations")]
async fn the_secret_is_shown_once_and_only_its_hash_is_kept(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let (secret, info) = make_token(&mut ana.browser, &["read"]).await;
    assert!(secret.starts_with("tuita_") && secret.len() == 6 + 43);
    assert_eq!(info["prefix"], secret[..12]);
    let listed = ana.browser.get("/tokens").await;
    assert!(
        !listed.body.contains(&secret),
        "the list never repeats the secret"
    );
    assert!(!listed.body.contains(&secret[12..]));
    assert_eq!(listed.json()[0]["name"], "script");
    assert_eq!(listed.json()[0]["scopes"], json!(["read"]));
    let stored: Vec<u8> = sqlx::query_scalar("select token_hash from api_tokens")
        .fetch_one(&db)
        .await
        .expect("hash");
    assert_eq!(stored, tuita_server::auth::hash_token(&secret));
    let text: String = sqlx::query_scalar("select name || prefix || scopes::text from api_tokens")
        .fetch_one(&db)
        .await
        .expect("row");
    assert!(!text.contains(&secret));
    ana.browser
        .post("/tokens", json!({ "name": "  " }))
        .await
        .err(StatusCode::BAD_REQUEST, "Name is required.");
    ana.browser
        .post("/tokens", json!({ "name": "x", "scopes": [] }))
        .await
        .err(StatusCode::BAD_REQUEST, "Pick at least one scope.");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_revoked_unknown_or_malformed_bearer_is_401_never_a_cookie_fallback(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let (secret, info) = make_token(&mut ana.browser, &["read"]).await;
    let invalid = "Invalid or revoked API token.";
    script(&app, &secret).get("/me").await.ok();
    ana.browser
        .delete(&format!("/tokens/{}", info["id"].as_str().expect("id")))
        .await
        .ok();
    script(&app, &secret)
        .get("/me")
        .await
        .err(StatusCode::UNAUTHORIZED, invalid);
    script(&app, "tuita_unknownunknownunknownunknownunknown00")
        .get("/me")
        .await
        .err(StatusCode::UNAUTHORIZED, invalid);
    script(&app, "garbage")
        .get("/me")
        .await
        .err(StatusCode::UNAUTHORIZED, invalid);
    // A valid session cookie does not rescue a bad bearer.
    ana.browser
        .headers
        .push(("authorization".to_owned(), "Bearer not-a-token".to_owned()));
    ana.browser
        .get("/me")
        .await
        .err(StatusCode::UNAUTHORIZED, invalid);
    ana.browser.headers.clear();
    ana.browser.get("/me").await.ok();
    ana.browser
        .delete("/tokens/00000000-0000-0000-0000-000000000000")
        .await
        .err(StatusCode::NOT_FOUND, "Token not found.");
}

#[sqlx::test(migrations = "./migrations")]
async fn csrf_applies_whenever_a_cookie_is_present_and_never_without_one(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let bob = app.person(&db, "bob").await;
    let (secret, _) = make_token(&mut ana.browser, &["read", "write"]).await;
    // No cookie and no Origin: a script. Passes.
    script(&app, &secret)
        .post("/posts", json!({ "text": "ok" }))
        .await
        .ok();
    // The cookie jar of a browser plus a bearer: the Origin is still required.
    let mut both = script(&app, &secret);
    both.set_cookie(
        tuita_server::auth::SESSION_COOKIE,
        ana.browser
            .cookie(tuita_server::auth::SESSION_COOKIE)
            .expect("cookie"),
    );
    both.post("/posts", json!({ "text": "no origin" }))
        .await
        .err(StatusCode::FORBIDDEN, "bad origin");
    both.origin = Some(common::ORIGIN.to_owned());
    both.post("/posts", json!({ "text": "with origin" }))
        .await
        .ok();
    // The bearer identifies the request: ana's cookie with bob's token acts as bob.
    let mut bob_browser = bob.browser;
    let (bob_secret, _) = make_token(&mut bob_browser, &["read"]).await;
    let mut mixed = script(&app, &bob_secret);
    mixed.set_cookie(
        tuita_server::auth::SESSION_COOKIE,
        ana.browser
            .cookie(tuita_server::auth::SESSION_COOKIE)
            .expect("cookie"),
    );
    mixed.origin = Some(common::ORIGIN.to_owned());
    assert_eq!(mixed.get("/me").await.ok()["user"]["user"]["handle"], "bob");
    mixed
        .post("/posts", json!({ "text": "as bob?" }))
        .await
        .err(
            StatusCode::FORBIDDEN,
            "This token doesn\u{2019}t have the \u{201c}write\u{201d} scope.",
        );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_token_gets_600_requests_per_ten_minutes_and_records_its_use(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let (secret, _) = make_token(&mut ana.browser, &["read"]).await;
    let used: Option<time::OffsetDateTime> =
        sqlx::query_scalar("select last_used_at from api_tokens")
            .fetch_one(&db)
            .await
            .expect("row");
    assert!(used.is_none());
    let mut client = script(&app, &secret);
    for _ in 0..600 {
        assert_eq!(client.get("/me").await.status, StatusCode::OK);
    }
    let over = client.get("/me").await;
    over.err(
        StatusCode::TOO_MANY_REQUESTS,
        "Too many requests. Try again in a few minutes.",
    );
    assert!(over.header("retry-after").parse::<u32>().expect("seconds") >= 1);
    let used: Option<time::OffsetDateTime> =
        sqlx::query_scalar("select last_used_at from api_tokens")
            .fetch_one(&db)
            .await
            .expect("row");
    assert!(used.is_some(), "the first use was recorded");
}

#[sqlx::test(migrations = "./migrations")]
async fn an_account_holds_at_most_twenty_tokens(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    for n in 0..20 {
        sqlx::query("insert into api_tokens (id, user_id, name, scopes, token_hash, prefix) values ($1, $2, 't', '{read}', $3, 'tuita_xxxxxx')")
            .bind(uuid::Uuid::now_v7())
            .bind(ana.id)
            .bind(vec![n as u8; 32])
            .execute(&db)
            .await
            .expect("token");
    }
    ana.browser
        .post("/tokens", json!({ "name": "one too many" }))
        .await
        .err(StatusCode::BAD_REQUEST, "You can have up to 20 tokens.");
    // Creation itself is limited to 20 per hour.
    let mut bea = app.person(&db, "bea").await;
    for _ in 0..20 {
        let created = bea
            .browser
            .post("/tokens", json!({ "name": "t" }))
            .await
            .ok();
        let id = created["info"]["id"].as_str().expect("id").to_owned();
        bea.browser.delete(&format!("/tokens/{id}")).await.ok();
    }
    bea.browser
        .post("/tokens", json!({ "name": "t" }))
        .await
        .err(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many requests. Try again in a few minutes.",
        );
}

#[sqlx::test(migrations = "./migrations")]
async fn bots_name_their_owner_and_suspended_owners_end_their_tokens(db: PgPool) {
    let app = common::app(db.clone());
    let mut bot = app.person(&db, "newsbot").await;
    let _owner = app.person(&db, "maker").await;
    let me = bot
        .browser
        .patch("/me/bot", json!({ "bot": true, "owner": "@maker" }))
        .await
        .ok();
    assert_eq!(me["user"]["bot"], true);
    assert_eq!(me["user"]["botOwner"], "maker");
    bot.browser
        .patch("/me/bot", json!({ "bot": true, "owner": "newsbot" }))
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "A bot can\u{2019}t be run by itself.",
        );
    bot.browser
        .patch("/me/bot", json!({ "bot": true, "owner": "ghost" }))
        .await
        .err(
            StatusCode::NOT_FOUND,
            "No account with that handle on this instance.",
        );
    let off = bot
        .browser
        .patch("/me/bot", json!({ "bot": false, "owner": "maker" }))
        .await
        .ok();
    assert_eq!(off["user"]["bot"], false);
    assert_eq!(off["user"]["botOwner"], Value::Null);
    let (secret, _) = make_token(&mut bot.browser, &["read"]).await;
    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(bot.id)
        .execute(&db)
        .await
        .expect("suspend");
    script(&app, &secret)
        .get("/me")
        .await
        .err(StatusCode::UNAUTHORIZED, "Invalid or revoked API token.");
}
