//! Browser tests: end-to-end encrypted messages.

use std::time::Duration;

use serde_json::Value;
use tokio::process::Command;
use tuita_e2e::{
    Ctx, Name, Page, Poll, Result, env,
    helpers::{PASSPHRASE, PASSWORD, handle, say, sign_up_via_api, thread, turn_on},
    poll_for,
};

const NEW_PASSPHRASE: &str = "a different long passphrase";

struct Account {
    who: String,
    ctx: Ctx,
    page: Page,
}

async fn account(prefix: &str) -> Result<Account> {
    let who = handle(prefix);
    let ctx = Ctx::new().await?;
    sign_up_via_api(&ctx, &who, &format!("Name {who}")).await?;
    let page = ctx.page().await?;
    Ok(Account { who, ctx, page })
}

/// What the server serves for the conversation with `peer`: the check that it holds no text.
async fn served(ctx: &Ctx, peer: &str) -> Result<Value> {
    let (status, view) = ctx
        .call("GET", &format!("/conversations/{peer}"), None)
        .await?;
    assert_eq!(status, 200, "{view}");
    Ok(view)
}

/// Plaintext and ciphertext of every message the two accounts exchanged, straight from the
/// database, as `text|ciphertext` lines.
async fn stored_text(a: &str, b: &str) -> Result<Vec<String>> {
    let database = env()
        .database_url
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("E2E_DATABASE_URL comes from scripts/e2e.sh"))?;
    let query = format!(
        "select m.text || '|' || coalesce(m.ciphertext, '') from messages m join users s on s.id = m.sender_id where s.handle in ('{a}', '{b}') order by m.id"
    );
    let out = Command::new("psql")
        .args([database, "-tA", "-c", &query])
        .output()
        .await?;
    assert!(
        out.status.success(),
        "psql: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect())
}

#[tokio::test]
async fn two_accounts_exchange_sealed_messages_and_a_new_device_unlocks_the_history() -> Result<()>
{
    let (ann, bo) = (account("ea").await?, account("eb").await?);

    ann.page.goto(&format!("/messages/{}", bo.who)).await?;
    ann.page
        .text(format!("@{} hasn’t turned on encrypted messages yet.", bo.who).as_str())
        .expect_visible()
        .await?;
    ann.page
        .label(Name::Exact("Message"))
        .expect_disabled()
        .await?;

    turn_on(&ann.page, PASSPHRASE).await?;
    turn_on(&bo.page, PASSPHRASE).await?;

    ann.page.goto(&format!("/messages/{}", bo.who)).await?;
    ann.page
        .text(Name::Exact("End-to-end encrypted"))
        .expect_visible()
        .await?;
    say(&ann.page, "meet at noon, sealed").await?;

    bo.page.goto(&format!("/messages/{}", ann.who)).await?;
    thread(&bo.page)
        .text("meet at noon, sealed")
        .expect_visible()
        .await?;
    say(&bo.page, "noon is fine").await?;
    thread(&ann.page)
        .text("noon is fine")
        .expect_visible()
        .await?;

    let view = served(&ann.ctx, &bo.who).await?;
    assert_eq!(view["mode"], "sealed");
    let messages = view["messages"].as_array().expect("messages");
    assert_eq!(messages.len(), 2);
    for message in messages {
        assert_eq!(message["text"], "");
        assert!(
            message["ciphertext"]
                .as_str()
                .is_some_and(|text| text.starts_with("v1.")),
            "{message}"
        );
    }
    for row in stored_text(&ann.who, &bo.who).await? {
        assert!(row.starts_with("|v1."), "{row}");
        assert!(!row.contains("noon"), "{row}");
    }

    ann.page.reload().await?;
    thread(&ann.page)
        .text("meet at noon, sealed")
        .expect_visible()
        .await?;

    // Same account on a device that has no key: locked until the passphrase is given.
    let second = ann.ctx.fork().await?;
    let phone = second.page().await?;
    phone.goto(&format!("/messages/{}", bo.who)).await?;
    phone
        .role("heading", Name::Exact("Unlock encrypted messages"))
        .expect_visible()
        .await?;
    phone.text("meet at noon, sealed").expect_count(0).await?;
    phone
        .text(Name::Exact("Encrypted message"))
        .first()
        .expect_visible()
        .await?;

    phone
        .label(Name::Exact("Passphrase"))
        .fill("not the passphrase")
        .await?;
    phone.role("button", Name::Exact("Unlock")).click().await?;
    phone
        .text("That passphrase is wrong.")
        .expect_visible()
        .await?;

    phone
        .label(Name::Exact("Passphrase"))
        .fill(PASSPHRASE)
        .await?;
    phone.role("button", Name::Exact("Unlock")).click().await?;
    thread(&phone)
        .text("meet at noon, sealed")
        .expect_visible()
        .await?;
    thread(&phone).text("noon is fine").expect_visible().await?;

    for page in [&ann.page, &bo.page, &phone] {
        let errors = page.console_errors().await;
        assert!(errors.is_empty(), "console errors: {errors:#?}");
    }
    Ok(())
}

#[tokio::test]
async fn a_contact_who_resets_encryption_blocks_sending_until_the_new_key_is_trusted() -> Result<()>
{
    let (ann, bo) = (account("ec").await?, account("ed").await?);
    turn_on(&ann.page, PASSPHRASE).await?;
    turn_on(&bo.page, PASSPHRASE).await?;

    ann.page.goto(&format!("/messages/{}", bo.who)).await?;
    say(&ann.page, "before the reset").await?;
    bo.page.goto(&format!("/messages/{}", ann.who)).await?;
    thread(&bo.page)
        .text("before the reset")
        .expect_visible()
        .await?;
    say(&bo.page, "from the old key").await?;
    let before = served(&ann.ctx, &bo.who).await?;

    bo.page.goto("/settings/encryption").await?;
    bo.page
        .role("button", Name::Contains("Reset encryption"))
        .click()
        .await?;
    let dialog = bo.page.role("dialog", Name::Exact("Reset encryption?"));
    dialog
        .label(Name::Exact("Passphrase"))
        .fill(NEW_PASSPHRASE)
        .await?;
    dialog
        .label("Confirm passphrase")
        .fill(NEW_PASSPHRASE)
        .await?;
    dialog.role("button", Name::Exact("Reset")).click().await?;
    bo.page
        .text("Encryption was reset")
        .expect_visible()
        .await?;
    poll_for(
        "the served key changes",
        Duration::from_secs(10),
        || async {
            let now = served(&ann.ctx, &bo.who).await?;
            Ok(if now["peerKey"] == before["peerKey"] {
                Poll::Retry("same key".to_owned())
            } else {
                Poll::Ready(())
            })
        },
    )
    .await?;

    ann.page.reload().await?;
    ann.page
        .text(format!("Name {}’s encryption key changed", bo.who).as_str())
        .expect_visible()
        .await?;
    ann.page
        .text("Safety number you trusted")
        .expect_visible()
        .await?;
    ann.page
        .text(Name::Exact("New safety number"))
        .expect_visible()
        .await?;
    ann.page
        .label(Name::Exact("Message"))
        .expect_disabled()
        .await?;
    thread(&ann.page)
        .text("before the reset")
        .expect_visible()
        .await?;

    ann.page
        .role("button", Name::Contains("Trust new key"))
        .click()
        .await?;
    ann.page
        .label(Name::Exact("Message"))
        .expect_enabled()
        .await?;
    thread(&ann.page)
        .text("Sent with a key you haven’t trusted")
        .expect_count(0)
        .await?;
    thread(&ann.page)
        .text("before the reset")
        .expect_visible()
        .await?;
    thread(&ann.page)
        .text("from the old key")
        .expect_visible()
        .await?;
    say(&ann.page, "after the reset").await?;

    bo.page.goto(&format!("/messages/{}", ann.who)).await?;
    thread(&bo.page)
        .text("after the reset")
        .expect_visible()
        .await?;
    say(&bo.page, "from the new key").await?;
    ann.page.reload().await?;
    thread(&ann.page)
        .text("from the new key")
        .expect_visible()
        .await?;
    thread(&ann.page)
        .text("from the old key")
        .expect_visible()
        .await?;
    thread(&ann.page)
        .text("Sent with a key you haven’t trusted")
        .expect_count(0)
        .await
}

#[tokio::test]
async fn a_toast_for_a_sealed_message_names_the_sender_and_carries_no_text() -> Result<()> {
    let (ann, bo) = (account("ee").await?, account("ef").await?);
    turn_on(&ann.page, PASSPHRASE).await?;
    turn_on(&bo.page, PASSPHRASE).await?;

    ann.page.goto("/").await?;
    bo.page.goto(&format!("/messages/{}", ann.who)).await?;
    say(&bo.page, "the secret in the toast").await?;

    ann.page
        .text(format!("@{} sent you a message", bo.who).as_str())
        .expect_visible()
        .await?;
    ann.page
        .text("the secret in the toast")
        .expect_count(0)
        .await?;

    ann.page.role("button", Name::Exact("View")).click().await?;
    thread(&ann.page)
        .text("the secret in the toast")
        .expect_visible()
        .await
}

#[tokio::test]
async fn logging_out_removes_the_key_from_the_device_and_the_next_sign_in_asks_to_unlock()
-> Result<()> {
    let (ann, bo) = (account("eg").await?, account("eh").await?);
    turn_on(&ann.page, PASSPHRASE).await?;
    turn_on(&bo.page, PASSPHRASE).await?;
    ann.page.goto(&format!("/messages/{}", bo.who)).await?;
    say(&ann.page, "kept only on the server, sealed").await?;

    ann.page
        .css("[data-c='account-menu'] .trigger")
        .click()
        .await?;
    ann.page
        .role("button", Name::Exact(&format!("Log out @{}", ann.who)))
        .click()
        .await?;
    ann.page
        .role("button", Name::Exact("Sign in"))
        .expect_visible()
        .await?;

    ann.page.goto("/login").await?;
    ann.page.label("Handle").fill(&ann.who).await?;
    ann.page.label("Password").fill(PASSWORD).await?;
    ann.page
        .role("button", Name::Exact("Sign in"))
        .click()
        .await?;
    ann.page
        .role("navigation", Name::Exact("Main"))
        .first()
        .expect_visible()
        .await?;

    ann.page.goto(&format!("/messages/{}", bo.who)).await?;
    ann.page
        .role("heading", Name::Exact("Unlock encrypted messages"))
        .expect_visible()
        .await?;
    ann.page
        .text("kept only on the server, sealed")
        .expect_count(0)
        .await?;

    ann.page
        .label(Name::Exact("Passphrase"))
        .fill(PASSPHRASE)
        .await?;
    ann.page
        .role("button", Name::Exact("Unlock"))
        .click()
        .await?;
    thread(&ann.page)
        .text("kept only on the server, sealed")
        .expect_visible()
        .await
}
