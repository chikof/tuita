//! Browser tests: sign-up, sign-in, sign-out, account switching, guests and the language picker.

use tuita_e2e::{
    Ctx, Name, Page, Result, Violation,
    helpers::{PASSWORD, handle, sign_up_via_api, sign_up_via_ui},
};

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

/// Opens the account menu in the rail.
async fn open_menu(page: &Page) -> Result<()> {
    page.css("[data-c='account-menu'] .trigger").click().await
}

/// The handle the account menu's trigger shows.
async fn active_handle(page: &Page) -> Result<String> {
    page.css("[data-c='account-menu'] .trigger .handle")
        .text_content()
        .await
}

#[tokio::test]
async fn signup_through_the_ui_opens_the_signed_in_home() -> Result<()> {
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    let who = handle("sg");
    sign_up_via_ui(&page, "Sign Up", &who).await?;

    page.expect_url(Name::Exact("/")).await?;
    page.role("navigation", Name::Exact("Main"))
        .first()
        .expect_visible()
        .await?;
    assert_eq!(active_handle(&page).await?.trim(), format!("@{who}"));
    clean(&page, "member home").await
}

#[tokio::test]
async fn logging_out_returns_to_the_landing_page() -> Result<()> {
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    let who = handle("lo");
    sign_up_via_ui(&page, "Log Out", &who).await?;

    open_menu(&page).await?;
    page.role("button", Name::Exact(&format!("Log out @{who}")))
        .click()
        .await?;
    page.role("button", Name::Exact("Sign in"))
        .expect_visible()
        .await?;
    page.expect_url(Name::Exact("/")).await?;
    let (status, me) = ctx.call("GET", "/me", None).await?;
    assert_eq!(status, 200);
    assert!(me["user"].is_null(), "still signed in: {me}");
    Ok(())
}

#[tokio::test]
async fn login_returns_to_the_page_named_by_next() -> Result<()> {
    let maker = Ctx::new().await?;
    let who = handle("ln");
    sign_up_via_api(&maker, &who, "Log In").await?;

    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    page.goto("/login?next=%2Fbookmarks").await?;
    page.label("Handle").fill(&who).await?;
    page.label("Password").fill(PASSWORD).await?;
    page.role("button", Name::Exact("Sign in")).click().await?;
    page.expect_url(Name::Exact("/bookmarks")).await?;
    page.role("navigation", Name::Exact("Main"))
        .first()
        .expect_visible()
        .await?;
    assert_eq!(active_handle(&page).await?.trim(), format!("@{who}"));
    Ok(())
}

#[tokio::test]
async fn a_wrong_password_shows_the_error_and_stays() -> Result<()> {
    let maker = Ctx::new().await?;
    let who = handle("wp");
    sign_up_via_api(&maker, &who, "Wrong").await?;

    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    page.goto("/login").await?;
    page.label("Handle").fill(&who).await?;
    page.label("Password").fill("not the password").await?;
    page.role("button", Name::Exact("Sign in")).click().await?;
    page.css("form p.error").expect_visible().await?;
    page.expect_url(Name::Exact("/login")).await
}

#[tokio::test]
async fn a_second_account_can_be_added_and_switched_to() -> Result<()> {
    let other = Ctx::new().await?;
    let second = handle("sb");
    sign_up_via_api(&other, &second, "Second").await?;

    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    let first = handle("sa");
    sign_up_via_ui(&page, "First", &first).await?;

    page.goto("/login?add=1").await?;
    page.role("heading", Name::Exact("Add another account"))
        .expect_visible()
        .await?;
    page.label("Handle").fill(&second).await?;
    page.label("Password").fill(PASSWORD).await?;
    page.role("button", Name::Exact("Sign in and switch"))
        .click()
        .await?;
    page.expect_url(Name::Exact("/")).await?;
    page.role("navigation", Name::Exact("Main"))
        .first()
        .expect_visible()
        .await?;
    assert_eq!(active_handle(&page).await?.trim(), format!("@{second}"));

    open_menu(&page).await?;
    page.role("button", Name::Re(&format!("@{first}$")))
        .click()
        .await?;
    page.wait_for_js(&format!(
        "document.querySelector(\"[data-c='account-menu'] .trigger .handle\")?.textContent.trim() === '@{first}'"
    ))
    .await?;
    let (_, me) = ctx.call("GET", "/me", None).await?;
    assert_eq!(me["user"]["user"]["handle"], first.as_str());
    Ok(())
}

#[tokio::test]
async fn a_guest_on_a_member_page_lands_on_login_with_next() -> Result<()> {
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    page.goto("/explore").await?;
    page.expect_url(Name::Exact("/login?next=%2Fexplore"))
        .await?;
    page.role("heading", Name::Exact("Sign in"))
        .expect_visible()
        .await
}

#[tokio::test]
async fn a_guest_on_a_profile_gets_a_preview_page_then_login() -> Result<()> {
    let maker = Ctx::new().await?;
    let who = handle("pv");
    sign_up_via_api(&maker, &who, "Preview").await?;
    let path = format!("/@{who}");

    let ctx = Ctx::new().await?;
    let (status, body) = ctx.fetch("GET", &path, None).await?;
    assert_eq!(status, 200);
    let html = body.as_str().unwrap_or_default();
    assert!(
        html.contains(r#"property="og:title""#),
        "no og:title in {html}"
    );
    assert!(
        html.contains("http-equiv=\"refresh\""),
        "no refresh in {html}"
    );

    let page = ctx.page().await?;
    page.goto(&path).await?;
    page.expect_url(Name::Re(r"/login\?next=%2F%40")).await?;
    page.role("heading", Name::Exact("Sign in"))
        .expect_visible()
        .await
}

#[tokio::test]
async fn the_landing_page_picker_switches_to_japanese_and_stays() -> Result<()> {
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    page.goto("/").await?;
    page.role("button", Name::Exact("Sign in"))
        .expect_visible()
        .await?;
    page.role("combobox", Name::Exact("Language"))
        .select("日本語")
        .await?;
    page.css("html")
        .expect_attr("lang", Name::Exact("ja"))
        .await?;
    page.role("button", Name::Exact("ログイン"))
        .expect_visible()
        .await?;
    page.reload().await?;
    page.css("html")
        .expect_attr("lang", Name::Exact("ja"))
        .await?;
    page.role("button", Name::Exact("ログイン"))
        .expect_visible()
        .await
}

#[tokio::test]
async fn the_sign_in_and_sign_up_pages_are_accessible() -> Result<()> {
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    for path in ["/", "/login", "/signup"] {
        page.goto(path).await?;
        page.css("main").first().expect_visible().await?;
        clean(&page, path).await?;
    }
    Ok(())
}
