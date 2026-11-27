//! The inbox end to end: a remote actor with its own RSA key signs deliveries to the router, and
//! the tests read the database back. A fake remote instance (an axum app on a loopback socket)
//! serves actors, counts the requests it gets and records the deliveries it receives.
#![allow(missing_docs)]

mod common;

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, LazyLock, Mutex};

use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use tokio::net::TcpListener;
use tuita_core::ap::{self, KeyPair, PrivateKey, PublicKey, RequestParts};
use tuita_server::db::remote::{self, RemoteProfile, Upsert};
use tuita_server::dto::{NotificationKindDto, StreamEvent};
use tuita_server::federation::keys;
use tuita_server::workers::delivery;
use url::Url;
use uuid::Uuid;

use common::{App, ORIGIN, Person, RawReply, app_with};

// ------------------------------------------------------------------ the fake instance

struct Received {
    target: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

#[derive(Default)]
struct Inner {
    docs: Mutex<HashMap<String, Value>>,
    gone: Mutex<HashSet<String>>,
    hits: Mutex<Vec<String>>,
    posts: Mutex<Vec<Received>>,
}

struct Fake {
    addr: SocketAddr,
    inner: Arc<Inner>,
}

async fn serve(
    State(inner): State<Arc<Inner>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if method == Method::POST {
        inner.posts.lock().expect("lock").push(Received {
            target: uri.to_string(),
            headers: headers
                .iter()
                .map(|(name, value)| {
                    (
                        name.as_str().to_owned(),
                        value.to_str().unwrap_or_default().to_owned(),
                    )
                })
                .collect(),
            body: body.to_vec(),
        });
        return StatusCode::ACCEPTED.into_response();
    }
    inner.hits.lock().expect("lock").push(uri.path().to_owned());
    if inner.gone.lock().expect("lock").contains(uri.path()) {
        return StatusCode::GONE.into_response();
    }
    let doc = inner.docs.lock().expect("lock").get(uri.path()).cloned();
    match doc {
        Some(doc) => (
            [(CONTENT_TYPE, "application/activity+json")],
            doc.to_string(),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
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

    fn gone(&self, path: &str) {
        self.inner
            .gone
            .lock()
            .expect("lock")
            .insert(path.to_owned());
    }

    /// GET requests so far.
    fn requests(&self) -> usize {
        self.inner.hits.lock().expect("lock").len()
    }

    fn requests_for(&self, path: &str) -> usize {
        self.inner
            .hits
            .lock()
            .expect("lock")
            .iter()
            .filter(|hit| *hit == path)
            .count()
    }

    fn delivered(&self) -> Vec<Received> {
        std::mem::take(&mut *self.inner.posts.lock().expect("lock"))
    }
}

// ------------------------------------------------------------------ keys and remote actors

static KEYS: LazyLock<[KeyPair; 2]> = LazyLock::new(|| {
    [
        ap::generate_keypair().expect("keypair"),
        ap::generate_keypair().expect("keypair"),
    ]
});

fn private(n: usize) -> PrivateKey {
    PrivateKey::from_pem(&KEYS[n].private_pem).expect("private key")
}

fn actor_doc(base: &str, name: &str, key: usize) -> Value {
    let id = format!("{base}/users/{name}");
    json!({
        "@context": ["https://www.w3.org/ns/activitystreams", "https://w3id.org/security/v1"],
        "id": id,
        "type": "Person",
        "preferredUsername": name,
        "name": format!("Name of {name}"),
        "inbox": format!("{id}/inbox"),
        "outbox": format!("{id}/outbox"),
        "followers": format!("{id}/followers"),
        "publicKey": {"id": format!("{id}#main-key"), "owner": id, "publicKeyPem": KEYS[key].public_pem}
    })
}

/// A remote account with its key. `stored` puts it in the database without any network traffic.
struct Remote {
    id: String,
    base: String,
    key_id: String,
    key: PrivateKey,
    handle: String,
    uid: Uuid,
}

impl Remote {
    async fn stored(db: &PgPool, fake: &Fake, name: &str, key: usize) -> Self {
        let base = fake.base();
        let id = format!("{base}/users/{name}");
        let profile = RemoteProfile {
            handle: name.to_owned(),
            domain: fake.domain(),
            display_name: name.to_owned(),
            bio: String::new(),
            website: String::new(),
            is_bot: false,
            is_org: false,
            locked: false,
            discoverable: true,
            inbox_url: format!("{id}/inbox"),
            shared_inbox_url: None,
            outbox_url: None,
            followers_url: Some(format!("{id}/followers")),
            featured_url: None,
            url: None,
            public_key_pem: KEYS[key].public_pem.clone(),
            created_at: None,
            ap_id: id.clone(),
        };
        let Upsert::Stored(row) = remote::upsert_actor(db, &profile).await.expect("upsert") else {
            panic!("not stored");
        };
        Self {
            key_id: format!("{id}#main-key"),
            key: private(key),
            handle: name.to_owned(),
            uid: row,
            id,
            base,
        }
    }

    /// An actor nobody stored: only its id and key exist.
    fn unknown(fake: &Fake, name: &str, key: usize) -> Self {
        let base = fake.base();
        let id = format!("{base}/users/{name}");
        Self {
            key_id: format!("{id}#main-key"),
            key: private(key),
            handle: name.to_owned(),
            uid: Uuid::nil(),
            id,
            base,
        }
    }

    fn activity(&self, kind: &str, object: Value) -> Value {
        let mut doc = json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": format!("{}/activities/{}", self.base, Uuid::now_v7()),
            "type": kind,
            "actor": self.id,
        });
        doc["object"] = object;
        doc
    }

    /// A public note by this actor that names `ada` in `cc`.
    fn note_to(&self, n: &str, content: &str, to: &[&str], cc: &[&str]) -> Value {
        json!({
            "id": format!("{}/users/{}/statuses/{n}", self.base, self.handle),
            "type": "Note",
            "attributedTo": self.id,
            "content": format!("<p>{content}</p>"),
            "published": ap::format_time(OffsetDateTime::now_utc()),
            "to": to,
            "cc": cc,
        })
    }

    fn note_id(&self, n: &str) -> String {
        format!("{}/users/{}/statuses/{n}", self.base, self.handle)
    }
}

fn ada_id() -> String {
    format!("{ORIGIN}/users/ada")
}

fn open(var: &str) -> Option<String> {
    matches!(
        var,
        "FEDERATION_ALLOW_HTTP" | "FEDERATION_ALLOW_PRIVATE_NETS"
    )
    .then(|| "1".to_owned())
}

struct World {
    app: App,
    db: PgPool,
    fake: Fake,
    ada: Person,
    bob: Remote,
}

async fn world(db: PgPool) -> World {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let ada = app.person(&db, "ada").await;
    let bob = Remote::stored(&db, &fake, "bob", 0).await;
    World {
        app,
        db,
        fake,
        ada,
        bob,
    }
}

// ------------------------------------------------------------------ sending

fn signed(
    key: &PrivateKey,
    key_id: &str,
    path: &str,
    body: &[u8],
    at: OffsetDateTime,
) -> Vec<(String, String)> {
    let url = Url::parse(&format!("{ORIGIN}{path}")).expect("url");
    ap::sign_post(key, key_id, &url, body, at)
        .expect("signs")
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value))
        .collect()
}

async fn raw(app: &App, path: &str, headers: &[(String, String)], body: &[u8]) -> RawReply {
    let view: Vec<(&str, &str)> = headers
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect();
    app.browser()
        .send_raw(Method::POST, path, &view, Body::from(body.to_vec()))
        .await
}

/// Delivers `body` to `path`, signed by `key` as `key_id` at time `at`.
async fn post_at(
    app: &App,
    path: &str,
    key: &PrivateKey,
    key_id: &str,
    body: &[u8],
    at: OffsetDateTime,
) -> RawReply {
    raw(app, path, &signed(key, key_id, path, body, at), body).await
}

async fn post_as(
    app: &App,
    path: &str,
    key: &PrivateKey,
    key_id: &str,
    activity: &Value,
) -> RawReply {
    let body = serde_json::to_vec(activity).expect("json");
    post_at(app, path, key, key_id, &body, OffsetDateTime::now_utc()).await
}

/// Delivers `activity` to the shared inbox as `who`.
async fn send(app: &App, who: &Remote, activity: &Value) -> RawReply {
    post_as(app, "/inbox", &who.key, &who.key_id, activity).await
}

#[track_caller]
fn accepted(reply: &RawReply) {
    assert_eq!(
        reply.status,
        StatusCode::ACCEPTED,
        "body: {}",
        String::from_utf8_lossy(&reply.bytes)
    );
    assert!(reply.bytes.is_empty(), "202 has no body");
}

/// Delivers `activity` as `who` and expects `202`.
async fn deliver(app: &App, who: &Remote, activity: &Value) {
    accepted(&send(app, who, activity).await);
}

// ------------------------------------------------------------------ reading the database

async fn count(db: &PgPool, sql: String) -> i64 {
    sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .fetch_one(db)
        .await
        .expect("count")
}

async fn follow_state(db: &PgPool, follower: Uuid, followee: Uuid) -> Option<String> {
    sqlx::query_scalar("select state from follows where follower_id = $1 and followee_id = $2")
        .bind(follower)
        .bind(followee)
        .fetch_optional(db)
        .await
        .expect("follow")
}

/// `(followers_count, following_count)` of `user`.
async fn counters(db: &PgPool, user: Uuid) -> (i32, i32) {
    sqlx::query_as("select followers_count, following_count from users where id = $1")
        .bind(user)
        .fetch_one(db)
        .await
        .expect("counters")
}

async fn insert_follow(db: &PgPool, follower: Uuid, followee: Uuid, state: &str, ap_id: &str) {
    sqlx::query(
        "insert into follows (follower_id, followee_id, state, ap_id) values ($1, $2, $3, $4)",
    )
    .bind(follower)
    .bind(followee)
    .bind(state)
    .bind(ap_id)
    .execute(db)
    .await
    .expect("follow");
    if state == "accepted" {
        sqlx::query("update users set followers_count = followers_count + 1 where id = $1")
            .bind(followee)
            .execute(db)
            .await
            .expect("followers");
        sqlx::query("update users set following_count = following_count + 1 where id = $1")
            .bind(follower)
            .execute(db)
            .await
            .expect("following");
    }
}

async fn notifications(db: &PgPool, user: Uuid, kind: &str) -> i64 {
    count(
        db,
        format!("select count(*) from notifications where user_id = '{user}' and kind = '{kind}'"),
    )
    .await
}

async fn seen(db: &PgPool) -> i64 {
    count(db, "select count(*) from ap_seen".to_owned()).await
}

async fn queued(db: &PgPool) -> i64 {
    count(db, "select count(*) from ap_deliveries".to_owned()).await
}

/// `(like_count, repost_count)` of a post.
async fn post_counts(db: &PgPool, post: Uuid) -> (i32, i32) {
    sqlx::query_as("select like_count, repost_count from posts where id = $1")
        .bind(post)
        .fetch_one(db)
        .await
        .expect("counts")
}

async fn post_by_ap_id(db: &PgPool, ap_id: &str) -> Option<(Uuid, String, bool)> {
    sqlx::query_as("select id, text, deleted_at is not null from posts where ap_id = $1")
        .bind(ap_id)
        .fetch_optional(db)
        .await
        .expect("post")
}

fn id_of(post: &Value) -> Uuid {
    post["id"].as_str().expect("id").parse().expect("uuid")
}

fn local_note(post: Uuid) -> String {
    format!("{ORIGIN}/users/ada/statuses/{post}")
}

async fn ada_post(w: &mut World, text: &str) -> Uuid {
    id_of(&w.ada.browser.post_text(text).await)
}

// ------------------------------------------------------------------ verification failures

#[sqlx::test(migrations = "./migrations")]
async fn an_unsigned_delivery_is_refused_and_changes_nothing(db: PgPool) {
    let w = world(db).await;
    let follow = w.bob.activity("Follow", json!(ada_id()));
    let body = serde_json::to_vec(&follow).expect("json");
    let reply = raw(&w.app, "/inbox", &[], &body).await;
    reply.err(StatusCode::UNAUTHORIZED, "Missing signature.");
    assert_eq!(seen(&w.db).await, 0);
    assert_eq!(follow_state(&w.db, w.bob.uid, w.ada.id).await, None);
    assert_eq!(w.fake.requests(), 0, "nothing was fetched");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_bad_digest_and_a_tampered_body_are_refused(db: PgPool) {
    let w = world(db).await;
    let follow = w.bob.activity("Follow", json!(ada_id()));
    let body = serde_json::to_vec(&follow).expect("json");
    let now = OffsetDateTime::now_utc();

    let mut headers = signed(&w.bob.key, &w.bob.key_id, "/inbox", &body, now);
    for (name, value) in &mut headers {
        if name == "digest" {
            *value = ap::digest_header(b"something else");
        }
    }
    raw(&w.app, "/inbox", &headers, &body)
        .await
        .err(StatusCode::UNAUTHORIZED, "Invalid signature.");

    let headers = signed(&w.bob.key, &w.bob.key_id, "/inbox", &body, now);
    let mut tampered = follow.clone();
    tampered["object"] = json!(format!("{ORIGIN}/users/someone"));
    let tampered = serde_json::to_vec(&tampered).expect("json");
    raw(&w.app, "/inbox", &headers, &tampered)
        .await
        .err(StatusCode::UNAUTHORIZED, "Invalid signature.");

    assert_eq!(seen(&w.db).await, 0);
    assert_eq!(follow_state(&w.db, w.bob.uid, w.ada.id).await, None);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_delivery_signed_for_another_host_is_refused(db: PgPool) {
    let w = world(db).await;
    let follow = w.bob.activity("Follow", json!(ada_id()));
    let body = serde_json::to_vec(&follow).expect("json");
    let now = OffsetDateTime::now_utc();
    let url = Url::parse("https://attacker.example/inbox").expect("url");
    let headers: Vec<(String, String)> = ap::sign_post(&w.bob.key, &w.bob.key_id, &url, &body, now)
        .expect("signs")
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value))
        .collect();
    raw(&w.app, "/inbox", &headers, &body)
        .await
        .err(StatusCode::UNAUTHORIZED, "Invalid signature.");
    assert_eq!(seen(&w.db).await, 0);
    assert_eq!(follow_state(&w.db, w.bob.uid, w.ada.id).await, None);

    // This instance's own host, in another case and with its default port, is still ours.
    let url = Url::parse("https://SOCIAL.example:443/inbox").expect("url");
    let headers: Vec<(String, String)> = ap::sign_post(&w.bob.key, &w.bob.key_id, &url, &body, now)
        .expect("signs")
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value))
        .collect();
    accepted(&raw(&w.app, "/inbox", &headers, &body).await);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_date_two_hours_off_either_way_is_refused(db: PgPool) {
    let w = world(db).await;
    let follow = w.bob.activity("Follow", json!(ada_id()));
    let body = serde_json::to_vec(&follow).expect("json");
    for offset in [Duration::hours(-2), Duration::hours(2)] {
        let at = OffsetDateTime::now_utc() + offset;
        post_at(&w.app, "/inbox", &w.bob.key, &w.bob.key_id, &body, at)
            .await
            .err(StatusCode::UNAUTHORIZED, "Invalid signature.");
    }
    let near = OffsetDateTime::now_utc() - Duration::minutes(50);
    accepted(&post_at(&w.app, "/inbox", &w.bob.key, &w.bob.key_id, &body, near).await);
    assert_eq!(
        follow_state(&w.db, w.bob.uid, w.ada.id).await.as_deref(),
        Some("accepted")
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_signature_from_another_key_is_refused(db: PgPool) {
    let w = world(db).await;
    // The fake still publishes the key bob is stored with, so the refresh finds nothing new.
    w.fake
        .put("/users/bob", actor_doc(&w.fake.base(), "bob", 0));
    let follow = w.bob.activity("Follow", json!(ada_id()));
    let reply = post_as(&w.app, "/inbox", &private(1), &w.bob.key_id, &follow).await;
    reply.err(StatusCode::UNAUTHORIZED, "Invalid signature.");
    assert_eq!(w.fake.requests_for("/users/bob"), 1, "one refresh, no more");
    assert_eq!(seen(&w.db).await, 0);
    assert_eq!(follow_state(&w.db, w.bob.uid, w.ada.id).await, None);
}

#[sqlx::test(migrations = "./migrations")]
async fn an_unknown_key_is_refused(db: PgPool) {
    let w = world(db).await;
    let ghost = Remote::unknown(&w.fake, "ghost", 1);
    let follow = ghost.activity("Follow", json!(ada_id()));
    send(&w.app, &ghost, &follow)
        .await
        .err(StatusCode::UNAUTHORIZED, "Unknown signing key.");
    assert_eq!(w.fake.requests_for("/users/ghost"), 1);
    assert_eq!(seen(&w.db).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_bad_signature_from_an_unknown_actor_stores_nothing(db: PgPool) {
    let w = world(db).await;
    let dave = Remote::unknown(&w.fake, "dave", 1);
    let mut doc = actor_doc(&w.fake.base(), "dave", 1);
    doc["icon"] = json!({"type": "Image", "url": format!("{}/icon.png", w.fake.base())});
    doc["image"] = json!({"type": "Image", "url": format!("{}/banner.png", w.fake.base())});
    doc["discoverable"] = json!(true);
    w.fake.put("/users/dave", doc);
    let follow = dave.activity("Follow", json!(ada_id()));
    // dave's document publishes key 1; the request is signed with key 0.
    post_as(&w.app, "/inbox", &private(0), &dave.key_id, &follow)
        .await
        .err(StatusCode::UNAUTHORIZED, "Invalid signature.");
    assert!(
        remote::find_by_ap_id(&w.db, &dave.id)
            .await
            .expect("find")
            .is_none(),
        "no account without a valid signature"
    );
    assert_eq!(w.fake.requests_for("/icon.png"), 0);
    assert_eq!(w.fake.requests_for("/banner.png"), 0);
    assert_eq!(seen(&w.db).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_failed_key_refresh_changes_nothing_and_happens_once_per_window(db: PgPool) {
    let w = world(db).await;
    let mut doc = actor_doc(&w.fake.base(), "bob", 0);
    doc["name"] = json!("Renamed by the fetch");
    w.fake.put("/users/bob", doc);
    let follow = w.bob.activity("Follow", json!(ada_id()));
    for _ in 0..3 {
        post_as(&w.app, "/inbox", &private(1), &w.bob.key_id, &follow)
            .await
            .err(StatusCode::UNAUTHORIZED, "Invalid signature.");
    }
    assert_eq!(
        w.fake.requests_for("/users/bob"),
        1,
        "one refresh per window"
    );
    let name: String = sqlx::query_scalar("select display_name from users where id = $1")
        .bind(w.bob.uid)
        .fetch_one(&w.db)
        .await
        .expect("name");
    assert_eq!(name, "bob", "the fetched profile was not stored");
}

#[sqlx::test(migrations = "./migrations")]
async fn the_key_must_belong_to_the_actor_of_the_activity(db: PgPool) {
    let w = world(db).await;
    let carol = Remote::stored(&w.db, &w.fake, "carol", 1).await;
    // Carol signs an activity that names bob as the actor.
    let spoof = w.bob.activity("Follow", json!(ada_id()));
    send(&w.app, &carol, &spoof)
        .await
        .err(StatusCode::UNAUTHORIZED, "Invalid signature.");
    // A key on another host cannot speak for bob either.
    let other = json!({
        "id": "https://elsewhere.example/a/1", "type": "Follow",
        "actor": w.bob.id, "object": ada_id(),
    });
    let body = serde_json::to_vec(&other).expect("json");
    post_at(
        &w.app,
        "/inbox",
        &w.bob.key,
        "https://elsewhere.example/users/bob#main-key",
        &body,
        OffsetDateTime::now_utc(),
    )
    .await
    .err(StatusCode::UNAUTHORIZED, "Invalid signature.");
    assert_eq!(follow_state(&w.db, w.bob.uid, w.ada.id).await, None);
    assert_eq!(
        follow_state(&w.db, carol.uid, w.ada.id).await,
        None,
        "carol did not follow either"
    );
    assert_eq!(seen(&w.db).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_rotated_key_is_fetched_once_and_the_delivery_goes_through(db: PgPool) {
    let mut w = world(db).await;
    w.fake
        .put("/users/bob", actor_doc(&w.fake.base(), "bob", 1));
    let like_target = ada_post(&mut w, "hello from ada").await;
    let like = w.bob.activity("Like", json!(local_note(like_target)));
    let reply = post_as(&w.app, "/inbox", &private(1), &w.bob.key_id, &like).await;
    accepted(&reply);
    assert_eq!(w.fake.requests_for("/users/bob"), 1);
    assert_eq!(post_counts(&w.db, like_target).await, (1, 0));
    let pem = remote::public_key_pem(&w.db, w.bob.uid)
        .await
        .expect("pem")
        .expect("stored key");
    assert_eq!(pem, KEYS[1].public_pem);
    // The new key keeps working without another fetch.
    let again = w.bob.activity("Announce", json!(local_note(like_target)));
    accepted(&post_as(&w.app, "/inbox", &private(1), &w.bob.key_id, &again).await);
    assert_eq!(w.fake.requests_for("/users/bob"), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn an_unknown_actor_is_fetched_stored_and_served(db: PgPool) {
    let w = world(db).await;
    let dave = Remote::unknown(&w.fake, "dave", 1);
    w.fake
        .put("/users/dave", actor_doc(&w.fake.base(), "dave", 1));
    let follow = dave.activity("Follow", json!(ada_id()));
    deliver(&w.app, &dave, &follow).await;
    let row = remote::find_by_ap_id(&w.db, &dave.id)
        .await
        .expect("find")
        .expect("dave is stored");
    assert_eq!(
        follow_state(&w.db, row.id, w.ada.id).await.as_deref(),
        Some("accepted")
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn an_actor_with_an_inbox_on_another_host_is_refused(db: PgPool) {
    let w = world(db).await;
    let dave = Remote::unknown(&w.fake, "dave", 1);
    let mut doc = actor_doc(&w.fake.base(), "dave", 1);
    doc["inbox"] = json!("https://victim.example/inbox");
    w.fake.put("/users/dave", doc);
    send(&w.app, &dave, &dave.activity("Follow", json!(ada_id())))
        .await
        .err(StatusCode::UNAUTHORIZED, "Unknown signing key.");
    assert!(
        remote::find_by_ap_id(&w.db, &dave.id)
            .await
            .expect("find")
            .is_none()
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn endpoints_on_another_host_are_dropped_and_the_rest_normalised(db: PgPool) {
    let w = world(db).await;
    let dave = Remote::unknown(&w.fake, "dave", 1);
    let mut doc = actor_doc(&w.fake.base(), "dave", 1);
    doc["endpoints"] = json!({"sharedInbox": "https://victim.example/inbox"});
    doc["outbox"] = json!("https://victim.example/outbox");
    doc["followers"] = json!(format!("HTTP://{}/users/dave/followers", w.fake.domain()));
    w.fake.put("/users/dave", doc);
    deliver(&w.app, &dave, &dave.activity("Follow", json!(ada_id()))).await;
    let (shared, outbox, followers): (Option<String>, Option<String>, Option<String>) =
        sqlx::query_as(
            "select shared_inbox_url, outbox_url, followers_url from users where ap_id = $1",
        )
        .bind(&dave.id)
        .fetch_one(&w.db)
        .await
        .expect("dave");
    assert_eq!(shared, None);
    assert_eq!(outbox, None);
    assert_eq!(
        followers.as_deref(),
        Some(format!("http://{}/users/dave/followers", w.fake.domain()).as_str())
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_key_id_below_the_actor_url_without_a_fragment_is_accepted(db: PgPool) {
    let mut w = world(db).await;
    let target = ada_post(&mut w, "hello from ada").await;
    // GoToSocial signs with `{actor}/main-key`.
    w.bob.key_id = format!("{}/main-key", w.bob.id);
    let like = w.bob.activity("Like", json!(local_note(target)));
    deliver(&w.app, &w.bob, &like).await;
    assert_eq!(post_counts(&w.db, target).await, (1, 0));

    let mut dave = Remote::unknown(&w.fake, "dave", 1);
    dave.key_id = format!("{}/main-key", dave.id);
    w.fake
        .put("/users/dave", actor_doc(&w.fake.base(), "dave", 1));
    deliver(&w.app, &dave, &dave.activity("Follow", json!(ada_id()))).await;
    assert_eq!(w.fake.requests_for("/users/dave"), 1);
    assert_eq!(w.fake.requests_for("/users/dave/main-key"), 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_key_id_below_another_actor_is_still_refused(db: PgPool) {
    let w = world(db).await;
    let carol = Remote::stored(&w.db, &w.fake, "carol", 1).await;
    let follow = w.bob.activity("Follow", json!(ada_id()));
    let key_id = format!("{}/main-key", carol.id);
    let reply = post_as(&w.app, "/inbox", &carol.key, &key_id, &follow).await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert_eq!(follow_state(&w.db, w.bob.uid, w.ada.id).await, None);
    assert_eq!(seen(&w.db).await, 0);
}

// ------------------------------------------------------------------ the body

#[sqlx::test(migrations = "./migrations")]
async fn a_body_over_one_mebibyte_is_413_and_bad_json_is_400(db: PgPool) {
    let w = world(db).await;
    let big = vec![b' '; 1024 * 1024 + 1];
    raw(&w.app, "/inbox", &[], &big)
        .await
        .err(StatusCode::PAYLOAD_TOO_LARGE, "Body too large.");

    for body in [&b"{not json"[..], b"[1, 2]", b"\"text\"", b"{}"] {
        raw(&w.app, "/inbox", &[], body)
            .await
            .err(StatusCode::BAD_REQUEST, "Invalid JSON.");
    }
    let deep = format!("{}1{}", "[".repeat(60), "]".repeat(60));
    raw(
        &w.app,
        "/inbox",
        &[],
        format!(r#"{{"type":"Like","x":{deep}}}"#).as_bytes(),
    )
    .await
    .err(StatusCode::BAD_REQUEST, "Invalid JSON.");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_personal_inbox_needs_a_live_local_account(db: PgPool) {
    let w = world(db).await;
    let follow = w.bob.activity("Follow", json!(ada_id()));
    let path = "/users/ada/inbox";
    accepted(&post_as(&w.app, path, &w.bob.key, &w.bob.key_id, &follow).await);
    assert_eq!(
        follow_state(&w.db, w.bob.uid, w.ada.id).await.as_deref(),
        Some("accepted")
    );

    let nobody = post_as(
        &w.app,
        "/users/nobody/inbox",
        &w.bob.key,
        &w.bob.key_id,
        &follow,
    )
    .await;
    assert_eq!(nobody.status, StatusCode::NOT_FOUND);
    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(w.ada.id)
        .execute(&w.db)
        .await
        .expect("suspend");
    let suspended = post_as(&w.app, path, &w.bob.key, &w.bob.key_id, &follow).await;
    assert_eq!(suspended.status, StatusCode::NOT_FOUND);
    sqlx::query("update users set suspended_at = null, deleted_at = now() where id = $1")
        .bind(w.ada.id)
        .execute(&w.db)
        .await
        .expect("delete");
    let deleted = post_as(&w.app, path, &w.bob.key, &w.bob.key_id, &follow).await;
    assert_eq!(deleted.status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_301st_delivery_from_one_address_is_429(db: PgPool) {
    let w = world(db).await;
    let mut browser = w.app.browser_at([203, 0, 113, 77]);
    for _ in 0..300 {
        let reply = browser
            .send_raw(Method::POST, "/inbox", &[], Body::from("{}"))
            .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    }
    let reply = browser
        .send_raw(Method::POST, "/inbox", &[], Body::from("{}"))
        .await;
    assert_eq!(reply.status, StatusCode::TOO_MANY_REQUESTS);
    assert!(!reply.header("retry-after").is_empty());
    // Another address has its own count.
    let mut other = w.app.browser_at([203, 0, 113, 78]);
    let reply = other
        .send_raw(Method::POST, "/inbox", &[], Body::from("{}"))
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
}

// ------------------------------------------------------------------ replays and failures

#[sqlx::test(migrations = "./migrations")]
async fn a_replayed_activity_changes_nothing(db: PgPool) {
    let mut w = world(db).await;
    let post = ada_post(&mut w, "liked twice").await;
    let like = w.bob.activity("Like", json!(local_note(post)));
    deliver(&w.app, &w.bob, &like).await;
    assert_eq!(post_counts(&w.db, post).await, (1, 0));
    assert_eq!(seen(&w.db).await, 1);

    // Undo the effect behind the pipeline's back, then replay: only the dedupe can say no.
    sqlx::query("delete from likes")
        .execute(&w.db)
        .await
        .expect("delete");
    sqlx::query("update posts set like_count = 0")
        .execute(&w.db)
        .await
        .expect("reset");
    deliver(&w.app, &w.bob, &like).await;
    assert_eq!(post_counts(&w.db, post).await, (0, 0));
    assert_eq!(
        count(&w.db, "select count(*) from likes".to_owned()).await,
        0
    );
    assert_eq!(seen(&w.db).await, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn an_activity_id_on_another_host_is_not_deduped(db: PgPool) {
    let mut w = world(db).await;
    let post = ada_post(&mut w, "post").await;
    let mut like = w.bob.activity("Like", json!(local_note(post)));
    like["id"] = json!("https://victim.example/activities/1");
    deliver(&w.app, &w.bob, &like).await;
    assert_eq!(seen(&w.db).await, 0, "a foreign id never poisons ap_seen");
    assert_eq!(post_counts(&w.db, post).await, (1, 0));
}

#[sqlx::test(migrations = "./migrations")]
async fn an_activity_without_an_id_is_processed_every_time(db: PgPool) {
    let mut w = world(db).await;
    let post = ada_post(&mut w, "post").await;
    let mut like = w.bob.activity("Like", json!(local_note(post)));
    like.as_object_mut().expect("object").remove("id");
    deliver(&w.app, &w.bob, &like).await;
    deliver(&w.app, &w.bob, &like).await;
    assert_eq!(seen(&w.db).await, 0);
    assert_eq!(post_counts(&w.db, post).await, (1, 0), "still idempotent");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_failure_on_our_side_forgets_the_activity_so_the_retry_works(db: PgPool) {
    let mut w = world(db).await;
    let post = ada_post(&mut w, "post").await;
    let like = w.bob.activity("Like", json!(local_note(post)));
    sqlx::query("alter table likes rename to likes_away")
        .execute(&w.db)
        .await
        .expect("rename");
    let reply = send(&w.app, &w.bob, &like).await;
    assert_eq!(reply.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(seen(&w.db).await, 0, "the seen row is gone");
    sqlx::query("alter table likes_away rename to likes")
        .execute(&w.db)
        .await
        .expect("rename back");
    deliver(&w.app, &w.bob, &like).await;
    assert_eq!(post_counts(&w.db, post).await, (1, 0));
    assert_eq!(seen(&w.db).await, 1);
}

// ------------------------------------------------------------------ domain blocks

#[sqlx::test(migrations = "./migrations")]
async fn a_suspended_domain_is_dropped_before_any_fetch(db: PgPool) {
    let w = world(db).await;
    sqlx::query("insert into domain_blocks (domain, severity) values ($1, 'suspend')")
        .bind(w.fake.domain())
        .execute(&w.db)
        .await
        .expect("block");
    // An actor nobody knows: answering it would need a fetch.
    let ghost = Remote::unknown(&w.fake, "ghost", 1);
    let follow = ghost.activity("Follow", json!(ada_id()));
    deliver(&w.app, &ghost, &follow).await;
    // And a stored one.
    let follow = w.bob.activity("Follow", json!(ada_id()));
    deliver(&w.app, &w.bob, &follow).await;
    // Even a signature that would never verify gets the same answer.
    let garbage = signed(
        &w.bob.key,
        &w.bob.key_id,
        "/inbox",
        b"{}",
        OffsetDateTime::now_utc(),
    );
    let reply = raw(
        &w.app,
        "/inbox",
        &garbage,
        b"{\"type\":\"Follow\",\"actor\":\"x\"}",
    )
    .await;
    accepted(&reply);

    assert_eq!(
        w.fake.requests(),
        0,
        "no request reached the blocked instance"
    );
    assert_eq!(follow_state(&w.db, w.bob.uid, w.ada.id).await, None);
    assert_eq!(seen(&w.db).await, 0);
    assert_eq!(queued(&w.db).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_silenced_domain_is_stored_but_hidden_from_public_places(db: PgPool) {
    let mut w = world(db).await;
    let note = w
        .bob
        .note_to("1", "zebra crossing", &[ap::PUBLIC], &[&ada_id()]);
    deliver(&w.app, &w.bob, &w.bob.activity("Create", note.clone())).await;
    let stored = post_by_ap_id(&w.db, &w.bob.note_id("1")).await;
    assert!(stored.is_some(), "stored");

    let ids = |page: &Value| -> Vec<String> {
        page["items"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item["text"].as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    };
    let search = |reply: Value| -> usize { reply["posts"].as_array().map_or(0, Vec::len) };
    let visible = ids(&w.ada.browser.get("/timelines/foryou").await.ok());
    assert!(
        visible.iter().any(|text| text.contains("zebra")),
        "{visible:?}"
    );
    assert_eq!(
        search(w.ada.browser.get("/search?q=zebra&type=posts").await.ok()),
        1
    );

    sqlx::query("insert into domain_blocks (domain, severity) values ($1, 'silence')")
        .bind(w.fake.domain())
        .execute(&w.db)
        .await
        .expect("silence");
    // Delivery still works under a silence block.
    let second = w
        .bob
        .note_to("2", "zebra again", &[ap::PUBLIC], &[&ada_id()]);
    deliver(&w.app, &w.bob, &w.bob.activity("Create", second)).await;
    assert!(post_by_ap_id(&w.db, &w.bob.note_id("2")).await.is_some());

    let hidden = ids(&w.ada.browser.get("/timelines/foryou").await.ok());
    assert!(
        !hidden.iter().any(|text| text.contains("zebra")),
        "{hidden:?}"
    );
    assert_eq!(
        search(w.ada.browser.get("/search?q=zebra&type=posts").await.ok()),
        0
    );
}

// ------------------------------------------------------------------ follows

fn parse_delivery(received: &Received, pem: &str, key_id: &str) -> Value {
    let view: Vec<(&str, &str)> = received
        .headers
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect();
    let parts = RequestParts {
        method: "POST",
        path_and_query: &received.target,
        headers: &view,
    };
    let header = parts.header("signature").expect("signature header");
    let parsed = ap::parse_signature_header(&header).expect("parses");
    assert_eq!(parsed.key_id, key_id);
    ap::check_request(&parts, &parsed, &received.body, OffsetDateTime::now_utc())
        .expect("date and digest pass");
    ap::verify_signature(&parts, &parsed, &PublicKey::from_pem(pem).expect("pem"))
        .expect("signature verifies");
    serde_json::from_slice(&received.body).expect("json body")
}

#[sqlx::test(migrations = "./migrations")]
async fn a_follow_is_accepted_counted_notified_and_answered_with_a_signed_accept(db: PgPool) {
    let w = world(db).await;
    let hub = w.app.state.hub();
    let _stream = hub.open_stream(w.ada.id);
    let mut live = hub.subscribe(w.ada.id);

    let follow = w.bob.activity("Follow", json!(ada_id()));
    deliver(&w.app, &w.bob, &follow).await;
    assert_eq!(
        follow_state(&w.db, w.bob.uid, w.ada.id).await.as_deref(),
        Some("accepted")
    );
    let stored_id: Option<String> =
        sqlx::query_scalar("select ap_id from follows where follower_id = $1")
            .bind(w.bob.uid)
            .fetch_one(&w.db)
            .await
            .expect("ap_id");
    assert_eq!(stored_id.as_deref(), follow["id"].as_str());
    assert_eq!(counters(&w.db, w.ada.id).await, (1, 0));
    assert_eq!(counters(&w.db, w.bob.uid).await, (0, 1));
    assert_eq!(notifications(&w.db, w.ada.id, "follow").await, 1);
    let event = live.try_recv().expect("a live event");
    match &*event {
        StreamEvent::Notification(note) => assert_eq!(note.kind, NotificationKindDto::Follow),
        other => panic!("expected a notification, got {}", other.name()),
    }

    assert_eq!(queued(&w.db).await, 1);
    let (signer, inbox): (Option<Uuid>, String) =
        sqlx::query_as("select signer_id, inbox_url from ap_deliveries")
            .fetch_one(&w.db)
            .await
            .expect("row");
    assert_eq!(signer, Some(w.ada.id));
    assert_eq!(inbox, format!("{}/inbox", w.bob.id));

    assert_eq!(delivery::run_once(&w.app.state).await, 1);
    let sent = w.fake.delivered().remove(0);
    let pem = keys::user_public_pem(&w.app.state, w.ada.id)
        .await
        .expect("pem");
    let accept = parse_delivery(&sent, &pem, &format!("{}#main-key", ada_id()));
    assert_eq!(accept["type"], "Accept");
    assert_eq!(accept["actor"], ada_id());
    assert_eq!(accept["object"]["id"], follow["id"]);
    assert_eq!(accept["object"]["type"], "Follow");
    assert_eq!(accept["object"]["actor"], w.bob.id);
    assert_eq!(accept["object"]["object"], ada_id());
}

#[sqlx::test(migrations = "./migrations")]
async fn a_second_follow_answers_again_and_does_not_count_twice(db: PgPool) {
    let w = world(db).await;
    let first = w.bob.activity("Follow", json!(ada_id()));
    deliver(&w.app, &w.bob, &first).await;
    let second = w.bob.activity("Follow", json!(ada_id()));
    deliver(&w.app, &w.bob, &second).await;
    assert_eq!(counters(&w.db, w.ada.id).await, (1, 0));
    assert_eq!(counters(&w.db, w.bob.uid).await, (0, 1));
    assert_eq!(notifications(&w.db, w.ada.id, "follow").await, 1);
    assert_eq!(queued(&w.db).await, 2, "the Accept went out again");
    // A replay of the first one is dropped by the dedupe.
    deliver(&w.app, &w.bob, &first).await;
    assert_eq!(queued(&w.db).await, 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_follow_from_a_blocked_account_is_rejected(db: PgPool) {
    let w = world(db).await;
    sqlx::query("insert into blocks (user_id, target_id) values ($1, $2)")
        .bind(w.ada.id)
        .bind(w.bob.uid)
        .execute(&w.db)
        .await
        .expect("block");
    let follow = w.bob.activity("Follow", json!(ada_id()));
    deliver(&w.app, &w.bob, &follow).await;
    assert_eq!(follow_state(&w.db, w.bob.uid, w.ada.id).await, None);
    assert_eq!(counters(&w.db, w.ada.id).await, (0, 0));
    assert_eq!(notifications(&w.db, w.ada.id, "follow").await, 0);
    let reply: Value = sqlx::query_scalar("select activity from ap_deliveries")
        .fetch_one(&w.db)
        .await
        .expect("a reply is queued");
    assert_eq!(reply["type"], "Reject");
    assert_eq!(reply["object"]["id"], follow["id"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_muted_follower_follows_without_a_notification(db: PgPool) {
    let w = world(db).await;
    sqlx::query("insert into mutes (user_id, target_id) values ($1, $2)")
        .bind(w.ada.id)
        .bind(w.bob.uid)
        .execute(&w.db)
        .await
        .expect("mute");
    deliver(&w.app, &w.bob, &w.bob.activity("Follow", json!(ada_id()))).await;
    assert_eq!(
        follow_state(&w.db, w.bob.uid, w.ada.id).await.as_deref(),
        Some("accepted")
    );
    assert_eq!(notifications(&w.db, w.ada.id, "follow").await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_follow_of_something_that_is_not_a_local_account_is_ignored(db: PgPool) {
    let w = world(db).await;
    for object in [
        json!("https://elsewhere.example/users/x"),
        json!(format!("{ORIGIN}/users/nobody")),
        json!(format!("{ORIGIN}/users/ada/statuses/{}", Uuid::now_v7())),
    ] {
        deliver(&w.app, &w.bob, &w.bob.activity("Follow", object)).await;
    }
    assert_eq!(queued(&w.db).await, 0);
    assert_eq!(
        count(&w.db, "select count(*) from follows".to_owned()).await,
        0
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn an_accept_turns_our_pending_follow_into_an_accepted_one(db: PgPool) {
    let w = world(db).await;
    let carol = Remote::stored(&w.db, &w.fake, "carol", 1).await;
    let ada_user = w.ada.id;
    let embedded_id = format!("{ORIGIN}/activities/f1");
    let string_id = format!("{ORIGIN}/activities/f2");
    insert_follow(&w.db, ada_user, w.bob.uid, "pending", &embedded_id).await;
    insert_follow(&w.db, ada_user, carol.uid, "pending", &string_id).await;

    // Carol cannot accept a follow that names bob.
    let wrong = carol.activity("Accept", json!(embedded_id));
    deliver(&w.app, &carol, &wrong).await;
    assert_eq!(
        follow_state(&w.db, ada_user, w.bob.uid).await.as_deref(),
        Some("pending")
    );
    // An unknown id changes nothing.
    let unknown = w
        .bob
        .activity("Accept", json!(format!("{ORIGIN}/activities/none")));
    deliver(&w.app, &w.bob, &unknown).await;
    assert_eq!(counters(&w.db, ada_user).await, (0, 0));

    // The Follow embedded, as Mastodon sends it.
    let accept = w.bob.activity(
        "Accept",
        json!({"id": embedded_id, "type": "Follow", "actor": ada_id(), "object": w.bob.id}),
    );
    deliver(&w.app, &w.bob, &accept).await;
    assert_eq!(
        follow_state(&w.db, ada_user, w.bob.uid).await.as_deref(),
        Some("accepted")
    );
    assert_eq!(counters(&w.db, ada_user).await, (0, 1));
    assert_eq!(counters(&w.db, w.bob.uid).await, (1, 0));
    // Another Accept for the same follow does not count again.
    let again = w.bob.activity("Accept", json!(embedded_id));
    deliver(&w.app, &w.bob, &again).await;
    assert_eq!(counters(&w.db, ada_user).await, (0, 1));

    // Only the id of the Follow.
    let accept = carol.activity("Accept", json!(string_id));
    deliver(&w.app, &carol, &accept).await;
    assert_eq!(
        follow_state(&w.db, ada_user, carol.uid).await.as_deref(),
        Some("accepted")
    );
    assert_eq!(counters(&w.db, ada_user).await, (0, 2));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_reject_removes_the_follow(db: PgPool) {
    let w = world(db).await;
    let id = format!("{ORIGIN}/activities/f1");
    insert_follow(&w.db, w.ada.id, w.bob.uid, "pending", &id).await;
    let reject = w.bob.activity(
        "Reject",
        json!({"id": id, "type": "Follow", "actor": ada_id(), "object": w.bob.id}),
    );
    deliver(&w.app, &w.bob, &reject).await;
    assert_eq!(follow_state(&w.db, w.ada.id, w.bob.uid).await, None);
    assert_eq!(counters(&w.db, w.ada.id).await, (0, 0));

    // A remote account that drops an accepted follower takes it out of the counters.
    insert_follow(&w.db, w.ada.id, w.bob.uid, "accepted", &id).await;
    let reject = w.bob.activity("Reject", json!(id));
    deliver(&w.app, &w.bob, &reject).await;
    assert_eq!(follow_state(&w.db, w.ada.id, w.bob.uid).await, None);
    assert_eq!(counters(&w.db, w.ada.id).await, (0, 0));
    assert_eq!(counters(&w.db, w.bob.uid).await, (0, 0));
}

#[sqlx::test(migrations = "./migrations")]
async fn an_undo_of_a_follow_removes_it_and_the_counters(db: PgPool) {
    let w = world(db).await;
    let carol = Remote::stored(&w.db, &w.fake, "carol", 1).await;
    let follow = w.bob.activity("Follow", json!(ada_id()));
    deliver(&w.app, &w.bob, &follow).await;
    assert_eq!(counters(&w.db, w.ada.id).await, (1, 0));

    // Carol cannot undo bob's follow, whatever she embeds.
    let foreign = carol.activity("Undo", follow.clone());
    deliver(&w.app, &carol, &foreign).await;
    assert_eq!(counters(&w.db, w.ada.id).await, (1, 0));
    let by_id = carol.activity("Undo", follow["id"].clone());
    deliver(&w.app, &carol, &by_id).await;
    assert_eq!(counters(&w.db, w.ada.id).await, (1, 0));

    let undo = w.bob.activity("Undo", follow.clone());
    deliver(&w.app, &w.bob, &undo).await;
    assert_eq!(follow_state(&w.db, w.bob.uid, w.ada.id).await, None);
    assert_eq!(counters(&w.db, w.ada.id).await, (0, 0));
    assert_eq!(counters(&w.db, w.bob.uid).await, (0, 0));

    // By the id of the follow, with the follow already gone: nothing changes.
    let again = w.bob.activity("Undo", follow["id"].clone());
    deliver(&w.app, &w.bob, &again).await;
    assert_eq!(counters(&w.db, w.ada.id).await, (0, 0));

    // A pending follow goes without touching the counters.
    insert_follow(&w.db, w.bob.uid, w.ada.id, "pending", "https://x.example/f").await;
    let undo = w.bob.activity(
        "Undo",
        json!({"id": "https://x.example/f", "type": "Follow", "actor": w.bob.id, "object": ada_id()}),
    );
    deliver(&w.app, &w.bob, &undo).await;
    assert_eq!(follow_state(&w.db, w.bob.uid, w.ada.id).await, None);
    assert_eq!(counters(&w.db, w.ada.id).await, (0, 0));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_block_severs_the_follows_both_ways(db: PgPool) {
    let w = world(db).await;
    insert_follow(
        &w.db,
        w.bob.uid,
        w.ada.id,
        "accepted",
        "https://x.example/f1",
    )
    .await;
    insert_follow(
        &w.db,
        w.ada.id,
        w.bob.uid,
        "accepted",
        "https://x.example/f2",
    )
    .await;
    assert_eq!(counters(&w.db, w.ada.id).await, (1, 1));
    let block = w.bob.activity("Block", json!(ada_id()));
    deliver(&w.app, &w.bob, &block).await;
    assert_eq!(follow_state(&w.db, w.bob.uid, w.ada.id).await, None);
    assert_eq!(follow_state(&w.db, w.ada.id, w.bob.uid).await, None);
    assert_eq!(counters(&w.db, w.ada.id).await, (0, 0));
    assert_eq!(counters(&w.db, w.bob.uid).await, (0, 0));
    assert_eq!(queued(&w.db).await, 0, "the remote's block is not answered");
    assert_eq!(
        count(&w.db, "select count(*) from blocks".to_owned()).await,
        0,
        "nothing is stored as a block"
    );
    let undo = w.bob.activity("Undo", block.clone());
    deliver(&w.app, &w.bob, &undo).await;
}

// ------------------------------------------------------------------ likes and reposts

#[sqlx::test(migrations = "./migrations")]
async fn a_like_and_an_announce_of_a_local_post_count_notify_and_stay_idempotent(db: PgPool) {
    let mut w = world(db).await;
    let post = ada_post(&mut w, "a post").await;
    let hub = w.app.state.hub();
    let _stream = hub.open_stream(w.ada.id);
    let mut live = hub.subscribe(w.ada.id);

    let like = w.bob.activity("Like", json!(local_note(post)));
    deliver(&w.app, &w.bob, &like).await;
    let again = w.bob.activity("Like", json!(local_note(post)));
    deliver(&w.app, &w.bob, &again).await;
    assert_eq!(post_counts(&w.db, post).await, (1, 0));
    assert_eq!(notifications(&w.db, w.ada.id, "like").await, 1);
    let event = live.try_recv().expect("a live event");
    match &*event {
        StreamEvent::Notification(note) => {
            assert_eq!(note.kind, NotificationKindDto::Like);
            assert_eq!(note.post.as_ref().map(|post| post.id), Some(post));
        }
        other => panic!("expected a notification, got {}", other.name()),
    }

    let announce = w.bob.activity("Announce", json!(local_note(post)));
    deliver(&w.app, &w.bob, &announce).await;
    let again = w.bob.activity("Announce", json!(local_note(post)));
    deliver(&w.app, &w.bob, &again).await;
    assert_eq!(post_counts(&w.db, post).await, (1, 1));
    assert_eq!(notifications(&w.db, w.ada.id, "repost").await, 1);
    let ap_id: Option<String> = sqlx::query_scalar("select ap_id from reposts where post_id = $1")
        .bind(post)
        .fetch_one(&w.db)
        .await
        .expect("repost");
    assert_eq!(ap_id.as_deref(), announce["id"].as_str());

    // Undo both: by the embedded activity.
    deliver(&w.app, &w.bob, &w.bob.activity("Undo", like.clone())).await;
    assert_eq!(post_counts(&w.db, post).await, (0, 1));
    deliver(&w.app, &w.bob, &w.bob.activity("Undo", announce.clone())).await;
    assert_eq!(post_counts(&w.db, post).await, (0, 0));
    assert_eq!(
        count(&w.db, "select count(*) from reposts".to_owned()).await,
        0
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn an_announce_id_on_another_host_is_not_stored(db: PgPool) {
    let mut w = world(db).await;
    let post = ada_post(&mut w, "a post").await;
    let mut squat = w.bob.activity("Announce", json!(local_note(post)));
    squat["id"] = json!("https://victim.example/activities/announce-1");
    deliver(&w.app, &w.bob, &squat).await;
    assert_eq!(post_counts(&w.db, post).await, (0, 1), "the repost counts");
    let ap_id: Option<String> = sqlx::query_scalar("select ap_id from reposts where post_id = $1")
        .bind(post)
        .fetch_one(&w.db)
        .await
        .expect("repost");
    assert_eq!(ap_id, None, "an id the actor does not own is not kept");
}

#[sqlx::test(migrations = "./migrations")]
async fn an_announce_is_undone_by_id_and_a_like_of_other_objects_is_ignored(db: PgPool) {
    let mut w = world(db).await;
    let post = ada_post(&mut w, "a post").await;
    let announce = w.bob.activity("Announce", json!(local_note(post)));
    deliver(&w.app, &w.bob, &announce).await;
    assert_eq!(post_counts(&w.db, post).await, (0, 1));
    let undo = w.bob.activity("Undo", announce["id"].clone());
    deliver(&w.app, &w.bob, &undo).await;
    assert_eq!(post_counts(&w.db, post).await, (0, 0));

    // Remote posts, unknown posts and ids of other accounts' handles do nothing.
    let remote_note = w.bob.note_id("9");
    for object in [
        json!(remote_note),
        json!(local_note(Uuid::now_v7())),
        json!(format!("{ORIGIN}/users/nobody/statuses/{post}")),
        json!("https://elsewhere.example/n/1"),
    ] {
        deliver(&w.app, &w.bob, &w.bob.activity("Like", object.clone())).await;
        deliver(&w.app, &w.bob, &w.bob.activity("Announce", object)).await;
    }
    assert_eq!(
        count(&w.db, "select count(*) from likes".to_owned()).await,
        0
    );
    assert_eq!(post_counts(&w.db, post).await, (0, 0));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_blocked_actor_gets_no_like_and_a_muted_one_gets_no_notification(db: PgPool) {
    let mut w = world(db).await;
    let post = ada_post(&mut w, "a post").await;
    let carol = Remote::stored(&w.db, &w.fake, "carol", 1).await;
    sqlx::query("insert into blocks (user_id, target_id) values ($1, $2)")
        .bind(w.ada.id)
        .bind(w.bob.uid)
        .execute(&w.db)
        .await
        .expect("block");
    sqlx::query("insert into mutes (user_id, target_id) values ($1, $2)")
        .bind(w.ada.id)
        .bind(carol.uid)
        .execute(&w.db)
        .await
        .expect("mute");
    deliver(
        &w.app,
        &w.bob,
        &w.bob.activity("Like", json!(local_note(post))),
    )
    .await;
    assert_eq!(post_counts(&w.db, post).await, (0, 0));
    deliver(
        &w.app,
        &carol,
        &carol.activity("Like", json!(local_note(post))),
    )
    .await;
    assert_eq!(post_counts(&w.db, post).await, (1, 0));
    assert_eq!(notifications(&w.db, w.ada.id, "like").await, 0);
}

// ------------------------------------------------------------------ notes

#[sqlx::test(migrations = "./migrations")]
async fn a_note_addressed_to_a_local_account_is_stored_and_notified(db: PgPool) {
    let w = world(db).await;
    let mut note = w
        .bob
        .note_to("1", "hello <b>ada</b>", &[ap::PUBLIC], &[&ada_id()]);
    note["tag"] = json!([{"type": "Mention", "href": ada_id(), "name": "@ada@social.example"}]);
    deliver(&w.app, &w.bob, &w.bob.activity("Create", note)).await;
    let (id, text, deleted) = post_by_ap_id(&w.db, &w.bob.note_id("1"))
        .await
        .expect("stored");
    assert_eq!(text, "hello ada");
    assert!(!deleted);
    assert_eq!(notifications(&w.db, w.ada.id, "mention").await, 1);
    let author: Uuid = sqlx::query_scalar("select author_id from posts where id = $1")
        .bind(id)
        .fetch_one(&w.db)
        .await
        .expect("author");
    assert_eq!(author, w.bob.uid);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_reply_to_a_local_post_is_stored_and_notified(db: PgPool) {
    let mut w = world(db).await;
    let post = ada_post(&mut w, "question?").await;
    let mut note = w.bob.note_to("r1", "an answer", &[ap::PUBLIC], &[]);
    note["inReplyTo"] = json!(local_note(post));
    deliver(&w.app, &w.bob, &w.bob.activity("Create", note)).await;
    assert!(post_by_ap_id(&w.db, &w.bob.note_id("r1")).await.is_some());
    assert_eq!(notifications(&w.db, w.ada.id, "reply").await, 1);
    let replies: i32 = sqlx::query_scalar("select reply_count from posts where id = $1")
        .bind(post)
        .fetch_one(&w.db)
        .await
        .expect("replies");
    assert_eq!(replies, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_reply_the_parent_author_blocked_or_ruled_out_is_dropped(db: PgPool) {
    let mut w = world(db).await;
    let post = ada_post(&mut w, "question?").await;
    let reply = |n: &str| {
        let mut note = w.bob.note_to(n, "an answer", &[ap::PUBLIC], &[]);
        note["inReplyTo"] = json!(local_note(post));
        w.bob.activity("Create", note)
    };
    let replies = |db: PgPool| async move {
        sqlx::query_scalar::<_, i32>("select reply_count from posts where id = $1")
            .bind(post)
            .fetch_one(&db)
            .await
            .expect("replies")
    };

    sqlx::query("update posts set reply_policy = 'mentioned' where id = $1")
        .bind(post)
        .execute(&w.db)
        .await
        .expect("policy");
    deliver(&w.app, &w.bob, &reply("r1")).await;
    assert!(post_by_ap_id(&w.db, &w.bob.note_id("r1")).await.is_none());
    assert_eq!(replies(w.db.clone()).await, 0, "reply policy refused it");

    sqlx::query("update posts set reply_policy = 'everyone' where id = $1")
        .bind(post)
        .execute(&w.db)
        .await
        .expect("policy");
    sqlx::query("insert into blocks (user_id, target_id) values ($1, $2)")
        .bind(w.ada.id)
        .bind(w.bob.uid)
        .execute(&w.db)
        .await
        .expect("block");
    deliver(&w.app, &w.bob, &reply("r2")).await;
    assert!(post_by_ap_id(&w.db, &w.bob.note_id("r2")).await.is_none());
    assert_eq!(replies(w.db.clone()).await, 0, "the block refused it");
    assert_eq!(notifications(&w.db, w.ada.id, "reply").await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_create_not_addressed_to_anyone_here_is_dropped(db: PgPool) {
    let w = world(db).await;
    let note = w.bob.note_to("1", "to nobody here", &[ap::PUBLIC], &[]);
    deliver(&w.app, &w.bob, &w.bob.activity("Create", note)).await;
    assert!(post_by_ap_id(&w.db, &w.bob.note_id("1")).await.is_none());
    assert_eq!(seen(&w.db).await, 1, "the delivery itself was handled");

    // The same note stored once a local account follows the author.
    insert_follow(
        &w.db,
        w.ada.id,
        w.bob.uid,
        "accepted",
        "https://x.example/f",
    )
    .await;
    let note = w.bob.note_to("2", "to my followers", &[ap::PUBLIC], &[]);
    deliver(&w.app, &w.bob, &w.bob.activity("Create", note)).await;
    assert!(post_by_ap_id(&w.db, &w.bob.note_id("2")).await.is_some());
}

#[sqlx::test(migrations = "./migrations")]
async fn another_accounts_followers_collection_does_not_get_a_note_stored(db: PgPool) {
    let w = world(db).await;
    let carol = Remote::stored(&w.db, &w.fake, "carol", 1).await;
    insert_follow(
        &w.db,
        w.ada.id,
        carol.uid,
        "accepted",
        "https://x.example/f",
    )
    .await;
    let carols_followers = format!("{}/followers", carol.id);
    let note = w.bob.note_to(
        "1",
        "hello carol's followers",
        &[ap::PUBLIC],
        &[&carols_followers],
    );
    deliver(&w.app, &w.bob, &w.bob.activity("Create", note)).await;
    assert!(post_by_ap_id(&w.db, &w.bob.note_id("1")).await.is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn a_direct_note_to_one_local_account_is_a_message(db: PgPool) {
    let w = world(db).await;
    let note = w.bob.note_to("dm1", "psst", &[&ada_id()], &[]);
    deliver(&w.app, &w.bob, &w.bob.activity("Create", note)).await;
    let (text, sender): (String, Uuid) =
        sqlx::query_as("select text, sender_id from messages where recipient_id = $1")
            .bind(w.ada.id)
            .fetch_one(&w.db)
            .await
            .expect("message");
    assert_eq!(text, "psst");
    assert_eq!(sender, w.bob.uid);
    assert!(post_by_ap_id(&w.db, &w.bob.note_id("dm1")).await.is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn authorship_rules_drop_what_a_remote_does_not_own(db: PgPool) {
    let mut w = world(db).await;
    let carol = Remote::stored(&w.db, &w.fake, "carol", 1).await;
    let post = ada_post(&mut w, "ada's own words").await;
    let posts_before = count(&w.db, "select count(*) from posts".to_owned()).await;

    // 1. A note that claims a local id as its own.
    let mut claim = w.bob.note_to("c1", "mine now", &[ap::PUBLIC], &[&ada_id()]);
    claim["id"] = json!(local_note(post));
    deliver(&w.app, &w.bob, &w.bob.activity("Create", claim.clone())).await;
    deliver(&w.app, &w.bob, &w.bob.activity("Update", claim)).await;
    // 2. An id on another host than the actor's.
    let mut elsewhere = w
        .bob
        .note_to("c2", "elsewhere", &[ap::PUBLIC], &[&ada_id()]);
    elsewhere["id"] = json!("https://elsewhere.example/notes/1");
    deliver(&w.app, &w.bob, &w.bob.activity("Create", elsewhere)).await;
    // 3. A note attributed to somebody else.
    let mut spoof = w
        .bob
        .note_to("c3", "carol says", &[ap::PUBLIC], &[&ada_id()]);
    spoof["attributedTo"] = json!(carol.id);
    deliver(&w.app, &w.bob, &w.bob.activity("Create", spoof)).await;
    let mut as_ada = w.bob.note_to("c4", "ada says", &[ap::PUBLIC], &[&ada_id()]);
    as_ada["attributedTo"] = json!(ada_id());
    deliver(&w.app, &w.bob, &w.bob.activity("Create", as_ada)).await;
    // 4. Deleting a local post.
    deliver(
        &w.app,
        &w.bob,
        &w.bob.activity("Delete", json!(local_note(post))),
    )
    .await;
    let tombstone = json!({"id": local_note(post), "type": "Tombstone"});
    deliver(&w.app, &w.bob, &w.bob.activity("Delete", tombstone)).await;

    assert_eq!(
        count(&w.db, "select count(*) from posts".to_owned()).await,
        posts_before
    );
    let (text, deleted): (String, bool) =
        sqlx::query_as("select text, deleted_at is not null from posts where id = $1")
            .bind(post)
            .fetch_one(&w.db)
            .await
            .expect("post");
    assert_eq!(text, "ada's own words");
    assert!(!deleted);
    for n in ["c1", "c2", "c3", "c4"] {
        assert!(
            post_by_ap_id(&w.db, &w.bob.note_id(n)).await.is_none(),
            "{n}"
        );
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn an_update_changes_the_authors_note_and_nobody_elses(db: PgPool) {
    let w = world(db).await;
    let carol = Remote::stored(&w.db, &w.fake, "carol", 1).await;
    let note = w
        .bob
        .note_to("1", "first draft", &[ap::PUBLIC], &[&ada_id()]);
    deliver(&w.app, &w.bob, &w.bob.activity("Create", note.clone())).await;

    let mut edited = note.clone();
    edited["content"] = json!("<p>second draft</p>");
    edited["updated"] = json!(ap::format_time(OffsetDateTime::now_utc()));
    // Carol cannot rewrite bob's note, even with bob's id inside.
    deliver(&w.app, &carol, &carol.activity("Update", edited.clone())).await;
    let (_, text, _) = post_by_ap_id(&w.db, &w.bob.note_id("1"))
        .await
        .expect("post");
    assert_eq!(text, "first draft");

    deliver(&w.app, &w.bob, &w.bob.activity("Update", edited)).await;
    let (id, text, _) = post_by_ap_id(&w.db, &w.bob.note_id("1"))
        .await
        .expect("post");
    assert_eq!(text, "second draft");
    let edited_at: Option<OffsetDateTime> =
        sqlx::query_scalar("select edited_at from posts where id = $1")
            .bind(id)
            .fetch_one(&w.db)
            .await
            .expect("edited_at");
    assert!(edited_at.is_some());
}

#[sqlx::test(migrations = "./migrations")]
async fn an_update_of_the_profile_applies_only_to_the_actor_itself(db: PgPool) {
    let w = world(db).await;
    let carol = Remote::stored(&w.db, &w.fake, "carol", 1).await;
    let mut profile = actor_doc(&w.fake.base(), "bob", 0);
    profile["name"] = json!("Robert");
    deliver(&w.app, &carol, &carol.activity("Update", profile.clone())).await;
    let name = |id: Uuid| {
        let db = w.db.clone();
        async move {
            sqlx::query_scalar::<_, String>("select display_name from users where id = $1")
                .bind(id)
                .fetch_one(&db)
                .await
                .expect("name")
        }
    };
    assert_eq!(name(w.bob.uid).await, "bob");
    deliver(&w.app, &w.bob, &w.bob.activity("Update", profile)).await;
    assert_eq!(name(w.bob.uid).await, "Robert");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_delete_removes_the_authors_note_by_id_or_tombstone(db: PgPool) {
    let w = world(db).await;
    let carol = Remote::stored(&w.db, &w.fake, "carol", 1).await;
    for n in ["1", "2", "3"] {
        let note = w.bob.note_to(n, "a note", &[ap::PUBLIC], &[&ada_id()]);
        deliver(&w.app, &w.bob, &w.bob.activity("Create", note)).await;
    }
    // Not the author.
    deliver(
        &w.app,
        &carol,
        &carol.activity("Delete", json!(w.bob.note_id("1"))),
    )
    .await;
    assert!(
        !post_by_ap_id(&w.db, &w.bob.note_id("1"))
            .await
            .expect("post")
            .2
    );

    deliver(
        &w.app,
        &w.bob,
        &w.bob.activity("Delete", json!(w.bob.note_id("1"))),
    )
    .await;
    let tombstone = json!({"id": w.bob.note_id("2"), "type": "Tombstone", "formerType": "Note"});
    deliver(&w.app, &w.bob, &w.bob.activity("Delete", tombstone)).await;
    let embedded = w.bob.note_to("3", "a note", &[ap::PUBLIC], &[]);
    deliver(&w.app, &w.bob, &w.bob.activity("Delete", embedded)).await;
    for n in ["1", "2", "3"] {
        assert!(
            post_by_ap_id(&w.db, &w.bob.note_id(n))
                .await
                .expect("post")
                .2,
            "{n}"
        );
    }
    assert_eq!(counters(&w.db, w.bob.uid).await, (0, 0));
    let posts: i32 = sqlx::query_scalar("select posts_count from users where id = $1")
        .bind(w.bob.uid)
        .fetch_one(&w.db)
        .await
        .expect("posts_count");
    assert_eq!(posts, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_delete_of_a_person_forgets_the_account_only_for_the_account_itself(db: PgPool) {
    let w = world(db).await;
    let carol = Remote::stored(&w.db, &w.fake, "carol", 1).await;
    deliver(&w.app, &carol, &carol.activity("Delete", json!(w.bob.id))).await;
    let gone = |id: Uuid| {
        let db = w.db.clone();
        async move {
            sqlx::query_scalar::<_, bool>("select deleted_at is not null from users where id = $1")
                .bind(id)
                .fetch_one(&db)
                .await
                .expect("deleted")
        }
    };
    assert!(!gone(w.bob.uid).await);
    deliver(&w.app, &w.bob, &w.bob.activity("Delete", json!(w.bob.id))).await;
    assert!(gone(w.bob.uid).await);
    assert!(!gone(carol.uid).await);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_farewell_of_an_account_whose_key_is_gone_is_accepted_and_ignored(db: PgPool) {
    let w = world(db).await;
    let carol = Remote::unknown(&w.fake, "carol", 1);
    w.fake.gone("/users/carol");
    let farewell = carol.activity("Delete", json!(carol.id));
    deliver(&w.app, &carol, &farewell).await;
    assert_eq!(w.fake.requests_for("/users/carol"), 1);
    assert!(
        remote::find_by_ap_id(&w.db, &carol.id)
            .await
            .expect("find")
            .is_none()
    );
    assert_eq!(seen(&w.db).await, 0);

    // Only that exact shape: any other activity from the gone key is refused...
    let follow = carol.activity("Follow", json!(ada_id()));
    send(&w.app, &carol, &follow)
        .await
        .err(StatusCode::UNAUTHORIZED, "Unknown signing key.");
    // ...and so is a Delete of somebody else.
    let other = carol.activity("Delete", json!(w.bob.id));
    send(&w.app, &carol, &other)
        .await
        .err(StatusCode::UNAUTHORIZED, "Unknown signing key.");
}

// ------------------------------------------------------------------ polls

async fn counts_of(db: &PgPool, post: Uuid) -> (Vec<i32>, i32) {
    sqlx::query_as("select counts, voters from polls where post_id = $1")
        .bind(post)
        .fetch_one(db)
        .await
        .expect("poll")
}

fn vote(who: &Remote, n: &str, option: &str, question: Uuid) -> Value {
    let note = json!({
        "id": who.note_id(n),
        "type": "Note",
        "name": option,
        "attributedTo": who.id,
        "to": [ada_id()],
        "inReplyTo": local_note(question),
        "published": ap::format_time(OffsetDateTime::now_utc()),
    });
    who.activity("Create", note)
}

#[sqlx::test(migrations = "./migrations")]
async fn a_remote_vote_counts_on_a_single_choice_poll_once(db: PgPool) {
    let mut w = world(db).await;
    let reply = w
        .ada
        .browser
        .post(
            "/posts",
            json!({"text": "tea or coffee?", "poll": {"options": ["tea", "coffee"], "durationHours": 24}}),
        )
        .await
        .ok();
    let poll = id_of(&reply);
    let carol = Remote::stored(&w.db, &w.fake, "carol", 1).await;

    deliver(&w.app, &w.bob, &vote(&w.bob, "v1", "coffee", poll)).await;
    assert_eq!(counts_of(&w.db, poll).await, (vec![0, 1], 1));
    // A second vote by the same account, a made-up option and a vote with content do nothing.
    deliver(&w.app, &w.bob, &vote(&w.bob, "v2", "tea", poll)).await;
    deliver(&w.app, &carol, &vote(&carol, "v3", "milk", poll)).await;
    assert_eq!(counts_of(&w.db, poll).await, (vec![0, 1], 1));
    deliver(&w.app, &carol, &vote(&carol, "v4", "tea", poll)).await;
    assert_eq!(counts_of(&w.db, poll).await, (vec![1, 1], 2));
    // Votes are never stored as posts.
    assert!(post_by_ap_id(&w.db, &w.bob.note_id("v1")).await.is_none());
    assert_eq!(
        count(&w.db, "select count(*) from poll_votes".to_owned()).await,
        2
    );
    // The vote shows in the API.
    let shown = w.ada.browser.get(&format!("/posts/{poll}")).await.ok();
    assert_eq!(shown["post"]["poll"]["voters"], 2);
    assert_eq!(shown["post"]["poll"]["options"][0]["votes"], 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_multiple_choice_poll_takes_one_create_per_choice(db: PgPool) {
    let mut w = world(db).await;
    let reply = w
        .ada
        .browser
        .post(
            "/posts",
            json!({"text": "pick", "poll": {"options": ["a", "b", "c"], "multiple": true, "durationHours": 24}}),
        )
        .await
        .ok();
    let poll = id_of(&reply);
    deliver(&w.app, &w.bob, &vote(&w.bob, "v1", "a", poll)).await;
    deliver(&w.app, &w.bob, &vote(&w.bob, "v2", "c", poll)).await;
    assert_eq!(counts_of(&w.db, poll).await, (vec![1, 0, 1], 1));
    // The same option again is not counted twice.
    deliver(&w.app, &w.bob, &vote(&w.bob, "v3", "a", poll)).await;
    assert_eq!(counts_of(&w.db, poll).await, (vec![1, 0, 1], 1));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_vote_after_the_end_or_from_a_blocked_account_does_not_count(db: PgPool) {
    let mut w = world(db).await;
    let carol = Remote::stored(&w.db, &w.fake, "carol", 1).await;
    let reply = w
        .ada
        .browser
        .post(
            "/posts",
            json!({"text": "vote", "poll": {"options": ["yes", "no"], "durationHours": 1}}),
        )
        .await
        .ok();
    let poll = id_of(&reply);
    sqlx::query("insert into blocks (user_id, target_id) values ($1, $2)")
        .bind(w.ada.id)
        .bind(carol.uid)
        .execute(&w.db)
        .await
        .expect("block");
    deliver(&w.app, &carol, &vote(&carol, "v1", "yes", poll)).await;
    assert_eq!(counts_of(&w.db, poll).await, (vec![0, 0], 0));
    sqlx::query("update polls set ends_at = now() - interval '1 minute'")
        .execute(&w.db)
        .await
        .expect("end");
    deliver(&w.app, &w.bob, &vote(&w.bob, "v2", "yes", poll)).await;
    assert_eq!(counts_of(&w.db, poll).await, (vec![0, 0], 0));
}

// ------------------------------------------------------------------ reports and the rest

#[sqlx::test(migrations = "./migrations")]
async fn a_flag_files_a_report_under_the_system_account(db: PgPool) {
    let mut w = world(db).await;
    let post = ada_post(&mut w, "reported words").await;
    let flag = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": format!("{}/flags/1", w.bob.base),
        "type": "Flag",
        "actor": w.bob.id,
        "content": "<p>this is <b>spam</b></p>",
        "object": [ada_id(), local_note(post), "https://elsewhere.example/users/x"],
    });
    deliver(&w.app, &w.bob, &flag).await;
    let (reporter, target, reported, reason, note): (String, Uuid, Option<Uuid>, String, String) =
        sqlx::query_as(
            "select r.handle, t.target_id, t.post_id, t.reason, t.note
               from reports t join users r on r.id = t.reporter_id",
        )
        .fetch_one(&w.db)
        .await
        .expect("one report");
    assert_eq!(reporter, "tuita");
    assert_eq!(target, w.ada.id);
    assert_eq!(reported, Some(post));
    assert_eq!(reason, "other");
    assert!(note.contains("spam"), "{note}");
    assert!(note.contains(&w.bob.id), "{note}");
    assert!(!note.contains("<b>"), "{note}");

    // A flag that names nothing local is dropped.
    let flag = json!({
        "id": format!("{}/flags/2", w.bob.base), "type": "Flag", "actor": w.bob.id,
        "content": "nope", "object": ["https://elsewhere.example/users/x"],
    });
    deliver(&w.app, &w.bob, &flag).await;
    assert_eq!(
        count(&w.db, "select count(*) from reports".to_owned()).await,
        1
    );
    // A flag of the account alone, with no post.
    let flag = json!({
        "id": format!("{}/flags/3", w.bob.base), "type": "Flag", "actor": w.bob.id,
        "content": "rude", "object": ada_id(),
    });
    deliver(&w.app, &w.bob, &flag).await;
    assert_eq!(
        count(&w.db, "select count(*) from reports".to_owned()).await,
        2
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn move_and_unknown_activities_are_accepted_and_ignored(db: PgPool) {
    let w = world(db).await;
    let moved = w.bob.activity("Move", json!(w.bob.id));
    deliver(&w.app, &w.bob, &moved).await;
    let odd = w.bob.activity("EmojiReact", json!(ada_id()));
    deliver(&w.app, &w.bob, &odd).await;
    let no_object =
        json!({"id": format!("{}/a/1", w.bob.base), "type": "Create", "actor": w.bob.id});
    deliver(&w.app, &w.bob, &no_object).await;
    assert_eq!(
        count(&w.db, "select count(*) from follows".to_owned()).await,
        0
    );
    assert_eq!(queued(&w.db).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_delivery_to_the_personal_inbox_is_processed_once_like_the_shared_one(db: PgPool) {
    let mut w = world(db).await;
    let post = ada_post(&mut w, "a post").await;
    let like = w.bob.activity("Like", json!(local_note(post)));
    accepted(&post_as(&w.app, "/users/ada/inbox", &w.bob.key, &w.bob.key_id, &like).await);
    deliver(&w.app, &w.bob, &like).await;
    assert_eq!(post_counts(&w.db, post).await, (1, 0));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_note_without_addressing_takes_the_addressing_of_its_activity(db: PgPool) {
    let w = world(db).await;
    let mut note = w.bob.note_to("a1", "addressed outside", &[], &[]);
    note.as_object_mut().expect("object").remove("to");
    note.as_object_mut().expect("object").remove("cc");
    let mut create = w.bob.activity("Create", note);
    create["to"] = json!([ap::PUBLIC]);
    create["cc"] = json!([ada_id()]);
    deliver(&w.app, &w.bob, &create).await;
    assert!(post_by_ap_id(&w.db, &w.bob.note_id("a1")).await.is_some());
}

#[sqlx::test(migrations = "./migrations")]
async fn articles_pages_and_questions_are_stored_as_posts_and_polls(db: PgPool) {
    let w = world(db).await;
    for (n, kind) in [("art", "Article"), ("page", "Page")] {
        let mut note = w.bob.note_to(n, "long form", &[ap::PUBLIC], &[&ada_id()]);
        note["type"] = json!(kind);
        deliver(&w.app, &w.bob, &w.bob.activity("Create", note)).await;
        assert!(
            post_by_ap_id(&w.db, &w.bob.note_id(n)).await.is_some(),
            "{kind}"
        );
    }

    let mut question = w
        .bob
        .note_to("q1", "tea or coffee?", &[ap::PUBLIC], &[&ada_id()]);
    question["type"] = json!("Question");
    question["oneOf"] = json!([
        {"type": "Note", "name": "tea", "replies": {"type": "Collection", "totalItems": 1}},
        {"type": "Note", "name": "coffee", "replies": {"type": "Collection", "totalItems": 0}}
    ]);
    question["endTime"] = json!("2999-01-01T00:00:00Z");
    deliver(&w.app, &w.bob, &w.bob.activity("Create", question.clone())).await;
    let (id, _, _) = post_by_ap_id(&w.db, &w.bob.note_id("q1"))
        .await
        .expect("stored");
    assert_eq!(counts_of(&w.db, id).await.0, vec![1, 0]);

    question["oneOf"][1]["replies"]["totalItems"] = json!(4);
    question["updated"] = json!(ap::format_time(OffsetDateTime::now_utc()));
    deliver(&w.app, &w.bob, &w.bob.activity("Update", question)).await;
    assert_eq!(counts_of(&w.db, id).await.0, vec![1, 4]);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_block_covers_subdomains_and_the_trailing_dot_form_of_a_domain(db: PgPool) {
    let w = world(db).await;
    sqlx::query("insert into domain_blocks (domain, severity) values ('evil.invalid', 'suspend')")
        .execute(&w.db)
        .await
        .expect("block");
    let stale = OffsetDateTime::now_utc() - Duration::hours(2);
    // The signature is stale, so a request that is not dropped for its domain is a 401.
    let from = async |host: &str| {
        let actor = format!("http://{host}/users/ghost");
        let body = serde_json::to_vec(&json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": format!("http://{host}/activities/1"),
            "type": "Follow",
            "actor": actor,
            "object": ada_id(),
        }))
        .expect("json");
        post_at(
            &w.app,
            "/inbox",
            &w.bob.key,
            &format!("{actor}#main-key"),
            &body,
            stale,
        )
        .await
    };
    for host in [
        "evil.invalid",
        "evil.invalid.",
        "EVIL.invalid",
        "sub.evil.invalid",
        "a.b.evil.invalid.",
    ] {
        accepted(&from(host).await);
    }
    from("notevil.invalid")
        .await
        .err(StatusCode::UNAUTHORIZED, "Invalid signature.");
    from("evil.invalid.example")
        .await
        .err(StatusCode::UNAUTHORIZED, "Invalid signature.");
    assert_eq!(seen(&w.db).await, 0);
}

async fn tombstones(db: &PgPool) -> Vec<String> {
    sqlx::query_scalar("select ap_id from ap_tombstones order by ap_id")
        .fetch_all(db)
        .await
        .expect("tombstones")
}

#[sqlx::test(migrations = "./migrations")]
async fn a_create_that_arrives_after_its_delete_is_not_stored(db: PgPool) {
    let w = world(db).await;
    let other = Fake::start().await;
    let carol = Remote::stored(&w.db, &other, "carol", 1).await;
    let note = w.bob.note_to("1", "too late", &[ap::PUBLIC], &[&ada_id()]);

    // A Delete from another instance says nothing about bob's note.
    deliver(
        &w.app,
        &carol,
        &carol.activity("Delete", json!(w.bob.note_id("1"))),
    )
    .await;
    assert!(tombstones(&w.db).await.is_empty());

    deliver(
        &w.app,
        &w.bob,
        &w.bob.activity("Delete", json!(w.bob.note_id("1"))),
    )
    .await;
    assert_eq!(tombstones(&w.db).await, [w.bob.note_id("1")]);

    deliver(&w.app, &w.bob, &w.bob.activity("Create", note.clone())).await;
    assert!(
        post_by_ap_id(&w.db, &w.bob.note_id("1")).await.is_none(),
        "the Create came after the Delete"
    );
    // An Update of a note that is not stored goes through the same door.
    deliver(&w.app, &w.bob, &w.bob.activity("Update", note)).await;
    assert!(post_by_ap_id(&w.db, &w.bob.note_id("1")).await.is_none());

    // Other notes are not affected.
    let second = w.bob.note_to("2", "on time", &[ap::PUBLIC], &[&ada_id()]);
    deliver(&w.app, &w.bob, &w.bob.activity("Create", second)).await;
    assert!(post_by_ap_id(&w.db, &w.bob.note_id("2")).await.is_some());
}

#[sqlx::test(migrations = "./migrations")]
async fn deleting_a_stored_note_leaves_a_tombstone_for_its_author_only(db: PgPool) {
    let w = world(db).await;
    let carol = Remote::stored(&w.db, &w.fake, "carol", 1).await;
    for n in ["1", "2"] {
        let note = w.bob.note_to(n, "a note", &[ap::PUBLIC], &[&ada_id()]);
        deliver(&w.app, &w.bob, &w.bob.activity("Create", note)).await;
    }
    // Carol is not the author of bob's stored note: nothing is deleted or remembered.
    deliver(
        &w.app,
        &carol,
        &carol.activity("Delete", json!(w.bob.note_id("1"))),
    )
    .await;
    assert!(tombstones(&w.db).await.is_empty());
    deliver(
        &w.app,
        &w.bob,
        &w.bob.activity("Delete", json!(w.bob.note_id("2"))),
    )
    .await;
    assert_eq!(tombstones(&w.db).await, [w.bob.note_id("2")]);
}

// ------------------------------------------------------------------ control characters

/// Delivers `activity` as `who` and returns the status: a delivery is a 202, or a 4xx when the
/// sender is at fault. Never a 5xx.
async fn status_of(app: &App, who: &Remote, activity: &Value) -> StatusCode {
    let reply = send(app, who, activity).await;
    assert!(
        !reply.status.is_server_error(),
        "{}: {}",
        reply.status,
        String::from_utf8_lossy(&reply.bytes)
    );
    reply.status
}

#[sqlx::test(migrations = "./migrations")]
async fn control_characters_in_free_text_are_stripped_and_the_note_is_stored(db: PgPool) {
    let w = world(db).await;
    let mut note = w.bob.note_to(
        "1",
        "he\u{0}llo\u{7} wor\tld\nx",
        &[ap::PUBLIC],
        &[&ada_id()],
    );
    note["summary"] = json!("c\u{0}w\u{1b}");
    deliver(&w.app, &w.bob, &w.bob.activity("Create", note)).await;
    let (text, cw): (String, String) =
        sqlx::query_as("select text, cw from posts where ap_id = $1")
            .bind(w.bob.note_id("1"))
            .fetch_one(&w.db)
            .await
            .expect("stored");
    assert_eq!((text.as_str(), cw.as_str()), ("hello wor\tld\nx", "cw"));

    let mut poll = w.bob.note_to("2", "pick", &[ap::PUBLIC], &[&ada_id()]);
    poll["type"] = json!("Question");
    poll["endTime"] = json!(ap::format_time(
        OffsetDateTime::now_utc() + Duration::hours(1)
    ));
    poll["oneOf"] = json!([
        {"type": "Note", "name": "ye\u{0}s", "replies": {"type": "Collection", "totalItems": 1}},
        {"type": "Note", "name": "\u{0}no\u{8}", "replies": {"type": "Collection", "totalItems": 0}}
    ]);
    deliver(&w.app, &w.bob, &w.bob.activity("Create", poll)).await;
    let options: Vec<String> = sqlx::query_scalar(
        "select pl.options from polls pl join posts p on p.id = pl.post_id where p.ap_id = $1",
    )
    .bind(w.bob.note_id("2"))
    .fetch_one(&w.db)
    .await
    .map(|options: Vec<String>| options)
    .expect("poll");
    assert_eq!(options, ["yes", "no"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn control_characters_in_ids_and_urls_drop_the_object_instead_of_failing(db: PgPool) {
    let w = world(db).await;
    let bad = "\u{0}";
    let base = w.bob.note_id("1");
    let posts = || {
        let db = w.db.clone();
        async move { count(&db, "select count(*) from posts".to_owned()).await }
    };

    // A note id with a NUL is not stored.
    let nul_id = format!("{base}{bad}");
    let note = json!({
        "id": nul_id, "type": "Note", "attributedTo": w.bob.id, "content": "<p>x</p>",
        "published": ap::format_time(OffsetDateTime::now_utc()),
        "to": [ap::PUBLIC], "cc": [ada_id()],
    });
    status_of(&w.app, &w.bob, &w.bob.activity("Create", note)).await;
    assert_eq!(posts().await, 0);

    // A NUL in a URL field, a reference or an addressee leaves the note without that field.
    let mut note = w
        .bob
        .note_to("2", "with bad refs", &[ap::PUBLIC], &[&ada_id()]);
    note["url"] = json!(format!("{}{bad}", w.bob.note_id("2")));
    note["inReplyTo"] = json!(format!("{}{bad}", w.bob.note_id("9")));
    note["quoteUrl"] = json!(format!("{}{bad}", w.bob.note_id("8")));
    note["cc"] = json!([ada_id(), format!("{ORIGIN}/users/ad{bad}a"), bad.to_owned()]);
    note["tag"] = json!([
        {"type": "Mention", "href": format!("{ORIGIN}/users/ad{bad}a"), "name": "@ada"},
        {"type": "Hashtag", "name": format!("#ta{bad}g"), "href": format!("https://x.example/{bad}")}
    ]);
    status_of(&w.app, &w.bob, &w.bob.activity("Create", note)).await;
    let row: Option<(Option<String>, Option<Uuid>)> =
        sqlx::query_as("select url, reply_to_id from posts where ap_id = $1")
            .bind(w.bob.note_id("2"))
            .fetch_optional(&w.db)
            .await
            .expect("row");
    if let Some((url, reply_to)) = row {
        assert_eq!((url, reply_to), (None, None));
    }

    // Activities and the objects they name.
    let ada = ada_id();
    for (kind, object) in [
        ("Follow", json!(format!("{ada}{bad}"))),
        ("Like", json!(format!("{base}{bad}"))),
        ("Announce", json!(format!("{base}{bad}"))),
        ("Delete", json!(format!("{base}{bad}"))),
        (
            "Delete",
            json!({"id": format!("{base}{bad}"), "type": "Tombstone"}),
        ),
        (
            "Undo",
            json!({"id": format!("{base}{bad}"), "type": "Follow", "actor": w.bob.id, "object": ada}),
        ),
        ("Accept", json!(format!("{base}{bad}"))),
        ("Block", json!(format!("{ada}{bad}"))),
        ("Flag", json!([format!("{ada}{bad}")])),
    ] {
        let mut activity = w.bob.activity(kind, object);
        activity["id"] = json!(format!("{}/activities/{kind}{bad}", w.bob.base));
        status_of(&w.app, &w.bob, &activity).await;
    }
    assert_eq!(
        seen(&w.db).await,
        2,
        "only the ids of the two Creates, which have none with a NUL, are remembered"
    );
    assert_eq!(queued(&w.db).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn control_characters_in_a_profile_are_stripped(db: PgPool) {
    let w = world(db).await;
    let mut profile = actor_doc(&w.fake.base(), "bob", 0);
    profile["name"] = json!("Rob\u{0}ert\u{7}");
    profile["summary"] = json!("<p>about\u{0} me</p>");
    profile["url"] = json!(format!("{}/@bob\u{0}", w.fake.base()));
    profile["attachment"] = json!([
        {"type": "PropertyValue", "name": "Website", "value": "https://home.example/\u{0}x"}
    ]);
    status_of(&w.app, &w.bob, &w.bob.activity("Update", profile)).await;
    let (name, bio, website, url): (String, String, String, Option<String>) =
        sqlx::query_as("select display_name, bio, website, url from users where id = $1")
            .bind(w.bob.uid)
            .fetch_one(&w.db)
            .await
            .expect("profile");
    assert_eq!(
        (name.as_str(), bio.as_str(), website.as_str(), url),
        ("Robert", "about me", "https://home.example/x", None)
    );
}
