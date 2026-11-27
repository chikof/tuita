//! Browser tests: every settings section renders clean, saves a field and shows it after a reload.

use serde_json::{Value, json};
use tuita_e2e::{
    Ctx, Name, Page, Result, Violation,
    helpers::{PASSWORD, call, handle, make_image, sign_up_via_api},
};

/// A new account and a page of its context, on `/settings/<section>`.
async fn member(prefix: &str, section: &str) -> Result<(Ctx, Page, String)> {
    let ctx = Ctx::new().await?;
    let who = handle(prefix);
    sign_up_via_api(&ctx, &who, "Settings Person").await?;
    let page = ctx.page().await?;
    page.goto(&format!("/settings/{section}")).await?;
    Ok((ctx, page, who))
}

async fn me(ctx: &Ctx) -> Result<Value> {
    Ok(call(ctx, "GET", "/me", None).await?["user"].clone())
}

async fn clean(page: &Page, what: &str) -> Result<()> {
    let errors = page.console_errors().await;
    assert!(errors.is_empty(), "{what}: console errors: {errors:#?}");
    let violations = page.axe().await?;
    assert!(
        violations.is_empty(),
        "{what}: axe violations:\n{}",
        Violation::report(&violations)
    );
    Ok(())
}

async fn toast(page: &Page, text: &str) -> Result<()> {
    page.css("#toaster")
        .text(Name::Contains(text))
        .expect_visible()
        .await
}

#[tokio::test]
async fn every_section_renders_without_console_errors_or_axe_violations() -> Result<()> {
    let ctx = Ctx::new().await?;
    sign_up_via_api(&ctx, &handle("sect"), "Settings Person").await?;
    let page = ctx.page().await?;
    page.goto("/settings").await?;
    page.role("heading", Name::Exact("Settings"))
        .first()
        .expect_visible()
        .await?;
    clean(&page, "/settings").await?;
    for section in [
        "account",
        "profile",
        "privacy",
        "encryption",
        "notifications",
        "devices",
        "passkeys",
        "affiliation",
        "bots",
        "appearance",
        "language",
        "about",
    ] {
        page.goto(&format!("/settings/{section}")).await?;
        page.css("#main h1").first().expect_visible().await?;
        page.eval("document.fonts.ready.then(() => true)").await?;
        clean(&page, section).await?;
    }
    Ok(())
}

#[tokio::test]
async fn the_index_links_to_a_section_and_a_section_goes_back() -> Result<()> {
    let (_ctx, page, who) = member("idx", "account").await?;
    page.goto("/settings").await?;
    page.role("link", Name::Exact(&format!("@{who}@")))
        .count()
        .await?;
    page.role("link", Name::Re("^Privacy"))
        .first()
        .click()
        .await?;
    page.expect_url(Name::Exact("/settings/privacy")).await?;
    page.role("button", Name::Exact("Back")).click().await?;
    page.expect_url(Name::Exact("/settings")).await
}

#[tokio::test]
async fn the_profile_saves_and_shows_after_a_reload() -> Result<()> {
    let (ctx, page, _who) = member("prof", "profile").await?;
    page.label("Bio").fill("Hello from the e2e suite").await?;
    page.label("Location").fill("Nowhere").await?;
    page.role("radio", Name::Exact("Square")).check().await.ok();
    page.role("button", Name::Exact("Save profile"))
        .click()
        .await?;
    toast(&page, "Profile updated").await?;
    page.label("Bio")
        .expect_value(Name::Exact("Hello from the e2e suite"))
        .await?;
    let user = me(&ctx).await?;
    assert_eq!(user["user"]["bio"], "Hello from the e2e suite");
    assert_eq!(user["user"]["location"], "Nowhere");
    page.reload().await?;
    page.label("Location")
        .expect_value(Name::Exact("Nowhere"))
        .await
}

#[tokio::test]
async fn the_password_changes_and_the_new_one_signs_in() -> Result<()> {
    let (_ctx, page, who) = member("pw", "account").await?;
    page.label(Name::Exact("Current password"))
        .fill(PASSWORD)
        .await?;
    page.label(Name::Exact("New password"))
        .fill("a much better one 42")
        .await?;
    page.label(Name::Exact("Confirm new password"))
        .fill("a much better one 42")
        .await?;
    page.role("button", Name::Exact("Update password"))
        .click()
        .await?;
    toast(&page, "Password updated").await?;

    let other = Ctx::new().await?;
    let (status, _) = other
        .call(
            "POST",
            "/auth/login",
            Some(&json!({"handle": who, "password": "a much better one 42", "add": false})),
        )
        .await?;
    assert_eq!(status, 200);
    Ok(())
}

#[tokio::test]
async fn a_mismatched_confirmation_is_refused_with_a_toast() -> Result<()> {
    let (_ctx, page, _who) = member("pwx", "account").await?;
    page.label(Name::Exact("Current password"))
        .fill(PASSWORD)
        .await?;
    page.label(Name::Exact("New password"))
        .fill("a much better one 42")
        .await?;
    page.label(Name::Exact("Confirm new password"))
        .fill("something else")
        .await?;
    page.role("button", Name::Exact("Update password"))
        .click()
        .await?;
    toast(&page, "don’t match").await
}

#[tokio::test]
async fn privacy_saves_who_may_message_discoverability_and_muted_words() -> Result<()> {
    let (ctx, page, _who) = member("priv", "privacy").await?;
    page.role("tab", Name::Exact("No one")).click().await?;
    toast(&page, "Message settings saved").await?;
    page.role("switch", Name::Exact("Show me in suggestions and search"))
        .click()
        .await?;
    page.role("switch", Name::Exact("Show me in suggestions and search"))
        .expect_attr("aria-checked", Name::Exact("false"))
        .await?;
    page.label("Word to mute").fill("Spoiler").await?;
    page.role("button", Name::Exact("Mute")).click().await?;
    page.role("button", Name::Exact("Unmute spoiler"))
        .expect_visible()
        .await?;

    let user = me(&ctx).await?;
    assert_eq!(user["settings"]["dmPolicy"], "none");
    assert_eq!(user["settings"]["discoverable"], false);
    assert_eq!(user["settings"]["mutedWords"], json!(["spoiler"]));

    page.reload().await?;
    page.role("tab", Name::Exact("No one"))
        .expect_attr("aria-selected", Name::Exact("true"))
        .await?;
    page.role("switch", Name::Exact("Show me in suggestions and search"))
        .expect_attr("aria-checked", Name::Exact("false"))
        .await?;
    page.role("button", Name::Exact("Unmute spoiler"))
        .click()
        .await?;
    page.text(Name::Exact("No muted words."))
        .expect_visible()
        .await?;
    assert_eq!(me(&ctx).await?["settings"]["mutedWords"], json!([]));
    Ok(())
}

#[tokio::test]
async fn a_blocked_account_is_listed_and_can_be_unblocked() -> Result<()> {
    let (ctx, page, _who) = member("blk", "privacy").await?;
    let other = Ctx::new().await?;
    let target = handle("tgt");
    sign_up_via_api(&other, &target, "Target").await?;
    call(&ctx, "POST", &format!("/users/{target}/block"), None).await?;
    call(&ctx, "POST", &format!("/users/{target}/mute"), None).await?;
    page.reload().await?;
    page.role("button", Name::Exact(&format!("Unblock @{target}")))
        .click()
        .await?;
    toast(&page, "Unblocked").await?;
    page.text(Name::Exact("You haven’t blocked anyone."))
        .expect_visible()
        .await?;
    page.role("button", Name::Exact(&format!("Unmute @{target}")))
        .click()
        .await?;
    page.text(Name::Exact("You haven’t muted anyone."))
        .expect_visible()
        .await
}

#[tokio::test]
async fn a_notification_kind_switches_off_and_stays_off() -> Result<()> {
    let (ctx, page, _who) = member("ntf", "notifications").await?;
    page.role("switch", Name::Exact("Likes")).click().await?;
    page.role("switch", Name::Exact("Likes"))
        .expect_attr("aria-checked", Name::Exact("false"))
        .await?;
    poll_until(&ctx, |user| user["settings"]["notifPrefs"]["like"] == false).await?;
    assert_eq!(
        me(&ctx).await?["settings"]["notifPrefs"]["reply"],
        true,
        "the other kinds keep their state"
    );
    page.reload().await?;
    page.role("switch", Name::Exact("Likes"))
        .expect_attr("aria-checked", Name::Exact("false"))
        .await
}

/// Waits until `/me` satisfies `done` (a switch saves in the background).
async fn poll_until(ctx: &Ctx, done: impl Fn(&Value) -> bool) -> Result<()> {
    for _ in 0..100 {
        if done(&me(ctx).await?) {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    anyhow::bail!("the setting was never saved")
}

#[tokio::test]
async fn another_session_is_listed_and_signs_out() -> Result<()> {
    let (_ctx, page, who) = member("dev", "devices").await?;
    page.text(Name::Exact("This device"))
        .expect_visible()
        .await?;
    let second = Ctx::new().await?;
    let (status, _) = second
        .call(
            "POST",
            "/auth/login",
            Some(&json!({"handle": who, "password": PASSWORD, "add": false})),
        )
        .await?;
    assert_eq!(status, 200);
    page.reload().await?;
    page.css("[data-c='sessions'] li").expect_count(2).await?;
    page.role("button", Name::Re("^Sign out (?!of all)"))
        .click()
        .await?;
    toast(&page, "Session signed out").await?;
    page.css("[data-c='sessions'] li").expect_count(1).await
}

#[tokio::test]
async fn the_automated_flag_and_an_api_token_save_and_the_secret_shows_once() -> Result<()> {
    let (ctx, page, _who) = member("bot", "bots").await?;
    let flag = page.role("switch", Name::Exact("This is an automated account"));
    flag.click().await?;
    flag.expect_attr("aria-checked", Name::Exact("true"))
        .await?;
    poll_until(&ctx, |user| user["user"]["bot"] == true).await?;
    page.reload().await?;
    page.role("switch", Name::Exact("This is an automated account"))
        .expect_attr("aria-checked", Name::Exact("true"))
        .await?;

    page.label("Token name").fill("ci bot").await?;
    page.role("button", Name::Exact("Create token"))
        .click()
        .await?;
    let secret = page.role("dialog", Name::Exact("Your new token"));
    secret.expect_visible().await?;
    secret
        .label("New API token")
        .expect_value(Name::Re("^tuita_"))
        .await?;
    secret.role("button", Name::Exact("Done")).click().await?;
    secret.expect_hidden().await?;
    page.css("[data-token]")
        .expect_text(Name::Contains("ci bot"))
        .await?;
    page.reload().await?;
    page.css("[data-token]")
        .expect_text(Name::Contains("never used"))
        .await?;

    page.role("button", Name::Exact("Revoke token ci bot"))
        .click()
        .await?;
    page.role("dialog", Name::Exact("Revoke “ci bot”?"))
        .role("button", Name::Exact("Revoke"))
        .click()
        .await?;
    page.text(Name::Exact("No tokens yet."))
        .expect_visible()
        .await
}

#[tokio::test]
async fn the_theme_is_stored_on_the_device_and_survives_a_reload() -> Result<()> {
    let (_ctx, page, _who) = member("look", "appearance").await?;
    page.role("tab", Name::Exact("Dark")).click().await?;
    page.css("html")
        .expect_attr("data-theme", Name::Exact("dark"))
        .await?;
    page.role("radio", Name::Exact("Phosphor")).check().await?;
    page.css("html")
        .expect_attr("data-ink", Name::Exact("phosphor"))
        .await?;
    page.reload().await?;
    page.css("html")
        .expect_attr("data-theme", Name::Exact("dark"))
        .await?;
    page.role("tab", Name::Exact("Dark"))
        .expect_attr("aria-selected", Name::Exact("true"))
        .await?;
    page.role("radio", Name::Exact("Phosphor"))
        .expect_checked(true)
        .await
}

#[tokio::test]
async fn about_names_the_instance_and_lists_the_shortcuts() -> Result<()> {
    let (_ctx, page, _who) = member("abt", "about").await?;
    page.css("dt")
        .has_text(Name::Exact("Domain"))
        .expect_visible()
        .await?;
    page.css("kbd").first().expect_visible().await?;
    page.role("link", Name::Exact("Terms of Service"))
        .expect_visible()
        .await
}

#[tokio::test]
async fn an_account_that_is_no_organization_says_so_on_the_affiliation_page() -> Result<()> {
    let (_ctx, page, _who) = member("aff", "affiliation").await?;
    page.text(Name::Contains("not affiliated"))
        .expect_visible()
        .await
}

#[tokio::test]
async fn an_avatar_is_cropped_uploaded_and_removed_again() -> Result<()> {
    let (_ctx, page, _who) = member("face", "profile").await?;
    let card = page.role("region", Name::Exact("Profile picture"));
    card.css("input[type=file]")
        .set_file_bytes("face.jpg", &make_image(1000, 600, 200)?)
        .await?;
    let dialog = page.role("dialog", Name::Exact("Edit profile picture"));
    dialog.expect_visible().await?;
    dialog.role("button", Name::Exact("Rotate")).click().await?;
    dialog.role("button", Name::Exact("Apply")).click().await?;
    toast(&page, "Picture saved.").await?;

    let stored = page.css("#main img[src^=\"/media/\"]").first();
    stored.expect_visible().await?;
    let side = page
        .eval(
            r#"(async () => {
            const img = document.querySelector('#main img[src^="/media/"]');
            await img.decode();
            return [img.naturalWidth, img.naturalHeight];
        })()"#,
        )
        .await?;
    assert_eq!(
        side,
        json!([400, 400]),
        "avatars are stored as 400 x 400 squares"
    );

    card.role("button", Name::Re("Remove")).click().await?;
    toast(&page, "Picture removed.").await?;
    page.css("#main img[src^=\"/media/\"]")
        .expect_count(0)
        .await?;
    let errors = page.console_errors().await;
    assert!(errors.is_empty(), "{errors:#?}");
    Ok(())
}
