//! Browser tests: passkeys.

use tuita_e2e::{
    Ctx, Name, Result,
    helpers::{handle, sign_up_via_api},
};

#[tokio::test]
async fn register_a_passkey_in_settings_sign_in_with_it_then_remove_it() -> Result<()> {
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    sign_up_via_api(&ctx, &handle("pk"), "Passkey Person").await?;

    page.goto("/settings/passkeys").await?;
    let authenticator = page.virtual_authenticator().await?;
    page.text(Name::Exact("No passkeys yet."))
        .expect_visible()
        .await?;
    page.role("button", Name::Exact("Add a passkey"))
        .click()
        .await?;
    let row = page.css("[data-passkey]");
    row.expect_count(1).await?;
    row.expect_text(Name::Contains("never used")).await?;
    assert_eq!(authenticator.credentials().await?, 1);

    // The conditional prompt would sign in on its own within tens of milliseconds, and whether the
    // explicit button can still be clicked first depends on timing: pin the explicit path.
    ctx.call(
        "POST",
        "/auth/logout",
        Some(&serde_json::json!({"all": true})),
    )
    .await?;
    page.init_script("PublicKeyCredential.isConditionalMediationAvailable = async () => false;")
        .await?;
    page.goto("/login").await?;
    page.role("button", Name::Exact("Sign in with a passkey"))
        .click()
        .await?;
    page.role("navigation", Name::Exact("Main"))
        .first()
        .expect_visible()
        .await?;
    page.expect_url(Name::Re("/$")).await?;

    page.goto("/settings/passkeys").await?;
    page.css("[data-passkey]")
        .expect_text(Name::Contains("last used"))
        .await?;
    page.role("button", Name::Re("^Remove passkey"))
        .click()
        .await?;
    page.role("dialog", Name::Any)
        .role("button", Name::Exact("Remove"))
        .click()
        .await?;
    page.text(Name::Exact("No passkeys yet."))
        .expect_visible()
        .await
}
