//! `PATCH /me`: the interface language.

mod common;

use axum::http::StatusCode;
use common::{Browser, app};
use serde_json::{Value, json};
use sqlx::PgPool;

async fn stored_language(browser: &mut Browser) -> Value {
    browser.get("/me").await.ok()["user"]["settings"]["language"].clone()
}

#[sqlx::test(migrations = "./migrations")]
async fn language_is_set_cleared_and_validated(db: PgPool) {
    let app = app(db);
    let mut ana = app.browser();
    ana.signup("ana").await;

    let me = ana.patch("/me", json!({ "language": "ja" })).await.ok();
    assert_eq!(me["settings"]["language"], "ja");
    assert_eq!(stored_language(&mut ana).await, "ja");

    let me = ana.patch("/me", json!({ "language": "" })).await.ok();
    assert_eq!(me["settings"]["language"], Value::Null);
    assert_eq!(stored_language(&mut ana).await, Value::Null);

    ana.patch("/me", json!({ "language": "ja" })).await.ok();
    ana.patch("/me", json!({ "language": "xx" }))
        .await
        .err(StatusCode::BAD_REQUEST, "Unknown language.");
    assert_eq!(stored_language(&mut ana).await, "ja");
}
