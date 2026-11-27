//! Browser tests: the post page, likes and reposts, polls, the lightbox, the hover card, content
//! warnings, the post dialogs and view counting.

use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tuita_e2e::{
    Ctx, Name, Page, Result, Violation,
    helpers::{call, handle, make_image, sign_up_via_api, upload_image},
};

/// Creates a post as the signed-in user of `ctx` and returns its id.
async fn post(ctx: &Ctx, body: Value) -> Result<String> {
    let mut request = json!({"text": "", "cw": "", "sensitive": false, "mediaIds": []});
    for (key, value) in body.as_object().into_iter().flatten() {
        request[key] = value.clone();
    }
    let made = call(ctx, "POST", "/posts", Some(&request)).await?;
    Ok(made["id"].as_str().expect("post id").to_owned())
}

/// A new account and its context.
async fn member(prefix: &str, name: &str) -> Result<(Ctx, String)> {
    let ctx = Ctx::new().await?;
    let who = handle(prefix);
    sign_up_via_api(&ctx, &who, name).await?;
    Ok((ctx, who))
}

/// The page of a post.
async fn open(ctx: &Ctx, who: &str, id: &str) -> Result<Page> {
    let page = ctx.page().await?;
    page.goto(&format!("/@{who}/{id}")).await?;
    Ok(page)
}

/// The card of the post the thread page is about.
fn main_card(page: &Page) -> tuita_e2e::Loc {
    page.css("article.main")
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

#[tokio::test]
async fn the_post_page_shows_the_post_its_ancestors_and_its_replies() -> Result<()> {
    let (ctx, who) = member("th", "Thread Maker").await?;
    let root = post(&ctx, json!({"text": "A root post to read"})).await?;
    let middle = post(&ctx, json!({"text": "The middle of it", "replyTo": root})).await?;
    post(
        &ctx,
        json!({"text": "An answer to the middle", "replyTo": middle}),
    )
    .await?;

    let page = open(&ctx, &who, &middle).await?;
    page.role("heading", Name::Exact("Post"))
        .expect_visible()
        .await?;
    main_card(&page)
        .text("The middle of it")
        .expect_visible()
        .await?;
    page.role("list", Name::Exact("Replies"))
        .text("An answer to the middle")
        .expect_visible()
        .await?;
    page.css(".ancestors")
        .text("A root post to read")
        .expect_visible()
        .await?;
    let title = page.eval("document.title").await?;
    assert!(
        title
            .as_str()
            .is_some_and(|t| t.contains("The middle of it") && t.contains("Thread Maker")),
        "tab title: {title}"
    );
    clean(&page, "the post page").await
}

#[tokio::test]
async fn like_and_repost_toggle_at_once_and_persist_on_reload() -> Result<()> {
    let (ctx, who) = member("lk", "Liker").await?;
    let id = post(&ctx, json!({"text": "Please like me"})).await?;
    let page = open(&ctx, &who, &id).await?;
    let card = main_card(&page);

    let like = card.role("button", Name::Re(r"^Like \("));
    like.expect_attr("aria-pressed", Name::Exact("false"))
        .await?;
    like.click().await?;
    card.role("button", Name::Exact("Like (1)"))
        .expect_attr("aria-pressed", Name::Exact("true"))
        .await?;

    card.role("button", Name::Re("^Reposts and quotes"))
        .click()
        .await?;
    page.role("menuitem", Name::Exact("Repost")).click().await?;
    card.role("button", Name::Exact("Bookmark")).click().await?;
    // The reload below renders from the server: wait until every request of the clicks landed, or the
    // page can be drawn before the last write (the leaving case has its own test).
    page.css("[data-c='post-actions'][data-pending]")
        .expect_count(0)
        .await?;
    card.role("button", Name::Exact("Bookmark"))
        .expect_attr("aria-pressed", Name::Exact("true"))
        .await?;

    page.reload().await?;
    let card = main_card(&page);
    card.role("button", Name::Exact("Like (1)"))
        .expect_attr("aria-pressed", Name::Exact("true"))
        .await?;
    card.role("button", Name::Exact("Bookmark"))
        .expect_attr("aria-pressed", Name::Exact("true"))
        .await?;
    let thread = call(&ctx, "GET", &format!("/posts/{id}"), None).await?;
    assert_eq!(thread["post"]["viewer"]["liked"], true);
    assert_eq!(thread["post"]["viewer"]["reposted"], true);
    assert_eq!(thread["post"]["counts"]["reposts"], 1);

    card.role("button", Name::Re("^Reposts and quotes"))
        .click()
        .await?;
    page.role("menuitem", Name::Exact("Undo repost"))
        .click()
        .await?;
    like_again(&card).await?;
    Ok(())
}

/// Unliking takes the count back to zero.
async fn like_again(card: &tuita_e2e::Loc) -> Result<()> {
    card.role("button", Name::Exact("Like (1)")).click().await?;
    card.role("button", Name::Exact("Like (0)"))
        .expect_attr("aria-pressed", Name::Exact("false"))
        .await
}

#[tokio::test]
async fn a_refused_like_is_rolled_back_with_a_message() -> Result<()> {
    let (ctx, who) = member("rb", "Rolled Back").await?;
    let id = post(&ctx, json!({"text": "Gone soon"})).await?;
    let page = open(&ctx, &who, &id).await?;
    let card = main_card(&page);
    call(&ctx, "DELETE", &format!("/posts/{id}"), None).await?;
    card.role("button", Name::Re(r"^Like \(")).click().await?;
    page.css("#toaster .toast").expect_visible().await?;
    card.role("button", Name::Exact("Like (0)"))
        .expect_attr("aria-pressed", Name::Exact("false"))
        .await
}

#[tokio::test]
async fn a_poll_vote_shows_the_results() -> Result<()> {
    let (author, who) = member("pa", "Poll Author").await?;
    let id = post(
        &author,
        json!({"text": "Red or blue?", "poll": {"options": ["Red", "Blue"], "multiple": false, "durationHours": 1}}),
    )
    .await?;
    let (voter, _) = member("pv", "Voter").await?;
    let page = open(&voter, &who, &id).await?;

    page.role("button", Name::Exact("Blue"))
        .expect_visible()
        .await?;
    page.role("button", Name::Exact("Red")).click().await?;
    page.css("[data-poll] .row.voted").expect_count(1).await?;
    page.css("[data-poll] .row").expect_count(2).await?;
    page.css("[data-poll] .row.voted .pct")
        .expect_text(Name::Exact("100%"))
        .await?;
    page.css("[data-poll] button.choice")
        .expect_count(0)
        .await?;
    let thread = call(&voter, "GET", &format!("/posts/{id}"), None).await?;
    assert_eq!(thread["post"]["poll"]["voted"], json!([0]));
    clean(&page, "the poll").await
}

#[tokio::test]
async fn the_lightbox_opens_from_the_media_grid_and_steps_through_the_pictures() -> Result<()> {
    let (ctx, who) = member("lb", "Photographer").await?;
    let mut media = Vec::new();
    for (i, (width, height)) in [(640, 480), (800, 600), (960, 540)].into_iter().enumerate() {
        let image = make_image(width, height, i as u32 * 70)?;
        media.push(
            upload_image(&ctx, &image, &format!("Gradient {}", i + 1))
                .await?
                .id,
        );
    }
    let id = post(&ctx, json!({"text": "Three gradients", "mediaIds": media})).await?;
    let page = open(&ctx, &who, &id).await?;

    let grid = main_card(&page).role("group", Name::Exact("3 attachments"));
    grid.expect_visible().await?;
    grid.expect_css("z-index", Name::Exact("1")).await?;
    grid.role("button", Name::Exact("Gradient 1"))
        .click()
        .await?;

    let viewer = page.role("dialog", Name::Exact("Media viewer"));
    viewer.expect_visible().await?;
    viewer
        .role("img", Name::Exact("Gradient 1"))
        .expect_visible()
        .await?;
    viewer
        .css(".top .count")
        .expect_text(Name::Exact("1 / 3"))
        .await?;
    viewer.role("button", Name::Exact("Next")).click().await?;
    viewer
        .role("img", Name::Exact("Gradient 2"))
        .expect_visible()
        .await?;
    viewer
        .css(".caption")
        .expect_text(Name::Exact("Gradient 2"))
        .await?;
    page.press("ArrowRight").await?;
    viewer
        .role("img", Name::Exact("Gradient 3"))
        .expect_visible()
        .await?;
    page.press("ArrowRight").await?;
    viewer
        .role("img", Name::Exact("Gradient 1"))
        .expect_visible()
        .await?;
    page.press("Escape").await?;
    viewer.expect_hidden().await?;
    clean(&page, "the media grid").await
}

#[tokio::test]
async fn a_single_picture_keeps_its_own_shape_and_opens_too() -> Result<()> {
    let (ctx, who) = member("sp", "One Picture").await?;
    let image = make_image(900, 300, 10)?;
    let media = upload_image(&ctx, &image, "A wide strip").await?.id;
    let id = post(&ctx, json!({"text": "One wide strip", "mediaIds": [media]})).await?;
    let page = open(&ctx, &who, &id).await?;
    main_card(&page)
        .role("button", Name::Exact("A wide strip"))
        .click()
        .await?;
    page.role("dialog", Name::Exact("Media viewer"))
        .role("img", Name::Exact("A wide strip"))
        .expect_visible()
        .await?;
    page.press("Escape").await
}

#[tokio::test]
async fn a_hover_card_appears_on_a_handle() -> Result<()> {
    let (ctx, who) = member("hv", "Hover Target").await?;
    let id = post(&ctx, json!({"text": "Point at me"})).await?;
    let page = open(&ctx, &who, &id).await?;

    main_card(&page).css("a.who").hover().await?;
    let card = page.role("dialog", Name::Exact("Profile card for Hover Target"));
    card.expect_visible().await?;
    card.text(format!("@{who}").as_str())
        .expect_visible()
        .await?;
    page.press("Escape").await?;
    card.expect_hidden().await
}

#[tokio::test]
async fn a_content_warning_hides_the_post_until_it_is_opened() -> Result<()> {
    let (ctx, who) = member("cw", "Careful").await?;
    let id = post(
        &ctx,
        json!({"text": "The butler did it", "cw": "Spoilers ahead"}),
    )
    .await?;
    let page = open(&ctx, &who, &id).await?;
    let card = main_card(&page);

    card.text("Spoilers ahead").expect_visible().await?;
    card.text("The butler did it").expect_hidden().await?;
    let toggle = card.role("button", Name::Exact("Show"));
    toggle
        .expect_attr("aria-expanded", Name::Exact("false"))
        .await?;
    toggle.click().await?;
    card.text("The butler did it").expect_visible().await?;
    card.role("button", Name::Exact("Hide"))
        .expect_attr("aria-expanded", Name::Exact("true"))
        .await?;
    clean(&page, "the content warning").await
}

#[tokio::test]
async fn sensitive_media_waits_behind_a_cover() -> Result<()> {
    let (ctx, who) = member("sm", "Sensitive").await?;
    let image = make_image(640, 480, 200)?;
    let media = upload_image(&ctx, &image, "Not for everyone").await?.id;
    let id = post(
        &ctx,
        json!({"text": "Look away", "sensitive": true, "mediaIds": [media]}),
    )
    .await?;
    let page = open(&ctx, &who, &id).await?;
    let card = main_card(&page);

    card.role("group", Name::Exact("Sensitive media"))
        .expect_visible()
        .await?;
    card.role("button", Name::Exact("Show")).click().await?;
    card.role("button", Name::Exact("Hide sensitive media"))
        .expect_visible()
        .await?;
    card.role("button", Name::Exact("Not for everyone"))
        .expect_visible()
        .await
}

#[tokio::test]
async fn editing_through_the_dialog_changes_the_post_and_keeps_the_history() -> Result<()> {
    let (ctx, who) = member("ed", "Editor").await?;
    let id = post(&ctx, json!({"text": "First draft"})).await?;
    let page = open(&ctx, &who, &id).await?;
    let card = main_card(&page);

    card.role("button", Name::Exact("More")).click().await?;
    page.role("menuitem", Name::Exact("Edit post"))
        .click()
        .await?;
    let dialog = page.role("dialog", Name::Exact("Edit post"));
    dialog.expect_visible().await?;
    let save = dialog.role("button", Name::Exact("Save"));
    save.expect_disabled().await?;
    dialog
        .label(Name::Exact("Post text"))
        .fill("Second draft")
        .await?;
    save.expect_enabled().await?;
    save.click().await?;
    card.text("Second draft").expect_visible().await?;
    dialog.expect_hidden().await?;

    card.role("button", Name::Exact("Edited")).click().await?;
    let history = page.role("dialog", Name::Exact("Edit history"));
    history.text("Second draft").expect_visible().await?;
    history.text("First draft").expect_visible().await?;
    history.text("Original").expect_visible().await
}

#[tokio::test]
async fn deleting_a_post_asks_first() -> Result<()> {
    let (ctx, who) = member("dl", "Deleter").await?;
    let id = post(&ctx, json!({"text": "Soon gone"})).await?;
    let page = open(&ctx, &who, &id).await?;
    let card = main_card(&page);

    card.role("button", Name::Exact("More")).click().await?;
    page.role("menuitem", Name::Exact("Delete post"))
        .click()
        .await?;
    let ask = page.role("dialog", Name::Exact("Delete post?"));
    ask.expect_visible().await?;
    ask.role("button", Name::Exact("Cancel")).click().await?;
    ask.expect_hidden().await?;
    let still = call(&ctx, "GET", &format!("/posts/{id}"), None).await?;
    assert_eq!(still["post"]["text"], "Soon gone");

    card.role("button", Name::Exact("More")).click().await?;
    page.role("menuitem", Name::Exact("Delete post"))
        .click()
        .await?;
    ask.role("button", Name::Exact("Delete post"))
        .click()
        .await?;
    page.expect_url(Name::Exact("/")).await?;
    let (status, _) = ctx.call("GET", &format!("/posts/{id}"), None).await?;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn reporting_a_post_sends_the_reason() -> Result<()> {
    let (author, who) = member("ra", "Reported").await?;
    let id = post(&author, json!({"text": "Buy my stuff"})).await?;
    let (reader, _) = member("rr", "Reporter").await?;
    let page = open(&reader, &who, &id).await?;

    main_card(&page)
        .role("button", Name::Exact("More"))
        .click()
        .await?;
    page.role("menuitem", Name::Exact("Report post"))
        .click()
        .await?;
    let dialog = page.role("dialog", Name::Exact("Report post"));
    dialog.expect_visible().await?;
    let send = dialog.role("button", Name::Exact("Send report"));
    send.expect_disabled().await?;
    dialog.role("radio", Name::Exact("Spam")).check().await?;
    send.expect_enabled().await?;
    send.click().await?;
    page.text("Thanks. Your report was sent to the moderators.")
        .expect_visible()
        .await?;
    dialog.expect_hidden().await
}

#[tokio::test]
async fn a_post_that_stayed_on_screen_is_counted_as_viewed() -> Result<()> {
    let (author, who) = member("sn", "Seen").await?;
    let id = post(&author, json!({"text": "Count me"})).await?;
    let (reader, _) = member("rd", "Reader").await?;
    let page = open(&reader, &who, &id).await?;
    main_card(&page).text("Count me").expect_visible().await?;

    let until = Instant::now() + Duration::from_secs(20);
    loop {
        let thread = call(&author, "GET", &format!("/posts/{id}"), None).await?;
        if thread["post"]["counts"]["views"] == 1 {
            break;
        }
        assert!(
            Instant::now() < until,
            "the view was never counted: {thread}"
        );
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    page.reload().await?;
    page.text(Name::Re("^1 View$")).expect_visible().await
}

#[tokio::test]
async fn a_quote_is_drawn_inside_the_post_that_quotes_it() -> Result<()> {
    let (ctx, who) = member("qt", "Quoter").await?;
    let original = post(&ctx, json!({"text": "Quote this please"})).await?;
    let quoting = post(&ctx, json!({"text": "Well said", "quoteOf": original})).await?;
    let page = open(&ctx, &who, &quoting).await?;
    let quote = main_card(&page).css("a.quote");
    quote.text("Quote this please").expect_visible().await?;
    quote.click().await?;
    page.expect_url(Name::Re(&format!("/@{who}/{original}$")))
        .await
}

#[tokio::test]
async fn a_reply_written_on_the_post_page_joins_the_thread() -> Result<()> {
    let (ctx, who) = member("rp", "Replier").await?;
    let id = post(&ctx, json!({"text": "Say something"})).await?;
    let page = open(&ctx, &who, &id).await?;

    page.label(Name::Exact("Post text"))
        .fill("Something")
        .await?;
    page.css("main")
        .role("button", Name::Exact("Reply"))
        .click()
        .await?;
    page.role("list", Name::Exact("Replies"))
        .text("Something")
        .expect_visible()
        .await?;
    main_card(&page)
        .role("button", Name::Exact("Reply (1)"))
        .expect_visible()
        .await
}

#[tokio::test]
async fn toggles_in_quick_succession_keep_each_other_and_survive_leaving_the_page() -> Result<()> {
    let (ctx, who) = member("qk", "Quick Hands").await?;
    let id = post(&ctx, json!({"text": "Tap tap tap"})).await?;
    let page = open(&ctx, &who, &id).await?;
    let card = main_card(&page);

    // Three requests overlap: an answer to one must not undo the optimistic state of another.
    card.role("button", Name::Re(r"^Like \(")).click().await?;
    card.role("button", Name::Exact("Bookmark")).click().await?;
    card.role("button", Name::Exact("Bookmark"))
        .expect_attr("aria-pressed", Name::Exact("true"))
        .await?;
    page.css("[data-c='post-actions'][data-pending]")
        .expect_count(0)
        .await?;
    card.role("button", Name::Exact("Like (1)"))
        .expect_attr("aria-pressed", Name::Exact("true"))
        .await?;
    card.role("button", Name::Exact("Bookmark"))
        .expect_attr("aria-pressed", Name::Exact("true"))
        .await?;

    // Leaving at once after a click does not cancel the request.
    card.role("button", Name::Exact("Bookmark")).click().await?;
    page.goto("/").await?;
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        let thread = call(&ctx, "GET", &format!("/posts/{id}"), None).await?;
        if thread["post"]["viewer"]["bookmarked"] == false {
            return Ok(());
        }
        assert!(Instant::now() < until, "the un-bookmark was lost: {thread}");
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

#[tokio::test]
async fn a_card_is_clipped_but_never_a_scroll_container() -> Result<()> {
    let (ctx, who) = member("sc", "Still Card").await?;
    let id = post(&ctx, json!({"text": "Hold still"})).await?;
    let page = open(&ctx, &who, &id).await?;
    // Scrolling a control into view used to scroll the card itself (overflow: hidden), which moved
    // its buttons under the pointer between press and release and lost the click.
    let top = page
        .eval(
            "(() => { const card = document.querySelector('article.main'); \
             card.scrollTop = 40; card.querySelector('[data-action=like]').scrollIntoView({ block: 'center' }); \
             return card.scrollTop; })()",
        )
        .await?;
    assert_eq!(top, 0, "the card scrolled");
    Ok(())
}
