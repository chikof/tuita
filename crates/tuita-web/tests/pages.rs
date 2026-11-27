//! The pages the web client serves: the shell of every page, guards, caching, fixed files.
//!
//! Each `#[sqlx::test]` gets a fresh database; `DATABASE_URL` must point at a running Postgres.
//! Run `topcoat asset bundle --package tuita` first (or point `TUITA_TEST_ASSETS` at a bundle).

mod common;

use axum::http::{Method, StatusCode};
use common::{Browser, app};
use serde_json::json;
use sqlx::PgPool;

const MIGRATIONS: &str = "../tuita-server/migrations";

fn head_of(html: &str) -> &str {
    let start = html.find("<head").expect("a head");
    let end = html.find("</head>").expect("head closes");
    &html[start..end]
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn guest_home_is_a_page_with_preview_tags_and_the_boot_script(db: PgPool) {
    let _ = MIGRATIONS;
    let mut guest = Browser::new(&app(db));
    let reply = guest.get("/").await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    assert_eq!(reply.header("cache-control"), "no-store");
    assert!(reply.header_all("vary").contains("Cookie"));
    assert_eq!(
        reply.header("content-security-policy"),
        tuita_server::CONTENT_SECURITY_POLICY
    );
    let html = &reply.body;
    assert!(html.contains("<html lang=\"en\""), "{html}");
    assert!(html.contains("property=\"og:title\""), "{html}");
    assert_eq!(
        html.matches("property=\"og:title\"").count(),
        1,
        "one og:title: {html}"
    );
    assert!(
        html.contains("<title>tuita · Every post, two inks.</title>"),
        "{html}"
    );
    let head = head_of(html);
    let boot = head
        .find("<script src=\"/_topcoat/assets/boot-")
        .expect("boot.js is a blocking script in the head");
    let first_style = head.find("rel=\"stylesheet\"").expect("stylesheets");
    assert!(boot < first_style, "boot.js loads before the stylesheets");
    assert!(head.contains("id=\"tuita-l10n\""));
    assert!(head.contains("data-js=\"stream\""));
    assert!(html.contains("/_topcoat/assets/topcoat-"), "runtime script");
    assert!(html.contains("id=\"toaster\""));
    assert!(
        !html.contains("<script>"),
        "no inline script anywhere: {html}"
    );
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn guest_on_a_member_page_goes_to_sign_in(db: PgPool) {
    let mut guest = Browser::new(&app(db));
    for (path, location) in [
        ("/settings", "/login?next=%2Fsettings"),
        ("/explore", "/login?next=%2Fexplore"),
        ("/lists/7/members", "/login?next=%2Flists%2F7%2Fmembers"),
        ("/@ana/followers", "/login?next=%2F%40ana%2Ffollowers"),
    ] {
        let reply = guest.get(path).await;
        assert_eq!(reply.status, StatusCode::SEE_OTHER, "{path}");
        assert_eq!(reply.header("location"), location, "{path}");
    }
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn guest_on_a_profile_or_post_gets_a_preview_that_redirects(db: PgPool) {
    let mut guest = Browser::new(&app(db));
    for path in ["/@ana", "/@ana/0190"] {
        let reply = guest.get(path).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.body);
        assert!(reply.body.contains("property=\"og:title\""), "{path}");
        let refresh = format!(
            "http-equiv=\"refresh\" content=\"0; url=/login?next={}\"",
            path.replace('@', "%40").replace('/', "%2F")
        );
        assert!(reply.body.contains(&refresh), "{path}: {}", reply.body);
        assert!(
            reply.body.contains("href=\"/login?next="),
            "{path}: visible link"
        );
    }
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn fixed_url_files_are_cached_for_an_hour(db: PgPool) {
    let mut guest = Browser::new(&app(db));
    for (path, kind) in [
        ("/favicon.svg", "image/svg+xml"),
        ("/icon-192.png", "image/png"),
        ("/icon-512.png", "image/png"),
        ("/icon-maskable-192.png", "image/png"),
        ("/icon-maskable-512.png", "image/png"),
        ("/manifest.webmanifest", "application/manifest+json"),
        ("/robots.txt", "text/plain; charset=utf-8"),
    ] {
        let reply = guest.get(path).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}");
        assert_eq!(
            reply.header("cache-control"),
            "public, max-age=3600",
            "{path}"
        );
        assert_eq!(reply.header("content-type"), kind, "{path}");
    }
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn pages_are_not_stored(db: PgPool) {
    let mut ana = Browser::new(&app(db));
    ana.signup("ana").await;
    for path in ["/", "/settings", "/@ana"] {
        let reply = ana.get(path).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.body);
        assert_eq!(reply.header("cache-control"), "no-store", "{path}");
        assert!(reply.header_all("vary").contains("Cookie"), "{path}");
    }
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn unmatched_urls_render_the_branded_404(db: PgPool) {
    let router = app(db);
    let mut guest = Browser::new(&router);
    let reply = guest.get("/nope/x/y").await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert!(reply.body.contains("Nothing here"), "{}", reply.body);
    assert!(
        reply.body.contains("id=\"toaster\""),
        "wrapped in the root layout"
    );
    assert_eq!(reply.header("cache-control"), "no-store");

    // A single unknown segment is a handle candidate: a member sees the same page.
    let mut ana = Browser::new(&router);
    ana.signup("ana").await;
    let reply = ana.get("/nope").await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert!(reply.body.contains("Nothing here"), "{}", reply.body);
    assert!(reply.body.contains("<html lang=\"en\""));
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn signed_in_visitors_leave_the_sign_in_pages(db: PgPool) {
    let router = app(db);
    let mut guest = Browser::new(&router);
    assert_eq!(guest.get("/login").await.status, StatusCode::OK);
    assert_eq!(guest.get("/signup").await.status, StatusCode::OK);

    let mut ana = Browser::new(&router);
    ana.signup("ana").await;
    let home = ana.get("/login").await;
    assert_eq!(home.status, StatusCode::SEE_OTHER);
    assert_eq!(home.header("location"), "/");
    let next = ana.get("/signup?next=%2Fexplore").await;
    assert_eq!(next.header("location"), "/explore");
    let evil = ana.get("/login?next=%2F%2Fevil.example").await;
    assert_eq!(evil.header("location"), "/");
    let add = ana.get("/login?add=1").await;
    assert_eq!(
        add.status,
        StatusCode::OK,
        "adding an account shows the form"
    );
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn a_fragment_request_renders_only_the_page(db: PgPool) {
    let mut guest = Browser::new(&app(db));
    guest.headers.push(("x-tuita-fragment".into(), "1".into()));
    let reply = guest.get("/login").await;
    assert_eq!(reply.status, StatusCode::OK);
    assert!(!reply.body.contains("<html"), "{}", reply.body);
    assert!(
        reply.body.contains("data-island=\"passkey-login\""),
        "the page itself is rendered: {}",
        reply.body
    );
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn members_get_the_shell_and_guests_and_bare_pages_do_not(db: PgPool) {
    let router = app(db);
    let mut guest = Browser::new(&router);
    for path in ["/", "/login", "/signup", "/terms"] {
        let reply = guest.get(path).await;
        assert!(
            !reply.body.contains("data-c=\"app-shell\""),
            "{path}: no shell for a guest"
        );
        assert!(
            !reply.body.contains("data-island=\"stream\""),
            "{path}: no stream for a guest"
        );
    }

    let mut ana = Browser::new(&router);
    ana.signup("ana").await;
    for path in ["/", "/terms", "/privacy", "/nope"] {
        let reply = ana.get(path).await;
        let html = &reply.body;
        assert!(html.contains("data-c=\"app-shell\""), "{path}: {html}");
        assert!(html.contains("<main id=\"main\">"), "{path}");
        assert!(
            path != "/" || html.contains("aria-current=\"page\""),
            "{path}: the current nav entry"
        );
        assert!(
            html.contains("data-island=\"stream\" data-handle=\"ana\"")
                || html.contains("data-handle=\"ana\" data-island=\"stream\""),
            "{path}: the stream marker"
        );
        assert!(html.contains("data-badge=\"notifications\""), "{path}");
        assert!(html.contains("data-badge=\"messages\""), "{path}");
        assert!(html.contains("<dialog id=\"shortcuts\""), "{path}");
        assert!(html.contains("id=\"drawer\""), "{path}");
        assert!(
            html.contains("data-forget-keys="),
            "{path}: logout controls"
        );
        assert!(html.contains("data-compose=\"new\""), "{path}");
    }

    let add = ana.get("/login?add=1").await;
    assert_eq!(add.status, StatusCode::OK);
    assert!(
        !add.body.contains("data-c=\"app-shell\""),
        "the sign-in page is bare even for a member"
    );
    assert!(
        add.body.contains("data-island=\"stream\""),
        "but the stream still runs"
    );
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn fragment_routes_answer_members_only(db: PgPool) {
    let router = app(db);
    let mut guest = Browser::new(&router);
    let reply = guest.get("/_tuita/new-posts").await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert!(reply.body.is_empty());

    let mut ana = Browser::new(&router);
    ana.signup("ana").await;
    let reply = ana.get("/_tuita/new-posts?feed=foryou").await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    assert_eq!(reply.header("cache-control"), "no-store");
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn the_language_follows_account_then_cookie_then_instance(db: PgPool) {
    let router = app(db);
    let mut guest = Browser::new(&router);
    let en = guest.get("/").await;
    assert!(en.body.contains("<html lang=\"en\""));
    assert!(en.body.contains("\"time_now\":\"now\""), "{}", en.body);

    guest.set_cookie("tuita_lang", "pl");
    let pl = guest.get("/").await;
    assert!(pl.body.contains("<html lang=\"pl\""));
    assert!(pl.body.contains("\"time_now\":\"teraz\""));

    // Not one of the twelve: ignored.
    guest.set_cookie("tuita_lang", "xx");
    assert!(guest.get("/").await.body.contains("<html lang=\"en\""));

    // The account's language beats the cookie.
    let mut ana = Browser::new(&router);
    ana.signup("ana").await;
    let patch = ana
        .request(Method::PATCH, "/api/v1/me", Some(json!({"language": "de"})))
        .await;
    assert_eq!(patch.status, StatusCode::OK, "{}", patch.body);
    ana.set_cookie("tuita_lang", "pl");
    assert!(ana.get("/").await.body.contains("<html lang=\"de\""));
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn a_flash_cookie_becomes_one_toast_and_is_expired(db: PgPool) {
    let mut guest = Browser::new(&app(db));
    guest.set_cookie("tuita_flash", "settings_saved");
    let reply = guest.get("/").await;
    assert!(
        reply
            .body
            .contains("<output hidden=\"\" data-toast=\"\" data-kind=\"info\">Saved</output>"),
        "{}",
        reply.body
    );
    assert!(
        reply
            .set_cookies()
            .contains(&("tuita_flash".to_owned(), String::new()))
    );

    // A key the page does not know shows nothing.
    guest.set_cookie("tuita_flash", "<script>alert(1)</script>");
    assert!(!guest.get("/").await.body.contains("data-toast"));
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn hashed_assets_keep_their_immutable_caching(db: PgPool) {
    let mut guest = Browser::new(&app(db));
    let home = guest.get("/").await;
    let start = home
        .body
        .find("/_topcoat/assets/boot-")
        .expect("boot.js link");
    let url = &home.body[start..];
    let url = &url[..url.find('"').expect("quote")];
    let reply = guest.get(url).await;
    assert_eq!(reply.status, StatusCode::OK, "{url}");
    assert!(
        reply.header("cache-control").contains("immutable"),
        "{}",
        reply.header("cache-control")
    );
    assert!(!reply.header_all("vary").contains("Cookie"));
}
