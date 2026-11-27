//! Browser tests: profiles, follow lists, the follow button, explore and recent searches, and
//! live notifications.

use std::time::Duration;

use serde_json::{Value, json};
use tuita_e2e::{
    Ctx, Name, Page, Result, Violation,
    helpers::{call, handle, make_image, sign_up_via_api, upload_image},
};

/// A new account and its context.
async fn member(prefix: &str, name: &str) -> Result<(Ctx, String)> {
    let ctx = Ctx::new().await?;
    let who = handle(prefix);
    sign_up_via_api(&ctx, &who, name).await?;
    Ok((ctx, who))
}

/// Creates a post as the signed-in user of `ctx` and returns its id.
async fn post(ctx: &Ctx, body: Value) -> Result<String> {
    let mut request = json!({"text": "", "cw": "", "sensitive": false, "mediaIds": []});
    for (key, value) in body.as_object().into_iter().flatten() {
        request[key] = value.clone();
    }
    let made = call(ctx, "POST", "/posts", Some(&request)).await?;
    Ok(made["id"].as_str().expect("post id").to_owned())
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

/// Every nav badge of `kind` is hidden (`true`) or at least one shows (`false`).
fn badges(kind: &str, hidden: bool) -> String {
    let rule = if hidden { "every" } else { "some" };
    let test = if hidden { "b.hidden" } else { "!b.hidden" };
    format!("[...document.querySelectorAll('[data-badge=\"{kind}\"]')].{rule}((b) => {test})")
}

#[tokio::test]
async fn following_and_unfollowing_from_a_profile() -> Result<()> {
    let (_target, ana) = member("fa", "Ana Followed").await?;
    let (viewer, _bea) = member("fb", "Bea Viewer").await?;
    let page = viewer.page().await?;
    page.goto(&format!("/@{ana}")).await?;

    let header = page.css("[data-c='profile-header']");
    page.role("heading", Name::Exact("Ana Followed"))
        .expect_visible()
        .await?;
    header.role("button", Name::Exact("Follow")).click().await?;
    header
        .role("button", Name::Exact("Following"))
        .expect_attr("aria-pressed", Name::Exact("true"))
        .await?;
    header
        .css("[data-count='followers']")
        .expect_text(Name::Exact("1"))
        .await?;
    let relation = call(&viewer, "GET", &format!("/users/{ana}"), None).await?;
    assert_eq!(relation["relation"]["following"], true);

    page.reload().await?;
    header
        .role("button", Name::Exact("Following"))
        .expect_visible()
        .await?;

    header
        .role("button", Name::Exact("Following"))
        .click()
        .await?;
    header
        .role("button", Name::Exact("Follow"))
        .expect_attr("aria-pressed", Name::Exact("false"))
        .await?;
    header
        .css("[data-count='followers']")
        .expect_text(Name::Exact("0"))
        .await?;
    let relation = call(&viewer, "GET", &format!("/users/{ana}"), None).await?;
    assert_eq!(relation["relation"]["following"], false);
    Ok(())
}

#[tokio::test]
async fn a_pinned_post_shows_once_and_the_profile_loads() -> Result<()> {
    let (ctx, who) = member("pn", "Pin Owner").await?;
    post(&ctx, json!({"text": "An ordinary note"})).await?;
    let pinned = post(&ctx, json!({"text": "This one is pinned"})).await?;
    post(&ctx, json!({"text": "A newer note"})).await?;
    call(&ctx, "POST", &format!("/posts/{pinned}/pin"), None).await?;

    let page = ctx.page().await?;
    page.goto(&format!("/@{who}")).await?;
    let list = page.role("list", Name::Exact("Posts"));
    list.text("An ordinary note").expect_visible().await?;
    list.text("A newer note").expect_visible().await?;
    list.css("article")
        .has_text("This one is pinned")
        .expect_count(1)
        .await?;
    list.css("article").expect_count(3).await?;
    clean(&page, "a profile with a pinned post").await
}

#[tokio::test]
async fn the_media_tab_lists_posts_with_media_and_filters_them() -> Result<()> {
    let (ctx, who) = member("md", "Media Maker").await?;
    post(&ctx, json!({"text": "Words and nothing else"})).await?;
    let image = make_image(640, 480, 40)?;
    let photo = upload_image(&ctx, &image, "A gradient").await?;
    post(
        &ctx,
        json!({"text": "Words with a picture", "mediaIds": [photo.id]}),
    )
    .await?;

    let page = ctx.page().await?;
    page.goto(&format!("/@{who}")).await?;
    page.role("list", Name::Exact("Posts"))
        .text("Words and nothing else")
        .expect_visible()
        .await?;
    page.role("tab", Name::Exact("Media")).click().await?;
    page.expect_url(Name::Re(r"\?tab=media$")).await?;
    let list = page.role("list", Name::Exact("Posts"));
    list.text("Words with a picture").expect_visible().await?;
    list.text("Words and nothing else").expect_hidden().await?;
    page.role("tab", Name::Exact("Media"))
        .expect_attr("aria-selected", Name::Exact("true"))
        .await?;

    page.css("[data-c='media-filter'] .trigger").click().await?;
    page.role("radio", Name::Exact("Videos")).click().await?;
    page.expect_url(Name::Re(r"media=video$")).await?;
    page.text("No videos yet").expect_visible().await?;

    page.css("[data-c='media-filter'] .trigger").click().await?;
    page.role("radio", Name::Exact("Photos")).click().await?;
    page.expect_url(Name::Re(r"media=photo$")).await?;
    page.role("list", Name::Exact("Posts"))
        .text("Words with a picture")
        .expect_visible()
        .await?;
    clean(&page, "the media tab").await
}

#[tokio::test]
async fn the_replies_and_likes_tabs_list_their_posts() -> Result<()> {
    let (ctx, who) = member("rl", "Reply Liker").await?;
    let (other, theirs) = member("rm", "Other Writer").await?;
    let root = post(&other, json!({"text": "Something to answer"})).await?;
    post(&ctx, json!({"text": "My answer to it", "replyTo": root})).await?;
    call(&ctx, "POST", &format!("/posts/{root}/like"), None).await?;
    let _ = theirs;

    let page = ctx.page().await?;
    page.goto(&format!("/@{who}?tab=replies")).await?;
    page.role("list", Name::Exact("Posts"))
        .text("My answer to it")
        .expect_visible()
        .await?;
    page.role("tab", Name::Exact("Likes")).click().await?;
    page.role("list", Name::Exact("Posts"))
        .text("Something to answer")
        .expect_visible()
        .await
}

#[tokio::test]
async fn the_followers_list_paginates_through_the_infinite_list() -> Result<()> {
    let (target, who) = member("fl", "Popular One").await?;
    let _ = &target;
    for i in 0..21 {
        let (follower, _) = member("fw", &format!("Follower {i}")).await?;
        call(&follower, "POST", &format!("/users/{who}/follow"), None).await?;
        follower.close().await?;
    }

    let page = target.page().await?;
    page.goto(&format!("/@{who}/followers")).await?;
    let rows = page
        .role("list", Name::Exact("Followers"))
        .css("[role='listitem']:not(.more)");
    rows.nth(19).expect_visible().await?;
    rows.nth(20).expect_hidden().await?;
    page.eval("document.querySelector('a[data-more]').scrollIntoView()")
        .await?;
    rows.nth(20).expect_visible().await?;
    page.css("a[data-more]").expect_count(0).await?;

    page.role("tab", Name::Exact("Following")).click().await?;
    page.text("Not following anyone").expect_visible().await?;
    Ok(())
}

#[tokio::test]
async fn followers_and_following_rows_carry_working_follow_buttons() -> Result<()> {
    let (viewer, me) = member("fr", "Row Viewer").await?;
    let (_a, ann) = member("fx", "Ann Row").await?;
    call(&viewer, "POST", &format!("/users/{ann}/follow"), None).await?;
    let page = viewer.page().await?;
    page.goto(&format!("/@{me}/following")).await?;
    let row = page.css("[data-c='user-row']").has_text("Ann Row");
    row.role("button", Name::Exact("Following")).click().await?;
    row.role("button", Name::Exact("Follow"))
        .expect_visible()
        .await?;
    settled(&page).await?;
    let relation = call(&viewer, "GET", &format!("/users/{ann}"), None).await?;
    assert_eq!(relation["relation"]["following"], false);
    Ok(())
}

#[tokio::test]
async fn explore_finds_people_and_posts() -> Result<()> {
    let word = format!("zebrafinch{}", &handle("x")[3..]);
    let (ctx, who) = member("ex", &format!("Finch {word}")).await?;
    post(&ctx, json!({"text": format!("A note about {word} birds")})).await?;
    let (viewer, _) = member("ev", "Explore Viewer").await?;

    let page = viewer.page().await?;
    page.goto("/explore").await?;
    page.role("heading", Name::Exact("Explore"))
        .expect_visible()
        .await?;
    let search = page.role("combobox", Name::Exact("Search posts, people and tags"));
    search.fill(&word).await?;
    search.press("Enter").await?;

    page.expect_url(Name::Re(r"/explore\?q=")).await?;
    page.role("list", Name::Exact("Posts"))
        .text(format!("A note about {word} birds").as_str())
        .expect_visible()
        .await?;
    page.role("tab", Name::Exact("Posts"))
        .expect_attr("aria-selected", Name::Exact("true"))
        .await?;

    page.role("tab", Name::Exact("People")).click().await?;
    page.expect_url(Name::Re(r"type=people")).await?;
    let people = page.role("list", Name::Exact("People"));
    people
        .text(format!("@{who}").as_str())
        .expect_visible()
        .await?;
    people.role("button", Name::Exact("Follow")).click().await?;
    people
        .role("button", Name::Exact("Following"))
        .expect_visible()
        .await?;
    settled(&page).await?;
    let relation = call(&viewer, "GET", &format!("/users/{who}"), None).await?;
    assert_eq!(relation["relation"]["following"], true);

    page.goto(&format!("/explore?q={word}-nothing-like-it"))
        .await?;
    page.text("No posts for").expect_visible().await?;
    Ok(())
}

#[tokio::test]
async fn the_explore_home_lists_trends_and_suggestions() -> Result<()> {
    let (ctx, _) = member("eh", "Home Poster").await?;
    let tag = format!("trend{}", &handle("t")[3..]);
    post(&ctx, json!({"text": format!("Talking about #{tag} today")})).await?;
    let (viewer, _) = member("ei", "Home Viewer").await?;
    let page = viewer.page().await?;
    page.goto("/explore").await?;
    page.role("heading", Name::Exact("Trending"))
        .expect_visible()
        .await?;
    page.role("heading", Name::Exact("Who to follow"))
        .expect_visible()
        .await?;
    page.role("tab", Name::Exact("People")).click().await?;
    page.expect_url(Name::Re(r"tab=people$")).await?;
    page.role("heading", Name::Exact("Who to follow"))
        .expect_visible()
        .await?;
    page.goto("/explore").await?;
    clean(&page, "the explore home").await
}

#[tokio::test]
async fn a_recent_search_persists_and_clears() -> Result<()> {
    let (ctx, _) = member("rs", "Recent Searcher").await?;
    let page = ctx.page().await?;
    page.goto("/explore").await?;
    let search = page.role("combobox", Name::Exact("Search posts, people and tags"));
    search.fill("pixel dithering").await?;
    search.press("Enter").await?;
    page.expect_url(Name::Re(r"q=pixel\+dithering|q=pixel%20dithering"))
        .await?;

    page.reload().await?;
    let search = page.role("combobox", Name::Exact("Search posts, people and tags"));
    search.click().await?;
    let recent = page.role("listbox", Name::Exact("Recent searches"));
    recent
        .role("option", Name::Exact("pixel dithering"))
        .expect_visible()
        .await?;
    clean(&page, "the open recent searches").await?;

    page.goto("/explore").await?;
    page.role("combobox", Name::Exact("Search posts, people and tags"))
        .click()
        .await?;
    page.role("option", Name::Exact("pixel dithering"))
        .click()
        .await?;
    page.expect_url(Name::Re(r"q=pixel")).await?;

    page.goto("/explore").await?;
    page.role("combobox", Name::Exact("Search posts, people and tags"))
        .click()
        .await?;
    page.role("button", Name::Exact("Clear")).click().await?;
    page.role("option", Name::Exact("pixel dithering"))
        .expect_hidden()
        .await?;
    page.reload().await?;
    page.role("combobox", Name::Exact("Search posts, people and tags"))
        .click()
        .await?;
    page.role("listbox", Name::Exact("Recent searches"))
        .expect_hidden()
        .await
}

#[tokio::test]
async fn a_search_tip_adds_its_operator_to_the_field() -> Result<()> {
    let (ctx, _) = member("st", "Tip Reader").await?;
    let page = ctx.page().await?;
    page.goto("/explore?q=cats").await?;
    page.role("button", Name::Exact("Search tips"))
        .click()
        .await?;
    page.role("button", Name::Exact("has:media"))
        .click()
        .await?;
    page.role("combobox", Name::Exact("Search posts, people and tags"))
        .expect_value(Name::Exact("cats has:media"))
        .await
}

#[tokio::test]
async fn a_notification_arrives_live_and_the_badge_clears_on_the_notifications_page() -> Result<()>
{
    let (me, mine) = member("nt", "Notified Person").await?;
    let (other, theirs) = member("no", "Notifying Person").await?;
    let page = me.page().await?;
    page.goto("/notifications").await?;
    page.text("No notifications yet").expect_visible().await?;
    // The live stream opens after the page loads; an event sent before it connects is lost.
    tokio::time::sleep(Duration::from_millis(1500)).await;

    call(&other, "POST", &format!("/users/{mine}/follow"), None).await?;
    let list = page.role("list", Name::Exact("Notifications"));
    list.text("Notifying Person").expect_visible().await?;
    page.text("No notifications yet").expect_hidden().await?;

    let note = post(&me, json!({"text": "Please like this"})).await?;
    page.goto("/explore").await?;
    page.wait_for_js(&badges("notifications", true)).await?;
    tokio::time::sleep(Duration::from_millis(1500)).await;
    call(&other, "POST", &format!("/posts/{note}/like"), None).await?;
    page.wait_for_js(&badges("notifications", false)).await?;

    page.goto("/notifications").await?;
    page.role("list", Name::Exact("Notifications"))
        .text("liked your post")
        .expect_visible()
        .await?;
    page.wait_for_js(&badges("notifications", true)).await?;
    page.reload().await?;
    page.wait_for_js(&badges("notifications", true)).await?;
    let _ = theirs;
    clean(&page, "the notifications page").await
}

#[tokio::test]
async fn the_mentions_tab_lists_only_replies_mentions_and_quotes() -> Result<()> {
    let (me, mine) = member("mt", "Mention Target").await?;
    let (other, _) = member("mu", "Mention Maker").await?;
    let root = post(&me, json!({"text": "A post to answer"})).await?;
    call(&other, "POST", &format!("/users/{mine}/follow"), None).await?;
    post(
        &other,
        json!({"text": format!("@{mine} this is a reply"), "replyTo": root}),
    )
    .await?;

    let page = me.page().await?;
    page.goto("/notifications").await?;
    page.role("list", Name::Exact("Notifications"))
        .text("followed you")
        .expect_visible()
        .await?;
    page.role("tab", Name::Exact("Mentions")).click().await?;
    page.expect_url(Name::Re(r"filter=mentions")).await?;
    let list = page.role("list", Name::Exact("Notifications"));
    list.text("this is a reply").expect_visible().await?;
    list.text("followed you").expect_hidden().await
}

#[tokio::test]
async fn the_hover_card_follow_button_follows() -> Result<()> {
    let (author, who) = member("hc", "Card Author").await?;
    let id = post(&author, json!({"text": "Hover over my name"})).await?;
    let (viewer, _) = member("hd", "Card Viewer").await?;
    let page = viewer.page().await?;
    page.goto(&format!("/@{who}/{id}")).await?;

    page.css("article.main a.who").hover().await?;
    let card = page.role("dialog", Name::Exact("Profile card for Card Author"));
    card.expect_visible().await?;
    card.role("button", Name::Exact("Follow")).click().await?;
    card.role("button", Name::Exact("Following"))
        .expect_visible()
        .await?;
    settled(&page).await?;
    let relation = call(&viewer, "GET", &format!("/users/{who}"), None).await?;
    assert_eq!(relation["relation"]["following"], true);
    Ok(())
}

#[tokio::test]
async fn the_right_rail_suggests_people_with_follow_buttons() -> Result<()> {
    member("rr", "Rail Suggestion").await?;
    let (viewer, _) = member("rv", "Rail Viewer").await?;
    let page = viewer.page().await?;
    page.set_viewport(1400, 900).await?;
    page.goto("/").await?;
    let card = page
        .css("[data-c='right-rail'] section")
        .has_text("Who to follow");
    let row = card.css(".person").first();
    let acct = row
        .css("a.who")
        .attr("href")
        .await?
        .expect("the suggestion links its profile");
    let who = acct.trim_start_matches("/@").to_owned();
    row.role("button", Name::Exact("Follow")).click().await?;
    row.role("button", Name::Exact("Following"))
        .expect_visible()
        .await?;
    settled(&page).await?;
    let relation = call(&viewer, "GET", &format!("/users/{who}"), None).await?;
    assert_eq!(relation["relation"]["following"], true);
    Ok(())
}

#[tokio::test]
async fn muting_and_blocking_from_the_profile_menu() -> Result<()> {
    let (_target, ana) = member("mb", "Mute Target").await?;
    let (viewer, _) = member("mv", "Mute Viewer").await?;
    let page = viewer.page().await?;
    page.goto(&format!("/@{ana}")).await?;
    let header = page.css("[data-c='profile-header']");

    header.role("button", Name::Exact("More")).click().await?;
    page.role("menuitem", Name::Re(r"^Mute @")).click().await?;
    page.css("[data-c='profile-header']")
        .text("Muted")
        .expect_visible()
        .await?;
    page.css("#toaster .toast")
        .text("Muted")
        .expect_visible()
        .await?;
    header.role("button", Name::Exact("More")).click().await?;
    page.role("menuitem", Name::Re(r"^Unmute @"))
        .click()
        .await?;
    // The page reloads after the answer; wait for it (the toast is replayed after the reload) before
    // opening the menu again, or the reload closes it.
    page.css("#toaster .toast")
        .text("Unmuted")
        .expect_visible()
        .await?;
    page.role("menuitem", Name::Re(r"^Unmute @"))
        .expect_hidden()
        .await?;

    header.role("button", Name::Exact("More")).click().await?;
    page.role("menuitem", Name::Re(r"^Block @")).click().await?;
    page.role("dialog", Name::Re(r"^Block @"))
        .role("button", Name::Exact("Block"))
        .click()
        .await?;
    page.text("You blocked").expect_visible().await?;
    page.role("button", Name::Exact("Unblock")).click().await?;
    page.role("tab", Name::Exact("Posts"))
        .expect_visible()
        .await
}

#[tokio::test]
async fn an_unreachable_remote_account_explains_and_offers_a_retry() -> Result<()> {
    let (ctx, _) = member("rm", "Remote Looker").await?;
    let page = ctx.page().await?;
    page.goto("/@mara@localhost:1").await?;
    page.text("Couldn’t load @mara@localhost:1")
        .expect_visible()
        .await?;
    page.role("button", Name::Exact("Try again"))
        .expect_visible()
        .await?;
    page.role("heading", Name::Exact("@mara@localhost:1"))
        .expect_visible()
        .await
}

#[tokio::test]
async fn profile_pages_are_accessible() -> Result<()> {
    let (ctx, who) = member("ax", "Axe Subject").await?;
    post(&ctx, json!({"text": "Something to read"})).await?;
    let (viewer, _) = member("ay", "Axe Viewer").await?;
    let page = viewer.page().await?;
    page.goto(&format!("/@{who}")).await?;
    page.role("list", Name::Exact("Posts"))
        .text("Something to read")
        .expect_visible()
        .await?;
    clean(&page, "a profile").await?;
    page.goto(&format!("/@{who}/followers")).await?;
    page.text("No followers yet").expect_visible().await?;
    clean(&page, "a follower list").await?;
    page.goto("/explore?q=nothing-at-all-here").await?;
    page.text("No posts for").expect_visible().await?;
    clean(&page, "an empty search").await?;
    page.goto("/notifications").await?;
    page.text("No notifications yet").expect_visible().await?;
    clean(&page, "empty notifications").await
}

/// Waits until no follow button has a request in flight, so the API shows what the page shows.
async fn settled(page: &Page) -> Result<()> {
    page.wait_for_js("!document.querySelector('[data-follow][data-busy]')")
        .await
}
