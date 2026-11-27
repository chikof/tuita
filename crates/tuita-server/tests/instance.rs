//! `GET /instance`: the language it suggests before sign-in.

mod common;

use common::{app, app_with};
use sqlx::PgPool;

fn trust_proxy(var: &str) -> Option<String> {
    (var == "TRUST_PROXY").then(|| "1".to_owned())
}

#[sqlx::test(migrations = "./migrations")]
async fn country_header_counts_only_behind_a_trusted_proxy(db: PgPool) {
    let app = app_with(db, trust_proxy);
    let mut browser = app.browser();
    browser.headers = vec![
        ("cf-ipcountry".to_owned(), "JP".to_owned()),
        ("accept-language".to_owned(), "de,en;q=0.5".to_owned()),
    ];
    let reply = browser.get("/instance").await.ok();
    assert_eq!(reply["detectedLanguage"], "ja");

    browser.headers = vec![("cf-ipcountry".to_owned(), "XX".to_owned())];
    let reply = browser.get("/instance").await.ok();
    assert_eq!(reply["detectedLanguage"], "en");
}

#[sqlx::test(migrations = "./migrations")]
async fn untrusted_country_header_is_ignored(db: PgPool) {
    let app = app(db);
    let mut browser = app.browser();
    browser.headers = vec![
        ("cf-ipcountry".to_owned(), "JP".to_owned()),
        ("accept-language".to_owned(), "de,en;q=0.5".to_owned()),
    ];
    let reply = browser.get("/instance").await.ok();
    assert_eq!(reply["detectedLanguage"], "de");
}

#[sqlx::test(migrations = "./migrations")]
async fn no_headers_suggest_english(db: PgPool) {
    let app = app(db);
    let reply = app.browser().get("/instance").await.ok();
    assert_eq!(reply["detectedLanguage"], "en");
}
