//! OAuth2 for Mastodon clients: the migration, app registration, the consent calls, the token
//! endpoint, revocation, the bearer rules for OAuth tokens and CORS.

mod common;

use std::collections::HashMap;

use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Method, Request, StatusCode};
use axum::middleware::from_fn_with_state;
use axum::routing::get;
use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use sqlx::PgPool;
use tower::ServiceExt as _;
use tuita_server::AppState;
use tuita_server::auth::bearer;
use tuita_server::mastodon::by_credential;
use url::{Url, form_urlencoded};
use uuid::Uuid;

use common::{App, Browser, Person, RawReply};

const CLIENT_ORIGIN: &str = "https://client.example";
/// RFC 7636 appendix B.
const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
const CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
const REDIRECT: &str = "https://client.example/callback";
const OOB: &str = "urn:ietf:wg:oauth:2.0:oob";

// ------------------------------------------------------------------ helpers

fn sha256(text: &str) -> Vec<u8> {
    Sha256::digest(text.as_bytes()).to_vec()
}

/// A script or web client on another origin: no cookies, its own address.
fn client(app: &App) -> Browser {
    let mut browser = app.browser_at([203, 0, 113, 7]);
    browser.origin = Some(CLIENT_ORIGIN.to_owned());
    browser
}

async fn call(
    browser: &mut Browser,
    method: Method,
    uri: &str,
    bearer: Option<&str>,
    body: Option<(&str, String)>,
) -> RawReply {
    let auth = bearer.map(|token| format!("Bearer {token}"));
    let mut headers: Vec<(&str, &str)> = Vec::new();
    if let Some(auth) = &auth {
        headers.push(("authorization", auth));
    }
    let (content_type, body) = match body {
        Some((content_type, body)) => (Some(content_type), Body::from(body)),
        None => (None, Body::empty()),
    };
    if let Some(content_type) = content_type {
        headers.push(("content-type", content_type));
    }
    browser.send_raw(method, uri, &headers, body).await
}

async fn get_at(browser: &mut Browser, uri: &str, bearer: Option<&str>) -> RawReply {
    call(browser, Method::GET, uri, bearer, None).await
}

async fn post_json(
    browser: &mut Browser,
    uri: &str,
    bearer: Option<&str>,
    body: &Value,
) -> RawReply {
    call(
        browser,
        Method::POST,
        uri,
        bearer,
        Some(("application/json", body.to_string())),
    )
    .await
}

async fn post_form(browser: &mut Browser, uri: &str, pairs: &[(&str, &str)]) -> RawReply {
    let body = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs)
        .finish();
    call(
        browser,
        Method::POST,
        uri,
        None,
        Some(("application/x-www-form-urlencoded", body)),
    )
    .await
}

/// `POST /api/v1/apps` as JSON and the registered app.
async fn register(client: &mut Browser, scopes: &str, redirect_uris: &str) -> Value {
    post_json(
        client,
        "/api/v1/apps",
        None,
        &json!({
            "client_name": "Test App",
            "redirect_uris": redirect_uris,
            "scopes": scopes,
            "website": "https://client.example",
        }),
    )
    .await
    .ok()
}

fn authorize_body(app: &Value, scope: &str, challenge: Option<&str>, approve: bool) -> Value {
    let mut body = json!({
        "clientId": app["client_id"],
        "redirectUri": REDIRECT,
        "scope": scope,
        "responseType": "code",
        "state": "st-1",
        "approve": approve,
    });
    if let Some(challenge) = challenge {
        body["codeChallenge"] = json!(challenge);
        body["codeChallengeMethod"] = json!("S256");
    }
    body
}

fn query_of(url: &str) -> HashMap<String, String> {
    Url::parse(url)
        .expect("url")
        .query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect()
}

/// Approves an authorization request as `user` and returns the code.
async fn get_code(user: &mut Person, app: &Value, scope: &str, challenge: Option<&str>) -> String {
    let reply = post_json(
        &mut user.browser,
        "/oauth/authorize",
        None,
        &authorize_body(app, scope, challenge, true),
    )
    .await
    .ok();
    let redirect = reply["redirect"].as_str().expect("redirect");
    assert!(redirect.starts_with(REDIRECT), "{redirect}");
    let query = query_of(redirect);
    assert_eq!(query["state"], "st-1");
    query["code"].clone()
}

async fn exchange(
    client: &mut Browser,
    app: &Value,
    code: &str,
    verifier: Option<&str>,
) -> RawReply {
    let mut pairs = vec![
        ("grant_type", "authorization_code"),
        ("client_id", app["client_id"].as_str().expect("id")),
        ("code", code),
        ("redirect_uri", REDIRECT),
    ];
    if let Some(verifier) = verifier {
        pairs.push(("code_verifier", verifier));
    }
    post_form(client, "/oauth/token", &pairs).await
}

async fn exchange_with_secret(client: &mut Browser, app: &Value, code: &str) -> RawReply {
    post_form(
        client,
        "/oauth/token",
        &[
            ("grant_type", "authorization_code"),
            ("client_id", app["client_id"].as_str().expect("id")),
            (
                "client_secret",
                app["client_secret"].as_str().expect("secret"),
            ),
            ("code", code),
            ("redirect_uri", REDIRECT),
        ],
    )
    .await
}

#[track_caller]
fn oauth_err(reply: &RawReply, status: StatusCode, code: &str) {
    assert_eq!(
        reply.status,
        status,
        "body: {}",
        String::from_utf8_lossy(&reply.bytes)
    );
    let body = reply.json();
    assert_eq!(body["error"], code, "{body}");
    assert!(
        body["error_description"]
            .as_str()
            .is_some_and(|d| !d.is_empty())
    );
    assert_eq!(reply.header("cache-control"), "no-store");
}

/// A token for `user` through the whole PKCE flow.
async fn oauth_token(app: &App, user: &mut Person, scopes: &str) -> (Value, String) {
    let mut client = client(app);
    let registered = register(&mut client, scopes, REDIRECT).await;
    let code = get_code(user, &registered, scopes, Some(CHALLENGE)).await;
    let token = exchange(&mut client, &registered, &code, Some(VERIFIER))
        .await
        .ok();
    (
        registered,
        token["access_token"].as_str().expect("token").to_owned(),
    )
}

// ------------------------------------------------------------------ the migration

#[sqlx::test(migrations = "./migrations")]
async fn the_migration_creates_the_oauth_tables_on_a_fresh_database(db: PgPool) {
    let tables: Vec<String> = sqlx::query_scalar(
        "select table_name::text from information_schema.tables
          where table_name in ('oauth_apps', 'oauth_codes') order by 1",
    )
    .fetch_all(&db)
    .await
    .expect("tables");
    assert_eq!(tables, ["oauth_apps", "oauth_codes"]);
    let nullable: String = sqlx::query_scalar(
        "select is_nullable::text from information_schema.columns
          where table_name = 'api_tokens' and column_name = 'user_id'",
    )
    .fetch_one(&db)
    .await
    .expect("column");
    assert_eq!(nullable, "YES");
}

#[sqlx::test(migrations = false)]
async fn the_migration_upgrades_the_m5_schema_and_keeps_its_tokens(db: PgPool) {
    tuita_server::MIGRATOR
        .run_to(6, &db)
        .await
        .expect("migrations up to M5");
    let user = Uuid::now_v7();
    sqlx::query("insert into users (id, handle, display_name) values ($1, 'ana', 'Ana')")
        .bind(user)
        .execute(&db)
        .await
        .expect("user");
    let token = Uuid::now_v7();
    sqlx::query(
        "insert into api_tokens (id, user_id, name, scopes, token_hash, prefix)
         values ($1, $2, 'old', '{read}', '\\x01', 'tuita_abcdef')",
    )
    .bind(token)
    .bind(user)
    .execute(&db)
    .await
    .expect("an M5 token");

    tuita_server::migrate(&db).await.expect("0007 applies");

    let kept: (Uuid, Option<Uuid>, Option<Uuid>) =
        sqlx::query_as("select id, user_id, app_id from api_tokens")
            .fetch_one(&db)
            .await
            .expect("kept");
    assert_eq!(kept, (token, Some(user), None));
    // An app-only token is allowed now, an ownerless one is not.
    let app = Uuid::now_v7();
    sqlx::query(
        "insert into oauth_apps (id, client_id, client_secret_hash, name, redirect_uris, scopes)
         values ($1, 'cid', '\\x02', 'App', '{https://a.example/}', '{read}')",
    )
    .bind(app)
    .execute(&db)
    .await
    .expect("app");
    let insert = |user: Option<Uuid>, app: Option<Uuid>, hash: &'static str| {
        let db = db.clone();
        async move {
            sqlx::query(
                "insert into api_tokens (id, user_id, name, scopes, token_hash, prefix, app_id)
                 values ($1, $2, 'n', '{read}', decode($3, 'hex'), 'tuita_x', $4)",
            )
            .bind(Uuid::now_v7())
            .bind(user)
            .bind(hash)
            .bind(app)
            .execute(&db)
            .await
        }
    };
    insert(None, Some(app), "03").await.expect("app-only token");
    insert(None, None, "04")
        .await
        .expect_err("a token needs an owner");
    // Deleting the app deletes its tokens.
    sqlx::query("delete from oauth_apps")
        .execute(&db)
        .await
        .expect("delete");
    let left: i64 = sqlx::query_scalar("select count(*) from api_tokens")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(left, 1);
}

// ------------------------------------------------------------------ app registration

#[sqlx::test(migrations = "./migrations")]
async fn an_app_registers_by_json_or_form_and_only_the_secret_hash_is_kept(db: PgPool) {
    let app = common::app(db.clone());
    let mut client = client(&app);
    let json = post_json(
        &mut client,
        "/api/v1/apps",
        None,
        &json!({
            "client_name": "  Phanpy ",
            "redirect_uris": [REDIRECT, "tusky://oauth-callback"],
            "scopes": "write read follow push",
            "website": "https://phanpy.social",
        }),
    )
    .await
    .ok();
    assert_eq!(json["name"], "Phanpy");
    assert_eq!(json["website"], "https://phanpy.social");
    assert_eq!(json["scopes"], json!(["read", "write", "follow", "push"]));
    assert_eq!(
        json["redirect_uris"],
        json!([REDIRECT, "tusky://oauth-callback"])
    );
    assert_eq!(
        json["redirect_uri"],
        format!("{REDIRECT}\ntusky://oauth-callback")
    );
    assert_eq!(json["client_secret_expires_at"], 0);
    assert_eq!(json["vapid_key"], "");
    let secret = json["client_secret"].as_str().expect("secret");
    assert!(secret.starts_with("tuita_") && secret.len() == 6 + 43);
    assert!(
        json["id"]
            .as_str()
            .is_some_and(|id| Uuid::parse_str(id).is_ok())
    );
    let stored: Vec<u8> =
        sqlx::query_scalar("select client_secret_hash from oauth_apps where client_id = $1")
            .bind(json["client_id"].as_str().expect("client id"))
            .fetch_one(&db)
            .await
            .expect("hash");
    assert_eq!(stored, sha256(secret));

    // A form with Rails-style arrays, no website, default scope.
    let form = post_form(
        &mut client,
        "/api/v1/apps",
        &[
            ("client_name", "Tusky"),
            ("redirect_uris[]", "tusky://oauth-callback"),
            ("redirect_uris[]", OOB),
        ],
    )
    .await
    .ok();
    assert_eq!(form["scopes"], json!(["read"]));
    assert_eq!(form["website"], Value::Null);
    assert_eq!(
        form["redirect_uris"],
        json!(["tusky://oauth-callback", OOB])
    );
    assert_ne!(form["client_id"], json["client_id"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn invalid_and_dangerous_registrations_are_refused(db: PgPool) {
    let app = common::app(db.clone());
    let mut client = client(&app);
    let refused = |reply: RawReply, message: &str| {
        assert_eq!(
            reply.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{}",
            String::from_utf8_lossy(&reply.bytes)
        );
        let error = reply.json()["error"].as_str().expect("error").to_owned();
        assert!(error.starts_with("Validation failed: "), "{error}");
        assert!(error.contains(message), "{error} should mention {message}");
    };
    let try_body = |body: Value| {
        let mut client = self::client(&app);
        async move { post_json(&mut client, "/api/v1/apps", None, &body).await }
    };
    for uri in [
        "javascript://%0aalert(1)",
        "data://text/html,x",
        "file:///etc/passwd",
        "http://example.com/cb",
        "https://example.com/cb#fragment",
        "https://user:pw@example.com/cb",
        "relative/path",
        "com.example:/cb",
    ] {
        refused(
            try_body(json!({"client_name": "A", "redirect_uris": uri})).await,
            "Redirect URI",
        );
    }
    refused(
        try_body(json!({"client_name": "A"})).await,
        "Redirect URI can't be blank",
    );
    refused(
        try_body(json!({"client_name": "A", "redirect_uris": [REDIRECT, "javascript://x"]})).await,
        "Redirect URI",
    );
    refused(
        try_body(json!({"client_name": "", "redirect_uris": REDIRECT})).await,
        "Name can't be blank",
    );
    refused(
        try_body(json!({"client_name": "x".repeat(101), "redirect_uris": REDIRECT})).await,
        "Name is invalid",
    );
    refused(
        try_body(json!({"client_name": "A", "redirect_uris": REDIRECT, "website": "javascript:alert(1)"})).await,
        "Website",
    );
    for scopes in ["read:statuses", "read admin", "messages"] {
        refused(
            try_body(json!({"client_name": "A", "redirect_uris": REDIRECT, "scopes": scopes}))
                .await,
            "Scopes",
        );
    }
    let too_many: Vec<String> = (0..11)
        .map(|n| format!("https://c{n}.example/cb"))
        .collect();
    refused(
        try_body(json!({"client_name": "A", "redirect_uris": too_many})).await,
        "too many",
    );
    let stored: i64 = sqlx::query_scalar("select count(*) from oauth_apps")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(stored, 0, "nothing was stored");
    // A malformed body is a 400, not a 500.
    let bad = call(
        &mut client,
        Method::POST,
        "/api/v1/apps",
        None,
        Some(("application/json", "{nope".to_owned())),
    )
    .await;
    bad.err(StatusCode::BAD_REQUEST, "Invalid JSON.");
}

#[sqlx::test(migrations = "./migrations")]
async fn registration_and_the_token_endpoint_are_rate_limited_by_address(db: PgPool) {
    let app = common::app(db);
    let mut client = client(&app);
    for _ in 0..30 {
        register(&mut client, "read", REDIRECT).await;
    }
    let limited = post_json(
        &mut client,
        "/api/v1/apps",
        None,
        &json!({"client_name": "A", "redirect_uris": REDIRECT}),
    )
    .await;
    assert_eq!(limited.status, StatusCode::TOO_MANY_REQUESTS);
    assert!(!limited.header("retry-after").is_empty());
    // Another address has its own allowance.
    let mut other = app.browser_at([203, 0, 113, 99]);
    register(&mut other, "read", REDIRECT).await;

    let mut caller = app.browser_at([203, 0, 113, 8]);
    for _ in 0..60 {
        let reply = post_form(&mut caller, "/oauth/token", &[("grant_type", "password")]).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST);
        assert_ne!(reply.status, StatusCode::TOO_MANY_REQUESTS);
    }
    let limited = post_form(&mut caller, "/oauth/token", &[("grant_type", "password")]).await;
    assert_eq!(limited.status, StatusCode::TOO_MANY_REQUESTS);
    let limited = post_form(&mut caller, "/oauth/revoke", &[("token", "x")]).await;
    assert_eq!(limited.status, StatusCode::TOO_MANY_REQUESTS);
}

// ------------------------------------------------------------------ consent and authorize

#[sqlx::test(migrations = "./migrations")]
async fn the_consent_card_gets_the_app_the_granted_scopes_and_where_the_code_goes(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let registered = register(
        &mut client,
        "read write follow push",
        "https://redirect.example:8443/cb?tenant=1",
    )
    .await;
    let id = registered["client_id"].as_str().expect("id");
    let consent = |extra: &[(&str, &str)]| {
        let mut pairs = vec![("client_id", id), ("response_type", "code")];
        pairs.extend_from_slice(extra);
        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(pairs)
            .finish();
        format!("/oauth/authorization?{query}")
    };

    let card = ana
        .browser
        .get(&consent(&[
            ("redirect_uri", "https://redirect.example:8443/cb?tenant=1"),
            ("scope", "read write follow push"),
            ("code_challenge", CHALLENGE),
            ("code_challenge_method", "S256"),
        ]))
        .await
        .ok();
    assert_eq!(
        card,
        json!({
            "appName": "Test App",
            "website": "https://client.example",
            "scopes": ["read", "write"],
            "redirectUri": "https://redirect.example:8443/cb?tenant=1",
            "redirectHost": "redirect.example:8443",
            "oob": false,
            "pkce": true,
        })
    );
    // With one registered URI the parameter may be left out; read is the default scope.
    let card = ana.browser.get(&consent(&[])).await.ok();
    assert_eq!(card["scopes"], json!(["read"]));
    assert_eq!(card["pkce"], false);

    let refused = |reply: common::Reply, code: &str| {
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.body);
        assert_eq!(reply.json()["error"], code, "{}", reply.body);
        assert!(reply.json()["error_description"].is_string());
    };
    refused(
        ana.browser.get("/oauth/authorization").await,
        "invalid_request",
    );
    refused(
        ana.browser
            .get("/oauth/authorization?client_id=nope&response_type=code")
            .await,
        "invalid_client",
    );
    refused(
        ana.browser
            .get(&consent(&[(
                "redirect_uri",
                "https://redirect.example:8443/cb",
            )]))
            .await,
        "invalid_request",
    );
    refused(
        ana.browser
            .get(&consent(&[(
                "redirect_uri",
                "https://redirect.example:8443/cb?tenant=1&x=1",
            )]))
            .await,
        "invalid_request",
    );
    refused(
        ana.browser
            .get(&consent(&[("scope", "read:statuses")]))
            .await,
        "invalid_scope",
    );
    refused(
        ana.browser.get(&consent(&[("scope", "push")])).await,
        "invalid_scope",
    );
    refused(
        ana.browser.get(&consent(&[("scope", "admin")])).await,
        "invalid_scope",
    );
    refused(
        ana.browser
            .get(&consent(&[
                ("code_challenge", CHALLENGE),
                ("code_challenge_method", "plain"),
            ]))
            .await,
        "invalid_request",
    );
    refused(
        ana.browser
            .get(&consent(&[("code_challenge", CHALLENGE)]))
            .await,
        "invalid_request",
    );
    refused(
        ana.browser
            .get(&consent(&[
                ("code_challenge", "short"),
                ("code_challenge_method", "S256"),
            ]))
            .await,
        "invalid_request",
    );
    refused(
        ana.browser
            .get(&consent(&[("code_challenge_method", "S256")]))
            .await,
        "invalid_request",
    );
    let mut wrong_type = consent(&[]);
    wrong_type = wrong_type.replace("response_type=code", "response_type=token");
    refused(
        ana.browser.get(&wrong_type).await,
        "unsupported_response_type",
    );
    refused(
        ana.browser
            .get(&consent(&[("state", &"s".repeat(2001))]))
            .await,
        "invalid_request",
    );
    // The app registered for read only cannot be asked for write.
    let narrow = register(&mut client, "read", REDIRECT).await;
    let narrow_id = narrow["client_id"].as_str().expect("id");
    let reply = ana
        .browser
        .get(&format!(
            "/oauth/authorization?client_id={narrow_id}&response_type=code&scope=read+write"
        ))
        .await;
    refused(reply, "invalid_scope");
    // Two registered URIs and none sent: ambiguous.
    let two = register(&mut client, "read", &format!("{REDIRECT} {OOB}")).await;
    let two_id = two["client_id"].as_str().expect("id");
    refused(
        ana.browser
            .get(&format!(
                "/oauth/authorization?client_id={two_id}&response_type=code"
            ))
            .await,
        "invalid_request",
    );
    let oob = ana
        .browser
        .get(&format!(
            "/oauth/authorization?client_id={two_id}&response_type=code&redirect_uri={}",
            form_urlencoded::byte_serialize(OOB.as_bytes()).collect::<String>()
        ))
        .await
        .ok();
    assert_eq!(oob["oob"], true);
    assert_eq!(oob["redirectHost"], "");
}

#[sqlx::test(migrations = "./migrations")]
async fn the_consent_info_needs_the_session_cookie_and_refuses_tokens(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let (registered, token) = oauth_token(&app, &mut ana, "read write").await;
    let uri = format!(
        "/api/v1/oauth/authorization?client_id={}&response_type=code",
        registered["client_id"].as_str().expect("id")
    );
    let mut script = client(&app);
    // No cookie: sign in required. A token, even a valid one: refused like every cookie-only path.
    get_at(&mut script, &uri, None)
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");
    get_at(&mut script, &uri, Some(&token)).await.err(
        StatusCode::FORBIDDEN,
        "API tokens can\u{2019}t be used for this. Sign in on the website instead.",
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn approving_redirects_with_a_code_and_the_state_and_keeps_the_registered_query(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let registered = register(
        &mut client,
        "read write",
        "https://app.example/cb?tenant=a%20b",
    )
    .await;
    let reply = post_json(
        &mut ana.browser,
        "/oauth/authorize",
        None,
        &json!({
            "clientId": registered["client_id"],
            "redirectUri": "https://app.example/cb?tenant=a%20b",
            "scope": "read write",
            "responseType": "code",
            "state": "x y&z=1#2",
            "codeChallenge": CHALLENGE,
            "codeChallengeMethod": "S256",
            "approve": true,
        }),
    )
    .await
    .ok();
    assert_eq!(reply["code"], Value::Null);
    let redirect = reply["redirect"].as_str().expect("redirect");
    assert!(
        redirect.starts_with("https://app.example/cb?tenant=a%20b&code="),
        "{redirect}"
    );
    let query = query_of(redirect);
    assert_eq!(query["tenant"], "a b");
    assert_eq!(query["state"], "x y&z=1#2");
    let code = &query["code"];
    assert_eq!(code.len(), 43);

    // Only the hash of the code is stored, for this user and app, expiring in about ten minutes.
    let row: (Vec<u8>, Uuid, Vec<String>, String, Option<String>, f64) = sqlx::query_as(
        "select code_hash, user_id, scopes, redirect_uri, code_challenge,
                extract(epoch from expires_at - now())::float8 from oauth_codes",
    )
    .fetch_one(&db)
    .await
    .expect("code row");
    assert_eq!(row.0, sha256(code));
    assert_eq!(row.1, ana.id);
    assert_eq!(row.2, ["read", "write"]);
    assert_eq!(row.3, "https://app.example/cb?tenant=a%20b");
    assert_eq!(row.4.as_deref(), Some(CHALLENGE));
    assert!((590.0..=600.0).contains(&row.5), "expires in {}s", row.5);
    let anywhere: i64 = sqlx::query_scalar(
        "select count(*) from oauth_codes where position($1 in redirect_uri) > 0",
    )
    .bind(code)
    .fetch_one(&db)
    .await
    .expect("count");
    assert_eq!(anywhere, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn denying_redirects_with_access_denied_and_makes_no_code(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let registered = register(&mut client, "read", REDIRECT).await;
    let reply = post_json(
        &mut ana.browser,
        "/oauth/authorize",
        None,
        &authorize_body(&registered, "read", None, false),
    )
    .await
    .ok();
    let redirect = reply["redirect"].as_str().expect("redirect");
    let query = query_of(redirect);
    assert!(redirect.starts_with(REDIRECT));
    assert_eq!(query["error"], "access_denied");
    assert_eq!(query["state"], "st-1");
    assert!(!query.contains_key("code"));
    assert_eq!(reply["code"], Value::Null);
    let codes: i64 = sqlx::query_scalar("select count(*) from oauth_codes")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(codes, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_out_of_band_uri_shows_the_code_instead_of_redirecting(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let registered = register(&mut client, "read", OOB).await;
    let mut body = authorize_body(&registered, "read", Some(CHALLENGE), true);
    body["redirectUri"] = json!(OOB);
    let reply = post_json(&mut ana.browser, "/oauth/authorize", None, &body)
        .await
        .ok();
    assert_eq!(reply["redirect"], Value::Null);
    let code = reply["code"].as_str().expect("code").to_owned();
    let exchange_body = [
        ("grant_type", "authorization_code"),
        ("client_id", registered["client_id"].as_str().expect("id")),
        ("code", &code),
        ("redirect_uri", OOB),
        ("code_verifier", VERIFIER),
    ];
    let token = post_form(&mut client, "/oauth/token", &exchange_body)
        .await
        .ok();
    assert_eq!(token["scope"], "read");
    // A denial shows nothing to redirect to.
    body["approve"] = json!(false);
    let reply = post_json(&mut ana.browser, "/oauth/authorize", None, &body)
        .await
        .ok();
    assert_eq!(reply, json!({"redirect": null, "code": null}));
}

#[sqlx::test(migrations = "./migrations")]
async fn authorize_needs_the_session_cookie_the_origin_and_no_bearer(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let (registered, token) = oauth_token(&app, &mut ana, "read write").await;
    let body = authorize_body(&registered, "read", Some(CHALLENGE), true);

    // No cookie: signed out. A bearer alone is no way in.
    let mut script = client(&app);
    script.origin = None;
    post_json(&mut script, "/oauth/authorize", None, &body)
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");
    post_json(&mut script, "/oauth/authorize", Some(&token), &body)
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");

    // The session plus a bearer header is refused too.
    post_json(&mut ana.browser, "/oauth/authorize", Some(&token), &body)
        .await
        .err(
            StatusCode::FORBIDDEN,
            "API tokens can\u{2019}t be used for this. Sign in on the website instead.",
        );

    // The CSRF guard: a cookie without the instance's Origin is refused, so a page on another
    // origin cannot approve an app for the signed-in user.
    let before: i64 = sqlx::query_scalar("select count(*) from oauth_codes")
        .fetch_one(&db)
        .await
        .expect("count");
    ana.browser.origin = Some("https://evil.example".to_owned());
    post_json(&mut ana.browser, "/oauth/authorize", None, &body)
        .await
        .err(StatusCode::FORBIDDEN, "bad origin");
    ana.browser.origin = None;
    post_json(&mut ana.browser, "/oauth/authorize", None, &body)
        .await
        .err(StatusCode::FORBIDDEN, "bad origin");
    let after: i64 = sqlx::query_scalar("select count(*) from oauth_codes")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(before, after);

    // And it is no CORS endpoint.
    ana.browser.origin = Some(common::ORIGIN.to_owned());
    let reply = post_json(&mut ana.browser, "/oauth/authorize", None, &body).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert!(reply.headers.get("access-control-allow-origin").is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn authorize_never_redirects_an_unregistered_uri_or_an_invalid_request(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let registered = register(&mut client, "read", REDIRECT).await;
    for (patch, code) in [
        (
            json!({"redirectUri": "https://evil.example/cb"}),
            "invalid_request",
        ),
        (
            json!({"redirectUri": "https://client.example/callback/"}),
            "invalid_request",
        ),
        (
            json!({"redirectUri": "HTTPS://client.example/callback"}),
            "invalid_request",
        ),
        (
            json!({"redirectUri": "https://client.example/callbackx"}),
            "invalid_request",
        ),
        (json!({"clientId": "unknown"}), "invalid_client"),
        (json!({"scope": "read write"}), "invalid_scope"),
        (json!({"scope": "read:accounts"}), "invalid_scope"),
        (
            json!({"responseType": "token"}),
            "unsupported_response_type",
        ),
        (
            json!({"codeChallengeMethod": "plain", "codeChallenge": CHALLENGE}),
            "invalid_request",
        ),
    ] {
        let mut body = authorize_body(&registered, "read", None, true);
        for (key, value) in patch.as_object().expect("patch") {
            body[key] = value.clone();
        }
        let reply = post_json(&mut ana.browser, "/oauth/authorize", None, &body).await;
        oauth_err(&reply, StatusCode::BAD_REQUEST, code);
        assert!(reply.json().get("redirect").is_none(), "{patch}");
    }
    let codes: i64 = sqlx::query_scalar("select count(*) from oauth_codes")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(codes, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_web_client_serves_the_consent_page_and_the_rest_of_oauth_is_backend(db: PgPool) {
    let app = common::app(db);
    let mut visitor = app.browser();
    let page = get_at(
        &mut visitor,
        "/oauth/authorize?client_id=x&response_type=code",
        None,
    )
    .await;
    assert_eq!(page.status, StatusCode::OK);
    assert_eq!(String::from_utf8_lossy(&page.bytes), "web");
    assert_eq!(page.header("x-frame-options"), "DENY");
    let missing = get_at(&mut visitor, "/oauth/nothing", None).await;
    missing.err(StatusCode::NOT_FOUND, "not found");
}

// ------------------------------------------------------------------ the token endpoint

#[sqlx::test(migrations = "./migrations")]
async fn a_code_with_a_pkce_challenge_becomes_a_token_that_reads_me(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let registered = register(&mut client, "read write follow push", REDIRECT).await;
    let code = get_code(
        &mut ana,
        &registered,
        "read write follow push",
        Some(CHALLENGE),
    )
    .await;

    let reply = exchange(&mut client, &registered, &code, Some(VERIFIER)).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.header("cache-control"), "no-store");
    assert_eq!(reply.header("pragma"), "no-cache");
    assert_eq!(reply.header("access-control-allow-origin"), "*");
    let token = reply.json();
    let secret = token["access_token"].as_str().expect("token");
    assert!(secret.starts_with("tuita_") && secret.len() == 6 + 43);
    assert_eq!(token["token_type"], "Bearer");
    assert_eq!(token["scope"], "read write");
    let created = token["created_at"].as_i64().expect("created_at");
    assert!((time_now() - created).abs() < 60);

    // The secret reads the account through the Mastodon API.
    let mut script = self::client(&app);
    let me = get_at(
        &mut script,
        "/api/v1/accounts/verify_credentials",
        Some(secret),
    )
    .await
    .ok();
    assert_eq!(me["username"], "ana");

    // Only the hash is stored; the token carries the app and the granted scopes.
    let row: (Vec<u8>, Vec<String>, Option<Uuid>, String, Uuid) =
        sqlx::query_as("select token_hash, scopes, app_id, name, user_id from api_tokens")
            .fetch_one(&db)
            .await
            .expect("token row");
    assert_eq!(row.0, sha256(secret));
    assert_eq!(row.1, ["read", "write"]);
    assert_eq!(
        row.2.map(|id| id.to_string()).as_deref(),
        registered["id"].as_str()
    );
    assert_eq!(row.3, "Test App");
    assert_eq!(row.4, ana.id);

    // The user sees it in the token list, under the app's name, and can delete it there.
    let listed = ana.browser.get("/tokens").await.ok();
    assert_eq!(listed[0]["name"], "Test App");
    assert_eq!(listed[0]["scopes"], json!(["read", "write"]));
    assert_eq!(listed[0]["prefix"], secret[..12]);
    let id = listed[0]["id"].as_str().expect("id").to_owned();
    ana.browser.delete(&format!("/tokens/{id}")).await.ok();
    get_at(
        &mut script,
        "/api/v1/accounts/verify_credentials",
        Some(secret),
    )
    .await
    .err(StatusCode::UNAUTHORIZED, "Invalid or revoked API token.");
}

fn time_now() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}

#[sqlx::test(migrations = "./migrations")]
async fn a_code_without_a_challenge_needs_the_client_secret(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let registered = register(&mut client, "read", REDIRECT).await;

    // No secret: refused, and the refusal leaves the code usable.
    let code = get_code(&mut ana, &registered, "read", None).await;
    oauth_err(
        &exchange(&mut client, &registered, &code, None).await,
        StatusCode::BAD_REQUEST,
        "invalid_grant",
    );
    // A verifier cannot stand in for a challenge that was never made.
    oauth_err(
        &exchange(&mut client, &registered, &code, Some(VERIFIER)).await,
        StatusCode::BAD_REQUEST,
        "invalid_grant",
    );
    // A wrong secret is a client error, and the code survives that too.
    let wrong = post_form(
        &mut client,
        "/oauth/token",
        &[
            ("grant_type", "authorization_code"),
            ("client_id", registered["client_id"].as_str().expect("id")),
            ("client_secret", "tuita_wrong"),
            ("code", &code),
            ("redirect_uri", REDIRECT),
        ],
    )
    .await;
    oauth_err(&wrong, StatusCode::UNAUTHORIZED, "invalid_client");
    assert_eq!(wrong.header("www-authenticate"), "Basic realm=\"tuita\"");
    let good = exchange_with_secret(&mut client, &registered, &code)
        .await
        .ok();
    assert_eq!(good["scope"], "read");
}

#[sqlx::test(migrations = "./migrations")]
async fn the_client_may_authenticate_with_http_basic(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let registered = register(&mut client, "read", REDIRECT).await;
    let code = get_code(&mut ana, &registered, "read", None).await;
    let basic = STANDARD.encode(format!(
        "{}:{}",
        registered["client_id"].as_str().expect("id"),
        registered["client_secret"].as_str().expect("secret")
    ));
    let body = form_urlencoded::Serializer::new(String::new())
        .extend_pairs([
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("redirect_uri", REDIRECT),
        ])
        .finish();
    let reply = client
        .send_raw(
            Method::POST,
            "/oauth/token",
            &[
                ("authorization", &format!("Basic {basic}")),
                ("content-type", "application/x-www-form-urlencoded"),
            ],
            Body::from(body),
        )
        .await;
    assert_eq!(reply.ok()["token_type"], "Bearer");
}

#[sqlx::test(migrations = "./migrations")]
async fn the_token_endpoint_takes_json_too(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let registered = register(&mut client, "read", REDIRECT).await;
    let code = get_code(&mut ana, &registered, "read", Some(CHALLENGE)).await;
    let reply = post_json(
        &mut client,
        "/oauth/token",
        None,
        &json!({
            "grant_type": "authorization_code",
            "client_id": registered["client_id"],
            "client_secret": registered["client_secret"],
            "code": code,
            "redirect_uri": REDIRECT,
            "code_verifier": VERIFIER,
        }),
    )
    .await;
    assert_eq!(reply.ok()["scope"], "read");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_wrong_code_a_wrong_verifier_or_a_wrong_redirect_is_invalid_grant(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let registered = register(
        &mut client,
        "read",
        &format!("{REDIRECT} https://client.example/other"),
    )
    .await;
    let code = get_code(&mut ana, &registered, "read", Some(CHALLENGE)).await;

    for wrong in ["", "nope", &"a".repeat(43), &"a".repeat(500)] {
        let reply = if wrong.is_empty() {
            post_form(
                &mut client,
                "/oauth/token",
                &[
                    ("grant_type", "authorization_code"),
                    ("client_id", registered["client_id"].as_str().expect("id")),
                    ("redirect_uri", REDIRECT),
                ],
            )
            .await
        } else {
            exchange(&mut client, &registered, wrong, Some(VERIFIER)).await
        };
        let expected = if wrong.is_empty() {
            "invalid_request"
        } else {
            "invalid_grant"
        };
        oauth_err(&reply, StatusCode::BAD_REQUEST, expected);
    }
    // Wrong verifier, no verifier, a malformed verifier, the plain challenge as a verifier.
    for verifier in [
        Some("a".repeat(43)),
        None,
        Some("short".to_owned()),
        Some(CHALLENGE.to_owned()),
        Some(format!("{VERIFIER}x")),
    ] {
        let reply = exchange(&mut client, &registered, &code, verifier.as_deref()).await;
        oauth_err(&reply, StatusCode::BAD_REQUEST, "invalid_grant");
    }
    // The other registered URI is not the one the code was made for; neither is a variant.
    for redirect in [
        "https://client.example/other",
        "https://client.example/callback/",
        "",
    ] {
        let mut pairs = vec![
            ("grant_type", "authorization_code"),
            ("client_id", registered["client_id"].as_str().expect("id")),
            ("code", code.as_str()),
            ("code_verifier", VERIFIER),
        ];
        if !redirect.is_empty() {
            pairs.push(("redirect_uri", redirect));
        }
        oauth_err(
            &post_form(&mut client, "/oauth/token", &pairs).await,
            StatusCode::BAD_REQUEST,
            "invalid_grant",
        );
    }
    // None of those burned the code.
    let ok = exchange(&mut client, &registered, &code, Some(VERIFIER)).await;
    assert_eq!(ok.status, StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_code_belongs_to_the_app_it_was_made_for(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let first = register(&mut client, "read", REDIRECT).await;
    let second = register(&mut client, "read", REDIRECT).await;
    let code = get_code(&mut ana, &first, "read", Some(CHALLENGE)).await;
    oauth_err(
        &exchange(&mut client, &second, &code, Some(VERIFIER)).await,
        StatusCode::BAD_REQUEST,
        "invalid_grant",
    );
    exchange(&mut client, &first, &code, Some(VERIFIER))
        .await
        .ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn an_expired_code_is_refused(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let registered = register(&mut client, "read", REDIRECT).await;
    let code = get_code(&mut ana, &registered, "read", Some(CHALLENGE)).await;
    sqlx::query("update oauth_codes set expires_at = now() - interval '1 second'")
        .execute(&db)
        .await
        .expect("expire");
    oauth_err(
        &exchange(&mut client, &registered, &code, Some(VERIFIER)).await,
        StatusCode::BAD_REQUEST,
        "invalid_grant",
    );
    let tokens: i64 = sqlx::query_scalar("select count(*) from api_tokens")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(tokens, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn reusing_a_code_is_refused_and_revokes_the_token_it_made(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let registered = register(&mut client, "read write", REDIRECT).await;
    let code = get_code(&mut ana, &registered, "read write", Some(CHALLENGE)).await;
    let first = exchange(&mut client, &registered, &code, Some(VERIFIER))
        .await
        .ok();
    let secret = first["access_token"].as_str().expect("token");
    let mut script = self::client(&app);
    get_at(
        &mut script,
        "/api/v1/accounts/verify_credentials",
        Some(secret),
    )
    .await
    .ok();

    let again = exchange(&mut client, &registered, &code, Some(VERIFIER)).await;
    oauth_err(&again, StatusCode::BAD_REQUEST, "invalid_grant");
    get_at(
        &mut script,
        "/api/v1/accounts/verify_credentials",
        Some(secret),
    )
    .await
    .err(StatusCode::UNAUTHORIZED, "Invalid or revoked API token.");
    // And the code stays dead, also with the secret instead of the verifier, and once expired.
    oauth_err(
        &exchange_with_secret(&mut client, &registered, &code).await,
        StatusCode::BAD_REQUEST,
        "invalid_grant",
    );
    sqlx::query("update oauth_codes set expires_at = now() - interval '2 hours'")
        .execute(&db)
        .await
        .expect("expire");
    oauth_err(
        &exchange(&mut client, &registered, &code, Some(VERIFIER)).await,
        StatusCode::BAD_REQUEST,
        "invalid_grant",
    );
    let tokens: i64 = sqlx::query_scalar("select count(*) from api_tokens")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(tokens, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn two_exchanges_of_one_code_at_once_leave_no_live_token(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let registered = register(&mut client, "read", REDIRECT).await;
    let code = get_code(&mut ana, &registered, "read", Some(CHALLENGE)).await;
    let mut other = app.browser_at([203, 0, 113, 9]);
    let (a, b) = tokio::join!(
        exchange(&mut client, &registered, &code, Some(VERIFIER)),
        exchange(&mut other, &registered, &code, Some(VERIFIER)),
    );
    let mut statuses = [a.status.as_u16(), b.status.as_u16()];
    statuses.sort_unstable();
    assert_eq!(statuses, [200, 400], "exactly one exchange wins");
    // The loser saw the code used and took the winner's token back.
    let tokens: i64 = sqlx::query_scalar("select count(*) from api_tokens")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(tokens, 0);
    let used: i64 =
        sqlx::query_scalar("select count(*) from oauth_codes where used_at is not null")
            .fetch_one(&db)
            .await
            .expect("count");
    assert_eq!(used, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn scopes_are_limited_to_what_the_app_registered(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let read_only = register(&mut client, "read", REDIRECT).await;
    for scope in [
        "read write",
        "write",
        "follow",
        "read:statuses",
        "write:statuses",
        "push",
        "admin",
    ] {
        let reply = post_json(
            &mut ana.browser,
            "/oauth/authorize",
            None,
            &authorize_body(&read_only, scope, None, true),
        )
        .await;
        oauth_err(&reply, StatusCode::BAD_REQUEST, "invalid_scope");
    }
    // `follow` is `write`; `push` is dropped; `read` alone stays read.
    let wide = register(&mut client, "read write follow push", REDIRECT).await;
    for (asked, granted) in [
        ("follow", "write"),
        ("read follow", "read write"),
        ("write push", "write"),
        ("read", "read"),
        ("", "read"),
    ] {
        let code = get_code(&mut ana, &wide, asked, Some(CHALLENGE)).await;
        let token = exchange(&mut client, &wide, &code, Some(VERIFIER))
            .await
            .ok();
        assert_eq!(token["scope"], granted, "{asked}");
    }
    // A write-only token cannot read, a read-only token cannot write.
    let (_, read_token) = oauth_token(&app, &mut ana, "read").await;
    let (_, write_token) = oauth_token(&app, &mut ana, "write").await;
    let mut script = self::client(&app);
    get_at(
        &mut script,
        "/api/v1/accounts/verify_credentials",
        Some(&read_token),
    )
    .await
    .ok();
    post_json(
        &mut script,
        "/api/v1/statuses",
        Some(&read_token),
        &json!({"status": "x"}),
    )
    .await
    .err(
        StatusCode::FORBIDDEN,
        "This token doesn\u{2019}t have the \u{201c}write\u{201d} scope.",
    );
    post_json(
        &mut script,
        "/api/v1/statuses",
        Some(&write_token),
        &json!({"status": "x"}),
    )
    .await
    .ok();
    get_at(
        &mut script,
        "/api/v1/accounts/verify_credentials",
        Some(&write_token),
    )
    .await
    .err(
        StatusCode::FORBIDDEN,
        "This token doesn\u{2019}t have the \u{201c}read\u{201d} scope.",
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn only_two_grant_types_exist(db: PgPool) {
    let app = common::app(db.clone());
    let mut client = client(&app);
    let registered = register(&mut client, "read", REDIRECT).await;
    let id = registered["client_id"].as_str().expect("id");
    let secret = registered["client_secret"].as_str().expect("secret");
    for grant in [
        "password",
        "refresh_token",
        "implicit",
        "urn:ietf:params:oauth:grant-type:device_code",
    ] {
        let reply = post_form(
            &mut client,
            "/oauth/token",
            &[
                ("grant_type", grant),
                ("client_id", id),
                ("client_secret", secret),
            ],
        )
        .await;
        oauth_err(&reply, StatusCode::BAD_REQUEST, "unsupported_grant_type");
    }
    oauth_err(
        &post_form(&mut client, "/oauth/token", &[("client_id", id)]).await,
        StatusCode::BAD_REQUEST,
        "invalid_request",
    );
    oauth_err(
        &post_form(
            &mut client,
            "/oauth/token",
            &[("grant_type", "authorization_code")],
        )
        .await,
        StatusCode::BAD_REQUEST,
        "invalid_request",
    );
    oauth_err(
        &post_form(
            &mut client,
            "/oauth/token",
            &[
                ("grant_type", "authorization_code"),
                ("client_id", "unknown"),
                ("code", "x"),
            ],
        )
        .await,
        StatusCode::UNAUTHORIZED,
        "invalid_client",
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn client_credentials_make_an_app_token_that_only_names_the_app(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let registered = register(&mut client, "read write", REDIRECT).await;
    let id = registered["client_id"].as_str().expect("id");
    let secret = registered["client_secret"].as_str().expect("secret");

    let token = post_form(
        &mut client,
        "/oauth/token",
        &[
            ("grant_type", "client_credentials"),
            ("client_id", id),
            ("client_secret", secret),
        ],
    )
    .await
    .ok();
    assert_eq!(token["scope"], "read");
    let bearer = token["access_token"].as_str().expect("token").to_owned();
    let row: (Option<Uuid>, Option<Uuid>) =
        sqlx::query_as("select user_id, app_id from api_tokens")
            .fetch_one(&db)
            .await
            .expect("row");
    assert_eq!(row.0, None);
    assert_eq!(
        row.1.map(|id| id.to_string()).as_deref(),
        registered["id"].as_str()
    );

    // It identifies the app...
    let mut script = self::client(&app);
    let verified = get_at(
        &mut script,
        "/api/v1/apps/verify_credentials",
        Some(&bearer),
    )
    .await
    .ok();
    assert_eq!(verified["name"], "Test App");
    assert_eq!(verified["id"], registered["id"]);
    assert_eq!(verified["scopes"], json!(["read", "write"]));
    assert_eq!(verified["redirect_uris"], json!([REDIRECT]));
    assert!(verified.get("client_secret").is_none());
    // ...and nobody else: every route that needs a user refuses it.
    for uri in [
        "/api/v1/me",
        "/api/v1/notifications",
        "/api/v1/timelines/following",
    ] {
        get_at(&mut script, uri, Some(&bearer)).await.err(
            StatusCode::FORBIDDEN,
            "This method requires an authenticated user.",
        );
    }
    post_json(
        &mut script,
        "/api/v1/posts",
        Some(&bearer),
        &json!({"text": "x"}),
    )
    .await
    .err(
        StatusCode::FORBIDDEN,
        "This method requires an authenticated user.",
    );

    // The secret is required, and it has to be right.
    oauth_err(
        &post_form(
            &mut client,
            "/oauth/token",
            &[("grant_type", "client_credentials"), ("client_id", id)],
        )
        .await,
        StatusCode::UNAUTHORIZED,
        "invalid_client",
    );
    oauth_err(
        &post_form(
            &mut client,
            "/oauth/token",
            &[
                ("grant_type", "client_credentials"),
                ("client_id", id),
                ("client_secret", "tuita_no"),
            ],
        )
        .await,
        StatusCode::UNAUTHORIZED,
        "invalid_client",
    );
    // A scope beyond the registration is refused; one within it is granted.
    let narrow = register(&mut client, "read", REDIRECT).await;
    oauth_err(
        &post_form(
            &mut client,
            "/oauth/token",
            &[
                ("grant_type", "client_credentials"),
                ("client_id", narrow["client_id"].as_str().expect("id")),
                (
                    "client_secret",
                    narrow["client_secret"].as_str().expect("secret"),
                ),
                ("scope", "read write"),
            ],
        )
        .await,
        StatusCode::BAD_REQUEST,
        "invalid_scope",
    );
    let wide = post_form(
        &mut client,
        "/oauth/token",
        &[
            ("grant_type", "client_credentials"),
            ("client_id", id),
            ("client_secret", secret),
            ("scope", "write"),
        ],
    )
    .await
    .ok();
    assert_eq!(wide["scope"], "write");
    // A user token verifies too, and personal tokens and strangers do not.
    let (_, user_token) = oauth_token(&app, &mut ana, "read").await;
    get_at(
        &mut script,
        "/api/v1/apps/verify_credentials",
        Some(&user_token),
    )
    .await
    .ok();
    let personal = ana
        .browser
        .post("/tokens", json!({"name": "p", "scopes": ["read"]}))
        .await
        .ok();
    get_at(
        &mut script,
        "/api/v1/apps/verify_credentials",
        Some(personal["token"].as_str().expect("secret")),
    )
    .await
    .err(StatusCode::UNAUTHORIZED, "The access token is invalid.");
    get_at(&mut script, "/api/v1/apps/verify_credentials", None)
        .await
        .err(StatusCode::UNAUTHORIZED, "The access token is invalid.");
    get_at(
        &mut script,
        "/api/v1/apps/verify_credentials",
        Some("tuita_nothing_to_see_here_at_all_____"),
    )
    .await
    .err(StatusCode::UNAUTHORIZED, "Invalid or revoked API token.");
}

#[sqlx::test(migrations = "./migrations")]
async fn an_account_keeps_at_most_100_oauth_tokens_and_they_do_not_use_up_personal_ones(
    db: PgPool,
) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let registered = register(&mut client, "read", REDIRECT).await;
    let app_id: Uuid = registered["id"]
        .as_str()
        .expect("id")
        .parse()
        .expect("uuid");
    let mut oldest = Uuid::nil();
    for n in 0..100_u8 {
        let id = Uuid::now_v7();
        if n == 0 {
            oldest = id;
        }
        sqlx::query(
            "insert into api_tokens (id, user_id, name, scopes, token_hash, prefix, app_id)
             values ($1, $2, 'old', '{read}', $3, 'tuita_x', $4)",
        )
        .bind(id)
        .bind(ana.id)
        .bind(sha256(&format!("old-{n}")))
        .bind(app_id)
        .execute(&db)
        .await
        .expect("token");
    }
    // A hundred OAuth tokens leave the 20 personal ones alone.
    for n in 0..20 {
        ana.browser
            .post("/tokens", json!({"name": format!("p{n}")}))
            .await
            .ok();
    }
    let code = get_code(&mut ana, &registered, "read", Some(CHALLENGE)).await;
    exchange(&mut client, &registered, &code, Some(VERIFIER))
        .await
        .ok();
    let held: i64 = sqlx::query_scalar("select count(*) from api_tokens where app_id is not null")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(held, 100);
    let gone: i64 = sqlx::query_scalar("select count(*) from api_tokens where id = $1")
        .bind(oldest)
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(gone, 0, "the oldest went");
    let personal: i64 = sqlx::query_scalar("select count(*) from api_tokens where app_id is null")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(personal, 20);
}

// ------------------------------------------------------------------ revocation

#[sqlx::test(migrations = "./migrations")]
async fn the_client_revokes_its_own_tokens_and_no_one_elses(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let (registered, token) = oauth_token(&app, &mut ana, "read").await;
    let mut other_client = client(&app);
    let other = register(&mut other_client, "read", REDIRECT).await;
    let mut script = client(&app);

    // Another app's credentials do not revoke it; the answer does not tell.
    let reply = post_form(
        &mut other_client,
        "/oauth/revoke",
        &[
            ("client_id", other["client_id"].as_str().expect("id")),
            (
                "client_secret",
                other["client_secret"].as_str().expect("secret"),
            ),
            ("token", &token),
        ],
    )
    .await;
    assert_eq!(reply.ok(), json!({}));
    get_at(
        &mut script,
        "/api/v1/accounts/verify_credentials",
        Some(&token),
    )
    .await
    .ok();

    // A wrong secret is refused.
    oauth_err(
        &post_form(
            &mut script,
            "/oauth/revoke",
            &[
                ("client_id", registered["client_id"].as_str().expect("id")),
                ("client_secret", "tuita_wrong"),
                ("token", &token),
            ],
        )
        .await,
        StatusCode::UNAUTHORIZED,
        "invalid_client",
    );
    get_at(
        &mut script,
        "/api/v1/accounts/verify_credentials",
        Some(&token),
    )
    .await
    .ok();

    // The owner revokes it; revoking again, or an unknown token, is still `{}`.
    for _ in 0..2 {
        let reply = post_form(
            &mut script,
            "/oauth/revoke",
            &[
                ("client_id", registered["client_id"].as_str().expect("id")),
                (
                    "client_secret",
                    registered["client_secret"].as_str().expect("secret"),
                ),
                ("token", &token),
            ],
        )
        .await;
        assert_eq!(reply.ok(), json!({}));
        assert_eq!(reply.header("cache-control"), "no-store");
    }
    get_at(
        &mut script,
        "/api/v1/accounts/verify_credentials",
        Some(&token),
    )
    .await
    .err(StatusCode::UNAUTHORIZED, "Invalid or revoked API token.");
    oauth_err(
        &post_form(
            &mut script,
            "/oauth/revoke",
            &[("client_id", registered["client_id"].as_str().expect("id"))],
        )
        .await,
        StatusCode::BAD_REQUEST,
        "invalid_request",
    );
}

// ------------------------------------------------------------------ the bearer gate

#[sqlx::test(migrations = "./migrations")]
async fn an_oauth_token_is_refused_on_the_cookie_only_paths_but_not_below_accounts(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    sqlx::query("update users set role = 'admin' where id = $1")
        .bind(ana.id)
        .execute(&db)
        .await
        .expect("admin");
    let (_, token) = oauth_token(&app, &mut ana, "read write follow").await;
    let mut script = client(&app);
    let refused = "API tokens can\u{2019}t be used for this. Sign in on the website instead.";
    for (method, uri) in [
        (Method::GET, "/api/v1/tokens"),
        (Method::POST, "/api/v1/tokens"),
        (Method::GET, "/api/v1/admin/stats"),
        (Method::GET, "/api/v1/accounts"),
        (Method::POST, "/api/v1/accounts/switch"),
        (Method::GET, "/api/v1/me/sessions"),
        (Method::GET, "/api/v1/me/export"),
        (Method::POST, "/api/v1/me/password"),
        (Method::DELETE, "/api/v1/me"),
        (Method::GET, "/api/v1/passkeys"),
        (Method::POST, "/api/v1/auth/logout"),
        (Method::GET, "/api/v1/oauth/authorization"),
    ] {
        call(&mut script, method.clone(), uri, Some(&token), None)
            .await
            .err(StatusCode::FORBIDDEN, refused);
    }
    // Below `/accounts` the gate lets the request pass, whatever the route then answers (404
    // before the accounts routes exist, their own answer after): never the cookie-only 403, never
    // a token error.
    for uri in [
        "/api/v1/accounts/verify_credentials",
        "/api/v1/accounts/0198aaaa-0000-7000-8000-000000000001",
        "/api/v1/accounts/relationships",
    ] {
        let reply = get_at(&mut script, uri, Some(&token)).await;
        assert_ne!(reply.status, StatusCode::FORBIDDEN, "{uri}");
        assert_ne!(reply.status, StatusCode::UNAUTHORIZED, "{uri}");
        assert!(
            !String::from_utf8_lossy(&reply.bytes).contains(refused),
            "{uri}"
        );
    }
    // POST /oauth/authorize is outside the API: the session cookie decides, never the bearer.
    post_json(
        &mut script,
        "/oauth/authorize",
        Some(&token),
        &json!({"clientId": "x", "approve": true}),
    )
    .await
    .err(StatusCode::UNAUTHORIZED, "sign in required");
}

#[sqlx::test(migrations = "./migrations")]
async fn conversations_need_plain_read_or_write_for_an_oauth_token(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let (_, read_token) = oauth_token(&app, &mut ana, "read").await;
    let personal = ana
        .browser
        .post("/tokens", json!({"name": "p", "scopes": ["read"]}))
        .await
        .ok();
    let mut script = client(&app);
    get_at(&mut script, "/api/v1/conversations", Some(&read_token))
        .await
        .ok();
    // A personal read token has no `messages` scope and stays refused.
    get_at(
        &mut script,
        "/api/v1/conversations",
        Some(personal["token"].as_str().expect("secret")),
    )
    .await
    .err(
        StatusCode::FORBIDDEN,
        "This token doesn\u{2019}t have the \u{201c}messages\u{201d} scope.",
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_suspended_owner_ends_the_oauth_token(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let (_, token) = oauth_token(&app, &mut ana, "read").await;
    let mut script = client(&app);
    get_at(
        &mut script,
        "/api/v1/accounts/verify_credentials",
        Some(&token),
    )
    .await
    .ok();
    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(ana.id)
        .execute(&db)
        .await
        .expect("suspend");
    get_at(
        &mut script,
        "/api/v1/accounts/verify_credentials",
        Some(&token),
    )
    .await
    .err(StatusCode::UNAUTHORIZED, "Invalid or revoked API token.");
}

#[sqlx::test(migrations = "./migrations")]
async fn the_streaming_token_may_come_in_the_query_or_the_subprotocol(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let (_, token) = oauth_token(&app, &mut ana, "read").await;
    let mut script = client(&app);
    // A bad token in the query is judged like a bad header; a good one passes the gate (what
    // the route then says about a request that is no websocket upgrade is the route's own).
    get_at(
        &mut script,
        "/api/v1/streaming?access_token=tuita_nothing_to_see_here_at_all_____",
        None,
    )
    .await
    .err(StatusCode::UNAUTHORIZED, "Invalid or revoked API token.");
    let ok = get_at(
        &mut script,
        &format!("/api/v1/streaming?access_token={token}"),
        None,
    )
    .await;
    assert_ne!(ok.status, StatusCode::UNAUTHORIZED);
    assert_ne!(ok.status, StatusCode::FORBIDDEN);
    let via_protocol = script
        .send_raw(
            Method::GET,
            "/api/v1/streaming",
            &[("sec-websocket-protocol", &token)],
            Body::empty(),
        )
        .await;
    assert_ne!(via_protocol.status, StatusCode::UNAUTHORIZED);
    // Other routes ignore the query: no token there means nobody is signed in.
    let anonymous = get_at(
        &mut script,
        &format!("/api/v1/me?access_token={token}"),
        None,
    )
    .await
    .ok();
    assert_eq!(anonymous, json!({"user": null}));
}

#[sqlx::test(migrations = "./migrations")]
async fn by_credential_sends_oauth_requests_to_the_mastodon_handler(db: PgPool) {
    async fn tuita() -> &'static str {
        "tuita"
    }
    async fn mastodon(State(state): State<AppState>) -> String {
        format!("mastodon {}", state.config().domain())
    }
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let (_, oauth) = oauth_token(&app, &mut ana, "read").await;
    let personal = ana
        .browser
        .post("/tokens", json!({"name": "p", "scopes": ["read"]}))
        .await
        .ok();
    let personal = personal["token"].as_str().expect("secret").to_owned();
    let router = Router::new()
        .route("/notifications", get(by_credential(tuita, mastodon)))
        .layer(from_fn_with_state(app.state.clone(), bearer::gate))
        .with_state(app.state.clone());
    let ask = |bearer: Option<String>, cookie: Option<String>| {
        let router = router.clone();
        async move {
            let mut request = Request::builder().uri("/notifications");
            if let Some(bearer) = bearer {
                request = request.header("authorization", format!("Bearer {bearer}"));
            }
            if let Some(cookie) = cookie {
                request = request.header("cookie", cookie);
            }
            let response = router
                .oneshot(request.body(Body::empty()).expect("request"))
                .await
                .expect("infallible");
            let status = response.status();
            let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("body");
            (status, String::from_utf8_lossy(&bytes).into_owned())
        }
    };
    assert_eq!(
        ask(Some(oauth), None).await,
        (StatusCode::OK, "mastodon social.example".to_owned())
    );
    assert_eq!(
        ask(Some(personal), None).await,
        (StatusCode::OK, "tuita".to_owned())
    );
    assert_eq!(ask(None, None).await, (StatusCode::OK, "tuita".to_owned()));
    let cookie = format!(
        "tuita_session={}",
        ana.browser.cookie("tuita_session").expect("session")
    );
    assert_eq!(
        ask(None, Some(cookie)).await,
        (StatusCode::OK, "tuita".to_owned())
    );
}

// ------------------------------------------------------------------ CORS and metadata

#[sqlx::test(migrations = "./migrations")]
async fn the_mastodon_api_answers_cors_preflights_and_marks_every_answer(db: PgPool) {
    let app = common::app(db);
    let mut web_client = app.browser_at([203, 0, 113, 20]);
    web_client.origin = Some("https://phanpy.social".to_owned());
    let preflight_headers = [
        ("access-control-request-method", "POST"),
        (
            "access-control-request-headers",
            "authorization,content-type,idempotency-key",
        ),
    ];
    for uri in [
        "/api/v1/statuses",
        "/api/v1/apps",
        "/api/v1/no/such/route",
        "/api/v2/search",
        "/api/v2/no/such/route",
        "/oauth/token",
        "/oauth/revoke",
    ] {
        let reply = web_client
            .send_raw(Method::OPTIONS, uri, &preflight_headers, Body::empty())
            .await;
        assert!(reply.status.is_success(), "{uri}: {}", reply.status);
        assert_eq!(reply.header("access-control-allow-origin"), "*", "{uri}");
        let allowed = reply
            .header("access-control-allow-headers")
            .to_ascii_lowercase();
        for header in ["authorization", "content-type", "idempotency-key"] {
            assert!(allowed.contains(header), "{uri}: {allowed}");
        }
        assert!(
            reply
                .header("access-control-allow-methods")
                .contains("POST"),
            "{uri}"
        );
        assert!(
            reply
                .headers
                .get("access-control-allow-credentials")
                .is_none(),
            "{uri}"
        );
    }
    // Real answers carry the origin header and expose Link; errors too.
    for (uri, status) in [
        ("/api/v1/instance", StatusCode::OK),
        ("/api/v1/tokens", StatusCode::UNAUTHORIZED),
        ("/api/v1/no/such/route", StatusCode::NOT_FOUND),
        ("/api/v2/no/such/route", StatusCode::NOT_FOUND),
        ("/.well-known/oauth-authorization-server", StatusCode::OK),
    ] {
        let reply = get_at(&mut web_client, uri, None).await;
        assert_eq!(reply.status, status, "{uri}");
        assert_eq!(reply.header("access-control-allow-origin"), "*", "{uri}");
        assert!(
            reply
                .headers
                .get("access-control-allow-credentials")
                .is_none(),
            "{uri}"
        );
    }
    let reply = get_at(&mut web_client, "/api/v1/instance", None).await;
    assert!(
        reply
            .header("access-control-expose-headers")
            .to_ascii_lowercase()
            .contains("link")
    );
    // The CSRF guard still stops a cookie from another origin, CORS or not.
    let mut ana_browser = app.browser_at([203, 0, 113, 21]);
    ana_browser.origin = Some("https://phanpy.social".to_owned());
    ana_browser.set_cookie("tuita_session", "x".repeat(43).as_str());
    post_json(
        &mut ana_browser,
        "/api/v1/posts",
        None,
        &json!({"text": "x"}),
    )
    .await
    .err(StatusCode::FORBIDDEN, "bad origin");
}

#[sqlx::test(migrations = "./migrations")]
async fn the_server_metadata_names_the_endpoints_and_only_s256(db: PgPool) {
    let app = common::app(db);
    let mut visitor = client(&app);
    let reply = get_at(
        &mut visitor,
        "/.well-known/oauth-authorization-server",
        None,
    )
    .await;
    assert_eq!(
        reply.ok(),
        json!({
            "issuer": "https://social.example",
            "authorization_endpoint": "https://social.example/oauth/authorize",
            "token_endpoint": "https://social.example/oauth/token",
            "revocation_endpoint": "https://social.example/oauth/revoke",
            "app_registration_endpoint": "https://social.example/api/v1/apps",
            "scopes_supported": ["read", "write", "follow", "push"],
            "response_types_supported": ["code"],
            "response_modes_supported": ["query"],
            "grant_types_supported": ["authorization_code", "client_credentials"],
            "token_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post", "none"],
            "code_challenge_methods_supported": ["S256"],
        })
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn secrets_appear_only_in_the_answer_meant_for_them(db: PgPool) {
    // The access token travels in a header and the code in a redirect the user's browser
    // follows once; neither appears in a response body other than the one meant for it.
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let registered = register(&mut client, "read", REDIRECT).await;
    let code = get_code(&mut ana, &registered, "read", Some(CHALLENGE)).await;
    let token = exchange(&mut client, &registered, &code, Some(VERIFIER))
        .await
        .ok();
    let secret = token["access_token"].as_str().expect("token");
    let mut script = self::client(&app);
    let me = get_at(
        &mut script,
        "/api/v1/accounts/verify_credentials",
        Some(secret),
    )
    .await;
    assert!(!String::from_utf8_lossy(&me.bytes).contains(secret));
    let verify = get_at(&mut script, "/api/v1/apps/verify_credentials", Some(secret)).await;
    let text = String::from_utf8_lossy(&verify.bytes);
    assert!(!text.contains(secret));
    assert!(!text.contains(registered["client_secret"].as_str().expect("client secret")));
    let listed = ana.browser.get("/tokens").await;
    assert!(!listed.body.contains(secret));
}

// A direct use of `URL_SAFE_NO_PAD` keeps the helper constants honest.
#[test]
fn the_pkce_constants_are_an_s256_pair() {
    assert_eq!(
        URL_SAFE_NO_PAD.encode(Sha256::digest(VERIFIER.as_bytes())),
        CHALLENGE
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn cleanup_keeps_a_used_code_a_day_past_its_expiry_so_a_replay_still_revokes(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut client = client(&app);
    let registered = register(&mut client, "read", REDIRECT).await;
    let used = get_code(&mut ana, &registered, "read", Some(CHALLENGE)).await;
    let token = exchange(&mut client, &registered, &used, Some(VERIFIER))
        .await
        .ok();
    let secret = token["access_token"].as_str().expect("token");
    let fresh = get_code(&mut ana, &registered, "read", Some(CHALLENGE)).await;
    let stale = get_code(&mut ana, &registered, "read", Some(CHALLENGE)).await;
    sqlx::query(
        "update oauth_codes set expires_at = now() - interval '23 hours' where code_hash = $1",
    )
    .bind(sha256(&used))
    .execute(&db)
    .await
    .expect("age the used code");
    sqlx::query(
        "update oauth_codes set expires_at = now() - interval '25 hours' where code_hash = $1",
    )
    .bind(sha256(&stale))
    .execute(&db)
    .await
    .expect("age the stale code");

    tuita_server::workers::cleanup::run_once(&app.state)
        .await
        .expect("cleanup");
    let left: Vec<Vec<u8>> = sqlx::query_scalar("select code_hash from oauth_codes order by 1")
        .fetch_all(&db)
        .await
        .expect("codes");
    let mut expected = vec![sha256(&used), sha256(&fresh)];
    expected.sort();
    assert_eq!(left, expected, "only the code a day past its expiry went");

    // The replay of the used code still takes the token back.
    oauth_err(
        &exchange(&mut client, &registered, &used, Some(VERIFIER)).await,
        StatusCode::BAD_REQUEST,
        "invalid_grant",
    );
    let mut script = self::client(&app);
    get_at(
        &mut script,
        "/api/v1/accounts/verify_credentials",
        Some(secret),
    )
    .await
    .err(StatusCode::UNAUTHORIZED, "Invalid or revoked API token.");
}

// ------------------------------------------------------------------ the OAuth allowlist

const REFUSED_FOR_TOKENS: &str =
    "API tokens can\u{2019}t be used for this. Sign in on the website instead.";

#[sqlx::test(migrations = "./migrations")]
async fn an_oauth_token_never_reaches_the_tuita_only_routes(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    common::insert_user(&db, "ada").await;
    let (_, read) = oauth_token(&app, &mut ana, "read").await;
    let (_, write) = oauth_token(&app, &mut ana, "write").await;
    let (_, both) = oauth_token(&app, &mut ana, "read write").await;
    let mut script = client(&app);
    let body = json!({"text": "x", "name": "x", "bot": true, "targetId": "x"});
    for token in [&read, &write, &both] {
        for (method, uri) in [
            (Method::GET, "/api/v1/conversations/ada"),
            (Method::POST, "/api/v1/conversations/ada"),
            (Method::DELETE, "/api/v1/conversations/ada/messages/1"),
            (Method::POST, "/api/v1/me/bot"),
            (Method::PATCH, "/api/v1/me"),
            (Method::GET, "/api/v1/me"),
            (Method::POST, "/api/v1/reports"),
            (Method::GET, "/api/v1/users/ada"),
            (Method::POST, "/api/v1/users/ada/follow"),
            (Method::POST, "/api/v1/posts"),
            (Method::POST, "/api/v1/affiliations"),
            (Method::POST, "/api/v1/notifications/read"),
            (Method::GET, "/api/v1/timelines/following"),
            (Method::GET, "/api/v1/stream"),
            (Method::GET, "/api/v1/badges"),
            (Method::POST, "/api/v1/media"),
        ] {
            let reply = call(
                &mut script,
                method.clone(),
                uri,
                Some(token),
                Some(("application/json", body.to_string())),
            )
            .await;
            reply.err(StatusCode::FORBIDDEN, REFUSED_FOR_TOKENS);
        }
    }
    // Nothing was sent: no message, no report, no post.
    for (table, sql) in [
        ("messages", "select count(*) from messages"),
        ("reports", "select count(*) from reports"),
        ("posts", "select count(*) from posts"),
    ] {
        let rows: i64 = sqlx::query_scalar(sql).fetch_one(&db).await.expect("count");
        assert_eq!(rows, 0, "{table}");
    }
    // Personal tokens and cookies keep the whole tuita API.
    let personal = ana
        .browser
        .post(
            "/tokens",
            json!({"name": "p", "scopes": ["read", "write", "messages"]}),
        )
        .await
        .ok();
    let personal = personal["token"].as_str().expect("secret");
    get_at(&mut script, "/api/v1/users/ada", Some(personal))
        .await
        .ok();
    post_json(
        &mut script,
        "/api/v1/posts",
        Some(personal),
        &json!({"text": "from a script"}),
    )
    .await
    .ok();
    get_at(&mut script, "/api/v1/conversations/ada", Some(personal))
        .await
        .ok();
    ana.browser.get("/users/ada").await.ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn every_mastodon_route_family_is_open_to_an_oauth_token(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let (_, token) = oauth_token(&app, &mut ana, "read write follow").await;
    let mut script = client(&app);
    let id = "0198aaaa-0000-7000-8000-000000000001";
    for (method, uri) in [
        (Method::GET, "/api/v1/apps/verify_credentials".to_owned()),
        (
            Method::GET,
            "/api/v1/accounts/verify_credentials".to_owned(),
        ),
        (
            Method::PATCH,
            "/api/v1/accounts/update_credentials".to_owned(),
        ),
        (Method::GET, "/api/v1/accounts/lookup?acct=ana".to_owned()),
        (Method::GET, "/api/v1/accounts/relationships".to_owned()),
        (Method::GET, format!("/api/v1/accounts/{id}")),
        (Method::GET, format!("/api/v1/accounts/{id}/statuses")),
        (Method::POST, format!("/api/v1/accounts/{id}/follow")),
        (Method::POST, "/api/v1/statuses".to_owned()),
        (Method::GET, format!("/api/v1/statuses/{id}")),
        (Method::DELETE, format!("/api/v1/statuses/{id}")),
        (Method::GET, format!("/api/v1/statuses/{id}/context")),
        (Method::POST, format!("/api/v1/statuses/{id}/favourite")),
        (Method::GET, "/api/v1/timelines/home".to_owned()),
        (Method::GET, "/api/v1/timelines/public".to_owned()),
        (Method::GET, "/api/v1/timelines/tag/rust".to_owned()),
        (Method::GET, format!("/api/v1/timelines/list/{id}")),
        (Method::GET, "/api/v1/notifications".to_owned()),
        (Method::GET, format!("/api/v1/notifications/{id}")),
        (Method::POST, "/api/v1/notifications/clear".to_owned()),
        (Method::POST, format!("/api/v1/notifications/{id}/dismiss")),
        (Method::GET, "/api/v1/lists".to_owned()),
        (Method::POST, "/api/v1/lists".to_owned()),
        (Method::GET, format!("/api/v1/lists/{id}")),
        (Method::PUT, format!("/api/v1/lists/{id}")),
        (Method::DELETE, format!("/api/v1/lists/{id}")),
        (Method::GET, format!("/api/v1/lists/{id}/accounts")),
        (Method::GET, "/api/v1/bookmarks".to_owned()),
        (Method::GET, "/api/v1/favourites".to_owned()),
        (Method::GET, "/api/v1/conversations".to_owned()),
        (Method::DELETE, format!("/api/v1/conversations/{id}")),
        (Method::POST, format!("/api/v1/conversations/{id}/read")),
        (Method::PUT, format!("/api/v1/media/{id}")),
        (Method::GET, format!("/api/v1/polls/{id}")),
        (Method::POST, format!("/api/v1/polls/{id}/votes")),
        (Method::GET, "/api/v1/instance".to_owned()),
        (Method::GET, "/api/v1/custom_emojis".to_owned()),
        (Method::GET, "/api/v1/streaming".to_owned()),
        (Method::GET, "/api/v1/scheduled_statuses".to_owned()),
        (Method::GET, "/api/v1/preferences".to_owned()),
        (Method::GET, "/api/v1/markers".to_owned()),
        (Method::GET, "/api/v1/filters".to_owned()),
        (Method::GET, "/api/v1/followed_tags".to_owned()),
        (Method::GET, "/api/v1/follow_requests".to_owned()),
        (Method::GET, "/api/v1/blocks".to_owned()),
        (Method::GET, "/api/v1/mutes".to_owned()),
        (Method::GET, "/api/v1/announcements".to_owned()),
        (Method::GET, "/api/v1/trends/tags".to_owned()),
        (Method::GET, "/api/v1/suggestions".to_owned()),
        (Method::GET, "/api/v1/endorsements".to_owned()),
        (Method::GET, "/api/v1/domain_blocks".to_owned()),
        (
            Method::GET,
            "/api/v1/accounts/familiar_followers".to_owned(),
        ),
        (Method::GET, "/api/v1/instance/rules".to_owned()),
        (Method::GET, "/api/v2/search?q=ana".to_owned()),
        (Method::GET, "/api/v2/instance".to_owned()),
        (Method::POST, "/api/v2/media".to_owned()),
        (Method::GET, "/api/v2/suggestions".to_owned()),
    ] {
        let reply = call(
            &mut script,
            method.clone(),
            &uri,
            Some(&token),
            Some(("application/json", "{}".to_owned())),
        )
        .await;
        // A missing route, a bad body or an unknown id may answer 404, 405 or 422; a 403 or
        // 401 would be the gate.
        assert!(
            reply.status != StatusCode::FORBIDDEN && reply.status != StatusCode::UNAUTHORIZED,
            "{method} {uri}: {} {}",
            reply.status,
            String::from_utf8_lossy(&reply.bytes)
        );
    }
}

/// The `.route("...")` paths written in the Mastodon source files.
fn registered_mastodon_paths() -> Vec<(String, String)> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/mastodon");
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir).expect("src/mastodon") {
        let path = entry.expect("entry").path();
        let name = path
            .file_name()
            .expect("name")
            .to_string_lossy()
            .into_owned();
        // `mod.rs` only has doc examples; `oauth.rs` serves the OAuth endpoints outside the
        // allowlist; `search.rs` has only v2 routes.
        if path.extension().is_none_or(|ext| ext != "rs")
            || ["mod.rs", "oauth.rs", "search.rs"].contains(&name.as_str())
        {
            continue;
        }
        let source = std::fs::read_to_string(&path).expect("source");
        for part in source.split(".route(").skip(1) {
            let rest = part.trim_start();
            let Some(rest) = rest.strip_prefix('"') else {
                continue;
            };
            let route = rest.split('"').next().expect("closing quote");
            // `/media` here is the v2 upload.
            if name == "media.rs" && route == "/media" {
                continue;
            }
            found.push((name.clone(), route.to_owned()));
        }
    }
    found
}

#[sqlx::test(migrations = "./migrations")]
async fn every_registered_mastodon_route_is_open_to_oauth_tokens(db: PgPool) {
    let routes = registered_mastodon_paths();
    assert!(
        routes.len() > 40,
        "the scan found only {} routes: {routes:?}",
        routes.len()
    );
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let (_, token) = oauth_token(&app, &mut ana, "read write follow").await;
    let mut script = client(&app);
    for (file, route) in routes {
        let mut uri = String::from("/api/v1");
        let mut rest = route.as_str();
        while let Some(start) = rest.find('{') {
            uri.push_str(&rest[..start]);
            uri.push_str("0198aaaa-0000-7000-8000-000000000001");
            rest = &rest[rest.find('}').expect("closing brace") + 1..];
        }
        uri.push_str(rest);
        // The route answers some method; it must not be the gate that turns every one away.
        let mut answers = Vec::new();
        for method in [
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ] {
            let reply = call(
                &mut script,
                method.clone(),
                &uri,
                Some(&token),
                Some(("application/json", "{}".to_owned())),
            )
            .await;
            answers.push(reply.status);
        }
        assert!(
            answers
                .iter()
                .any(|status| *status != StatusCode::FORBIDDEN),
            "{file}: {route} is refused for OAuth tokens; add it to `mastodon::ALLOWED`: {answers:?}"
        );
    }
}
