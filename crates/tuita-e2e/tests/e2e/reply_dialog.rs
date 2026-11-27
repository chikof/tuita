//! Browser tests: the reply dialog.

use serde_json::json;
use tuita_e2e::{
    Ctx, Name, Result,
    helpers::{call, handle, sign_up_via_api},
};

/// Testers saw the placeholder of the reply dialog cut in half: the box was sized while the dialog was closed.
#[tokio::test]
async fn the_reply_dialog_sizes_its_text_box_to_the_whole_placeholder() -> Result<()> {
    let ctx = Ctx::new().await?;
    let who = handle("rd");
    sign_up_via_api(&ctx, &who, "Reply Dialog").await?;
    let made = call(
        &ctx,
        "POST",
        "/posts",
        Some(&json!({"text": "reply to me", "cw": "", "sensitive": false, "mediaIds": []})),
    )
    .await?;
    let id = made["id"].as_str().expect("the post id");
    call(&ctx, "PATCH", "/me", Some(&json!({"language": "es"}))).await?;

    let page = ctx.page().await?;
    page.goto(&format!("/@{who}/{id}")).await?;
    page.role("button", Name::ReI("Responder"))
        .first()
        .click()
        .await?;

    let dialog = page.role("dialog", Name::Any);
    let box_ = dialog.css("textarea").first();
    box_.expect_visible().await?;
    page.eval("document.fonts.ready.then(() => true)").await?;
    page.wait_for_js(
        "(() => { const el = document.querySelector('dialog[open] textarea'); return el && el.clientHeight - el.scrollHeight >= 0; })()",
    )
    .await?;
    let height = page
        .eval("document.querySelector('dialog[open] textarea').clientHeight")
        .await?;
    assert!(
        height.as_f64().unwrap_or(0.0) > 40.0,
        "clientHeight {height}"
    );
    Ok(())
}
