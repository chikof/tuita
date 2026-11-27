//! The recorded Mastodon, Misskey and GoToSocial payloads of `tuita-core/tests/fixtures/ap`
//! stored through `federation::{lookup, ingest}` into a real database.
//!
//! Each fixture names its host as `mastodon.example`, `misskey.example` or `gts.example`. A test
//! rewrites those to three loopback servers that serve the fixtures and a tiny PNG for every
//! picture, so avatars and attachments are really fetched, re-encoded and stored.
#![allow(missing_docs)]

mod common;

use std::collections::HashMap;
use std::io::Cursor;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};

use axum::Router;
use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use sqlx::PgPool;
use time::OffsetDateTime;
use tokio::net::TcpListener;
use tuita_core::ap::{self, Actor, IdOrObject, Note, Tombstone};
use tuita_server::db::users::UserRow;
use tuita_server::federation::ingest::{self, Context, Stored};
use tuita_server::federation::{FedError, lookup};
use uuid::Uuid;

use common::{App, ORIGIN, app_with, insert_user};

const BEA_POST: &str = "0192a4b6-7c3e-7f10-9a42-3d5c8e1f2a07";

// ------------------------------------------------------------------ fake servers

#[derive(Default)]
struct Inner {
    docs: Mutex<HashMap<String, Value>>,
    hits: Mutex<Vec<String>>,
}

struct Site {
    addr: SocketAddr,
    inner: Arc<Inner>,
}

static PNG: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
        32,
        24,
        image::Rgb([40, 120, 200]),
    ));
    let mut out = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .expect("png");
    out
});

async fn serve(State(inner): State<Arc<Inner>>, uri: Uri) -> Response {
    inner.hits.lock().expect("lock").push(uri.path().to_owned());
    let full = uri
        .path_and_query()
        .map_or(String::new(), |pq| pq.as_str().to_owned());
    let doc = {
        let docs = inner.docs.lock().expect("lock");
        docs.get(&full).or_else(|| docs.get(uri.path())).cloned()
    };
    if let Some(doc) = doc {
        return (
            [(CONTENT_TYPE, "application/activity+json")],
            doc.to_string(),
        )
            .into_response();
    }
    if [".jpg", ".png", ".webp", ".gif"]
        .iter()
        .any(|ext| uri.path().ends_with(ext))
    {
        return ([(CONTENT_TYPE, "image/png")], PNG.clone()).into_response();
    }
    StatusCode::NOT_FOUND.into_response()
}

impl Site {
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

    fn authority(&self) -> String {
        self.addr.to_string()
    }

    fn put(&self, path: &str, doc: Value) {
        self.inner
            .docs
            .lock()
            .expect("lock")
            .insert(path.to_owned(), doc);
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
}

/// One loopback server per software.
struct World {
    mastodon: Site,
    misskey: Site,
    gts: Site,
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tuita-core/tests/fixtures/ap")
}

impl World {
    async fn start() -> Self {
        Self {
            mastodon: Site::start().await,
            misskey: Site::start().await,
            gts: Site::start().await,
        }
    }

    /// A fixture with its three fake hosts pointed at the loopback servers.
    fn load(&self, name: &str) -> Value {
        let mut text = std::fs::read_to_string(fixtures_dir().join(format!("{name}.json")))
            .expect("fixture file");
        for (host, site) in [
            ("mastodon.example", &self.mastodon),
            ("misskey.example", &self.misskey),
            ("gts.example", &self.gts),
        ] {
            text = text.replace(&format!("https://{host}"), &site.base());
        }
        serde_json::from_str(&text).expect("fixture json")
    }
}

// ------------------------------------------------------------------ environment

static DATA: LazyLock<String> = LazyLock::new(|| {
    std::env::temp_dir()
        .join(format!("tuita-ap-fixtures-{}", Uuid::now_v7()))
        .display()
        .to_string()
});

fn open(var: &str) -> Option<String> {
    match var {
        "FEDERATION_ALLOW_HTTP" | "FEDERATION_ALLOW_PRIVATE_NETS" => Some("1".to_owned()),
        "DATA_DIR" => Some(DATA.clone()),
        _ => None,
    }
}

struct Env {
    app: App,
    db: PgPool,
    world: World,
    bea: Uuid,
}

impl Env {
    /// A test instance with the local account `bea` and her post `BEA_POST`.
    async fn new(db: PgPool) -> Self {
        let app = app_with(db.clone(), open);
        let bea = insert_user(&db, "bea").await;
        sqlx::query("insert into posts (id, author_id, text) values ($1, $2, 'bea writes')")
            .bind(Uuid::parse_str(BEA_POST).expect("uuid"))
            .bind(bea)
            .execute(&db)
            .await
            .expect("local post");
        Self {
            app,
            db,
            world: World::start().await,
            bea,
        }
    }

    fn load(&self, name: &str) -> Value {
        self.world.load(name)
    }

    /// The actor fixture `name`, checked and stored like a fetched actor.
    async fn actor(&self, name: &str) -> UserRow {
        let actor: Actor = serde_json::from_value(self.load(name)).expect("actor");
        let origin = actor.id.clone();
        let checked = lookup::check_actor(&self.app.state, actor, &origin, None).expect("checked");
        lookup::store_actor(&self.app.state, &checked)
            .await
            .expect("stored")
            .expect("not blocked")
    }

    async fn store(&self, author: &UserRow, note: &Value, ctx: Context) -> Stored {
        let note: Note = serde_json::from_value(note.clone()).expect("note");
        ingest::store_note(&self.app.state, author, &note, ctx)
            .await
            .expect("store_note")
    }

    /// Stores a note that must become a new post.
    async fn post_of(&self, author: &UserRow, note: &Value, ctx: Context) -> Uuid {
        match self.store(author, note, ctx).await {
            Stored::Post { id, created: true } => id,
            other => panic!("expected a new post, got {other:?}"),
        }
    }

    async fn row(&self, id: Uuid) -> PostRow {
        sqlx::query_as(
            "select text, cw, sensitive, visibility, ap_id, url, created_at, edited_at,
                    reply_to_id, root_id, quote_of_id, author_id, deleted_at
               from posts where id = $1",
        )
        .bind(id)
        .fetch_one(&self.db)
        .await
        .expect("post row")
    }

    async fn tags(&self, id: Uuid) -> Vec<String> {
        sqlx::query_scalar("select tag from post_tags where post_id = $1 order by tag")
            .bind(id)
            .fetch_all(&self.db)
            .await
            .expect("tags")
    }

    async fn mentions(&self, id: Uuid) -> Vec<String> {
        sqlx::query_scalar(
            "select u.handle from mentions m join users u on u.id = m.user_id
              where m.post_id = $1 order by u.handle",
        )
        .bind(id)
        .fetch_all(&self.db)
        .await
        .expect("mentions")
    }

    /// `(alt, kind, mime)` of the attachments in order.
    async fn media(&self, id: Uuid) -> Vec<(String, String, String)> {
        sqlx::query_as(
            "select m.alt, m.kind, m.mime from post_media pm join media m on m.id = pm.media_id
              where pm.post_id = $1 order by pm.position",
        )
        .bind(id)
        .fetch_all(&self.db)
        .await
        .expect("media")
    }

    async fn notifications(&self, id: Uuid) -> Vec<(Uuid, String)> {
        sqlx::query_as(
            "select user_id, kind from notifications where post_id = $1 order by kind, user_id",
        )
        .bind(id)
        .fetch_all(&self.db)
        .await
        .expect("notifications")
    }
}

#[derive(Debug, sqlx::FromRow)]
struct PostRow {
    text: String,
    cw: String,
    sensitive: bool,
    visibility: String,
    ap_id: Option<String>,
    url: Option<String>,
    created_at: OffsetDateTime,
    edited_at: Option<OffsetDateTime>,
    reply_to_id: Option<Uuid>,
    root_id: Option<Uuid>,
    quote_of_id: Option<Uuid>,
    author_id: Uuid,
    deleted_at: Option<OffsetDateTime>,
}

fn time(text: &str) -> OffsetDateTime {
    ap::parse_time(text).expect("time")
}

fn string(value: &Value, key: &str) -> String {
    value[key].as_str().expect("string field").to_owned()
}

// ------------------------------------------------------------------ actors

#[derive(Debug, sqlx::FromRow)]
struct ActorExtras {
    outbox_url: Option<String>,
    followers_url: Option<String>,
    featured_url: Option<String>,
    public_key_pem: Option<String>,
    created_at: OffsetDateTime,
}

async fn extras(env: &Env, id: Uuid) -> ActorExtras {
    sqlx::query_as(
        "select outbox_url, followers_url, featured_url, public_key_pem, created_at
           from users where id = $1",
    )
    .bind(id)
    .fetch_one(&env.db)
    .await
    .expect("extras")
}

async fn media_kinds(env: &Env, owner: Uuid) -> Vec<String> {
    sqlx::query_scalar("select kind from media where owner_id = $1")
        .bind(owner)
        .fetch_all(&env.db)
        .await
        .expect("media")
}

#[sqlx::test(migrations = "./migrations")]
async fn a_mastodon_person_is_stored_with_fields_inboxes_and_pictures(db: PgPool) {
    let env = Env::new(db).await;
    let base = env.world.mastodon.base();
    let row = env.actor("mastodon-actor-person").await;
    assert_eq!(row.handle, "ada");
    assert_eq!(
        row.domain.as_deref(),
        Some(env.world.mastodon.authority().as_str())
    );
    assert_eq!(
        row.ap_id.as_deref(),
        Some(format!("{base}/users/ada").as_str())
    );
    assert_eq!(row.display_name, "Ada Example :blobcat:");
    assert_eq!(row.bio, "Writes Rust, walks in woods. #rustlang");
    assert_eq!(row.website, "https://ada.example/");
    assert_eq!((row.is_bot, row.is_org), (Some(false), Some(false)));
    assert_eq!(row.manually_approves, Some(false));
    assert_eq!(row.discoverable, Some(true));
    assert_eq!(
        row.inbox_url.as_deref(),
        Some(format!("{base}/users/ada/inbox").as_str())
    );
    assert_eq!(
        row.shared_inbox_url.as_deref(),
        Some(format!("{base}/inbox").as_str())
    );
    assert_eq!(row.url.as_deref(), Some(format!("{base}/@ada").as_str()));
    assert!(row.avatar_id.is_some() && row.banner_id.is_some());
    // Avatar, banner and the `:blobcat:` emoji the profile text uses.
    assert_eq!(media_kinds(&env, row.id).await, ["image", "image", "image"]);
    assert_eq!(
        env.world
            .mastodon
            .hits_of("/system/accounts/avatars/000/000/001/original/ada.jpg"),
        1
    );
    let more = extras(&env, row.id).await;
    assert_eq!(
        more.outbox_url.as_deref(),
        Some(format!("{base}/users/ada/outbox").as_str())
    );
    assert_eq!(
        more.followers_url.as_deref(),
        Some(format!("{base}/users/ada/followers").as_str())
    );
    assert_eq!(
        more.featured_url.as_deref(),
        Some(format!("{base}/users/ada/collections/featured").as_str())
    );
    assert!(
        more.public_key_pem
            .as_deref()
            .is_some_and(|pem| pem.starts_with("-----BEGIN PUBLIC KEY-----"))
    );
    assert_eq!(more.created_at, time("2022-11-09T00:00:00Z"));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_locked_mastodon_bot_with_one_field_and_two_icons(db: PgPool) {
    let env = Env::new(db).await;
    let row = env.actor("mastodon-actor-locked-bot").await;
    assert_eq!(row.handle, "weatherbot");
    assert_eq!(row.display_name, "Weather Bot");
    assert_eq!(row.bio, "Posts the forecast. Follows are approved by hand.");
    assert_eq!(row.website, "");
    assert_eq!(row.is_bot, Some(true));
    assert_eq!(row.manually_approves, Some(true));
    assert_eq!(row.discoverable, Some(false));
    assert!(row.avatar_id.is_some(), "the first icon is copied");
    assert!(row.banner_id.is_none());
    assert_eq!(media_kinds(&env, row.id).await, ["image"]);
    let base = env.world.mastodon.base();
    assert_eq!(
        env.world
            .mastodon
            .hits_of("/system/accounts/avatars/000/000/002/original/bot.webp"),
        1
    );
    assert_eq!(
        env.world
            .mastodon
            .hits_of("/system/accounts/avatars/000/000/002/small/bot.png"),
        0,
        "the second icon of {base} is not fetched"
    );
    assert_eq!(extras(&env, row.id).await.featured_url, None);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_misskey_person_is_stored_with_its_shared_inbox_and_fields(db: PgPool) {
    let env = Env::new(db).await;
    let base = env.world.misskey.base();
    let row = env.actor("misskey-actor-person").await;
    assert_eq!(row.handle, "ada");
    assert_eq!(
        row.ap_id.as_deref(),
        Some(format!("{base}/users/9a0example01").as_str())
    );
    assert_eq!(row.display_name, "Ada :ablobcat:");
    assert!(row.bio.starts_with("Hello! I like #cats"), "{}", row.bio);
    assert!(
        row.bio.ends_with("and https://home.example/ada"),
        "{}",
        row.bio
    );
    assert_eq!(row.website, "https://home.example/ada");
    assert_eq!(row.manually_approves, Some(false));
    assert_eq!(row.discoverable, Some(true));
    assert_eq!(
        row.shared_inbox_url.as_deref(),
        Some(format!("{base}/inbox").as_str())
    );
    assert!(row.avatar_id.is_some() && row.banner_id.is_some());
    let more = extras(&env, row.id).await;
    assert_eq!(
        more.featured_url.as_deref(),
        Some(format!("{base}/users/9a0example01/collections/featured").as_str())
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn gotosocial_accounts_are_stored_without_a_shared_inbox(db: PgPool) {
    let env = Env::new(db).await;
    let base = env.world.gts.base();
    let person = env.actor("gts-actor-person").await;
    assert_eq!(person.display_name, "ada :rainbow:");
    assert_eq!(person.bio, "i post about things that concern me");
    assert_eq!(person.website, "https://ada.example/");
    assert_eq!(person.shared_inbox_url, None);
    assert_eq!(
        person.inbox_url.as_deref(),
        Some(format!("{base}/users/ada/inbox").as_str())
    );
    assert_eq!(person.manually_approves, Some(false));
    assert_eq!(person.discoverable, Some(true));
    assert!(person.avatar_id.is_some() && person.banner_id.is_some());
    assert_eq!(
        extras(&env, person.id).await.created_at,
        time("2022-09-29T13:57:24Z")
    );

    let service = env.actor("gts-actor-service").await;
    assert_eq!(service.handle, "1happyturtle");
    assert_eq!(service.is_bot, Some(true));
    assert_eq!(service.manually_approves, Some(true));
    assert_eq!(service.discoverable, Some(false));
    assert!(service.avatar_id.is_none() && service.banner_id.is_none());
    assert_eq!(service.website, "");
}

#[sqlx::test(migrations = "./migrations")]
async fn an_actor_that_breaks_a_rule_is_refused(db: PgPool) {
    let env = Env::new(db).await;
    let state = &env.app.state;
    let value = env.load("mastodon-actor-person");
    let read = |value: &Value| -> Actor { serde_json::from_value(value.clone()).expect("actor") };
    let id = string(&value, "id");

    let served_elsewhere =
        lookup::check_actor(state, read(&value), "https://evil.example/users/ada", None);
    assert!(matches!(served_elsewhere, Err(FedError::Unreadable(_))));

    let mut application = value.clone();
    application["type"] = json!("Application");
    assert!(matches!(
        lookup::check_actor(state, read(&application), &id, None),
        Err(FedError::NotFound)
    ));

    let mut other_owner = value.clone();
    other_owner["publicKey"]["owner"] =
        json!(format!("{}/users/mallory", env.world.mastodon.base()));
    assert!(matches!(
        lookup::check_actor(state, read(&other_owner), &id, None),
        Err(FedError::Unreadable(_))
    ));

    let mut no_inbox = value.clone();
    no_inbox["inbox"] = json!("javascript:alert(1)");
    assert!(matches!(
        lookup::check_actor(state, read(&no_inbox), &id, None),
        Err(FedError::Unreadable(_))
    ));

    let mut bad_key = value.clone();
    bad_key["publicKey"]["publicKeyPem"] =
        json!("-----BEGIN PUBLIC KEY-----\nAAAA\n-----END PUBLIC KEY-----\n");
    assert!(matches!(
        lookup::check_actor(state, read(&bad_key), &id, None),
        Err(FedError::Unreadable(_))
    ));

    let wrong_handle = lookup::check_actor(
        state,
        read(&value),
        &id,
        Some(("grace", &env.world.mastodon.authority())),
    );
    assert!(matches!(wrong_handle, Err(FedError::Unreadable(_))));
    lookup::check_actor(
        state,
        read(&value),
        &id,
        Some(("ADA", &env.world.mastodon.authority())),
    )
    .expect("the handle matches without regard to case");
}

// ------------------------------------------------------------------ notes

#[sqlx::test(migrations = "./migrations")]
async fn a_mastodon_note_is_stored_with_text_tags_mentions_and_pictures(db: PgPool) {
    let env = Env::new(db).await;
    let ada = env.actor("mastodon-actor-person").await;
    let value = env.load("mastodon-note-public");
    let id = env.post_of(&ada, &value, Context::Inbox).await;

    let row = env.row(id).await;
    assert_eq!(
        row.text,
        "Good morning @bea@social.example! Photos from the walk #rustlang #Autumn :blobcat:\n\nSecond paragraph & a line\nbreak."
    );
    assert_eq!(row.cw, "");
    assert!(!row.sensitive);
    assert_eq!(row.visibility, "public");
    assert_eq!(row.ap_id, Some(string(&value, "id")));
    assert_eq!(row.url, Some(string(&value, "url")));
    assert_eq!(row.created_at, time("2026-11-08T09:30:12Z"));
    assert_eq!(row.edited_at, None);
    assert_eq!((row.reply_to_id, row.quote_of_id), (None, None));
    assert_eq!(row.author_id, ada.id);
    assert_eq!(env.tags(id).await, ["autumn", "rustlang"]);
    assert_eq!(env.mentions(id).await, ["bea"]);
    assert_eq!(
        env.media(id).await,
        [
            (
                "A path through a wood in autumn".to_owned(),
                "image".to_owned(),
                "image/webp".to_owned()
            ),
            (String::new(), "image".to_owned(), "image/webp".to_owned()),
        ]
    );
    assert_eq!(
        env.notifications(id).await,
        [(env.bea, "mention".to_owned())]
    );
    assert_eq!(
        env.world
            .mastodon
            .hits_of("/system/media_attachments/files/113/200/000/000/000/001/original/walk1.jpg"),
        1
    );

    let again = env.store(&ada, &value, Context::Inbox).await;
    assert_eq!(
        again,
        Stored::Post { id, created: false },
        "a replay updates and never duplicates"
    );
    let rows: i64 = sqlx::query_scalar("select count(*) from posts where ap_id = $1")
        .bind(string(&value, "id"))
        .fetch_one(&env.db)
        .await
        .expect("count");
    assert_eq!(rows, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_mastodon_reply_with_a_content_warning_is_unlisted_and_linked_to_its_parent(db: PgPool) {
    let env = Env::new(db).await;
    let ada = env.actor("mastodon-actor-person").await;
    let parent = env
        .post_of(&ada, &env.load("mastodon-note-public"), Context::Inbox)
        .await;
    let value = env.load("mastodon-note-cw");
    let id = env.post_of(&ada, &value, Context::Lookup).await;
    let row = env.row(id).await;
    assert_eq!(row.cw, "Spoilers: season finale & more");
    assert!(row.sensitive);
    assert_eq!(row.visibility, "unlisted", "Public in cc");
    assert_eq!(row.text, "The ending was great.");
    assert_eq!(row.edited_at, Some(time("2026-11-08T10:05:30Z")));
    assert_eq!(row.reply_to_id, Some(parent));
    assert_eq!(row.root_id, Some(parent));
    assert_eq!(env.media(id).await.len(), 1);
    assert!(env.tags(id).await.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn a_followers_only_mastodon_note_is_stored_for_followers(db: PgPool) {
    let env = Env::new(db).await;
    let ada = env.actor("mastodon-actor-person").await;
    let id = env
        .post_of(
            &ada,
            &env.load("mastodon-note-followers-only"),
            Context::Lookup,
        )
        .await;
    let row = env.row(id).await;
    assert_eq!(row.visibility, "followers");
    assert_eq!(row.text, "Followers only.");
    assert!(env.media(id).await.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn a_direct_mastodon_note_to_a_local_account_is_a_message(db: PgPool) {
    let env = Env::new(db).await;
    let ada = env.actor("mastodon-actor-person").await;
    let value = env.load("mastodon-note-direct");
    let stored = env.store(&ada, &value, Context::Inbox).await;
    assert_eq!(stored, Stored::Message { created: true });
    let rows: Vec<(String, Uuid, Uuid, Option<String>)> =
        sqlx::query_as("select text, sender_id, recipient_id, ap_id from messages")
            .fetch_all(&env.db)
            .await
            .expect("messages");
    assert_eq!(
        rows,
        [(
            "only you should read this.".to_owned(),
            ada.id,
            env.bea,
            Some(string(&value, "id")),
        )]
    );
    let posts: i64 = sqlx::query_scalar("select count(*) from posts where ap_id = $1")
        .bind(string(&value, "id"))
        .fetch_one(&env.db)
        .await
        .expect("count");
    assert_eq!(posts, 0, "a message is not a post");
    assert_eq!(
        env.store(&ada, &value, Context::Inbox).await,
        Stored::Message { created: false }
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn mastodon_polls_keep_their_options_counts_and_end(db: PgPool) {
    let env = Env::new(db).await;
    let ada = env.actor("mastodon-actor-person").await;
    let one = env
        .post_of(&ada, &env.load("mastodon-question-oneof"), Context::Lookup)
        .await;
    let many = env
        .post_of(
            &ada,
            &env.load("mastodon-question-anyof-closed"),
            Context::Lookup,
        )
        .await;
    let poll = |id: Uuid| {
        let db = env.db.clone();
        async move {
            sqlx::query_as::<_, (Vec<String>, bool, Vec<i32>, i32, OffsetDateTime)>(
                "select options, multiple, counts, voters, ends_at from polls where post_id = $1",
            )
            .bind(id)
            .fetch_one(&db)
            .await
            .expect("poll")
        }
    };
    assert_eq!(
        poll(one).await,
        (
            vec!["Tea".to_owned(), "Coffee".to_owned()],
            false,
            vec![4, 3],
            7,
            time("2026-11-09T13:00:00Z"),
        )
    );
    assert_eq!(env.row(one).await.text, "Tea or coffee?");
    assert_eq!(
        poll(many).await,
        (
            vec!["Rust".to_owned(), "Go".to_owned(), "Zig".to_owned()],
            true,
            vec![8, 5, 0],
            10,
            time("2026-11-02T08:00:00Z"),
        )
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_misskey_note_stores_its_quote_mention_hashtag_and_picture(db: PgPool) {
    let env = Env::new(db).await;
    let mastodon = env.actor("mastodon-actor-person").await;
    let quoted = env
        .post_of(&mastodon, &env.load("mastodon-note-public"), Context::Inbox)
        .await;
    let ada = env.actor("misskey-actor-person").await;
    let value = env.load("misskey-note-public");
    let id = env.post_of(&ada, &value, Context::Inbox).await;
    let row = env.row(id).await;
    let quote_link = string(&env.load("mastodon-note-public"), "id");
    assert!(row.text.starts_with("Look at this #Cats"), "{}", row.text);
    assert!(
        row.text.contains("from @bea@social.example :ablobcat:"),
        "{}",
        row.text
    );
    assert!(
        row.text.ends_with(&format!("RE: {quote_link}")),
        "{}",
        row.text
    );
    assert_eq!(row.quote_of_id, Some(quoted));
    assert_eq!(row.visibility, "public");
    assert_eq!(row.created_at, time("2026-11-08T08:00:00Z"));
    assert_eq!(env.tags(id).await, ["cats"]);
    assert_eq!(env.mentions(id).await, ["bea"]);
    assert_eq!(
        env.media(id).await,
        [(
            "A cat asleep on a sofa".to_owned(),
            "image".to_owned(),
            "image/webp".to_owned()
        )]
    );
    assert_eq!(
        env.notifications(id).await,
        [(env.bea, "mention".to_owned())]
    );
    let quotes: i32 = sqlx::query_scalar("select quote_count from posts where id = $1")
        .bind(quoted)
        .fetch_one(&env.db)
        .await
        .expect("count");
    assert_eq!(quotes, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn misskey_content_warnings_and_polls(db: PgPool) {
    let env = Env::new(db).await;
    let ada = env.actor("misskey-actor-person").await;

    let home = env
        .post_of(&ada, &env.load("misskey-note-cw"), Context::Lookup)
        .await;
    let row = env.row(home).await;
    assert_eq!(
        (row.cw.as_str(), row.sensitive),
        ("Mildly spicy opinions", true)
    );
    assert_eq!(
        row.visibility, "unlisted",
        "home visibility: followers in to, Public in cc"
    );
    assert_eq!(row.text, "Pineapple belongs on pizza.");

    let empty = env
        .post_of(&ada, &env.load("misskey-note-cw-empty"), Context::Lookup)
        .await;
    let row = env.row(empty).await;
    assert_eq!(
        row.cw, "\u{200b}",
        "Misskey's empty warning stays a zero-width space"
    );
    assert!(row.sensitive);
    assert_eq!(row.visibility, "public");

    let poll = env
        .post_of(&ada, &env.load("misskey-question-oneof"), Context::Lookup)
        .await;
    let (options, multiple, counts, voters, ends): (
        Vec<String>,
        bool,
        Vec<i32>,
        i32,
        OffsetDateTime,
    ) = sqlx::query_as(
        "select options, multiple, counts, voters, ends_at from polls where post_id = $1",
    )
    .bind(poll)
    .fetch_one(&env.db)
    .await
    .expect("poll");
    assert_eq!(options, ["Apple", "Banana", "Cherry"]);
    assert!(!multiple);
    assert_eq!(counts, [2, 5, 0]);
    assert_eq!(voters, 7, "no votersCount: the votes are added up");
    assert_eq!(ends, time("2026-11-10T09:30:00Z"));
}

#[sqlx::test(migrations = "./migrations")]
async fn gotosocial_notes_with_single_values_and_a_reply(db: PgPool) {
    let env = Env::new(db).await;
    let ada = env.actor("gts-actor-person").await;

    let public = env.load("gts-note-public");
    let id = env.post_of(&ada, &public, Context::Lookup).await;
    let row = env.row(id).await;
    assert_eq!(
        row.text,
        "Welcome to the #welcome thread. This is an example note."
    );
    assert_eq!(row.cw, "", "an empty summary is no warning");
    assert!(!row.sensitive);
    assert_eq!(row.visibility, "public");
    assert_eq!(row.created_at, time("2021-11-20T13:32:16Z"));
    assert_eq!(row.url, Some(string(&public, "url")));
    assert_eq!(env.tags(id).await, ["welcome"]);
    let media = env.media(id).await;
    assert_eq!(media.len(), 2);
    assert_eq!(
        media[0].0,
        "Black and white image of some 50's style text saying: Welcome On Board"
    );
    assert_eq!(media[1].0, "A rainbow over a hill");

    let single = env
        .post_of(&ada, &env.load("gts-note-single-values"), Context::Lookup)
        .await;
    let row = env.row(single).await;
    assert_eq!(row.text, "here's a stinky creature -> :shocked_pikachu:");
    assert_eq!(row.visibility, "public");
    assert_eq!(env.media(single).await.len(), 1);
    assert!(
        env.tags(single).await.is_empty(),
        "an emoji is not a hashtag"
    );

    let reply = env
        .post_of(&ada, &env.load("gts-note-reply-mention"), Context::Inbox)
        .await;
    let row = env.row(reply).await;
    let parent = Uuid::parse_str(BEA_POST).expect("uuid");
    assert_eq!((row.reply_to_id, row.root_id), (Some(parent), Some(parent)));
    assert_eq!(
        row.text,
        "@bea@social.example @zed@other.example thanks both!"
    );
    assert_eq!(
        env.mentions(reply).await,
        ["bea"],
        "only local accounts are stored"
    );
    assert_eq!(
        env.notifications(reply).await,
        [(env.bea, "reply".to_owned())],
        "the author of the parent hears of a reply once"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_create_activity_from_mastodon_replies_to_a_local_post(db: PgPool) {
    let env = Env::new(db).await;
    let ada = env.actor("mastodon-actor-person").await;
    let create = env.load("mastodon-activity-create");
    let id = env.post_of(&ada, &create["object"], Context::Inbox).await;
    let row = env.row(id).await;
    assert_eq!(row.text, "@bea@social.example agreed!");
    assert_eq!(
        row.reply_to_id,
        Some(Uuid::parse_str(BEA_POST).expect("uuid"))
    );
    assert_eq!(row.visibility, "public");
    assert_eq!(env.notifications(id).await, [(env.bea, "reply".to_owned())]);
}

#[sqlx::test(migrations = "./migrations")]
async fn odd_shapes_of_author_audience_and_type_still_store(db: PgPool) {
    let env = Env::new(db).await;
    let ada = env.actor("mastodon-actor-person").await;

    let array = env
        .post_of(
            &ada,
            &env.load("generic-note-attributedto-array"),
            Context::Lookup,
        )
        .await;
    assert_eq!(
        env.row(array).await.author_id,
        ada.id,
        "the first of two authors"
    );
    assert_eq!(env.row(array).await.visibility, "public");
    let object = env
        .post_of(
            &ada,
            &env.load("generic-note-attributedto-object"),
            Context::Lookup,
        )
        .await;
    assert_eq!(env.row(object).await.author_id, ada.id);

    let nobody = env
        .post_of(
            &ada,
            &env.load("generic-note-addressing-missing"),
            Context::Lookup,
        )
        .await;
    let row = env.row(nobody).await;
    assert_eq!(row.visibility, "direct", "no to and no cc reaches nobody");
    assert_eq!(row.text, "No to and no cc.");

    let article_value = env.load("generic-note-article");
    let article = env.post_of(&ada, &article_value, Context::Lookup).await;
    let row = env.row(article).await;
    assert_eq!(
        row.text,
        "Long form text, first paragraph.\n\nSecond paragraph."
    );
    assert_eq!(row.cw, "A short essay.");
    assert_eq!(
        row.url,
        Some(format!(
            "{}/articles/rust-in-the-woods",
            env.world.mastodon.base()
        ))
    );
    assert_eq!(env.media(article).await.len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_note_by_someone_else_or_with_a_foreign_id_is_refused(db: PgPool) {
    let env = Env::new(db).await;
    let gts = env.actor("gts-actor-person").await;
    let mastodon_note: Note =
        serde_json::from_value(env.load("mastodon-note-public")).expect("note");
    let got = ingest::store_note(&env.app.state, &gts, &mastodon_note, Context::Lookup).await;
    assert!(matches!(got, Err(FedError::Unreadable(_))), "{got:?}");

    let ada = env.actor("mastodon-actor-person").await;
    let mut claims_local = env.load("mastodon-note-public");
    claims_local["id"] = json!(format!("{ORIGIN}/users/ada/statuses/{BEA_POST}"));
    let note: Note = serde_json::from_value(claims_local).expect("note");
    let got = ingest::store_note(&env.app.state, &ada, &note, Context::Lookup).await;
    assert!(matches!(got, Err(FedError::Unreadable(_))), "{got:?}");

    let posts: i64 = sqlx::query_scalar("select count(*) from posts where ap_id is not null")
        .fetch_one(&env.db)
        .await
        .expect("count");
    assert_eq!(posts, 0);
}

// ------------------------------------------------------------------ updates and deletes

#[sqlx::test(migrations = "./migrations")]
async fn an_update_edits_the_post_and_a_tombstone_deletes_it(db: PgPool) {
    let env = Env::new(db).await;
    let ada = env.actor("mastodon-actor-person").await;
    let gts = env.actor("gts-actor-person").await;
    let id = env
        .post_of(&ada, &env.load("mastodon-note-public"), Context::Inbox)
        .await;

    let update = env.load("mastodon-activity-update-note");
    let edited: Note = serde_json::from_value(update["object"].clone()).expect("note");
    let stored = ingest::apply_note_update(&env.app.state, &ada, &edited)
        .await
        .expect("update");
    assert_eq!(stored, Stored::Post { id, created: false });
    let row = env.row(id).await;
    assert_eq!(row.text, "Good morning! (edited: fixed a typo)");
    assert_eq!(row.edited_at, Some(time("2026-11-08T13:30:00Z")));
    assert!(env.tags(id).await.is_empty());
    assert_eq!(
        env.media(id).await.len(),
        2,
        "an edit does not touch the pictures"
    );
    let history: i64 = sqlx::query_scalar("select count(*) from post_edits where post_id = $1")
        .bind(id)
        .fetch_one(&env.db)
        .await
        .expect("history");
    assert_eq!(history, 1);

    let delete = env.load("mastodon-activity-delete-tombstone");
    let tombstone: IdOrObject<Tombstone> =
        serde_json::from_value(delete["object"].clone()).expect("tombstone");
    let target = tombstone.id().expect("id").to_owned();
    assert_eq!(Some(target.as_str()), row.ap_id.as_deref());
    assert!(
        !ingest::delete_note_by_ap_id(&env.app.state, &gts, &target)
            .await
            .expect("other author"),
        "only the author deletes"
    );
    assert!(
        ingest::delete_note_by_ap_id(&env.app.state, &ada, &target)
            .await
            .expect("delete")
    );
    assert!(env.row(id).await.deleted_at.is_some());
    assert!(
        !ingest::delete_note_by_ap_id(&env.app.state, &ada, &target)
            .await
            .expect("again")
    );
    assert_eq!(
        env.store(&ada, &env.load("mastodon-note-public"), Context::Lookup)
            .await,
        Stored::Dropped,
        "a deleted note stays deleted when it is delivered again"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn bare_id_and_misskey_deletes_find_their_posts(db: PgPool) {
    let env = Env::new(db).await;
    let ada = env.actor("mastodon-actor-person").await;
    let id = env
        .post_of(
            &ada,
            &env.load("generic-note-attributedto-array"),
            Context::Lookup,
        )
        .await;
    let bare = env.load("generic-activity-delete-bare-id");
    let target = ap::id_of(&bare["object"]).expect("id").to_owned();
    assert_eq!(env.row(id).await.ap_id.as_deref(), Some(target.as_str()));
    assert!(
        ingest::delete_note_by_ap_id(&env.app.state, &ada, &target)
            .await
            .expect("delete")
    );
    assert!(env.row(id).await.deleted_at.is_some());

    let misskey = env.actor("misskey-actor-person").await;
    let note_id = env
        .post_of(&misskey, &env.load("misskey-note-public"), Context::Lookup)
        .await;
    let tombstone = env.load("misskey-activity-delete-tombstone");
    let target = ap::id_of(&tombstone["object"]).expect("id").to_owned();
    assert_eq!(
        env.row(note_id).await.ap_id.as_deref(),
        Some(target.as_str())
    );
    assert!(
        ingest::delete_note_by_ap_id(&env.app.state, &misskey, &target)
            .await
            .expect("delete")
    );
}

// ------------------------------------------------------------------ fetching

#[sqlx::test(migrations = "./migrations")]
async fn a_note_fetched_by_url_brings_its_author_and_every_part(db: PgPool) {
    let env = Env::new(db).await;
    let gts = &env.world.gts;
    gts.put("/users/ada", env.load("gts-actor-person"));
    let note = env.load("gts-note-public");
    let url = string(&note, "id");
    gts.put("/users/ada/statuses/01FF25D5Q0DH7CHD57CTRS6WK0", note);
    let id = lookup::fetch_remote_post(&env.app.state, None, &url)
        .await
        .expect("fetched");
    let row = env.row(id).await;
    assert_eq!(
        row.text,
        "Welcome to the #welcome thread. This is an example note."
    );
    let author = lookup::actor_by_url(&env.app.state, &format!("{}/users/ada", gts.base()))
        .await
        .expect("author is stored");
    assert_eq!(row.author_id, author.id);
    assert_eq!(author.display_name, "ada :rainbow:");
    assert_eq!(env.media(id).await.len(), 2);

    let misskey = &env.world.misskey;
    misskey.put("/users/9a0example01", env.load("misskey-actor-person"));
    let note = env.load("misskey-note-cw");
    let url = string(&note, "id");
    misskey.put("/notes/9a0noteid002", note);
    let id = lookup::fetch_remote_post(&env.app.state, None, &url)
        .await
        .expect("fetched");
    assert_eq!(env.row(id).await.cw, "Mildly spicy opinions");
}

#[sqlx::test(migrations = "./migrations")]
async fn outbox_pages_of_all_three_servers_backfill_their_public_posts(db: PgPool) {
    let env = Env::new(db).await;

    let mastodon = env.actor("mastodon-actor-person").await;
    let site = &env.world.mastodon;
    site.put("/users/ada/outbox", env.load("mastodon-collection-outbox"));
    site.put(
        "/users/ada/outbox?page=true",
        env.load("mastodon-collection-outbox-page"),
    );
    let outbox = format!("{}/users/ada/outbox", site.base());
    assert_eq!(
        lookup::backfill_outbox(&env.app.state, &mastodon, &outbox).await,
        1,
        "the Announce is skipped"
    );
    let stored: Vec<String> = sqlx::query_scalar("select text from posts where author_id = $1")
        .bind(mastodon.id)
        .fetch_all(&env.db)
        .await
        .expect("texts");
    assert_eq!(stored, ["Outbox item."]);

    let misskey = env.actor("misskey-actor-person").await;
    let site = &env.world.misskey;
    site.put(
        "/users/9a0example01/outbox",
        env.load("misskey-collection-outbox-page"),
    );
    let outbox = format!("{}/users/9a0example01/outbox", site.base());
    assert_eq!(
        lookup::backfill_outbox(&env.app.state, &misskey, &outbox).await,
        1
    );
    let cw: String = sqlx::query_scalar("select cw from posts where author_id = $1")
        .bind(misskey.id)
        .fetch_one(&env.db)
        .await
        .expect("cw");
    assert_eq!(cw, "Mildly spicy opinions");

    let gts = env.actor("gts-actor-person").await;
    let site = &env.world.gts;
    site.put("/users/ada/outbox", env.load("gts-collection-outbox"));
    site.put(
        "/users/ada/outbox?page=true",
        env.load("gts-collection-outbox-page"),
    );
    for suffix in ["K7", "K8"] {
        let mut note = env.load("gts-note-single-values");
        let id = format!(
            "{}/users/ada/statuses/01FJC1MKPVX2VMWP2ST93Q90{suffix}",
            site.base()
        );
        note["id"] = json!(id);
        note["url"] = json!(format!(
            "{}/@ada/statuses/01FJC1MKPVX2VMWP2ST93Q90{suffix}",
            site.base()
        ));
        site.put(
            &format!("/users/ada/statuses/01FJC1MKPVX2VMWP2ST93Q90{suffix}"),
            note,
        );
    }
    let outbox = format!("{}/users/ada/outbox", site.base());
    assert_eq!(
        lookup::backfill_outbox(&env.app.state, &gts, &outbox).await,
        2,
        "a Create whose object is a URL is fetched"
    );
    let ids: Vec<String> =
        sqlx::query_scalar("select ap_id from posts where author_id = $1 order by ap_id")
            .bind(gts.id)
            .fetch_all(&env.db)
            .await
            .expect("ids");
    assert_eq!(
        ids,
        [
            format!(
                "{}/users/ada/statuses/01FJC1MKPVX2VMWP2ST93Q90K7",
                site.base()
            ),
            format!(
                "{}/users/ada/statuses/01FJC1MKPVX2VMWP2ST93Q90K8",
                site.base()
            ),
        ]
    );
}
