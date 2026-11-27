//! Remote lookup and ingest against a fake remote instance: an axum app on a loopback socket that
//! serves WebFinger, actors, outboxes, notes and media.
#![allow(missing_docs)]

mod common;

use std::collections::HashMap;
use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use axum::Router;
use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tuita_core::ap::{self, Note};
use tuita_server::db::remote::{self, RemoteProfile, Upsert};
use tuita_server::db::users::{self, UserRow};
use tuita_server::federation::ingest::{self, Context, Stored};
use tuita_server::federation::{FedError, lookup};
use uuid::Uuid;

use common::{App, Browser, ORIGIN, app_with, insert_user};

// ------------------------------------------------------------------ the fake instance

#[derive(Default)]
struct Inner {
    docs: Mutex<HashMap<String, Value>>,
    raw: Mutex<HashMap<String, (&'static str, Vec<u8>)>>,
    webfinger: Mutex<HashMap<String, String>>,
    webfinger_status: Mutex<Option<u16>>,
    jrd: Mutex<Option<Vec<u8>>>,
    hits: Mutex<Vec<String>>,
}

struct Fake {
    addr: SocketAddr,
    inner: Arc<Inner>,
}

async fn serve(State(inner): State<Arc<Inner>>, uri: Uri) -> Response {
    inner.hits.lock().expect("lock").push(uri.path().to_owned());
    if uri.path() == "/.well-known/webfinger" {
        if let Some(status) = *inner.webfinger_status.lock().expect("lock") {
            return StatusCode::from_u16(status)
                .expect("status")
                .into_response();
        }
        if let Some(body) = inner.jrd.lock().expect("lock").clone() {
            return ([(CONTENT_TYPE, "application/jrd+json")], body).into_response();
        }
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

    fn hits(&self) -> Vec<String> {
        self.inner.hits.lock().expect("lock").clone()
    }

    fn hits_of(&self, path: &str) -> usize {
        self.hits().iter().filter(|hit| *hit == path).count()
    }

    /// Serves `name` as an actor with an outbox of `notes` (newest first), and makes WebFinger
    /// find it.
    fn add_actor(&self, name: &str, notes: Vec<Value>) -> Value {
        let base = self.base();
        let actor = actor_json(&base, name);
        self.put(&format!("/users/{name}"), actor.clone());
        self.inner.webfinger.lock().expect("lock").insert(
            format!("acct:{name}@{}", self.domain()),
            format!("{base}/users/{name}"),
        );
        self.put(
            &format!("/users/{name}/outbox"),
            json!({
                "id": format!("{base}/users/{name}/outbox"),
                "type": "OrderedCollection",
                "totalItems": notes.len(),
                "first": format!("{base}/users/{name}/outbox?page=true")
            }),
        );
        // Query strings are not part of the path the fake serves.
        let items: Vec<Value> = notes;
        self.put(
            &format!("/users/{name}/outbox"),
            json!({
                "id": format!("{base}/users/{name}/outbox"),
                "type": "OrderedCollection",
                "totalItems": items.len(),
                "first": {
                    "id": format!("{base}/users/{name}/outbox?page=true"),
                    "type": "OrderedCollectionPage",
                    "orderedItems": items
                }
            }),
        );
        self.put_raw("/avatar.png", "image/png", png(64, 48));
        self.put_raw("/banner.png", "image/png", png(300, 100));
        actor
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

fn actor_json(base: &str, name: &str) -> Value {
    let id = format!("{base}/users/{name}");
    json!({
        "@context": ["https://www.w3.org/ns/activitystreams", "https://w3id.org/security/v1"],
        "id": id,
        "type": "Person",
        "preferredUsername": name,
        "name": format!("Name of {name}"),
        "summary": "<p>Bio of <b>the</b> account<script>alert(1)</script></p>",
        "url": format!("{base}/@{name}"),
        "inbox": format!("{id}/inbox"),
        "outbox": format!("{id}/outbox"),
        "followers": format!("{id}/followers"),
        "endpoints": {"sharedInbox": format!("{base}/inbox")},
        "icon": {"type": "Image", "mediaType": "image/png", "url": format!("{base}/avatar.png")},
        "image": {"type": "Image", "url": format!("{base}/banner.png")},
        "attachment": [
            {"type": "PropertyValue", "name": "Website", "value": "<a href=\"https://home.example/ann\">home.example/ann</a>"},
            {"type": "PropertyValue", "name": "Pronouns", "value": "she/her"}
        ],
        "manuallyApprovesFollowers": true,
        "discoverable": true,
        "publicKey": {"id": format!("{id}#main-key"), "owner": id, "publicKeyPem": pem()}
    })
}

fn note_json(base: &str, name: &str, n: u32) -> Value {
    json!({
        "id": format!("{base}/users/{name}/statuses/{n}"),
        "type": "Note",
        "attributedTo": format!("{base}/users/{name}"),
        "content": format!("<p>post number {n}</p>"),
        "published": format!("2026-01-{n:02}T10:00:00Z"),
        "url": format!("{base}/@{name}/{n}"),
        "to": [ap::PUBLIC],
        "cc": [format!("{base}/users/{name}/followers")]
    })
}

fn create(base: &str, name: &str, note: &Value) -> Value {
    json!({
        "id": format!("{}/activity", note["id"].as_str().unwrap_or_default()),
        "type": "Create",
        "actor": format!("{base}/users/{name}"),
        "object": note
    })
}

/// The outbox of `count` public notes of `name`, newest first.
fn outbox_of(base: &str, name: &str, count: u32) -> Vec<Value> {
    (1..=count)
        .rev()
        .map(|n| create(base, name, &note_json(base, name, n)))
        .collect()
}

/// Where this test binary's copied media goes.
static DATA: LazyLock<String> = LazyLock::new(|| {
    std::env::temp_dir()
        .join(format!("tuita-lookup-{}", Uuid::now_v7()))
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

fn authority(base: &str) -> String {
    ap::url_authority(base).expect("authority")
}

/// Stores a remote account without any network traffic.
async fn remote_user(db: &PgPool, base: &str, name: &str) -> UserRow {
    let ap_id = format!("{base}/users/{name}");
    let profile = RemoteProfile {
        handle: name.to_owned(),
        domain: authority(base),
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

fn note(value: &Value) -> Note {
    serde_json::from_value(value.clone()).expect("note")
}

async fn count(db: &PgPool, sql: &'static str, bind: &str) -> i64 {
    sqlx::query_scalar(sql)
        .bind(bind)
        .fetch_one(db)
        .await
        .expect("count")
}

fn local_actor(handle: &str) -> String {
    format!("{ORIGIN}/users/{handle}")
}

// ------------------------------------------------------------------ resolving

#[sqlx::test(migrations = "./migrations")]
async fn an_unknown_account_is_resolved_stored_and_backfilled(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    fake.add_actor("ann", outbox_of(&fake.base(), "ann", 25));

    let row = lookup::resolve_acct(&app.state, None, "ann", &fake.domain())
        .await
        .expect("resolved");
    assert_eq!(row.handle, "ann");
    assert_eq!(row.domain.as_deref(), Some(fake.domain().as_str()));
    assert_eq!(row.display_name, "Name of ann");
    assert_eq!(row.bio, "Bio of the account");
    assert_eq!(row.website, "https://home.example/ann");
    // The Pronouns field the actor carries is not stored.
    assert_eq!(row.manually_approves, Some(true));
    assert_eq!(
        row.url.as_deref(),
        Some(format!("{}/@ann", fake.base()).as_str())
    );
    assert_eq!(
        row.inbox_url.as_deref(),
        Some(format!("{}/users/ann/inbox", fake.base()).as_str())
    );
    assert_eq!(
        row.shared_inbox_url.as_deref(),
        Some(format!("{}/inbox", fake.base()).as_str())
    );
    assert!(row.fetched_at.is_some());
    assert_eq!(row.posts_count, 20, "the first 20 items of the outbox");

    let (avatar, banner): (Option<Uuid>, Option<Uuid>) =
        sqlx::query_as("select avatar_id, banner_id from users where id = $1")
            .bind(row.id)
            .fetch_one(&db)
            .await
            .expect("row");
    let paths: Vec<(String, String, Uuid)> =
        sqlx::query_as("select kind, mime, owner_id from media where id = any($1)")
            .bind(vec![
                avatar.expect("avatar copied"),
                banner.expect("banner copied"),
            ])
            .fetch_all(&db)
            .await
            .expect("media");
    assert_eq!(paths.len(), 2);
    assert!(
        paths
            .iter()
            .all(|(kind, _, owner)| kind == "image" && *owner == row.id)
    );

    // Newest first: posts 25 down to 6 are stored, ordered by their published time.
    let texts: Vec<String> =
        sqlx::query_scalar("select text from posts where author_id = $1 order by id desc limit 3")
            .bind(row.id)
            .fetch_all(&db)
            .await
            .expect("posts");
    assert_eq!(
        texts,
        ["post number 25", "post number 24", "post number 23"]
    );
    let oldest: String =
        sqlx::query_scalar("select text from posts where author_id = $1 order by id asc limit 1")
            .bind(row.id)
            .fetch_one(&db)
            .await
            .expect("oldest");
    assert_eq!(oldest, "post number 6");
    let created: time::OffsetDateTime = sqlx::query_scalar(
        "select created_at from posts where author_id = $1 and text = 'post number 6'",
    )
    .bind(row.id)
    .fetch_one(&db)
    .await
    .expect("created");
    assert_eq!(ap::format_time(created), "2026-01-06T10:00:00Z");

    // A second lookup is served from the database: no request of any kind.
    let before = fake.hits().len();
    lookup::resolve_acct(&app.state, None, "ANN", &fake.domain())
        .await
        .expect("again");
    assert_eq!(fake.hits().len(), before);
}

#[sqlx::test(migrations = "./migrations")]
async fn outbox_items_that_are_ids_are_fetched_one_by_one(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let base = fake.base();
    let ids: Vec<Value> = (1..=22_u32)
        .rev()
        .map(|n| {
            fake.put(
                &format!("/users/bob/statuses/{n}"),
                note_json(&base, "bob", n),
            );
            json!(format!("{base}/users/bob/statuses/{n}"))
        })
        .collect();
    fake.add_actor("bob", ids);
    let row = lookup::resolve_acct(&app.state, None, "bob", &fake.domain())
        .await
        .expect("resolved");
    assert_eq!(row.posts_count, 20);
    let fetched = fake
        .hits()
        .iter()
        .filter(|hit| hit.contains("/statuses/"))
        .count();
    assert_eq!(fetched, 20, "at most 20 individual fetches");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_stale_profile_is_served_stale_and_refreshed_once_in_the_background(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    fake.add_actor("ann", Vec::new());
    let first = lookup::resolve_acct(&app.state, None, "ann", &fake.domain())
        .await
        .expect("resolved");
    assert_eq!(first.display_name, "Name of ann");
    let actor_hits = fake.hits_of("/users/ann");

    let mut changed = actor_json(&fake.base(), "ann");
    changed["name"] = json!("Ann Renamed");
    fake.put("/users/ann", changed);
    sqlx::query("update users set fetched_at = now() - interval '25 hours' where id = $1")
        .bind(first.id)
        .execute(&db)
        .await
        .expect("age");

    let mut burst = Vec::new();
    for _ in 0..6 {
        let state = app.state.clone();
        let domain = fake.domain();
        burst.push(tokio::spawn(async move {
            lookup::resolve_acct(&state, None, "ann", &domain).await
        }));
    }
    for call in burst {
        let row = call.await.expect("task").expect("resolved");
        assert!(
            row.display_name == "Name of ann" || row.display_name == "Ann Renamed",
            "{}",
            row.display_name
        );
    }
    let mut renamed = false;
    for _ in 0..100 {
        let name: String = sqlx::query_scalar("select display_name from users where id = $1")
            .bind(first.id)
            .fetch_one(&db)
            .await
            .expect("name");
        if name == "Ann Renamed" {
            renamed = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(renamed, "the background refresh stored the new profile");
    assert_eq!(
        fake.hits_of("/users/ann"),
        actor_hits + 1,
        "six concurrent views start one refresh"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn no_refresh_starts_once_the_server_is_shutting_down(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    fake.add_actor("ann", Vec::new());
    let first = lookup::resolve_acct(&app.state, None, "ann", &fake.domain())
        .await
        .expect("resolved");
    sqlx::query("update users set fetched_at = now() - interval '25 hours' where id = $1")
        .bind(first.id)
        .execute(&db)
        .await
        .expect("age");
    let hits = fake.hits().len();
    app.state.shutdown().cancel();
    let stale = lookup::resolve_acct(&app.state, None, "ann", &fake.domain())
        .await
        .expect("served stale");
    assert_eq!(stale.id, first.id);
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(fake.hits().len(), hits);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_failed_refresh_is_retried_in_an_hour_not_on_every_view(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    fake.add_actor("ann", Vec::new());
    let first = lookup::resolve_acct(&app.state, None, "ann", &fake.domain())
        .await
        .expect("resolved");
    fake.inner.docs.lock().expect("lock").remove("/users/ann");
    sqlx::query("update users set fetched_at = now() - interval '30 hours' where id = $1")
        .bind(first.id)
        .execute(&db)
        .await
        .expect("age");
    lookup::resolve_acct(&app.state, None, "ann", &fake.domain())
        .await
        .expect("stale");
    let mut delayed = false;
    for _ in 0..100 {
        let age: f64 = sqlx::query_scalar(
            "select extract(epoch from now() - fetched_at)::float8 from users where id = $1",
        )
        .bind(first.id)
        .fetch_one(&db)
        .await
        .expect("age");
        if age < 24.0 * 3600.0 {
            delayed = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(delayed, "a failed refresh pushes the next try an hour out");
}

// ------------------------------------------------------------------ errors and bindings

#[sqlx::test(migrations = "./migrations")]
async fn lookup_failures_map_to_not_found_and_bad_gateway(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let domain = fake.domain();

    let missing = lookup::resolve_acct(&app.state, None, "nobody", &domain).await;
    assert!(
        matches!(missing, Err(FedError::NotFound)),
        "{:?}",
        missing.as_ref().err()
    );

    *fake.inner.webfinger_status.lock().expect("lock") = Some(500);
    let broken = lookup::resolve_acct(&app.state, None, "nobody", &domain).await;
    assert!(
        matches!(broken, Err(FedError::Unreachable(_))),
        "{:?}",
        broken.as_ref().err()
    );
    *fake.inner.webfinger_status.lock().expect("lock") = None;

    // The JRD points at an HTML page.
    fake.inner.webfinger.lock().expect("lock").insert(
        format!("acct:html@{domain}"),
        format!("{}/page", fake.base()),
    );
    fake.put_raw("/page", "text/html", b"<html>hi</html>".to_vec());
    let html = lookup::resolve_acct(&app.state, None, "html", &domain).await;
    assert!(
        matches!(html, Err(FedError::Unreadable(_))),
        "{:?}",
        html.as_ref().err()
    );

    // A closed port.
    let dead = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let dead_domain = dead.local_addr().expect("addr").to_string();
    drop(dead);
    let refused = lookup::resolve_acct(&app.state, None, "ann", &dead_domain).await;
    assert!(
        matches!(refused, Err(FedError::Unreachable(_))),
        "{:?}",
        refused.as_ref().err()
    );

    assert_eq!(
        count(&db, "select count(*) from users where domain = $1", &domain).await,
        0
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn the_http_api_reports_404_and_502_with_the_documented_messages(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let domain = fake.domain();
    let mut viewer = app.person(&db, "viewer").await.browser;

    viewer
        .get(&format!("/users/nobody@{domain}"))
        .await
        .err(StatusCode::NOT_FOUND, "User not found.");

    *fake.inner.webfinger_status.lock().expect("lock") = Some(503);
    viewer.get(&format!("/users/nobody@{domain}")).await.err(
        StatusCode::BAD_GATEWAY,
        &format!("Couldn’t reach {domain} right now."),
    );
    *fake.inner.webfinger_status.lock().expect("lock") = None;

    fake.inner.webfinger.lock().expect("lock").insert(
        format!("acct:html@{domain}"),
        format!("{}/page", fake.base()),
    );
    fake.put_raw("/page", "text/html", b"<html>hi</html>".to_vec());
    viewer.get(&format!("/users/html@{domain}")).await.err(
        StatusCode::BAD_GATEWAY,
        &format!("{domain} sent something tuita can’t read."),
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn an_actor_that_breaks_a_binding_rule_is_refused_and_nothing_is_stored(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let other = Fake::start().await;
    let base = fake.base();
    let domain = fake.domain();
    let serve = |name: &str, edit: &dyn Fn(&mut Value)| {
        let mut actor = actor_json(&base, name);
        edit(&mut actor);
        fake.put(&format!("/users/{name}"), actor);
        fake.inner.webfinger.lock().expect("lock").insert(
            format!("acct:{name}@{domain}"),
            format!("{base}/users/{name}"),
        );
    };
    serve("elsewhere", &|actor| {
        actor["id"] = json!(format!("{}/users/elsewhere", other.base()));
        actor["publicKey"]["owner"] = actor["id"].clone();
    });
    serve("owner", &|actor| {
        actor["publicKey"]["owner"] = json!(format!("{}/users/someone-else", fake.base()));
    });
    serve("keyhost", &|actor| {
        actor["publicKey"]["id"] = json!(format!("{}/users/keyhost#main-key", other.base()));
    });
    serve("renamed", &|actor| {
        actor["preferredUsername"] = json!("somebody");
    });
    serve("instance", &|actor| actor["type"] = json!("Application"));
    serve("nokey", &|actor| {
        actor.as_object_mut().expect("object").remove("publicKey");
    });
    serve("badkey", &|actor| {
        actor["publicKey"]["publicKeyPem"] =
            json!("-----BEGIN PUBLIC KEY-----\nnope\n-----END PUBLIC KEY-----");
    });
    serve("noinbox", &|actor| {
        actor.as_object_mut().expect("object").remove("inbox");
    });
    serve("badinbox", &|actor| {
        actor["inbox"] = json!("javascript:alert(1)")
    });
    serve("localid", &|actor| {
        actor["id"] = json!(format!("{ORIGIN}/users/ada"));
        actor["publicKey"]["owner"] = actor["id"].clone();
    });
    serve("weird", &|actor| {
        actor["preferredUsername"] = json!("weird handle");
    });

    for name in [
        "elsewhere",
        "owner",
        "keyhost",
        "renamed",
        "nokey",
        "badkey",
        "noinbox",
        "badinbox",
        "localid",
        "weird",
    ] {
        let got = lookup::resolve_acct(&app.state, None, name, &domain).await;
        assert!(
            matches!(
                got,
                Err(FedError::Unreadable(_)) | Err(FedError::Unreachable(_))
            ),
            "{name}: {:?}",
            got.as_ref().err()
        );
    }
    let app_actor = lookup::resolve_acct(&app.state, None, "instance", &domain).await;
    assert!(
        matches!(app_actor, Err(FedError::NotFound)),
        "{:?}",
        app_actor.as_ref().err()
    );
    assert_eq!(
        count(
            &db,
            "select count(*) from users where domain is not null",
            ""
        )
        .await,
        0,
        "no refused actor left a row"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_jrd_without_a_usable_activitypub_self_link_is_refused(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let domain = fake.domain();
    let base = fake.base();
    let serve_jrd = |body: Vec<u8>| {
        *fake.inner.jrd.lock().expect("lock") = Some(body);
    };
    let ask = || lookup::resolve_acct(&app.state, None, "ann", &domain);

    let wrong_type =
        json!({"links": [{"rel": "self", "type": "text/html", "href": format!("{base}/x")}]});
    serve_jrd(wrong_type.to_string().into_bytes());
    assert!(matches!(ask().await, Err(FedError::NotFound)));
    let wrong_scheme = json!({"links": [{"rel": "self", "type": "application/activity+json", "href": "ftp://x.example/u"}]});
    serve_jrd(wrong_scheme.to_string().into_bytes());
    assert!(matches!(ask().await, Err(FedError::NotFound)));
    serve_jrd(b"<xml>not json</xml>".to_vec());
    assert!(matches!(ask().await, Err(FedError::Unreadable(_))));
    serve_jrd(vec![b' '; 300 * 1024]);
    assert!(matches!(ask().await, Err(FedError::Unreadable(_))));
    assert_eq!(
        count(
            &db,
            "select count(*) from users where domain is not null",
            ""
        )
        .await,
        0
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn hostile_documents_are_unreadable(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let domain = fake.domain();
    let base = fake.base();
    let wf = |name: &str| {
        fake.inner.webfinger.lock().expect("lock").insert(
            format!("acct:{name}@{domain}"),
            format!("{base}/users/{name}"),
        );
    };
    wf("huge");
    let mut huge = actor_json(&base, "huge");
    huge["summary"] = json!("x".repeat(3 * 1024 * 1024));
    fake.put_raw(
        "/users/huge",
        "application/activity+json",
        huge.to_string().into_bytes(),
    );
    wf("deep");
    let mut deep = actor_json(&base, "deep").to_string();
    deep.pop();
    deep.push_str(&format!(r#","x":{}1{}}}"#, "[".repeat(60), "]".repeat(60)));
    fake.put_raw(
        "/users/deep",
        "application/activity+json",
        deep.into_bytes(),
    );
    wf("wide");
    let mut wide = actor_json(&base, "wide").to_string();
    wide.pop();
    wide.push_str(&format!(r#","x":[{}1]}}"#, "1,".repeat(30_000)));
    fake.put_raw(
        "/users/wide",
        "application/activity+json",
        wide.into_bytes(),
    );
    wf("array");
    fake.put_raw(
        "/users/array",
        "application/activity+json",
        b"[1,2,3]".to_vec(),
    );
    for name in ["huge", "deep", "wide", "array"] {
        let got = lookup::resolve_acct(&app.state, None, name, &domain).await;
        assert!(
            matches!(got, Err(FedError::Unreadable(_))),
            "{name}: {:?}",
            got.as_ref().err()
        );
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn a_suspended_domain_is_never_contacted(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    fake.add_actor("ann", outbox_of(&fake.base(), "ann", 3));
    let domain = fake.domain();
    sqlx::query("insert into domain_blocks (domain, severity) values ($1, 'suspend')")
        .bind(&domain)
        .execute(&db)
        .await
        .expect("block");

    let got = lookup::resolve_acct(&app.state, None, "ann", &domain).await;
    assert!(
        matches!(got, Err(FedError::NotFound)),
        "{:?}",
        got.as_ref().err()
    );
    let post = lookup::fetch_remote_post(
        &app.state,
        None,
        &format!("{}/users/ann/statuses/1", fake.base()),
    )
    .await;
    assert!(
        matches!(post, Err(FedError::NotFound)),
        "{:?}",
        post.as_ref().err()
    );
    let by_url = lookup::actor_by_url(&app.state, &format!("{}/users/ann", fake.base())).await;
    assert!(
        matches!(by_url, Err(FedError::NotFound)),
        "{:?}",
        by_url.as_ref().err()
    );
    assert!(
        fake.hits().is_empty(),
        "no request reached the blocked domain"
    );

    // A silence block still resolves: it only hides the instance from public places.
    sqlx::query("update domain_blocks set severity = 'silence' where domain = $1")
        .bind(&domain)
        .execute(&db)
        .await
        .expect("silence");
    lookup::resolve_acct(&app.state, None, "ann", &domain)
        .await
        .expect("resolves under silence");
    assert!(!fake.hits().is_empty());

    // Lifting the suspension hid nothing for good: a stored account is hidden while blocked.
    sqlx::query("update domain_blocks set severity = 'suspend' where domain = $1")
        .bind(&domain)
        .execute(&db)
        .await
        .expect("suspend");
    let hits = fake.hits().len();
    let hidden = lookup::resolve_acct(&app.state, None, "ann", &domain).await;
    assert!(
        matches!(hidden, Err(FedError::NotFound)),
        "{:?}",
        hidden.as_ref().err()
    );
    assert_eq!(fake.hits().len(), hits);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_remote_cannot_claim_this_instances_domain(db: PgPool) {
    let app = app_with(db.clone(), open);
    insert_user(&db, "ada").await;
    let own = app.state.config().domain().to_owned();
    // The address names this instance: no network, no remote account.
    let got = lookup::resolve_acct(&app.state, None, "mallory", &own).await;
    assert!(
        matches!(got, Err(FedError::NotFound)),
        "{:?}",
        got.as_ref().err()
    );

    // A fake host serving an actor whose id is one of ours is refused.
    let fake = Fake::start().await;
    let domain = fake.domain();
    let mut claim = actor_json(&fake.base(), "ada");
    claim["id"] = json!(local_actor("ada"));
    claim["publicKey"]["owner"] = claim["id"].clone();
    fake.put("/users/ada", claim);
    fake.inner.webfinger.lock().expect("lock").insert(
        format!("acct:ada@{domain}"),
        format!("{}/users/ada", fake.base()),
    );
    let got = lookup::resolve_acct(&app.state, None, "ada", &domain).await;
    assert!(
        matches!(got, Err(FedError::Unreadable(_))),
        "{:?}",
        got.as_ref().err()
    );

    // The same through the actor-update path of the inbox pipeline.
    let ann = remote_user(&db, "https://remote.example", "ann").await;
    let mut hijack: ap::Actor =
        serde_json::from_value(actor_json("https://remote.example", "ann")).expect("actor");
    hijack.id = local_actor("ada");
    let got = ingest::apply_actor_update(&app.state, &ann, hijack).await;
    assert!(
        matches!(got, Err(FedError::Unreadable(_))),
        "{:?}",
        got.as_ref().err()
    );

    let local_ada: Option<String> =
        sqlx::query_scalar("select ap_id from users where handle = 'ada' and domain is null")
            .fetch_one(&db)
            .await
            .expect("ada");
    assert_eq!(local_ada, None, "the local account kept its identity");
}

#[sqlx::test(migrations = "./migrations")]
async fn only_lookups_that_use_the_network_spend_the_limit(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    fake.add_actor("ann", Vec::new());
    let domain = fake.domain();
    let viewer = insert_user(&db, "viewer").await;
    for _ in 0..40 {
        // A stored account never reaches the network.
        lookup::resolve_acct(&app.state, Some(viewer), "ann", &domain)
            .await
            .expect("known");
    }
    for n in 0..29 {
        let got =
            lookup::resolve_acct(&app.state, Some(viewer), &format!("ghost{n}"), &domain).await;
        assert!(
            matches!(got, Err(FedError::NotFound)),
            "{n}: {:?}",
            got.as_ref().err()
        );
    }
    // The first lookup of "ann" above already spent one: 29 of the 30 misses fit.
    let over = lookup::resolve_acct(&app.state, Some(viewer), "ghost99", &domain).await;
    assert!(
        matches!(over, Err(FedError::RateLimited { .. })),
        "{:?}",
        over.as_ref().err()
    );
    // Known accounts are still served.
    lookup::resolve_acct(&app.state, Some(viewer), "ann", &domain)
        .await
        .expect("known account is free");
}

// ------------------------------------------------------------------ routes

#[sqlx::test(migrations = "./migrations")]
async fn a_remote_handle_works_in_the_users_routes(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    fake.add_actor("ann", outbox_of(&fake.base(), "ann", 21));
    let domain = fake.domain();
    let mut me = app.person(&db, "viewer").await.browser;

    let card = me.get(&format!("/users/ann@{domain}")).await.ok();
    assert_eq!(card["handle"], "ann");
    assert_eq!(card["domain"], domain);
    assert_eq!(card["displayName"], "Name of ann");
    assert_eq!(card["bio"], "Bio of the account");
    assert_eq!(card["locked"], true);
    assert_eq!(card["url"], format!("{}/@ann", fake.base()));
    assert_eq!(card["counts"]["posts"], 20);
    assert!(card["avatarUrl"].is_string());

    let with_at = me.get(&format!("/users/@ann@{domain}")).await.ok();
    assert_eq!(with_at["id"], card["id"]);

    let posts = me.get(&format!("/users/ann@{domain}/posts")).await.ok();
    assert_eq!(posts["items"].as_array().expect("items").len(), 20);
    assert_eq!(posts["items"][0]["text"], "post number 21");
    assert_eq!(posts["items"][0]["visibility"], "public");
    assert_eq!(posts["items"][0]["url"], format!("{}/@ann/21", fake.base()));

    // This instance's own domain is the local account.
    let ada = insert_user(&db, "ada").await;
    let own = me
        .get(&format!("/users/ada@{}", app.state.config().domain()))
        .await
        .ok();
    assert_eq!(own["id"], ada.to_string());
    assert_eq!(own["domain"], Value::Null);

    // Following leaves a pending request: delivery is the outbound slice's job.
    let followed = me
        .post_empty(&format!("/users/ann@{domain}/follow"))
        .await
        .ok();
    assert_eq!(followed["relation"]["followPending"], true);
    assert_eq!(followed["relation"]["following"], false);
    assert_eq!(followed["counts"]["followers"], 0);
    let state: String = sqlx::query_scalar(
        "select f.state from follows f join users u on u.id = f.followee_id where u.handle = 'ann'",
    )
    .fetch_one(&db)
    .await
    .expect("follow");
    assert_eq!(state, "pending");
    let unfollowed = me.delete(&format!("/users/ann@{domain}/follow")).await.ok();
    assert_eq!(unfollowed["relation"]["followPending"], false);
    assert_eq!(unfollowed["counts"]["followers"], 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn search_finds_a_remote_account_and_a_remote_post(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let base = fake.base();
    fake.add_actor("ann", outbox_of(&base, "ann", 2));
    let mut single = note_json(&base, "ann", 9);
    single["content"] = json!("<p>a post to find by url</p>");
    fake.put("/users/ann/statuses/9", single);
    let domain = fake.domain();
    let mut me = app.person(&db, "viewer").await.browser;

    for q in [format!("@ann@{domain}"), format!("ann@{domain}")] {
        for kind in ["posts", "people"] {
            let found = me.get(&format!("/search?q={q}&type={kind}")).await.ok();
            let users = found["users"].as_array().expect("users");
            assert_eq!(users.len(), 1, "{q} {kind}: {found}");
            assert_eq!(users[0]["handle"], "ann");
            assert_eq!(users[0]["domain"], domain);
        }
    }

    let url = format!("{base}/users/ann/statuses/9");
    let found = me.get(&format!("/search?q={url}")).await.ok();
    let posts = found["posts"].as_array().expect("posts");
    assert_eq!(posts.len(), 1, "{found}");
    assert_eq!(posts[0]["text"], "a post to find by url");
    assert_eq!(posts[0]["author"]["handle"], "ann");

    // The page address (`url`) of a stored post finds it again, with no request.
    let hits = fake.hits().len();
    let again = me.get(&format!("/search?q={base}/@ann/9")).await.ok();
    assert_eq!(again["posts"].as_array().expect("posts").len(), 1);
    assert_eq!(fake.hits().len(), hits);

    // Failures are empty results, never errors.
    for q in [
        format!("@ghost@{domain}"),
        format!("{base}/users/ann/statuses/404"),
        "@x@127.0.0.1:1".to_owned(),
        "http://127.0.0.1:1/nope".to_owned(),
    ] {
        let got = me.get(&format!("/search?q={q}")).await;
        let body = got.ok();
        assert_eq!(body["users"], json!([]), "{q}");
        assert_eq!(body["posts"], json!([]), "{q}");
    }

    // This instance's own handle form finds the local account.
    insert_user(&db, "ada").await;
    let own = me
        .get(&format!("/search?q=ada@{}", app.state.config().domain()))
        .await
        .ok();
    assert_eq!(own["users"][0]["handle"], "ada");
}

// ------------------------------------------------------------------ ingest

#[sqlx::test(migrations = "./migrations")]
async fn a_note_maps_to_a_post(db: PgPool) {
    let app = app_with(db.clone(), open);
    let base = "https://remote.example";
    let ann = remote_user(&db, base, "ann").await;
    insert_user(&db, "ada").await;
    insert_user(&db, "bea").await;
    let followers = format!("{base}/users/ann/followers");
    let cases = [
        (json!([ap::PUBLIC]), json!([&followers]), "public"),
        (json!([&followers]), json!([ap::PUBLIC]), "unlisted"),
        (json!([&followers]), json!([]), "followers"),
        (
            json!([local_actor("ada"), local_actor("bea")]),
            json!([]),
            "direct",
        ),
    ];
    for (n, (to, cc, want)) in cases.into_iter().enumerate() {
        let mut value = note_json(base, "ann", u32::try_from(n + 1).expect("n"));
        value["to"] = to;
        value["cc"] = cc;
        value["summary"] = json!("spoiler &amp; more");
        value["sensitive"] = json!(true);
        value["content"] = json!(
            "<p>hello <a href=\"https://remote.example/tags/Rust\" class=\"mention hashtag\" rel=\"tag\">#<span>Rust</span></a><br>second line</p><p>next <script>alert(1)</script>paragraph</p>"
        );
        value["tag"] = json!([
            {"type": "Hashtag", "name": "#Rust", "href": "https://remote.example/tags/rust"},
            {"type": "Hashtag", "name": "#RUST"},
            {"type": "Emoji", "name": ":blob:", "icon": {"url": "https://remote.example/e.png"}}
        ]);
        let stored = ingest::store_note(&app.state, &ann, &note(&value), Context::Lookup)
            .await
            .expect("stored");
        let Stored::Post { id, created: true } = stored else {
            panic!("{want}: {stored:?}");
        };
        let row: (String, String, bool, String, Option<String>, bool) = sqlx::query_as(
            "select text, cw, sensitive, visibility, url, edited_at is null from posts where id = $1",
        )
        .bind(id)
        .fetch_one(&db)
        .await
        .expect("row");
        assert_eq!(
            row.0, "hello #Rust\nsecond line\n\nnext paragraph",
            "{want}"
        );
        assert_eq!(row.1, "spoiler & more");
        assert!(row.2);
        assert_eq!(row.3, want);
        assert_eq!(
            row.4.as_deref(),
            Some(format!("{base}/@ann/{}", n + 1).as_str())
        );
        assert!(row.5);
        let tags: Vec<String> =
            sqlx::query_scalar("select tag from post_tags where post_id = $1 order by tag")
                .bind(id)
                .fetch_all(&db)
                .await
                .expect("tags");
        assert_eq!(tags, ["rust"], "emoji and repeats are dropped");
    }
    let posts_count: i32 = sqlx::query_scalar("select posts_count from users where id = $1")
        .bind(ann.id)
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(posts_count, 4);
}

#[sqlx::test(migrations = "./migrations")]
async fn long_text_is_cut_and_dates_are_clamped(db: PgPool) {
    let app = app_with(db.clone(), open);
    let base = "https://remote.example";
    let ann = remote_user(&db, base, "ann").await;
    let mut value = note_json(base, "ann", 1);
    value["content"] = json!(format!("<p>{}</p>", "ä".repeat(6000)));
    value["published"] = json!("2999-01-01T00:00:00Z");
    value["updated"] = json!("2998-01-01T00:00:00Z");
    let Stored::Post { id, .. } =
        ingest::store_note(&app.state, &ann, &note(&value), Context::Lookup)
            .await
            .expect("stored")
    else {
        panic!("not a post");
    };
    let (text, created, edited): (String, time::OffsetDateTime, Option<time::OffsetDateTime>) =
        sqlx::query_as("select text, created_at, edited_at from posts where id = $1")
            .bind(id)
            .fetch_one(&db)
            .await
            .expect("row");
    assert_eq!(text.chars().count(), ingest::MAX_TEXT_CHARS);
    assert!(created <= time::OffsetDateTime::now_utc());
    assert!(
        edited.is_none(),
        "an edit time that is not after the post is no edit"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn storing_the_same_note_twice_updates_and_never_duplicates(db: PgPool) {
    let app = app_with(db.clone(), open);
    let base = "https://remote.example";
    let ann = remote_user(&db, base, "ann").await;
    let value = note_json(base, "ann", 1);
    let first = ingest::store_note(&app.state, &ann, &note(&value), Context::Backfill)
        .await
        .expect("first");
    let Stored::Post { id, created: true } = first else {
        panic!("{first:?}");
    };
    let again = ingest::store_note(&app.state, &ann, &note(&value), Context::Inbox).await;
    assert!(matches!(again, Ok(Stored::Post { created: false, id: same }) if same == id));
    let mut edited = value.clone();
    edited["content"] = json!("<p>edited text</p>");
    edited["updated"] = json!("2026-01-02T00:00:00Z");
    let updated = ingest::apply_note_update(&app.state, &ann, &note(&edited))
        .await
        .expect("updated");
    assert_eq!(updated, Stored::Post { id, created: false });
    assert_eq!(
        count(
            &db,
            "select count(*) from posts where ap_id = $1",
            &note(&value).id
        )
        .await,
        1
    );
    let (text, edited_at): (String, Option<time::OffsetDateTime>) =
        sqlx::query_as("select text, edited_at from posts where id = $1")
            .bind(id)
            .fetch_one(&db)
            .await
            .expect("row");
    assert_eq!(text, "edited text");
    assert_eq!(
        edited_at.map(ap::format_time).as_deref(),
        Some("2026-01-02T00:00:00Z")
    );
    let history: Vec<String> = sqlx::query_scalar("select text from post_edits where post_id = $1")
        .bind(id)
        .fetch_all(&db)
        .await
        .expect("history");
    assert_eq!(history, ["post number 1"]);

    // Another account cannot edit or delete it.
    let bob = remote_user(&db, base, "bob").await;
    let mut hijack = edited.clone();
    hijack["attributedTo"] = json!(format!("{base}/users/bob"));
    hijack["content"] = json!("<p>hijacked</p>");
    let got = ingest::store_note(&app.state, &bob, &note(&hijack), Context::Lookup)
        .await
        .expect("handled");
    assert_eq!(got, Stored::Dropped);
    assert!(
        !ingest::delete_note_by_ap_id(&app.state, &bob, &note(&value).id)
            .await
            .expect("delete")
    );
    assert!(
        ingest::delete_note_by_ap_id(&app.state, &ann, &note(&value).id)
            .await
            .expect("delete")
    );
    let (deleted, count_after): (bool, i32) = sqlx::query_as(
        "select p.deleted_at is not null, u.posts_count from posts p join users u on u.id = p.author_id where p.id = $1",
    )
    .bind(id)
    .fetch_one(&db)
    .await
    .expect("row");
    assert!(deleted);
    assert_eq!(count_after, 0);
    // A deleted post stays deleted when the same id arrives again.
    let revived = ingest::store_note(&app.state, &ann, &note(&value), Context::Lookup)
        .await
        .expect("handled");
    assert_eq!(revived, Stored::Dropped);
}

#[sqlx::test(migrations = "./migrations")]
async fn notes_that_break_a_binding_rule_are_unreadable(db: PgPool) {
    let app = app_with(db.clone(), open);
    let base = "https://remote.example";
    let ann = remote_user(&db, base, "ann").await;
    let bad = [
        ("id on another host", {
            let mut value = note_json(base, "ann", 1);
            value["id"] = json!("https://evil.example/users/ann/statuses/1");
            value
        }),
        ("another author", {
            let mut value = note_json(base, "ann", 2);
            value["attributedTo"] = json!(format!("{base}/users/bob"));
            value
        }),
        ("no author", {
            let mut value = note_json(base, "ann", 3);
            value
                .as_object_mut()
                .expect("object")
                .remove("attributedTo");
            value
        }),
        ("a local id", {
            let mut value = note_json(base, "ann", 4);
            value["id"] = json!(format!("{ORIGIN}/users/ada/statuses/{}", Uuid::now_v7()));
            value
        }),
        ("not a url", {
            let mut value = note_json(base, "ann", 5);
            value["id"] = json!("javascript:alert(1)");
            value
        }),
    ];
    for (what, value) in bad {
        let got = ingest::store_note(&app.state, &ann, &note(&value), Context::Lookup).await;
        assert!(
            matches!(got, Err(FedError::Unreadable(_))),
            "{what}: {got:?}"
        );
    }
    assert_eq!(
        count(
            &db,
            "select count(*) from posts where author_id::text <> $1",
            ""
        )
        .await,
        0
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn backfill_keeps_public_posts_only_and_empty_posts_are_dropped(db: PgPool) {
    let app = app_with(db.clone(), open);
    let base = "https://remote.example";
    let ann = remote_user(&db, base, "ann").await;
    let mut private = note_json(base, "ann", 1);
    private["to"] = json!([format!("{base}/users/ann/followers")]);
    private["cc"] = json!([]);
    assert_eq!(
        ingest::store_note(&app.state, &ann, &note(&private), Context::Backfill)
            .await
            .expect("handled"),
        Stored::Dropped
    );
    let mut unlisted = note_json(base, "ann", 2);
    unlisted["to"] = json!([format!("{base}/users/ann/followers")]);
    unlisted["cc"] = json!([ap::PUBLIC]);
    assert!(matches!(
        ingest::store_note(&app.state, &ann, &note(&unlisted), Context::Backfill).await,
        Ok(Stored::Post { created: true, .. })
    ));
    let mut empty = note_json(base, "ann", 3);
    empty["content"] = json!("<p></p>");
    assert_eq!(
        ingest::store_note(&app.state, &ann, &note(&empty), Context::Lookup)
            .await
            .expect("handled"),
        Stored::Dropped
    );
}

// ---- should_store

#[sqlx::test(migrations = "./migrations")]
async fn should_store_follows_the_plans_rule(db: PgPool) {
    let app = app_with(db.clone(), open);
    let base = "https://remote.example";
    let ann = remote_user(&db, base, "ann").await;
    let stranger = remote_user(&db, base, "stranger").await;
    let followed = remote_user(&db, base, "followed").await;
    let pending = remote_user(&db, base, "pending").await;
    let ada = insert_user(&db, "ada").await;
    for (target, state) in [(followed.id, "accepted"), (pending.id, "pending")] {
        sqlx::query("insert into follows (follower_id, followee_id, state) values ($1, $2, $3)")
            .bind(ada)
            .bind(target)
            .bind(state)
            .execute(&db)
            .await
            .expect("follow");
    }
    let ada_post = Uuid::now_v7();
    sqlx::query("insert into posts (id, author_id, text) values ($1, $2, 'local')")
        .bind(ada_post)
        .bind(ada)
        .execute(&db)
        .await
        .expect("post");

    let public = |author: &str| {
        let mut value = note_json(base, author, 1);
        value["to"] = json!([ap::PUBLIC]);
        value["cc"] = json!([]);
        value
    };
    let check = |author: &UserRow, value: Value| {
        let state = app.state.clone();
        let author = author.clone();
        async move {
            ingest::should_store(&state, &author, &note(&value))
                .await
                .expect("checked")
        }
    };

    assert!(
        !check(&stranger, public("stranger")).await,
        "a stranger's public post"
    );
    let mut to_ada = public("stranger");
    to_ada["to"] = json!([local_actor("ada")]);
    assert!(check(&stranger, to_ada).await, "addressed to a local user");
    let mut cc_ada = public("stranger");
    cc_ada["cc"] = json!([local_actor("ADA")]);
    assert!(
        check(&stranger, cc_ada).await,
        "the handle compares without case"
    );
    let mut mention = public("stranger");
    mention["tag"] = json!([{"type": "Mention", "href": local_actor("ada"), "name": "@ada"}]);
    assert!(check(&stranger, mention).await, "mentions a local user");
    let mut unknown = public("stranger");
    unknown["to"] = json!([local_actor("nobody")]);
    assert!(
        !check(&stranger, unknown).await,
        "a local id that is no account"
    );
    let mut reply = public("stranger");
    reply["inReplyTo"] = json!(format!("{ORIGIN}/users/ada/statuses/{ada_post}"));
    assert!(check(&stranger, reply).await, "a reply to a local post");
    let mut wrong_reply = public("stranger");
    wrong_reply["inReplyTo"] = json!(format!("{ORIGIN}/users/ada/statuses/{}", Uuid::now_v7()));
    assert!(!check(&stranger, wrong_reply).await, "a reply to nothing");
    let mut remote_reply = public("stranger");
    remote_reply["inReplyTo"] = json!(format!("{base}/users/ann/statuses/1"));
    assert!(
        !check(&stranger, remote_reply).await,
        "a reply to a remote post"
    );
    assert!(
        check(&followed, public("followed")).await,
        "by a followed account"
    );
    assert!(
        check(&pending, public("pending")).await,
        "by an account with a pending follow"
    );
    let mut to_followers = public("ann");
    to_followers["to"] = json!([format!("{base}/users/followed/followers")]);
    assert!(
        !check(&ann, to_followers).await,
        "to the followers of a followed account that is not the author"
    );
    let mut other_followers = public("ann");
    other_followers["to"] = json!([format!("{base}/users/stranger/followers")]);
    assert!(
        !check(&ann, other_followers).await,
        "to the followers of a stranger"
    );

    // The same rule decides what store_note keeps for the inbox.
    assert_eq!(
        ingest::store_note(
            &app.state,
            &stranger,
            &note(&public("stranger")),
            Context::Inbox
        )
        .await
        .expect("handled"),
        Stored::Dropped
    );
    assert!(matches!(
        ingest::store_note(
            &app.state,
            &followed,
            &note(&public("followed")),
            Context::Inbox
        )
        .await,
        Ok(Stored::Post { created: true, .. })
    ));
}

// ---- links: replies, quotes, mentions, notifications

#[sqlx::test(migrations = "./migrations")]
async fn replies_quotes_and_mentions_link_and_notify_local_accounts(db: PgPool) {
    let app = app_with(db.clone(), open);
    let base = "https://remote.example";
    let ann = remote_user(&db, base, "ann").await;
    let ada = insert_user(&db, "ada").await;
    let bea = insert_user(&db, "bea").await;
    let ada_post = Uuid::now_v7();
    sqlx::query("insert into posts (id, author_id, text) values ($1, $2, 'local')")
        .bind(ada_post)
        .bind(ada)
        .execute(&db)
        .await
        .expect("post");
    let ada_note = format!("{ORIGIN}/users/ada/statuses/{ada_post}");

    let mut reply = note_json(base, "ann", 1);
    reply["inReplyTo"] = json!(ada_note);
    reply["tag"] = json!([
        {"type": "Mention", "href": local_actor("ada"), "name": "@ada"},
        {"type": "Mention", "href": local_actor("bea"), "name": "@bea"},
        {"type": "Mention", "href": "https://elsewhere.example/users/zed", "name": "@zed@elsewhere.example"}
    ]);
    let Stored::Post { id: reply_id, .. } =
        ingest::store_note(&app.state, &ann, &note(&reply), Context::Inbox)
            .await
            .expect("reply")
    else {
        panic!("not a post");
    };
    let (reply_to, root): (Option<Uuid>, Option<Uuid>) =
        sqlx::query_as("select reply_to_id, root_id from posts where id = $1")
            .bind(reply_id)
            .fetch_one(&db)
            .await
            .expect("links");
    assert_eq!(reply_to, Some(ada_post));
    assert_eq!(root, Some(ada_post));
    let replies: i32 = sqlx::query_scalar("select reply_count from posts where id = $1")
        .bind(ada_post)
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(replies, 1);
    let mentioned: Vec<Uuid> =
        sqlx::query_scalar("select user_id from mentions where post_id = $1")
            .bind(reply_id)
            .fetch_all(&db)
            .await
            .expect("mentions");
    assert!(mentioned.contains(&ada) && mentioned.contains(&bea));
    assert_eq!(mentioned.len(), 2, "only local accounts are stored");
    let kinds: Vec<(Uuid, String)> =
        sqlx::query_as("select user_id, kind from notifications where post_id = $1 order by kind")
            .bind(reply_id)
            .fetch_all(&db)
            .await
            .expect("notifications");
    assert!(kinds.contains(&(ada, "reply".to_owned())), "{kinds:?}");
    assert!(kinds.contains(&(bea, "mention".to_owned())), "{kinds:?}");
    assert_eq!(
        kinds.len(),
        2,
        "the replied-to author gets one notification, not two"
    );

    // A quote of a stored post links and counts, and notifies.
    let mut quote = note_json(base, "ann", 2);
    quote["quoteUrl"] = json!(ada_note);
    quote["tag"] = json!([{"type": "Mention", "href": local_actor("ada"), "name": "@ada"}]);
    let Stored::Post { id: quote_id, .. } =
        ingest::store_note(&app.state, &ann, &note(&quote), Context::Inbox)
            .await
            .expect("quote")
    else {
        panic!("not a post");
    };
    let (quote_of,): (Option<Uuid>,) =
        sqlx::query_as("select quote_of_id from posts where id = $1")
            .bind(quote_id)
            .fetch_one(&db)
            .await
            .expect("quote");
    assert_eq!(quote_of, Some(ada_post));
    let quotes: i32 = sqlx::query_scalar("select quote_count from posts where id = $1")
        .bind(ada_post)
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(quotes, 1);
    let quote_notice: i64 = sqlx::query_scalar(
        "select count(*) from notifications where post_id = $1 and kind = 'quote' and user_id = $2",
    )
    .bind(quote_id)
    .bind(ada)
    .fetch_one(&db)
    .await
    .expect("count");
    assert_eq!(quote_notice, 1);

    // Misskey's spelling, and a reply to a stored remote post.
    let mut misskey = note_json(base, "ann", 3);
    misskey["_misskey_quote"] = json!(format!("{base}/users/ann/statuses/2"));
    misskey["inReplyTo"] = json!(format!("{base}/users/ann/statuses/1"));
    let Stored::Post { id: third, .. } =
        ingest::store_note(&app.state, &ann, &note(&misskey), Context::Backfill)
            .await
            .expect("misskey")
    else {
        panic!("not a post");
    };
    let (quote_of, reply_to, root): (Option<Uuid>, Option<Uuid>, Option<Uuid>) =
        sqlx::query_as("select quote_of_id, reply_to_id, root_id from posts where id = $1")
            .bind(third)
            .fetch_one(&db)
            .await
            .expect("links");
    assert_eq!(quote_of, Some(quote_id));
    assert_eq!(reply_to, Some(reply_id));
    assert_eq!(root, Some(ada_post), "the thread root is the local post");
    let backfill_notices: i64 =
        sqlx::query_scalar("select count(*) from notifications where post_id = $1")
            .bind(third)
            .fetch_one(&db)
            .await
            .expect("count");
    assert_eq!(backfill_notices, 0, "a backfill notifies nobody");

    // A reply to a post that is not stored keeps no parent and stays usable.
    let mut orphan = note_json(base, "ann", 4);
    orphan["inReplyTo"] = json!("https://elsewhere.example/users/zed/statuses/77");
    let Stored::Post { id: orphan_id, .. } =
        ingest::store_note(&app.state, &ann, &note(&orphan), Context::Backfill)
            .await
            .expect("orphan")
    else {
        panic!("not a post");
    };
    let (reply_to, root): (Option<Uuid>, Option<Uuid>) =
        sqlx::query_as("select reply_to_id, root_id from posts where id = $1")
            .bind(orphan_id)
            .fetch_one(&db)
            .await
            .expect("links");
    assert_eq!((reply_to, root), (None, None));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_looked_up_reply_fetches_its_parent_once_and_no_further(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let base = fake.base();
    fake.add_actor("ann", Vec::new());
    fake.add_actor("bob", Vec::new());
    let mut grand = note_json(&base, "ann", 1);
    grand["content"] = json!("<p>grandparent</p>");
    let mut parent = note_json(&base, "bob", 2);
    parent["content"] = json!("<p>parent</p>");
    parent["inReplyTo"] = grand["id"].clone();
    let mut child = note_json(&base, "ann", 3);
    child["inReplyTo"] = parent["id"].clone();
    fake.put("/users/ann/statuses/1", grand);
    fake.put("/users/bob/statuses/2", parent);
    fake.put("/users/ann/statuses/3", child);

    let id = lookup::fetch_remote_post(&app.state, None, &format!("{base}/users/ann/statuses/3"))
        .await
        .expect("fetched");
    let (reply_to, text): (Option<Uuid>, String) =
        sqlx::query_as("select p.reply_to_id, p.text from posts p where p.id = $1")
            .bind(id)
            .fetch_one(&db)
            .await
            .expect("post");
    assert_eq!(text, "post number 3");
    let parent_id = reply_to.expect("the parent was fetched");
    let (parent_text, grand_link): (String, Option<Uuid>) =
        sqlx::query_as("select text, reply_to_id from posts where id = $1")
            .bind(parent_id)
            .fetch_one(&db)
            .await
            .expect("parent");
    assert_eq!(parent_text, "parent");
    assert_eq!(grand_link, None, "only one level is fetched");
    assert_eq!(fake.hits_of("/users/ann/statuses/1"), 0);
    assert_eq!(
        count(&db, "select count(*) from users where handle = $1", "bob").await,
        1,
        "the parent's author was resolved and stored"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_fetched_note_must_live_on_the_host_that_served_it(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let other = Fake::start().await;
    fake.add_actor("ann", Vec::new());
    let mut lying = note_json(&fake.base(), "ann", 1);
    lying["id"] = json!(format!("{}/users/ann/statuses/1", other.base()));
    fake.put("/users/ann/statuses/1", lying);
    let got = lookup::fetch_remote_post(
        &app.state,
        None,
        &format!("{}/users/ann/statuses/1", fake.base()),
    )
    .await;
    assert!(
        matches!(got, Err(FedError::Unreadable(_))),
        "{:?}",
        got.as_ref().err()
    );

    let mut foreign_author = note_json(&fake.base(), "ann", 2);
    foreign_author["attributedTo"] = json!(format!("{}/users/mallory", other.base()));
    fake.put("/users/ann/statuses/2", foreign_author);
    let got = lookup::fetch_remote_post(
        &app.state,
        None,
        &format!("{}/users/ann/statuses/2", fake.base()),
    )
    .await;
    assert!(got.is_err(), "{got:?}");
    assert_eq!(
        count(
            &db,
            "select count(*) from posts where ap_id is not null",
            ""
        )
        .await,
        0
    );

    // Our own URLs are not looked up remotely.
    let own = lookup::fetch_remote_post(
        &app.state,
        None,
        &format!("{ORIGIN}/users/ada/statuses/{}", Uuid::now_v7()),
    )
    .await;
    assert!(
        matches!(own, Err(FedError::NotFound)),
        "{:?}",
        own.as_ref().err()
    );
}

// ---- polls

#[sqlx::test(migrations = "./migrations")]
async fn a_question_becomes_a_poll_and_an_update_changes_its_counts(db: PgPool) {
    let app = app_with(db.clone(), open);
    let base = "https://remote.example";
    let ann = remote_user(&db, base, "ann").await;
    let mut value = note_json(base, "ann", 1);
    value["type"] = json!("Question");
    value["content"] = json!("<p>tea or coffee?</p>");
    value["oneOf"] = json!([
        {"type": "Note", "name": "tea", "replies": {"type": "Collection", "totalItems": 3}},
        {"type": "Note", "name": "coffee", "replies": {"type": "Collection", "totalItems": 5}}
    ]);
    value["endTime"] = json!("2999-01-01T00:00:00Z");
    value["votersCount"] = json!(8);
    let Stored::Post { id, .. } =
        ingest::store_note(&app.state, &ann, &note(&value), Context::Lookup)
            .await
            .expect("stored")
    else {
        panic!("not a post");
    };
    let poll = |id: Uuid| {
        let db = db.clone();
        async move {
            sqlx::query_as::<_, (Vec<String>, bool, Vec<i32>, i32, bool)>(
                "select options, multiple, counts, voters, ends_at > now() from polls where post_id = $1",
            )
            .bind(id)
            .fetch_one(&db)
            .await
            .expect("poll")
        }
    };
    assert_eq!(
        poll(id).await,
        (
            vec!["tea".to_owned(), "coffee".to_owned()],
            false,
            vec![3, 5],
            8,
            true
        )
    );
    value["oneOf"][0]["replies"]["totalItems"] = json!(10);
    value["votersCount"] = json!(15);
    value["updated"] = json!("2026-02-01T00:00:00Z");
    ingest::apply_note_update(&app.state, &ann, &note(&value))
        .await
        .expect("updated");
    assert_eq!(poll(id).await.2, vec![10, 5]);
    assert_eq!(poll(id).await.3, 15);

    // A multiple-choice poll and a poll that ended.
    let mut multi = note_json(base, "ann", 2);
    multi["type"] = json!("Question");
    multi["anyOf"] = json!([{"name": "a"}, {"name": "b"}, {"name": "c"}]);
    multi["closed"] = json!("2026-01-05T00:00:00Z");
    let Stored::Post { id: multi_id, .. } =
        ingest::store_note(&app.state, &ann, &note(&multi), Context::Lookup)
            .await
            .expect("stored")
    else {
        panic!("not a post");
    };
    let multi_poll = poll(multi_id).await;
    assert!(multi_poll.1, "anyOf is a multiple-choice poll");
    assert!(!multi_poll.4, "a closed poll has ended");
}

// ---- attachments

#[sqlx::test(migrations = "./migrations")]
async fn attachments_are_copied_reencoded_and_capped_at_four(db: PgPool) {
    let data = std::env::temp_dir().join(format!("tuita-lookup-{}", Uuid::now_v7()));
    let data_dir = data.display().to_string();
    let app = app_with(db.clone(), |var| match var {
        "DATA_DIR" => Some(data_dir.clone()),
        other => open(other),
    });
    let fake = Fake::start().await;
    let base = fake.base();
    fake.add_actor("ann", Vec::new());
    for n in 1..=6 {
        fake.put_raw(&format!("/media/{n}.png"), "image/png", png(40 + n, 30));
    }
    fake.put_raw(
        "/media/notimage.png",
        "image/png",
        b"not an image at all".to_vec(),
    );
    let ann = remote_user(&db, &base, "ann").await;
    let mut value = note_json(&base, "ann", 1);
    let mut docs: Vec<Value> = (1..=6)
        .map(|n| {
            json!({"type": "Document", "mediaType": "image/png",
                   "url": format!("{base}/media/{n}.png"), "name": format!("alt {n}")})
        })
        .collect();
    docs.insert(
        1,
        json!({"type": "Document", "mediaType": "image/png", "url": format!("{base}/media/missing.png")}),
    );
    docs.insert(
        2,
        json!({"type": "Document", "mediaType": "image/png", "url": format!("{base}/media/notimage.png")}),
    );
    docs.insert(
        3,
        json!({"type": "Document", "mediaType": "image/png", "url": "javascript:alert(1)"}),
    );
    value["attachment"] = json!(docs);
    let Stored::Post { id, .. } =
        ingest::store_note(&app.state, &ann, &note(&value), Context::Lookup)
            .await
            .expect("stored")
    else {
        panic!("not a post");
    };
    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        "select m.kind, m.mime, m.path, m.alt from post_media pm join media m on m.id = pm.media_id
          where pm.post_id = $1 order by pm.position",
    )
    .bind(id)
    .fetch_all(&db)
    .await
    .expect("media");
    // The first four attachments are tried; the three bad ones are skipped, not fatal.
    assert!(!rows.is_empty() && rows.len() <= 4, "{rows:?}");
    for (kind, mime, path, _) in &rows {
        assert_eq!(kind, "image");
        assert_eq!(mime, "image/webp", "PNG is re-encoded, not stored as sent");
        assert!(path.ends_with(".webp"));
        assert!(data.join("media").join(path).exists());
    }
    assert_eq!(rows[0].3, "alt 1");

    // Four good attachments out of six: exactly four are kept.
    let mut value = note_json(&base, "ann", 2);
    value["attachment"] = json!(
        (1..=6)
            .map(|n| json!({"type": "Document", "mediaType": "image/png",
                            "url": format!("{base}/media/{n}.png"), "name": format!("alt {n}")}))
            .collect::<Vec<_>>()
    );
    let Stored::Post { id, .. } =
        ingest::store_note(&app.state, &ann, &note(&value), Context::Lookup)
            .await
            .expect("stored")
    else {
        panic!("not a post");
    };
    let kept: i64 = sqlx::query_scalar("select count(*) from post_media where post_id = $1")
        .bind(id)
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(kept, 4);
    for n in 5..=6 {
        assert_eq!(
            fake.hits_of(&format!("/media/{n}.png")),
            0,
            "attachment {n} is never fetched"
        );
    }
    let owners: Vec<Uuid> = sqlx::query_scalar(
        "select m.owner_id from post_media pm join media m on m.id = pm.media_id where pm.post_id = $1",
    )
    .bind(id)
    .fetch_all(&db)
    .await
    .expect("owners");
    assert!(owners.iter().all(|owner| *owner == ann.id));
    let _ = std::fs::remove_dir_all(&data);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_post_with_only_a_failing_attachment_and_no_text_is_dropped(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let base = fake.base();
    let ann = remote_user(&db, &base, "ann").await;
    let mut value = note_json(&base, "ann", 1);
    value["content"] = json!("");
    value["attachment"] =
        json!([{"type": "Document", "mediaType": "image/png", "url": format!("{base}/nope.png")}]);
    let got = ingest::store_note(&app.state, &ann, &note(&value), Context::Lookup)
        .await
        .expect("handled");
    assert_eq!(got, Stored::Dropped);
}

// ---- direct messages

#[sqlx::test(migrations = "./migrations")]
async fn a_direct_note_to_one_local_account_becomes_a_message(db: PgPool) {
    let app = app_with(db.clone(), open);
    let base = "https://remote.example";
    let ann = remote_user(&db, base, "ann").await;
    let mut ada = app.person(&db, "ada").await;
    insert_user(&db, "bea").await;
    let dm = |n: u32, to: Vec<String>| {
        let mut value = note_json(base, "ann", n);
        value["to"] = json!(to);
        value["cc"] = json!([]);
        value["content"] = json!(format!("<p>secret {n}</p>"));
        value
    };

    // One local recipient: a message.
    let first = dm(1, vec![local_actor("ada")]);
    let stored = ingest::store_note(&app.state, &ann, &note(&first), Context::Inbox)
        .await
        .expect("stored");
    assert_eq!(stored, Stored::Message { created: true });
    let again = ingest::store_note(&app.state, &ann, &note(&first), Context::Inbox)
        .await
        .expect("replay");
    assert_eq!(again, Stored::Message { created: false });
    assert_eq!(
        count(
            &db,
            "select count(*) from messages where ap_id = $1",
            &note(&first).id
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &db,
            "select count(*) from posts where ap_id = $1",
            &note(&first).id
        )
        .await,
        0
    );
    let thread = ada
        .browser
        .get(&format!("/conversations/ann@{}", authority(base)))
        .await
        .ok();
    assert_eq!(thread["messages"][0]["text"], "secret 1");
    assert_eq!(thread["messages"][0]["mine"], false);

    // Two local recipients: a direct post, hidden from the timelines.
    let two = dm(2, vec![local_actor("ada"), local_actor("bea")]);
    let Stored::Post { id: direct, .. } =
        ingest::store_note(&app.state, &ann, &note(&two), Context::Inbox)
            .await
            .expect("stored")
    else {
        panic!("not a post");
    };
    let visibility: String = sqlx::query_scalar("select visibility from posts where id = $1")
        .bind(direct)
        .fetch_one(&db)
        .await
        .expect("visibility");
    assert_eq!(visibility, "direct");
    let mentioned: i64 = sqlx::query_scalar("select count(*) from mentions where post_id = $1")
        .bind(direct)
        .fetch_one(&db)
        .await
        .expect("mentions");
    assert_eq!(mentioned, 2, "the addressed accounts can see it");

    // A local and a remote recipient: a direct post, not a message.
    let mixed = dm(
        3,
        vec![
            local_actor("ada"),
            "https://other.example/users/zed".to_owned(),
        ],
    );
    assert!(matches!(
        ingest::store_note(&app.state, &ann, &note(&mixed), Context::Inbox).await,
        Ok(Stored::Post { .. })
    ));
    // A reply to a stored post is a post too, not a message.
    let mut reply = dm(4, vec![local_actor("ada")]);
    reply["inReplyTo"] = json!(note(&mixed).id);
    assert!(matches!(
        ingest::store_note(&app.state, &ann, &note(&reply), Context::Inbox).await,
        Ok(Stored::Post { .. })
    ));
}

#[sqlx::test(migrations = "./migrations")]
async fn direct_messages_respect_policy_blocks_and_suspension(db: PgPool) {
    let app = app_with(db.clone(), open);
    let base = "https://remote.example";
    let ann = remote_user(&db, base, "ann").await;
    let ada = insert_user(&db, "ada").await;
    let dm = |n: u32| {
        let mut value = note_json(base, "ann", n);
        value["to"] = json!([local_actor("ada")]);
        value["cc"] = json!([]);
        note(&value)
    };
    let send = |n: u32| {
        let state = app.state.clone();
        let ann = ann.clone();
        let note = dm(n);
        async move {
            ingest::store_note(&state, &ann, &note, Context::Inbox)
                .await
                .expect("handled")
        }
    };

    sqlx::query("update users set dm_policy = 'none' where id = $1")
        .bind(ada)
        .execute(&db)
        .await
        .expect("policy");
    assert_eq!(send(1).await, Stored::Dropped, "dm_policy none");
    sqlx::query("update users set dm_policy = 'following' where id = $1")
        .bind(ada)
        .execute(&db)
        .await
        .expect("policy");
    assert_eq!(send(2).await, Stored::Dropped, "ada does not follow ann");
    sqlx::query("insert into follows (follower_id, followee_id) values ($1, $2)")
        .bind(ada)
        .bind(ann.id)
        .execute(&db)
        .await
        .expect("follow");
    assert_eq!(
        send(3).await,
        Stored::Message { created: true },
        "ada follows ann"
    );
    sqlx::query("insert into blocks (user_id, target_id) values ($1, $2)")
        .bind(ada)
        .bind(ann.id)
        .execute(&db)
        .await
        .expect("block");
    assert_eq!(send(4).await, Stored::Dropped, "blocked");
    sqlx::query("delete from blocks")
        .execute(&db)
        .await
        .expect("unblock");
    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(ada)
        .execute(&db)
        .await
        .expect("suspend");
    assert_eq!(send(5).await, Stored::Dropped, "suspended recipient");
    assert_eq!(
        count(&db, "select count(*) from messages where ap_id <> $1", "").await,
        1
    );
}

// ---- profile updates and deletion

#[sqlx::test(migrations = "./migrations")]
async fn an_actor_update_changes_the_profile_and_a_delete_hides_the_account(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let base = fake.base();
    fake.add_actor("ann", outbox_of(&base, "ann", 2));
    let ann = lookup::resolve_acct(&app.state, None, "ann", &fake.domain())
        .await
        .expect("resolved");
    assert_eq!(ann.posts_count, 2);

    let mut update: ap::Actor = serde_json::from_value(actor_json(&base, "ann")).expect("actor");
    update.name = Some("Ann, updated".to_owned());
    update.summary = Some("<p>new bio</p>".to_owned());
    update.manually_approves_followers = false;
    let after = ingest::apply_actor_update(&app.state, &ann, update.clone())
        .await
        .expect("updated");
    assert_eq!(after.display_name, "Ann, updated");
    assert_eq!(after.bio, "new bio");
    assert_eq!(after.manually_approves, Some(false));
    assert_eq!(after.id, ann.id);

    // A different actor cannot be applied to this account, and the key must be its own.
    let mut other = update.clone();
    other.id = format!("{base}/users/bob");
    let got = ingest::apply_actor_update(&app.state, &ann, other).await;
    assert!(
        matches!(got, Err(FedError::Unreadable(_))),
        "{:?}",
        got.as_ref().err()
    );
    let mut stolen = update;
    stolen.public_key.as_mut().expect("key").owner = format!("{base}/users/bob");
    let got = ingest::apply_actor_update(&app.state, &ann, stolen).await;
    assert!(
        matches!(got, Err(FedError::Unreadable(_))),
        "{:?}",
        got.as_ref().err()
    );

    let ada = insert_user(&db, "ada").await;
    sqlx::query("insert into follows (follower_id, followee_id) values ($1, $2)")
        .bind(ada)
        .bind(ann.id)
        .execute(&db)
        .await
        .expect("follow");
    ingest::forget_actor(&app.state, &after)
        .await
        .expect("forgotten");
    let (deleted, live_posts): (bool, i64) = sqlx::query_as(
        "select u.deleted_at is not null, (select count(*) from posts where author_id = u.id and deleted_at is null)
           from users u where u.id = $1",
    )
    .bind(ann.id)
    .fetch_one(&db)
    .await
    .expect("row");
    assert!(deleted);
    assert_eq!(live_posts, 0, "their posts are hidden");
    assert_eq!(
        count(
            &db,
            "select count(*) from follows where followee_id::text <> $1",
            ""
        )
        .await,
        0
    );
    let gone = lookup::resolve_acct(&app.state, None, "ann", &fake.domain()).await;
    assert!(
        matches!(gone, Err(FedError::NotFound)),
        "{:?}",
        gone.as_ref().err()
    );
    // A refetch does not bring a deleted account back.
    let by_url = lookup::actor_by_url(&app.state, &format!("{base}/users/ann")).await;
    assert!(
        matches!(by_url, Err(FedError::NotFound)),
        "{:?}",
        by_url.as_ref().err()
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn an_instance_silence_keeps_posts_out_of_public_places_only(db: PgPool) {
    let app: App = app_with(db.clone(), open);
    let base = "https://remote.example";
    let ann = remote_user(&db, base, "ann").await;
    let Stored::Post { id, .. } = ingest::store_note(
        &app.state,
        &ann,
        &note(&note_json(base, "ann", 1)),
        Context::Lookup,
    )
    .await
    .expect("stored") else {
        panic!("not a post");
    };
    sqlx::query(
        "insert into domain_blocks (domain, severity) values ('remote.example', 'silence')",
    )
    .execute(&db)
    .await
    .expect("silence");
    let mut me = app.person(&db, "viewer").await.browser;
    let found = me.get("/search?q=post%20number&type=posts").await.ok();
    assert_eq!(
        found["posts"],
        json!([]),
        "search leaves silenced posts out"
    );
    let thread = me.get(&format!("/posts/{id}")).await;
    assert_eq!(thread.status, StatusCode::OK, "{}", thread.body);
}

#[sqlx::test(migrations = "./migrations")]
async fn an_outbox_whose_first_page_is_a_link_is_followed_on_the_same_host_only(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let other = Fake::start().await;
    let base = fake.base();
    fake.add_actor("cy", Vec::new());
    fake.put(
        "/users/cy/outbox",
        json!({"id": format!("{base}/users/cy/outbox"), "type": "OrderedCollection", "totalItems": 3,
               "first": format!("{base}/users/cy/outbox?page=true")}),
    );
    fake.put(
        "/users/cy/outbox?page=true",
        json!({"id": format!("{base}/users/cy/outbox?page=true"), "type": "OrderedCollectionPage",
               "orderedItems": outbox_of(&base, "cy", 3)}),
    );
    let row = lookup::resolve_acct(&app.state, None, "cy", &fake.domain())
        .await
        .expect("resolved");
    assert_eq!(row.posts_count, 3);

    // A `first` link to another host is never fetched.
    fake.add_actor("dee", Vec::new());
    fake.put(
        "/users/dee/outbox",
        json!({"id": format!("{base}/users/dee/outbox"), "type": "OrderedCollection",
               "first": format!("{}/users/dee/outbox?page=true", other.base())}),
    );
    let dee = lookup::resolve_acct(&app.state, None, "dee", &fake.domain())
        .await
        .expect("resolved");
    assert_eq!(dee.posts_count, 0);
    assert!(other.hits().is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn following_or_messaging_a_suspended_instance_is_forbidden(db: PgPool) {
    let app = app_with(db.clone(), open);
    let mut me = app.person(&db, "viewer").await.browser;
    sqlx::query("insert into domain_blocks (domain, severity) values ('bad.example', 'suspend')")
        .execute(&db)
        .await
        .expect("block");
    me.post_empty("/users/ann@bad.example/follow")
        .await
        .err(StatusCode::FORBIDDEN, "You can’t follow this account.");
    me.post("/conversations/ann@bad.example", json!({"text": "hi"}))
        .await
        .err(StatusCode::FORBIDDEN, "You can’t message this account.");
    me.get("/users/ann@bad.example")
        .await
        .err(StatusCode::NOT_FOUND, "User not found.");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_post_cannot_claim_another_instances_page_address(db: PgPool) {
    let app = app_with(db.clone(), open);
    let good = Fake::start().await;
    let base = good.base();
    good.add_actor("bob", Vec::new());
    let mut real = note_json(&base, "bob", 1);
    real["content"] = json!("<p>the real post</p>");
    good.put("/@bob/1", real.clone());
    good.put("/users/bob/statuses/1", real);

    let evil = remote_user(&db, "https://evil.example", "mallory").await;
    let mut fake = note_json("https://evil.example", "mallory", 1);
    fake["content"] = json!("<p>spoofed</p>");
    fake["url"] = json!(format!("{base}/@bob/1"));
    ingest::store_note(&app.state, &evil, &note(&fake), Context::Lookup)
        .await
        .expect("stored");

    let id = lookup::fetch_remote_post(&app.state, None, &format!("{base}/@bob/1"))
        .await
        .expect("fetched");
    let (text, author): (String, String) = sqlx::query_as(
        "select p.text, u.handle from posts p join users u on u.id = p.author_id where p.id = $1",
    )
    .bind(id)
    .fetch_one(&db)
    .await
    .expect("post");
    assert_eq!((text.as_str(), author.as_str()), ("the real post", "bob"));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_refresh_copies_an_image_only_when_its_url_changed(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    fake.add_actor("ann", Vec::new());
    let row = lookup::resolve_acct(&app.state, None, "ann", &fake.domain())
        .await
        .expect("resolved");
    assert_eq!(fake.hits_of("/avatar.png"), 1);
    assert_eq!(fake.hits_of("/banner.png"), 1);

    lookup::refresh_actor(&app.state, &row)
        .await
        .expect("refreshed");
    assert_eq!(fake.hits_of("/avatar.png"), 1, "same URL, no second copy");
    assert_eq!(fake.hits_of("/banner.png"), 1);

    let mut changed = actor_json(&fake.base(), "ann");
    changed["icon"]["url"] = json!(format!("{}/avatar2.png", fake.base()));
    fake.put("/users/ann", changed);
    fake.put_raw("/avatar2.png", "image/png", png(32, 32));
    lookup::refresh_actor(&app.state, &row)
        .await
        .expect("refreshed");
    assert_eq!(fake.hits_of("/avatar2.png"), 1, "a new URL is copied");
    assert_eq!(fake.hits_of("/avatar.png"), 1);
    assert_eq!(fake.hits_of("/banner.png"), 1);
}

/// An image server that counts how many requests it serves at the same time.
struct Gauge {
    now: std::sync::atomic::AtomicUsize,
    peak: std::sync::atomic::AtomicUsize,
}

async fn slow_image(State(gauge): State<Arc<Gauge>>) -> Response {
    use std::sync::atomic::Ordering::SeqCst;
    let now = gauge.now.fetch_add(1, SeqCst) + 1;
    gauge.peak.fetch_max(now, SeqCst);
    tokio::time::sleep(Duration::from_millis(150)).await;
    gauge.now.fetch_sub(1, SeqCst);
    ([(CONTENT_TYPE, "image/png")], png(32, 32)).into_response()
}

#[sqlx::test(migrations = "./migrations")]
async fn at_most_four_remote_images_are_copied_at_once(db: PgPool) {
    use std::sync::atomic::Ordering::SeqCst;
    let app = app_with(db.clone(), open);
    let gauge = Arc::new(Gauge {
        now: 0.into(),
        peak: 0.into(),
    });
    let router = Router::new().fallback(slow_image).with_state(gauge.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let base = format!("http://{}", listener.local_addr().expect("addr"));
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });

    let mut stores = Vec::new();
    for n in 0..6 {
        let name = format!("user{n}");
        let mut doc = actor_json(&base, &name);
        doc["icon"]["url"] = json!(format!("{base}/{name}-avatar.png"));
        doc["image"]["url"] = json!(format!("{base}/{name}-banner.png"));
        let actor: ap::Actor = serde_json::from_value(doc).expect("actor");
        let actor = lookup::check_actor(&app.state, actor, &format!("{base}/users/{name}"), None)
            .expect("checked");
        let state = app.state.clone();
        stores.push(tokio::spawn(async move {
            lookup::store_actor(&state, &actor).await
        }));
    }
    for store in stores {
        let row = store.await.expect("task").expect("stored").expect("row");
        assert!(row.avatar_id.is_some() && row.banner_id.is_some());
    }
    let peak = gauge.peak.load(SeqCst);
    assert!(
        (2..=4).contains(&peak),
        "{peak} images were fetched at the same time"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn another_actor_id_cannot_take_over_a_stored_handle(db: PgPool) {
    let base = "https://remote.example";
    let bob = remote_user(&db, base, "bob").await;
    let rival = RemoteProfile {
        handle: "Bob".to_owned(),
        domain: authority(base),
        display_name: "not bob".to_owned(),
        bio: String::new(),
        website: String::new(),
        is_bot: false,
        is_org: false,
        locked: false,
        discoverable: true,
        inbox_url: format!("{base}/other/bob/inbox"),
        shared_inbox_url: None,
        outbox_url: None,
        followers_url: None,
        featured_url: None,
        url: None,
        public_key_pem: "not bob's key".to_owned(),
        created_at: None,
        ap_id: format!("{base}/other/bob"),
    };
    let got = remote::upsert_actor(&db, &rival).await.expect("upsert");
    assert_eq!(got, Upsert::Conflict);
    let kept = users::by_id(&db, bob.id).await.expect("load").expect("row");
    assert_eq!(kept.ap_id, bob.ap_id);
    assert_eq!(kept.display_name, "bob");
    assert_eq!(
        remote::public_key_pem(&db, bob.id)
            .await
            .expect("pem")
            .as_deref(),
        Some(pem())
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_block_covers_subdomains_and_the_trailing_dot_form_of_a_domain(db: PgPool) {
    let app = app_with(db.clone(), open);
    let mut me = app.person(&db, "viewer").await.browser;
    sqlx::query("insert into domain_blocks (domain, severity) values ('evil.invalid', 'suspend')")
        .execute(&db)
        .await
        .expect("block");

    for domain in [
        "evil.invalid.",
        "sub.evil.invalid",
        "sub.evil.invalid.",
        "EVIL.invalid",
    ] {
        let got = lookup::resolve_acct(&app.state, None, "ann", domain).await;
        assert!(
            matches!(got, Err(FedError::NotFound)),
            "{domain}: {:?}",
            got.as_ref().err()
        );
        let url = format!("https://{domain}/users/ann");
        let by_url = lookup::actor_by_url(&app.state, &url).await;
        assert!(
            matches!(by_url, Err(FedError::NotFound)),
            "{url}: {:?}",
            by_url.as_ref().err()
        );
        let post = lookup::fetch_remote_post(&app.state, None, &format!("{url}/statuses/1")).await;
        assert!(
            matches!(post, Err(FedError::NotFound)),
            "{url}: {:?}",
            post.as_ref().err()
        );
        me.post_empty(&format!("/users/ann@{domain}/follow"))
            .await
            .err(StatusCode::FORBIDDEN, "You can’t follow this account.");
    }
    let other = lookup::resolve_acct(&app.state, None, "ann", "notevil.invalid").await;
    assert!(
        !matches!(other, Err(FedError::NotFound)),
        "a longer name is not a subdomain: {:?}",
        other.as_ref().err()
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_block_on_a_domain_hides_the_posts_of_its_subdomains(db: PgPool) {
    let app: App = app_with(db.clone(), open);
    let base = "https://sub.remote.example";
    let ann = remote_user(&db, base, "ann").await;
    let Stored::Post { id, .. } = ingest::store_note(
        &app.state,
        &ann,
        &note(&note_json(base, "ann", 1)),
        Context::Lookup,
    )
    .await
    .expect("stored") else {
        panic!("not a post");
    };
    let mut me = app.person(&db, "viewer").await.browser;
    async fn search(me: &mut Browser) -> Value {
        me.get("/search?q=post%20number&type=posts").await.ok()["posts"].clone()
    }
    assert_ne!(search(&mut me).await, json!([]), "visible before any block");

    sqlx::query(
        "insert into domain_blocks (domain, severity) values ('remote.example', 'silence')",
    )
    .execute(&db)
    .await
    .expect("silence");
    assert_eq!(search(&mut me).await, json!([]), "silenced by the parent");
    assert_eq!(me.get(&format!("/posts/{id}")).await.status, StatusCode::OK);

    sqlx::query("update domain_blocks set severity = 'suspend'")
        .execute(&db)
        .await
        .expect("suspend");
    me.get(&format!("/posts/{id}"))
        .await
        .err(StatusCode::NOT_FOUND, "This post is unavailable.");
}
