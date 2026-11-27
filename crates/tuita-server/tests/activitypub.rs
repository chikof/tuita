//! The public ActivityPub surface: WebFinger, NodeInfo, actors, notes, collections and the
//! `/@handle` negotiation. Documents are compared against literal JSON.
#![allow(missing_docs)]

mod common;

use axum::body::Body;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tuita_server::{AppState, Config, router};
use uuid::Uuid;

use common::{App, Person, RawReply, app};

const PUBLIC: &str = "https://www.w3.org/ns/activitystreams#Public";
const AP: &str = "application/activity+json";
const BASE: &str = "https://social.example";

/// An app whose `PUBLIC_URL` is `url`, as two instances on loopback ports have.
fn app_at(db: PgPool, url: &str) -> App {
    let config = Config::from_lookup(|var| match var {
        "DATABASE_URL" => Some("postgres://unused".to_owned()),
        "PUBLIC_URL" => Some(url.to_owned()),
        _ => None,
    })
    .expect("config");
    let media = tuita_server::media::MediaStore::from_config(&config).expect("media store");
    let state = AppState::new(db, config, media);
    state.set_ready(true);
    let router = router(state.clone(), common::stub_web());
    App { state, router }
}

async fn get(app: &App, path: &str, accept: &str) -> RawReply {
    app.browser()
        .send_raw(Method::GET, path, &[("accept", accept)], Body::empty())
        .await
}

async fn get_ap(app: &App, path: &str) -> RawReply {
    get(app, path, AP).await
}

async fn insert_media(db: &PgPool, owner: Uuid, kind: &str, mime: &str) -> Uuid {
    let id = Uuid::now_v7();
    let ext = mime.rsplit('/').next().unwrap_or("bin");
    sqlx::query(
        "insert into media (id, owner_id, kind, mime, path, bytes, width, height, alt)
         values ($1, $2, $3, $4, $5, 10, 640, 480, $6)",
    )
    .bind(id)
    .bind(owner)
    .bind(kind)
    .bind(mime)
    .bind(format!("{id}.{ext}"))
    .bind(if kind == "image" { "a cat" } else { "" })
    .execute(db)
    .await
    .expect("insert media");
    id
}

/// Runs `sql`, binding `arg` as `$1` when the statement has one.
async fn set(db: &PgPool, sql: &str, arg: impl ToString) {
    let query = sqlx::query(sqlx::AssertSqlSafe(sql.to_owned()));
    let query = if sql.contains("$1") {
        query.bind(arg.to_string())
    } else {
        query
    };
    query.execute(db).await.expect("update");
}

async fn set_visibility(db: &PgPool, post: &str, visibility: &str) {
    sqlx::query("update posts set visibility = $2 where id = $1::uuid")
        .bind(post)
        .bind(visibility)
        .execute(db)
        .await
        .expect("visibility");
}

fn id_of(post: &Value) -> String {
    post["id"].as_str().expect("post id").to_owned()
}

fn note_id(handle: &str, post: &str) -> String {
    format!("{BASE}/users/{handle}/statuses/{post}")
}

async fn note_of(app: &App, handle: &str, post: &str) -> Value {
    get_ap(app, &format!("/users/{handle}/statuses/{post}"))
        .await
        .ok()
}

async fn signed_up(app: &App, db: &PgPool, handle: &str) -> Person {
    app.person(db, handle).await
}

#[sqlx::test(migrations = "./migrations")]
async fn webfinger_names_the_actor_and_the_instance_actor(db: PgPool) {
    let app = app_at(db.clone(), "http://127.0.0.1:8080");
    common::insert_user(&db, "ada").await;
    let domain = "127.0.0.1:8080";

    let reply = get(
        &app,
        &format!("/.well-known/webfinger?resource=acct:ada@{domain}"),
        "application/jrd+json",
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(
        reply.header("content-type"),
        "application/jrd+json; charset=utf-8"
    );
    assert_eq!(
        reply.ok(),
        json!({
            "subject": "acct:ada@127.0.0.1:8080",
            "aliases": ["http://127.0.0.1:8080/@ada", "http://127.0.0.1:8080/users/ada"],
            "links": [
                {"rel": "http://webfinger.net/rel/profile-page", "type": "text/html",
                 "href": "http://127.0.0.1:8080/@ada"},
                {"rel": "self", "type": "application/activity+json",
                 "href": "http://127.0.0.1:8080/users/ada"},
            ],
        })
    );
    let upper = get(
        &app,
        &format!("/.well-known/webfinger?resource=acct:ADA@{domain}"),
        "*/*",
    )
    .await;
    assert_eq!(upper.ok()["subject"], "acct:ada@127.0.0.1:8080");
    let by_url = get(
        &app,
        "/.well-known/webfinger?resource=http://127.0.0.1:8080/users/ada",
        "*/*",
    )
    .await;
    assert_eq!(by_url.ok()["subject"], "acct:ada@127.0.0.1:8080");

    let instance = get(
        &app,
        &format!("/.well-known/webfinger?resource=acct:{domain}@{domain}"),
        "*/*",
    )
    .await
    .ok();
    assert_eq!(instance["subject"], "acct:127.0.0.1:8080@127.0.0.1:8080");
    assert_eq!(instance["links"][1]["href"], "http://127.0.0.1:8080/actor");
}

#[sqlx::test(migrations = "./migrations")]
async fn webfinger_refuses_bad_and_unknown_resources(db: PgPool) {
    let app = app(db.clone());
    let ada = common::insert_user(&db, "ada").await;
    common::insert_user(&db, "bob").await;
    set(
        &db,
        "update users set suspended_at = now() where handle = 'bob'",
        "",
    )
    .await;
    let _ = ada;
    for resource in [
        "",
        "ada",
        "acct:ada",
        "acct:@social.example",
        "mailto:ada@social.example",
    ] {
        let path = format!("/.well-known/webfinger?resource={resource}");
        get(&app, &path, "*/*")
            .await
            .err(StatusCode::BAD_REQUEST, "Missing or invalid resource.");
    }
    get(&app, "/.well-known/webfinger", "*/*")
        .await
        .err(StatusCode::BAD_REQUEST, "Missing or invalid resource.");
    for resource in [
        "acct:ghost@social.example",
        "acct:ada@other.example",
        "acct:bob@social.example",
        "acct:tuita@other.example",
    ] {
        let path = format!("/.well-known/webfinger?resource={resource}");
        assert_eq!(
            get(&app, &path, "*/*").await.status,
            StatusCode::NOT_FOUND,
            "{resource}"
        );
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn host_meta_and_nodeinfo(db: PgPool) {
    let app = app(db.clone());
    let ada = signed_up(&app, &db, "ada").await;
    let mut ada = ada;
    ada.browser.post_text("one").await;
    ada.browser.post_text("two").await;
    common::insert_user(&db, "bob").await;
    // A remote account and its post count for nothing.
    let remote = Uuid::now_v7();
    sqlx::query("insert into users (id, handle, domain) values ($1, 'eve', 'b.example')")
        .bind(remote)
        .execute(&db)
        .await
        .expect("remote");
    sqlx::query("insert into posts (id, author_id, text, ap_id) values ($1, $2, 'hi', 'https://b.example/n/1')")
        .bind(Uuid::now_v7())
        .bind(remote)
        .execute(&db)
        .await
        .expect("remote post");

    let meta = get(&app, "/.well-known/host-meta", "*/*").await;
    assert_eq!(meta.status, StatusCode::OK);
    assert_eq!(
        meta.header("content-type"),
        "application/xrd+xml; charset=utf-8"
    );
    let xml = String::from_utf8(meta.bytes).expect("utf8");
    assert!(xml.contains(
        r#"<Link rel="lrdd" template="https://social.example/.well-known/webfinger?resource={uri}"/>"#
    ), "{xml}");

    let links = get(&app, "/.well-known/nodeinfo", "*/*").await.ok();
    assert_eq!(
        links,
        json!({"links": [
            {
                "rel": "http://nodeinfo.diaspora.software/ns/schema/2.0",
                "href": "https://social.example/nodeinfo/2.0",
            },
            {
                "rel": "http://nodeinfo.diaspora.software/ns/schema/2.1",
                "href": "https://social.example/nodeinfo/2.1",
            },
        ]})
    );
    let info = get(&app, "/nodeinfo/2.1", "*/*").await;
    assert!(info.header("content-type").starts_with("application/json"));
    let info = info.ok();
    assert_eq!(info["version"], "2.1");
    assert_eq!(info["software"]["name"], "tuita");
    assert_eq!(info["software"]["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(info["protocols"], json!(["activitypub"]));
    assert_eq!(info["openRegistrations"], true);
    assert_eq!(
        info["usage"],
        json!({"users": {"total": 2}, "localPosts": 2})
    );

    set(
        &db,
        "insert into instance_settings (key, value) values ('signups', to_jsonb($1::text))",
        "closed",
    )
    .await;
    assert_eq!(
        get(&app, "/nodeinfo/2.1", "*/*").await.ok()["openRegistrations"],
        false
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn actor_documents_for_person_bot_and_organization(db: PgPool) {
    let app = app(db.clone());
    let ada = common::insert_user(&db, "ada").await;
    let bot = common::insert_user(&db, "robo").await;
    let org = common::insert_user(&db, "acme").await;
    set(
        &db,
        "update users set is_bot = true where handle = 'robo'",
        "",
    )
    .await;
    set(
        &db,
        "update users set is_org = true, discoverable = false where handle = 'acme'",
        "",
    )
    .await;
    let avatar = insert_media(&db, ada, "image", "image/webp").await;
    let banner = insert_media(&db, ada, "image", "image/png").await;
    sqlx::query(
        "update users set avatar_id = $2, banner_id = $3, bio = $4,
                website = 'https://ada.example/' where id = $1",
    )
    .bind(ada)
    .bind(avatar)
    .bind(banner)
    .bind("Hi <b> #rust\n\nsecond @acme")
    .execute(&db)
    .await
    .expect("profile");
    sqlx::query(
        "insert into affiliations (id, org_id, user_id, label, status)
         values ($1, $2, $3, 'Engineer', 'active')",
    )
    .bind(Uuid::now_v7())
    .bind(org)
    .bind(ada)
    .execute(&db)
    .await
    .expect("affiliation");
    let _ = bot;

    let reply = get_ap(&app, "/users/ada").await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(
        reply.header("content-type"),
        "application/activity+json; charset=utf-8"
    );
    assert_eq!(reply.header("cache-control"), "public, max-age=180");
    let mut doc = reply.ok();
    let pem = doc["publicKey"]["publicKeyPem"]
        .as_str()
        .expect("pem")
        .to_owned();
    assert!(pem.starts_with("-----BEGIN PUBLIC KEY-----"), "{pem}");
    let published = doc["published"].as_str().expect("published").to_owned();
    assert!(published.ends_with('Z'));
    doc.as_object_mut().expect("object").remove("published");
    doc["publicKey"]["publicKeyPem"] = json!("PEM");
    assert_eq!(
        doc,
        json!({
            "@context": tuita_core::ap::context(),
            "id": "https://social.example/users/ada",
            "type": "Person",
            "preferredUsername": "ada",
            "name": "Name of ada",
            "summary": "<p>Hi &lt;b&gt; <a href=\"https://social.example/tags/rust\" class=\"mention hashtag\" rel=\"tag nofollow noopener noreferrer\">#<span>rust</span></a></p><p>second <span class=\"h-card\"><a href=\"https://social.example/@acme\" class=\"u-url mention\" rel=\"nofollow noopener noreferrer\">@<span>acme</span></a></span></p>",
            "url": "https://social.example/@ada",
            "icon": {"type": "Image", "mediaType": "image/webp", "url": format!("{BASE}/media/{avatar}.webp")},
            "image": {"type": "Image", "mediaType": "image/png", "url": format!("{BASE}/media/{banner}.png")},
            "inbox": "https://social.example/users/ada/inbox",
            "outbox": "https://social.example/users/ada/outbox",
            "followers": "https://social.example/users/ada/followers",
            "following": "https://social.example/users/ada/following",
            "featured": "https://social.example/users/ada/featured",
            "endpoints": {"sharedInbox": "https://social.example/inbox"},
            "publicKey": {
                "id": "https://social.example/users/ada#main-key",
                "owner": "https://social.example/users/ada",
                "publicKeyPem": "PEM",
            },
            "manuallyApprovesFollowers": false,
            "discoverable": true,
            "attachment": [
                {"type": "PropertyValue", "name": "Website",
                 "value": "<a href=\"https://ada.example/\" rel=\"me nofollow noopener noreferrer\" target=\"_blank\">https://ada.example/</a>"},
                {"type": "PropertyValue", "name": "Affiliated with",
                 "value": "Engineer, <a href=\"https://social.example/@acme\" rel=\"nofollow noopener noreferrer\">Name of acme</a>"},
            ],
        })
    );

    let bot = get_ap(&app, "/users/robo").await.ok();
    assert_eq!(bot["type"], "Service");
    assert_eq!(bot["attachment"], json!([]));
    assert_eq!(bot["summary"], "");
    assert!(bot.get("icon").is_none());
    let org = get_ap(&app, "/users/acme").await.ok();
    assert_eq!(org["type"], "Organization");
    assert_eq!(org["discoverable"], false);
    assert_eq!(
        get_ap(&app, "/users/ADA").await.ok()["id"],
        "https://social.example/users/ada"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn deleted_accounts_are_gone_and_suspended_or_unknown_ones_are_not_found(db: PgPool) {
    let app = app(db.clone());
    common::insert_user(&db, "gone").await;
    common::insert_user(&db, "held").await;
    set(
        &db,
        "update users set deleted_at = now() where handle = 'gone'",
        "",
    )
    .await;
    set(
        &db,
        "update users set suspended_at = now() where handle = 'held'",
        "",
    )
    .await;

    let gone = get_ap(&app, "/users/gone").await;
    assert_eq!(gone.status, StatusCode::GONE);
    assert_eq!(
        gone.header("content-type"),
        "application/activity+json; charset=utf-8"
    );
    assert_eq!(
        gone.json(),
        json!({
            "@context": tuita_core::ap::context(),
            "id": "https://social.example/users/gone",
            "type": "Tombstone",
        })
    );
    for path in [
        "/users/held",
        "/users/nobody",
        "/users/held/outbox",
        "/users/held/followers",
        "/users/held/following",
        "/users/held/featured",
        "/users/gone/outbox",
        "/users/nobody/outbox",
    ] {
        assert_eq!(
            get_ap(&app, path).await.status,
            StatusCode::NOT_FOUND,
            "{path}"
        );
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn a_plain_note_with_mentions_and_hashtags(db: PgPool) {
    let app = app(db.clone());
    let mut ada = signed_up(&app, &db, "ada").await;
    common::insert_user(&db, "bob").await;
    let post = ada
        .browser
        .post_text("hi @bob & friends #Rust https://example.com/x\nline two")
        .await;
    let id = id_of(&post);

    let reply = get_ap(&app, &format!("/users/ada/statuses/{id}")).await;
    assert_eq!(
        reply.header("content-type"),
        "application/activity+json; charset=utf-8"
    );
    assert_eq!(reply.header("cache-control"), "public, max-age=180");
    let mut doc = reply.ok();
    assert!(doc["published"].as_str().is_some_and(|t| t.ends_with('Z')));
    doc.as_object_mut().expect("object").remove("published");
    assert_eq!(
        doc,
        json!({
            "@context": tuita_core::ap::context(),
            "id": note_id("ada", &id),
            "type": "Note",
            "attributedTo": "https://social.example/users/ada",
            "content": "<p>hi <span class=\"h-card\"><a href=\"https://social.example/@bob\" class=\"u-url mention\" rel=\"nofollow noopener noreferrer\">@<span>bob</span></a></span> &amp; friends <a href=\"https://social.example/tags/rust\" class=\"mention hashtag\" rel=\"tag nofollow noopener noreferrer\">#<span>Rust</span></a> <a href=\"https://example.com/x\" rel=\"nofollow noopener noreferrer\" target=\"_blank\">https://example.com/x</a><br>line two</p>",
            "sensitive": false,
            "url": format!("{BASE}/@ada/{id}"),
            "to": [PUBLIC],
            "cc": ["https://social.example/users/ada/followers", "https://social.example/users/bob"],
            "attachment": [],
            "tag": [
                {"type": "Mention", "href": "https://social.example/users/bob", "name": "@bob"},
                {"type": "Hashtag", "name": "#rust", "href": "https://social.example/tags/rust"},
            ],
        })
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn content_warnings_edits_replies_and_quotes(db: PgPool) {
    let app = app(db.clone());
    let mut ada = signed_up(&app, &db, "ada").await;
    let mut bob = signed_up(&app, &db, "bob").await;
    let root = ada.browser.post_text("root").await;
    let root_id = id_of(&root);
    let cw = ada
        .browser
        .post(
            "/posts",
            json!({"text": "secret", "cw": "spoilers", "sensitive": true}),
        )
        .await
        .ok();
    let doc = note_of(&app, "ada", &id_of(&cw)).await;
    assert_eq!(doc["summary"], "spoilers");
    assert_eq!(doc["sensitive"], true);
    assert!(doc.get("updated").is_none());

    let reply = bob
        .browser
        .post("/posts", json!({"text": "answer", "replyTo": root_id}))
        .await
        .ok();
    let quote = bob
        .browser
        .post("/posts", json!({"text": "look", "quoteOf": root_id}))
        .await
        .ok();
    let reply_doc = note_of(&app, "bob", &id_of(&reply)).await;
    assert_eq!(reply_doc["inReplyTo"], note_id("ada", &root_id));
    assert_eq!(
        reply_doc["cc"],
        json!([
            "https://social.example/users/bob/followers",
            "https://social.example/users/ada"
        ])
    );
    let quote_doc = note_of(&app, "bob", &id_of(&quote)).await;
    assert_eq!(quote_doc["quoteUrl"], note_id("ada", &root_id));
    assert_eq!(quote_doc["_misskey_quote"], note_id("ada", &root_id));
    assert!(quote_doc.get("inReplyTo").is_none());

    ada.browser
        .patch(
            &format!("/posts/{root_id}"),
            json!({"text": "root, edited"}),
        )
        .await
        .ok();
    let edited = note_of(&app, "ada", &root_id).await;
    assert!(edited["updated"].as_str().is_some_and(|t| t.ends_with('Z')));
    assert_eq!(edited["content"], "<p>root, edited</p>");
}

#[sqlx::test(migrations = "./migrations")]
async fn media_documents_for_images_and_video(db: PgPool) {
    let app = app(db.clone());
    let mut ada = signed_up(&app, &db, "ada").await;
    let one = insert_media(&db, ada.id, "image", "image/webp").await;
    let two = insert_media(&db, ada.id, "image", "image/png").await;
    let video = insert_media(&db, ada.id, "video", "video/mp4").await;
    let images = ada
        .browser
        .post(
            "/posts",
            json!({"text": "pics", "mediaIds": [one.to_string(), two.to_string()]}),
        )
        .await
        .ok();
    let clip = ada
        .browser
        .post(
            "/posts",
            json!({"text": "clip", "mediaIds": [video.to_string()]}),
        )
        .await
        .ok();
    let doc = note_of(&app, "ada", &id_of(&images)).await;
    assert_eq!(
        doc["attachment"],
        json!([
            {"type": "Document", "mediaType": "image/webp", "url": format!("{BASE}/media/{one}.webp"),
             "name": "a cat", "width": 640, "height": 480},
            {"type": "Document", "mediaType": "image/png", "url": format!("{BASE}/media/{two}.png"),
             "name": "a cat", "width": 640, "height": 480},
        ])
    );
    let doc = note_of(&app, "ada", &id_of(&clip)).await;
    assert_eq!(
        doc["attachment"],
        json!([{"type": "Document", "mediaType": "video/mp4", "url": format!("{BASE}/media/{video}.mp4"),
                "width": 640, "height": 480}])
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn polls_are_questions(db: PgPool) {
    let app = app(db.clone());
    let mut ada = signed_up(&app, &db, "ada").await;
    let mut bob = signed_up(&app, &db, "bob").await;
    let single = ada
        .browser
        .post("/posts", json!({"text": "tea or coffee?", "poll": {"options": ["tea", "coffee"], "durationHours": 24}}))
        .await
        .ok();
    let multi = ada
        .browser
        .post(
            "/posts",
            json!({"text": "pick", "poll": {"options": ["a", "b", "c"], "multiple": true, "durationHours": 1}}),
        )
        .await
        .ok();
    bob.browser
        .post(
            &format!("/posts/{}/vote", id_of(&single)),
            json!({"choices": [1]}),
        )
        .await
        .ok();

    let doc = note_of(&app, "ada", &id_of(&single)).await;
    assert_eq!(doc["type"], "Question");
    assert_eq!(
        doc["oneOf"],
        json!([
            {"name": "tea", "replies": {"totalItems": 0}},
            {"name": "coffee", "replies": {"totalItems": 1}},
        ])
    );
    assert_eq!(doc["votersCount"], 1);
    assert!(doc["endTime"].as_str().is_some_and(|t| t.ends_with('Z')));
    assert!(doc.get("anyOf").is_none());
    assert!(doc.get("closed").is_none());

    let doc = note_of(&app, "ada", &id_of(&multi)).await;
    assert_eq!(doc["type"], "Question");
    assert_eq!(doc["anyOf"].as_array().map(Vec::len), Some(3));
    assert!(doc.get("oneOf").is_none());

    set(
        &db,
        "update polls set ends_at = now() - interval '1 hour' where post_id = $1::uuid",
        id_of(&single),
    )
    .await;
    let doc = note_of(&app, "ada", &id_of(&single)).await;
    assert!(doc["closed"].as_str().is_some_and(|t| t.ends_with('Z')));
}

#[sqlx::test(migrations = "./migrations")]
async fn addressing_follows_visibility_and_only_open_posts_are_served(db: PgPool) {
    let app = app(db.clone());
    let mut ada = signed_up(&app, &db, "ada").await;
    common::insert_user(&db, "bob").await;
    let post = id_of(&ada.browser.post_text("hello @bob").await);
    let followers = "https://social.example/users/ada/followers";
    let bob = "https://social.example/users/bob";
    for (visibility, to, cc) in [
        ("public", json!([PUBLIC]), json!([followers, bob])),
        ("unlisted", json!([followers]), json!([PUBLIC, bob])),
    ] {
        set_visibility(&db, &post, visibility).await;
        let doc = note_of(&app, "ada", &post).await;
        assert_eq!(doc["to"], to, "{visibility}");
        assert_eq!(doc["cc"], cc, "{visibility}");
        let activity = get_ap(&app, &format!("/users/ada/statuses/{post}/activity"))
            .await
            .ok();
        assert_eq!(activity["to"], to, "{visibility}");
        assert_eq!(activity["cc"], cc, "{visibility}");
    }
    for visibility in ["followers", "direct"] {
        set_visibility(&db, &post, visibility).await;
        for path in [
            format!("/users/ada/statuses/{post}"),
            format!("/users/ada/statuses/{post}/activity"),
        ] {
            assert_eq!(
                get_ap(&app, &path).await.status,
                StatusCode::NOT_FOUND,
                "{visibility} {path}"
            );
        }
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn a_note_needs_the_right_author_and_a_live_post(db: PgPool) {
    let app = app(db.clone());
    let mut ada = signed_up(&app, &db, "ada").await;
    common::insert_user(&db, "bob").await;
    let post = id_of(&ada.browser.post_text("mine").await);
    assert_eq!(
        get_ap(&app, &format!("/users/bob/statuses/{post}"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get_ap(&app, "/users/ada/statuses/not-a-uuid").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get_ap(&app, &format!("/users/ada/statuses/{}", Uuid::now_v7()))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    ada.browser.delete(&format!("/posts/{post}")).await.ok();
    assert_eq!(
        get_ap(&app, &format!("/users/ada/statuses/{post}"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let kept = id_of(&ada.browser.post_text("kept").await);
    assert_eq!(
        get_ap(&app, &format!("/users/ada/statuses/{kept}"))
            .await
            .status,
        StatusCode::OK
    );
    set(
        &db,
        "update users set suspended_at = now() where handle = 'ada'",
        "",
    )
    .await;
    assert_eq!(
        get_ap(&app, &format!("/users/ada/statuses/{kept}"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn the_create_activity_wraps_the_note(db: PgPool) {
    let app = app(db.clone());
    let mut ada = signed_up(&app, &db, "ada").await;
    let post = id_of(&ada.browser.post_text("wrapped").await);
    let doc = get_ap(&app, &format!("/users/ada/statuses/{post}/activity"))
        .await
        .ok();
    assert_eq!(doc["id"], format!("{}/activity", note_id("ada", &post)));
    assert_eq!(doc["type"], "Create");
    assert_eq!(doc["actor"], "https://social.example/users/ada");
    assert_eq!(doc["to"], json!([PUBLIC]));
    assert_eq!(doc["object"]["id"], note_id("ada", &post));
    assert_eq!(doc["object"]["content"], "<p>wrapped</p>");
    assert!(doc["object"].get("@context").is_none());
    assert_eq!(doc["@context"], tuita_core::ap::context());
}

#[sqlx::test(migrations = "./migrations")]
async fn the_outbox_pages_newest_first_and_skips_private_and_deleted_posts(db: PgPool) {
    let app = app(db.clone());
    let mut ada = signed_up(&app, &db, "ada").await;
    let mut ids = Vec::new();
    for n in 0..25 {
        ids.push(id_of(&ada.browser.post_text(&format!("post {n}")).await));
    }
    let hidden = id_of(&ada.browser.post_text("unlisted").await);
    set_visibility(&db, &hidden, "unlisted").await;
    let deleted = id_of(&ada.browser.post_text("deleted").await);
    ada.browser.delete(&format!("/posts/{deleted}")).await.ok();

    let root = get_ap(&app, "/users/ada/outbox").await.ok();
    assert_eq!(
        root,
        json!({
            "@context": tuita_core::ap::context(),
            "id": "https://social.example/users/ada/outbox",
            "type": "OrderedCollection",
            "totalItems": 25,
            "first": "https://social.example/users/ada/outbox?page=true",
        })
    );
    let first = get_ap(&app, "/users/ada/outbox?page=true").await.ok();
    assert_eq!(first["type"], "OrderedCollectionPage");
    assert_eq!(
        first["id"],
        "https://social.example/users/ada/outbox?page=true"
    );
    assert_eq!(first["partOf"], "https://social.example/users/ada/outbox");
    let items = first["orderedItems"].as_array().expect("items");
    assert_eq!(items.len(), 20);
    assert_eq!(items[0]["type"], "Create");
    assert_eq!(items[0]["object"]["id"], note_id("ada", &ids[24]));
    assert_eq!(items[19]["object"]["id"], note_id("ada", &ids[5]));
    assert!(items[0].get("@context").is_none());
    let next = format!(
        "https://social.example/users/ada/outbox?page=true&before={}",
        ids[5]
    );
    assert_eq!(first["next"], next);

    let second = get_ap(
        &app,
        &format!("/users/ada/outbox?page=true&before={}", ids[5]),
    )
    .await
    .ok();
    assert_eq!(second["id"], next);
    let items = second["orderedItems"].as_array().expect("items");
    assert_eq!(items.len(), 5);
    assert_eq!(items[4]["object"]["id"], note_id("ada", &ids[0]));
    assert!(second.get("next").is_none());

    assert_eq!(
        get_ap(&app, "/users/ada/outbox?page=true&before=nope")
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
    let empty = get_ap(
        &app,
        &format!("/users/ada/outbox?page=true&before={}", ids[0]),
    )
    .await
    .ok();
    assert_eq!(empty["orderedItems"], json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn followers_following_and_featured(db: PgPool) {
    let app = app(db.clone());
    let mut ada = signed_up(&app, &db, "ada").await;
    let mut bob = signed_up(&app, &db, "bob").await;
    bob.browser.post_empty("/users/ada/follow").await.ok();
    let followers = get_ap(&app, "/users/ada/followers").await.ok();
    assert_eq!(
        followers,
        json!({
            "@context": tuita_core::ap::context(),
            "id": "https://social.example/users/ada/followers",
            "type": "OrderedCollection",
            "totalItems": 1,
        })
    );
    assert_eq!(
        get_ap(&app, "/users/bob/following").await.ok()["totalItems"],
        1
    );
    assert_eq!(
        get_ap(&app, "/users/ada/following").await.ok()["totalItems"],
        0
    );

    let empty = get_ap(&app, "/users/ada/featured").await.ok();
    assert_eq!(
        empty,
        json!({
            "@context": tuita_core::ap::context(),
            "id": "https://social.example/users/ada/featured",
            "type": "OrderedCollection",
            "totalItems": 0,
            "orderedItems": [],
        })
    );
    let pinned = id_of(&ada.browser.post_text("pin me").await);
    ada.browser
        .post_empty(&format!("/posts/{pinned}/pin"))
        .await
        .ok();
    let featured = get_ap(&app, "/users/ada/featured").await.ok();
    assert_eq!(featured["totalItems"], 1);
    assert_eq!(featured["orderedItems"][0]["id"], note_id("ada", &pinned));
    assert_eq!(featured["orderedItems"][0]["type"], "Note");
    set_visibility(&db, &pinned, "followers").await;
    assert_eq!(
        get_ap(&app, "/users/ada/featured").await.ok()["totalItems"],
        0
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn the_instance_actor_is_an_application(db: PgPool) {
    let app = app(db.clone());
    let reply = get_ap(&app, "/actor").await;
    assert_eq!(
        reply.header("content-type"),
        "application/activity+json; charset=utf-8"
    );
    let mut doc = reply.ok();
    let pem = doc["publicKey"]["publicKeyPem"]
        .as_str()
        .expect("pem")
        .to_owned();
    assert!(pem.starts_with("-----BEGIN PUBLIC KEY-----"));
    doc["publicKey"]["publicKeyPem"] = json!("PEM");
    assert_eq!(
        doc,
        json!({
            "@context": tuita_core::ap::context(),
            "id": "https://social.example/actor",
            "type": "Application",
            "preferredUsername": "social.example",
            "name": "tuita",
            "url": "https://social.example",
            "inbox": "https://social.example/inbox",
            "publicKey": {
                "id": "https://social.example/actor#main-key",
                "owner": "https://social.example/actor",
                "publicKeyPem": "PEM",
            },
            "manuallyApprovesFollowers": true,
            "discoverable": false,
        })
    );
    assert_eq!(
        get_ap(&app, "/actor").await.ok()["publicKey"]["publicKeyPem"],
        pem
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn at_handle_urls_negotiate_between_activitypub_and_the_web_client(db: PgPool) {
    let app = app(db.clone());
    let mut ada = signed_up(&app, &db, "ada").await;
    let post = id_of(&ada.browser.post_text("negotiated").await);

    let redirect = get_ap(&app, "/@ada").await;
    assert_eq!(redirect.status, StatusCode::FOUND);
    assert_eq!(redirect.header("location"), "/users/ada");
    let redirect = get(
        &app,
        &format!("/@ada/{post}"),
        "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"",
    )
    .await;
    assert_eq!(redirect.status, StatusCode::FOUND);
    assert_eq!(
        redirect.header("location"),
        format!("/users/ada/statuses/{post}")
    );

    let page = get(&app, "/@ada", "text/html,application/xhtml+xml").await;
    assert_eq!(page.status, StatusCode::OK);
    assert_eq!(String::from_utf8_lossy(&page.bytes), "web");
    assert_eq!(get(&app, "/@ada", "*/*").await.status, StatusCode::OK);

    // A remote profile is not ours to redirect.
    assert_eq!(
        get_ap(&app, "/@ada@b.example").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get_ap(&app, &format!("/@ada@b.example/{post}"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&app, "/@ada@b.example", "text/html").await.status,
        StatusCode::OK
    );
    // The redirect target serves the actor.
    assert_eq!(get_ap(&app, "/users/ada").await.status, StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_system_account_is_served_nowhere(db: PgPool) {
    let app = app(db.clone());
    let mut ada = signed_up(&app, &db, "ada").await;
    let exists: bool =
        sqlx::query_scalar("select exists (select 1 from users where handle = 'tuita')")
            .fetch_one(&db)
            .await
            .expect("system account");
    assert!(exists, "the migration creates it");

    assert_eq!(
        get_ap(&app, "/users/tuita").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get_ap(&app, "/users/tuita/outbox").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(
            &app,
            "/.well-known/webfinger?resource=acct:tuita@social.example",
            "application/jrd+json"
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    let profile = |head: String| head.contains("content=\"profile\"");
    let head = |path: &'static str| tuita_server::routes::head::head_tags(&app.state, path);
    assert!(profile(head("/@ada").await), "control");
    assert!(!profile(head("/@tuita").await));
    ada.browser
        .get("/users/tuita")
        .await
        .err(StatusCode::NOT_FOUND, "User not found.");
    ada.browser
        .post_empty("/users/tuita/follow")
        .await
        .err(StatusCode::NOT_FOUND, "User not found.");
    let found = ada.browser.get("/search?q=tuita&type=people").await.ok();
    assert_eq!(found["users"], json!([]));
}
