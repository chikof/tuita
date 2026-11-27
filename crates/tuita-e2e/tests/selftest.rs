//! Tests the harness itself against a tiny axum page, without the tuita app. Needs Chromium
//! (`E2E_CHROME`, or the nix system one); skips itself when there is none.

use std::time::Duration;

use anyhow::Context as _;
use axum::{
    Json, Router,
    http::{HeaderMap, header},
    response::{Html, IntoResponse},
    routing::get,
};
use serde_json::{Value, json};
use tuita_e2e::{
    Ctx, CtxOptions, Motion, Name, Result, Scheme, Violation, chrome_available,
    helpers::run_js_unit,
};

const HOME: &str = r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Self test</title></head><body>
<main>
<h1>Self test</h1>
<nav aria-label="Main"><a href="/clean">Clean page</a></nav>
<form id="f" onsubmit="event.preventDefault(); document.getElementById('hello').textContent = 'Hello, ' + document.getElementById('name').value + '!'">
  <label for="name">Display name</label><input id="name">
  <label><input type="checkbox" id="agree"> I agree</label>
  <label for="choice">Choice</label>
  <select id="choice"><option value="a">Alpha</option><option value="b">Beta</option></select>
  <button type="submit">Greet</button>
</form>
<p id="hello"></p>
<ul aria-label="Items"><li>First item</li><li>Second <b>item</b></li></ul>
<button id="add" onclick="const li = document.createElement('li'); li.textContent = 'Added item'; document.querySelector('ul').append(li)">Add item</button>
<button id="open" onclick="document.getElementById('dlg').showModal()">Open dialog</button>
<dialog id="dlg" aria-label="Sample dialog"><p>Inside dialog</p><button onclick="this.closest('dialog').close()">Close dialog</button></dialog>
<label for="up">Upload</label>
<input type="file" id="up" onchange="document.getElementById('files').textContent = [...this.files].map((f) => f.name + ':' + f.size).join(',')">
<p id="files"></p>
<button onclick="fetch('/echo').then((r) => r.json()).then((j) => { document.getElementById('seen').textContent = j.xff })">Echo address</button>
<p id="seen"></p>
<button onclick="console.error('boom from page')">Log error</button>
<button id="hov" onmouseover="this.dataset.hovered = 'yes'">Hover me</button>
<div hidden>Hidden text</div>
<p id="marks">Text with &#x2068;isolation&#x2069; marks</p>
<p id="late"></p>
<script>setTimeout(() => { document.getElementById('late').textContent = 'Arrived late' }, 400)</script>
</main></body></html>"#;

const ISLAND: &str = r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Island</title>
<script type="module" data-js="demo" src="/demo.js"></script></head><body><main><h1>Island</h1></main></body></html>"#;

const DEMO_JS: &str = "export const add = (a, b) => a + b;";

const CLEAN: &str = r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Clean</title></head>
<body><main><h1>Clean page</h1><p>Nothing to see.</p></main></body></html>"#;

const BAD: &str = r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Bad</title></head>
<body><main><h1>Bad page</h1><img src="data:image/gif;base64,R0lGODlhAQABAAAAACH5BAEKAAEALAAAAAABAAEAAAICTAEAOw=="></main></body></html>"#;

const NOISY: &str = r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Noisy</title></head>
<body><main><h1>Noisy page</h1></main>
<script>console.error('boom on load'); fetch('/missing'); setTimeout(() => { throw new Error('uncaught thing') }, 0)</script></body></html>"#;

const KEYS: &str = r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Keys</title></head>
<body><main><h1>Keys</h1><label for="k">Typed</label><input id="k"><p id="log"></p>
<script>document.getElementById('k').addEventListener('keydown', (e) => { document.getElementById('log').textContent += e.key + (e.ctrlKey ? '+ctrl' : '') + ' ' })</script>
</main></body></html>"#;

async fn echo(headers: HeaderMap) -> impl IntoResponse {
    let get = |name: &str| {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_owned()
    };
    Json(
        json!({"xff": get("x-forwarded-for"), "lang": get("accept-language"), "cookie": get("cookie"), "origin": get("origin")}),
    )
}

async fn echo_body(Json(body): Json<Value>) -> impl IntoResponse {
    Json(json!({"got": body}))
}

/// Serves the pages on 127.0.0.1 and returns the `localhost` origin: WebAuthn refuses IP literals.
async fn serve() -> Result<String> {
    let app = Router::new()
        .route("/", get(|| async { Html(HOME) }))
        .route("/clean", get(|| async { Html(CLEAN) }))
        .route("/bad", get(|| async { Html(BAD) }))
        .route("/noisy", get(|| async { Html(NOISY) }))
        .route(
            "/island",
            get(|| async {
                (
                    [(
                        header::CONTENT_SECURITY_POLICY,
                        "default-src 'self'; script-src 'self' 'unsafe-eval'",
                    )],
                    Html(ISLAND),
                )
            }),
        )
        .route(
            "/demo.js",
            get(|| async { ([(header::CONTENT_TYPE, "text/javascript")], DEMO_JS) }),
        )
        .route("/keys", get(|| async { Html(KEYS) }))
        .route("/healthz", get(|| async { "ok" }))
        .route("/echo", get(echo).post(echo_body))
        .route(
            "/api/v1/ping",
            get(|| async { Json(json!({"pong": true})) }),
        )
        .route(
            "/login",
            get(|| async { ([(header::SET_COOKIE, "sid=abc123; Path=/")], "signed in") }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    tokio::spawn(async move { axum::serve(listener, app).await });
    Ok(format!("http://localhost:{port}"))
}

macro_rules! needs_chrome {
    () => {
        if !chrome_available() {
            eprintln!("skipping: no Chromium (set E2E_CHROME)");
            return Ok(());
        }
    };
}

#[tokio::test]
async fn finders_actions_and_expectations() -> Result<()> {
    needs_chrome!();
    let base = serve().await?;
    let ctx = Ctx::with_base(&base).await?;
    let page = ctx.page().await?;
    page.goto("/").await?;
    assert!(page.url().await?.starts_with(&base));

    // finders
    page.role("heading", Name::Exact("Self test"))
        .expect_visible()
        .await?;
    page.role("navigation", Name::Exact("Main"))
        .role("link", "clean")
        .expect_visible()
        .await?;
    page.label("Display name").expect_visible().await?;
    page.text("Second item").expect_visible().await?;
    page.role("list", Name::Exact("Items"))
        .expect_visible()
        .await?;
    page.role("list", Name::Exact("Items"))
        .role("listitem", Name::Any)
        .expect_count(2)
        .await?;
    page.role("listitem", Name::Any)
        .nth(1)
        .expect_text(Name::Exact("Second item"))
        .await?;
    page.role("listitem", Name::Any)
        .last()
        .expect_text(Name::Contains("SECOND"))
        .await?;
    page.role("listitem", Name::Any)
        .has_text("First")
        .expect_count(1)
        .await?;
    page.role("list", Name::Any)
        .has(&page.css("b"))
        .expect_count(1)
        .await?;
    page.css("b")
        .within(&page.role("list", Name::Any))
        .expect_text(Name::Exact("item"))
        .await?;
    page.text(Name::Re("^Second\\s+item$"))
        .expect_visible()
        .await?;
    page.text(Name::ReI("^second item$"))
        .expect_visible()
        .await?;
    page.text("Hidden text").expect_hidden().await?;
    page.text("Hidden text").expect_count(0).await?;
    page.css("div[hidden]").expect_hidden().await?;
    page.css("#marks")
        .expect_text(Name::Exact("Text with isolation marks"))
        .await?;
    page.css("#marks")
        .expect_attr("id", Name::Exact("marks"))
        .await?;
    assert_eq!(
        page.css("#marks").text_content().await?,
        "Text with isolation marks"
    );

    // buttons, fill, submit by Enter, strictness
    page.label("Display name").fill("Ana").await?;
    page.label("Display name")
        .expect_value(Name::Exact("Ana"))
        .await?;
    page.role("button", Name::Exact("Greet")).click().await?;
    page.css("#hello")
        .expect_text(Name::Exact("Hello, Ana!"))
        .await?;
    page.label("Display name").fill("").await?;
    page.label("Display name")
        .expect_value(Name::Exact(""))
        .await?;
    page.label("Display name").press_sequentially("Bo").await?;
    page.label("Display name").press("Enter").await?;
    page.css("#hello")
        .expect_text(Name::Exact("Hello, Bo!"))
        .await?;
    page.role("button", Name::Exact("Add item")).click().await?;
    page.role("listitem", Name::Exact("Added item"))
        .expect_visible()
        .await?;
    page.role("list", Name::Exact("Items"))
        .role("listitem", Name::Any)
        .expect_count(3)
        .await?;

    page.set_timeout(Duration::from_millis(400));
    let error = page
        .role("listitem", Name::Any)
        .click()
        .await
        .err()
        .map(|e| format!("{e:#}"));
    let error = error.unwrap_or_default();
    assert!(error.contains("strict mode violation: 3"), "{error}");
    let error = page
        .role("button", Name::Exact("Nope"))
        .click()
        .await
        .err()
        .map(|e| format!("{e:#}"))
        .unwrap_or_default();
    assert!(
        error.contains("no element matches") && error.contains("Nope"),
        "{error}"
    );
    page.set_timeout(Duration::from_secs(10));

    // checkbox, select
    page.label("I agree").check().await?;
    page.label("I agree").expect_checked(true).await?;
    page.label("I agree").uncheck().await?;
    page.label("I agree").expect_checked(false).await?;
    page.role("checkbox", "agree").check().await?;
    page.role("combobox", "Choice").select("Beta").await?;
    page.role("combobox", "Choice")
        .expect_value(Name::Exact("b"))
        .await?;
    page.label("Choice").select("a").await?;
    page.label("Choice").expect_value(Name::Exact("a")).await?;

    // dialog
    page.role("dialog", Name::Any).expect_hidden().await?;
    page.role("button", Name::Exact("Open dialog"))
        .click()
        .await?;
    page.role("dialog", Name::Exact("Sample dialog"))
        .expect_visible()
        .await?;
    page.role("button", Name::Exact("Close dialog"))
        .click()
        .await?;
    page.role("dialog", Name::Any).expect_hidden().await?;

    // hover, late content, eval
    page.role("button", Name::Exact("Hover me")).hover().await?;
    page.css("#hov")
        .expect_attr("data-hovered", Name::Exact("yes"))
        .await?;
    page.text("Arrived late").expect_visible().await?;
    assert_eq!(page.eval("1 + 1").await?, json!(2));
    assert_eq!(
        page.eval("(async () => ({ a: [1, 'x'] }))()").await?,
        json!({"a": [1, "x"]})
    );
    assert_eq!(page.eval_fn("(n) => n * 2", &json!(21)).await?, json!(42));
    let thrown = page
        .eval("(() => { throw new Error('kaboom') })()")
        .await
        .err()
        .map(|e| format!("{e:#}"))
        .unwrap_or_default();
    assert!(thrown.contains("kaboom"), "{thrown}");
    page.css("button")
        .has_text("Add item")
        .expect_css("cursor", Name::Any)
        .await?;
    assert!(page.css("#hov").bounding_box().await?.width > 0.0);
    page.wait_for_js("document.title === 'Self test'").await?;

    // navigation
    page.role("link", "Clean page").click().await?;
    page.expect_url(Name::Exact("/clean")).await?;
    page.role("heading", Name::Exact("Clean page"))
        .expect_visible()
        .await?;
    page.reload().await?;
    page.expect_url(Name::Re("/clean$")).await?;
    Ok(())
}

#[tokio::test]
async fn files_and_keys() -> Result<()> {
    needs_chrome!();
    let base = serve().await?;
    let ctx = Ctx::with_base(&base).await?;
    let page = ctx.page().await?;
    page.goto("/").await?;
    let path = std::env::temp_dir().join(format!("tuita-e2e-selftest-{}.txt", std::process::id()));
    tokio::fs::write(&path, "hello").await?;
    page.label("Upload")
        .set_files(&[&path])
        .await
        .context("set_files")?;
    page.css("#files")
        .expect_text(Name::Contains("tuita-e2e-selftest"))
        .await?;
    page.css("#files").expect_text(Name::Re(":5$")).await?;
    page.label("Upload")
        .set_file_bytes("photo.jpg", &[1, 2, 3])
        .await?;
    page.css("#files")
        .expect_text(Name::Exact("photo.jpg:3"))
        .await?;
    page.label("Upload")
        .set_files::<&str>(&[])
        .await
        .context("clear")?;
    page.css("#files").expect_text(Name::Exact("")).await?;
    tokio::fs::remove_file(&path).await?;

    page.goto("/keys").await?;
    let input = page.label("Typed");
    input.press("Shift+Tab").await?;
    input.press("Control+k").await?;
    input.press("a").await?;
    input.press("Escape").await?;
    page.css("#log")
        .expect_text(Name::Exact("Tab k+ctrl a Escape"))
        .await?;
    Ok(())
}

#[tokio::test]
async fn contexts_get_their_own_address_and_cookies() -> Result<()> {
    needs_chrome!();
    let base = serve().await?;
    let one = Ctx::with_base(&base).await?;
    let two = Ctx::with_base(&base).await?;
    assert_ne!(one.ip(), two.ip());
    assert!(one.ip().starts_with("10."), "{}", one.ip());

    let page = one.page().await?;
    page.goto("/").await?;
    page.role("button", Name::Exact("Echo address"))
        .click()
        .await?;
    page.css("#seen").expect_text(Name::Exact(one.ip())).await?;

    let (status, body) = two.fetch("GET", "/echo", None).await?;
    assert_eq!(status, 200);
    assert_eq!(body["xff"], two.ip());
    let (status, body) = one.call("GET", "/ping", None).await?;
    assert_eq!((status, &body["pong"]), (200, &json!(true)));
    let (status, body) = one.fetch("POST", "/echo", Some(&json!({"a": 1}))).await?;
    assert_eq!((status, &body["got"]), (200, &json!({"a": 1})));
    assert!(body["got"].is_object());

    // cookies stay in their context and travel to a fork
    assert_eq!(
        one.fetch("GET", "/login", None).await?.1,
        json!("signed in")
    );
    let (_, mine) = one.fetch("GET", "/echo", None).await?;
    assert!(
        mine["cookie"]
            .as_str()
            .unwrap_or_default()
            .contains("sid=abc123"),
        "{mine}"
    );
    let (_, theirs) = two.fetch("GET", "/echo", None).await?;
    assert_eq!(theirs["cookie"], "");
    let copy = one.fork().await?;
    let (_, copied) = copy.fetch("GET", "/echo", None).await?;
    assert!(
        copied["cookie"]
            .as_str()
            .unwrap_or_default()
            .contains("sid=abc123"),
        "{copied}"
    );
    assert_ne!(copy.ip(), one.ip());
    one.close().await?;
    Ok(())
}

#[tokio::test]
async fn console_errors_are_collected() -> Result<()> {
    needs_chrome!();
    let base = serve().await?;
    let ctx = Ctx::with_base(&base).await?;
    let page = ctx.page().await?;
    page.goto("/clean").await?;
    assert!(
        page.console_errors().await.is_empty(),
        "favicon 404 must not count: {:?}",
        page.console_errors().await
    );

    page.goto("/noisy").await?;
    page.wait_for_js("document.readyState === 'complete'")
        .await?;
    let errors = page.console_errors().await;
    let all = errors.join("\n");
    assert!(all.contains("boom on load"), "{all}");
    assert!(all.contains("uncaught thing"), "{all}");
    assert!(all.contains("404") && all.contains("/missing"), "{all}");
    assert!(!all.contains("favicon"), "{all}");
    page.clear_console_errors();
    assert!(page.console_errors().await.is_empty());

    page.goto("/").await?;
    page.role("button", Name::Exact("Log error"))
        .click()
        .await?;
    let errors = page.console_errors().await;
    assert!(
        errors.iter().any(|e| e.contains("boom from page")),
        "{errors:?}"
    );
    Ok(())
}

#[tokio::test]
async fn axe_reports_violations() -> Result<()> {
    needs_chrome!();
    let base = serve().await?;
    let ctx = Ctx::with_base(&base).await?;
    let page = ctx.page().await?;

    page.goto("/clean").await?;
    let clean = page.axe().await?;
    assert!(clean.is_empty(), "{}", Violation::report(&clean));

    page.goto("/bad").await?;
    let found = page.axe().await?;
    let image = found.iter().find(|v| v.id == "image-alt");
    assert!(
        image.is_some(),
        "expected image-alt in {}",
        Violation::report(&found)
    );
    let image = image.ok_or_else(|| anyhow::anyhow!("unreachable"))?;
    assert_eq!(image.nodes.len(), 1);
    assert!(
        image.nodes[0].target.contains("img"),
        "{:?}",
        image.nodes[0]
    );
    assert!(Violation::report(&found).contains("image-alt"));
    // a scan limited to a subtree without the image is clean
    assert!(page.axe_with(Some("h1"), &["wcag2a"]).await?.is_empty());
    Ok(())
}

#[tokio::test]
async fn emulation_viewport_screenshots_and_options() -> Result<()> {
    needs_chrome!();
    let base = serve().await?;
    let ctx = Ctx::with(CtxOptions {
        base: Some(base.clone()),
        locale: Some("pl-PL".to_owned()),
        viewport: Some((1000, 700)),
        touch: true,
    })
    .await?;
    let page = ctx.page().await?;
    page.goto("/").await?;
    assert_eq!(page.eval("innerWidth").await?, json!(1000));
    assert_eq!(page.eval("navigator.language").await?, json!("pl-PL"));
    assert_eq!(page.eval("'ontouchstart' in window").await?, json!(true));
    let (_, seen) = ctx.fetch("GET", "/echo", None).await?;
    assert!(
        seen["lang"]
            .as_str()
            .unwrap_or_default()
            .starts_with("pl-PL"),
        "{seen}"
    );

    page.set_viewport(390, 844).await?;
    assert_eq!(
        page.eval("[innerWidth, innerHeight]").await?,
        json!([390, 844])
    );
    page.emulate(Scheme::Dark, Motion::Reduce).await?;
    assert_eq!(page.eval("matchMedia('(prefers-color-scheme: dark)').matches && matchMedia('(prefers-reduced-motion: reduce)').matches").await?, json!(true));
    page.emulate(Scheme::Light, Motion::NoPreference).await?;
    assert_eq!(page.eval("matchMedia('(prefers-color-scheme: light)').matches && !matchMedia('(prefers-reduced-motion: reduce)').matches").await?, json!(true));

    page.init_script("window.__seen = 'init-ran'").await?;
    page.reload().await?;
    assert_eq!(page.eval("window.__seen").await?, json!("init-ran"));

    let dir = std::env::temp_dir().join(format!("tuita-e2e-shots-{}", std::process::id()));
    let shot = dir.join("nested/page.png");
    page.screenshot(&shot).await?;
    let full = dir.join("full.png");
    page.screenshot_full(&full).await?;
    let element = dir.join("element.png");
    page.role("heading", Name::Exact("Self test"))
        .screenshot(&element)
        .await?;
    for path in [&shot, &full, &element] {
        let bytes = tokio::fs::read(path).await?;
        assert_eq!(&bytes[..4], b"\x89PNG", "{}", path.display());
    }
    tokio::fs::remove_dir_all(&dir).await?;
    Ok(())
}

#[tokio::test]
async fn virtual_authenticator_makes_passkeys() -> Result<()> {
    needs_chrome!();
    let base = serve().await?;
    let ctx = Ctx::with_base(&base).await?;
    let page = ctx.page().await?;
    page.goto("/").await?;
    let auth = page.virtual_authenticator().await?;
    let made = page
        .eval(
            "(async () => { const c = await navigator.credentials.create({ publicKey: { challenge: new Uint8Array(32), rp: { name: 't', id: location.hostname }, user: { id: new Uint8Array(8), name: 'u', displayName: 'u' }, pubKeyCredParams: [{ type: 'public-key', alg: -7 }], authenticatorSelection: { residentKey: 'required', userVerification: 'required' } } }); return c.type })()",
        )
        .await?;
    assert_eq!(made, "public-key");
    assert_eq!(auth.credentials().await?, 1);
    auth.remove().await?;
    Ok(())
}

#[tokio::test]
async fn js_unit_vectors_run_against_island_modules() -> Result<()> {
    needs_chrome!();
    let base = serve().await?;
    let ctx = Ctx::with_base(&base).await?;
    let page = ctx.page().await?;
    page.bypass_csp().await?;
    page.goto("/island").await?;

    let good = "export async function run(mod) { if (mod.add(2, 3) !== 5) throw new Error('add is wrong'); }";
    run_js_unit(&page, "demo", good).await?;

    let bad = "export async function run(mod) { if (mod.add(2, 3) !== 6) throw new Error('known answer differs'); }";
    let error = run_js_unit(&page, "demo", bad)
        .await
        .err()
        .map(|e| format!("{e:#}"))
        .unwrap_or_default();
    assert!(error.contains("known answer differs"), "{error}");

    let error = run_js_unit(&page, "missing", good)
        .await
        .err()
        .map(|e| format!("{e:#}"))
        .unwrap_or_default();
    assert!(
        error.contains("one <script data-js=\"missing\">"),
        "{error}"
    );
    Ok(())
}
