//! `/oauth/authorize`, `/developers`, `/privacy` and `/terms` through the whole application.
//!
//! Each `#[sqlx::test]` gets a fresh database; `DATABASE_URL` must point at a running Postgres.
//! Run `topcoat asset bundle --package tuita` first (or point `TUITA_TEST_ASSETS` at a bundle).

mod common;

use axum::http::{Method, StatusCode};
use common::{Browser, app};
use serde_json::{Value, json};
use sqlx::PgPool;

const CALLBACK: &str = "https://client.example/callback";

/// Registers an app the way a Mastodon client does and answers its client id.
async fn register(browser: &mut Browser, redirect: &str) -> String {
    let reply = browser
        .request(
            Method::POST,
            "/api/v1/apps",
            Some(json!({
                "client_name": "Tuba",
                "redirect_uris": redirect,
                "scopes": "read write",
                "website": "https://client.example",
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "register: {}", reply.body);
    let app: Value = serde_json::from_str(&reply.body).expect("json");
    app["client_id"].as_str().expect("client id").to_owned()
}

fn authorize_url(client_id: &str, redirect: &str) -> String {
    format!(
        "/oauth/authorize?client_id={client_id}&redirect_uri={}&response_type=code&scope=read+write&state=xyz",
        redirect.replace(':', "%3A").replace('/', "%2F")
    )
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn a_guest_is_sent_to_sign_in_and_the_request_comes_back_with_them(db: PgPool) {
    let router = app(db);
    let mut guest = Browser::new(&router);
    let reply = guest
        .get("/oauth/authorize?client_id=abc&state=a%26b")
        .await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER, "{}", reply.body);
    assert_eq!(
        reply.header("location"),
        "/login?next=%2Foauth%2Fauthorize%3Fclient_id%3Dabc%26state%3Da%2526b"
    );
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn a_signed_in_user_sees_the_card_for_a_good_request(db: PgPool) {
    let router = app(db);
    let mut client = Browser::new(&router);
    let id = register(&mut client, CALLBACK).await;
    let mut user = Browser::new(&router);
    user.signup("oauth_a").await;

    let reply = user.get(&authorize_url(&id, CALLBACK)).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    let html = &reply.body;
    assert!(html.contains("<h1>Authorize an app</h1>"), "{html}");
    assert!(html.contains("data-state=\"ask\""));
    assert!(html.contains("data-app-name"));
    assert!(html.contains("Tuba"));
    assert!(html.contains("@oauth_a"));
    assert!(html.contains("data-scope=\"read\"") && html.contains("data-scope=\"write\""));
    assert!(html.contains("client.example"));
    assert!(html.contains("href=\"https://client.example/\""));
    assert!(
        html.contains("href=\"/login?add=1&amp;next=%2Foauth%2Fauthorize")
            || html.contains("href=\"/login?add=1&next=%2Foauth%2Fauthorize"),
        "the switch-account link keeps the request: {html}"
    );
    assert!(
        !html.contains("data-c=\"app-shell\""),
        "a bare page: {html}"
    );
    assert_eq!(reply.header("cache-control"), "no-store");
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn a_request_the_server_refuses_is_the_error_card(db: PgPool) {
    let router = app(db);
    let mut client = Browser::new(&router);
    let id = register(&mut client, CALLBACK).await;
    let mut user = Browser::new(&router);
    user.signup("oauth_b").await;

    for (url, text) in [
        (
            authorize_url(&id, "https://evil.example/steal"),
            "address the app didn’t register",
        ),
        (
            authorize_url("nope", CALLBACK),
            "This app isn’t registered here.",
        ),
        (
            authorize_url(&id, CALLBACK).replace("response_type=code", "response_type=token"),
            "kind of sign-in",
        ),
        ("/oauth/authorize".to_owned(), "The request is malformed"),
    ] {
        let reply = user.get(&url).await;
        assert_eq!(reply.status, StatusCode::OK, "{url}");
        let html = &reply.body;
        assert!(html.contains("data-state=\"failed\""), "{url}: {html}");
        assert!(html.contains("This request can’t be authorized"), "{url}");
        assert!(html.contains("Nothing was sent to the app."), "{url}");
        assert!(html.contains(text), "{url}: {html}");
        assert!(!html.contains("data-app-name"), "{url}");
    }
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn guests_read_the_developer_docs_and_the_legal_pages_on_pages_of_their_own(db: PgPool) {
    let mut guest = Browser::new(&app(db));
    for (path, h1, text) in [
        ("/developers", "Developers", "Authentication"),
        ("/terms", "Terms of Service", "1. Agreement"),
        ("/privacy", "Privacy Policy", "The short version"),
    ] {
        let reply = guest.get(path).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.body);
        let html = &reply.body;
        assert!(
            html.contains(&format!("<title>{h1} · tuita</title>")),
            "{path}"
        );
        assert!(html.contains("<main class=\"solo\""), "{path}: {html}");
        assert!(html.contains("<h1>"), "{path}");
        assert!(
            html.contains(text) || html.contains("1. Agreement"),
            "{path}"
        );
        assert!(!html.contains("data-c=\"app-shell\""), "{path}");
        assert_eq!(reply.header("cache-control"), "no-store");
    }
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn members_read_them_inside_the_shell(db: PgPool) {
    let mut user = Browser::new(&app(db));
    user.signup("oauth_c").await;
    for path in ["/developers", "/terms", "/privacy"] {
        let reply = user.get(path).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}");
        let html = &reply.body;
        assert!(html.contains("data-c=\"page-bar\""), "{path}: {html}");
        assert!(!html.contains("<main class=\"solo\""), "{path}");
    }
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn the_developer_docs_name_this_instance_and_list_every_endpoint(db: PgPool) {
    let mut guest = Browser::new(&app(db));
    let reply = guest.get("/developers").await;
    let html = &reply.body;
    assert!(
        html.contains("curl https://social.example/api/v1/me"),
        "{html}"
    );
    assert!(html.contains("/api/v1/timelines/following"));
    assert!(html.contains("/api/v1/conversations/{handle}"));
    assert_eq!(
        html.matches("<tr>").count(),
        1 + 22,
        "header row and 22 endpoints"
    );
}
