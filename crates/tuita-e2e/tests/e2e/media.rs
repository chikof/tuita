//! Browser tests: attaching images and a video in the composer, the lightbox and avatar upload.

use std::path::PathBuf;

use serde_json::json;
use tuita_e2e::{
    Ctx, Loc, Name, Page, Result,
    helpers::{handle, make_image, sign_up_via_ui},
};

const SIZES: [(u32, u32); 4] = [(640, 480), (800, 600), (960, 540), (1200, 800)];

fn image_input(page: &Page) -> Loc {
    page.css("input[type=file][accept*=\"image/png\"]").first()
}

/// A fresh directory for the files of one test.
async fn scratch() -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("tuita-e2e-{}", uuid::Uuid::now_v7()));
    tokio::fs::create_dir_all(&dir).await?;
    Ok(dir)
}

#[tokio::test]
async fn four_images_with_alt_text_are_posted_the_lightbox_shows_original_pixels_a_fifth_is_refused()
-> Result<()> {
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    sign_up_via_ui(&page, "Photographer", &handle("photo")).await?;

    let dir = scratch().await?;
    let mut files = Vec::new();
    for (i, (width, height)) in SIZES.into_iter().enumerate() {
        let path = dir.join(format!("photo-{width}.jpg"));
        tokio::fs::write(&path, make_image(width, height, i as u32 * 70)?).await?;
        files.push(path);
    }
    image_input(&page).set_files(&files).await?;

    let alts: Vec<String> = SIZES
        .iter()
        .map(|(width, _)| format!("A gradient, {width} pixels wide"))
        .collect();
    for (i, alt) in alts.iter().enumerate() {
        page.label(Name::Exact(&format!(
            "Description (alt text) of attachment {}",
            i + 1
        )))
        .fill(alt)
        .await?;
    }
    page.set_timeout(std::time::Duration::from_secs(20));
    page.role("progressbar", Name::Any).expect_count(0).await?;
    page.set_timeout(std::time::Duration::from_secs(10));

    // A fifth image is refused in the UI: nothing joins the tray and the add button is disabled.
    image_input(&page)
        .set_file_bytes("fifth.jpg", &make_image(320, 240, 300)?)
        .await?;
    page.text("You can attach up to 4 images.")
        .expect_visible()
        .await?;
    page.label(Name::Re(r"^Description \(alt text\) of attachment"))
        .expect_count(4)
        .await?;
    page.role("button", Name::Exact("Add images"))
        .expect_disabled()
        .await?;

    page.label(Name::Exact("Post text"))
        .fill("Four gradients")
        .await?;
    page.css("main")
        .role("button", Name::Exact("Post"))
        .click()
        .await?;

    let card = page.role("article", Name::Any).has_text("Four gradients");
    let grid = card.role("group", Name::Exact("4 attachments"));
    grid.expect_visible().await?;
    // Above the card's hover wash: photos are never dithered.
    grid.expect_css("z-index", Name::Exact("1")).await?;
    for alt in &alts {
        grid.role("img", Name::Exact(alt)).expect_visible().await?;
    }

    grid.role("button", Name::Exact(&alts[0])).click().await?;
    let viewer = page.role("dialog", Name::Exact("Media viewer"));
    viewer.expect_visible().await?;
    for (i, (width, _)) in SIZES.into_iter().enumerate() {
        viewer
            .role("img", Name::Exact(&alts[i]))
            .expect_visible()
            .await?;
        let natural = page
            .eval(&format!(
                "(async () => {{ const img = [...document.querySelectorAll('dialog[open] img')].find((e) => e.alt === {}); await img.decode(); return img.naturalWidth; }})()",
                json!(alts[i])
            ))
            .await?;
        assert_eq!(
            natural.as_u64(),
            Some(u64::from(width)),
            "lightbox image {} is the uploaded {width} px",
            i + 1
        );
        if i < SIZES.len() - 1 {
            viewer.role("button", Name::Exact("Next")).click().await?;
        }
    }
    page.press("Escape").await?;
    viewer.expect_hidden().await?;
    let errors = page.console_errors().await;
    assert!(errors.is_empty(), "console errors: {errors:#?}");
    Ok(())
}

#[tokio::test]
async fn an_avatar_is_cropped_uploaded_and_removed_again() -> Result<()> {
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    sign_up_via_ui(&page, "Avatar Person", &handle("face")).await?;
    page.goto("/settings/profile").await?;

    let card = page.role("region", Name::Exact("Profile picture"));
    card.css("input[type=file]")
        .set_file_bytes("face.jpg", &make_image(1000, 600, 200)?)
        .await?;
    let dialog = page.role("dialog", Name::Exact("Edit profile picture"));
    dialog.expect_visible().await?;
    dialog.role("button", Name::Exact("Rotate")).click().await?;
    dialog.role("button", Name::Exact("Apply")).click().await?;
    page.text("Picture saved.").expect_visible().await?;

    // The right rail can list accounts whose names carry emotes: look only at the page body.
    let stored = page.css("#main img[src^=\"/media/\"]").first();
    stored.expect_visible().await?;
    let side = page
        .eval("(async () => { const img = document.querySelector('#main img[src^=\"/media/\"]'); await img.decode(); return [img.naturalWidth, img.naturalHeight]; })()")
        .await?;
    assert_eq!(
        side,
        json!([400, 400]),
        "avatars are stored as 400 x 400 squares"
    );

    card.role("button", Name::Re("Remove")).click().await?;
    page.text("Picture removed.").expect_visible().await?;
    page.css("#main img[src^=\"/media/\"]")
        .expect_count(0)
        .await?;
    let errors = page.console_errors().await;
    assert!(errors.is_empty(), "console errors: {errors:#?}");
    Ok(())
}

#[tokio::test]
async fn a_video_uploads_with_a_poster_frame_and_plays_inline() -> Result<()> {
    let dir = scratch().await?;
    let file = dir.join("clip.webm");
    let made = std::process::Command::new("ffmpeg")
        .args(["-loglevel", "error", "-f", "lavfi", "-i"])
        .arg("testsrc=duration=1:size=320x240:rate=10")
        .args(["-c:v", "libvpx", "-b:v", "200k", "-y"])
        .arg(&file)
        .status();
    if !made.is_ok_and(|status| status.success()) {
        eprintln!("skipped: ffmpeg is not installed");
        return Ok(());
    }
    let ctx = Ctx::new().await?;
    let page = ctx.page().await?;
    sign_up_via_ui(&page, "Videographer", &handle("film")).await?;
    page.css("input[type=file][accept*=\"video/mp4\"]")
        .first()
        .set_files(&[file])
        .await?;
    page.label(Name::Exact("Description (alt text) of attachment 1"))
        .fill("A test pattern")
        .await?;
    page.set_timeout(std::time::Duration::from_secs(20));
    page.role("progressbar", Name::Any).expect_count(0).await?;
    page.set_timeout(std::time::Duration::from_secs(10));
    page.css("main")
        .role("button", Name::Exact("Post"))
        .click()
        .await?;

    let video = page.role("article", Name::Any).first().css("video");
    video.expect_visible().await?;
    video.expect_attr("poster", Name::Re("^/media/")).await?;
    video
        .expect_attr("aria-label", Name::Exact("A test pattern"))
        .await
}
