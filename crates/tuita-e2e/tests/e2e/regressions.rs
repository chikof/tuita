//! Bugs reported against the web client: each test fails on the code that had the bug.

use std::time::Duration;

use serde_json::json;
use tuita_e2e::{
    Ctx, Name, Result,
    helpers::{call, handle, sign_up_via_api},
};

async fn member(prefix: &str) -> Result<(Ctx, String)> {
    let ctx = Ctx::new().await?;
    let who = handle(prefix);
    sign_up_via_api(&ctx, &who, "Regression").await?;
    Ok((ctx, who))
}

async fn post(ctx: &Ctx, text: &str) -> Result<String> {
    let body = json!({"text": text, "cw": "", "sensitive": false, "mediaIds": []});
    let made = call(ctx, "POST", "/posts", Some(&body)).await?;
    Ok(made["id"].as_str().expect("post id").to_owned())
}

/// The pill counted from the first post of the list, but "For you" is ranked: its first post is not
/// its newest, so older posts above it read as new ones.
#[tokio::test]
async fn the_new_posts_pill_stays_away_when_nothing_is_new() -> Result<()> {
    let (author, _) = member("pa").await?;
    let mut first = String::new();
    for n in 0..6 {
        let id = post(&author, &format!("older post {n}")).await?;
        if n == 0 {
            first = id;
        }
    }
    // Liked by others, the oldest post ranks first: the first post of the list is not the newest.
    for n in 0..5 {
        let (fan, _) = member(&format!("pl{n}")).await?;
        call(&fan, "POST", &format!("/posts/{first}/like"), None).await?;
    }
    let (viewer, _) = member("pv").await?;
    let page = viewer.page().await?;
    page.goto("/").await?;
    page.text("older post 0").expect_visible().await?;
    // The stream announces the badges on connect, which makes the page ask for the pill.
    page.eval("document.querySelector('[data-island=\"feed\"]').dispatchEvent(new CustomEvent('tuita-stream'))")
        .await?;
    tokio::time::sleep(Duration::from_secs(2)).await;
    page.css("[data-c='new-posts-pill'] .pill")
        .expect_count(0)
        .await?;

    post(&author, "fresh post").await?;
    page.eval("document.querySelector('[data-island=\"feed\"]').dispatchEvent(new CustomEvent('tuita-stream'))")
        .await?;
    page.css("[data-c='new-posts-pill'] .pill")
        .expect_count(1)
        .await
}

/// A click on the video player is the player's: it plays or pauses, and never opens the post.
#[tokio::test]
async fn clicking_a_video_never_opens_the_post() -> Result<()> {
    let (ctx, me) = member("pm").await?;
    let id = post(&ctx, "a post with a player").await?;
    let page = ctx.page().await?;
    page.goto(&format!("/@{me}")).await?;
    let card = page.css("article[data-href]").first();
    card.expect_visible().await?;
    page.eval(
        "document.querySelector('article[data-href]').insertAdjacentHTML('beforeend', '<div data-c=\"video-player\"><div id=\"v\" style=\"width:200px;height:100px\">player</div></div>')",
    )
    .await?;
    page.css("#v").click().await?;
    page.css("#v").click().await?;
    tokio::time::sleep(Duration::from_millis(500)).await;
    let at = page.eval("location.pathname").await?;
    assert_eq!(at, json!(format!("/@{me}")), "the click left the profile");

    card.text("a post with a player").click().await?;
    page.expect_url(Name::Re(&format!("/@{me}/{id}$"))).await
}

/// The edit box shows the text it edits, not a single clipped line.
#[tokio::test]
async fn the_edit_box_is_tall_enough_to_read() -> Result<()> {
    let (ctx, me) = member("pe").await?;
    let id = post(
        &ctx,
        "ahora tuita usa https://github.com/tokio-rs/topcoat para el front",
    )
    .await?;
    let page = ctx.page().await?;
    page.goto(&format!("/@{me}/{id}")).await?;
    page.role("article", Name::Any)
        .first()
        .role("button", Name::Exact("More"))
        .click()
        .await?;
    page.role("menuitem", Name::Exact("Edit post"))
        .click()
        .await?;
    let edit = page.role("dialog", Name::Exact("Edit post"));
    edit.label(Name::Exact("Post text"))
        .expect_visible()
        .await?;
    let height = page
        .eval("document.querySelector('dialog[open] textarea[name=text]').clientHeight")
        .await?;
    assert!(
        height.as_f64().unwrap_or(0.0) >= 100.0,
        "the edit box is {height}px tall"
    );
    Ok(())
}

/// The profile tabs share the width of the page and each underline is as wide as its label.
#[tokio::test]
async fn the_profile_tabs_share_the_width() -> Result<()> {
    let (ctx, me) = member("pt").await?;
    let page = ctx.page().await?;
    page.goto(&format!("/@{me}")).await?;
    page.role("tab", Name::Exact("Posts"))
        .expect_visible()
        .await?;
    let widths = page
        .eval(
            "(() => { const tabs = [...document.querySelectorAll('[data-c=\"link-tabs\"] [role=tab]')]; const list = document.querySelector('[data-c=\"link-tabs\"] [role=tablist]'); const under = document.querySelector('[data-c=\"link-tabs\"] .indicator'); return { tabs: tabs.map((t) => Math.round(t.getBoundingClientRect().width)), list: Math.round(list.getBoundingClientRect().width), under: Math.round(under.getBoundingClientRect().width) }; })()",
        )
        .await?;
    let tabs: Vec<f64> = widths["tabs"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|w| w.as_f64())
        .collect();
    let list = widths["list"].as_f64().unwrap_or(0.0);
    assert_eq!(tabs.len(), 4, "{widths}");
    assert!(
        tabs.iter().all(|w| (w - list / 4.0).abs() <= 2.0),
        "tabs are uneven: {widths}"
    );
    assert!(
        widths["under"].as_f64().unwrap_or(f64::MAX) < tabs[0] * 0.6,
        "{widths}"
    );
    Ok(())
}
