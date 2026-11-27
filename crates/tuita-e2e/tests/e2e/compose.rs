//! Browser tests: the composer and the compose dialog.
//!
//! Every test opens the dialog from a member page (`/terms`) and checks what was created through the
//! API, so none depends on how posts are displayed.

use serde_json::{Value, json};
use tuita_e2e::{
    Ctx, Loc, Name, Page, Result, Violation,
    helpers::{call, handle, make_image, new_admin, sign_up_via_api},
};

/// A signed-in context, a page on a member route and the account's handle.
async fn signed_in(prefix: &str) -> Result<(Ctx, Page, String)> {
    let ctx = Ctx::new().await?;
    let who = handle(prefix);
    sign_up_via_api(&ctx, &who, "Compose Test").await?;
    let page = ctx.page().await?;
    page.goto("/terms").await?;
    Ok((ctx, page, who))
}

/// Opens the dialog with the shell's compose button and returns it.
async fn open(page: &Page) -> Result<Loc> {
    page.role("button", Name::Exact("Post"))
        .first()
        .click()
        .await?;
    let dialog = page.role("dialog", Name::Exact("New post"));
    dialog.expect_visible().await?;
    Ok(dialog)
}

fn text_box(dialog: &Loc) -> Loc {
    dialog.role("combobox", Name::Exact("Post text"))
}

async fn posts_of(ctx: &Ctx, who: &str) -> Result<Vec<Value>> {
    let page = call(ctx, "GET", &format!("/users/{who}/posts"), None).await?;
    Ok(page["items"].as_array().cloned().unwrap_or_default())
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
async fn a_post_with_a_poll_and_a_content_warning_is_created_through_the_dialog() -> Result<()> {
    let (ctx, page, who) = signed_in("cpl").await?;
    let dialog = open(&page).await?;
    let send = dialog.role("button", Name::Exact("Post"));
    send.expect_disabled().await?;

    text_box(&dialog).fill("Which colour?").await?;
    dialog
        .role("button", Name::Exact("Add a content warning"))
        .click()
        .await?;
    dialog
        .role("textbox", Name::Exact("Content warning"))
        .fill("polls")
        .await?;
    dialog
        .role("button", Name::Exact("Add a poll"))
        .click()
        .await?;
    send.expect_disabled().await?;

    dialog.label(Name::Exact("Choice 1")).fill("Red").await?;
    dialog.label(Name::Exact("Choice 2")).fill("Blue").await?;
    send.expect_enabled().await?;
    dialog
        .role("button", Name::Exact("Add a choice"))
        .click()
        .await?;
    dialog
        .label(Name::Exact("Choice 3 (optional)"))
        .fill("Green")
        .await?;
    dialog
        .role("button", Name::Exact("Remove choice 3"))
        .click()
        .await?;
    dialog
        .label(Name::Contains("Choice 3"))
        .expect_hidden()
        .await?;
    dialog.role("tab", Name::Exact("3 d")).click().await?;
    dialog
        .role("switch", Name::Exact("Allow multiple choices"))
        .click()
        .await?;

    dialog
        .role("button", Name::Exact("Everyone can reply"))
        .click()
        .await?;
    page.role("menuitem", Name::Exact("Only people you mention can reply"))
        .click()
        .await?;
    dialog
        .role("button", Name::Exact("Only people you mention can reply"))
        .expect_visible()
        .await?;

    clean(&page, "compose dialog with a poll").await?;

    send.click().await?;
    page.text(Name::Exact("Post sent")).expect_visible().await?;
    dialog.expect_hidden().await?;

    let posts = posts_of(&ctx, &who).await?;
    let post = posts.first().expect("the post is listed");
    assert_eq!(post["text"], "Which colour?");
    assert_eq!(post["cw"], "polls");
    assert_eq!(post["replyPolicy"], "mentioned");
    let options: Vec<&str> = post["poll"]["options"]
        .as_array()
        .expect("a poll")
        .iter()
        .filter_map(|o| o["text"].as_str())
        .collect();
    assert_eq!(options, ["Red", "Blue"]);
    assert_eq!(post["poll"]["multiple"], true);
    let length = page
        .eval_fn(
            "(poll) => Math.round((Date.parse(poll.endsAt) - Date.now()) / 3_600_000)",
            &post["poll"],
        )
        .await?;
    assert_eq!(length, json!(72), "the 3 d length was chosen");
    Ok(())
}

#[tokio::test]
async fn a_draft_survives_a_reload_and_is_gone_after_posting() -> Result<()> {
    let (ctx, page, who) = signed_in("cdr").await?;
    let me = call(&ctx, "GET", "/me", None).await?;
    let id = me["user"]["user"]["id"].as_str().expect("my id").to_owned();

    let dialog = open(&page).await?;
    text_box(&dialog).fill("half written").await?;
    page.wait_for_js(&format!(
        "localStorage.getItem('tuita:draft:{id}:new') === 'half written'"
    ))
    .await?;

    page.reload().await?;
    let dialog = open(&page).await?;
    text_box(&dialog)
        .expect_value(Name::Exact("half written"))
        .await?;

    dialog.role("button", Name::Exact("Post")).click().await?;
    page.text(Name::Exact("Post sent")).expect_visible().await?;
    dialog.expect_hidden().await?;
    page.wait_for_js(&format!(
        "localStorage.getItem('tuita:draft:{id}:new') === null"
    ))
    .await?;
    let posts = posts_of(&ctx, &who).await?;
    assert_eq!(posts[0]["text"], "half written");

    let dialog = open(&page).await?;
    text_box(&dialog).expect_value(Name::Exact("")).await?;
    Ok(())
}

#[tokio::test]
async fn signing_out_forgets_the_draft() -> Result<()> {
    let (ctx, page, who) = signed_in("cfg").await?;
    let me = call(&ctx, "GET", "/me", None).await?;
    let id = me["user"]["user"]["id"].as_str().expect("my id").to_owned();
    let dialog = open(&page).await?;
    text_box(&dialog).fill("secret plans").await?;
    page.wait_for_js(&format!(
        "localStorage.getItem('tuita:draft:{id}:new') === 'secret plans'"
    ))
    .await?;
    page.press("Escape").await?;
    dialog.expect_hidden().await?;

    page.css("[data-c='account-menu'] .trigger").click().await?;
    page.role("button", Name::Exact(&format!("Log out @{who}")))
        .click()
        .await?;
    page.role("button", Name::Exact("Sign in"))
        .expect_visible()
        .await?;
    page.wait_for_js(&format!(
        "Object.keys(localStorage).every((k) => !k.startsWith('tuita:draft:{id}:'))"
    ))
    .await?;
    Ok(())
}

#[tokio::test]
async fn a_thread_posts_every_part_in_order() -> Result<()> {
    let (ctx, page, who) = signed_in("cth").await?;
    let dialog = open(&page).await?;
    text_box(&dialog).fill("Thread part one").await?;
    dialog
        .role("button", Name::Exact("Add another post"))
        .click()
        .await?;
    dialog
        .role("textbox", Name::Exact("Thread post 2"))
        .fill("Thread part two")
        .await?;
    dialog
        .role("button", Name::Exact("Add another post"))
        .click()
        .await?;
    dialog
        .role("textbox", Name::Exact("Thread post 3"))
        .fill("Thread part three")
        .await?;
    dialog
        .role("button", Name::Exact("Remove thread post 3"))
        .click()
        .await?;
    dialog
        .role("textbox", Name::Exact("Thread post 3"))
        .expect_hidden()
        .await?;
    dialog
        .role("button", Name::Exact("Post all"))
        .click()
        .await?;
    page.text(Name::Exact("Thread of 2 posts sent"))
        .expect_visible()
        .await?;

    let posts = posts_of(&ctx, &who).await?;
    let root = posts
        .iter()
        .find(|p| p["text"] == "Thread part one")
        .expect("the first part");
    let id = root["id"].as_str().expect("an id");
    let thread = call(&ctx, "GET", &format!("/posts/{id}"), None).await?;
    let replies = thread["replies"].as_array().expect("replies");
    assert_eq!(replies.len(), 1, "{thread}");
    assert_eq!(replies[0]["text"], "Thread part two");
    Ok(())
}

#[tokio::test]
async fn a_scheduled_post_is_saved_instead_of_published() -> Result<()> {
    let (ctx, page, who) = signed_in("csc").await?;
    let dialog = open(&page).await?;
    text_box(&dialog).fill("See you later").await?;
    dialog
        .role("button", Name::Exact("Schedule post"))
        .click()
        .await?;
    page.role("button", Name::Contains("In an hour"))
        .click()
        .await?;
    dialog
        .text(Name::Contains("Will post"))
        .expect_visible()
        .await?;
    dialog
        .role("button", Name::Exact("Add another post"))
        .expect_hidden()
        .await?;
    dialog
        .role("button", Name::Exact("Schedule"))
        .click()
        .await?;
    page.text(Name::Contains("Scheduled for"))
        .expect_visible()
        .await?;
    dialog.expect_hidden().await?;

    let scheduled = call(&ctx, "GET", "/scheduled", None).await?;
    let first = &scheduled[0];
    assert_eq!(first["text"], "See you later");
    assert_eq!(first["status"], "pending");
    assert!(posts_of(&ctx, &who).await?.is_empty());
    Ok(())
}

#[tokio::test]
async fn the_counter_shows_what_is_left_and_blocks_an_overlong_post() -> Result<()> {
    let (_ctx, page, _who) = signed_in("cct").await?;
    let dialog = open(&page).await?;
    let send = dialog.role("button", Name::Exact("Post"));
    dialog
        .role("img", Name::Contains("characters"))
        .expect_hidden()
        .await?;

    text_box(&dialog).fill(&"a".repeat(495)).await?;
    dialog
        .role("img", Name::Exact("5 characters left"))
        .expect_visible()
        .await?;
    send.expect_enabled().await?;

    text_box(&dialog).fill(&"a".repeat(505)).await?;
    dialog
        .role("img", Name::Contains("5 characters over"))
        .expect_visible()
        .await?;
    send.expect_disabled().await?;

    text_box(&dialog)
        .fill("look https://example.com/a/very/long/path/that/counts/as/23/characters")
        .await?;
    dialog
        .role("img", Name::Contains("characters left"))
        .expect_visible()
        .await?;
    send.expect_enabled().await?;
    Ok(())
}

#[tokio::test]
async fn people_and_tags_are_suggested_and_the_pick_lands_in_the_text() -> Result<()> {
    let (ctx, page, _who) = signed_in("csg").await?;
    let other_ctx = Ctx::new().await?;
    let other = handle("zqx");
    sign_up_via_api(&other_ctx, &other, "Zed Person").await?;
    let tag = handle("zqt");
    call(
        &other_ctx,
        "POST",
        "/posts",
        Some(
            &json!({"text": format!("hello #{tag}"), "cw": "", "sensitive": false, "mediaIds": []}),
        ),
    )
    .await?;
    let _ = &ctx;

    let dialog = open(&page).await?;
    let text = text_box(&dialog);
    text.fill(&format!("hi @{other}")).await?;
    dialog
        .role("option", Name::Contains(&format!("@{other}")))
        .expect_visible()
        .await?;
    text.press("Enter").await?;
    text.expect_value(Name::Exact(&format!("hi @{other} ")))
        .await?;
    dialog
        .role("listbox", Name::Exact("Suggestions"))
        .expect_hidden()
        .await?;

    text.fill(&format!("#{tag}")).await?;
    dialog
        .role("option", Name::Contains(&format!("#{tag}")))
        .expect_visible()
        .await?;
    text.press("Escape").await?;
    dialog
        .role("listbox", Name::Exact("Suggestions"))
        .expect_hidden()
        .await?;
    dialog.expect_visible().await?;
    Ok(())
}

#[tokio::test]
async fn an_emoji_from_the_picker_lands_at_the_caret() -> Result<()> {
    let (_ctx, page, _who) = signed_in("cem").await?;
    let dialog = open(&page).await?;
    let text = text_box(&dialog);
    text.fill("so good").await?;
    dialog
        .role("button", Name::Exact("Add emoji"))
        .click()
        .await?;
    page.role("button", Name::Exact("🔥"))
        .expect_visible()
        .await?;
    clean(&page, "emoji picker open").await?;
    page.role("button", Name::Exact("🔥")).click().await?;
    text.expect_value(Name::Exact("so good🔥")).await?;

    dialog
        .role("button", Name::Exact("Schedule post"))
        .click()
        .await?;
    page.role("button", Name::Contains("In an hour"))
        .expect_visible()
        .await?;
    clean(&page, "schedule picker open").await?;
    Ok(())
}

/// Uploads an emote through the API from a page of `ctx`.
async fn upload_emote(ctx: &Ctx, shortcode: &str, category: &str) -> Result<()> {
    let bytes = make_image(96, 96, 20)?;
    let out = ctx
        .api_page()
        .await?
        .eval_fn(
            r"async ([bytes, shortcode, category]) => {
                const form = new FormData();
                form.append('shortcode', shortcode);
                form.append('category', category);
                form.append('file', new Blob([new Uint8Array(bytes)], { type: 'image/jpeg' }), 'blob.jpg');
                const res = await fetch('/api/v1/admin/emojis', { method: 'POST', credentials: 'same-origin', body: form });
                return res.status;
            }",
            &json!([bytes, shortcode, category]),
        )
        .await?;
    assert_eq!(out, json!(200), "the emote upload");
    Ok(())
}

#[tokio::test]
async fn custom_emotes_are_suggested_after_a_colon_and_listed_in_the_picker() -> Result<()> {
    let (admin, _who) = new_admin("cmt").await?;
    let code = handle("zq");
    let shortcode = format!("{code}_blob");
    let token = format!(":{shortcode}:");
    upload_emote(&admin, &shortcode, "Blobs").await?;

    let page = admin.page().await?;
    page.goto("/terms").await?;
    let dialog = open(&page).await?;
    let text = text_box(&dialog);
    text.fill(&format!("hi :{code}")).await?;
    dialog
        .role("option", Name::Contains(&token))
        .expect_visible()
        .await?;
    text.press("Enter").await?;
    text.expect_value(Name::Exact(&format!("hi {token} ")))
        .await?;

    text.fill("").await?;
    dialog
        .role("button", Name::Exact("Add emoji"))
        .click()
        .await?;
    page.role("tab", Name::Exact("Custom")).click().await?;
    page.role("heading", Name::Exact("Blobs"))
        .expect_visible()
        .await?;
    page.role("button", Name::Exact(&token)).click().await?;
    text.expect_value(Name::Exact(&format!("{token} "))).await?;
    Ok(())
}

#[tokio::test]
async fn a_multipart_form_posts_by_xhr_reloads_on_success_and_toasts_the_error() -> Result<()> {
    let (admin, _who) = new_admin("cmp").await?;
    let page = admin.page().await?;
    page.goto("/terms").await?;
    let code = handle("zm");
    let inject = r#"(() => {
        window.__before_reload = true;
        document.body.insertAdjacentHTML('beforeend',
            '<form id="mp" data-island="multipart-form" data-action="/api/v1/admin/emojis" data-failure="The upload failed.">' +
            '<input name="shortcode" aria-label="Emote code"><input type="file" name="file" id="mpfile">' +
            '<button type="submit">Send emote</button></form>');
    })()"#;
    page.eval(inject).await?;
    let bytes = make_image(64, 64, 120)?;

    page.label(Name::Exact("Emote code")).fill(&code).await?;
    page.css("#mpfile")
        .set_file_bytes("blob.jpg", &bytes)
        .await?;
    page.role("button", Name::Exact("Send emote"))
        .click()
        .await?;
    page.wait_for_js("window.__before_reload === undefined")
        .await?;
    let list = call(&admin, "GET", "/emojis", None).await?;
    assert!(
        list.as_array()
            .expect("a list")
            .iter()
            .any(|e| e["shortcode"] == json!(code)),
        "{list}"
    );

    page.eval(inject).await?;
    page.label(Name::Exact("Emote code")).fill(&code).await?;
    page.css("#mpfile")
        .set_file_bytes("blob.jpg", &bytes)
        .await?;
    page.role("button", Name::Exact("Send emote"))
        .click()
        .await?;
    page.css("#toaster .toast").expect_visible().await?;
    page.role("button", Name::Exact("Send emote"))
        .expect_enabled()
        .await?;
    page.eval("window.__before_reload")
        .await
        .map(|v| assert_eq!(v, json!(true), "no reload on failure"))?;
    Ok(())
}

#[tokio::test]
async fn an_image_uploads_with_alt_text_and_joins_the_post() -> Result<()> {
    let (ctx, page, who) = signed_in("cim").await?;
    let dialog = open(&page).await?;
    let bytes = make_image(640, 480, 200)?;
    page.css("dialog#compose input[data-role='images']")
        .set_file_bytes("gradient.jpg", &bytes)
        .await?;

    let alt = dialog.role(
        "textbox",
        Name::Exact("Description (alt text) of attachment 1"),
    );
    alt.fill("A blue gradient").await?;
    dialog
        .role("button", Name::Exact("Remove image"))
        .expect_visible()
        .await?;
    dialog.css(".tray .status").expect_hidden().await?;
    clean(&page, "compose dialog with an image").await?;

    dialog.role("button", Name::Exact("Post")).click().await?;
    page.text(Name::Exact("Post sent")).expect_visible().await?;

    let posts = posts_of(&ctx, &who).await?;
    let media = posts[0]["media"].as_array().expect("media");
    assert_eq!(media.len(), 1, "{:?}", posts[0]);
    assert_eq!(media[0]["alt"], "A blue gradient");
    assert_eq!(posts[0]["text"], "");
    Ok(())
}

#[tokio::test]
async fn a_dragged_file_lights_the_dropzone_and_a_drop_attaches_it() -> Result<()> {
    let (_ctx, page, _who) = signed_in("cdz").await?;
    let dialog = open(&page).await?;
    let bytes = make_image(64, 48, 10)?;
    let drag = r"(async (kind, bytes, name, type) => {
        const zone = document.querySelector('dialog#compose [data-island=dropzone]');
        const data = new DataTransfer();
        data.items.add(new File([new Uint8Array(bytes)], name, { type }));
        zone.dispatchEvent(new DragEvent(kind, { bubbles: true, cancelable: true, dataTransfer: data }));
        return [zone.hasAttribute('data-over'), zone.hasAttribute('data-rejected')];
    })";

    let pdf = page
        .eval(&format!(
            "{drag}('dragenter', [1], 'a.pdf', 'application/pdf')"
        ))
        .await?;
    assert_eq!(
        pdf,
        json!([true, true]),
        "a refused type turns the zone red"
    );
    let png = page
        .eval(&format!("{drag}('dragenter', [1], 'a.png', 'image/png')"))
        .await?;
    assert_eq!(png, json!([true, false]));
    let left = page
        .eval(
            "(() => { const z = document.querySelector('dialog#compose [data-island=dropzone]'); z.dispatchEvent(new DragEvent('dragleave', { bubbles: true })); return z.hasAttribute('data-over'); })()",
        )
        .await?;
    assert_eq!(left, json!(false));

    page.eval(&format!(
        "{drag}('drop', {}, 'drop.jpg', 'image/jpeg')",
        json!(bytes)
    ))
    .await?;
    dialog
        .role(
            "textbox",
            Name::Exact("Description (alt text) of attachment 1"),
        )
        .expect_visible()
        .await?;
    page.eval(
        "document.querySelector('dialog#compose [data-island=dropzone]').hasAttribute('data-over')",
    )
    .await
    .map(|over| assert_eq!(over, json!(false)))?;

    // A poll switches the zone off: nothing lights up, nothing attaches.
    dialog
        .role("button", Name::Exact("Remove image"))
        .click()
        .await?;
    dialog
        .role("button", Name::Exact("Add a poll"))
        .click()
        .await?;
    let off = page
        .eval(&format!("{drag}('dragenter', [1], 'a.png', 'image/png')"))
        .await?;
    assert_eq!(off, json!([false, false]), "a poll excludes media");
    Ok(())
}

#[tokio::test]
async fn a_reply_and_a_quote_open_the_dialog_with_their_target() -> Result<()> {
    let (ctx, page, who) = signed_in("crq").await?;
    let made = call(
        &ctx,
        "POST",
        "/posts",
        Some(&json!({"text": "reply to me", "cw": "", "sensitive": false, "mediaIds": []})),
    )
    .await?;
    let id = made["id"].as_str().expect("the post id").to_owned();
    let trigger = |kind: &str| {
        format!(
            "(() => {{ const b = document.createElement('button'); b.type = 'button'; b.dataset.compose = '{kind}'; b.dataset.postId = '{id}'; b.textContent = '{kind}'; document.body.append(b); b.click(); b.remove(); }})()"
        )
    };

    page.eval(&trigger("reply")).await?;
    let dialog = page.role("dialog", Name::Exact("Reply"));
    dialog.expect_visible().await?;
    dialog
        .text(Name::Exact(&format!("Replying to @{who}")))
        .expect_visible()
        .await?;
    dialog
        .role("button", Name::Exact("Add another post"))
        .expect_hidden()
        .await?;
    dialog
        .role("button", Name::Exact("Schedule post"))
        .expect_hidden()
        .await?;
    let reply_box = dialog.role("combobox", Name::Exact("Post text"));
    reply_box.fill("a fine reply").await?;
    dialog.role("button", Name::Exact("Reply")).click().await?;
    page.text(Name::Exact("Post sent")).expect_visible().await?;
    dialog.expect_hidden().await?;

    let thread = call(&ctx, "GET", &format!("/posts/{id}"), None).await?;
    assert_eq!(thread["replies"][0]["text"], "a fine reply");

    page.eval(&trigger("quote")).await?;
    let dialog = page.role("dialog", Name::Exact("Quote post"));
    dialog.expect_visible().await?;
    dialog
        .text(Name::Exact("reply to me"))
        .expect_visible()
        .await?;
    dialog
        .role("combobox", Name::Exact("Post text"))
        .expect_value(Name::Exact(""))
        .await?;
    dialog
        .role("combobox", Name::Exact("Post text"))
        .fill("worth a quote")
        .await?;
    dialog.role("button", Name::Exact("Post")).click().await?;
    dialog.expect_hidden().await?;
    let quotes = call(&ctx, "GET", &format!("/posts/{id}/quotes"), None).await?;
    let first = quotes["items"].get(0).expect("a quote");
    assert_eq!(first["text"], "worth a quote");
    Ok(())
}

#[tokio::test]
async fn the_n_shortcut_opens_the_dialog_and_focuses_the_box() -> Result<()> {
    let (_ctx, page, _who) = signed_in("cnk").await?;
    page.press("n").await?;
    let dialog = page.role("dialog", Name::Exact("New post"));
    dialog.expect_visible().await?;
    page.wait_for_js("document.activeElement?.matches('dialog#compose textarea')")
        .await?;
    Ok(())
}

/// The markup of `ui::cropper` (kept in step with `crates/tuita-web/src/ui/cropper.rs`), inside the island that uploads.
const CROP_UPLOAD: &str = r#"(() => {
    window.__before_reload = true;
    document.body.insertAdjacentHTML('beforeend',
        '<div id="cu" data-island="crop-upload" data-purpose="avatar">' +
        '<input type="file" data-role="pick" id="cupick" accept="image/*">' +
        '<dialog data-role="crop" aria-label="Crop picture">' +
        '<form class="cropper" data-c="cropper" data-island="cropper" data-kind="avatar" data-shape="circle" novalidate>' +
        '<div class="stage avatar" role="application" tabindex="0" aria-label="Crop area"></div>' +
        '<button type="button" data-act="zoom-out" aria-label="Zoom out" disabled>-</button>' +
        '<input type="range" data-role="zoom" min="1" max="6" step="0.01" value="1" aria-label="Zoom" disabled>' +
        '<button type="button" data-act="zoom-in" aria-label="Zoom in" disabled>+</button>' +
        '<button type="button" data-act="rotate" aria-label="Rotate" disabled>r</button>' +
        '<button type="button" data-act="reset" aria-label="Reset" disabled>x</button>' +
        '<button type="button" data-act="cancel">Cancel crop</button>' +
        '<button type="submit" disabled>Apply crop</button>' +
        '<input type="hidden" name="crop"></form></dialog></div>');
})()"#;

#[tokio::test]
async fn a_cropped_picture_uploads_with_its_rect_and_the_page_reloads() -> Result<()> {
    let (ctx, page, _who) = signed_in("ccr").await?;
    page.eval(CROP_UPLOAD).await?;
    let bytes = make_image(640, 480, 300)?;

    page.css("#cupick").set_file_bytes("me.jpg", &bytes).await?;
    let crop = page.role("dialog", Name::Exact("Crop picture"));
    crop.expect_visible().await?;
    let apply = crop.role("button", Name::Exact("Apply crop"));
    apply.expect_enabled().await?;
    apply.click().await?;
    page.wait_for_js("window.__before_reload === undefined")
        .await?;

    let me = call(&ctx, "GET", "/me", None).await?;
    assert!(me["user"]["user"]["avatarUrl"].is_string(), "{me}");
    Ok(())
}
