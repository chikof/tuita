//! The `/design` gallery: a guest gets it bare, with every indexed component in place.
//!
//! Each `#[sqlx::test]` gets a fresh database; `DATABASE_URL` must point at a running Postgres.
//! Run `topcoat asset bundle --package tuita` first (or point `TUITA_TEST_ASSETS` at a bundle).

mod common;

use axum::http::StatusCode;
use common::{Browser, app};
use sqlx::PgPool;

/// The `href` of every link in `<nav aria-label="Components">`, without the `#`.
fn index_of(html: &str) -> Vec<&str> {
    let start = html
        .find("<nav aria-label=\"Components\"")
        .expect("the component index");
    let nav = &html[start
        ..html[start..]
            .find("</nav>")
            .map_or(html.len(), |end| start + end)];
    nav.split("href=\"#")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .collect()
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn the_gallery_is_a_bare_page_with_an_anchor_for_every_component(db: PgPool) {
    let mut guest = Browser::new(&app(db));
    let reply = guest.get("/design").await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    assert_eq!(reply.header("cache-control"), "no-store");
    let html = &reply.body;
    assert!(
        html.contains("<title>Design system · tuita</title>"),
        "{html}"
    );
    assert!(html.contains("<h1>Design system</h1>"), "{html}");
    assert!(!html.contains("id=\"main\""), "no app shell: {html}");

    let ids = index_of(html);
    assert!(ids.len() >= 36, "{} anchors", ids.len());
    for id in ids {
        assert_eq!(
            html.matches(&format!("<section id=\"{id}\"")).count(),
            1,
            "section#{id}"
        );
    }
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn a_member_gets_the_gallery_without_the_shell_too(db: PgPool) {
    let mut member = Browser::new(&app(db));
    member.signup("ada").await;
    let reply = member.get("/design").await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    assert!(!reply.body.contains("id=\"main\""), "no app shell");
    assert!(reply.body.contains("<h1>Design system</h1>"));
}
