//! Browser tests: federation against a second real instance.
//!
//! Every test starts its own peer (they share one port, so [`PEER_LOCK`] runs them one after the
//! other) and builds the state it needs, instead of leaning on the tests before it. The states
//! that need a remote come from real conditions:
//!
//! - a pending follow request: the remote account is stored as one that approves its followers and
//!   its instance is stopped before the follow, so no Accept ever arrives;
//! - non-public remote posts: the peer's own `posts.visibility` column default is set before bob
//!   posts, so his instance federates an unlisted and a followers-only note with the addressing a
//!   real instance gives them (tuita itself only offers public posts); the direct post, which
//!   tuita cannot send, is stored in the first instance's database as the inbox stores it;
//! - an unreachable instance: a handle at `localhost:<port nothing listens on>`.

use std::time::Duration;

use serde_json::{Value, json};
use tokio::sync::MutexGuard;
use tuita_e2e::{
    Ctx, Name, Page, Poll, Result, env,
    helpers::{
        PEER_LOCK, Peer, call, closed_port, handle, make_image, new_admin, promote_on, psql,
        psql_value, sign_up_via_api, start_peer, upload_emote,
    },
    poll_for,
};

const FIRST_POST: &str = "Greetings from the other instance, before anyone followed.";
const SECOND_POST: &str = "A second note, delivered after the follow.";

/// A peer with one account (`bob`, who has posted `FIRST_POST`) and an admin on the first
/// instance (`ann`), held exclusively until it is stopped or dropped.
struct Fed {
    _lock: MutexGuard<'static, ()>,
    peer: Option<Peer>,
    bob: String,
    bob_ctx: Ctx,
    ann: String,
    ann_ctx: Ctx,
    domain: String,
    base: String,
}

impl Fed {
    async fn start() -> Result<Self> {
        let lock = PEER_LOCK.lock().await;
        let peer = start_peer().await?;
        let bob = handle("bob");
        let bob_ctx = Ctx::with_base(&peer.base).await?;
        sign_up_via_api(&bob_ctx, &bob, "Bob Peer").await?;
        post(&bob_ctx, FIRST_POST).await?;
        let (ann_ctx, ann) = new_admin("ann").await?;
        Ok(Self {
            _lock: lock,
            domain: peer.domain.clone(),
            base: peer.base.clone(),
            peer: Some(peer),
            bob,
            bob_ctx,
            ann,
            ann_ctx,
        })
    }

    /// `bob@localhost:<peer port>`.
    fn acct(&self) -> String {
        format!("{}@{}", self.bob, self.domain)
    }

    /// bob's path in `/users/{acct}`.
    fn user_path(&self) -> String {
        format!("/users/{}", enc(&self.acct()))
    }

    /// Makes the first instance fetch bob (a profile lookup stores the account and his posts).
    async fn lookup(&self) -> Result<Value> {
        call(&self.ann_ctx, "GET", &self.user_path(), None).await
    }

    /// ann follows bob through the API and waits for his instance to accept.
    async fn follow(&self) -> Result<()> {
        self.lookup().await?;
        call(
            &self.ann_ctx,
            "POST",
            &format!("{}/follow", self.user_path()),
            Some(&json!({})),
        )
        .await?;
        let path = self.user_path();
        poll_for(
            "the peer accepts the follow",
            Duration::from_secs(30),
            || {
                let path = path.clone();
                async move {
                    let user = call(&self.ann_ctx, "GET", &path, None).await?;
                    Ok(if user["relation"]["following"] == json!(true) {
                        Poll::Ready(())
                    } else {
                        Poll::Retry(user["relation"].to_string())
                    })
                }
            },
        )
        .await
    }

    async fn stop_peer(&mut self) -> Result<()> {
        match self.peer.take() {
            Some(peer) => peer.stop().await,
            None => Ok(()),
        }
    }

    async fn stop(mut self) -> Result<()> {
        self.stop_peer().await?;
        self.bob_ctx.close().await?;
        self.ann_ctx.close().await
    }
}

/// `encodeURIComponent` for the characters of an account (`user@host:port`).
fn enc(acct: &str) -> String {
    acct.replace('@', "%40").replace(':', "%3A")
}

async fn post(ctx: &Ctx, text: &str) -> Result<()> {
    call(
        ctx,
        "POST",
        "/posts",
        Some(&json!({"text": text, "cw": "", "sensitive": false, "mediaIds": []})),
    )
    .await?;
    Ok(())
}

fn profile(page: &Page) -> tuita_e2e::Loc {
    page.css(".profile")
}

fn article(page: &Page, text: &str) -> tuita_e2e::Loc {
    page.css("article[data-post]").has_text(text)
}

/// Reloads `page` at `path` until `check` finds what it looks for, for at most `secs` seconds.
async fn until<F, Fut>(what: &str, secs: u64, page: &Page, path: &str, check: F) -> Result<()>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<bool>>,
{
    poll_for(what, Duration::from_secs(secs), || async {
        page.goto(path).await?;
        Ok(if check().await? {
            Poll::Ready(())
        } else {
            Poll::Retry("not there yet".to_owned())
        })
    })
    .await
}

/// The Following tab of the home timeline.
async fn following_tab(page: &Page) -> Result<()> {
    page.goto("/").await?;
    page.role("tab", Name::Exact("Following")).click().await
}

#[tokio::test]
async fn a_remote_profile_and_its_post_render_with_their_home_domain() -> Result<()> {
    let fed = Fed::start().await?;
    let page = fed.ann_ctx.page().await?;
    let (domain, base) = (&fed.domain, &fed.base);
    page.goto(&format!("/@{}@{domain}", fed.bob)).await?;
    profile(&page)
        .css("div.name")
        .expect_text(Name::Contains("Bob Peer"))
        .await?;
    profile(&page)
        .css(".line .handle")
        .expect_text(Name::Exact(&format!("@{}@{domain}", fed.bob)))
        .await?;
    profile(&page)
        .css(&format!(r#"[data-domain="{domain}"]"#))
        .expect_visible()
        .await?;
    profile(&page)
        .role("link", Name::Contains(&format!("View profile on {domain}")))
        .expect_attr("href", Name::Exact(&format!("{base}/@{}", fed.bob)))
        .await?;

    let card = article(&page, FIRST_POST);
    card.expect_visible().await?;
    card.css(&format!(r#"[data-domain="{domain}"]"#))
        .expect_visible()
        .await?;
    card.role("link", Name::Contains(&format!("View on {domain}")))
        .expect_attr(
            "href",
            Name::Re(&format!("^{}/@{}/", base.replace('.', r"\."), fed.bob)),
        )
        .await?;

    card.css("a.time").click().await?;
    page.expect_url(Name::Re(&format!(
        r"^http://localhost:\d+/@{}(%40|@){}/",
        fed.bob,
        domain.replace(':', "(:|%3A)")
    )))
    .await?;
    let main = page.css("article[data-post]");
    main.expect_text(Name::Contains(FIRST_POST)).await?;
    main.css(&format!(r#"[data-domain="{domain}"]"#))
        .expect_visible()
        .await?;
    fed.stop().await
}

#[tokio::test]
async fn following_a_remote_account_is_requested_then_accepted_by_its_instance() -> Result<()> {
    let fed = Fed::start().await?;
    let page = fed.ann_ctx.page().await?;
    let path = format!("/@{}@{}", fed.bob, fed.domain);
    page.goto(&path).await?;
    profile(&page)
        .role("button", Name::Exact("Follow"))
        .click()
        .await?;
    profile(&page)
        .role("button", Name::Re("^(Requested|Following)$"))
        .expect_visible()
        .await?;
    let following = profile(&page).role("button", Name::Exact("Following"));
    until("the follow is accepted", 30, &page, &path, || {
        following.is_visible()
    })
    .await?;
    fed.stop().await
}

#[tokio::test]
async fn a_post_of_the_remote_account_reaches_the_following_timeline_with_its_domain() -> Result<()>
{
    let fed = Fed::start().await?;
    fed.follow().await?;
    post(&fed.bob_ctx, SECOND_POST).await?;
    let page = fed.ann_ctx.page().await?;
    let second = article(&page, SECOND_POST);
    until("the post is on the timeline", 40, &page, "/", || async {
        following_tab(&page).await?;
        second.is_visible().await
    })
    .await?;
    second
        .css(&format!(r#"[data-domain="{}"]"#, fed.domain))
        .expect_visible()
        .await?;
    fed.stop().await
}

#[tokio::test]
async fn a_custom_emote_of_the_remote_instance_renders_on_the_following_timeline() -> Result<()> {
    let fed = Fed::start().await?;
    fed.follow().await?;
    // Bob becomes staff on his own instance to upload an emote there.
    let peer = fed.peer.as_ref().expect("the peer runs");
    promote_on(&peer.database, &fed.bob).await?;
    let shortcode = format!("peer{}", &handle("e")[2..]);
    upload_emote(&fed.bob_ctx, &shortcode, &make_image(120, 120, 80)?).await?;

    post(
        &fed.bob_ctx,
        &format!("An emote across instances :{shortcode}:"),
    )
    .await?;
    let page = fed.ann_ctx.page().await?;
    let text = article(&page, "An emote across instances");
    until(
        "the emote post is on the timeline",
        40,
        &page,
        "/",
        || async {
            following_tab(&page).await?;
            text.is_visible().await
        },
    )
    .await?;
    let emote = text.css(&format!(r#"img.emote[alt=":{shortcode}:"]"#));
    emote.expect_visible().await?;
    emote.expect_attr("src", Name::Re("^/media/")).await?;
    fed.stop().await
}

#[tokio::test]
async fn followers_following_and_search_list_the_remote_account_with_its_domain() -> Result<()> {
    let fed = Fed::start().await?;
    fed.follow().await?;
    let page = fed.ann_ctx.page().await?;
    let domain = &fed.domain;
    let bob = format!("@{}@{domain}", fed.bob);

    page.goto(&format!("/@{}/following", fed.ann)).await?;
    page.css(".row")
        .has_text(bob.as_str())
        .css(&format!(r#"[data-domain="{domain}"]"#))
        .expect_visible()
        .await?;

    page.goto(&format!("/@{}@{domain}/followers", fed.bob))
        .await?;
    page.css("#main .row")
        .has_text(format!("@{}", fed.ann).as_str())
        .expect_visible()
        .await?;

    page.goto(&format!("/explore?q={}&type=people", enc(&bob)))
        .await?;
    page.css(".row")
        .has_text(bob.as_str())
        .css(&format!(r#"[data-domain="{domain}"]"#))
        .expect_visible()
        .await?;
    fed.stop().await
}

#[tokio::test]
async fn a_message_to_a_remote_handle_and_a_report_about_it_use_the_user_at_domain_handle()
-> Result<()> {
    let fed = Fed::start().await?;
    let page = fed.ann_ctx.page().await?;
    let acct = fed.acct();
    page.goto(&format!("/@{acct}")).await?;
    profile(&page)
        .role("button", Name::Exact("Message"))
        .click()
        .await?;
    page.expect_url(Name::Re(&format!(
        r"/messages/{}(%40|@){}$",
        fed.bob,
        fed.domain.replace(':', "(:|%3A)")
    )))
    .await?;
    page.label(Name::Exact("Message"))
        .fill("hello across instances")
        .await?;
    page.role("button", Name::Exact("Send")).click().await?;
    page.role("list", Name::Re("^Messages with"))
        .text("hello across instances")
        .expect_visible()
        .await?;
    poll_for(
        "the peer holds the message",
        Duration::from_secs(30),
        || async {
            let conversations = call(&fed.bob_ctx, "GET", "/conversations", None).await?;
            Ok(
                if conversations.to_string().contains("hello across instances") {
                    Poll::Ready(())
                } else {
                    Poll::Retry(conversations.to_string())
                },
            )
        },
    )
    .await?;

    page.goto(&format!("/@{acct}")).await?;
    profile(&page)
        .role("button", Name::Exact("More"))
        .click()
        .await?;
    page.role("menuitem", Name::Contains(&format!("Report @{acct}")))
        .click()
        .await?;
    let dialog = page.role("dialog", Name::Contains(&format!("Report @{acct}")));
    dialog.text(Name::Exact("Spam")).click().await?;
    dialog
        .role("button", Name::Exact("Send report"))
        .click()
        .await?;
    page.text("Thanks. Your report was sent to the moderators.")
        .expect_visible()
        .await?;
    fed.stop().await
}

#[tokio::test]
async fn an_unknown_account_and_an_unreachable_instance_say_so_with_a_retry() -> Result<()> {
    let fed = Fed::start().await?;
    let page = fed.ann_ctx.page().await?;
    page.goto(&format!("/@nobody_here@{}", fed.domain)).await?;
    page.role("heading", Name::Exact("No such account"))
        .expect_visible()
        .await?;

    let dead = format!("localhost:{}", closed_port().await?);
    page.goto(&format!("/@ghost@{dead}")).await?;
    let alert = page
        .role("alert", Name::Any)
        .has_text(format!("Couldn’t load @ghost@{dead}").as_str());
    alert.expect_visible().await?;
    alert
        .expect_text(Name::Contains(&format!("Couldn’t reach {dead} right now.")))
        .await?;
    alert
        .role("button", Name::Exact("Try again"))
        .expect_visible()
        .await?;
    fed.stop().await
}

#[tokio::test]
async fn staff_block_soften_and_unblock_an_instance() -> Result<()> {
    let fed = Fed::start().await?;
    let page = fed.ann_ctx.page().await?;
    let domain = format!(
        "blocked-{}.example",
        handle("d").to_lowercase().replace('_', "-")
    );
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
    page.text(format!("{domain} is now set to Suspend").as_str())
        .expect_visible()
        .await?;
    row.expect_attr("data-severity", Name::Exact("suspend"))
        .await?;
    let federation = call(&fed.ann_ctx, "GET", "/admin/federation", None).await?;
    let severity = federation["blocks"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|b| b["domain"] == json!(domain))
        .map(|b| b["severity"].clone());
    assert_eq!(severity, Some(json!("suspend")), "{federation}");

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
    page.text(format!("{domain} is no longer blocked").as_str())
        .expect_visible()
        .await?;
    fed.stop().await
}

#[tokio::test]
async fn the_queue_shows_a_failing_instance_and_retry_now_requeues_it() -> Result<()> {
    let mut fed = Fed::start().await?;
    let page = fed.ann_ctx.page().await?;
    fed.lookup().await?;
    let stored = call(
        &fed.ann_ctx,
        "GET",
        &format!("{}/posts", fed.user_path()),
        None,
    )
    .await?;
    let id = stored["items"][0]["id"]
        .as_str()
        .expect("the lookup stored bob's post")
        .to_owned();
    fed.stop_peer().await?;
    call(&fed.ann_ctx, "POST", &format!("/posts/{id}/like"), None).await?;

    let failing = page.css(&format!(r#"[data-failing="{}"]"#, fed.domain));
    until(
        "the queue lists the instance",
        60,
        &page,
        "/admin/federation",
        || failing.is_visible(),
    )
    .await?;
    failing
        .css(".error code")
        .expect_text(Name::Re("."))
        .await?;
    page.css("[data-queue-count]").expect_visible().await?;

    page.role("button", Name::Exact("Retry now"))
        .click()
        .await?;
    page.text(Name::Re(r"deliver(y|ies) will be retried now\."))
        .expect_visible()
        .await?;

    let federation = call(&fed.ann_ctx, "GET", "/admin/federation", None).await?;
    for key in ["knownDomains", "remoteAccounts", "remotePosts"] {
        let n = federation[key].as_u64().unwrap_or_default();
        assert!(n >= 1, "{key} is {n}: {federation}");
    }
    fed.stop().await
}

#[tokio::test]
async fn a_pending_follow_request_is_cancelled_with_a_click() -> Result<()> {
    let mut fed = Fed::start().await?;
    // The first instance knows bob as an account that approves its followers, and his instance
    // goes away before the follow: the request stays pending, as it does for a locked account
    // that never answers.
    fed.lookup().await?;
    psql(
        &env().database_url.clone().unwrap_or_default(),
        &format!(
            "update users set manually_approves = true where lower(handle) = lower('{}') and domain = '{}'",
            fed.bob, fed.domain
        ),
    )
    .await?;
    fed.stop_peer().await?;

    let page = fed.ann_ctx.page().await?;
    page.goto(&format!("/@{}", fed.acct())).await?;
    profile(&page)
        .text("Approves followers")
        .expect_visible()
        .await?;
    profile(&page)
        .role("button", Name::Exact("Follow"))
        .click()
        .await?;
    let requested = profile(&page).role("button", Name::Exact("Requested"));
    requested.expect_visible().await?;
    // The button flips before the request lands; the request is stored as pending.
    poll_for(
        "the follow request is pending",
        Duration::from_secs(10),
        || async {
            let user = call(&fed.ann_ctx, "GET", &fed.user_path(), None).await?;
            Ok(if user["relation"]["followPending"] == json!(true) {
                Poll::Ready(())
            } else {
                Poll::Retry(user["relation"].to_string())
            })
        },
    )
    .await?;

    requested.click().await?;
    profile(&page)
        .role("button", Name::Exact("Follow"))
        .expect_visible()
        .await?;
    page.text("Follow request cancelled")
        .expect_visible()
        .await?;
    let after = call(&fed.ann_ctx, "GET", &fed.user_path(), None).await?;
    assert_eq!(after["relation"]["followPending"], json!(false), "{after}");
    assert_eq!(after["relation"]["following"], json!(false), "{after}");
    fed.stop().await
}

#[tokio::test]
async fn non_public_remote_posts_carry_a_visibility_badge() -> Result<()> {
    let fed = Fed::start().await?;
    fed.follow().await?;
    let peer = fed.peer.as_ref().expect("the peer runs");
    // tuita only offers public posts, so bob's instance gets a different default visibility
    // before each note: its own federation code then addresses them as an unlisted and a
    // followers-only note.
    for (visibility, text) in [
        ("unlisted", "unlisted note"),
        ("followers", "followers note"),
    ] {
        psql(
            &peer.database,
            &format!("alter table posts alter column visibility set default '{visibility}'"),
        )
        .await?;
        post(&fed.bob_ctx, text).await?;
    }
    // A direct note to ann alone arrives as a message and tuita cannot mention remote accounts, so
    // the third note is stored the way the inbox stores a direct post that names ann. No timeline
    // lists a direct post: ann opens it by its address.
    let main = env().database_url.clone().unwrap_or_default();
    let direct_id = psql_value(
        &main,
        &format!(
            "with bob as (select id, ap_id from users where lower(handle) = lower('{bob}') and domain = '{domain}'),
                  ann as (select id from users where lower(handle) = lower('{ann}') and domain is null),
                  made as (
                      insert into posts (id, author_id, text, visibility, ap_id)
                      select gen_random_uuid(), bob.id, 'direct note', 'direct', bob.ap_id || '/direct-note' from bob
                      returning id),
                  mention as (
                      insert into mentions (post_id, user_id) select made.id, ann.id from made, ann)
             select id from made",
            bob = fed.bob,
            domain = fed.domain,
            ann = fed.ann,
        ),
    )
    .await?;

    let page = fed.ann_ctx.page().await?;
    let path = format!("/@{}", fed.acct());
    let unlisted = article(&page, "unlisted note");
    let followers = article(&page, "followers note");
    until("both notes arrive", 40, &page, &path, || async {
        Ok(unlisted.is_visible().await? && followers.is_visible().await?)
    })
    .await?;
    unlisted
        .css(r#"[data-visibility="unlisted"]"#)
        .expect_text(Name::Exact("Unlisted"))
        .await?;
    followers
        .css(r#"[data-visibility="followers"]"#)
        .expect_text(Name::Exact("Followers only"))
        .await?;
    article(&page, "direct note").expect_count(0).await?;

    page.goto(&format!("/@{}/{direct_id}", fed.acct())).await?;
    article(&page, "direct note")
        .css(r#"[data-visibility="direct"]"#)
        .expect_text(Name::Exact("Direct"))
        .await?;
    fed.stop().await
}
