//! The Mastodon status, timeline, notification, media, poll and conversation routes, driven
//! through OAuth tokens: JSON keys clients parse, pagination, authorisation and refusals.
#![allow(missing_docs)]

mod common;
#[path = "common/oauth_client.rs"]
mod oauth_client;

use axum::http::{Method, StatusCode};
use common::{Person, RawReply};
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder as _, Rgba, RgbaImage};
use oauth_client::{Api, Env, sign_in};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

fn png(width: u32, height: u32) -> Vec<u8> {
    let image = RgbaImage::from_fn(width, height, |x, y| Rgba([x as u8, y as u8, 7, 255]));
    let mut out = Vec::new();
    PngEncoder::new(&mut out)
        .write_image(image.as_raw(), width, height, ExtendedColorType::Rgba8)
        .expect("encode png");
    out
}

/// Three accounts: `ana` and `bo` follow each other and hold OAuth tokens; `cy` follows nobody
/// and has only a session.
struct World {
    env: Env,
    db: PgPool,
    ana: Person,
    bo: Person,
    cy: Person,
    ana_api: Api,
    bo_api: Api,
}

async fn world(db: PgPool) -> World {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;
    let mut bo = env.app.person(&db, "bo").await;
    let cy = env.app.person(&db, "cy").await;
    ana.browser.post_empty("/users/bo/follow").await.ok();
    bo.browser.post_empty("/users/ana/follow").await.ok();
    let ana_api = sign_in(&env.app, &mut ana, "read write follow", [203, 0, 113, 10]).await;
    let bo_api = sign_in(&env.app, &mut bo, "read write follow", [203, 0, 113, 11]).await;
    World {
        env,
        db,
        ana,
        bo,
        cy,
        ana_api,
        bo_api,
    }
}

/// The path and query of the `rel` link of a reply.
fn link(reply: &RawReply, rel: &str) -> Option<String> {
    reply
        .header("link")
        .split(", ")
        .find(|part| part.ends_with(&format!("rel=\"{rel}\"")))
        .map(|part| {
            let url = part
                .split(';')
                .next()
                .unwrap_or_default()
                .trim_matches(['<', '>']);
            url.strip_prefix(common::ORIGIN).unwrap_or(url).to_owned()
        })
}

fn ids(list: &Value) -> Vec<String> {
    list.as_array()
        .expect("array")
        .iter()
        .map(|item| item["id"].as_str().expect("id").to_owned())
        .collect()
}

async fn post_status(api: &mut Api, text: &str) -> Value {
    api.post_json("/api/v1/statuses", &json!({"status": text}))
        .await
        .ok()
}

#[track_caller]
fn refused(reply: &RawReply, status: StatusCode) -> String {
    assert_eq!(
        reply.status,
        status,
        "body: {}",
        String::from_utf8_lossy(&reply.bytes)
    );
    reply.json()["error"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

async fn make_followers_only(db: &PgPool, id: &str) {
    sqlx::query("update posts set visibility = 'followers' where id = $1")
        .bind(id.parse::<Uuid>().expect("uuid"))
        .execute(db)
        .await
        .expect("update visibility");
}

// ------------------------------------------------------------------ posting

#[sqlx::test(migrations = "./migrations")]
async fn a_status_has_the_keys_clients_parse(db: PgPool) {
    let mut w = world(db).await;
    let status = w
        .ana_api
        .post_form(
            "/api/v1/statuses",
            &[
                ("status", "Hi @bo, see #Rust & <b>this</b>"),
                ("spoiler_text", "warning"),
                ("language", "en"),
            ],
        )
        .await
        .ok();
    for key in [
        "id",
        "uri",
        "created_at",
        "account",
        "content",
        "visibility",
        "sensitive",
        "spoiler_text",
        "media_attachments",
        "mentions",
        "tags",
        "emojis",
        "reblogs_count",
        "favourites_count",
        "replies_count",
        "url",
        "in_reply_to_id",
        "in_reply_to_account_id",
        "reblog",
        "poll",
        "card",
        "favourited",
        "reblogged",
        "bookmarked",
        "pinned",
        "edited_at",
    ] {
        assert!(status.get(key).is_some(), "missing {key}: {status}");
    }
    assert_eq!(status["visibility"], "public");
    assert_eq!(status["spoiler_text"], "warning");
    assert_eq!(status["sensitive"], true);
    assert_eq!(status["mentions"][0]["acct"], "bo");
    assert_eq!(status["tags"][0]["name"], "rust");
    let content = status["content"].as_str().expect("content");
    assert!(content.contains("&lt;b&gt;this&lt;/b&gt;"), "{content}");
    assert!(!content.contains("<b>"), "{content}");
    assert_eq!(status["account"]["acct"], "ana");
    assert_eq!(status["media_attachments"], json!([]));
    assert_eq!(status["emojis"], json!([]));
    // The mention reached bo.
    let notes = w.bo_api.get("/api/v1/notifications").await.ok();
    assert_eq!(notes[0]["type"], "mention");
    assert_eq!(notes[0]["status"]["id"], status["id"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn only_what_tuita_can_do_is_accepted_and_a_read_token_cannot_post(db: PgPool) {
    let mut w = world(db).await;
    for visibility in ["unlisted", "private", "direct"] {
        let reply = w
            .ana_api
            .post_json(
                "/api/v1/statuses",
                &json!({"status": "x", "visibility": visibility}),
            )
            .await;
        let message = refused(&reply, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(message.contains("public"), "{message}");
    }
    let reply = w
        .ana_api
        .post_json(
            "/api/v1/statuses",
            &json!({"status": "x", "visibility": "bogus"}),
        )
        .await;
    refused(&reply, StatusCode::UNPROCESSABLE_ENTITY);
    // Nothing was posted by the refusals.
    let posts: i64 = sqlx::query_scalar("select count(*) from posts")
        .fetch_one(&w.db)
        .await
        .expect("count");
    assert_eq!(posts, 0);

    for body in [
        json!({"status": "x", "poll": {"options": ["a", "b"], "expires_in": 300}}),
        json!({"status": "x", "poll": {"options": ["a", "b"], "expires_in": 3600, "hide_totals": true}}),
        json!({"status": ""}),
        json!({"status": "x", "media_ids": ["not-a-uuid"]}),
        json!({"status": "x".repeat(600)}),
    ] {
        let reply = w.ana_api.post_json("/api/v1/statuses", &body).await;
        refused(&reply, StatusCode::UNPROCESSABLE_ENTITY);
    }

    let mut ana = w.ana;
    let mut reader = sign_in(&w.env.app, &mut ana, "read", [203, 0, 113, 12]).await;
    assert_eq!(
        reader.get("/api/v1/timelines/home").await.status,
        StatusCode::OK
    );
    let reply = reader
        .post_json("/api/v1/statuses", &json!({"status": "no"}))
        .await;
    refused(&reply, StatusCode::FORBIDDEN);
    assert_eq!(
        reader
            .post("/api/v1/statuses/00000000-0000-0000-0000-000000000000/favourite")
            .await
            .status,
        StatusCode::FORBIDDEN
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn an_idempotency_key_returns_the_first_status(db: PgPool) {
    let mut w = world(db).await;
    let body = json!({"status": "once"});
    let headers = [("idempotency-key", "key-1")];
    let first = w
        .ana_api
        .send_json(Method::POST, "/api/v1/statuses", &headers, &body)
        .await
        .ok();
    let second = w
        .ana_api
        .send_json(Method::POST, "/api/v1/statuses", &headers, &body)
        .await
        .ok();
    assert_eq!(first["id"], second["id"]);
    let other = w
        .ana_api
        .send_json(
            Method::POST,
            "/api/v1/statuses",
            &[("idempotency-key", "key-2")],
            &body,
        )
        .await
        .ok();
    assert_ne!(first["id"], other["id"]);
    // The key belongs to its account.
    let theirs = w
        .bo_api
        .send_json(Method::POST, "/api/v1/statuses", &headers, &body)
        .await
        .ok();
    assert_ne!(first["id"], theirs["id"]);
    let count: i64 = sqlx::query_scalar("select count(*) from posts where author_id = $1")
        .bind(w.ana.id)
        .fetch_one(&w.db)
        .await
        .expect("count");
    assert_eq!(count, 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_word_filter_reply_policy_and_blocks_apply_to_mastodon_posts(db: PgPool) {
    let mut w = world(db).await;
    sqlx::query(
        "insert into instance_settings (key, value) values ('content_filter',
            '{\"posts\": \"off\", \"bios\": \"off\", \"blocked\": \"forbiddenword\", \"allowed\": \"\", \"reserved\": \"\"}'::jsonb)",
    )
    .execute(&w.db)
    .await
    .expect("filter");
    let reply = w
        .ana_api
        .post_json(
            "/api/v1/statuses",
            &json!({"status": "a forbiddenword here"}),
        )
        .await;
    let message = refused(&reply, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(message.contains("isn\u{2019}t allowed"), "{message}");
    let status = post_status(&mut w.ana_api, "a post").await;
    let id = status["id"].as_str().expect("id").to_owned();
    // Reply policy `mentioned`: bo may not reply.
    sqlx::query("update posts set reply_policy = 'mentioned' where id = $1")
        .bind(id.parse::<Uuid>().expect("uuid"))
        .execute(&w.db)
        .await
        .expect("policy");
    let reply = w
        .bo_api
        .post_json(
            "/api/v1/statuses",
            &json!({"status": "reply", "in_reply_to_id": id}),
        )
        .await;
    refused(&reply, StatusCode::FORBIDDEN);
    // A blocked author cannot be replied to.
    w.ana.browser.post_empty("/users/bo/block").await.ok();
    let reply = w
        .bo_api
        .post_json(
            "/api/v1/statuses",
            &json!({"status": "reply", "in_reply_to_id": id}),
        )
        .await;
    assert!(
        matches!(reply.status, StatusCode::FORBIDDEN | StatusCode::NOT_FOUND),
        "{}",
        String::from_utf8_lossy(&reply.bytes)
    );
}

// ------------------------------------------------------------------ reading one status

#[sqlx::test(migrations = "./migrations")]
async fn a_status_is_hidden_from_those_who_may_not_see_it(db: PgPool) {
    let mut w = world(db).await;
    let status = post_status(&mut w.ana_api, "secret").await;
    let id = status["id"].as_str().expect("id").to_owned();
    let path = format!("/api/v1/statuses/{id}");

    // Public: bo reads it with viewer state; anonymous readers keep the M4 answer.
    let seen = w.bo_api.get(&path).await.ok();
    assert_eq!(seen["id"], id.as_str());
    assert_eq!(seen["favourited"], false);
    let mut anonymous = w.env.app.browser();
    let anon = anonymous
        .send_raw(Method::GET, &path, &[], axum::body::Body::empty())
        .await
        .ok();
    assert_eq!(anon["id"], id.as_str());

    // Followers-only: bo follows ana, cy does not.
    make_followers_only(&w.db, &id).await;
    assert_eq!(w.bo_api.get(&path).await.status, StatusCode::OK);
    let mut cy_api = sign_in(&w.env.app, &mut w.cy, "read write", [203, 0, 113, 13]).await;
    refused(&cy_api.get(&path).await, StatusCode::NOT_FOUND);
    refused(
        &cy_api.get(&format!("{path}/context")).await,
        StatusCode::NOT_FOUND,
    );
    refused(
        &cy_api.post(&format!("{path}/favourite")).await,
        StatusCode::NOT_FOUND,
    );
    refused(
        &cy_api.post(&format!("{path}/reblog")).await,
        StatusCode::NOT_FOUND,
    );
    refused(
        &cy_api.get(&format!("{path}/favourited_by")).await,
        StatusCode::NOT_FOUND,
    );
    // Anonymous readers do not see it either.
    refused(
        &anonymous
            .send_raw(Method::GET, &path, &[], axum::body::Body::empty())
            .await,
        StatusCode::NOT_FOUND,
    );

    // Public again, but bo is blocked by ana.
    sqlx::query("update posts set visibility = 'public' where id = $1")
        .bind(id.parse::<Uuid>().expect("uuid"))
        .execute(&w.db)
        .await
        .expect("visibility");
    w.ana.browser.post_empty("/users/bo/block").await.ok();
    refused(&w.bo_api.get(&path).await, StatusCode::NOT_FOUND);
    refused(
        &w.bo_api.get(&format!("{path}/context")).await,
        StatusCode::NOT_FOUND,
    );
    // Unknown and malformed ids.
    refused(
        &w.bo_api
            .get("/api/v1/statuses/00000000-0000-7000-8000-000000000000")
            .await,
        StatusCode::NOT_FOUND,
    );
    refused(
        &w.bo_api.get("/api/v1/statuses/nope").await,
        StatusCode::NOT_FOUND,
    );
}

// ------------------------------------------------------------------ edit, delete, history

#[sqlx::test(migrations = "./migrations")]
async fn a_status_can_be_edited_and_deleted_by_its_author_only(db: PgPool) {
    let mut w = world(db).await;
    let media = w.ana_api.upload("image/png", png(20, 20), "").await.ok();
    let status = w
        .ana_api
        .post_json(
            "/api/v1/statuses",
            &json!({"status": "first", "media_ids": [media["id"]]}),
        )
        .await
        .ok();
    let path = format!("/api/v1/statuses/{}", status["id"].as_str().expect("id"));

    let edited = w
        .ana_api
        .put_json(
            &path,
            &json!({"status": "second", "spoiler_text": "cw", "media_ids": [media["id"]], "sensitive": true}),
        )
        .await
        .ok();
    assert_eq!(edited["id"], status["id"]);
    assert!(
        edited["content"]
            .as_str()
            .expect("content")
            .contains("second")
    );
    assert!(edited["edited_at"].is_string());
    assert_eq!(edited["spoiler_text"], "cw");

    let source = w.ana_api.get(&format!("{path}/source")).await.ok();
    assert_eq!(source["text"], "second");
    assert_eq!(source["spoiler_text"], "cw");
    let history = w.ana_api.get(&format!("{path}/history")).await.ok();
    let history = history.as_array().expect("history");
    assert_eq!(history.len(), 2);
    assert!(history[0]["content"].as_str().expect("c").contains("first"));
    assert!(
        history[1]["content"]
            .as_str()
            .expect("c")
            .contains("second")
    );

    // Tuita cannot change media or the poll while editing.
    let reply = w
        .ana_api
        .put_json(
            &path,
            &json!({"status": "third", "media_ids": [Uuid::now_v7()]}),
        )
        .await;
    refused(&reply, StatusCode::UNPROCESSABLE_ENTITY);

    // Someone else may read it but not change or delete it.
    refused(
        &w.bo_api.put_json(&path, &json!({"status": "mine"})).await,
        StatusCode::FORBIDDEN,
    );
    refused(&w.bo_api.delete(&path).await, StatusCode::FORBIDDEN);
    refused(
        &w.bo_api.get(&format!("{path}/source")).await,
        StatusCode::FORBIDDEN,
    );

    let deleted = w.ana_api.delete(&path).await.ok();
    assert_eq!(deleted["id"], status["id"]);
    assert_eq!(deleted["text"], "second");
    refused(&w.ana_api.get(&path).await, StatusCode::NOT_FOUND);
    refused(&w.ana_api.delete(&path).await, StatusCode::NOT_FOUND);
}

// ------------------------------------------------------------------ context

#[sqlx::test(migrations = "./migrations")]
async fn the_context_lists_ancestors_and_every_reply_below_in_order(db: PgPool) {
    let mut w = world(db).await;
    let a = post_status(&mut w.ana_api, "a").await;
    let reply = |text: &str, to: &Value| json!({"status": text, "in_reply_to_id": to["id"]});
    let b = w
        .bo_api
        .post_json("/api/v1/statuses", &reply("b", &a))
        .await
        .ok();
    let c = w
        .ana_api
        .post_json("/api/v1/statuses", &reply("c", &b))
        .await
        .ok();
    let d = w
        .bo_api
        .post_json("/api/v1/statuses", &reply("d", &c))
        .await
        .ok();
    let e = w
        .ana_api
        .post_json("/api/v1/statuses", &reply("e", &a))
        .await
        .ok();
    assert_eq!(b["in_reply_to_id"], a["id"]);
    assert_eq!(b["in_reply_to_account_id"], a["account"]["id"]);

    let context = w
        .ana_api
        .get(&format!(
            "/api/v1/statuses/{}/context",
            c["id"].as_str().expect("id")
        ))
        .await
        .ok();
    assert_eq!(ids(&context["ancestors"]), [sid(&a), sid(&b)]);
    assert_eq!(ids(&context["descendants"]), [sid(&d)]);

    let root = w
        .bo_api
        .get(&format!(
            "/api/v1/statuses/{}/context",
            a["id"].as_str().expect("id")
        ))
        .await
        .ok();
    assert_eq!(root["ancestors"], json!([]));
    let mut below = ids(&root["descendants"]);
    below.sort();
    let mut want = vec![
        sid(&b).to_owned(),
        sid(&c).to_owned(),
        sid(&d).to_owned(),
        sid(&e).to_owned(),
    ];
    want.sort();
    assert_eq!(below, want);
    assert_eq!(ids(&root["descendants"]), want, "oldest first");

    // Blocks are the viewer's: a stranger still sees every reply, ana, who blocks bo, loses bo's
    // reply and everything below it (her own reply to it included).
    w.ana.browser.post_empty("/users/bo/block").await.ok();
    let mut dee = w.env.app.person(&w.db, "dee").await;
    let mut dee_api = sign_in(&w.env.app, &mut dee, "read", [203, 0, 113, 14]).await;
    let context = dee_api
        .get(&format!(
            "/api/v1/statuses/{}/context",
            a["id"].as_str().expect("id")
        ))
        .await
        .ok();
    assert_eq!(ids(&context["descendants"]).len(), 4);
    let context = w
        .ana_api
        .get(&format!(
            "/api/v1/statuses/{}/context",
            a["id"].as_str().expect("id")
        ))
        .await
        .ok();
    assert_eq!(ids(&context["descendants"]), [sid(&e)]);
}

// ------------------------------------------------------------------ actions

#[sqlx::test(migrations = "./migrations")]
async fn favourite_reblog_bookmark_and_pin_change_the_viewer_state(db: PgPool) {
    let mut w = world(db).await;
    let status = post_status(&mut w.ana_api, "likable").await;
    let id = status["id"].as_str().expect("id").to_owned();
    let path = format!("/api/v1/statuses/{id}");

    let liked = w.bo_api.post(&format!("{path}/favourite")).await.ok();
    assert_eq!(
        (
            liked["favourited"].clone(),
            liked["favourites_count"].clone()
        ),
        (json!(true), json!(1))
    );
    // Repeating is harmless.
    let again = w.bo_api.post(&format!("{path}/favourite")).await.ok();
    assert_eq!(again["favourites_count"], 1);
    let by = w.ana_api.get(&format!("{path}/favourited_by")).await.ok();
    assert_eq!(by[0]["acct"], "bo");
    assert!(by[0].get("followers_count").is_some());
    let unliked = w.bo_api.post(&format!("{path}/unfavourite")).await.ok();
    assert_eq!(
        (
            unliked["favourited"].clone(),
            unliked["favourites_count"].clone()
        ),
        (json!(false), json!(0))
    );
    assert_eq!(
        w.ana_api.get(&format!("{path}/favourited_by")).await.ok(),
        json!([])
    );

    // A boost answers with a wrapper that holds the status; unboost answers with the status.
    let boost = w.bo_api.post(&format!("{path}/reblog")).await.ok();
    assert_ne!(boost["id"], status["id"]);
    assert_eq!(boost["reblog"]["id"], status["id"]);
    assert_eq!(boost["account"]["acct"], "bo");
    assert_eq!(boost["reblogged"], true);
    assert_eq!(boost["content"], "");
    let reblogged_by = w.ana_api.get(&format!("{path}/reblogged_by")).await.ok();
    assert_eq!(reblogged_by[0]["acct"], "bo");
    let home = w.ana_api.get("/api/v1/timelines/home").await.ok();
    assert_eq!(
        home[0]["id"], boost["id"],
        "the boost is in the home timeline of a follower"
    );
    let unboost = w.bo_api.post(&format!("{path}/unreblog")).await.ok();
    assert_eq!(unboost["id"], status["id"]);
    assert_eq!(unboost["reblogged"], false);
    assert_eq!(unboost["reblogs_count"], 0);

    // Bookmarks.
    let bookmarked = w.bo_api.post(&format!("{path}/bookmark")).await.ok();
    assert_eq!(bookmarked["bookmarked"], true);
    let list = w.bo_api.get("/api/v1/bookmarks").await.ok();
    assert_eq!(ids(&list), [id.as_str()]);
    assert_eq!(
        w.bo_api.post(&format!("{path}/unbookmark")).await.ok()["bookmarked"],
        false
    );
    assert_eq!(w.bo_api.get("/api/v1/bookmarks").await.ok(), json!([]));

    // Pins: only the author's own statuses.
    refused(
        &w.bo_api.post(&format!("{path}/pin")).await,
        StatusCode::UNPROCESSABLE_ENTITY,
    );
    assert_eq!(
        w.ana_api.post(&format!("{path}/pin")).await.ok()["pinned"],
        true
    );
    assert_eq!(w.ana_api.get(&path).await.ok()["pinned"], true);
    assert_eq!(
        w.ana_api.post(&format!("{path}/unpin")).await.ok()["pinned"],
        false
    );

    // A followers-only status cannot be boosted.
    make_followers_only(&w.db, &id).await;
    refused(
        &w.bo_api.post(&format!("{path}/reblog")).await,
        StatusCode::UNPROCESSABLE_ENTITY,
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn favourites_and_bookmarks_page_by_their_own_record(db: PgPool) {
    let mut w = world(db).await;
    let mut posted = Vec::new();
    for n in 0..5 {
        let status = post_status(&mut w.ana_api, &format!("post {n}")).await;
        let id = status["id"].as_str().expect("id").to_owned();
        w.bo_api
            .post(&format!("/api/v1/statuses/{id}/favourite"))
            .await
            .ok();
        w.bo_api
            .post(&format!("/api/v1/statuses/{id}/bookmark"))
            .await
            .ok();
        posted.push(id);
    }
    posted.reverse();
    for path in ["/api/v1/favourites", "/api/v1/bookmarks"] {
        let one = w.bo_api.get(&format!("{path}?limit=2")).await;
        assert_eq!(ids(&one.ok()), posted[..2]);
        let next = link(&one, "next").expect("next link");
        let two = w.bo_api.get(&next).await;
        assert_eq!(ids(&two.ok()), posted[2..4]);
        let next = link(&two, "next").expect("next link");
        let three = w.bo_api.get(&next).await;
        assert_eq!(ids(&three.ok()), posted[4..]);
        assert_eq!(link(&three, "next"), None);
        assert!(
            w.bo_api.get(&format!("{path}?max_id=nope")).await.status == StatusCode::BAD_REQUEST
        );
    }
}

// ------------------------------------------------------------------ timelines

#[sqlx::test(migrations = "./migrations")]
async fn the_home_timeline_pages_with_max_min_and_since_ids(db: PgPool) {
    let mut w = world(db).await;
    let mut posted = Vec::new();
    for n in 0..5 {
        let status = post_status(&mut w.bo_api, &format!("post {n}")).await;
        posted.push(status["id"].as_str().expect("id").to_owned());
    }
    // Newest first.
    let mut newest_first = posted.clone();
    newest_first.reverse();

    let first = w.ana_api.get("/api/v1/timelines/home?limit=2").await;
    assert_eq!(ids(&first.ok()), newest_first[..2]);
    let next = link(&first, "next").expect("next");
    assert!(next.contains("max_id="), "{next}");
    assert!(next.contains("limit=2"), "{next}");
    let prev = link(&first, "prev").expect("prev");
    assert!(prev.contains("min_id="), "{prev}");
    assert!(
        first
            .header("link")
            .contains("https://social.example/api/v1/timelines/home?")
    );

    let second = w.ana_api.get(&next).await;
    assert_eq!(ids(&second.ok()), newest_first[2..4]);
    let last = w.ana_api.get(&link(&second, "next").expect("next")).await;
    assert_eq!(ids(&last.ok()), newest_first[4..]);
    assert_eq!(link(&last, "next"), None);

    // `prev` of the first page reads what is newer; a new post shows up there.
    let fresh = post_status(&mut w.bo_api, "fresh").await;
    let newer = w.ana_api.get(&prev).await;
    assert_eq!(ids(&newer.ok()), [sid(&fresh)]);
    // `min_id` returns the rows right above the id, newest first.
    let above = w
        .ana_api
        .get(&format!(
            "/api/v1/timelines/home?min_id={}&limit=2",
            posted[0]
        ))
        .await
        .ok();
    assert_eq!(ids(&above), [posted[2].as_str(), posted[1].as_str()]);
    // `since_id` returns the newest rows above the id.
    let since = w
        .ana_api
        .get(&format!(
            "/api/v1/timelines/home?since_id={}&limit=2",
            posted[0]
        ))
        .await
        .ok();
    assert_eq!(ids(&since), [sid(&fresh), posted[4].as_str()]);
    // An unfollowed account's posts stay out.
    let mut cy = w.cy;
    let mut cy_api = sign_in(&w.env.app, &mut cy, "read write", [203, 0, 113, 15]).await;
    post_status(&mut cy_api, "stranger").await;
    let home = w.ana_api.get("/api/v1/timelines/home?limit=40").await.ok();
    assert_eq!(home.as_array().expect("array").len(), 6);
    // A muted account's posts are hidden from home.
    w.ana.browser.post_empty("/users/bo/mute").await.ok();
    assert_eq!(
        w.ana_api.get("/api/v1/timelines/home").await.ok(),
        json!([])
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn the_public_tag_and_list_timelines_filter_by_scope(db: PgPool) {
    let mut w = world(db).await;
    let plain = post_status(&mut w.bo_api, "plain").await;
    let tagged = post_status(&mut w.bo_api, "with #Fediverse tag").await;
    let media = w.bo_api.upload("image/png", png(16, 16), "alt").await.ok();
    let with_media = w
        .bo_api
        .post_json(
            "/api/v1/statuses",
            &json!({"status": "pic", "media_ids": [media["id"]]}),
        )
        .await
        .ok();
    let mut cy_api = sign_in(&w.env.app, &mut w.cy, "read write", [203, 0, 113, 16]).await;
    post_status(&mut cy_api, "unrelated").await;

    let public = w.ana_api.get("/api/v1/timelines/public").await.ok();
    assert_eq!(ids(&public).len(), 4);
    let local = w
        .ana_api
        .get("/api/v1/timelines/public?local=true")
        .await
        .ok();
    assert_eq!(ids(&local).len(), 4);
    let remote = w
        .ana_api
        .get("/api/v1/timelines/public?remote=true")
        .await
        .ok();
    assert_eq!(remote, json!([]));
    let only_media = w
        .ana_api
        .get("/api/v1/timelines/public?only_media=true")
        .await
        .ok();
    assert_eq!(ids(&only_media), [sid(&with_media)]);

    let tag = w.ana_api.get("/api/v1/timelines/tag/fediverse").await.ok();
    assert_eq!(ids(&tag), [sid(&tagged)]);
    let tag = w
        .ana_api
        .get("/api/v1/timelines/tag/%23FEDIVERSE")
        .await
        .ok();
    assert_eq!(ids(&tag), [sid(&tagged)]);
    assert_eq!(
        w.ana_api.get("/api/v1/timelines/tag/nothing").await.ok(),
        json!([])
    );

    // A list holds bo; only bo's posts show.
    let list = w
        .ana
        .browser
        .post("/lists", json!({"name": "Pals"}))
        .await
        .ok();
    let list_id = list["id"].as_str().expect("list id").to_owned();
    w.ana
        .browser
        .post_empty(&format!("/lists/{list_id}/members/bo"))
        .await
        .ok();
    let items = w
        .ana_api
        .get(&format!("/api/v1/timelines/list/{list_id}"))
        .await
        .ok();
    assert_eq!(ids(&items).len(), 3);
    assert!(ids(&items).contains(&sid(&plain).to_owned()));
    // Another account's list is not readable.
    refused(
        &w.bo_api
            .get(&format!("/api/v1/timelines/list/{list_id}"))
            .await,
        StatusCode::NOT_FOUND,
    );
    refused(
        &w.ana_api.get("/api/v1/timelines/list/nope").await,
        StatusCode::NOT_FOUND,
    );
}

// ------------------------------------------------------------------ notifications

#[sqlx::test(migrations = "./migrations")]
async fn notifications_filter_by_type_page_and_can_be_dismissed(db: PgPool) {
    let mut w = world(db).await;
    let status = post_status(&mut w.ana_api, "popular").await;
    let id = status["id"].as_str().expect("id").to_owned();
    w.bo_api
        .post(&format!("/api/v1/statuses/{id}/favourite"))
        .await
        .ok();
    w.bo_api
        .post(&format!("/api/v1/statuses/{id}/reblog"))
        .await
        .ok();
    w.bo_api
        .post_json(
            "/api/v1/statuses",
            &json!({"status": "@ana a reply", "in_reply_to_id": id}),
        )
        .await
        .ok();

    let all = w.ana_api.get("/api/v1/notifications").await.ok();
    let kinds: Vec<&str> = all
        .as_array()
        .expect("array")
        .iter()
        .map(|n| n["type"].as_str().expect("type"))
        .collect();
    // Newest first: the reply (a mention), the boost, the favourite, the follow.
    assert_eq!(kinds, ["mention", "reblog", "favourite", "follow"]);
    for note in all.as_array().expect("array") {
        for key in ["id", "type", "created_at", "account"] {
            assert!(note.get(key).is_some(), "{key}");
        }
        assert_eq!(note["account"]["acct"], "bo");
    }
    assert!(all[3]["status"].is_null());
    assert_eq!(all[1]["status"]["id"], id.as_str());

    let only = w
        .ana_api
        .get("/api/v1/notifications?types[]=favourite&types[]=follow")
        .await
        .ok();
    assert_eq!(only.as_array().expect("array").len(), 2);
    let without = w
        .ana_api
        .get("/api/v1/notifications?exclude_types[]=follow&exclude_types[]=mention")
        .await
        .ok();
    assert_eq!(without.as_array().expect("array").len(), 2);
    let none = w
        .ana_api
        .get("/api/v1/notifications?types[]=status")
        .await
        .ok();
    assert_eq!(none, json!([]));

    let first = w.ana_api.get("/api/v1/notifications?limit=3").await;
    assert_eq!(ids(&first.ok()).len(), 3);
    let rest = w
        .ana_api
        .get(&link(&first, "next").expect("next"))
        .await
        .ok();
    assert_eq!(ids(&rest), [ids(&all)[3].as_str()]);

    let one = w
        .ana_api
        .get(&format!("/api/v1/notifications/{}", ids(&all)[2]))
        .await
        .ok();
    assert_eq!(one["type"], "favourite");
    // Someone else's notification is a 404, for show and dismiss alike.
    refused(
        &w.bo_api
            .get(&format!("/api/v1/notifications/{}", ids(&all)[2]))
            .await,
        StatusCode::NOT_FOUND,
    );
    refused(
        &w.bo_api
            .post(&format!("/api/v1/notifications/{}/dismiss", ids(&all)[2]))
            .await,
        StatusCode::NOT_FOUND,
    );
    assert_eq!(
        w.ana_api
            .post(&format!("/api/v1/notifications/{}/dismiss", ids(&all)[2]))
            .await
            .ok(),
        json!({})
    );
    assert_eq!(
        ids(&w.ana_api.get("/api/v1/notifications").await.ok()).len(),
        3
    );
    assert_eq!(
        w.ana_api.post("/api/v1/notifications/clear").await.ok(),
        json!({})
    );
    assert_eq!(w.ana_api.get("/api/v1/notifications").await.ok(), json!([]));
    // Clearing only touched ana's notifications; bo still has the follow.
    assert_eq!(
        w.bo_api
            .get("/api/v1/notifications")
            .await
            .ok()
            .as_array()
            .map(Vec::len),
        Some(1)
    );

    // Tuita's own session keeps the grouped shape on the same path.
    let grouped = w.bo.browser.get("/notifications").await.ok();
    assert!(grouped["items"].is_array(), "{grouped}");
}

// ------------------------------------------------------------------ media

#[sqlx::test(migrations = "./migrations")]
async fn media_uploads_share_tuitas_pipeline_and_can_be_described(db: PgPool) {
    let mut w = world(db).await;
    let media = w
        .ana_api
        .upload("image/png", png(30, 20), "first")
        .await
        .ok();
    for key in [
        "id",
        "type",
        "url",
        "preview_url",
        "remote_url",
        "text_url",
        "meta",
        "description",
        "blurhash",
    ] {
        assert!(media.get(key).is_some(), "missing {key}: {media}");
    }
    assert_eq!(media["type"], "image");
    assert!(
        media["url"]
            .as_str()
            .expect("url")
            .starts_with("https://social.example/media/")
    );
    assert_eq!(media["meta"]["original"]["width"], 30);

    // Not an image.
    let bad = w
        .ana_api
        .upload("image/png", b"not an image".to_vec(), "")
        .await;
    assert!(
        bad.status.is_client_error(),
        "{}",
        String::from_utf8_lossy(&bad.bytes)
    );
    // Over the image size limit.
    let big = w
        .ana_api
        .upload("image/png", vec![0; 11 * 1024 * 1024], "")
        .await;
    assert_eq!(big.status, StatusCode::PAYLOAD_TOO_LARGE);
    // A `purpose` field cannot set an avatar through the Mastodon route.
    let (kind, body) = common::multipart(&[
        common::Part::file("file", "image/png", png(10, 10)),
        common::Part::text("purpose", "avatar"),
    ]);
    let reply = w
        .ana_api
        .send(Method::POST, "/api/v2/media", &[], Some((&kind, body)))
        .await;
    reply.ok();
    let avatar: Option<Uuid> = sqlx::query_scalar("select avatar_id from users where id = $1")
        .bind(w.ana.id)
        .fetch_one(&w.db)
        .await
        .expect("avatar");
    assert_eq!(avatar, None);

    let path = format!("/api/v1/media/{}", media["id"].as_str().expect("id"));
    let updated = w
        .ana_api
        .put_json(&path, &json!({"description": "second"}))
        .await
        .ok();
    assert_eq!(updated["description"], "second");
    // Other accounts' uploads are not theirs to change or attach.
    refused(
        &w.bo_api
            .put_json(&path, &json!({"description": "mine"}))
            .await,
        StatusCode::NOT_FOUND,
    );
    let theirs = w
        .bo_api
        .post_json(
            "/api/v1/statuses",
            &json!({"status": "x", "media_ids": [media["id"]]}),
        )
        .await;
    refused(&theirs, StatusCode::UNPROCESSABLE_ENTITY);
    // A scope refusal: no token, no upload.
    let mut anonymous = w.env.app.browser();
    let anon = anonymous
        .send_raw(
            Method::POST,
            "/api/v2/media",
            &[],
            axum::body::Body::empty(),
        )
        .await;
    assert_eq!(anon.status, StatusCode::UNAUTHORIZED);
}

// ------------------------------------------------------------------ polls

#[sqlx::test(migrations = "./migrations")]
async fn polls_can_be_read_and_voted_in_once(db: PgPool) {
    let mut w = world(db).await;
    let status = w
        .ana_api
        .post_form(
            "/api/v1/statuses",
            &[
                ("status", "Which one?"),
                ("poll[options][]", "red"),
                ("poll[options][]", "blue"),
                ("poll[expires_in]", "86400"),
                ("poll[multiple]", "false"),
            ],
        )
        .await
        .ok();
    let poll = &status["poll"];
    for key in [
        "id",
        "expires_at",
        "expired",
        "multiple",
        "votes_count",
        "voters_count",
        "voted",
        "own_votes",
        "options",
        "emojis",
    ] {
        assert!(poll.get(key).is_some(), "missing {key}: {poll}");
    }
    assert_eq!(poll["options"][0]["title"], "red");
    assert_eq!(poll["expired"], false);
    assert_eq!(poll["voted"], false);
    let id = poll["id"].as_str().expect("poll id").to_owned();

    let seen = w.bo_api.get(&format!("/api/v1/polls/{id}")).await.ok();
    assert_eq!(seen["votes_count"], 0);
    let voted = w
        .bo_api
        .post_json(
            &format!("/api/v1/polls/{id}/votes"),
            &json!({"choices": [1]}),
        )
        .await
        .ok();
    assert_eq!(voted["voted"], true);
    assert_eq!(voted["own_votes"], json!([1]));
    assert_eq!(voted["votes_count"], 1);
    assert_eq!(voted["options"][1]["votes_count"], 1);
    refused(
        &w.bo_api
            .post_form(&format!("/api/v1/polls/{id}/votes"), &[("choices[]", "0")])
            .await,
        StatusCode::UNPROCESSABLE_ENTITY,
    );
    // The status shows the viewer's vote.
    let again = w
        .bo_api
        .get(&format!(
            "/api/v1/statuses/{}",
            status["id"].as_str().expect("id")
        ))
        .await
        .ok();
    assert_eq!(again["poll"]["own_votes"], json!([1]));

    // Bad choices, and a poll nobody may see.
    refused(
        &w.ana_api
            .post_json(
                &format!("/api/v1/polls/{id}/votes"),
                &json!({"choices": [7]}),
            )
            .await,
        StatusCode::UNPROCESSABLE_ENTITY,
    );
    refused(
        &w.ana_api
            .post_json(
                &format!("/api/v1/polls/{id}/votes"),
                &json!({"choices": []}),
            )
            .await,
        StatusCode::UNPROCESSABLE_ENTITY,
    );
    let mut cy = w.cy;
    let mut cy_api = sign_in(&w.env.app, &mut cy, "read write", [203, 0, 113, 17]).await;
    make_followers_only(&w.db, &id).await;
    refused(
        &cy_api.get(&format!("/api/v1/polls/{id}")).await,
        StatusCode::NOT_FOUND,
    );
    refused(
        &cy_api
            .post_json(
                &format!("/api/v1/polls/{id}/votes"),
                &json!({"choices": [0]}),
            )
            .await,
        StatusCode::NOT_FOUND,
    );
    refused(
        &w.bo_api.get("/api/v1/polls/nope").await,
        StatusCode::NOT_FOUND,
    );
}

// ------------------------------------------------------------------ conversations

#[sqlx::test(migrations = "./migrations")]
async fn direct_messages_are_conversations_with_a_direct_last_status(db: PgPool) {
    let mut w = world(db).await;
    w.bo.browser
        .post("/conversations/ana", json!({"text": "psst"}))
        .await
        .ok();
    let list = w.ana_api.get("/api/v1/conversations").await.ok();
    let conversation = &list[0];
    for key in ["id", "unread", "accounts", "last_status"] {
        assert!(conversation.get(key).is_some(), "missing {key}");
    }
    assert_eq!(conversation["unread"], true);
    assert_eq!(conversation["accounts"][0]["acct"], "bo");
    assert_eq!(conversation["last_status"]["visibility"], "direct");
    assert!(
        conversation["last_status"]["content"]
            .as_str()
            .expect("content")
            .contains("psst")
    );
    let id = conversation["id"].as_str().expect("id").to_owned();

    // Tuita's own session keeps its shape on the same path.
    let own = w.ana.browser.get("/conversations").await.ok();
    assert!(own["items"].is_array(), "{own}");

    let read = w
        .ana_api
        .post(&format!("/api/v1/conversations/{id}/read"))
        .await
        .ok();
    assert_eq!(read["unread"], false);
    assert_eq!(
        w.ana_api.get("/api/v1/conversations").await.ok()[0]["unread"],
        false
    );
    refused(
        &w.ana_api
            .post(&format!("/api/v1/conversations/{}/read", w.cy.id))
            .await,
        StatusCode::NOT_FOUND,
    );

    // Deleting removes the viewer's own messages; the other side's stay.
    w.ana
        .browser
        .post("/conversations/bo", json!({"text": "hush"}))
        .await
        .ok();
    assert_eq!(
        w.ana_api
            .delete(&format!("/api/v1/conversations/{id}"))
            .await
            .ok(),
        json!({})
    );
    let after = w.ana_api.get("/api/v1/conversations").await.ok();
    assert!(
        after[0]["last_status"]["content"]
            .as_str()
            .expect("content")
            .contains("psst")
    );
    refused(
        &w.ana_api
            .delete(&format!("/api/v1/conversations/{}", w.cy.id))
            .await,
        StatusCode::NOT_FOUND,
    );

    // Pages by max_id.
    for n in 0..3 {
        let mut other = w.env.app.person(&w.db, &format!("zed{n}")).await;
        other
            .browser
            .post("/conversations/ana", json!({"text": format!("hello {n}")}))
            .await
            .ok();
    }
    let first = w.ana_api.get("/api/v1/conversations?limit=2").await;
    assert_eq!(ids(&first.ok()).len(), 2);
    let second = w
        .ana_api
        .get(&link(&first, "next").expect("next"))
        .await
        .ok();
    assert_eq!(second.as_array().expect("array").len(), 2);
}

// ------------------------------------------------------------------ scheduled statuses

#[sqlx::test(migrations = "./migrations")]
async fn a_scheduled_status_is_held_until_its_time(db: PgPool) {
    let mut w = world(db).await;
    let at = time::OffsetDateTime::now_utc() + time::Duration::hours(2);
    let at = at
        .format(&time::format_description::well_known::Rfc3339)
        .expect("format");
    let scheduled = w
        .ana_api
        .post_json(
            "/api/v1/statuses",
            &json!({"status": "later", "scheduled_at": at}),
        )
        .await
        .ok();
    for key in ["id", "scheduled_at", "params", "media_attachments"] {
        assert!(scheduled.get(key).is_some(), "missing {key}");
    }
    assert_eq!(scheduled["params"]["text"], "later");
    assert_eq!(w.bo_api.get("/api/v1/timelines/home").await.ok(), json!([]));
    let list = w.ana_api.get("/api/v1/scheduled_statuses").await.ok();
    assert_eq!(ids(&list), [sid(&scheduled)]);
    let too_soon = (time::OffsetDateTime::now_utc() + time::Duration::seconds(5))
        .format(&time::format_description::well_known::Rfc3339)
        .expect("format");
    refused(
        &w.ana_api
            .post_json(
                "/api/v1/statuses",
                &json!({"status": "now", "scheduled_at": too_soon}),
            )
            .await,
        StatusCode::UNPROCESSABLE_ENTITY,
    );
    assert_eq!(
        w.ana_api
            .delete(&format!("/api/v1/scheduled_statuses/{}", sid(&scheduled)))
            .await
            .ok(),
        json!({})
    );
    assert_eq!(
        w.ana_api.get("/api/v1/scheduled_statuses").await.ok(),
        json!([])
    );
}

fn sid(value: &Value) -> &str {
    value["id"].as_str().expect("id")
}
