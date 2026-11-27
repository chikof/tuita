//! Custom emotes on the wire: local emotes leave as ActivityPub `Emoji` tags (notes, actors and
//! actor updates), and the emotes of incoming notes and actors are copied through `SafeClient`
//! and shown through the API. A fake remote instance serves the actors and the icons.
#![allow(missing_docs)]

mod common;
#[path = "common/oauth_client.rs"]
mod oauth_client;

use std::collections::HashMap;
use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::{Arc, LazyLock, Mutex};

use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::http::{Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use common::{App, ORIGIN, app, app_with, insert_user};
use oauth_client::{Env, sign_in};
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tuita_core::ap::{self, Note, Tag};
use tuita_server::db::emoji;
use tuita_server::db::remote::{self, RemoteProfile, Upsert};
use tuita_server::db::users::{self, UserRow};
use tuita_server::federation::ingest::{self, Context, Stored};
use tuita_server::federation::lookup;
use uuid::Uuid;

// ------------------------------------------------------------------ the fake instance

#[derive(Default)]
struct Inner {
    docs: Mutex<HashMap<String, Value>>,
    raw: Mutex<HashMap<String, (&'static str, Vec<u8>)>>,
    webfinger: Mutex<HashMap<String, String>>,
    hits: Mutex<Vec<String>>,
}

struct Fake {
    addr: SocketAddr,
    inner: Arc<Inner>,
}

async fn serve(State(inner): State<Arc<Inner>>, uri: Uri) -> Response {
    inner.hits.lock().expect("lock").push(uri.path().to_owned());
    if uri.path() == "/.well-known/webfinger" {
        let resource = url::form_urlencoded::parse(uri.query().unwrap_or("").as_bytes())
            .find(|(key, _)| key == "resource")
            .map(|(_, value)| value.into_owned())
            .unwrap_or_default();
        let href = inner
            .webfinger
            .lock()
            .expect("lock")
            .get(&resource)
            .cloned();
        return match href {
            Some(href) => (
                [(CONTENT_TYPE, "application/jrd+json")],
                json!({"subject": resource, "links": [
                    {"rel": "self", "type": "application/activity+json", "href": href}
                ]})
                .to_string(),
            )
                .into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        };
    }
    let doc = inner.docs.lock().expect("lock").get(uri.path()).cloned();
    if let Some(doc) = doc {
        return (
            [(CONTENT_TYPE, "application/activity+json")],
            doc.to_string(),
        )
            .into_response();
    }
    if let Some((kind, bytes)) = inner.raw.lock().expect("lock").get(uri.path()) {
        return ([(CONTENT_TYPE, *kind)], bytes.clone()).into_response();
    }
    StatusCode::NOT_FOUND.into_response()
}

impl Fake {
    async fn start() -> Self {
        let inner = Arc::new(Inner::default());
        let router = Router::new().fallback(serve).with_state(inner.clone());
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            let _ = axum::serve(listener, router).await;
        });
        Self { addr, inner }
    }

    fn base(&self) -> String {
        format!("http://{}", self.addr)
    }

    fn domain(&self) -> String {
        self.addr.to_string()
    }

    fn put(&self, path: &str, doc: Value) {
        self.inner
            .docs
            .lock()
            .expect("lock")
            .insert(path.to_owned(), doc);
    }

    fn put_raw(&self, path: &str, kind: &'static str, bytes: Vec<u8>) {
        self.inner
            .raw
            .lock()
            .expect("lock")
            .insert(path.to_owned(), (kind, bytes));
    }

    fn hits_of(&self, path: &str) -> usize {
        self.inner
            .hits
            .lock()
            .expect("lock")
            .iter()
            .filter(|hit| *hit == path)
            .count()
    }

    /// Serves actor `name` and makes WebFinger find it; `tweak` edits the document first.
    fn add_actor(&self, name: &str, tweak: impl FnOnce(&mut Value)) {
        let base = self.base();
        let id = format!("{base}/users/{name}");
        let mut actor = json!({
            "@context": ["https://www.w3.org/ns/activitystreams", "https://w3id.org/security/v1"],
            "id": id,
            "type": "Person",
            "preferredUsername": name,
            "name": format!("Name of {name}"),
            "summary": "<p>A bio</p>",
            "url": format!("{base}/@{name}"),
            "inbox": format!("{id}/inbox"),
            "outbox": format!("{id}/outbox"),
            "followers": format!("{id}/followers"),
            "publicKey": {"id": format!("{id}#main-key"), "owner": id, "publicKeyPem": pem()}
        });
        tweak(&mut actor);
        self.put(&format!("/users/{name}"), actor);
        self.put(
            &format!("/users/{name}/outbox"),
            json!({
                "id": format!("{id}/outbox"),
                "type": "OrderedCollection",
                "totalItems": 0,
                "first": {"id": format!("{id}/outbox?page=true"), "type": "OrderedCollectionPage", "orderedItems": []}
            }),
        );
        self.inner
            .webfinger
            .lock()
            .expect("lock")
            .insert(format!("acct:{name}@{}", self.domain()), id);
    }
}

static PEM: LazyLock<String> =
    LazyLock::new(|| ap::generate_keypair().expect("keypair").public_pem);

fn pem() -> &'static str {
    &PEM
}

fn png(width: u32, height: u32) -> Vec<u8> {
    let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
        width,
        height,
        image::Rgb([200, 30, 30]),
    ));
    let mut out = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .expect("png");
    out
}

/// Where this test binary's copied media goes.
static DATA: LazyLock<String> = LazyLock::new(|| {
    std::env::temp_dir()
        .join(format!("tuita-emoji-federation-{}", Uuid::now_v7()))
        .display()
        .to_string()
});

/// A config that lets the app reach the fake instance on loopback.
fn peer_config(var: &str) -> Option<String> {
    match var {
        "FEDERATION_ALLOW_HTTP" | "FEDERATION_ALLOW_PRIVATE_NETS" => Some("1".to_owned()),
        "DATA_DIR" => Some(DATA.clone()),
        _ => None,
    }
}

fn emoji_tag(base: &str, name: &str, path: &str) -> Value {
    json!({
        "type": "Emoji",
        "id": format!("{base}/emojis/{name}"),
        "name": format!(":{name}:"),
        "icon": {"type": "Image", "mediaType": "image/png", "url": format!("{base}{path}")},
    })
}

fn note_with(base: &str, name: &str, n: u32, content: &str, tags: &[Value]) -> Note {
    serde_json::from_value(json!({
        "id": format!("{base}/users/{name}/statuses/{n}"),
        "type": "Note",
        "attributedTo": format!("{base}/users/{name}"),
        "content": format!("<p>{content}</p>"),
        "published": "2026-01-02T10:00:00Z",
        "url": format!("{base}/@{name}/{n}"),
        "to": [ap::PUBLIC],
        "cc": [format!("{base}/users/{name}/followers")],
        "tag": tags,
    }))
    .expect("note")
}

/// Stores a remote account without any network traffic.
async fn remote_user(db: &PgPool, base: &str, name: &str) -> UserRow {
    let ap_id = format!("{base}/users/{name}");
    let profile = RemoteProfile {
        handle: name.to_owned(),
        domain: ap::url_authority(base).expect("authority"),
        display_name: name.to_owned(),
        bio: String::new(),
        website: String::new(),
        is_bot: false,
        is_org: false,
        locked: false,
        discoverable: true,
        inbox_url: format!("{ap_id}/inbox"),
        shared_inbox_url: None,
        outbox_url: None,
        followers_url: Some(format!("{ap_id}/followers")),
        featured_url: None,
        url: None,
        public_key_pem: pem().to_owned(),
        created_at: None,
        ap_id,
    };
    let Upsert::Stored(id) = remote::upsert_actor(db, &profile).await.expect("upsert") else {
        panic!("not stored");
    };
    users::by_id(db, id).await.expect("load").expect("row")
}

async fn store(app: &App, author: &UserRow, note: &Note) -> Uuid {
    match ingest::store_note(&app.state, author, note, Context::Lookup)
        .await
        .expect("stored")
    {
        Stored::Post { id, .. } => id,
        other => panic!("not a post: {other:?}"),
    }
}

/// The copied emotes of `domain` as `(shortcode, mime, width, height)`, by shortcode.
async fn copies(db: &PgPool, domain: &str) -> Vec<(String, String, i32, i32)> {
    sqlx::query_as(
        "select e.shortcode, m.mime, coalesce(m.width, 0), coalesce(m.height, 0)
           from custom_emojis e join media m on m.id = e.media_id
          where e.domain = $1 order by lower(e.shortcode)",
    )
    .bind(domain)
    .fetch_all(db)
    .await
    .expect("copies")
}

// ------------------------------------------------------------------ outbound

/// A local emote: a media row and its `custom_emojis` row. Returns the file name.
async fn local_emote(db: &PgPool, owner: Uuid, shortcode: &str) -> String {
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
    sqlx::query("insert into custom_emojis (id, shortcode, media_id) values ($1, $2, $3)")
        .bind(Uuid::now_v7())
        .bind(shortcode)
        .bind(id)
        .execute(db)
        .await
        .expect("emote");
    path
}

/// A remote follower of `followee` with a fresh profile, so no lookup touches the network.
async fn remote_follower(db: &PgPool, followee: Uuid) {
    let id = Uuid::now_v7();
    sqlx::query(
        "insert into users (id, handle, domain, display_name, ap_id, inbox_url, followers_url,
                            fetched_at)
         values ($1, 'bob', 'a.example', 'bob', 'https://a.example/users/bob',
                 'https://a.example/users/bob/inbox', 'https://a.example/users/bob/followers', now())",
    )
    .bind(id)
    .execute(db)
    .await
    .expect("remote");
    sqlx::query(
        "insert into follows (follower_id, followee_id, state) values ($1, $2, 'accepted')",
    )
    .bind(id)
    .bind(followee)
    .execute(db)
    .await
    .expect("follow");
}

async fn queued(db: &PgPool) -> Vec<Value> {
    sqlx::query_scalar("select activity from ap_deliveries order by id")
        .fetch_all(db)
        .await
        .expect("queue")
}

fn expected_tag(name: &str, path: &str) -> Value {
    json!({
        "type": "Emoji",
        "id": format!("{ORIGIN}/emojis/{name}"),
        "name": format!(":{name}:"),
        "icon": {
            "type": "Image",
            "mediaType": "image/webp",
            "url": format!("{ORIGIN}/media/{path}"),
        },
    })
}

#[sqlx::test(migrations = "./migrations")]
async fn a_note_delivers_one_emoji_tag_per_local_emote_it_mentions(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let blob = local_emote(&db, ana.id, "blob").await;
    let wave = local_emote(&db, ana.id, "Wave").await;
    local_emote(&db, ana.id, "unused").await;
    remote_follower(&db, ana.id).await;

    ana.browser
        .post(
            "/posts",
            json!({
                "text": "hi :blob: :BLOB: :nope: #rust",
                "cw": "careful :wave:",
                "poll": {"options": ["a :wave:", "b"], "multiple": false, "durationHours": 24},
            }),
        )
        .await
        .ok();
    let rows = queued(&db).await;
    assert_eq!(rows.len(), 1);
    let object = &rows[0]["object"];
    assert_eq!(object["type"], "Question");
    let tags = object["tag"].as_array().expect("tags");
    assert_eq!(tags.len(), 3, "{tags:?}");
    assert_eq!(tags[0]["type"], "Hashtag");
    assert_eq!(tags[1], expected_tag("blob", &blob));
    assert_eq!(tags[2], expected_tag("Wave", &wave));
    assert!(
        rows[0]["@context"]
            .as_array()
            .expect("context")
            .iter()
            .any(|entry| entry["Emoji"] == "toot:Emoji")
    );

    let back: Note = serde_json::from_value(object.clone()).expect("our own note");
    assert!(back.tag.iter().any(|tag| matches!(
        tag,
        Tag::Emoji { name, icon, media_type, .. }
            if name == "blob"
                && *icon == format!("{ORIGIN}/media/{blob}")
                && media_type.as_deref() == Some("image/webp")
    )));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_note_without_emote_tokens_has_no_emoji_tag(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    local_emote(&db, ana.id, "blob").await;
    remote_follower(&db, ana.id).await;
    ana.browser.post_text("it is 12:30:45 and :nope:").await;
    let rows = queued(&db).await;
    assert_eq!(rows[0]["object"]["tag"], json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn the_actor_and_its_update_carry_the_emotes_of_the_name_and_bio(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let blob = local_emote(&db, ana.id, "blob").await;
    let wave = local_emote(&db, ana.id, "wave").await;
    remote_follower(&db, ana.id).await;

    ana.browser
        .patch(
            "/me",
            json!({"displayName": "Ana :blob:", "bio": "waving :wave: :nope:"}),
        )
        .await
        .ok();
    let rows = queued(&db).await;
    let update = rows
        .iter()
        .find(|row| row["type"] == "Update")
        .expect("actor update");
    assert_eq!(
        update["object"]["tag"],
        json!([expected_tag("blob", &blob), expected_tag("wave", &wave)])
    );

    let reply = ana
        .browser
        .send_raw(
            Method::GET,
            "/users/ana",
            &[("accept", "application/activity+json")],
            Body::empty(),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(
        reply.json()["tag"],
        json!([expected_tag("blob", &blob), expected_tag("wave", &wave)])
    );
    insert_user(&db, "plain").await;
    let mut anonymous = app.browser();
    let doc = anonymous
        .send_raw(
            Method::GET,
            "/users/plain",
            &[("accept", "application/activity+json")],
            Body::empty(),
        )
        .await
        .json();
    assert!(doc.get("tag").is_none(), "{doc}");
}

// ------------------------------------------------------------------ inbound

#[sqlx::test(migrations = "./migrations")]
async fn an_incoming_note_has_its_used_emotes_copied_and_shown(db: PgPool) {
    let env = Env::with(db.clone(), peer_config);
    let fake = Fake::start().await;
    let base = fake.base();
    fake.put_raw("/e/blob.png", "image/png", png(300, 300));
    fake.put_raw("/e/unused.png", "image/png", png(16, 16));
    let ann = remote_user(&db, &base, "ann").await;
    let note = note_with(
        &base,
        "ann",
        1,
        "hello :blob:",
        &[
            emoji_tag(&base, "blob", "/e/blob.png"),
            emoji_tag(&base, "unused", "/e/unused.png"),
        ],
    );
    let post = store(&env.app, &ann, &note).await;

    let found = copies(&db, &fake.domain()).await;
    assert_eq!(found.len(), 1, "{found:?}");
    let (shortcode, mime, width, height) = &found[0];
    assert_eq!(shortcode, "blob");
    assert_eq!(mime, "image/webp", "re-encoded, not stored as sent");
    assert!(*width <= 128 && *height <= 128 && *width > 0, "{found:?}");
    assert_eq!(fake.hits_of("/e/blob.png"), 1);
    assert_eq!(
        fake.hits_of("/e/unused.png"),
        0,
        "an emote the text does not use is not fetched"
    );

    let mut viewer = env.app.person(&db, "viewer").await;
    let thread = viewer.browser.get(&format!("/posts/{post}")).await.ok();
    let emojis = thread["post"]["emojis"].as_array().expect("emojis");
    assert_eq!(emojis.len(), 1, "{emojis:?}");
    assert_eq!(emojis[0]["shortcode"], "blob");
    let url = emojis[0]["url"].as_str().expect("url");
    assert!(url.starts_with("/media/"), "{url}");
    assert_eq!(thread["post"]["text"], "hello :blob:");

    let mut api = sign_in(&env.app, &mut viewer, "read", [203, 0, 113, 20]).await;
    let status = api.get(&format!("/api/v1/statuses/{post}")).await.ok();
    assert_eq!(status["emojis"].as_array().map(Vec::len), Some(1));
    assert_eq!(status["emojis"][0]["shortcode"], "blob");
    assert_eq!(status["emojis"][0]["url"], format!("{ORIGIN}{url}"));
    assert_eq!(
        status["emojis"][0]["static_url"],
        status["emojis"][0]["url"]
    );
    assert_eq!(status["emojis"][0]["visible_in_picker"], true);

    // The remote copy is no local emote.
    assert!(
        emoji::resolve(&db, None, &[":blob:"])
            .await
            .expect("resolve")
            .is_empty()
    );

    // The same note again, and an edit of it, fetch nothing: the copy exists.
    store(&env.app, &ann, &note).await;
    let edited = note_with(
        &base,
        "ann",
        1,
        "edited :blob:",
        &[emoji_tag(&base, "blob", "/e/blob.png")],
    );
    store(&env.app, &ann, &edited).await;
    assert_eq!(fake.hits_of("/e/blob.png"), 1);
}
#[sqlx::test(migrations = "./migrations")]
async fn emotes_in_the_content_warning_and_poll_options_are_copied(db: PgPool) {
    let app = app_with(db.clone(), peer_config);
    let fake = Fake::start().await;
    let base = fake.base();
    fake.put_raw("/e/cw.png", "image/png", png(20, 20));
    fake.put_raw("/e/opt.png", "image/png", png(20, 20));
    let ann = remote_user(&db, &base, "ann").await;
    let mut note =
        serde_json::to_value(note_with(&base, "ann", 2, "poll", &[])).expect("serialise");
    note["type"] = json!("Question");
    note["summary"] = json!("spoiler :cw:");
    note["oneOf"] = json!([
        {"name": "yes :opt:", "replies": {"totalItems": 1}},
        {"name": "no", "replies": {"totalItems": 0}}
    ]);
    note["endTime"] = json!("2999-01-01T00:00:00Z");
    note["tag"] = json!([
        emoji_tag(&base, "cw", "/e/cw.png"),
        emoji_tag(&base, "opt", "/e/opt.png")
    ]);
    let note: Note = serde_json::from_value(note).expect("question");
    store(&app, &ann, &note).await;
    let found: Vec<String> = copies(&db, &fake.domain())
        .await
        .into_iter()
        .map(|(shortcode, ..)| shortcode)
        .collect();
    assert_eq!(found, ["cw", "opt"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_direct_message_has_its_emotes_copied(db: PgPool) {
    let app = app_with(db.clone(), peer_config);
    let fake = Fake::start().await;
    let base = fake.base();
    fake.put_raw("/e/blob.png", "image/png", png(20, 20));
    let ann = remote_user(&db, &base, "ann").await;
    insert_user(&db, "ana").await;
    let mut note = serde_json::to_value(note_with(
        &base,
        "ann",
        3,
        "psst :blob:",
        &[emoji_tag(&base, "blob", "/e/blob.png")],
    ))
    .expect("serialise");
    note["to"] = json!([format!("{ORIGIN}/users/ana")]);
    note["cc"] = json!([]);
    let note: Note = serde_json::from_value(note).expect("note");
    let stored = ingest::store_note(&app.state, &ann, &note, Context::Inbox)
        .await
        .expect("stored");
    assert_eq!(stored, Stored::Message { created: true });
    assert_eq!(copies(&db, &fake.domain()).await.len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_failed_emote_is_skipped_and_the_text_stays(db: PgPool) {
    let app = app_with(db.clone(), peer_config);
    let fake = Fake::start().await;
    let base = fake.base();
    fake.put_raw("/e/good.png", "image/png", png(20, 20));
    fake.put_raw("/e/text.png", "image/png", b"not an image at all".to_vec());
    fake.put_raw("/e/huge.png", "image/png", vec![0; 300 * 1024]);
    let ann = remote_user(&db, &base, "ann").await;
    let mut js = emoji_tag(&base, "script", "/e/good.png");
    js["icon"]["url"] = json!("javascript:alert(1)");
    let mut ours = emoji_tag(&base, "ours", "/e/good.png");
    ours["icon"]["url"] = json!(format!("{ORIGIN}/media/x.png"));
    let mut bad_name = emoji_tag(&base, "bad", "/e/good.png");
    bad_name["name"] = json!(":b-d:");
    let tags = vec![
        emoji_tag(&base, "missing", "/e/missing.png"),
        emoji_tag(&base, "text", "/e/text.png"),
        emoji_tag(&base, "huge", "/e/huge.png"),
        js,
        ours,
        bad_name,
        emoji_tag(&base, "good", "/e/good.png"),
    ];
    let note = note_with(
        &base,
        "ann",
        4,
        ":missing: :text: :huge: :script: :ours: :b-d: :good:",
        &tags,
    );
    let post = store(&app, &ann, &note).await;

    let found = copies(&db, &fake.domain()).await;
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, "good");
    let text: String = sqlx::query_scalar("select text from posts where id = $1")
        .bind(post)
        .fetch_one(&db)
        .await
        .expect("text");
    assert_eq!(text, ":missing: :text: :huge: :script: :ours: :b-d: :good:");
    assert_eq!(fake.hits_of("/e/missing.png"), 1);
    assert_eq!(fake.hits_of("/e/text.png"), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn at_most_sixteen_emoji_tags_of_an_object_are_looked_at(db: PgPool) {
    let app = app_with(db.clone(), peer_config);
    let fake = Fake::start().await;
    let base = fake.base();
    let names: Vec<String> = (1..=20).map(|n| format!("e{n:02}")).collect();
    for name in &names {
        fake.put_raw(&format!("/e/{name}.png"), "image/png", png(8, 8));
    }
    let ann = remote_user(&db, &base, "ann").await;
    let tags: Vec<Value> = names
        .iter()
        .map(|name| emoji_tag(&base, name, &format!("/e/{name}.png")))
        .collect();
    let text = names
        .iter()
        .map(|name| format!(":{name}:"))
        .collect::<Vec<_>>()
        .join(" ");
    store(&app, &ann, &note_with(&base, "ann", 5, &text, &tags)).await;
    assert_eq!(copies(&db, &fake.domain()).await.len(), 16);
    for name in &names[..16] {
        assert_eq!(fake.hits_of(&format!("/e/{name}.png")), 1, "{name}");
    }
    for name in &names[16..] {
        assert_eq!(fake.hits_of(&format!("/e/{name}.png")), 0, "{name}");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn an_actor_has_the_emotes_of_its_name_and_bio_copied(db: PgPool) {
    let app = app_with(db.clone(), peer_config);
    let fake = Fake::start().await;
    let base = fake.base();
    fake.put_raw("/e/rainbow.png", "image/png", png(40, 40));
    fake.put_raw("/e/star.png", "image/png", png(40, 40));
    let tags = json!([
        emoji_tag(&base, "rainbow", "/e/rainbow.png"),
        emoji_tag(&base, "star", "/e/star.png"),
        {"type": "Hashtag", "href": format!("{base}/tags/x"), "name": "#x"},
    ]);
    fake.add_actor("ann", |actor| {
        actor["name"] = json!("Ann :rainbow:");
        actor["summary"] = json!("<p>I like :star:</p>");
        actor["tag"] = tags;
    });
    let row = lookup::resolve_acct(&app.state, None, "ann", &fake.domain())
        .await
        .expect("resolved");
    assert_eq!(row.display_name, "Ann :rainbow:");
    assert_eq!(row.bio, "I like :star:");
    let found: Vec<String> = copies(&db, &fake.domain())
        .await
        .into_iter()
        .map(|(shortcode, ..)| shortcode)
        .collect();
    assert_eq!(found, ["rainbow", "star"]);

    let mut viewer = app.person(&db, "viewer").await;
    let handle = format!("ann@{}", fake.domain());
    let card = viewer.browser.get(&format!("/users/{handle}")).await.ok();
    let shortcodes: Vec<&str> = card["emojis"]
        .as_array()
        .expect("emojis")
        .iter()
        .map(|emoji| emoji["shortcode"].as_str().expect("shortcode"))
        .collect();
    assert_eq!(shortcodes, ["rainbow", "star"], "{card}");
}

#[sqlx::test(migrations = "./migrations")]
async fn an_emote_is_never_fetched_from_a_private_address(db: PgPool) {
    // Plain http is allowed here, private networks are not: the fake lives on loopback.
    let app = app_with(db.clone(), |var| match var {
        "FEDERATION_ALLOW_HTTP" => Some("1".to_owned()),
        "DATA_DIR" => Some(DATA.clone()),
        _ => None,
    });
    let fake = Fake::start().await;
    let base = fake.base();
    fake.put_raw("/e/blob.png", "image/png", png(20, 20));
    let ann = remote_user(&db, &base, "ann").await;
    let note = note_with(
        &base,
        "ann",
        6,
        "hi :blob:",
        &[emoji_tag(&base, "blob", "/e/blob.png")],
    );
    store(&app, &ann, &note).await;
    assert_eq!(fake.hits_of("/e/blob.png"), 0);
    assert!(copies(&db, &fake.domain()).await.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn a_changed_icon_address_replaces_the_copy_and_the_same_address_fetches_nothing(db: PgPool) {
    let env = Env::with(db.clone(), peer_config);
    let fake = Fake::start().await;
    let base = fake.base();
    fake.put_raw("/e/blob-a.png", "image/png", png(20, 20));
    fake.put_raw("/e/blob-b.png", "image/png", png(40, 40));
    let ann = remote_user(&db, &base, "ann").await;
    let mut viewer = env.app.person(&db, "viewer").await;

    let first = note_with(
        &base,
        "ann",
        1,
        "one :blob:",
        &[emoji_tag(&base, "blob", "/e/blob-a.png")],
    );
    let post = store(&env.app, &ann, &first).await;
    let found = copies(&db, &fake.domain()).await;
    assert_eq!((found[0].2, found[0].3), (20, 20), "{found:?}");
    let thread = viewer.browser.get(&format!("/posts/{post}")).await.ok();
    let old_url = thread["post"]["emojis"][0]["url"]
        .as_str()
        .expect("url")
        .to_owned();
    let source = emoji::remote_source(&db, &fake.domain(), "blob")
        .await
        .expect("source");
    assert_eq!(source, Some(format!("{base}/e/blob-a.png")));

    // The same address again: nothing is fetched.
    store(&env.app, &ann, &first).await;
    assert_eq!(fake.hits_of("/e/blob-a.png"), 1);

    // The sender replaced the image: a new address replaces the copy.
    let second = note_with(
        &base,
        "ann",
        2,
        "two :blob:",
        &[emoji_tag(&base, "blob", "/e/blob-b.png")],
    );
    let later = store(&env.app, &ann, &second).await;
    assert_eq!(fake.hits_of("/e/blob-b.png"), 1);
    let found = copies(&db, &fake.domain()).await;
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!((found[0].2, found[0].3), (40, 40), "{found:?}");
    for id in [post, later] {
        let thread = viewer.browser.get(&format!("/posts/{id}")).await.ok();
        let url = thread["post"]["emojis"][0]["url"].as_str().expect("url");
        assert!(url.starts_with("/media/"), "{url}");
        assert_ne!(url, old_url, "both posts show the new image");
    }

    // And the new address is now the one that is remembered.
    store(&env.app, &ann, &second).await;
    assert_eq!(fake.hits_of("/e/blob-b.png"), 1);
    assert_eq!(fake.hits_of("/e/blob-a.png"), 1);
}
