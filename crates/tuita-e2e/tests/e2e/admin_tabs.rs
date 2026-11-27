//! Browser tests: the admin console. Every tab renders for an admin and is axe clean at 1280 px, a
//! moderator gets exactly the tabs that are not admin only, and each tab runs one action.

use serde_json::{Value, json};
use tuita_e2e::{
    Ctx, Name, Page, Result, Violation,
    helpers::{call, handle, make_image, new_admin, sign_up_via_api},
};

const ADMIN_TABS: [(&str, &str); 10] = [
    ("/admin", "Overview"),
    ("/admin/reports", "Reports"),
    ("/admin/users", "People"),
    ("/admin/instance", "Instance"),
    ("/admin/federation", "Federation"),
    ("/admin/announcements", "Announcements"),
    ("/admin/invites", "Invites"),
    ("/admin/emotes", "Emotes"),
    ("/admin/filter", "Content filter"),
    ("/admin/audit", "Audit log"),
];

const MODERATOR_TABS: [&str; 5] = ["Overview", "Reports", "People", "Emotes", "Content filter"];

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

/// Shows a toast with exactly `text`.
async fn toast(page: &Page, text: &str) -> Result<()> {
    page.css("#toaster .toast")
        .has_text(Name::Exact(text))
        .expect_visible()
        .await
}

/// An account that is not staff, with one public post.
async fn member(prefix: &str, text: &str) -> Result<(Ctx, String, String)> {
    let ctx = Ctx::new().await?;
    let who = handle(prefix);
    sign_up_via_api(&ctx, &who, "Some Member").await?;
    let post = call(
        &ctx,
        "POST",
        "/posts",
        Some(&json!({"text": text, "cw": "", "sensitive": false, "mediaIds": []})),
    )
    .await?;
    let id = post["id"].as_str().unwrap_or_default().to_owned();
    Ok((ctx, who, id))
}

/// Files a report about `post` as `reporter`.
async fn report(reporter: &Ctx, post: &str, note: &str) -> Result<()> {
    call(
        reporter,
        "POST",
        "/reports",
        Some(&json!({"postId": post, "reason": "spam", "note": note})),
    )
    .await?;
    Ok(())
}

#[tokio::test]
async fn every_tab_renders_for_an_admin_and_is_axe_clean() -> Result<()> {
    let (staff, _me) = new_admin("tabs").await?;
    let (bad, bad_handle, post) = member("bad", "buy followers now").await?;
    report(&staff, &post, "an ad").await?;
    call(
        &staff,
        "POST",
        "/admin/invites",
        Some(&json!({"note": "for the tabs", "maxUses": 2, "days": 7})),
    )
    .await?;
    call(
        &staff,
        "PATCH",
        &format!("/admin/users/{bad_handle}"),
        Some(&json!({"verified": true})),
    )
    .await?;

    for (path, label) in ADMIN_TABS {
        let page = staff.page().await?;
        page.set_viewport(1280, 900).await?;
        page.goto(path).await?;
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        page.role("navigation", Name::Exact("Admin sections"))
            .role("link", Name::Exact(label))
            .expect_attr("aria-current", Name::Exact("page"))
            .await?;
        page.role("heading", Name::Exact("Admin"))
            .expect_visible()
            .await?;
        clean(&page, path).await?;
        page.close().await?;
    }
    bad.close().await?;
    Ok(())
}

#[tokio::test]
async fn a_moderator_gets_exactly_the_tabs_that_are_not_admin_only() -> Result<()> {
    let (staff, _me) = new_admin("mod").await?;
    let moderator = Ctx::new().await?;
    let who = handle("mdr");
    sign_up_via_api(&moderator, &who, "Mod Erator").await?;
    call(
        &staff,
        "PATCH",
        &format!("/admin/users/{who}"),
        Some(&json!({"role": "moderator"})),
    )
    .await?;

    let page = moderator.page().await?;
    page.goto("/admin").await?;
    let nav = page.role("navigation", Name::Exact("Admin sections"));
    nav.role("link", Name::Any).first().expect_visible().await?;
    assert_eq!(
        nav.role("link", Name::Any).texts().await?,
        MODERATOR_TABS,
        "the tabs a moderator sees"
    );

    for (path, label) in ADMIN_TABS {
        let status = page
            .eval(&format!(
                "fetch({path:?}, {{credentials: 'same-origin'}}).then((r) => r.status)"
            ))
            .await?;
        let allowed = MODERATOR_TABS.contains(&label);
        assert_eq!(
            status,
            if allowed { 200 } else { 403 },
            "{path} for a moderator"
        );
    }
    page.goto("/admin/federation").await?;
    page.role("heading", Name::Exact("Staff only"))
        .expect_visible()
        .await?;
    page.role("button", Name::Exact("Go home"))
        .expect_visible()
        .await?;
    let status = page
        .eval("fetch('/admin/nope', {credentials: 'same-origin'}).then((r) => r.status)")
        .await?;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn a_plain_member_gets_the_denied_state() -> Result<()> {
    // The first account of a fresh instance becomes its admin: make sure another one came first.
    let first = Ctx::new().await?;
    sign_up_via_api(&first, &handle("first"), "First Person").await?;
    let (ctx, _who, _post) = member("plain", "just a post").await?;
    let page = ctx.page().await?;
    page.goto("/admin").await?;
    page.role("heading", Name::Exact("Staff only"))
        .expect_visible()
        .await?;
    page.role("navigation", Name::Exact("Admin sections"))
        .expect_hidden()
        .await?;
    let status = page
        .eval("fetch('/admin', {credentials: 'same-origin'}).then((r) => r.status)")
        .await?;
    assert_eq!(status, 403);
    Ok(())
}

#[tokio::test]
async fn reports_are_resolved_from_the_queue() -> Result<()> {
    let (staff, _me) = new_admin("rep").await?;
    let (bad, bad_handle, post) = member("bad", "buy followers now").await?;
    let text = format!("buy followers now {bad_handle}");
    // The post carries the handle so the card can be told apart from reports of other tests.
    let (_other, _who, second) = member("ad", &text).await?;
    report(&staff, &post, "an ad").await?;
    report(&staff, &second, "another ad").await?;

    let page = staff.page().await?;
    page.goto("/admin/reports").await?;
    let card = page.css("[data-report]").has_text(text.as_str());
    card.expect_visible().await?;
    card.role("button", Name::Exact("Dismiss")).click().await?;
    toast(&page, "Dismissed").await?;
    card.expect_count(0).await?;

    page.role("tab", Name::Exact("Resolved")).click().await?;
    let done = page.css("[data-report]").has_text(text.as_str());
    done.expect_visible().await?;
    done.text(Name::Re("Dismissed by @"))
        .expect_visible()
        .await?;
    bad.close().await?;
    Ok(())
}

#[tokio::test]
async fn a_report_with_a_post_can_delete_it_and_suspend_the_account() -> Result<()> {
    let (staff, _me) = new_admin("rds").await?;
    let (bad, bad_handle, post) = member("bad", "free gift cards inside").await?;
    report(&staff, &post, "scam").await?;

    let page = staff.page().await?;
    page.goto("/admin/reports").await?;
    let text = format!("@{bad_handle}");
    let card = page.css("[data-report]").has_text(text.as_str());
    card.role("button", Name::Exact("Delete and suspend"))
        .click()
        .await?;
    page.role("dialog", Name::Any)
        .role("button", Name::Exact("Confirm"))
        .click()
        .await?;
    card.expect_count(0).await?;
    let detail = call(&staff, "GET", &format!("/admin/users/{bad_handle}"), None).await?;
    assert_eq!(detail["user"]["user"]["suspended"], Value::Bool(true));
    bad.close().await?;
    Ok(())
}

#[tokio::test]
async fn people_are_found_verified_and_opened() -> Result<()> {
    let (staff, _me) = new_admin("usr").await?;
    let (member_ctx, who, _post) = member("vfy", "hello").await?;

    let page = staff.page().await?;
    page.goto("/admin/users").await?;
    page.role("searchbox", Name::Exact("Search people"))
        .fill(&who)
        .await?;
    let row = page.css("[data-c='admin-user-row']").has_text(who.as_str());
    row.expect_visible().await?;
    page.css("[data-c='admin-user-row']")
        .expect_count(1)
        .await?;

    row.role("button", Name::Exact(&format!("Manage @{who}")))
        .click()
        .await?;
    page.role("menuitem", Name::Exact("Verify")).click().await?;
    toast(&page, &format!("@{who} is verified")).await?;
    row.role("img", Name::Exact("Verified account"))
        .expect_visible()
        .await?;
    let detail = call(&staff, "GET", &format!("/admin/users/{who}"), None).await?;
    assert_eq!(detail["user"]["user"]["verified"], Value::Bool(true));

    row.role("button", Name::Exact(&format!("Details for @{who}")))
        .click()
        .await?;
    let sheet = page.role("dialog", Name::Exact("Account details"));
    sheet
        .role("heading", Name::Exact("Moderation history"))
        .expect_visible()
        .await?;
    sheet
        .text(Name::Contains("Account verified"))
        .expect_visible()
        .await?;
    member_ctx.close().await?;
    Ok(())
}

#[tokio::test]
async fn instance_settings_are_saved() -> Result<()> {
    let (staff, _me) = new_admin("ins").await?;
    let page = staff.page().await?;
    page.goto("/admin/instance").await?;
    let name = format!("Pixel {}", handle("n"));
    page.label("Instance name").fill(&name).await?;
    page.role("button", Name::Exact("Save")).click().await?;
    toast(&page, "Instance settings saved").await?;
    let settings = call(&staff, "GET", "/admin/settings", None).await?;
    assert_eq!(settings["name"], json!(name));
    assert_eq!(settings["signups"], json!("open"));

    page.reload().await?;
    page.label("Instance name")
        .expect_value(Name::Exact(&name))
        .await?;
    page.role("radio", Name::Exact("Open"))
        .expect_checked(true)
        .await?;
    Ok(())
}

#[tokio::test]
async fn a_domain_is_blocked_softened_and_unblocked() -> Result<()> {
    let (staff, _me) = new_admin("fed").await?;
    let page = staff.page().await?;
    page.goto("/admin/federation").await?;
    page.css("[data-fingerprint]")
        .expect_attr("data-fingerprint", Name::Re("^[0-9a-f]{64}$"))
        .await?;
    page.css("[data-fingerprint]")
        .expect_text(Name::Re("^([0-9a-f]{8} ){7}[0-9a-f]{8}$"))
        .await?;

    let form = page
        .css("form")
        .has(&page.role("button", Name::Exact("Block instance")));
    form.role("button", Name::Exact("Block instance"))
        .click()
        .await?;
    form.role("alert", Name::Any)
        .expect_text(Name::Exact("Enter a domain."))
        .await?;

    let domain = format!(
        "blocked-{}.example",
        handle("d").to_lowercase().replace('_', "-")
    );
    form.label("Domain").fill(&domain).await?;
    form.label("Reason").fill("Spam waves").await?;
    form.role("button", Name::Exact("Block instance"))
        .click()
        .await?;
    let row = page.css(&format!("[data-block='{domain}']"));
    row.expect_visible().await?;
    row.text(Name::Contains("Spam waves"))
        .expect_visible()
        .await?;
    row.expect_attr("data-severity", Name::Exact("silence"))
        .await?;

    row.label("Severity").select("suspend").await?;
    toast(&page, &format!("{domain} is now set to Suspend")).await?;
    row.expect_attr("data-severity", Name::Exact("suspend"))
        .await?;

    row.role(
        "button",
        Name::Exact(&format!("Remove the block on {domain}")),
    )
    .click()
    .await?;
    page.role(
        "dialog",
        Name::Exact(&format!("Remove the block on {domain}?")),
    )
    .role("button", Name::Exact("Remove"))
    .click()
    .await?;
    row.expect_count(0).await?;
    toast(&page, &format!("{domain} is no longer blocked")).await?;
    Ok(())
}

#[tokio::test]
async fn announcements_are_published_and_ended() -> Result<()> {
    let (staff, _me) = new_admin("ann").await?;
    let page = staff.page().await?;
    page.goto("/admin/announcements").await?;
    let text = format!("Maintenance window {}", handle("a"));
    page.role("textbox", Name::Exact("Message"))
        .fill(&text)
        .await?;
    page.role("button", Name::Exact("Publish")).click().await?;

    let item = page.css("[data-announcement]").has_text(text.as_str());
    item.expect_visible().await?;
    toast(&page, "Announcement published").await?;
    let banner = page.role("region", Name::Exact("Announcements"));
    banner.text(Name::Contains(&text)).expect_visible().await?;

    item.role("button", Name::Exact("End")).click().await?;
    item.text(Name::Contains("Not showing"))
        .expect_visible()
        .await?;
    item.role("button", Name::Exact("End"))
        .expect_count(0)
        .await?;
    banner.text(Name::Contains(&text)).expect_count(0).await?;
    Ok(())
}

#[tokio::test]
async fn invites_are_created_and_revoked() -> Result<()> {
    let (staff, _me) = new_admin("inv").await?;
    let page = staff.page().await?;
    page.goto("/admin/invites").await?;
    let note = format!("for {}", handle("i"));
    page.label("Note").fill(&note).await?;
    page.label("Uses").select("1").await?;
    page.role("button", Name::Exact("Create code"))
        .click()
        .await?;
    let entry = page.css("[data-invite]").has_text(note.as_str());
    entry.expect_visible().await?;
    let code = entry.css("code").text_content().await?;
    assert_eq!(code.trim().len(), 11, "{code}");
    assert!(
        code.trim().chars().enumerate().all(|(i, c)| if i == 5 {
            c == '-'
        } else {
            c.is_ascii_alphanumeric()
        }),
        "{code}"
    );

    entry.role("button", Name::Exact("Revoke")).click().await?;
    page.role("dialog", Name::Any)
        .role("button", Name::Exact("Revoke"))
        .click()
        .await?;
    entry.text(Name::Exact("Revoked")).expect_visible().await?;
    entry
        .role("button", Name::Exact("Revoke"))
        .expect_count(0)
        .await?;
    Ok(())
}

#[tokio::test]
async fn emotes_are_uploaded_recategorised_and_deleted() -> Result<()> {
    let (staff, _me) = new_admin("emo").await?;
    let page = staff.page().await?;
    page.goto("/admin/emotes").await?;
    let code = format!("zq{}", handle("e").to_lowercase().replace('-', "_"));
    let shortcode = &code[..code.len().min(30)];

    page.label(Name::Exact("Shortcode")).fill("a").await?;
    page.text(Name::Re("Use only letters, numbers and underscores"))
        .expect_visible()
        .await?;
    page.css("[data-island=emote-form] input[type=file]")
        .set_file_bytes("big.png", &vec![1u8; 300 * 1024])
        .await?;
    page.text(Name::Exact("Image is too large (max 256 KB)."))
        .expect_visible()
        .await?;
    page.css("[data-island=emote-form] input[type=file]")
        .set_file_bytes("x.txt", b"hello")
        .await?;
    page.text(Name::Re("isn.t a PNG")).expect_visible().await?;

    page.label(Name::Exact("Shortcode")).fill(shortcode).await?;
    page.label(Name::Exact("Category")).fill("Blobs").await?;
    page.css("[data-island=emote-form] input[type=file]")
        .set_file_bytes("blob.jpg", &make_image(100, 100, 40)?)
        .await?;
    page.role("button", Name::Exact("Upload emote"))
        .click()
        .await?;
    toast(&page, &format!("Added :{shortcode}:.")).await?;
    let row = page.css(&format!("[data-emote='{shortcode}']"));
    row.expect_visible().await?;

    let field = page.label(Name::Exact(&format!("Category of :{shortcode}:")));
    field.expect_value(Name::Exact("Blobs")).await?;
    field.fill("Moved").await?;
    field.press("Enter").await?;
    toast(&page, "Category saved.").await?;
    let listed = call(&staff, "GET", "/emojis", None).await?;
    let found = listed
        .as_array()
        .and_then(|all| all.iter().find(|e| e["shortcode"] == json!(shortcode)))
        .cloned()
        .unwrap_or_default();
    assert_eq!(found["category"], json!("Moved"));

    row.role("button", Name::Exact(&format!("Delete :{shortcode}:")))
        .click()
        .await?;
    page.role("dialog", Name::Any)
        .role("button", Name::Exact("Delete emote"))
        .click()
        .await?;
    toast(&page, "Emote deleted.").await?;
    row.expect_count(0).await?;
    Ok(())
}

#[tokio::test]
async fn the_content_filter_is_saved_and_scanned() -> Result<()> {
    let (staff, _me) = new_admin("flt").await?;
    let letters: String = handle("w")
        .chars()
        .filter(char::is_ascii_alphabetic)
        .collect();
    let word = format!("zzflt{}", letters.to_lowercase());
    let flagged = Ctx::new().await?;
    let who = handle("flg");
    let signup = json!({"handle": who, "displayName": format!("{word} Person"), "password": tuita_e2e::helpers::PASSWORD, "acceptTerms": true, "add": false});
    let (status, body) = flagged.call("POST", "/auth/signup", Some(&signup)).await?;
    assert!((200..300).contains(&status), "{status} {body}");

    let page = staff.page().await?;
    page.goto("/admin/filter").await?;
    page.label("Blocked words").fill(&word).await?;
    page.role("button", Name::Exact("Save filter"))
        .click()
        .await?;
    toast(&page, "Filter saved").await?;
    let settings = call(&staff, "GET", "/admin/settings", None).await?;
    assert_eq!(settings["filter"]["blocked"], json!(word));

    page.role("button", Name::Exact("Scan now")).click().await?;
    let hit = page.css("[data-c='admin-user-row']").has_text(who.as_str());
    hit.expect_visible().await?;
    hit.text(Name::Contains("contains"))
        .expect_visible()
        .await?;
    flagged.close().await?;
    call(
        &staff,
        "PATCH",
        "/admin/settings",
        Some(&json!({"filter": {"blocked": "", "allowed": "", "reserved": ""}})),
    )
    .await?;
    Ok(())
}

#[tokio::test]
async fn the_audit_log_lists_what_staff_did_and_pages_through_it() -> Result<()> {
    let (staff, _me) = new_admin("aud").await?;
    let (member_ctx, who, _post) = member("aud", "hello").await?;
    for verified in [true, false, true, false] {
        call(
            &staff,
            "PATCH",
            &format!("/admin/users/{who}"),
            Some(&json!({"verified": verified})),
        )
        .await?;
    }
    let page = staff.page().await?;
    page.goto("/admin/audit").await?;
    let rows = page.css("[data-action]").has_text(who.as_str());
    rows.first().expect_visible().await?;
    rows.first()
        .text(Name::Contains("Verification removed"))
        .expect_visible()
        .await?;
    rows.first().css("time").expect_visible().await?;
    member_ctx.close().await?;
    Ok(())
}
