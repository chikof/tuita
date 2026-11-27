//! Browser tests: direct messages (the read receipt and the pages).

use std::time::Duration;

use tuita_e2e::{
    Ctx, Name, Page, Result, Violation,
    helpers::{PASSPHRASE, handle, say, sign_up_via_ui, thread, turn_on},
};

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
async fn two_people_exchange_a_direct_message_and_the_read_receipt_arrives() -> Result<()> {
    let (a, b) = (handle("ann"), handle("ben"));
    let (ca, cb) = (Ctx::new().await?, Ctx::new().await?);
    let (pa, pb) = (ca.page().await?, cb.page().await?);
    sign_up_via_ui(&pa, "Ann", &a).await?;
    sign_up_via_ui(&pb, "Ben", &b).await?;

    turn_on(&pa, PASSPHRASE).await?;
    turn_on(&pb, PASSPHRASE).await?;

    pa.goto(&format!("/messages/{b}")).await?;
    say(&pa, "hello Ben").await?;

    pb.goto("/messages").await?;
    // Ben has not opened the conversation, so nothing is pinned and the preview stays sealed.
    let row = pb.role("link", Name::Re(&format!("@{a}"))).first();
    row.expect_visible().await?;
    row.click().await?;
    thread(&pb).text("hello Ben").expect_visible().await?;
    // The sender learns of a read by polling: every 15 seconds.
    pa.set_timeout(Duration::from_secs(25));
    pa.text(Name::Exact("Seen")).expect_visible().await
}

#[tokio::test]
async fn the_list_and_a_thread_are_accessible_and_a_conversation_opens_from_the_list() -> Result<()>
{
    let (a, b) = (handle("lia"), handle("lib"));
    let (ca, cb) = (Ctx::new().await?, Ctx::new().await?);
    let (pa, pb) = (ca.page().await?, cb.page().await?);
    sign_up_via_ui(&pa, "Lia", &a).await?;
    sign_up_via_ui(&pb, "Lib", &b).await?;
    turn_on(&pa, PASSPHRASE).await?;
    turn_on(&pb, PASSPHRASE).await?;

    pa.goto("/messages").await?;
    pa.role("heading", Name::Exact("Messages"))
        .first()
        .expect_visible()
        .await?;
    pa.role("heading", Name::Exact("Start conversation"))
        .expect_visible()
        .await?;
    clean(&pa, "empty /messages").await?;

    // Starting a conversation from the dialog.
    pa.role("button", Name::Exact("New message"))
        .first()
        .click()
        .await?;
    pa.label(Name::Exact("Search people"))
        .first()
        .fill(&b)
        .await?;
    pa.role("button", Name::Contains(&format!("@{b}")))
        .click()
        .await?;
    pa.expect_url(Name::Contains(&format!("/messages/{b}")))
        .await?;
    say(&pa, "first words").await?;
    clean(&pa, "a thread").await?;

    // The list shows the conversation and filters it.
    pa.goto("/messages").await?;
    let row = pa.role("link", Name::Re(&format!("@{b}"))).first();
    row.expect_visible().await?;
    pa.label(Name::Exact("Search conversations"))
        .fill("zzz-nobody")
        .await?;
    pa.text("No conversations match your search.")
        .expect_visible()
        .await?;
    row.expect_hidden().await?;
    pa.label(Name::Exact("Search conversations"))
        .fill(&b)
        .await?;
    row.expect_visible().await?;
    clean(&pa, "/messages with a conversation").await?;

    // A message deleted from the thread disappears for both.
    row.click().await?;
    thread(&pa).text("first words").expect_visible().await?;
    pa.role("button", Name::Exact("Delete")).click().await?;
    pa.role("dialog", Name::Exact("Delete this message?"))
        .role("button", Name::Exact("Delete"))
        .click()
        .await?;
    thread(&pa).text("first words").expect_count(0).await
}
