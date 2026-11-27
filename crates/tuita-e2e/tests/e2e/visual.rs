//! Browser tests: the visual suite. Every page is opened at two widths,
//! in two themes and with two motion settings, and checked for horizontal scroll, headers hidden
//! under the top bar, tablists spilling out of their boxes, text spilling out of its box (language
//! pages), the endpoint table, axe violations (1280 px, motion on) and console errors. A full-page
//! screenshot of every combination goes to `target/e2e/visual/<name>-<width>-<theme>-<motion>.png`.
//!
//! **One test per page.** The 8 combinations of a page (2 for a language page) run concurrently inside
//! one test, behind a shared limit of concurrent browser contexts, and every failure is collected and
//! reported as `<name> <width> <theme> <motion>: <what>`, so a failure names its combination without
//! 550 test functions. A page's prefix is its name: `scripts/e2e.sh visual::consent` runs the three
//! consent pages.
//!
//! **Real conditions, no request stubs.** The consent card comes from
//! real `/oauth/authorize` requests of registered apps. Remote pages come from a second tuita
//! instance (`world()`), a tiny ActivityPub server that publishes a locked account and an inbox that
//! fails, and a closed port. The sign-up page that asks for an invite and the empty federation tab
//! are the second instance's own pages. The banner is a real announcement and the busy federation tab
//! is real deliveries that fail.

use std::{
    collections::HashMap,
    future::{Future, IntoFuture},
    path::PathBuf,
    rc::Rc,
    sync::LazyLock,
    time::Duration,
};

use anyhow::Context as _;
use axum::{
    Router,
    extract::{Path, Query},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use futures_util::future::{FutureExt as _, LocalBoxFuture, join_all};
use serde_json::{Value, json};
use tokio::{
    net::TcpListener,
    runtime::Builder,
    sync::{Mutex, OnceCell, Semaphore},
};
use tuita_e2e::{
    Ctx, Loc, Name, Page, Result, Violation,
    helpers::{
        Peer, Seeded, call, handle, make_image, new_admin, promote_on, seed, sign_up_via_api,
        start_peer_on,
    },
};
use uuid::Uuid;

const WIDTHS: [u32; 2] = [1280, 390];
const THEMES: [&str; 2] = ["dark", "light"];
const MOTIONS: [&str; 2] = ["motion", "reduced"];

/// Contexts open at once in this process: Chromium renders every full-page screenshot.
static SLOTS: Semaphore = Semaphore::const_new(6);

static PROMOTIONS: Mutex<()> = Mutex::const_new(());

// ---- what a page is opened as

/// An async step on the page, shared by the combinations of one test.
type Hook = Rc<dyn Fn(Page) -> LocalBoxFuture<'static, Result<()>>>;

fn hook<F, Fut>(step: F) -> Hook
where
    F: Fn(Page) -> Fut + 'static,
    Fut: Future<Output = Result<()>> + 'static,
{
    Rc::new(move |page| step(page).boxed_local())
}

/// Who opens the page.
enum Viewer<'a> {
    /// Nobody is signed in, on the instance under test.
    Guest,
    /// Nobody is signed in, on another instance.
    GuestOn(String),
    /// A copy of this session's cookies: the same user, a new browser.
    Session(&'a Ctx),
    /// A brand new admin account per combination, for pages behind a per-user rate limit.
    NewAdmin,
}

struct Spec<'a> {
    name: String,
    path: String,
    viewer: Viewer<'a>,
    /// Interface language, stored in the browser; the page is also checked for text overflowing its boxes.
    locale: Option<&'static str>,
    /// Runs before the page opens (a virtual authenticator).
    before: Option<Hook>,
    /// Runs after the page settled, before the screenshot.
    prepare: Option<Hook>,
    /// Console errors the page is meant to log: a browser noting a failure response it was sent on purpose.
    tolerated: Vec<&'static str>,
}

impl<'a> Spec<'a> {
    fn new(name: &str, path: impl Into<String>, viewer: Viewer<'a>) -> Self {
        Self {
            name: name.to_owned(),
            path: path.into(),
            viewer,
            locale: None,
            before: None,
            prepare: None,
            tolerated: Vec::new(),
        }
    }

    fn locale(mut self, code: &'static str) -> Self {
        self.locale = Some(code);
        self
    }

    fn before<F, Fut>(mut self, step: F) -> Self
    where
        F: Fn(Page) -> Fut + 'static,
        Fut: Future<Output = Result<()>> + 'static,
    {
        self.before = Some(hook(step));
        self
    }

    fn prepare<F, Fut>(mut self, step: F) -> Self
    where
        F: Fn(Page) -> Fut + 'static,
        Fut: Future<Output = Result<()>> + 'static,
    {
        self.prepare = Some(hook(step));
        self
    }

    fn tolerate(mut self, error: &'static str) -> Self {
        self.tolerated.push(error);
        self
    }
}

// ---- the checks, as JS that returns the findings

/// The film grain covers the whole document.
const GRAIN: &str = r"(() => {
    if (!document.body.classList.contains('grain')) return [];
    const layer = parseFloat(getComputedStyle(document.body, '::before').height);
    const page = document.documentElement.scrollHeight;
    return layer >= page - 1 ? [] : [`grain layer ${layer} is shorter than the page ${page}`];
})()";

/// Page headers are not partly under the top bar, at the top and at the end of the page.
const HEADERS_UNDER_TOP_BAR: &str = r"(() => {
    const bar = document.querySelector('.top');
    if (!bar || getComputedStyle(bar).display === 'none') return [];
    const edge = bar.getBoundingClientRect().bottom;
    const covered = () =>
        [...document.querySelectorAll('header.bar')].flatMap((h) => {
            const r = h.getBoundingClientRect();
            return r.bottom > edge && r.top < edge - 0.5 ? [`${r.top} < ${edge}`] : [];
        });
    const found = covered();
    scrollTo(0, document.documentElement.scrollHeight);
    const atEnd = covered();
    scrollTo(0, 0);
    return [...found, ...atEnd];
})()";

const HORIZONTAL_SCROLL: &str = r"(() => {
    const over = document.documentElement.scrollWidth - document.documentElement.clientWidth;
    return over <= 0 ? [] : [`${over}px of horizontal scroll`];
})()";

/// Tablists stay inside the content box of every ancestor. A tablist in a closed dialog has no box.
const TABLIST_SPILLS: &str = r"[...document.querySelectorAll('[role=tablist]')].filter((el) => el.checkVisibility()).flatMap((el) => {
    const right = el.getBoundingClientRect().right;
    let edge = Infinity;
    for (let box = el.parentElement; box; box = box.parentElement) {
        const style = getComputedStyle(box);
        if (style.display === 'contents' || style.display === 'inline') continue;
        const content =
            box.getBoundingClientRect().right -
            parseFloat(style.paddingRight) -
            parseFloat(style.borderRightWidth);
        edge = Math.min(edge, content);
    }
    return right > edge + 0.5 ? [`${el.getAttribute('aria-label')}: ${right} > ${edge}`] : [];
})";

/// The bar chart's date labels do not overlap.
const CHART_TICKS: &str = r"(() => {
    const ticks = [...document.querySelectorAll('.ticks span')]
        .map((el) => el.getBoundingClientRect())
        .map((r) => [r.left, r.right]);
    return ticks
        .filter(([left], i) => i > 0 && left < ticks[i - 1][1] - 0.5)
        .map(([left], i) => `date label at ${left} overlaps the one before`);
})()";

/// Endpoint table cells keep their words whole: a method or scope on one line, a path on at most
/// `limit` lines. Text only: a `<wbr>` reports a box of its own that is not a line.
const ENDPOINT_CELLS: &str = r"((limit) => {
    const lines = (el) => {
        const tops = new Set();
        const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
        for (let node = walker.nextNode(); node; node = walker.nextNode()) {
            const range = document.createRange();
            range.selectNodeContents(node);
            for (const rect of range.getClientRects()) tops.add(Math.round(rect.top));
        }
        return tops.size;
    };
    const found = [];
    for (const el of document.querySelectorAll('td.method, td.scope'))
        if (lines(el) > 1) found.push(`${el.textContent?.trim()} breaks inside a word`);
    for (const el of document.querySelectorAll('td.path code'))
        if (lines(el) > limit) found.push(`${el.textContent} wraps into ${lines(el)} lines`);
    return found;
})";

/// Staggered entrances (landing, admin bento) are transparent until their turn.
const ENTRANCES_DONE: &str = r"[...document.querySelectorAll('.enter')].every((el) =>
    el.getAnimations().every((a) => a.playState === 'finished'))";

const SOMETHING_VISIBLE: &str = r"[...document.body.querySelectorAll('*')].some(
    (el) => !['SCRIPT', 'STYLE'].includes(el.tagName) && el.checkVisibility())";

fn strings(found: &Value) -> Vec<String> {
    found
        .as_array()
        .map(|all| {
            all.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn screenshots() -> PathBuf {
    let target = std::env::var_os("CARGO_TARGET_DIR").map_or_else(
        || PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../target")),
        PathBuf::from,
    );
    target.join("e2e").join("visual")
}

async fn settle(page: &Page) -> Result<()> {
    page.wait_for_js(SOMETHING_VISIBLE).await?;
    page.css(r#"[role="list"][aria-busy="true"]"#)
        .expect_count(0)
        .await?;
    page.eval("document.fonts.ready.then(() => true)").await?;
    page.wait_for_js(ENTRANCES_DONE).await
}

/// One page in one browser: the findings of every check, or an error when the page did not get that far.
async fn visit(spec: &Spec<'_>, width: u32, theme: &str, motion: &str) -> Result<Vec<String>> {
    let ctx = match &spec.viewer {
        Viewer::Guest => Ctx::new().await?,
        Viewer::GuestOn(base) => Ctx::with_base(base).await?,
        Viewer::Session(session) => session.fork().await?,
        Viewer::NewAdmin => {
            // `admin promote` is a process with its own database pool: one at a time keeps the
            // peak of connections low when many runs share one Postgres.
            let _one = PROMOTIONS.lock().await;
            new_admin("flt").await?.0
        }
    };
    let page = ctx.page().await?;
    page.set_viewport(width, 844).await?;
    let prefs = json!({
        "mode": theme, "ink": "bone", "contrast": "normal", "textScale": 1,
        "motion": if motion == "reduced" { "reduce" } else { "system" },
    });
    page.init_script(&format!(
        "localStorage.setItem('tuita:prefs', JSON.stringify({prefs}))"
    ))
    .await?;
    if let Some(code) = spec.locale {
        // An account without a language of its own and a guest both follow this cookie.
        page.goto("/healthz").await?;
        page.eval(&format!(
            "document.cookie = 'tuita_lang={code}; Path=/; Max-Age=3600'"
        ))
        .await?;
    }
    if let Some(before) = &spec.before {
        before(page.clone()).await?;
    }
    page.goto(&spec.path).await.context("opening the page")?;
    settle(&page)
        .await
        .context("waiting for the page to settle")?;
    if let Some(prepare) = &spec.prepare {
        prepare(page.clone()).await.context("preparing the page")?;
    }
    page.css("html")
        .expect_attr("data-theme", Name::Exact(theme))
        .await?;
    if spec.name == "404" {
        page.text(Name::Exact("Nothing here"))
            .first()
            .expect_visible()
            .await?;
    }

    let mut found = strings(&page.eval(GRAIN).await?);
    if width == 390 {
        for line in strings(&page.eval(HEADERS_UNDER_TOP_BAR).await?) {
            found.push(format!("page header is partly under the top bar: {line}"));
        }
    }
    found.extend(strings(&page.eval(HORIZONTAL_SCROLL).await?));
    if let Some(code) = spec.locale {
        page.css("html")
            .expect_attr("lang", Name::Exact(code))
            .await?;
        for line in tuita_e2e::helpers::overflowing(&page).await? {
            found.push(format!("{code} text leaves its box: {line}"));
        }
    }
    for line in strings(&page.eval(TABLIST_SPILLS).await?) {
        found.push(format!("tablist leaves every ancestor content box: {line}"));
    }
    if spec.name == "admin" {
        found.extend(strings(&page.eval(CHART_TICKS).await?));
    }
    if spec.name.starts_with("developers") {
        let limit = if width == 1280 { 1 } else { 2 };
        found.extend(strings(
            &page.eval(&format!("({ENDPOINT_CELLS})({limit})")).await?,
        ));
    }
    if width == 1280 && motion == "motion" {
        let violations = page.axe().await.context("running axe")?;
        if !violations.is_empty() {
            found.push(format!(
                "axe wcag2a/aa:\n{}",
                Violation::report(&violations)
            ));
        }
    }
    page.screenshot_full(screenshots().join(format!("{}-{width}-{theme}-{motion}.png", spec.name)))
        .await
        .context("taking the screenshot")?;
    for error in page.console_errors().await {
        if !spec.tolerated.iter().any(|ok| error.contains(ok)) {
            found.push(format!("console: {error}"));
        }
    }
    ctx.close().await?;
    Ok(found)
}

/// Every combination of the page; fails listing each combination that has findings.
async fn run(spec: &Spec<'_>) -> Result<()> {
    let mut combinations = Vec::new();
    for width in WIDTHS {
        for theme in THEMES {
            for motion in MOTIONS {
                // A language page only needs one theme and motion: the check is about box metrics.
                if spec.locale.is_some() && (theme != "dark" || motion != "motion") {
                    continue;
                }
                combinations.push((width, theme, motion));
            }
        }
    }
    let total = combinations.len();
    let outcomes = join_all(
        combinations
            .into_iter()
            .map(|(width, theme, motion)| async move {
                let label = format!("{} {width} {theme} {motion}", spec.name);
                let _slot = SLOTS.acquire().await;
                match visit(spec, width, theme, motion).await {
                    Ok(found) => found
                        .into_iter()
                        .map(|line| format!("{label}: {line}"))
                        .collect::<Vec<_>>(),
                    Err(error) => vec![format!("{label}: {error:#}")],
                }
            }),
    )
    .await;
    let failures: Vec<String> = outcomes.into_iter().flatten().collect();
    assert!(
        failures.is_empty(),
        "{} of {total} combinations of {} have findings:\n{}",
        failures.len(),
        spec.name,
        failures.join("\n")
    );
    Ok(())
}

/// The checks are only worth something if they find what they look for.
#[tokio::test]
async fn the_checks_report_sideways_scroll_a_tablist_spill_overlapping_ticks_and_a_broken_endpoint_word()
-> Result<()> {
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    page.goto("/healthz").await?;
    let found = |script: String| {
        let page = page.clone();
        async move { page.eval(&script).await.map(|v| strings(&v)) }
    };
    assert_eq!(
        found(HORIZONTAL_SCROLL.to_owned()).await?,
        Vec::<String>::new()
    );
    page.eval(
        r##"document.body.innerHTML =
        '<div style="width:100px"><div role="tablist" aria-label="Wide" style="width:300px;height:10px"></div></div>' +
        '<div class="ticks" style="position:relative;height:20px">' +
        '<span style="position:absolute;left:0;width:50px">a</span>' +
        '<span style="position:absolute;left:20px;width:50px">b</span></div>' +
        '<table><tr><td class="method" style="display:block;width:20px;word-break:break-all">DELETE</td></tr></table>' +
        '<div style="width:3000px;height:5px"></div>'"##,
    )
    .await?;
    assert_eq!(found(HORIZONTAL_SCROLL.to_owned()).await?.len(), 1);
    assert_eq!(found(TABLIST_SPILLS.to_owned()).await?.len(), 1);
    assert_eq!(found(CHART_TICKS.to_owned()).await?.len(), 1);
    assert_eq!(
        found(format!("({ENDPOINT_CELLS})(1)")).await?,
        vec!["DELETE breaks inside a word".to_owned()]
    );
    Ok(())
}

// ---- the world the pages need

static MEMBER: OnceCell<Seeded> = OnceCell::const_new();

/// The signed-in, staff-level account every signed-in page opens as (`seed`), made once per process.
async fn member() -> Result<&'static Seeded> {
    MEMBER.get_or_try_init(seed).await
}

/// A second tuita instance, with an account that has a post, and an admin: the home of every remote page.
struct World {
    peer: Peer,
    /// The account `@<mara>@<peer domain>` is "Mara Okafor".
    mara: String,
    /// The admin of the second instance, who signed in before it went invite-only.
    admin: Ctx,
}

const REMOTE_POST: &str = "Hello from another instance. Federated, not cloned.";

static WORLD: OnceCell<World> = OnceCell::const_new();

/// A port nothing listens on right now.
fn free_port() -> Result<u16> {
    Ok(std::net::TcpListener::bind("127.0.0.1:0")?
        .local_addr()?
        .port())
}

async fn world() -> Result<&'static World> {
    WORLD
        .get_or_try_init(|| async {
            let peer = start_peer_on(free_port()?, "visual").await?;
            let ctx = Ctx::with_base(&peer.base).await?;
            let mara = handle("mara");
            sign_up_via_api(&ctx, &mara, "Mara Okafor").await?;
            call(
                &ctx,
                "PATCH",
                "/me",
                Some(&json!({"bio": "Writes about small networks. Federated, not cloned."})),
            )
            .await?;
            let body = json!({"text": REMOTE_POST, "cw": "", "sensitive": false, "mediaIds": []});
            call(&ctx, "POST", "/posts", Some(&body)).await?;
            ctx.close().await?;

            let admin = Ctx::with_base(&peer.base).await?;
            let who = handle("padm");
            sign_up_via_api(&admin, &who, "Peer Admin").await?;
            promote_on(&peer.database, &who).await?;
            // Last: once the instance asks for an invite, nobody else can sign up there.
            call(
                &admin,
                "PATCH",
                "/admin/settings",
                Some(&json!({"signups": "invite"})),
            )
            .await?;
            Ok(World { peer, mara, admin })
        })
        .await
}

/// A tiny ActivityPub server: `locked` approves followers by hand and `broken` has an inbox that
/// answers 500, so a follow of it queues a delivery that keeps failing.
struct Fake {
    domain: String,
}

/// A public key nobody holds the private half of; the server only checks that it parses.
const PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----\n\
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAj+ehPWJ9JUivYAT1KvWN\n\
mvc3k94hmOmO0iHekHS69C8gVBNVNMpAf7xg3PKRgtEGTWvGGUOoacMdtSAkulgQ\n\
Rnj+UJ8/Ew35ErrvA6EleOY8qYqelWNgAz2k6sMmKvmTwR8R09MFnmDUtJylMKdk\n\
6Bjif7Jufm3rkNacYQyboga3kNg9G4GafB9KzxFYZtF30uN+pgCWKQVwuH51mNyq\n\
oehrlcRmgu/l8eTQsHmXFc3B/fVJFvGK93mWwIOiPqLVttJTDr1C43/P8alRjdiN\n\
gOEakJgK67yBWDABTLM4V93an3ggDmM3ibNcFcaM/X7dPR8XXtsiPxGkccUCukgB\n\
7wIDAQAB\n\
-----END PUBLIC KEY-----\n";

fn activity(body: &Value) -> Response {
    let mut response = body.to_string().into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/activity+json"),
    );
    response
}

fn fake_router(origin: String) -> Router {
    let actor_origin = origin.clone();
    let finger_origin = origin;
    Router::new()
        .route(
            "/.well-known/webfinger",
            get(move |Query(query): Query<HashMap<String, String>>| async move {
                let resource = query.get("resource").cloned().unwrap_or_default();
                let name = resource
                    .strip_prefix("acct:")
                    .and_then(|acct| acct.split('@').next())
                    .unwrap_or_default();
                if name != "locked" && name != "broken" {
                    return StatusCode::NOT_FOUND.into_response();
                }
                let mut response = json!({
                    "subject": resource,
                    "links": [{
                        "rel": "self",
                        "type": "application/activity+json",
                        "href": format!("{finger_origin}/users/{name}"),
                    }],
                })
                .to_string()
                .into_response();
                response.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/jrd+json"),
                );
                response
            }),
        )
        .route(
            "/users/{name}",
            get(move |Path(name): Path<String>| async move {
                if name != "locked" && name != "broken" {
                    return StatusCode::NOT_FOUND.into_response();
                }
                let id = format!("{actor_origin}/users/{name}");
                activity(&json!({
                    "@context": "https://www.w3.org/ns/activitystreams",
                    "id": id,
                    "type": "Person",
                    "preferredUsername": name,
                    "name": if name == "locked" { "Noor Hadid" } else { "Unreachable Inbox" },
                    "summary": "<p>Approves every follower by hand.</p>",
                    "url": format!("{actor_origin}/@{name}"),
                    "inbox": format!("{id}/inbox"),
                    "manuallyApprovesFollowers": name == "locked",
                    "publicKey": {"id": format!("{id}#main-key"), "owner": id, "publicKeyPem": PUBLIC_KEY},
                }))
            }),
        )
        .route(
            "/users/{name}/inbox",
            post(|Path(name): Path<String>| async move {
                if name == "broken" {
                    StatusCode::INTERNAL_SERVER_ERROR
                } else {
                    StatusCode::ACCEPTED
                }
            }),
        )
}

/// Starts the fake server on a thread and runtime of its own: a test's runtime ends with the test.
fn start_fake() -> std::result::Result<Fake, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("visual-activitypub".to_owned())
        .spawn(move || {
            let runtime = match Builder::new_current_thread().enable_all().build() {
                Ok(runtime) => runtime,
                Err(error) => {
                    let _ = tx.send(Err(error.to_string()));
                    return;
                }
            };
            runtime.block_on(async move {
                let listener = match TcpListener::bind("127.0.0.1:0").await {
                    Ok(listener) => listener,
                    Err(error) => {
                        let _ = tx.send(Err(error.to_string()));
                        return;
                    }
                };
                let port = listener.local_addr().map(|a| a.port()).unwrap_or_default();
                let _ = tx.send(Ok(port));
                let router = fake_router(format!("http://localhost:{port}"));
                let _ = axum::serve(listener, router).into_future().await;
            });
        })
        .map_err(|error| error.to_string())?;
    let port = rx.recv().map_err(|error| error.to_string())??;
    Ok(Fake {
        domain: format!("localhost:{port}"),
    })
}

fn fake() -> Result<&'static Fake> {
    static FAKE: LazyLock<std::result::Result<Fake, String>> = LazyLock::new(start_fake);
    FAKE.as_ref()
        .map_err(|message| anyhow::anyhow!("starting the ActivityPub server: {message}"))
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

// ---- pages from simple parts

macro_rules! guest {
    ($test:ident, $name:expr, $path:expr) => {
        #[tokio::test]
        async fn $test() -> Result<()> {
            run(&Spec::new($name, $path, Viewer::Guest)).await
        }
    };
}

macro_rules! signed_in {
    ($test:ident, $name:expr, |$seeded:ident| $path:expr) => {
        #[tokio::test]
        async fn $test() -> Result<()> {
            let $seeded = member().await?;
            run(&Spec::new($name, $path, Viewer::Session(&$seeded.ctx))).await
        }
    };
    ($test:ident, $name:expr, $path:expr) => {
        signed_in!($test, $name, |_s| $path);
    };
}

guest!(home_out, "home-out", "/");
guest!(signup, "signup", "/signup");
guest!(design, "design", "/design");
guest!(developers_out, "developers-out", "/developers");
signed_in!(home, "home", "/");
signed_in!(profile, "profile", |s| format!("/@{}", s.handle));
signed_in!(thread, "thread", |s| format!(
    "/@{}/{}",
    s.handle, s.post_id
));
signed_in!(explore, "explore", "/explore");
signed_in!(notifications, "notifications", "/notifications");
signed_in!(settings, "settings", "/settings");
signed_in!(settings_profile, "settings-profile", "/settings/profile");
signed_in!(messages, "messages", "/messages");
signed_in!(conversation, "conversation", |s| format!(
    "/messages/{}",
    s.other
));
signed_in!(lists, "lists", "/lists");
signed_in!(list, "list", |s| format!("/lists/{}", s.list_id));
signed_in!(list_members, "list-members", |s| format!(
    "/lists/{}/members",
    s.list_id
));
signed_in!(scheduled, "scheduled", "/scheduled");
signed_in!(
    settings_notifications,
    "settings-notifications",
    "/settings/notifications"
);
signed_in!(settings_devices, "settings-devices", "/settings/devices");
signed_in!(settings_passkeys, "settings-passkeys", "/settings/passkeys");
signed_in!(
    settings_affiliation,
    "settings-affiliation",
    "/settings/affiliation"
);
signed_in!(settings_bots, "settings-bots", "/settings/bots");
signed_in!(developers, "developers", "/developers");
signed_in!(admin, "admin", "/admin");
signed_in!(admin_reports, "admin-reports", "/admin/reports");
signed_in!(admin_users, "admin-users", "/admin/users");
signed_in!(admin_instance, "admin-instance", "/admin/instance");
signed_in!(
    admin_announcements,
    "admin-announcements",
    "/admin/announcements"
);
/// The browser notes the 404 of the document itself.
#[tokio::test]
async fn not_found() -> Result<()> {
    let seeded = member().await?;
    run(&Spec::new("404", "/nope-404", Viewer::Session(&seeded.ctx)).tolerate("status of 404"))
        .await
}

/// The pages that are checked in other languages (text overflow).
macro_rules! language {
    ($module:ident, $code:literal) => {
        mod $module {
            use super::*;

            #[tokio::test]
            async fn home_out() -> Result<()> {
                run(&Spec::new(concat!("home-out-", $code), "/", Viewer::Guest).locale($code)).await
            }

            async fn signed_in(name: &str, path: &str) -> Result<()> {
                let seeded = member().await?;
                run(&Spec::new(name, path, Viewer::Session(&seeded.ctx)).locale($code)).await
            }

            #[tokio::test]
            async fn home() -> Result<()> {
                signed_in(concat!("home-", $code), "/").await
            }

            #[tokio::test]
            async fn settings() -> Result<()> {
                signed_in(concat!("settings-", $code), "/settings").await
            }

            #[tokio::test]
            async fn settings_profile() -> Result<()> {
                signed_in(concat!("settings-profile-", $code), "/settings/profile").await
            }

            #[tokio::test]
            async fn settings_language() -> Result<()> {
                signed_in(concat!("settings-language-", $code), "/settings/language").await
            }

            #[tokio::test]
            async fn explore() -> Result<()> {
                signed_in(concat!("explore-", $code), "/explore").await
            }

            #[tokio::test]
            async fn notifications() -> Result<()> {
                signed_in(concat!("notifications-", $code), "/notifications").await
            }
        }
    };
}

language!(ja, "ja");
language!(de, "de");
language!(fr, "fr");
language!(pl, "pl");

// ---- pages with something to do first

#[tokio::test]
async fn login() -> Result<()> {
    run(
        &Spec::new("login", "/login", Viewer::Guest).prepare(|page| async move {
            page.role("button", Name::Exact("Sign in with a passkey"))
                .expect_visible()
                .await
        }),
    )
    .await
}

/// An instance that asks for an invite code: the second instance, which went invite-only.
#[tokio::test]
async fn signup_invite() -> Result<()> {
    let world = world().await?;
    let spec = Spec::new(
        "signup-invite",
        "/signup",
        Viewer::GuestOn(world.peer.base.clone()),
    )
    .prepare(|page| async move { page.label("Invite code").expect_visible().await });
    run(&spec).await
}

fn image_input(page: &Page) -> Loc {
    page.css(r#"input[type=file][accept*="image/png"]"#).first()
}

#[tokio::test]
async fn composer_media() -> Result<()> {
    let seeded = member().await?;
    let dir = std::env::temp_dir().join(format!("tuita-e2e-visual-{}", Uuid::now_v7()));
    tokio::fs::create_dir_all(&dir).await?;
    let mut files = Vec::new();
    for i in 0..3u32 {
        let path = dir.join(format!("shot-{i}.jpg"));
        tokio::fs::write(&path, make_image(900, 600, 20 + i * 100)?).await?;
        files.push(path);
    }
    let spec =
        Spec::new("composer-media", "/", Viewer::Session(&seeded.ctx)).prepare(move |page| {
            let files = files.clone();
            async move {
                image_input(&page).set_files(&files).await?;
                page.label(Name::Exact("Description (alt text) of attachment 1"))
                    .fill("A warm gradient")
                    .await?;
                page.label(Name::Exact("Post text"))
                    .fill("Three photos on the way")
                    .await?;
                page.set_timeout(Duration::from_secs(20));
                page.role("progressbar", Name::Any).expect_count(0).await
            }
        });
    run(&spec).await
}

#[tokio::test]
async fn composer_poll() -> Result<()> {
    let seeded = member().await?;
    let spec =
        Spec::new("composer-poll", "/", Viewer::Session(&seeded.ctx)).prepare(|page| async move {
            page.role("button", Name::Exact("Add a poll"))
                .click()
                .await?;
            page.label(Name::Exact("Post text"))
                .fill("Which ink is best?")
                .await?;
            page.label(Name::Exact("Choice 1")).fill("Bone").await?;
            page.label(Name::Exact("Choice 2")).fill("Phosphor").await
        });
    run(&spec).await
}

#[tokio::test]
async fn admin_users_detail() -> Result<()> {
    let seeded = member().await?;
    let spec = Spec::new(
        "admin-users-detail",
        "/admin/users",
        Viewer::Session(&seeded.ctx),
    )
    .prepare(|page| async move {
        page.role("button", Name::Re("^Details for @"))
            .first()
            .click()
            .await?;
        page.role("dialog", Name::Exact("Account details"))
            .role("heading", Name::Exact("Moderation history"))
            .expect_visible()
            .await
    });
    run(&spec).await
}

const MAINTENANCE: &str = "Maintenance on Saturday at 02:00 UTC.";

/// Ends the announcements the page published.
async fn end_announcements(ctx: &Ctx, starting: &str) -> Result<()> {
    let list = call(ctx, "GET", "/admin/announcements", None).await?;
    for item in list.as_array().into_iter().flatten() {
        let ours = item["active"] == true
            && item["text"]
                .as_str()
                .is_some_and(|text| text.starts_with(starting));
        if let (true, Some(id)) = (ours, item["id"].as_str()) {
            call(ctx, "DELETE", &format!("/admin/announcements/{id}"), None).await?;
        }
    }
    Ok(())
}

#[tokio::test]
async fn admin_announcements_listed() -> Result<()> {
    let seeded = member().await?;
    let spec = Spec::new(
        "admin-announcements-listed",
        "/admin/announcements",
        Viewer::Session(&seeded.ctx),
    )
    .prepare(|page| async move {
        page.role("textbox", Name::Exact("Message"))
            .fill(MAINTENANCE)
            .await?;
        // Publishing reloads the page once the API has answered; wait for the new document.
        page.eval("window.beforePublish = true").await?;
        page.role("button", Name::Exact("Publish")).click().await?;
        page.wait_for_js(
            "window.beforePublish === undefined && document.readyState === 'complete'",
        )
        .await?;
        settle(&page).await?;
        page.css("[data-announcement]")
            .has_text("Maintenance on Saturday")
            .first()
            .expect_visible()
            .await
    });
    let outcome = run(&spec).await;
    end_announcements(&seeded.ctx, "Maintenance on Saturday").await?;
    outcome
}

#[tokio::test]
async fn admin_invites() -> Result<()> {
    let seeded = member().await?;
    let spec = Spec::new(
        "admin-invites",
        "/admin/invites",
        Viewer::Session(&seeded.ctx),
    )
    .prepare(|page| async move {
        page.wait_for_js("document.querySelectorAll('[data-invite]').length >= 2")
            .await?;
        page.text(Name::Exact("Revoked"))
            .first()
            .expect_visible()
            .await
    });
    run(&spec).await
}

/// A brand new admin per combination, because scanning is limited per staff user. The seeded filter
/// and the account it flags are what the scan finds.
#[tokio::test]
async fn admin_filter() -> Result<()> {
    member().await?;
    let spec =
        Spec::new("admin-filter", "/admin/filter", Viewer::NewAdmin).prepare(|page| async move {
            page.role("radio", Name::Any).expect_count(0).await?;
            page.css(
                r#"textarea[name="blocked"], textarea[name="allowed"], textarea[name="reserved"]"#,
            )
            .expect_count(3)
            .await?;
            page.role("button", Name::Exact("Scan now")).click().await?;
            page.role("status", Name::Any)
                .has_text(Name::Re("need attention|needs attention"))
                .expect_visible()
                .await?;
            page.role("button", Name::Re("^Details for @"))
                .first()
                .expect_visible()
                .await
        });
    run(&spec).await
}

#[tokio::test]
async fn admin_audit() -> Result<()> {
    let seeded = member().await?;
    let spec = Spec::new("admin-audit", "/admin/audit", Viewer::Session(&seeded.ctx)).prepare(
        |page| async move {
            page.wait_for_js("document.querySelectorAll('[data-action]').length > 10")
                .await?;
            // Rows show readable labels, not action keys.
            page.text(Name::Re(
                r"\b(user|post|report|invite|announcement|instance|admin)\.[a-z_]+\b",
            ))
            .expect_count(0)
            .await?;
            page.css("[data-action]")
                .first()
                .expect_text(Name::Re(
                    "Account|Invite|Announcement|Post|Report|Instance|Role|Verification|Organization",
                ))
                .await
        },
    );
    run(&spec).await
}

#[tokio::test]
async fn settings_bots_token() -> Result<()> {
    let seeded = member().await?;
    let spec = Spec::new(
        "settings-bots-token",
        "/settings/bots",
        Viewer::Session(&seeded.ctx),
    )
    .prepare(|page| async move {
        page.label("Token name").fill("release bot").await?;
        page.role("button", Name::Exact("Create token"))
            .click()
            .await?;
        let dialog = page.role("dialog", Name::Any);
        dialog.label("New API token").expect_visible().await?;
        dialog
            .text("it won’t be shown again")
            .expect_visible()
            .await
    });
    run(&spec).await
}

#[tokio::test]
async fn settings_passkeys_listed() -> Result<()> {
    let seeded = member().await?;
    let spec = Spec::new(
        "settings-passkeys-listed",
        "/settings/passkeys",
        Viewer::Session(&seeded.ctx),
    )
    .before(|page| async move { page.virtual_authenticator().await.map(drop) })
    .prepare(|page| async move {
        page.role("button", Name::Exact("Add a passkey"))
            .click()
            .await?;
        page.css("[data-passkey]").first().expect_visible().await
    });
    run(&spec).await
}

// ---- the consent card, from real requests

const OOB: &str = "urn:ietf:wg:oauth:2.0:oob";

/// What a client does first: registers itself. Returns the authorize URL of that client.
async fn consent_url(name: &str, website: &str, redirect: &str, scopes: &str) -> Result<String> {
    let ctx = Ctx::new().await?;
    let app = call(
        &ctx,
        "POST",
        "/apps",
        Some(&json!({
            "client_name": name, "redirect_uris": redirect, "scopes": scopes, "website": website,
        })),
    )
    .await?;
    ctx.close().await?;
    let id = app["client_id"].as_str().unwrap_or_default();
    Ok(authorize_url(id, redirect, scopes))
}

fn authorize_url(client_id: &str, redirect: &str, scope: &str) -> String {
    format!(
        "/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}\
         &code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM&code_challenge_method=S256&state=visual",
        enc(client_id),
        enc(redirect),
        enc(scope)
    )
}

fn authorize_button(page: &Page) -> Loc {
    page.role("button", Name::Exact("Authorize"))
}

#[tokio::test]
async fn consent() -> Result<()> {
    let seeded = member().await?;
    let url = consent_url(
        "Tuba",
        "https://tuba.example",
        "https://tuba.example/oauth/callback",
        "read write",
    )
    .await?;
    let spec = Spec::new("consent", url, Viewer::Session(&seeded.ctx)).prepare(|page| async move {
        page.css(r#"[data-state="ask"] [data-app-name]"#)
            .expect_text(Name::Exact("Tuba"))
            .await?;
        authorize_button(&page).expect_visible().await
    });
    run(&spec).await
}

/// Long names and addresses. The server refuses a `javascript:` website at registration and grants
/// OAuth apps only `read` and `write` (`follow` is `write`), so the card shows two scopes.
#[tokio::test]
async fn consent_long() -> Result<()> {
    let seeded = member().await?;
    let url = consent_url(
        "A very long application name that keeps going and going past the card",
        "https://an-address-that-keeps-going-and-going-with-no-room-left-in-the-card.example.org/and/a/path",
        "https://a-very-long-hostname-for-an-app.example.org/oauth/callback/with/a/long/path?and=query",
        "read write follow",
    )
    .await?;
    let spec = Spec::new("consent-long", url, Viewer::Session(&seeded.ctx))
        .prepare(|page| async move { page.css("[data-scope]").expect_count(2).await });
    run(&spec).await
}

/// A client id nobody registered.
#[tokio::test]
async fn consent_error() -> Result<()> {
    let seeded = member().await?;
    let url = authorize_url(
        "visual-client",
        "https://tuba.example/oauth/callback",
        "read write",
    );
    let spec =
        Spec::new("consent-error", url, Viewer::Session(&seeded.ctx)).prepare(|page| async move {
            page.role("alert", Name::Any)
                .role("heading", Name::Exact("This request can’t be authorized"))
                .expect_visible()
                .await?;
            authorize_button(&page).expect_count(0).await
        });
    run(&spec).await
}

/// An out-of-band app: Authorize answers with a code on screen (a real code, issued per combination).
#[tokio::test]
async fn consent_code() -> Result<()> {
    let seeded = member().await?;
    let url = consent_url("Tuba", "https://tuba.example", OOB, "read write").await?;
    let spec =
        Spec::new("consent-code", url, Viewer::Session(&seeded.ctx)).prepare(|page| async move {
            authorize_button(&page).click().await?;
            page.role("textbox", Name::Exact("Authorization code"))
                .expect_visible()
                .await?;
            page.role("button", Name::Exact("Copy code"))
                .expect_visible()
                .await
        });
    run(&spec).await
}

// ---- the banner and the federation tab

/// Two real announcements, a warning with a link and an info, for as long as the page is open.
#[tokio::test]
async fn banner() -> Result<()> {
    let seeded = member().await?;
    let warning = "A sign-in problem is being fixed. Some people see an error on the first try.";
    for (text, tone, link) in [
        (warning, "warning", "/terms"),
        ("Banner: maintenance on Saturday at 02:00 UTC.", "info", ""),
    ] {
        let body = json!({"text": text, "tone": tone, "link": link, "days": 1});
        call(&seeded.ctx, "POST", "/admin/announcements", Some(&body)).await?;
    }
    let spec = Spec::new("banner", "/", Viewer::Session(&seeded.ctx)).prepare(|page| async move {
        page.role("region", Name::Exact("Announcements"))
            .expect_visible()
            .await
    });
    let outcome = run(&spec).await;
    end_announcements(&seeded.ctx, "A sign-in problem").await?;
    end_announcements(&seeded.ctx, "Banner: maintenance").await?;
    outcome
}

/// The second instance's own federation tab: nothing queued, nothing blocked.
#[tokio::test]
async fn admin_federation_empty() -> Result<()> {
    let world = world().await?;
    let spec = Spec::new(
        "admin-federation-empty",
        "/admin/federation",
        Viewer::Session(&world.admin),
    )
    .prepare(|page| async move {
        page.text("Nothing is waiting. Every delivery has been sent.")
            .expect_visible()
            .await?;
        page.text("No instances are blocked.")
            .expect_visible()
            .await?;
        page.css("[data-fingerprint]").expect_visible().await
    });
    run(&spec).await
}

/// Two blocked instances and a follow of an account whose inbox answers 500, which queues a
/// delivery that keeps failing.
#[tokio::test]
async fn admin_federation() -> Result<()> {
    let seeded = member().await?;
    let fake = fake()?;
    for (domain, severity) in [("down.example", "suspend"), ("loud.example", "silence")] {
        let body = json!({"domain": domain, "severity": severity, "reason": "Visual suite"});
        call(&seeded.ctx, "POST", "/admin/federation/blocks", Some(&body)).await?;
    }
    let follower = Ctx::new().await?;
    sign_up_via_api(&follower, &handle("flw"), "Follower Person").await?;
    let target = enc(&format!("broken@{}", fake.domain));
    call(&follower, "POST", &format!("/users/{target}/follow"), None).await?;
    let failing = tuita_e2e::poll("waiting for the failing delivery", || async {
        let state = call(&seeded.ctx, "GET", "/admin/federation", None).await?;
        let found = state["queue"]["failing"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|row| row["domain"] == fake.domain.as_str());
        Ok(if found {
            tuita_e2e::Poll::Ready(())
        } else {
            tuita_e2e::Poll::Retry(state["queue"].to_string())
        })
    });
    tokio::time::timeout(Duration::from_secs(60), failing).await??;

    let domain = fake.domain.clone();
    let spec = Spec::new(
        "admin-federation",
        "/admin/federation",
        Viewer::Session(&seeded.ctx),
    )
    .prepare(move |page| {
        let row = format!(r#"[data-failing="{domain}"]"#);
        async move {
            page.wait_for_js("document.querySelectorAll('[data-block]').length >= 2")
                .await?;
            page.css(&row).expect_visible().await?;
            page.role("button", Name::Exact("Retry now"))
                .expect_enabled()
                .await
        }
    });
    run(&spec).await
}

// ---- remote pages, from real instances

#[tokio::test]
async fn remote_profile() -> Result<()> {
    let world = world().await?;
    let seeded = member().await?;
    let domain = world.peer.domain.clone();
    let spec = Spec::new(
        "remote-profile",
        format!("/@{}@{domain}", world.mara),
        Viewer::Session(&seeded.ctx),
    )
    .prepare(move |page| {
        let domain = domain.clone();
        async move {
            page.css(".profile .name")
                .first()
                .expect_text(Name::Contains("Mara Okafor"))
                .await?;
            page.css(&format!(r#".profile [data-domain="{domain}"]"#))
                .expect_visible()
                .await?;
            page.role("link", Name::Exact(&format!("View profile on {domain}")))
                .expect_visible()
                .await?;
            page.css("article[data-post]")
                .first()
                .expect_text(Name::Contains(REMOTE_POST))
                .await
        }
    });
    run(&spec).await
}

/// An account that approves followers by hand, followed by a user whose request stays pending.
#[tokio::test]
async fn remote_profile_pending() -> Result<()> {
    let fake = fake()?;
    let viewer = Ctx::new().await?;
    sign_up_via_api(&viewer, &handle("req"), "Requesting Person").await?;
    let acct = format!("locked@{}", fake.domain);
    call(
        &viewer,
        "POST",
        &format!("/users/{}/follow", enc(&acct)),
        None,
    )
    .await?;
    let spec = Spec::new(
        "remote-profile-pending",
        format!("/@{acct}"),
        Viewer::Session(&viewer),
    )
    .prepare(|page| async move {
        page.role("button", Name::Exact("Requested"))
            .expect_visible()
            .await?;
        page.text("Approves followers").expect_visible().await
    });
    run(&spec).await
}

/// The second instance has no such account.
#[tokio::test]
async fn remote_profile_missing() -> Result<()> {
    let world = world().await?;
    let seeded = member().await?;
    let spec = Spec::new(
        "remote-profile-missing",
        format!("/@nobody@{}", world.peer.domain),
        Viewer::Session(&seeded.ctx),
    )
    .tolerate("status of 404")
    .prepare(|page| async move {
        page.role("heading", Name::Exact("No such account"))
            .expect_visible()
            .await
    });
    run(&spec).await
}

/// The instance does not listen: a port that was just free.
#[tokio::test]
async fn remote_profile_unreachable() -> Result<()> {
    let seeded = member().await?;
    let spec = Spec::new(
        "remote-profile-unreachable",
        format!("/@gone@localhost:{}", free_port()?),
        Viewer::Session(&seeded.ctx),
    )
    .tolerate("status of 502")
    .prepare(|page| async move {
        page.role("alert", Name::Any)
            .text("Couldn’t reach")
            .expect_visible()
            .await?;
        page.role("button", Name::Exact("Try again"))
            .expect_visible()
            .await
    });
    run(&spec).await
}

#[tokio::test]
async fn remote_post() -> Result<()> {
    let world = world().await?;
    let seeded = member().await?;
    let acct = format!("{}@{}", world.mara, world.peer.domain);
    // Reading the account's posts is what stores them here, with ids of this instance.
    let posts = call(
        &seeded.ctx,
        "GET",
        &format!("/users/{}/posts", enc(&acct)),
        None,
    )
    .await?;
    let id = posts["items"][0]["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("the remote account's posts are empty: {posts}"))?
        .to_owned();
    let domain = world.peer.domain.clone();
    let spec = Spec::new(
        "remote-post",
        format!("/@{acct}/{id}"),
        Viewer::Session(&seeded.ctx),
    )
    .prepare(move |page| {
        let domain = domain.clone();
        async move {
            page.css("article[data-post]")
                .first()
                .expect_text(Name::Contains(REMOTE_POST))
                .await?;
            page.css(&format!(r#"article [data-domain="{domain}"]"#))
                .first()
                .expect_visible()
                .await?;
            page.role("link", Name::Exact(&format!("View on {domain}")))
                .first()
                .expect_visible()
                .await
        }
    });
    run(&spec).await
}

#[tokio::test]
async fn search_remote() -> Result<()> {
    let world = world().await?;
    let seeded = member().await?;
    let domain = world.peer.domain.clone();
    let query = enc(&format!("@{}@{domain}", world.mara));
    let spec = Spec::new(
        "search-remote",
        format!("/explore?q={query}&type=people"),
        Viewer::Session(&seeded.ctx),
    )
    .prepare(move |page| {
        let domain = domain.clone();
        async move {
            page.css(&format!(r#"[data-domain="{domain}"]"#))
                .first()
                .expect_visible()
                .await?;
            page.css("#main")
                .role("button", Name::Exact("Follow"))
                .first()
                .expect_visible()
                .await
        }
    });
    run(&spec).await
}
