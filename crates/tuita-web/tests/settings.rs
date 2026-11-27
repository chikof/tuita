//! Full-app tests of the settings pages: every section renders for a member, an unknown one is a
//! 404 and a guest is sent to sign in.

mod common;

use axum::http::StatusCode;
use common::{Browser, app};
use sqlx::PgPool;

/// Each section with text only that section shows.
const SECTIONS: [(&str, &str); 12] = [
    ("account", "Change password"),
    ("profile", "Display name"),
    ("privacy", "Muted words"),
    ("encryption", "Encrypted"),
    ("notifications", "Desktop notifications"),
    ("devices", "Devices &amp; sessions"),
    ("passkeys", "No passkeys yet."),
    ("affiliation", "Your affiliation"),
    ("bots", "API tokens"),
    ("appearance", "Text size"),
    ("language", "Language"),
    ("about", "Navigation"),
];

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn every_section_renders_for_a_member(db: PgPool) {
    let mut ana = Browser::new(&app(db));
    ana.signup("ana").await;
    let index = ana.get("/settings").await;
    assert_eq!(index.status, StatusCode::OK, "{}", index.body);
    for (slug, _) in SECTIONS {
        assert!(
            index.body.contains(&format!(r#"href="/settings/{slug}""#)),
            "{slug} is listed: {}",
            index.body
        );
    }
    for (slug, text) in SECTIONS {
        let reply = ana.get(&format!("/settings/{slug}")).await;
        assert_eq!(reply.status, StatusCode::OK, "{slug}: {}", reply.body);
        assert!(
            reply.body.contains(text),
            "{slug} shows {text}: {}",
            reply.body
        );
        assert_eq!(reply.header("cache-control"), "no-store", "{slug}");
    }
}

#[sqlx::test(migrations = "../tuita-server/migrations")]
async fn an_unknown_section_is_a_404_and_a_guest_is_sent_to_sign_in(db: PgPool) {
    let router = app(db);
    let mut ana = Browser::new(&router);
    ana.signup("ana").await;
    assert_eq!(
        ana.get("/settings/secrets").await.status,
        StatusCode::NOT_FOUND
    );

    let mut guest = Browser::new(&router);
    let reply = guest.get("/settings/privacy").await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER);
    assert_eq!(
        reply.header("location"),
        "/login?next=%2Fsettings%2Fprivacy"
    );
}
