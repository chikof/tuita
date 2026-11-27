//! Browser tests: the language picker, the languages' layouts and which language wins (the
//! server-rendered client keeps the choice in the `tuita_lang` cookie and, for an account, in its settings).

use serde_json::json;
use tuita_e2e::{
    Ctx, CtxOptions, Name, Page, Result,
    helpers::{call, handle, overflowing, sign_up_via_api},
};

/// A signed-in account in a browser of `locale`, `width` px wide, with no language of its own.
async fn account(locale: &str, width: u32) -> Result<(Ctx, Page)> {
    let ctx = Ctx::with(CtxOptions {
        locale: Some(locale.to_owned()),
        viewport: Some((width, 844)),
        ..CtxOptions::default()
    })
    .await?;
    sign_up_via_api(&ctx, &handle("lang"), "Lang Tester").await?;
    let page = ctx.page().await?;
    Ok((ctx, page))
}

async fn language_of_account(ctx: &Ctx) -> Result<serde_json::Value> {
    let me = call(ctx, "GET", "/me", None).await?;
    Ok(me["user"]["settings"]["language"].clone())
}

async fn expect_lang(page: &Page, lang: &str) -> Result<()> {
    page.css("html")
        .expect_attr("lang", Name::Exact(lang))
        .await
}

#[tokio::test]
async fn settings_language_switches_to_japanese_keeps_it_after_reload_and_on_the_account()
-> Result<()> {
    let (ctx, page) = account("en-US", 1280).await?;
    page.goto("/settings/language").await?;
    page.role("combobox", Name::Exact("Language"))
        .expect_visible()
        .await?;
    expect_lang(&page, "en").await?;

    page.role("combobox", Name::Exact("Language"))
        .select("日本語")
        .await?;
    expect_lang(&page, "ja").await?;
    for name in ["ホーム", "通知", "設定"] {
        page.role("link", Name::Exact(name))
            .first()
            .expect_visible()
            .await?;
    }
    assert_eq!(ctx.cookie("tuita_lang").await?.as_deref(), Some("ja"));
    assert_eq!(language_of_account(&ctx).await?, json!("ja"));

    page.reload().await?;
    expect_lang(&page, "ja").await?;
    page.role("link", Name::Exact("ホーム"))
        .first()
        .expect_visible()
        .await
}

#[tokio::test]
async fn the_overflow_check_reports_text_that_spills_out_of_or_is_cut_off_by_its_box() -> Result<()>
{
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    page.eval(
        r##"document.body.innerHTML =
        '<a id=a href="#" style="display:block;width:40px;white-space:nowrap">ポストを予約する長い文字列</a>' +
        '<p id=b style="width:60px;height:10px;overflow:hidden">設定 設定 設定 設定 設定 設定</p>'"##,
    )
    .await?;
    let found = overflowing(&page).await?;
    assert!(
        found
            .iter()
            .any(|line| line.starts_with("a ") && line.contains("wider than its box")),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .any(|line| line.starts_with("p ") && line.contains("cut off")),
        "{found:?}"
    );
    Ok(())
}

const PAGES: [&str; 5] = [
    "/",
    "/settings",
    "/settings/language",
    "/explore",
    "/notifications",
];

async fn japanese_does_not_overflow(width: u32, theme: &str) -> Result<()> {
    let (ctx, page) = account("en-US", width).await?;
    call(&ctx, "PATCH", "/me", Some(&json!({"language": "ja"}))).await?;
    page.init_script(&format!(
        "localStorage.setItem('tuita:prefs', JSON.stringify({{mode: '{theme}'}}))"
    ))
    .await?;
    for path in PAGES {
        page.goto(path).await?;
        expect_lang(&page, "ja").await?;
        page.css("html")
            .expect_attr("data-theme", Name::Exact(theme))
            .await?;
        page.css("#main").expect_visible().await?;
        page.css("#main :is(h1, h2, p, button, a)")
            .first()
            .expect_visible()
            .await?;
        page.eval("document.fonts.ready.then(() => true)").await?;
        assert_eq!(overflowing(&page).await?, Vec::<String>::new(), "{path}");
        if path == "/settings/language" {
            page.screenshot(format!(
                "target/e2e/languages/ja-settings-language-{width}-{theme}.png"
            ))
            .await?;
        }
    }
    Ok(())
}

#[tokio::test]
async fn japanese_does_not_overflow_the_layout_at_1280_dark() -> Result<()> {
    japanese_does_not_overflow(1280, "dark").await
}

#[tokio::test]
async fn japanese_does_not_overflow_the_layout_at_1280_light() -> Result<()> {
    japanese_does_not_overflow(1280, "light").await
}

#[tokio::test]
async fn japanese_does_not_overflow_the_layout_at_390_dark() -> Result<()> {
    japanese_does_not_overflow(390, "dark").await
}

#[tokio::test]
async fn japanese_does_not_overflow_the_layout_at_390_light() -> Result<()> {
    japanese_does_not_overflow(390, "light").await
}

async fn picker_looks_right(width: u32) -> Result<()> {
    let (_ctx, page) = account("en-US", width).await?;
    page.goto("/settings/language").await?;
    let card = page.role("region", Name::Exact("Language"));
    card.expect_visible().await?;
    card.screenshot(format!("target/e2e/languages/picker-settings-{width}.png"))
        .await?;

    let guest = Ctx::with(CtxOptions {
        viewport: Some((width, 844)),
        ..CtxOptions::default()
    })
    .await?;
    let landing = guest.page().await?;
    landing.goto("/").await?;
    landing
        .role("combobox", Name::Exact("Language"))
        .expect_visible()
        .await?;
    landing
        .css("header")
        .first()
        .screenshot(format!("target/e2e/languages/picker-landing-{width}.png"))
        .await
}

#[tokio::test]
async fn the_language_picker_looks_right_in_settings_and_on_the_landing_page_at_1280() -> Result<()>
{
    picker_looks_right(1280).await
}

#[tokio::test]
async fn the_language_picker_looks_right_in_settings_and_on_the_landing_page_at_390() -> Result<()>
{
    picker_looks_right(390).await
}

/// Every other language has longer words than English somewhere: no screen may spill sideways at
/// phone width.
#[tokio::test]
async fn the_other_ten_languages_do_not_overflow_the_layout_at_390() -> Result<()> {
    for locale in ["pl", "de", "es", "fr", "pt", "uk", "it", "nl", "tr", "cs"] {
        let (ctx, page) = account("en-US", 390).await?;
        call(&ctx, "PATCH", "/me", Some(&json!({"language": locale}))).await?;
        for path in PAGES {
            page.goto(path).await?;
            expect_lang(&page, locale).await?;
            page.css("#main :is(h1, h2, p, button, a)")
                .first()
                .expect_visible()
                .await?;
            page.eval("document.fonts.ready.then(() => true)").await?;
            assert_eq!(
                overflowing(&page).await?,
                Vec::<String>::new(),
                "{locale} {path}"
            );
        }
        ctx.close().await?;
    }
    Ok(())
}

#[tokio::test]
async fn the_landing_page_picker_switches_language_without_an_account() -> Result<()> {
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    page.goto("/").await?;
    page.role("button", Name::Exact("Sign in"))
        .expect_visible()
        .await?;
    page.role("combobox", Name::Exact("Language"))
        .select("Deutsch")
        .await?;
    expect_lang(&page, "de").await?;
    page.role("button", Name::Exact("Anmelden"))
        .expect_visible()
        .await?;
    assert_eq!(ctx.cookie("tuita_lang").await?.as_deref(), Some("de"));
    page.reload().await?;
    page.role("button", Name::Exact("Anmelden"))
        .expect_visible()
        .await
}

#[tokio::test]
async fn a_signed_out_visitor_gets_the_language_the_server_detects_from_the_browser() -> Result<()>
{
    let ctx = Ctx::with(CtxOptions {
        locale: Some("pl-PL".to_owned()),
        ..CtxOptions::default()
    })
    .await?;
    let page = ctx.page().await?;
    page.goto("/").await?;
    expect_lang(&page, "pl").await?;
    page.role("combobox", Name::Exact("Język"))
        .expect_value(Name::Exact("pl"))
        .await?;
    assert_eq!(
        ctx.cookie("tuita_lang").await?,
        None,
        "detecting is not choosing"
    );
    Ok(())
}

#[tokio::test]
async fn the_account_language_beats_the_browser_and_a_stale_stored_choice() -> Result<()> {
    let ctx = Ctx::with(CtxOptions {
        locale: Some("de-DE".to_owned()),
        ..CtxOptions::default()
    })
    .await?;
    let page = ctx.page().await?;
    // A choice made while signed out is stored in the cookie...
    page.goto("/").await?;
    page.role("combobox", Name::Exact("Sprache"))
        .select("日本語")
        .await?;
    expect_lang(&page, "ja").await?;
    // ...and the account's own language still wins over it.
    sign_up_via_api(&ctx, &handle("lang"), "Lang Tester").await?;
    call(&ctx, "PATCH", "/me", Some(&json!({"language": "fr"}))).await?;
    page.goto("/").await?;
    expect_lang(&page, "fr").await?;
    page.role("link", Name::Exact("Accueil"))
        .first()
        .expect_visible()
        .await
}

#[tokio::test]
async fn opening_the_app_never_writes_the_language_to_the_account() -> Result<()> {
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    page.goto("/").await?;
    page.role("combobox", Name::Exact("Language"))
        .select("日本語")
        .await?;
    expect_lang(&page, "ja").await?;
    sign_up_via_api(&ctx, &handle("lang"), "Lang Tester").await?;
    page.goto("/").await?;
    expect_lang(&page, "ja").await?;
    assert_eq!(language_of_account(&ctx).await?, json!(null));
    Ok(())
}
