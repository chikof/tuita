//! Smoke test: the app boots in a real browser with nothing wrong on the landing page.

use tuita_e2e::{Ctx, Result, Violation};

#[tokio::test]
async fn landing_loads_clean() -> Result<()> {
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    page.goto("/").await?;

    page.css("html[data-theme]").expect_count(1).await?;
    page.wait_for_js(r#"document.querySelector('script[src*="/_topcoat/assets/topcoat-"]')"#)
        .await?;

    let errors = page.console_errors().await;
    assert!(errors.is_empty(), "console errors: {errors:#?}");
    let violations = page.axe().await?;
    assert!(
        violations.is_empty(),
        "axe violations:\n{}",
        Violation::report(&violations)
    );
    Ok(())
}

#[tokio::test]
async fn an_unknown_page_shows_not_found_and_stays_put() -> Result<()> {
    let ctx = Ctx::new().await?;
    let me = tuita_e2e::helpers::handle("nf");
    tuita_e2e::helpers::sign_up_via_api(&ctx, &me, "Not Found").await?;
    let page = ctx.page().await?;
    page.init_script(
        "{ const k = 'loads' + location.pathname; sessionStorage.setItem(k, String(Number(sessionStorage.getItem(k) ?? 0) + 1)); }",
    )
    .await?;
    for path in [
        "/nothing/here",
        "/nothing",
        "/@nobodyhere",
        "/@nobodyhere/00000000-0000-0000-0000-000000000000",
        "/lists/00000000-0000-0000-0000-000000000000",
        "/messages/nobodyhere",
        "/settings/nothing",
        "/admin/nothing",
        "/lists/not-a-uuid",
        "/@me/not-a-post",
        "/@me/00000000-0000-0000-0000-000000000000",
        "/@me/nothing",
        "/tags/nothing/else",
    ] {
        let path = &path.replace("@me", &format!("@{me}"));
        page.goto("/").await?;
        page.goto(path).await?;
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        let loads = page
            .eval(&format!(
                "Number(sessionStorage.getItem('loads' + {path:?}))"
            ))
            .await?;
        let at = page.eval("location.pathname").await?;
        assert_eq!(
            at.as_str().map(str::to_owned),
            path.split('?').next().map(str::to_owned),
            "{path} went elsewhere"
        );
        assert_eq!(loads, serde_json::json!(1), "{path} reloaded itself");
    }
    Ok(())
}
