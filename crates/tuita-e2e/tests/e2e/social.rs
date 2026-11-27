//! Browser tests: following, the Following timeline, likes and the live notification toast,
//! and the home feed's tabs, cookie and live parts.
//!
//! `b_follows_a...` drives the profile page and its Follow button; `following_feed_...` does the same
//! through the API and runs on the home page alone.

use serde_json::json;
use tuita_e2e::{
    Ctx, Name, Page, Result,
    helpers::{call, handle, sign_up_via_api, sign_up_via_ui},
};

async fn no_errors(pages: &[&Page]) {
    for page in pages {
        let errors = page.console_errors().await;
        assert!(errors.is_empty(), "console errors: {errors:#?}");
    }
}

/// The rest of the flow: A posts on the home page, B sees it on Following and likes it, A gets a toast.
async fn post_see_like(page_a: &Page, page_b: &Page, a: &str, b: &str) -> Result<()> {
    let text = format!("hello from {a} #tuita");
    page_a.goto("/").await?;
    page_a.label(Name::Exact("Post text")).fill(&text).await?;
    page_a
        .css("main")
        .role("button", Name::Exact("Post"))
        .click()
        .await?;
    page_a
        .css("main")
        .text(text.as_str())
        .expect_visible()
        .await?;

    page_b.goto("/").await?;
    page_b.role("tab", Name::Exact("Following")).click().await?;
    let card = page_b.role("article", Name::Any).has_text(text.as_str());
    card.expect_visible().await?;

    card.role("button", Name::Re(r"^Like \(")).click().await?;
    card.role("button", Name::Exact("Like (1)"))
        .expect_attr("aria-pressed", Name::Exact("true"))
        .await?;
    page_a
        .text(format!("@{b} liked your post").as_str())
        .expect_visible()
        .await
}

#[tokio::test]
async fn b_follows_a_a_posts_b_sees_it_on_following_and_likes_it_a_gets_a_toast() -> Result<()> {
    let (a, b) = (handle("alice"), handle("bob"));
    let (ctx_a, ctx_b) = (Ctx::new().await?, Ctx::new().await?);
    let (page_a, page_b) = (ctx_a.page().await?, ctx_b.page().await?);
    sign_up_via_ui(&page_a, "Alice", &a).await?;
    sign_up_via_ui(&page_b, "Bob", &b).await?;

    page_b.goto(&format!("/@{a}")).await?;
    page_b
        .css("#main")
        .role("button", Name::Exact("Follow"))
        .click()
        .await?;
    page_b
        .role("button", Name::Exact("Following"))
        .expect_visible()
        .await?;

    post_see_like(&page_a, &page_b, &a, &b).await?;
    no_errors(&[&page_a, &page_b]).await;
    ctx_a.close().await?;
    ctx_b.close().await
}

#[tokio::test]
async fn following_feed_shows_a_followed_accounts_post_which_the_other_can_like() -> Result<()> {
    let (a, b) = (handle("fa"), handle("fb"));
    let (ctx_a, ctx_b) = (Ctx::new().await?, Ctx::new().await?);
    let (page_a, page_b) = (ctx_a.page().await?, ctx_b.page().await?);
    sign_up_via_ui(&page_a, "Alice", &a).await?;
    sign_up_via_ui(&page_b, "Bob", &b).await?;
    call(
        &ctx_b,
        "POST",
        &format!("/users/{a}/follow"),
        Some(&json!({})),
    )
    .await?;

    post_see_like(&page_a, &page_b, &a, &b).await?;
    no_errors(&[&page_a, &page_b]).await;
    ctx_a.close().await?;
    ctx_b.close().await
}

#[tokio::test]
async fn the_feed_tab_is_a_link_and_the_choice_survives_a_reload() -> Result<()> {
    let ctx = Ctx::new().await?;
    sign_up_via_api(&ctx, &handle("tab"), "Tabber").await?;
    let page = ctx.page().await?;
    page.goto("/").await?;
    page.role("tab", Name::Exact("For you"))
        .expect_attr("aria-selected", Name::Exact("true"))
        .await?;
    page.role("tab", Name::Exact("Following")).click().await?;
    page.expect_url(Name::Re(r"/\?feed=following$")).await?;
    page.role("tab", Name::Exact("Following"))
        .expect_attr("aria-selected", Name::Exact("true"))
        .await?;
    page.text("Your circle is quiet").expect_visible().await?;

    page.goto("/").await?;
    page.role("tab", Name::Exact("Following"))
        .expect_attr("aria-selected", Name::Exact("true"))
        .await?;
    page.role("link", Name::Exact("Find people"))
        .expect_visible()
        .await?;
    page.role("tab", Name::Exact("For you")).click().await?;
    page.expect_url(Name::Re(r"/\?feed=foryou$")).await?;
    page.goto("/").await?;
    page.role("tab", Name::Exact("For you"))
        .expect_attr("aria-selected", Name::Exact("true"))
        .await?;
    no_errors(&[&page]).await;
    Ok(())
}

#[tokio::test]
async fn a_new_post_of_a_followed_account_raises_the_pill_which_starts_the_timeline_over()
-> Result<()> {
    let (a, b) = (handle("pa"), handle("pb"));
    let (ctx_a, ctx_b) = (Ctx::new().await?, Ctx::new().await?);
    sign_up_via_api(&ctx_a, &a, "Writer").await?;
    sign_up_via_api(&ctx_b, &b, "Reader").await?;
    call(
        &ctx_b,
        "POST",
        &format!("/users/{a}/follow"),
        Some(&json!({})),
    )
    .await?;
    let post = |text: &str| json!({"text": text, "cw": "", "sensitive": false, "mediaIds": []});
    call(&ctx_a, "POST", "/posts", Some(&post("an older one"))).await?;

    let page = ctx_b.page().await?;
    page.goto("/?feed=following").await?;
    page.role("article", Name::Any)
        .has_text("an older one")
        .expect_visible()
        .await?;
    page.role("button", Name::Re("^Show .* new post"))
        .expect_count(0)
        .await?;

    call(&ctx_a, "POST", "/posts", Some(&post("a fresh one"))).await?;
    // A badge event from the stream, or the 30 s poll, asks for the pill; send the event instead of waiting.
    page.eval("document.querySelector('[data-island=feed]').dispatchEvent(new CustomEvent('tuita-stream', { bubbles: true }))")
        .await?;
    let pill = page.role("button", Name::Re("^Show .* new post"));
    pill.expect_visible().await?;
    pill.click().await?;
    page.role("article", Name::Any)
        .has_text("a fresh one")
        .expect_visible()
        .await?;
    pill.expect_count(0).await?;
    no_errors(&[&page]).await;
    Ok(())
}
