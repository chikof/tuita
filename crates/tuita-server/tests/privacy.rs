//! Privacy hardening of the transport and the session: cache headers, HSTS, the forwarded
//! address and what a password change revokes.

mod common;

use axum::body::Body;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use common::{App, Browser, PASSWORD, RawReply};

const CLIENT_ORIGIN: &str = "https://client.example";
const REDIRECT: &str = "https://client.example/callback";
/// RFC 7636 appendix B.
const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
const CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

async fn raw_get(browser: &mut Browser, uri: &str) -> RawReply {
    browser.send_raw(Method::GET, uri, &[], Body::empty()).await
}

/// A script that sends only `Authorization: Bearer`.
fn script(app: &App, secret: &str) -> Browser {
    let mut client = app.browser();
    client.origin = None;
    client
        .headers
        .push(("authorization".to_owned(), format!("Bearer {secret}")));
    client
}

fn secure(value: &'static str) -> impl Fn(&str) -> Option<String> {
    move |var| (var == "SECURE_COOKIES").then(|| value.to_owned())
}

// ------------------------------------------------------------------ cache

#[sqlx::test(migrations = "./migrations")]
async fn cache_private_json_is_never_stored(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    for path in ["/me", "/conversations", "/tokens", "/me/export"] {
        let reply = ana.browser.get(path).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.body);
        assert_eq!(reply.header("cache-control"), "no-store", "{path}");
        assert_eq!(reply.header("vary"), "Cookie, Authorization", "{path}");
    }
    let refused = app.browser().get("/me/sessions").await;
    assert_eq!(refused.status, StatusCode::UNAUTHORIZED);
    assert_eq!(refused.header("cache-control"), "no-store", "errors too");
    let missing = ana.browser.get("/no/such/route").await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(missing.header("cache-control"), "no-store");
}

#[sqlx::test(migrations = "./migrations")]
async fn cache_keeps_the_public_status(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let post = ana.browser.post_text("hello").await;
    let id = post["id"].as_str().expect("id");

    let mut reader = app.browser();
    let public = reader.get(&format!("/statuses/{id}")).await;
    assert_eq!(public.status, StatusCode::OK, "{}", public.body);
    assert_eq!(public.header("cache-control"), "public, max-age=60");

    let signed_in = ana.browser.get(&format!("/statuses/{id}")).await;
    assert_eq!(signed_in.status, StatusCode::OK, "{}", signed_in.body);
    assert_eq!(signed_in.header("cache-control"), "no-store");
}

// ------------------------------------------------------------------ hsts

#[sqlx::test(migrations = "./migrations")]
async fn hsts_is_sent_when_cookies_are_secure(db: PgPool) {
    let app = common::app_with(db, secure("true"));
    let mut anyone = app.browser();
    for uri in [
        "/api/v1/me",
        "/api/v1/no/such/route",
        "/healthz",
        "/favicon.svg",
        "/",
    ] {
        let reply = raw_get(&mut anyone, uri).await;
        assert_eq!(
            reply.header("strict-transport-security"),
            "max-age=31536000",
            "{uri}"
        );
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn hsts_is_absent_when_cookies_are_not_secure(db: PgPool) {
    let app = common::app_with(db, secure("false"));
    let mut anyone = app.browser();
    for uri in ["/api/v1/me", "/healthz", "/favicon.svg", "/"] {
        let reply = raw_get(&mut anyone, uri).await;
        assert!(
            !reply.headers.contains_key("strict-transport-security"),
            "{uri}"
        );
    }
}

// ------------------------------------------------------------------ forwarded

/// The address stored with `ana`'s login session after a signup that sent `lines`.
async fn session_ip(app: &App, lines: &[&str]) -> String {
    let mut browser = app.browser();
    for line in lines {
        browser
            .headers
            .push(("x-forwarded-for".to_owned(), (*line).to_owned()));
    }
    browser.signup("ana").await;
    let sessions = browser.get("/me/sessions").await.ok();
    sessions[0]["ip"].as_str().expect("ip").to_owned()
}

#[sqlx::test(migrations = "./migrations")]
async fn forwarded_the_last_value_of_the_last_line_wins(db: PgPool) {
    let app = common::app_with(db, |var| (var == "TRUST_PROXY").then(|| "true".to_owned()));
    let ip = session_ip(&app, &["6.6.6.6, 7.7.7.7", "8.8.8.8, 203.0.113.9"]).await;
    assert_eq!(ip, "203.0.113.9");
}

#[sqlx::test(migrations = "./migrations")]
async fn forwarded_an_appended_hop_beats_the_clients_own(db: PgPool) {
    let app = common::app_with(db, |var| (var == "TRUST_PROXY").then(|| "true".to_owned()));
    assert_eq!(
        session_ip(&app, &["6.6.6.6, 203.0.113.9"]).await,
        "203.0.113.9"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn forwarded_is_ignored_without_trust_proxy(db: PgPool) {
    let app = common::app(db);
    // The fake browser connects from 198.51.100.1.
    assert_eq!(
        session_ip(&app, &["6.6.6.6", "203.0.113.9"]).await,
        "198.51.100.1"
    );
}

// ------------------------------------------------------------------ password

/// An OAuth token and a second, unredeemed code for the signed-in `user`.
async fn oauth_token_and_spare_code(app: &App, user: &mut Browser) -> String {
    let mut client = app.browser_at([203, 0, 113, 7]);
    client.origin = Some(CLIENT_ORIGIN.to_owned());
    let registered = client
        .post(
            "/apps",
            json!({
                "client_name": "Test App",
                "redirect_uris": REDIRECT,
                "scopes": "read",
                "website": CLIENT_ORIGIN,
            }),
        )
        .await
        .ok();
    let mut codes = Vec::new();
    for _ in 0..2 {
        let body = json!({
            "clientId": registered["client_id"],
            "redirectUri": REDIRECT,
            "scope": "read",
            "responseType": "code",
            "state": "st",
            "approve": true,
            "codeChallenge": CHALLENGE,
            "codeChallengeMethod": "S256",
        });
        let reply = user
            .send_raw(
                Method::POST,
                "/oauth/authorize",
                &[("content-type", "application/json")],
                Body::from(body.to_string()),
            )
            .await
            .ok();
        let redirect = reply["redirect"].as_str().expect("redirect").to_owned();
        let url = url::Url::parse(&redirect).expect("url");
        let code = url
            .query_pairs()
            .find(|(key, _)| key == "code")
            .expect("code")
            .1
            .into_owned();
        codes.push(code);
    }
    let form = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs([
            ("grant_type", "authorization_code"),
            ("client_id", registered["client_id"].as_str().expect("id")),
            ("code", codes[0].as_str()),
            ("redirect_uri", REDIRECT),
            ("code_verifier", VERIFIER),
        ])
        .finish();
    let token: Value = client
        .send_raw(
            Method::POST,
            "/oauth/token",
            &[("content-type", "application/x-www-form-urlencoded")],
            Body::from(form),
        )
        .await
        .ok();
    token["access_token"].as_str().expect("token").to_owned()
}

const TOKENS: &str = "select count(*) from api_tokens where user_id = $1";
const CODES: &str = "select count(*) from oauth_codes where user_id = $1";
const PASSKEYS: &str = "select count(*) from passkeys where user_id = $1";
const SESSIONS: &str = "select count(*) from sessions where user_id = $1";

async fn left(db: &PgPool, sql: &'static str, user: uuid::Uuid) -> i64 {
    sqlx::query_scalar(sql)
        .bind(user)
        .fetch_one(db)
        .await
        .expect("count")
}

#[sqlx::test(migrations = "./migrations")]
async fn password_change_revokes_tokens_and_codes_but_keeps_the_session_and_passkeys(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.browser();
    let me = ana.signup("ana").await;
    let user: uuid::Uuid = me["user"]["id"]
        .as_str()
        .expect("id")
        .parse()
        .expect("uuid");
    let mut bob = app.browser_at([198, 51, 100, 9]);
    bob.signup("bob").await;
    let bob_token = bob
        .post("/tokens", json!({ "name": "bob", "scopes": ["read"] }))
        .await
        .ok()["token"]
        .as_str()
        .expect("secret")
        .to_owned();

    let personal = ana
        .post("/tokens", json!({ "name": "script", "scopes": ["read"] }))
        .await
        .ok()["token"]
        .as_str()
        .expect("secret")
        .to_owned();
    let oauth = oauth_token_and_spare_code(&app, &mut ana).await;
    sqlx::query(
        "insert into passkeys (id, user_id, cred_id, passkey) values ($1, $2, '\\x01', '{}')",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(user)
    .execute(&db)
    .await
    .expect("passkey");

    assert_eq!(
        script(&app, &personal).get("/me").await.status,
        StatusCode::OK
    );
    assert_eq!(
        script(&app, &oauth)
            .get("/accounts/verify_credentials")
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(left(&db, TOKENS, user).await, 2);
    assert_eq!(left(&db, CODES, user).await, 2);

    ana.post(
        "/me/password",
        json!({ "current": PASSWORD, "next": "a brand new pass" }),
    )
    .await
    .ok();

    assert_eq!(
        left(&db, TOKENS, user).await,
        0,
        "personal and OAuth tokens"
    );
    assert_eq!(left(&db, CODES, user).await, 0, "redeemed and spare codes");
    assert_eq!(left(&db, PASSKEYS, user).await, 1, "passkeys stay");
    assert_eq!(
        left(&db, SESSIONS, user).await,
        1,
        "the current session stays"
    );

    script(&app, &personal)
        .get("/me")
        .await
        .err(StatusCode::UNAUTHORIZED, "Invalid or revoked API token.");
    assert_eq!(
        script(&app, &oauth)
            .get("/accounts/verify_credentials")
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        ana.get("/me").await.ok()["user"]["user"]["handle"],
        "ana",
        "the reply must not log the caller out"
    );
    assert_eq!(
        script(&app, &bob_token).get("/me").await.status,
        StatusCode::OK,
        "other accounts keep their tokens"
    );
}
