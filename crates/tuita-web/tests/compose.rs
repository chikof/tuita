//! The composer markup the shell renders on every member page.
//!
//! Each `#[sqlx::test]` gets a fresh database; `DATABASE_URL` must point at a running Postgres.
//! Run `topcoat asset bundle --package tuita` first (or point `TUITA_TEST_ASSETS` at a bundle).

mod common;

use axum::http::StatusCode;
use common::{Browser, app};
use sqlx::PgPool;

/// The attribute values of every `data-role="name"` element opening tag in `html`.
fn tag_of<'a>(html: &'a str, needle: &str) -> &'a str {
    let at = html
        .find(needle)
        .unwrap_or_else(|| panic!("{needle} is missing from the page"));
    let start = html[..at].rfind('<').expect("an opening tag");
    let end = at + html[at..].find('>').expect("the tag closes");
    &html[start..=end]
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn every_member_page_carries_the_compose_dialog_with_a_composer(db: PgPool) {
    let mut member = Browser::new(&app(db));
    member.signup("ada").await;
    let reply = member.get("/terms").await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    let html = &reply.body;

    assert!(html.contains(r#"<dialog id="compose""#), "{html}");
    assert!(
        html.contains(r#"<h2 id="compose-title">New post</h2>"#),
        "{html}"
    );

    let form = tag_of(html, r#"data-island="composer""#);
    assert!(form.contains(r#"data-context="dialog""#), "{form}");
    assert!(form.contains(r#"data-slot="new""#), "{form}");
    assert!(
        form.contains("data-user=\""),
        "drafts are stored per account: {form}"
    );
    assert!(form.contains("novalidate"), "{form}");

    let area = tag_of(html, r#"data-role="text""#);
    assert!(area.contains(r#"role="combobox""#), "{area}");
    assert!(area.contains(r#"aria-label="Post text""#), "{area}");
    assert!(area.contains("What’s happening?"), "{area}");

    assert!(html.contains(r#"role="toolbar""#), "{html}");
    for label in [
        "Add images",
        "Add a video",
        "Add a poll",
        "Add emoji",
        "Add a content warning",
        "Mark media as sensitive",
        "Schedule post",
        "Who can reply",
    ] {
        assert!(
            html.contains(&format!("aria-label=\"{label}\"")),
            "{label}: {html}"
        );
    }
    assert!(html.contains(r#"data-island="dropzone""#), "{html}");
    assert!(html.contains(r#"data-island="media-tray""#), "{html}");
    assert!(html.contains(r#"data-role="send""#), "{html}");
    assert!(
        html.contains("data-compose=\"new\""),
        "the shell's compose buttons: {html}"
    );
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn guests_get_no_compose_dialog(db: PgPool) {
    let mut guest = Browser::new(&app(db));
    let reply = guest.get("/terms").await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    assert!(!reply.body.contains(r#"id="compose""#), "{}", reply.body);
}
