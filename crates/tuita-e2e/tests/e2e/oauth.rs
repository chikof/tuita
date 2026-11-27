//! Browser tests: the OAuth consent card, the pages around it and the `?next=` rule.
//! The app's own site is a tiny local server on a loopback port, which the
//! server accepts as an `http` redirect target; it logs every request so a test can say that
//! nothing reached the app before the user answered.

use std::{
    future::IntoFuture,
    sync::{Arc, Mutex, PoisonError},
};

use axum::{Router, http::Uri, response::Html};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use tokio::net::TcpListener;
use tuita_e2e::{
    Ctx, Name, Page, Result, Violation,
    helpers::{PASSWORD, call, handle, sign_up_via_api},
};
use uuid::Uuid;

const OOB: &str = "urn:ietf:wg:oauth:2.0:oob";

/// The app's website and callback, and what they were asked for.
struct Client {
    origin: String,
    seen: Arc<Mutex<Vec<String>>>,
}

impl Client {
    async fn start() -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let origin = format!("http://127.0.0.1:{}", listener.local_addr()?.port());
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = Arc::clone(&seen);
        let site = Router::new().fallback(move |uri: Uri| {
            let log = Arc::clone(&log);
            async move {
                log.lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(uri.to_string());
                Html("<title>client</title>client")
            }
        });
        tokio::spawn(axum::serve(listener, site).into_future());
        Ok(Self { origin, seen })
    }

    fn callback(&self) -> String {
        format!("{}/callback", self.origin)
    }

    fn requests(&self) -> Vec<String> {
        self.seen
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

struct App {
    id: String,
    secret: String,
}

/// What a Mastodon client does first: registers itself, from a browser that holds no session.
async fn register(site: &str, redirect: &str) -> Result<App> {
    let ctx = Ctx::new().await?;
    let app = call(
        &ctx,
        "POST",
        "/apps",
        Some(&json!({
            "client_name": "Tuba",
            "redirect_uris": redirect,
            "scopes": "read write",
            "website": site,
        })),
    )
    .await?;
    Ok(App {
        id: app["client_id"].as_str().unwrap_or_default().to_owned(),
        secret: app["client_secret"].as_str().unwrap_or_default().to_owned(),
    })
}

/// A PKCE verifier: 64 unreserved characters.
fn verifier() -> String {
    format!("{}{}", Uuid::now_v7().simple(), Uuid::now_v7().simple())
}

fn challenge_of(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

/// `encodeURIComponent`.
fn enc(text: &str) -> String {
    let mut out = String::new();
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&byte) {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn authorize_url(app: &App, redirect: &str, challenge: &str, overrides: &[(&str, &str)]) -> String {
    let mut pairs = vec![
        ("client_id", app.id.as_str()),
        ("redirect_uri", redirect),
        ("response_type", "code"),
        ("scope", "read write"),
        ("code_challenge", challenge),
        ("code_challenge_method", "S256"),
        ("state", "xyz"),
    ];
    for (key, value) in overrides {
        if let Some(pair) = pairs.iter_mut().find(|pair| pair.0 == *key) {
            pair.1 = value;
        }
    }
    let query: Vec<String> = pairs
        .iter()
        .map(|(key, value)| format!("{key}={}", enc(value)))
        .collect();
    format!("/oauth/authorize?{}", query.join("&"))
}

/// The client trades the code for a token, again without a session.
async fn exchange(
    app: &App,
    code: &str,
    redirect: &str,
    code_verifier: &str,
) -> Result<(u16, Value)> {
    let ctx = Ctx::new().await?;
    ctx.fetch(
        "POST",
        "/oauth/token",
        Some(&json!({
            "grant_type": "authorization_code",
            "client_id": app.id,
            "client_secret": app.secret,
            "code": code,
            "redirect_uri": redirect,
            "code_verifier": code_verifier,
        })),
    )
    .await
}

/// A signed-in browser context; the user's handle comes back with it.
async fn signed_in() -> Result<(Ctx, Page, String)> {
    let ctx = Ctx::new().await?;
    let who = handle("oa");
    sign_up_via_api(&ctx, &who, "OAuth Tester").await?;
    let page = ctx.page().await?;
    Ok((ctx, page, who))
}

/// The path (with query) of the page's URL on this site, or the whole URL when it is elsewhere.
async fn location(page: &Page) -> Result<String> {
    let url = page.url().await?;
    Ok(url
        .strip_prefix(page.base())
        .map_or(url.clone(), str::to_owned))
}

fn query_of(url: &str) -> Vec<(String, String)> {
    let Some((_, query)) = url.split_once('?') else {
        return Vec::new();
    };
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}

fn param(url: &str, name: &str) -> Option<String> {
    query_of(url)
        .into_iter()
        .find_map(|(key, value)| (key == name).then_some(value))
}

async fn clean(page: &Page, what: &str) -> Result<()> {
    let errors = page.console_errors().await;
    assert!(errors.is_empty(), "{what}: console errors: {errors:#?}");
    let violations = page.axe().await?;
    assert!(
        violations.is_empty(),
        "{what}: axe violations:\n{}",
        Violation::report(&violations)
    );
    Ok(())
}

#[tokio::test]
async fn signed_out_sign_in_see_the_card_approve_and_trade_the_code_for_a_token() -> Result<()> {
    let owner = Ctx::new().await?;
    let who = handle("oa");
    sign_up_via_api(&owner, &who, "OAuth Tester").await?;
    let client = Client::start().await?;
    let app = register(&client.origin, &client.callback()).await?;
    let secret = verifier();

    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    page.goto(&authorize_url(
        &app,
        &client.callback(),
        &challenge_of(&secret),
        &[],
    ))
    .await?;
    page.expect_url(Name::Re(
        r"/login\?next=%2Foauth%2Fauthorize%3Fclient_id%3D",
    ))
    .await?;
    page.role("link", Name::Exact("New here? Create an account"))
        .expect_attr("href", Name::Re(r"^/signup\?next=%2Foauth%2Fauthorize"))
        .await?;

    page.label("Handle").fill(&who).await?;
    page.label("Password").fill(PASSWORD).await?;
    page.role("button", Name::Exact("Sign in")).click().await?;

    page.expect_url(Name::Re(r"/oauth/authorize\?client_id="))
        .await?;
    page.role("heading", Name::Exact("Authorize an app"))
        .expect_visible()
        .await?;
    let card = page.css("[data-state=\"ask\"]");
    card.css("[data-app-name]")
        .expect_text(Name::Exact("Tuba"))
        .await?;
    let link = card.role("link", Name::Exact(&client.origin));
    link.expect_attr("href", Name::Exact(&format!("{}/", client.origin)))
        .await?;
    card.text(format!("@{who}").as_str())
        .expect_visible()
        .await?;
    card.css("[data-redirect]")
        .expect_text(Name::Contains("127.0.0.1"))
        .await?;
    card.css("[data-scope]").expect_count(2).await?;
    card.css("[data-scope=\"read\"]").expect_visible().await?;
    card.css("[data-scope=\"write\"]").expect_visible().await?;
    card.text("Push notifications aren’t supported.")
        .expect_visible()
        .await?;
    assert_eq!(
        client.requests(),
        Vec::<String>::new(),
        "nothing is sent to the app before the answer"
    );
    clean(&page, "the consent card").await?;

    card.role("button", Name::Exact("Authorize"))
        .click()
        .await?;
    page.expect_url(Name::Re(&format!(
        "^{}/callback\\?",
        client.origin.replace('.', "\\.")
    )))
    .await?;
    let landed = page.url().await?;
    assert_eq!(landed.split('?').next(), Some(client.callback().as_str()));
    assert_eq!(param(&landed, "state").as_deref(), Some("xyz"));
    let code = param(&landed, "code").expect("the redirect carries a code");
    assert!(!code.is_empty());

    let (status, token) = exchange(&app, &code, &client.callback(), &secret).await?;
    assert_eq!(status, 200, "{token}");
    assert_eq!(token["token_type"], "Bearer");
    assert_eq!(token["scope"], "read write");
    let (replay, _) = exchange(&app, &code, &client.callback(), &secret).await?;
    assert_eq!(replay, 400, "a code works once");

    let errors = page.console_errors().await;
    assert!(errors.is_empty(), "console errors: {errors:#?}");
    Ok(())
}

#[tokio::test]
async fn deny_sends_the_app_access_denied_and_no_code() -> Result<()> {
    let (_ctx, page, _who) = signed_in().await?;
    let client = Client::start().await?;
    let app = register(&client.origin, &client.callback()).await?;

    page.goto(&authorize_url(
        &app,
        &client.callback(),
        &challenge_of(&verifier()),
        &[],
    ))
    .await?;
    let card = page.css("[data-state=\"ask\"]");
    let deny = card.role("button", Name::Exact("Deny"));
    deny.expect_visible().await?;
    deny.click().await?;

    page.expect_url(Name::Re(&format!(
        "^{}/callback\\?",
        client.origin.replace('.', "\\.")
    )))
    .await?;
    let landed = page.url().await?;
    assert_eq!(param(&landed, "error").as_deref(), Some("access_denied"));
    assert_eq!(param(&landed, "state").as_deref(), Some("xyz"));
    assert_eq!(param(&landed, "code"), None);

    let errors = page.console_errors().await;
    assert!(errors.is_empty(), "console errors: {errors:#?}");
    Ok(())
}

#[tokio::test]
async fn a_request_that_fails_validation_shows_the_error_card_and_goes_nowhere() -> Result<()> {
    let (_ctx, page, _who) = signed_in().await?;
    let client = Client::start().await?;
    let app = register(&client.origin, &client.callback()).await?;
    let pkce = challenge_of(&verifier());
    let unknown = App {
        id: "nope".to_owned(),
        secret: String::new(),
    };

    let refused = [
        (
            authorize_url(&app, "https://evil.example/steal", &pkce, &[]),
            "address the app didn’t register",
        ),
        (
            authorize_url(&unknown, &client.callback(), &pkce, &[]),
            "isn’t registered here",
        ),
        (
            authorize_url(
                &app,
                &client.callback(),
                &pkce,
                &[("scope", "read:statuses")],
            ),
            "access that tuita doesn’t offer",
        ),
        (
            authorize_url(
                &app,
                &client.callback(),
                &pkce,
                &[("response_type", "token")],
            ),
            "kind of sign-in",
        ),
    ];
    for (url, text) in refused {
        page.goto(&url).await?;
        let alert = page.role("alert", Name::Any);
        alert
            .role("heading", Name::Exact("This request can’t be authorized"))
            .expect_visible()
            .await?;
        alert.expect_text(Name::Contains(text)).await?;
        alert
            .expect_text(Name::Contains("Nothing was sent to the app."))
            .await?;
        page.role("button", Name::Exact("Authorize"))
            .expect_count(0)
            .await?;
        assert!(
            location(&page).await?.starts_with("/oauth/authorize?"),
            "the page stays"
        );
        clean(&page, "the error card").await?;
    }
    page.goto("/oauth/authorize").await?;
    page.role("alert", Name::Any)
        .expect_text(Name::Contains("The request is malformed"))
        .await?;

    assert_eq!(
        client.requests(),
        Vec::<String>::new(),
        "the app heard nothing"
    );
    Ok(())
}

#[tokio::test]
async fn an_out_of_band_app_gets_its_code_on_screen_with_a_copy_button() -> Result<()> {
    let (ctx, page, _who) = signed_in().await?;
    ctx.grant(&["clipboard-read", "clipboard-write"]).await?;
    let client = Client::start().await?;
    let app = register(&client.origin, OOB).await?;
    let secret = verifier();

    page.goto(&authorize_url(&app, OOB, &challenge_of(&secret), &[]))
        .await?;
    let card = page.css("[data-state=\"ask\"]");
    card.css("[data-redirect]")
        .expect_text(Name::Contains("A code is shown here"))
        .await?;
    card.role("button", Name::Exact("Authorize"))
        .click()
        .await?;

    let field = page.role("textbox", Name::Exact("Authorization code"));
    field.expect_visible().await?;
    let code = field.value().await?;
    assert!(code.len() > 20, "code: {code:?}");
    assert_eq!(
        location(&page).await?.split('?').next(),
        Some("/oauth/authorize"),
        "the page stays"
    );
    clean(&page, "the code").await?;

    page.role("button", Name::Exact("Copy code"))
        .click()
        .await?;
    page.text("Code copied").first().expect_visible().await?;
    let copied = page.eval("navigator.clipboard.readText()").await?;
    assert_eq!(copied, Value::String(code.clone()));

    let (status, _) = exchange(&app, &code, OOB, &secret).await?;
    assert_eq!(status, 200);

    page.goto(&authorize_url(&app, OOB, &challenge_of(&verifier()), &[]))
        .await?;
    page.role("button", Name::Exact("Deny")).click().await?;
    page.text("Denied. Nothing was shared with the app.")
        .expect_visible()
        .await?;
    page.role("textbox", Name::Any).expect_count(0).await?;
    clean(&page, "the denied card").await?;
    Ok(())
}

#[tokio::test]
async fn sign_in_follows_next_only_to_a_path_on_this_site() -> Result<()> {
    let setup = Ctx::new().await?;
    let who = handle("nx");
    sign_up_via_api(&setup, &who, "Next Tester").await?;

    for next in [
        "https://evil.example/",
        "//evil.example/x",
        "/\\evil.example",
        "/safe-path-404",
    ] {
        let ctx = Ctx::new().await?;
        let page = ctx.page().await?;
        page.goto(&format!("/login?next={}", enc(next))).await?;
        page.label("Handle").fill(&who).await?;
        page.label("Password").fill(PASSWORD).await?;
        page.role("button", Name::Exact("Sign in")).click().await?;
        if next.starts_with("/s") {
            page.text("Nothing here").first().expect_visible().await?;
            assert_eq!(location(&page).await?, "/safe-path-404");
        } else {
            page.role("navigation", Name::Exact("Main"))
                .first()
                .expect_visible()
                .await?;
            assert_eq!(location(&page).await?, "/", "{next}");
        }
    }

    // Already signed in: the sign-in page forwards at once.
    let page = setup.page().await?;
    page.goto(&format!("/login?next={}", enc("/lists"))).await?;
    page.expect_url(Name::Exact("/lists")).await?;
    page.role("heading", Name::Exact("Lists"))
        .first()
        .expect_visible()
        .await?;
    // Dot segments collapse to a protocol-relative URL: it must stay on this origin.
    for next in [
        "/.//evil.example/x",
        "/..//evil.example",
        "/%2e//evil.example",
    ] {
        page.goto(&format!("/login?next={}", enc(next))).await?;
        page.expect_url(Name::Exact("/")).await?;
    }
    page.goto(&format!("/login?next={}", enc("/safe-path-404")))
        .await?;
    page.text("Nothing here").first().expect_visible().await?;
    Ok(())
}

#[tokio::test]
async fn the_developer_docs_and_the_legal_pages_pass_axe_for_guests_and_members() -> Result<()> {
    let guest = Ctx::new().await?;
    let page = guest.page().await?;
    for (path, heading) in [
        ("/developers", "Bots & API"),
        ("/terms", "Terms of Service"),
        ("/privacy", "Privacy Policy"),
    ] {
        page.goto(path).await?;
        page.role("heading", Name::Exact(heading))
            .first()
            .expect_visible()
            .await?;
        clean(&page, &format!("{path} as a guest")).await?;
    }

    let (_ctx, member, _who) = signed_in().await?;
    for (path, heading) in [
        ("/developers", "Developers"),
        ("/terms", "Terms of Service"),
        ("/privacy", "Privacy Policy"),
    ] {
        member.goto(path).await?;
        member
            .role("heading", Name::Exact(heading))
            .first()
            .expect_visible()
            .await?;
        member
            .role("navigation", Name::Exact("Main"))
            .first()
            .expect_visible()
            .await?;
        clean(&member, &format!("{path} as a member")).await?;
    }
    Ok(())
}
