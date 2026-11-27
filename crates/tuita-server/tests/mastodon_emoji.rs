//! Custom emotes through the Mastodon API: `GET /api/v1/custom_emojis` and the `emojis` array of
//! accounts, statuses, polls, notifications, conversations and edit history.
#![allow(missing_docs)]

mod common;
#[path = "common/oauth_client.rs"]
mod oauth_client;

use common::Person;
use oauth_client::{Api, Env, sign_in};
use serde_json::{Value, json};
use sqlx::PgPool;
use tuita_server::db::emoji;
use uuid::Uuid;

const BASE: &str = "https://social.example";

/// Stores an emote: a media row and its `custom_emojis` row, local when `domain` is `None`.
/// Returns the file name of the image.
async fn emote(
    db: &PgPool,
    owner: Uuid,
    shortcode: &str,
    domain: Option<&str>,
    category: Option<&str>,
) -> String {
    let id = Uuid::now_v7();
    let path = format!("{id}.webp");
    sqlx::query(
        "insert into media (id, owner_id, kind, mime, path, bytes, width, height)
         values ($1, $2, 'image', 'image/webp', $3, 10, 16, 16)",
    )
    .bind(id)
    .bind(owner)
    .bind(&path)
    .execute(db)
    .await
    .expect("media row");
    match domain {
        Some(domain) => {
            emoji::upsert_remote(db, domain, shortcode, id, "https://remote.example/e.png")
                .await
                .expect("remote emote")
        }
        None => {
            sqlx::query(
                "insert into custom_emojis (id, shortcode, media_id, category)
                 values ($1, $2, $3, $4)",
            )
            .bind(Uuid::now_v7())
            .bind(shortcode)
            .bind(id)
            .bind(category)
            .execute(db)
            .await
            .expect("local emote");
        }
    }
    path
}

fn shortcodes(emojis: &Value) -> Vec<&str> {
    emojis
        .as_array()
        .expect("emojis array")
        .iter()
        .map(|emoji| emoji["shortcode"].as_str().expect("shortcode"))
        .collect()
}

struct World {
    _env: Env,
    ana: Person,
    bo: Person,
    ana_api: Api,
    bo_api: Api,
    blob: String,
}

/// `ana` and `bo` with OAuth tokens and two local emotes, `blob` and `wave`; `bo` follows `ana`.
async fn world(db: &PgPool) -> World {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(db, "ana").await;
    let mut bo = env.app.person(db, "bo").await;
    bo.browser.post_empty("/users/ana/follow").await.ok();
    let blob = emote(db, ana.id, "blob", None, None).await;
    emote(db, ana.id, "wave", None, Some("Hands")).await;
    let ana_api = sign_in(&env.app, &mut ana, "read write follow", [203, 0, 113, 10]).await;
    let bo_api = sign_in(&env.app, &mut bo, "read write follow", [203, 0, 113, 11]).await;
    World {
        _env: env,
        ana,
        bo,
        ana_api,
        bo_api,
        blob,
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn custom_emojis_lists_the_local_emotes_by_category_then_shortcode(db: PgPool) {
    let mut w = world(&db).await;
    emote(&db, w.ana.id, "zed", None, None).await;
    emote(&db, w.ana.id, "far", Some("remote.example"), None).await;
    let list = w.bo_api.get("/api/v1/custom_emojis").await.ok();
    assert_eq!(shortcodes(&list), ["blob", "zed", "wave"]);
    assert_eq!(
        list[0],
        json!({
            "shortcode": "blob",
            "url": format!("{BASE}/media/{}", w.blob),
            "static_url": format!("{BASE}/media/{}", w.blob),
            "visible_in_picker": true,
            "category": null,
        })
    );
    assert_eq!(list[2]["category"], "Hands");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_status_and_its_account_carry_the_emotes_they_use(db: PgPool) {
    let mut w = world(&db).await;
    w.ana
        .browser
        .patch("/me", json!({"displayName": "Ana :wave:"}))
        .await
        .ok();
    let status = w
        .ana_api
        .post_json(
            "/api/v1/statuses",
            &json!({"status": "hi :blob: and :nope: :BLOB:", "spoiler_text": "cw :wave:"}),
        )
        .await
        .ok();
    assert_eq!(shortcodes(&status["emojis"]), ["blob", "wave"]);
    assert_eq!(
        status["emojis"][0]["url"],
        format!("{BASE}/media/{}", w.blob)
    );
    assert_eq!(shortcodes(&status["account"]["emojis"]), ["wave"]);

    let id = status["id"].as_str().expect("id");
    let seen = w.bo_api.get(&format!("/api/v1/statuses/{id}")).await.ok();
    assert_eq!(seen["emojis"], status["emojis"]);
    let timeline = w.bo_api.get("/api/v1/timelines/home").await.ok();
    assert_eq!(shortcodes(&timeline[0]["emojis"]), ["blob", "wave"]);
    assert_eq!(shortcodes(&timeline[0]["account"]["emojis"]), ["wave"]);
    let account = w
        .bo_api
        .get(&format!("/api/v1/accounts/{}", w.ana.id))
        .await
        .ok();
    assert_eq!(shortcodes(&account["emojis"]), ["wave"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_status_without_tokens_has_an_empty_array(db: PgPool) {
    let mut w = world(&db).await;
    let status = w
        .ana_api
        .post_json(
            "/api/v1/statuses",
            &json!({"status": "no emotes at 12:30:45"}),
        )
        .await
        .ok();
    assert_eq!(status["emojis"], json!([]));
    assert_eq!(status["account"]["emojis"], json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_poll_carries_the_emotes_of_its_options(db: PgPool) {
    let mut w = world(&db).await;
    let status = w
        .ana_api
        .post_json(
            "/api/v1/statuses",
            &json!({
                "status": "pick",
                "poll": {"options": ["yes :blob:", "no :wave:"], "expires_in": 86_400},
            }),
        )
        .await
        .ok();
    assert_eq!(shortcodes(&status["poll"]["emojis"]), ["blob", "wave"]);
    assert_eq!(shortcodes(&status["emojis"]), ["blob", "wave"]);
    let poll = status["poll"]["id"].as_str().expect("poll id");
    let again = w.bo_api.get(&format!("/api/v1/polls/{poll}")).await.ok();
    assert_eq!(shortcodes(&again["emojis"]), ["blob", "wave"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn notifications_and_conversations_carry_emotes(db: PgPool) {
    let mut w = world(&db).await;
    w.bo.browser
        .patch("/me", json!({"displayName": "Bo :blob:"}))
        .await
        .ok();
    let status = w
        .ana_api
        .post_json("/api/v1/statuses", &json!({"status": "look :wave:"}))
        .await
        .ok();
    let id = status["id"].as_str().expect("id");
    w.bo_api
        .post(&format!("/api/v1/statuses/{id}/favourite"))
        .await
        .ok();
    let notes = w.ana_api.get("/api/v1/notifications").await.ok();
    let favourite = notes
        .as_array()
        .expect("notifications")
        .iter()
        .find(|note| note["type"] == "favourite")
        .expect("favourite");
    assert_eq!(shortcodes(&favourite["status"]["emojis"]), ["wave"]);
    assert_eq!(shortcodes(&favourite["account"]["emojis"]), ["blob"]);

    w.bo.browser
        .post("/conversations/ana", json!({"text": "psst :wave:"}))
        .await
        .ok();
    let conversations = w.ana_api.get("/api/v1/conversations").await.ok();
    assert_eq!(
        shortcodes(&conversations[0]["last_status"]["emojis"]),
        ["wave"]
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn every_edit_history_entry_lists_its_own_emotes(db: PgPool) {
    let mut w = world(&db).await;
    let status = w
        .ana_api
        .post_json("/api/v1/statuses", &json!({"status": "first :blob:"}))
        .await
        .ok();
    let path = format!("/api/v1/statuses/{}", status["id"].as_str().expect("id"));
    w.ana_api
        .put_json(&path, &json!({"status": "second :wave:"}))
        .await
        .ok();
    let history = w.ana_api.get(&format!("{path}/history")).await.ok();
    let history = history.as_array().expect("history");
    assert_eq!(history.len(), 2);
    assert_eq!(shortcodes(&history[0]["emojis"]), ["blob"]);
    assert_eq!(shortcodes(&history[1]["emojis"]), ["wave"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_remote_post_uses_only_its_own_domains_emotes(db: PgPool) {
    let mut w = world(&db).await;
    let remote = Uuid::now_v7();
    sqlx::query(
        "insert into users (id, handle, domain, display_name, ap_id, inbox_url, fetched_at)
         values ($1, 'zed', 'remote.example', 'Zed :far:', 'https://remote.example/users/zed',
                 'https://remote.example/users/zed/inbox', now())",
    )
    .bind(remote)
    .execute(&db)
    .await
    .expect("remote user");
    let far = emote(&db, remote, "far", Some("remote.example"), None).await;
    // A local emote of the same name must not leak into a remote author's text.
    emote(&db, w.ana.id, "far", None, None).await;
    let post = Uuid::now_v7();
    sqlx::query(
        "insert into posts (id, author_id, text, ap_id, url, visibility)
         values ($1, $2, 'hello :far: :blob:', 'https://remote.example/users/zed/statuses/1',
                 'https://remote.example/@zed/1', 'public')",
    )
    .bind(post)
    .bind(remote)
    .execute(&db)
    .await
    .expect("remote post");

    let status = w.bo_api.get(&format!("/api/v1/statuses/{post}")).await.ok();
    assert_eq!(
        status["emojis"],
        json!([{
            "shortcode": "far",
            "url": format!("{BASE}/media/{far}"),
            "static_url": format!("{BASE}/media/{far}"),
            "visible_in_picker": true,
            "category": null,
        }])
    );
    assert_eq!(shortcodes(&status["account"]["emojis"]), ["far"]);
    // The picker lists the local `far`, never the remote copy.
    let listed = w.bo_api.get("/api/v1/custom_emojis").await.ok();
    assert_eq!(shortcodes(&listed), ["blob", "far", "wave"]);
    assert_ne!(listed[1]["url"], format!("{BASE}/media/{far}"));
}
