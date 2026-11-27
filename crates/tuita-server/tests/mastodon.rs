//! The M6 acceptance flow: two accounts sign in to a Mastodon client through OAuth2
//! (authorization code and PKCE, approved with a session cookie), one posts a status with media,
//! the other reads it in `timelines/home` and favourites it, and the author sees the notification.
#![allow(missing_docs)]

mod common;
#[path = "common/oauth_client.rs"]
mod oauth_client;

use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder as _, Rgba, RgbaImage};
use serde_json::json;
use sqlx::PgPool;

use oauth_client::{Env, sign_in};

fn png(width: u32, height: u32) -> Vec<u8> {
    let image = RgbaImage::from_fn(width, height, |x, y| Rgba([x as u8, y as u8, 7, 255]));
    let mut out = Vec::new();
    PngEncoder::new(&mut out)
        .write_image(image.as_raw(), width, height, ExtendedColorType::Rgba8)
        .expect("encode png");
    out
}

#[sqlx::test(migrations = "./migrations")]
async fn a_mastodon_client_posts_with_media_reads_home_favourites_and_notifies(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;
    let mut bo = env.app.person(&db, "bo").await;
    ana.browser.post_empty("/users/bo/follow").await.ok();

    let mut bo_api = sign_in(&env.app, &mut bo, "read write follow", [203, 0, 113, 10]).await;
    let mut ana_api = sign_in(&env.app, &mut ana, "read write follow", [203, 0, 113, 11]).await;

    let media = bo_api
        .upload("image/png", png(40, 30), "a gradient")
        .await
        .ok();
    assert_eq!(media["type"], "image");
    assert_eq!(media["description"], "a gradient");
    let media_id = media["id"].as_str().expect("media id").to_owned();

    let posted = bo_api
        .post_json(
            "/api/v1/statuses",
            &json!({"status": "hello from #mastodon", "media_ids": [media_id], "visibility": "public"}),
        )
        .await
        .ok();
    let status_id = posted["id"].as_str().expect("status id").to_owned();
    assert_eq!(posted["account"]["acct"], "bo");
    assert_eq!(posted["media_attachments"][0]["id"], media_id.as_str());
    assert_eq!(posted["tags"][0]["name"], "mastodon");
    assert_eq!(posted["favourited"], false);

    let home = ana_api.get("/api/v1/timelines/home").await.ok();
    let first = &home[0];
    assert_eq!(first["id"], status_id.as_str());
    assert_eq!(first["media_attachments"][0]["type"], "image");
    assert!(
        first["content"]
            .as_str()
            .expect("content")
            .contains("hello from")
    );

    let liked = ana_api
        .post(&format!("/api/v1/statuses/{status_id}/favourite"))
        .await
        .ok();
    assert_eq!(liked["favourited"], true);
    assert_eq!(liked["favourites_count"], 1);

    let notifications = bo_api.get("/api/v1/notifications").await.ok();
    // Newest first: the favourite, then the follow that made the post reach the home timeline.
    assert_eq!(notifications.as_array().expect("array").len(), 2);
    assert_eq!(notifications[1]["type"], "follow");
    let note = &notifications[0];
    assert_eq!(note["type"], "favourite");
    assert_eq!(note["account"]["acct"], "ana");
    assert_eq!(note["status"]["id"], status_id.as_str());
}
