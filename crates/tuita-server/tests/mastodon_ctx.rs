//! The Mastodon mappers fed with what the real API and database give: posts as the timeline
//! loader builds them, accounts with their counters, and the lookups `Ctx` makes in batches.

mod common;

use serde_json::{Value, json};
use sqlx::PgPool;
use tuita_server::db::{posts, users};
use tuita_server::dto::PostDto;
use tuita_server::mastodon::ctx::Ctx;
use tuita_server::mastodon::dto::{self, account_from_ref, iso, statuses};
use uuid::Uuid;

fn json_of(value: &impl serde::Serialize) -> Value {
    serde_json::to_value(value).expect("serializes")
}

fn page_posts(page: &Value) -> Vec<PostDto> {
    page["items"]
        .as_array()
        .expect("items")
        .iter()
        .map(|item| serde_json::from_value(item.clone()).expect("a PostDto"))
        .collect()
}

#[sqlx::test(migrations = "./migrations")]
async fn real_posts_map_with_mentions_the_reply_author_counters_and_boosts(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut ben = app.person(&db, "ben").await;
    ana.browser.post_empty("/users/ben/follow").await.ok();

    let original = ana
        .browser
        .post_text("hello @ben #Rust https://x.example/a")
        .await;
    let original_id: Uuid = original["id"].as_str().expect("id").parse().expect("uuid");
    let reply = ben
        .browser
        .post(
            "/posts",
            json!({"text": "hi @ana", "replyTo": original_id, "cw": "warning"}),
        )
        .await
        .ok();
    let reply_id: Uuid = reply["id"].as_str().expect("id").parse().expect("uuid");
    ben.browser
        .post_empty(&format!("/posts/{original_id}/repost"))
        .await
        .ok();
    ana.browser
        .post_empty(&format!("/posts/{original_id}/like"))
        .await
        .ok();

    // Ana's following timeline has ben's boost of her post; replies are not on it, so the
    // reply is loaded by id.
    let page = ana.browser.get("/timelines/following").await.ok();
    let mut timeline = page_posts(&page);
    assert!(
        timeline.iter().any(|post| post.reposted_by.is_some()),
        "the timeline has ben's boost"
    );
    timeline.extend(
        posts::load(&db, ana.id, &[reply_id], None)
            .await
            .expect("the reply loads"),
    );
    let ctx = Ctx::for_posts(&app.state, &timeline)
        .await
        .expect("context loads");
    let mapped = statuses(&timeline, &ctx);
    let value = json_of(&mapped);

    let reply_status = value
        .as_array()
        .expect("array")
        .iter()
        .find(|status| status["id"] == reply_id.to_string())
        .expect("the reply");
    assert_eq!(reply_status["in_reply_to_id"], original_id.to_string());
    assert_eq!(
        reply_status["in_reply_to_account_id"],
        ana.id.to_string(),
        "the author of the parent comes from the context"
    );
    assert_eq!(reply_status["spoiler_text"], "warning");
    assert_eq!(reply_status["sensitive"], true);
    assert_eq!(reply_status["account"]["acct"], "ben");
    // Ben follows nobody, ana follows ben, he wrote one reply: profile counters, not zeros.
    assert_eq!(reply_status["account"]["followers_count"], 1);
    assert_eq!(reply_status["account"]["following_count"], 0);
    assert_eq!(reply_status["account"]["statuses_count"], 1);
    assert_eq!(
        reply_status["account"]["last_status_at"],
        iso(timeline
            .iter()
            .find(|post| post.id == reply_id)
            .expect("reply")
            .created_at)[..10]
    );
    let content = reply_status["content"].as_str().expect("html");
    assert!(
        content.contains("<a href=\"https://social.example/@ana\" class=\"u-url mention\""),
        "{content}"
    );
    assert_eq!(
        reply_status["mentions"],
        json!([{
            "id": ana.id.to_string(),
            "username": "ana",
            "url": "https://social.example/@ana",
            "acct": "ana",
        }])
    );

    // The boost wraps ana's own post, which carries its mentions, tags, counters and her likes.
    let boost = value
        .as_array()
        .expect("array")
        .iter()
        .find(|status| status["reblog"].is_object())
        .expect("the boost");
    assert_eq!(boost["account"]["acct"], "ben");
    assert_eq!(boost["content"], "");
    let inner = &boost["reblog"];
    assert_eq!(inner["id"], original_id.to_string());
    assert_eq!(inner["account"]["acct"], "ana");
    assert_eq!(inner["favourites_count"], 1);
    assert_eq!(inner["reblogs_count"], 1);
    assert_eq!(inner["replies_count"], 1);
    assert_eq!(inner["favourited"], true);
    assert_eq!(inner["reblogged"], false);
    assert_eq!(
        inner["tags"],
        json!([{"name": "rust", "url": "https://social.example/tags/rust"}])
    );
    assert_eq!(inner["mentions"][0]["acct"], "ben");
    assert!(
        inner["content"]
            .as_str()
            .expect("html")
            .contains("class=\"mention hashtag\"")
    );
    assert_eq!(
        inner["uri"],
        format!("https://social.example/users/ana/statuses/{original_id}")
    );
    assert_eq!(
        boost["id"],
        timeline
            .iter()
            .find(|post| post.reposted_by.is_some())
            .expect("entry")
            .entry_id
            .to_string()
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_remote_account_and_its_post_map_to_user_at_domain(db: PgPool) {
    let app = common::app(db.clone());
    let ana = app.person(&db, "ana").await;
    let remote = Uuid::now_v7();
    sqlx::query(
        "insert into users (id, handle, domain, display_name, ap_id, url, manually_approves, posts_count)
         values ($1, 'carol', 'far.example', 'Carol', 'https://far.example/users/carol',
                 'https://far.example/@carol', true, 1)",
    )
    .bind(remote)
    .execute(&db)
    .await
    .expect("remote user");
    let post = Uuid::now_v7();
    sqlx::query(
        "insert into posts (id, author_id, text, url, ap_id, visibility)
         values ($1, $2, 'from afar @ana', 'https://far.example/@carol/9',
                 'https://far.example/users/carol/statuses/9', 'public')",
    )
    .bind(post)
    .bind(remote)
    .execute(&db)
    .await
    .expect("remote post");
    sqlx::query("insert into mentions (post_id, user_id) values ($1, $2)")
        .bind(post)
        .bind(ana.id)
        .execute(&db)
        .await
        .expect("mention");

    let loaded = posts::load(&db, ana.id, &[post], None).await.expect("load");
    assert_eq!(loaded.len(), 1);
    let ctx = Ctx::for_posts(&app.state, &loaded).await.expect("context");
    let status = json_of(&dto::status(&loaded[0], &ctx));
    assert_eq!(status["account"]["acct"], "carol@far.example");
    assert_eq!(status["account"]["username"], "carol");
    assert_eq!(status["account"]["url"], "https://far.example/@carol");
    assert_eq!(status["account"]["uri"], "https://far.example/users/carol");
    assert_eq!(status["account"]["locked"], true);
    assert_eq!(status["account"]["statuses_count"], 1);
    assert_eq!(status["url"], "https://far.example/@carol/9");
    assert_eq!(status["uri"], "https://far.example/@carol/9");
    assert_eq!(status["mentions"][0]["acct"], "ana");

    // An account list needs only the ids: `Ctx::for_users` fills the same facts.
    let card = users::refs(&db, &[remote]).await.expect("refs");
    let ctx = Ctx::for_users(&app.state, &[remote])
        .await
        .expect("context");
    let account = json_of(&account_from_ref(&card[&remote], &ctx));
    assert_eq!(account["acct"], "carol@far.example");
    assert_eq!(account["uri"], "https://far.example/users/carol");
    assert_ne!(account["last_status_at"], Value::Null);
}
