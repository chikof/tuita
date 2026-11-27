//! Browser tests: staff flows that cross areas (reports, announcements, invites, tokens,
//! affiliations, devices, moderator access).
//!
//! Instance-wide switches (sign-up mode, a published announcement) run on a second real instance,
//! not on the instance the rest of the suite signs up on at the same time, so this file
//! runs last.

use serde_json::json;
use tokio::sync::MutexGuard;
use tuita_e2e::{
    Ctx, Name, Result,
    helpers::{
        PASSWORD, PEER_LOCK, Peer, call, handle, promote_on, sign_up_via_api, sign_up_via_ui,
        start_peer, submit_signup,
    },
};

/// A second instance with one staff account, held exclusively until dropped.
struct Instance {
    _lock: MutexGuard<'static, ()>,
    peer: Peer,
    staff: Ctx,
}

impl Instance {
    async fn start(prefix: &str) -> Result<Self> {
        let lock = PEER_LOCK.lock().await;
        let peer = start_peer().await?;
        let staff = Ctx::with_base(&peer.base).await?;
        let who = handle(prefix);
        sign_up_via_api(&staff, &who, "Admin Person").await?;
        promote_on(&peer.database, &who).await?;
        Ok(Self {
            _lock: lock,
            peer,
            staff,
        })
    }

    async fn stop(self) -> Result<()> {
        self.staff.close().await?;
        self.peer.stop().await
    }
}

#[tokio::test]
async fn staff_resolve_a_report_with_delete_and_suspend_post_gone_account_suspended_two_audit_rows()
-> Result<()> {
    let (staff, _me) = tuita_e2e::helpers::new_admin("adm").await?;
    let bad = handle("bad");
    let reporter = handle("rep");

    let bad_ctx = Ctx::new().await?;
    sign_up_via_api(&bad_ctx, &bad, "Bad Actor").await?;
    let post = call(
        &bad_ctx,
        "POST",
        "/posts",
        Some(&json!({
            "text": format!("buy followers now {bad}"),
            "cw": "",
            "sensitive": false,
            "mediaIds": [],
        })),
    )
    .await?;

    let rep_ctx = Ctx::new().await?;
    sign_up_via_api(&rep_ctx, &reporter, "Reporter").await?;
    call(
        &rep_ctx,
        "POST",
        "/reports",
        Some(&json!({"postId": post["id"], "reason": "spam", "note": "an ad"})),
    )
    .await?;

    let page = staff.page().await?;
    page.goto("/admin/reports").await?;
    let card = page
        .css("[data-report]")
        .has_text(format!("buy followers now {bad}").as_str());
    card.expect_visible().await?;
    card.role("button", Name::Exact("Delete and suspend"))
        .click()
        .await?;
    page.role("dialog", Name::Any)
        .role("button", Name::Exact("Confirm"))
        .click()
        .await?;
    card.expect_count(0).await?;

    let audit = call(&staff, "GET", "/admin/audit?limit=40", None).await?;
    let target = format!("@{bad}");
    let mut actions: Vec<&str> = audit["items"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|row| row["target"] == json!(target))
        .filter_map(|row| row["action"].as_str())
        .collect();
    actions.sort_unstable();
    assert_eq!(actions, ["post.delete", "user.suspend"]);

    let detail = call(&staff, "GET", &format!("/admin/users/{bad}"), None).await?;
    assert_eq!(detail["user"]["user"]["suspended"], json!(true));
    bad_ctx.close().await?;
    rep_ctx.close().await?;
    Ok(())
}

#[tokio::test]
async fn an_announcement_shows_is_dismissed_and_stays_dismissed_after_a_reload() -> Result<()> {
    let instance = Instance::start("ann").await?;
    let page = instance.staff.page().await?;
    let text = format!("Maintenance window {}", handle("ann"));
    page.goto("/admin/announcements").await?;
    page.role("textbox", Name::Exact("Message"))
        .fill(&text)
        .await?;
    page.role("button", Name::Exact("Publish")).click().await?;
    let banner = page.role("region", Name::Exact("Announcements"));
    banner.text(Name::Contains(&text)).expect_visible().await?;

    banner
        .role("button", Name::Exact("Dismiss announcement"))
        .click()
        .await?;
    banner.expect_count(0).await?;

    page.goto("/").await?;
    page.role("navigation", Name::Exact("Main"))
        .first()
        .expect_visible()
        .await?;
    page.role("region", Name::Exact("Announcements"))
        .expect_count(0)
        .await?;
    instance.stop().await
}

#[tokio::test]
async fn invite_only_sign_up_takes_a_code_once_when_max_uses_is_1() -> Result<()> {
    let instance = Instance::start("inv").await?;
    let page = instance.staff.page().await?;
    page.goto("/admin/invites").await?;
    page.label("Uses").expect_visible().await?;
    let codes = page.css("[data-invite]");
    let before = codes.count().await?;
    page.label("Uses").select("1").await?;
    page.role("button", Name::Exact("Create code"))
        .click()
        .await?;
    codes.expect_count(before + 1).await?;
    let code = codes.first().css("code").inner_text().await?;
    let code = code.trim().to_owned();
    let shape = code.len() == 11
        && code.char_indices().all(|(at, c)| {
            if at == 5 {
                c == '-'
            } else {
                c.is_ascii_uppercase() || c.is_ascii_digit()
            }
        });
    assert!(shape, "invite code shape: {code}");

    call(
        &instance.staff,
        "PATCH",
        "/admin/settings",
        Some(&json!({"signups": "invite"})),
    )
    .await?;

    let first = Ctx::with_base(&instance.peer.base).await?;
    let p1 = first.page().await?;
    p1.goto("/signup").await?;
    p1.label("Invite code").expect_visible().await?;
    p1.label("Invite code").fill(&code).await?;
    p1.label("Display name").fill("Invited One").await?;
    p1.label("Handle").fill(&handle("inv")).await?;
    p1.label("Password").fill(PASSWORD).await?;
    p1.role("checkbox", Name::Re("I.m at least"))
        .check()
        .await?;
    submit_signup(&p1, &p1.role("button", "Create account")).await?;
    first.close().await?;

    let second = Ctx::with_base(&instance.peer.base).await?;
    let p2 = second.page().await?;
    p2.goto(&format!("/signup?invite={code}")).await?;
    p2.label("Display name").fill("Invited Two").await?;
    p2.label("Handle").fill(&handle("inv")).await?;
    p2.label("Password").fill(PASSWORD).await?;
    p2.role("checkbox", Name::Re("I.m at least"))
        .check()
        .await?;
    p2.role("button", "Create account").click().await?;
    p2.role("alert", Name::Any)
        .expect_text(Name::Contains("invalid or has been used up"))
        .await?;
    second.close().await?;
    instance.stop().await
}

#[tokio::test]
async fn a_new_token_shows_its_secret_once_and_works_as_a_bearer() -> Result<()> {
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    sign_up_via_ui(&page, "Token Person", &handle("tok")).await?;
    page.goto("/settings/bots").await?;
    page.label("Token name").fill("weather bot").await?;
    page.role("button", "Create token").click().await?;

    let dialog = page.role("dialog", Name::Any);
    dialog
        .text("it won’t be shown again")
        .expect_visible()
        .await?;
    let secret = dialog.label("New API token").value().await?;
    let body = secret.strip_prefix("tuita_").unwrap_or_default();
    assert!(
        body.len() == 43
            && body
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
        "token shape: {secret}"
    );
    dialog.role("button", "Done").click().await?;
    page.css("[data-token]")
        .expect_text(Name::Contains("weather bot"))
        .await?;

    page.reload().await?;
    page.css("[data-token]")
        .expect_text(Name::Contains(&secret[..12]))
        .await?;
    page.text(Name::Contains(&secret)).expect_count(0).await?;
    page.css(&format!("input[value=\"{secret}\"]"))
        .expect_count(0)
        .await?;

    let status = |path: &str| {
        let script = format!(
            "fetch('/api/v1{path}', {{ credentials: 'omit', headers: {{ authorization: 'Bearer ' + {} }} }}).then((r) => r.status)",
            json!(secret)
        );
        let page = page.clone();
        async move { page.eval(&script).await }
    };
    assert_eq!(status("/me").await?, json!(200), "a token reads /me");
    assert_eq!(
        status("/tokens").await?,
        json!(403),
        "a token may not manage tokens"
    );
    Ok(())
}

#[tokio::test]
async fn an_organization_invites_the_member_accepts_either_side_removes() -> Result<()> {
    let (staff, _me) = tuita_e2e::helpers::new_admin("orga").await?;
    let org_handle = handle("org");
    let member_handle = handle("mem");

    let org_ctx = Ctx::new().await?;
    sign_up_via_api(&org_ctx, &org_handle, "Acme Org").await?;
    call(
        &staff,
        "PATCH",
        &format!("/admin/users/{org_handle}/org"),
        Some(&json!({"org": true})),
    )
    .await?;
    let member_ctx = Ctx::new().await?;
    sign_up_via_api(&member_ctx, &member_handle, "Member Person").await?;

    let org = org_ctx.page().await?;
    org.goto("/settings/affiliation").await?;
    org.label("Handle").fill(&member_handle).await?;
    org.label("Label").fill("Engineer").await?;
    org.role("button", "Send invitation").click().await?;
    org.css("[data-affiliation]")
        .expect_text(Name::Contains(&member_handle))
        .await?;
    org.text(Name::Exact("Invited")).expect_visible().await?;

    let member = member_ctx.page().await?;
    member.goto("/settings/affiliation").await?;
    member.role("button", "Accept").click().await?;
    // The invitation row names the organization too: the accept has landed once its button is gone.
    member.role("button", "Accept").expect_count(0).await?;
    member
        .css("[data-affiliation]")
        .expect_text(Name::Contains(&org_handle))
        .await?;

    org.reload().await?;
    org.text(Name::Exact("Invited")).expect_count(0).await?;
    org.role("button", Name::Re(&format!("^Remove @{}", member_handle)))
        .click()
        .await?;
    org.role("dialog", Name::Any)
        .role("button", "Remove")
        .click()
        .await?;
    org.text("No affiliates yet.").expect_visible().await?;
    Ok(())
}

#[tokio::test]
async fn the_device_list_marks_this_device_and_signs_out_the_others() -> Result<()> {
    let who = handle("dev");
    let first = Ctx::new().await?;
    sign_up_via_api(&first, &who, "Device Person").await?;
    let second = Ctx::new().await?;
    call(
        &second,
        "POST",
        "/auth/login",
        Some(&json!({"handle": who, "password": PASSWORD, "add": false})),
    )
    .await?;

    let page = first.page().await?;
    page.goto("/settings/devices").await?;
    page.text("This device").expect_count(1).await?;
    page.css("li")
        .has_text("Last active")
        .expect_count(2)
        .await?;
    page.role("button", "Sign out of all other devices")
        .click()
        .await?;
    page.role("dialog", Name::Any)
        .role("button", "Sign out others")
        .click()
        .await?;
    page.css("li")
        .has_text("Last active")
        .expect_count(1)
        .await?;
    Ok(())
}

#[tokio::test]
async fn a_moderator_sees_the_moderation_tabs_only_and_is_refused_the_rest() -> Result<()> {
    let (staff, _me) = tuita_e2e::helpers::new_admin("modr").await?;
    let mod_handle = handle("mod");
    let mod_ctx = Ctx::new().await?;
    sign_up_via_api(&mod_ctx, &mod_handle, "Mod Person").await?;
    call(
        &staff,
        "PATCH",
        &format!("/admin/users/{mod_handle}"),
        Some(&json!({"role": "moderator"})),
    )
    .await?;

    let page = mod_ctx.page().await?;
    page.goto("/admin").await?;
    let tabs = page.role("navigation", Name::Exact("Admin sections"));
    tabs.role("link", Name::Exact("Reports"))
        .expect_visible()
        .await?;
    tabs.role("link", Name::Exact("Instance"))
        .expect_count(0)
        .await?;
    page.goto("/admin/instance").await?;
    page.text("Staff only").expect_visible().await?;
    Ok(())
}
