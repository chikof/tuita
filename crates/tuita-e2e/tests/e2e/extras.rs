//! Browser tests of what the home page and the post page do with posts: polls, scheduling, threads,
//! editing, reporting, view counting and pull to refresh.
//!
//! "two people exchange a direct message and the read receipt arrives" lives in `messages::`. Posts are
//! read off the post page or the Following timeline, so none of these tests needs a profile page.

use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tuita_e2e::{
    Ctx, CtxOptions, Loc, Name, Page, Result,
    helpers::{call, handle, sign_up_via_api, sign_up_via_ui},
};

/// The id of the newest post of `who`.
async fn latest_post(ctx: &Ctx, who: &str) -> Result<String> {
    let page = call(ctx, "GET", &format!("/users/{who}/posts"), None).await?;
    let id = page["items"][0]["id"].as_str().expect("a post");
    Ok(id.to_owned())
}

fn post_body(text: &str) -> Value {
    json!({"text": text, "cw": "", "sensitive": false, "mediaIds": []})
}

/// An RFC 3339 instant `hours` from now, as JSON.
async fn iso_in(page: &Page, hours: f64) -> Result<Value> {
    page.eval(&format!(
        "new Date(Date.now() + {hours} * 3600e3).toISOString()"
    ))
    .await
}

fn send(page: &Page) -> Loc {
    page.css("main").role("button", Name::Exact("Post"))
}

#[tokio::test]
async fn a_poll_is_posted_voted_on_by_someone_else_and_shows_its_results() -> Result<()> {
    let author = handle("poll");
    let (ca, cb) = (Ctx::new().await?, Ctx::new().await?);
    let (pa, pb) = (ca.page().await?, cb.page().await?);
    sign_up_via_ui(&pa, "Pollster", &author).await?;
    sign_up_via_ui(&pb, "Voter", &handle("vote")).await?;

    pa.role("button", Name::Exact("Add a poll")).click().await?;
    pa.label(Name::Exact("Post text"))
        .fill("Which ink?")
        .await?;
    pa.label(Name::Exact("Choice 1")).fill("Bone").await?;
    pa.label(Name::Exact("Choice 2")).fill("Amber").await?;
    send(&pa).click().await?;
    let mine = pa.role("article", Name::Any).has_text("Which ink?");
    mine.text("0 votes").expect_visible().await?;

    let id = latest_post(&ca, &author).await?;
    pb.goto(&format!("/@{author}/{id}")).await?;
    let card = pb.role("article", Name::Any).has_text("Which ink?");
    card.role("button", Name::Contains("Bone")).click().await?;
    card.text("1 vote").expect_visible().await?;
    card.text("100%").expect_visible().await?;
    card.role("button", Name::Contains("Bone"))
        .expect_count(0)
        .await
}

#[tokio::test]
async fn a_post_is_scheduled_and_listed_on_the_scheduled_page() -> Result<()> {
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    sign_up_via_ui(&page, "Planner", &handle("plan")).await?;
    page.label(Name::Exact("Post text"))
        .fill("Out later")
        .await?;
    page.role("button", Name::Exact("Schedule post"))
        .click()
        .await?;
    page.role("button", Name::Re("In an hour")).click().await?;
    page.text(Name::Re("Will post ")).expect_visible().await?;
    page.css("main")
        .role("button", Name::Exact("Schedule"))
        .click()
        .await?;
    page.text(Name::Re("^Scheduled for "))
        .expect_visible()
        .await?;

    page.goto("/scheduled").await?;
    page.text("Out later").expect_visible().await?;
    page.role("list", Name::Exact("Scheduled posts"))
        .text(Name::Exact("Scheduled"))
        .expect_visible()
        .await?;
    let errors = page.console_errors().await;
    assert!(errors.is_empty(), "console errors: {errors:#?}");
    let violations = page.axe().await?;
    assert!(
        violations.is_empty(),
        "axe:\n{}",
        tuita_e2e::Violation::report(&violations)
    );
    Ok(())
}

#[tokio::test]
async fn a_scheduled_post_can_be_moved_posted_now_and_removed() -> Result<()> {
    let ctx = Ctx::new().await?;
    let who = handle("sch");
    sign_up_via_api(&ctx, &who, "Scheduler").await?;
    let page = ctx.page().await?;
    page.goto("/terms").await?;
    for text in ["Move me", "Post me now", "Drop me"] {
        let mut body = post_body(text);
        body["publishAt"] = iso_in(&page, 2.0).await?;
        call(&ctx, "POST", "/scheduled", Some(&body)).await?;
    }
    page.goto("/scheduled").await?;
    let card = |text: &str| page.role("article", Name::Any).has_text(text);

    card("Move me")
        .role("button", Name::Exact("Change time"))
        .click()
        .await?;
    let dialog = page.role("dialog", Name::Exact("Change time"));
    dialog.expect_visible().await?;
    let moved = dialog.label(Name::Exact("Post at"));
    let later = iso_in(&page, 72.0).await?;
    page.eval_fn(
        "(at) => { const i = document.querySelector('#reschedule-when'); const d = new Date(at); \
         const p = (n) => String(n).padStart(2, '0'); \
         i.value = `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}T${p(d.getHours())}:${p(d.getMinutes())}`; \
         i.dispatchEvent(new Event('input', { bubbles: true })); }",
        &later,
    )
    .await?;
    moved.expect_visible().await?;
    dialog.role("button", Name::Exact("Save")).click().await?;
    dialog.expect_hidden().await?;
    let items = call(&ctx, "GET", "/scheduled", None).await?;
    let moved_at = items
        .as_array()
        .and_then(|all| all.iter().find(|item| item["text"] == "Move me"))
        .and_then(|item| item["publishAt"].as_str())
        .expect("the moved post")
        .to_owned();
    let days = page
        .eval_fn(
            "(at) => (Date.parse(at) - Date.now()) / 86400000",
            &json!(moved_at),
        )
        .await?
        .as_f64()
        .unwrap_or_default();
    assert!(
        (2.5..=3.1).contains(&days),
        "moved about three days out: {moved_at}"
    );

    card("Post me now")
        .role("button", Name::Exact("Post now"))
        .click()
        .await?;
    page.text("Posted.").expect_visible().await?;
    card("Post me now")
        .role("link", Name::Exact("View"))
        .expect_visible()
        .await?;
    let posted = call(&ctx, "GET", &format!("/users/{who}/posts"), None).await?;
    assert_eq!(posted["items"][0]["text"], "Post me now");

    card("Drop me")
        .role("button", Name::Exact("Cancel"))
        .click()
        .await?;
    let ask = page.role("dialog", Name::Exact("Cancel this scheduled post?"));
    ask.role("button", Name::Exact("Cancel post"))
        .click()
        .await?;
    card("Drop me").expect_count(0).await?;
    let left = call(&ctx, "GET", "/scheduled", None).await?;
    assert!(
        left.as_array()
            .is_some_and(|all| all.iter().all(|item| item["text"] != "Drop me")),
        "{left}"
    );
    Ok(())
}

#[tokio::test]
async fn a_thread_is_posted_as_a_chain_of_replies() -> Result<()> {
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    let me = handle("chain");
    sign_up_via_ui(&page, "Threader", &me).await?;
    page.label(Name::Exact("Post text"))
        .fill("First of two")
        .await?;
    page.role("button", Name::Exact("Add another post"))
        .click()
        .await?;
    page.label(Name::Exact("Thread post 2"))
        .fill("Second of two")
        .await?;
    page.css("main")
        .role("button", Name::Exact("Post all"))
        .click()
        .await?;
    page.text("Thread of 2 posts sent").expect_visible().await?;
    // The first post is on the home timeline now; a pill from other tests' posts can cover its links,
    // so the chain is read from the post's own page.
    page.role("article", Name::Any)
        .has_text("First of two")
        .expect_visible()
        .await?;
    let posts = call(&ctx, "GET", &format!("/users/{me}/posts"), None).await?;
    let first = posts["items"]
        .as_array()
        .and_then(|all| all.iter().find(|post| post["text"] == "First of two"))
        .and_then(|post| post["id"].as_str())
        .expect("the first post of the thread")
        .to_owned();
    page.goto(&format!("/@{me}/{first}")).await?;
    page.role("article", Name::Any)
        .has_text("Second of two")
        .expect_visible()
        .await
}

#[tokio::test]
async fn a_post_is_edited_its_history_is_shown_and_another_account_reports_it() -> Result<()> {
    let author = handle("edit");
    let (ca, cb) = (Ctx::new().await?, Ctx::new().await?);
    let (pa, pb) = (ca.page().await?, cb.page().await?);
    sign_up_via_ui(&pa, "Editor", &author).await?;
    sign_up_via_ui(&pb, "Reporter", &handle("rep")).await?;

    pa.label(Name::Exact("Post text"))
        .fill("Original post")
        .await?;
    send(&pa).click().await?;
    let mine = pa.role("article", Name::Any).has_text("Original post");
    mine.role("button", Name::Exact("More")).click().await?;
    pa.role("menuitem", Name::Exact("Edit post"))
        .click()
        .await?;
    let edit = pa.role("dialog", Name::Exact("Edit post"));
    edit.label(Name::Exact("Post text"))
        .fill("First post")
        .await?;
    edit.role("button", Name::Exact("Save")).click().await?;
    pa.role("article", Name::Any)
        .has_text("First post")
        .expect_visible()
        .await?;

    // The home timeline can show a pill over its first post; the post page cannot.
    let id = latest_post(&ca, &author).await?;
    pa.goto(&format!("/@{author}/{id}")).await?;
    pa.role("button", Name::Exact("Edited")).click().await?;
    let history = pa.role("dialog", Name::Exact("Edit history"));
    history
        .text(Name::Exact("Original"))
        .expect_visible()
        .await?;
    history.text("Original post").expect_visible().await?;

    pb.goto(&format!("/@{author}/{id}")).await?;
    let theirs = pb.css("article.main");
    theirs.role("button", Name::Exact("More")).click().await?;
    pb.role("menuitem", Name::Exact("Report post"))
        .click()
        .await?;
    let report = pb.role("dialog", Name::Exact("Report post"));
    report.text(Name::Exact("Spam")).click().await?;
    report
        .role("button", Name::Exact("Send report"))
        .click()
        .await?;
    pb.text("Thanks. Your report was sent to the moderators.")
        .expect_visible()
        .await
}

#[tokio::test]
async fn a_post_that_stayed_on_screen_is_counted_as_viewed() -> Result<()> {
    let author = handle("seen");
    let (ca, cb) = (Ctx::new().await?, Ctx::new().await?);
    sign_up_via_api(&ca, &author, "Author").await?;
    sign_up_via_api(&cb, &handle("read"), "Reader").await?;
    let made = call(&ca, "POST", "/posts", Some(&post_body("Count me"))).await?;
    let id = made["id"].as_str().expect("a post id").to_owned();
    call(
        &cb,
        "POST",
        &format!("/users/{author}/follow"),
        Some(&json!({})),
    )
    .await?;

    let page = cb.page().await?;
    page.goto("/?feed=following").await?;
    page.text("Count me").expect_visible().await?;
    let until = Instant::now() + Duration::from_secs(20);
    loop {
        let thread = call(&ca, "GET", &format!("/posts/{id}"), None).await?;
        if thread["post"]["counts"]["views"] == 1 {
            return Ok(());
        }
        assert!(
            Instant::now() < until,
            "the view was never counted: {thread}"
        );
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

#[tokio::test]
async fn pulling_the_home_timeline_down_reloads_it() -> Result<()> {
    let ctx = Ctx::with(CtxOptions {
        viewport: Some((390, 844)),
        touch: true,
        ..CtxOptions::default()
    })
    .await?;
    sign_up_via_api(&ctx, &handle("pull"), "Puller").await?;
    let page = ctx.page().await?;
    page.goto("/").await?;
    page.role("tab", Name::Exact("For you"))
        .expect_visible()
        .await?;
    let loads = "performance.getEntriesByType('resource').filter((e) => e.name.includes('/?feed=foryou')).length";
    page.wait_for_js(&format!(
        "document.querySelector('[data-island=feed]') && {loads} === 0"
    ))
    .await?;
    page.eval(
        "(() => {
            const touch = (y) => new Touch({ identifier: 1, target: document.body, clientX: 100, clientY: y });
            const fire = (type, y) => window.dispatchEvent(new TouchEvent(type, {
                touches: type === 'touchend' ? [] : [touch(y)],
                changedTouches: [touch(y)]
            }));
            fire('touchstart', 100);
            fire('touchmove', 300);
            fire('touchend', 300);
        })()",
    )
    .await?;
    page.wait_for_js(&format!("{loads} > 0")).await?;
    page.css("[data-pull-indicator]").expect_count(0).await
}

#[tokio::test]
async fn the_feed_pages_load_clean_and_pass_axe() -> Result<()> {
    let ctx = Ctx::new().await?;
    let who = handle("clean");
    sign_up_via_api(&ctx, &who, "Clean Pages").await?;
    let made = call(
        &ctx,
        "POST",
        "/posts",
        Some(&post_body("A tagged one #cleantag")),
    )
    .await?;
    let id = made["id"].as_str().expect("a post id").to_owned();
    call(
        &ctx,
        "POST",
        &format!("/posts/{id}/bookmark"),
        Some(&json!({})),
    )
    .await?;
    let list = call(
        &ctx,
        "POST",
        "/lists",
        Some(&json!({"name": "Close ones", "description": "People I trust", "private": true})),
    )
    .await?;
    let list_id = list["id"].as_str().expect("a list id").to_owned();
    let other = Ctx::new().await?;
    let friend = handle("friend");
    sign_up_via_api(&other, &friend, "Friend").await?;
    call(
        &ctx,
        "POST",
        &format!("/lists/{list_id}/members/{friend}"),
        Some(&json!({})),
    )
    .await?;
    let page = ctx.page().await?;
    page.goto("/terms").await?;
    let mut body = post_body("Later");
    body["publishAt"] = iso_in(&page, 2.0).await?;
    call(&ctx, "POST", "/scheduled", Some(&body)).await?;

    for path in [
        "/".to_owned(),
        "/?feed=following".to_owned(),
        "/bookmarks".to_owned(),
        "/bookmarks?tab=likes".to_owned(),
        "/lists".to_owned(),
        format!("/lists/{list_id}"),
        format!("/lists/{list_id}/members"),
        "/scheduled".to_owned(),
        "/tags/cleantag".to_owned(),
        "/tags/nothingtagged".to_owned(),
    ] {
        page.goto(&path).await?;
        page.css("#main").expect_visible().await?;
        page.eval("document.fonts.ready.then(() => true)").await?;
        let errors = page.console_errors().await;
        assert!(errors.is_empty(), "{path}: console errors: {errors:#?}");
        let violations = page.axe().await?;
        assert!(
            violations.is_empty(),
            "{path}: axe violations:\n{}",
            tuita_e2e::Violation::report(&violations)
        );
    }
    Ok(())
}

#[tokio::test]
async fn a_list_is_created_renamed_stocked_and_deleted_through_its_dialogs() -> Result<()> {
    let ctx = Ctx::new().await?;
    sign_up_via_api(&ctx, &handle("own"), "List Owner").await?;
    let other = Ctx::new().await?;
    let friend = handle("mem");
    sign_up_via_api(&other, &friend, "Member").await?;
    let page = ctx.page().await?;

    page.goto("/lists").await?;
    page.text("No lists yet").expect_visible().await?;
    page.role("button", Name::Exact("New list"))
        .first()
        .click()
        .await?;
    let editor = page.role("dialog", Name::Exact("New list"));
    editor.expect_visible().await?;
    let save = editor.role("button", Name::Exact("Save"));
    save.expect_disabled().await?;
    editor.label(Name::Exact("Name")).fill("Friends").await?;
    save.expect_enabled().await?;
    editor
        .label(Name::Exact("Description"))
        .fill("Close ones")
        .await?;
    editor
        .role("switch", Name::Exact("Private list"))
        .click()
        .await?;
    save.click().await?;
    page.role("heading", Name::Exact("Friends"))
        .expect_visible()
        .await?;
    page.text("Close ones").expect_visible().await?;
    page.text(Name::Exact("Private")).expect_visible().await?;
    let url = page.url().await?;
    let id = url.rsplit('/').next().expect("the list id").to_owned();
    let made = call(&ctx, "GET", &format!("/lists/{id}"), None).await?;
    assert_eq!(made["private"], true);
    assert_eq!(made["description"], "Close ones");

    page.role("button", Name::Exact("Edit list"))
        .click()
        .await?;
    let editor = page.role("dialog", Name::Exact("Edit list"));
    editor
        .label(Name::Exact("Name"))
        .expect_value(Name::Exact("Friends"))
        .await?;
    editor
        .label(Name::Exact("Name"))
        .fill("Close friends")
        .await?;
    editor.role("button", Name::Exact("Save")).click().await?;
    page.role("heading", Name::Exact("Close friends"))
        .expect_visible()
        .await?;

    call(
        &ctx,
        "POST",
        &format!("/lists/{id}/members/{friend}"),
        Some(&json!({})),
    )
    .await?;
    page.goto(&format!("/lists/{id}/members")).await?;
    let body = page.css("#main");
    body.text(format!("@{friend}").as_str())
        .expect_visible()
        .await?;
    body.text("1 member").expect_visible().await?;
    page.role("button", Name::Exact("Remove")).click().await?;
    page.text("No members").expect_visible().await?;
    let left = call(&ctx, "GET", &format!("/lists/{id}"), None).await?;
    assert_eq!(left["members"], 0);

    page.role("button", Name::Exact("Delete list"))
        .click()
        .await?;
    let ask = page.role("dialog", Name::Exact("Delete this list?"));
    ask.role("button", Name::Exact("Delete list"))
        .click()
        .await?;
    page.expect_url(Name::Re("/lists$")).await?;
    page.text("No lists yet").expect_visible().await?;
    let errors = page.console_errors().await;
    assert!(errors.is_empty(), "console errors: {errors:#?}");
    Ok(())
}

#[tokio::test]
async fn the_list_picker_makes_a_new_list_with_the_person_in_it() -> Result<()> {
    let (ctx, other) = (Ctx::new().await?, Ctx::new().await?);
    sign_up_via_api(&ctx, &handle("pk"), "Picker").await?;
    let friend = handle("pf");
    sign_up_via_api(&other, &friend, "Friend").await?;

    let page = ctx.page().await?;
    page.goto(&format!("/@{friend}")).await?;
    page.css("[data-c='profile-header']")
        .role("button", Name::Exact("More"))
        .click()
        .await?;
    page.role("menuitem", Name::Re("^Add to lists"))
        .click()
        .await?;
    let picker = page.role("dialog", Name::Re("^Add .* to lists"));
    picker
        .text("You have no lists yet.")
        .expect_visible()
        .await?;
    picker
        .role("button", Name::Exact("New list"))
        .click()
        .await?;
    let editor = page.role("dialog", Name::Exact("New list"));
    editor.label(Name::Exact("Name")).fill("Reading").await?;
    editor.role("button", Name::Exact("Save")).click().await?;

    let picker = page.role("dialog", Name::Re("^Add .* to lists"));
    picker
        .role("checkbox", Name::Exact("Reading"))
        .expect_checked(true)
        .await?;
    let lists = call(&ctx, "GET", &format!("/lists?member={friend}"), None).await?;
    assert_eq!(lists[0]["name"], "Reading");
    assert_eq!(lists[0]["hasMember"], true);
    Ok(())
}
