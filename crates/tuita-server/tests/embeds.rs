//! Link previews: `/oembed`, the Mastodon status JSON and the Open Graph head's alternate links.
#![allow(missing_docs)]

mod common;

use axum::body::Body;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use common::{App, app};

const BASE: &str = "https://social.example";

fn id_of(post: &Value) -> String {
    post["id"].as_str().expect("post id").to_owned()
}

async fn exec(db: &PgPool, sql: &str, arg: &str) {
    sqlx::query(sqlx::AssertSqlSafe(sql.to_owned()))
        .bind(arg.to_owned())
        .execute(db)
        .await
        .expect("update");
}

async fn page(app: &App, path: &str) -> String {
    tuita_server::routes::head::head_tags(&app.state, path).await
}

async fn oembed(app: &App, url: &str) -> common::Reply {
    let query: String = url::form_urlencoded::byte_serialize(url.as_bytes()).collect();
    app.browser()
        .get_path(&format!("/oembed?url={query}"))
        .await
}

impl common::Browser {
    /// `GET` of a path outside `/api/v1`.
    async fn get_path(&mut self, path: &str) -> common::Reply {
        let raw = self.send_raw(Method::GET, path, &[], Body::empty()).await;
        common::Reply {
            status: raw.status,
            headers: raw.headers,
            body: String::from_utf8_lossy(&raw.bytes).into_owned(),
        }
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn oembed_gives_an_escaped_blockquote_for_a_public_post(db: PgPool) {
    let app = app(db.clone());
    let mut ada = app.person(&db, "ada").await;
    exec(
        &db,
        "update users set display_name = $1 where handle = 'ada'",
        "Ada <3 \"q\"",
    )
    .await;
    let post = id_of(
        &ada.browser
            .post_text("<script>alert(1)</script> & 'quotes'\nsecond line")
            .await,
    );

    let page_url = format!("{BASE}/@ada/{post}");
    let reply = oembed(&app, &page_url).await;
    assert_eq!(reply.header("access-control-allow-origin"), "*");
    assert_eq!(reply.header("cache-control"), "public, max-age=180");
    let doc = reply.ok();
    let date = doc["html"].as_str().expect("html").to_owned();
    let created: String =
        sqlx::query_scalar("select created_at::date::text from posts where id = $1::uuid")
            .bind(&post)
            .fetch_one(&db)
            .await
            .expect("date");
    assert_eq!(
        doc,
        json!({
            "version": "1.0",
            "type": "rich",
            "provider_name": "tuita",
            "provider_url": BASE,
            "author_name": "Ada <3 \"q\" (@ada)",
            "author_url": format!("{BASE}/@ada"),
            "html": format!(
                "<blockquote class=\"tuita-embed\" cite=\"{page_url}\"><p>&lt;script&gt;alert(1)&lt;/script&gt; &amp; &#39;quotes&#39;<br>second line</p>&mdash; Ada &lt;3 &quot;q&quot; (@ada) <a href=\"{page_url}\">{created}</a></blockquote>"
            ),
            "width": 550,
            "height": null,
            "cache_age": 180,
        })
    );
    assert!(!date.contains("<script"));
    assert!(!date.contains("<iframe"));

    let canonical = oembed(&app, &format!("{BASE}/users/ada/statuses/{post}"))
        .await
        .ok();
    assert_eq!(canonical["html"], doc["html"]);

    let sized = |query: &str| format!("/oembed?url={page_url}&{query}");
    let narrow = app
        .browser()
        .get_path(&sized("maxwidth=100&maxheight=300"))
        .await
        .ok();
    assert_eq!(
        (narrow["width"].clone(), narrow["height"].clone()),
        (json!(200), json!(300))
    );
    let wide = app.browser().get_path(&sized("maxwidth=9999")).await.ok();
    assert_eq!(wide["width"], 550);
    let middle = app.browser().get_path(&sized("maxwidth=320")).await.ok();
    assert_eq!(middle["width"], 320);
}

#[sqlx::test(migrations = "./migrations")]
async fn oembed_shows_the_content_warning_instead_of_the_text(db: PgPool) {
    let app = app(db.clone());
    let mut ada = app.person(&db, "ada").await;
    let post = id_of(
        &ada.browser
            .post("/posts", json!({"text": "hidden", "cw": "spoilers"}))
            .await
            .ok(),
    );
    let doc = oembed(&app, &format!("{BASE}/@ada/{post}")).await.ok();
    let html = doc["html"].as_str().expect("html");
    assert!(html.contains("<p>CW: spoilers</p>"), "{html}");
    assert!(!html.contains("hidden"), "{html}");
}

#[sqlx::test(migrations = "./migrations")]
async fn oembed_refuses_missing_foreign_and_non_public_targets(db: PgPool) {
    let app = app(db.clone());
    let mut ada = app.person(&db, "ada").await;
    let public = id_of(&ada.browser.post_text("public").await);
    let unlisted = id_of(&ada.browser.post_text("unlisted").await);
    let followers = id_of(&ada.browser.post_text("followers").await);
    let deleted = id_of(&ada.browser.post_text("deleted").await);
    exec(
        &db,
        "update posts set visibility = 'unlisted' where id = $1::uuid",
        &unlisted,
    )
    .await;
    exec(
        &db,
        "update posts set visibility = 'followers' where id = $1::uuid",
        &followers,
    )
    .await;
    ada.browser.delete(&format!("/posts/{deleted}")).await.ok();

    let mut anon = app.browser();
    anon.get_path("/oembed")
        .await
        .err(StatusCode::BAD_REQUEST, "url is required.");
    anon.get_path("/oembed?url=")
        .await
        .err(StatusCode::BAD_REQUEST, "url is required.");
    for url in [
        format!("{BASE}/@ada"),
        format!("{BASE}/@ada/{unlisted}"),
        format!("{BASE}/@ada/{followers}"),
        format!("{BASE}/@ada/{deleted}"),
        format!("{BASE}/@ada/{}", Uuid::now_v7()),
        format!("{BASE}/@bob/{public}"),
        format!("https://other.example/@ada/{public}"),
        format!("{BASE}/p/{public}"),
        format!("{BASE}/users/ada/statuses/not-a-uuid"),
        "not a url".to_owned(),
    ] {
        oembed(&app, &url)
            .await
            .err(StatusCode::NOT_FOUND, "Not found.");
    }
    assert_eq!(
        oembed(&app, &format!("{BASE}/@ada/{public}")).await.status,
        StatusCode::OK
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn the_mastodon_status_has_the_fields_embedders_read(db: PgPool) {
    let app = app(db.clone());
    let mut ada = app.person(&db, "ada").await;
    common::insert_user(&db, "bob").await;
    let avatar = Uuid::now_v7();
    sqlx::query(
        "insert into media (id, owner_id, kind, mime, path, bytes, width, height)
         values ($1, $2, 'image', 'image/webp', $3, 10, 4, 3)",
    )
    .bind(avatar)
    .bind(ada.id)
    .bind(format!("{avatar}.webp"))
    .execute(&db)
    .await
    .expect("avatar");
    let image = Uuid::now_v7();
    sqlx::query(
        "insert into media (id, owner_id, kind, mime, path, bytes, width, height, alt)
         values ($1, $2, 'image', 'image/png', $3, 10, 640, 480, 'a cat')",
    )
    .bind(image)
    .bind(ada.id)
    .bind(format!("{image}.png"))
    .execute(&db)
    .await
    .expect("image");
    sqlx::query(
        "update users set avatar_id = $2, bio = 'bio text', display_name = 'Ada' where id = $1",
    )
    .bind(ada.id)
    .bind(avatar)
    .execute(&db)
    .await
    .expect("profile");
    let root = id_of(&ada.browser.post_text("root").await);
    let post = ada
        .browser
        .post(
            "/posts",
            json!({"text": "hi @bob #Rust", "replyTo": root, "mediaIds": [image.to_string()]}),
        )
        .await
        .ok();
    let id = id_of(&post);
    exec(
        &db,
        "update posts set like_count = 3, repost_count = 2 where id = $1::uuid",
        &id,
    )
    .await;

    let reply = app.browser().get(&format!("/statuses/{id}")).await;
    assert_eq!(reply.header("access-control-allow-origin"), "*");
    assert_eq!(reply.header("cache-control"), "public, max-age=60");
    let doc = reply.ok();
    let bob: Uuid = sqlx::query_scalar("select id from users where handle = 'bob'")
        .fetch_one(&db)
        .await
        .expect("bob");
    assert_eq!(doc["id"], id);
    assert_eq!(doc["uri"], format!("{BASE}/users/ada/statuses/{id}"));
    assert_eq!(doc["url"], format!("{BASE}/@ada/{id}"));
    assert_eq!(doc["visibility"], "public");
    assert_eq!(doc["sensitive"], false);
    assert_eq!(doc["spoiler_text"], "");
    assert_eq!(doc["language"], Value::Null);
    assert_eq!(doc["reblog"], Value::Null);
    assert_eq!(doc["application"], Value::Null);
    assert_eq!(doc["poll"], Value::Null);
    assert_eq!(doc["in_reply_to_id"], root);
    assert_eq!(doc["in_reply_to_account_id"], ada.id.to_string());
    assert_eq!(doc["favourites_count"], 3);
    assert_eq!(doc["reblogs_count"], 2);
    assert_eq!(doc["replies_count"], 0);
    assert!(doc["created_at"].as_str().is_some_and(|t| t.ends_with('Z')));
    assert!(
        doc["content"]
            .as_str()
            .is_some_and(|html| html.starts_with("<p>hi <span class=\"h-card\">"))
    );
    assert_eq!(
        doc["mentions"],
        json!([{"id": bob.to_string(), "username": "bob", "url": format!("{BASE}/@bob"), "acct": "bob"}])
    );
    assert_eq!(
        doc["tags"],
        json!([{"name": "rust", "url": format!("{BASE}/tags/rust")}])
    );
    assert_eq!(
        doc["media_attachments"],
        json!([{
            "id": image.to_string(),
            "type": "image",
            "url": format!("{BASE}/media/{image}.png"),
            "preview_url": format!("{BASE}/media/{image}.png"),
            "remote_url": null,
            "text_url": null,
            "description": "a cat",
            "blurhash": null,
            "meta": {
                "original": {"width": 640, "height": 480, "size": "640x480", "aspect": 640.0 / 480.0},
                "small": {"width": 640, "height": 480, "size": "640x480", "aspect": 640.0 / 480.0},
            },
        }])
    );
    let account = &doc["account"];
    assert_eq!(account["id"], ada.id.to_string());
    assert_eq!(account["username"], "ada");
    assert_eq!(account["acct"], "ada");
    assert_eq!(account["display_name"], "Ada");
    assert_eq!(account["url"], format!("{BASE}/@ada"));
    assert_eq!(account["uri"], format!("{BASE}/users/ada"));
    assert_eq!(account["avatar"], format!("{BASE}/media/{avatar}.webp"));
    assert_eq!(account["avatar_static"], account["avatar"]);
    assert_eq!(account["header"], "");
    assert_eq!(account["note"], "<p>bio text</p>");
    assert_eq!(account["followers_count"], 0);
    assert_eq!(account["statuses_count"], 2);
    assert_eq!(account["bot"], false);
    assert_eq!(account["locked"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_mastodon_status_covers_polls_unlisted_posts_and_hides_the_rest(db: PgPool) {
    let app = app(db.clone());
    let mut ada = app.person(&db, "ada").await;
    let poll = id_of(
        &ada.browser
            .post(
                "/posts",
                json!({"text": "tea?", "poll": {"options": ["yes", "no"], "durationHours": 2}}),
            )
            .await
            .ok(),
    );
    let doc = app.browser().get(&format!("/statuses/{poll}")).await.ok();
    assert_eq!(doc["poll"]["id"], poll);
    assert_eq!(doc["poll"]["multiple"], false);
    assert_eq!(doc["poll"]["expired"], false);
    assert_eq!(doc["poll"]["votes_count"], 0);
    assert_eq!(
        doc["poll"]["options"],
        json!([{"title": "yes", "votes_count": 0}, {"title": "no", "votes_count": 0}])
    );

    exec(
        &db,
        "update posts set visibility = 'unlisted' where id = $1::uuid",
        &poll,
    )
    .await;
    assert_eq!(
        app.browser().get(&format!("/statuses/{poll}")).await.ok()["visibility"],
        "unlisted"
    );
    for hidden in ["followers", "direct"] {
        exec(
            &db,
            &format!("update posts set visibility = '{hidden}' where id = $1::uuid"),
            &poll,
        )
        .await;
        app.browser()
            .get(&format!("/statuses/{poll}"))
            .await
            .err(StatusCode::NOT_FOUND, "Post not found.");
    }
    exec(
        &db,
        "update posts set visibility = 'public' where id = $1::uuid",
        &poll,
    )
    .await;
    exec(
        &db,
        "update users set suspended_at = now() where handle = 'ada'",
        "",
    )
    .await;
    app.browser()
        .get(&format!("/statuses/{poll}"))
        .await
        .err(StatusCode::NOT_FOUND, "Post not found.");
    app.browser()
        .get("/statuses/not-a-uuid")
        .await
        .err(StatusCode::NOT_FOUND, "Post not found.");
    app.browser()
        .get(&format!("/statuses/{}", Uuid::now_v7()))
        .await
        .err(StatusCode::NOT_FOUND, "Post not found.");
}

#[sqlx::test(migrations = "./migrations")]
async fn post_pages_link_to_the_note_and_oembed_and_show_views(db: PgPool) {
    let app = app(db.clone());
    let mut ada = app.person(&db, "ada").await;
    let post = id_of(&ada.browser.post_text("a \"quoted\" <b>post</b>").await);
    exec(
        &db,
        "update posts set views = 1234, like_count = 2, reply_count = 1 where id = $1::uuid",
        &post,
    )
    .await;

    let html = page(&app, &format!("/@ada/{post}")).await;
    assert!(
        html.contains(&format!(
            r#"<link rel="alternate" type="application/activity+json" href="{BASE}/users/ada/statuses/{post}">"#
        )),
        "{html}"
    );
    let encoded = format!("https%3A%2F%2Fsocial.example%2F%40ada%2F{post}");
    assert!(
        html.contains(&format!(
            r#"<link rel="alternate" type="application/json+oembed" href="{BASE}/oembed?url={encoded}">"#
        )),
        "{html}"
    );
    assert!(
        html.contains(
            r#"<meta property="og:description" content="a &quot;quoted&quot; &lt;b&gt;post&lt;/b&gt;

💬 1   ❤️ 2   👁 1.2K">"#
        ),
        "{html}"
    );
    assert!(!html.contains("<b>post</b>"));

    exec(
        &db,
        "update posts set visibility = 'unlisted' where id = $1::uuid",
        &post,
    )
    .await;
    let html = page(&app, &format!("/@ada/{post}")).await;
    assert!(html.contains("application/activity+json"));
    assert!(!html.contains("json+oembed"), "{html}");

    exec(
        &db,
        "update posts set visibility = 'followers' where id = $1::uuid",
        &post,
    )
    .await;
    let html = page(&app, &format!("/@ada/{post}")).await;
    assert!(!html.contains("rel=\"alternate\""), "{html}");
}

#[sqlx::test(migrations = "./migrations")]
async fn profile_pages_link_to_the_actor_only_when_it_is_visible(db: PgPool) {
    let app = app(db.clone());
    common::insert_user(&db, "ada").await;
    let html = page(&app, "/@ada").await;
    assert!(
        html.contains(&format!(
            r#"<link rel="alternate" type="application/activity+json" href="{BASE}/users/ada">"#
        )),
        "{html}"
    );
    assert!(!html.contains("json+oembed"));
    exec(
        &db,
        "update users set suspended_at = now() where handle = 'ada'",
        "",
    )
    .await;
    assert!(!page(&app, "/@ada").await.contains("rel=\"alternate\""));
    assert!(!page(&app, "/@ghost").await.contains("rel=\"alternate\""));
    assert!(
        !page(&app, "/@ada@b.example")
            .await
            .contains("rel=\"alternate\"")
    );
}
