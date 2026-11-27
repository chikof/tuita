//! Browser tests: custom emotes, from the admin form to the picker.

use serde_json::{Value, json};
use tuita_e2e::{
    Ctx, Name, Result,
    helpers::{PASSPHRASE, call, handle, make_image, new_admin, send_dm, sign_up_via_api, turn_on},
};
use uuid::Uuid;

// The shell carries its own file inputs (the compose dialog), so the form's one is named by its island.
const FILE_INPUT: &str = "[data-island=emote-form] input[type=file]";

/// Six lowercase hex digits no other run uses.
fn nonce() -> String {
    Uuid::now_v7().simple().to_string()[26..].to_owned()
}

/// The post id of a created post.
async fn created(ctx: &Ctx, body: Value) -> Result<String> {
    let made = call(ctx, "POST", "/posts", Some(&body)).await?;
    Ok(made["id"].as_str().unwrap_or_default().to_owned())
}

#[tokio::test]
async fn staff_upload_an_emote_and_it_renders_and_autocompletes_for_everyone() -> Result<()> {
    let code = format!("zq{}", nonce());
    let shortcode = format!("{code}_blob");
    let token = format!(":{shortcode}:");
    let (admin, admin_handle) = new_admin("emo").await?;
    let admin_page = admin.page().await?;

    admin_page.goto("/admin/emotes").await?;
    admin_page
        .label(Name::Exact("Shortcode"))
        .fill(&shortcode)
        .await?;
    admin_page
        .label(Name::Exact("Category"))
        .fill("Blobs")
        .await?;
    admin_page
        .css(FILE_INPUT)
        .set_file_bytes("blob.jpg", &make_image(200, 200, 20)?)
        .await?;
    admin_page
        .role("button", Name::Exact("Upload emote"))
        .click()
        .await?;
    admin_page
        .text(Name::Contains(&token))
        .first()
        .expect_visible()
        .await?;

    call(
        &admin,
        "PATCH",
        "/me",
        Some(&json!({"displayName": format!("Admin {token}"), "bio": format!("bio {token}")})),
    )
    .await?;
    let post = created(
        &admin,
        json!({"text": format!("hello {token} at 12:30:45 and :{code}_missing:")}),
    )
    .await?;
    let with_cw = created(
        &admin,
        json!({"text": "hidden body", "cw": format!("cw {token}")}),
    )
    .await?;
    let poll = created(
        &admin,
        json!({
            "text": "a poll",
            "poll": {"options": [format!("yes {token}"), "no"], "durationHours": 1, "multiple": false},
        }),
    )
    .await?;

    let reader = Ctx::new().await?;
    let reader_handle = handle("rdr");
    sign_up_via_api(&reader, &reader_handle, "Reader").await?;
    let page = reader.page().await?;
    turn_on(&page, PASSPHRASE).await?;
    turn_on(&admin_page, PASSPHRASE).await?;
    admin_page
        .goto(&format!("/messages/{reader_handle}"))
        .await?;
    send_dm(&admin_page, &format!("dm {token}")).await?;
    let emotes = page
        .css("#main")
        .css(&format!(r#"img.emote[alt="{token}"]"#));

    page.goto(&format!("/@{admin_handle}/{post}")).await?;
    // The author's name and the text each carry the emote; an unknown shortcode and a time stay text.
    emotes.expect_count(2).await?;
    page.text(Name::Contains(&format!(":{code}_missing:")))
        .expect_visible()
        .await?;
    page.text("12:30:45").expect_visible().await?;

    page.goto(&format!("/@{admin_handle}/{with_cw}")).await?;
    emotes.first().expect_visible().await?;
    page.goto(&format!("/@{admin_handle}/{poll}")).await?;
    emotes.nth(1).expect_visible().await?;
    page.goto(&format!("/@{admin_handle}")).await?;
    emotes.nth(1).expect_visible().await?;
    page.goto(&format!("/messages/{admin_handle}")).await?;
    emotes.last().expect_visible().await?;

    page.goto("/").await?;
    let editor = page.css("textarea").first();
    editor.click().await?;
    editor.press_sequentially(&format!("hi :{code}")).await?;
    page.role("option", Name::Re(&shortcode))
        .expect_visible()
        .await?;
    page.press("Enter").await?;
    editor
        .expect_value(Name::Exact(&format!("hi {token} ")))
        .await
}

#[tokio::test]
async fn the_admin_form_checks_input_categories_can_change_emotes_can_be_deleted_and_the_picker_lists_them()
-> Result<()> {
    let (admin, _me) = new_admin("ui").await?;
    let page = admin.page().await?;
    let code = format!("ui{}", &nonce()[..5]);
    page.goto("/admin/emotes").await?;

    page.label(Name::Exact("Shortcode")).fill("a").await?;
    page.text(Name::Re("2 to 32 characters"))
        .expect_visible()
        .await?;
    page.css(FILE_INPUT)
        .set_file_bytes("big.png", &vec![1u8; 300 * 1024])
        .await?;
    page.text("Image is too large (max 256 KB).")
        .expect_visible()
        .await?;
    page.css(FILE_INPUT)
        .set_file_bytes("x.txt", b"hello")
        .await?;
    page.text(Name::Re("isn’t a PNG")).expect_visible().await?;

    for (name, category) in [(format!("{code}_a"), "Pack"), (format!("{code}_b"), "")] {
        page.label(Name::Exact("Shortcode")).fill(&name).await?;
        page.label(Name::Exact("Category")).fill(category).await?;
        page.css(FILE_INPUT)
            .set_file_bytes("e.jpg", &make_image(100, 100, 40)?)
            .await?;
        page.role("button", Name::Exact("Upload emote"))
            .click()
            .await?;
        page.text(format!("Added :{name}:.").as_str())
            .expect_visible()
            .await?;
    }

    let category = page.label(Name::Exact(&format!("Category of :{code}_b:")));
    category.fill("Moved").await?;
    category.press("Enter").await?;
    page.text("Category saved.").expect_visible().await?;
    let listed = call(&admin, "GET", "/emojis", None).await?;
    let moved = listed
        .as_array()
        .into_iter()
        .flatten()
        .find(|e| e["shortcode"] == json!(format!("{code}_b")))
        .map(|e| e["category"].clone());
    assert_eq!(moved, Some(json!("Moved")), "{listed}");

    page.goto("/").await?;
    page.role("button", Name::ReI("emoji"))
        .first()
        .click()
        .await?;
    page.role("tab", "Custom").click().await?;
    page.text("Pack").expect_visible().await?;
    page.text("Moved").expect_visible().await?;
    page.role("button", Name::Contains(&format!(":{code}_a:")))
        .click()
        .await?;
    page.css("textarea")
        .first()
        .expect_value(Name::Exact(&format!(":{code}_a: ")))
        .await?;

    page.goto("/admin/emotes").await?;
    page.role("button", Name::Contains(&format!("Delete :{code}_a:")))
        .click()
        .await?;
    page.role("dialog", Name::Any)
        .role("button", "Delete emote")
        .click()
        .await?;
    page.text("Emote deleted.").expect_visible().await?;
    let after = call(&admin, "GET", "/emojis", None).await?;
    let gone = after
        .as_array()
        .into_iter()
        .flatten()
        .all(|e| e["shortcode"] != json!(format!("{code}_a")));
    assert!(gone, "{after}");

    let user = Ctx::new().await?;
    sign_up_via_api(&user, &handle("usr"), "User").await?;
    let (status, _) = user.call("GET", "/admin/emojis", None).await?;
    assert_eq!(status, 403, "a member may not list the admin emote view");
    Ok(())
}
