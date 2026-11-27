//! The delivery worker against a fake inbox: signatures, outcomes of each status, backoff,
//! the attempt cap, blocked domains and concurrent passes.
#![allow(missing_docs)]

mod common;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use serde_json::{Value, json};
use sqlx::PgPool;
use time::OffsetDateTime;
use tokio::net::TcpListener;
use tuita_core::ap::{self, PublicKey, RequestParts};
use tuita_server::db::deliveries;
use tuita_server::federation::keys;
use tuita_server::workers::delivery::{self, MAX_ERROR_CHARS};
use uuid::Uuid;

use common::{App, ORIGIN, app, app_with, insert_user};

struct Received {
    target: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

type Seen = Arc<Mutex<Vec<Received>>>;

/// `POST /inbox/{status}` answers with that status and a long body; `/slow` waits first.
async fn inbox(
    State(seen): State<Seen>,
    Path(status): Path<u16>,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    seen.lock().expect("lock").push(Received {
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
    (
        StatusCode::from_u16(status).expect("status"),
        "x".repeat(1000),
    )
        .into_response()
}

async fn slow(State(seen): State<Seen>, uri: Uri, body: Bytes) -> Response {
    seen.lock().expect("lock").push(Received {
        target: uri.to_string(),
        headers: Vec::new(),
        body: body.to_vec(),
    });
    tokio::time::sleep(Duration::from_millis(300)).await;
    StatusCode::ACCEPTED.into_response()
}

struct Fake {
    addr: SocketAddr,
    seen: Seen,
}

impl Fake {
    async fn start() -> Self {
        let seen: Seen = Arc::default();
        let router = Router::new()
            .route("/inbox/{status}", post(inbox))
            .route("/slow", post(slow))
            .with_state(seen.clone());
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            let _ = axum::serve(listener, router).await;
        });
        Self { addr, seen }
    }

    fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.addr)
    }

    fn count(&self) -> usize {
        self.seen.lock().expect("lock").len()
    }
}

fn open(var: &str) -> Option<String> {
    matches!(
        var,
        "FEDERATION_ALLOW_HTTP" | "FEDERATION_ALLOW_PRIVATE_NETS"
    )
    .then(|| "1".to_owned())
}

fn activity(actor: &str) -> Value {
    json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": format!("{ORIGIN}/activities/{}", Uuid::now_v7()),
        "type": "Like",
        "actor": actor,
        "object": "https://remote.example/notes/1",
    })
}

async fn add(app: &App, signer: Option<Uuid>, url: &str, actor: &str) -> Uuid {
    deliveries::enqueue(app.state.db(), signer, url, &activity(actor))
        .await
        .expect("enqueue");
    sqlx::query_scalar("select id from ap_deliveries where inbox_url = $1 order by id desc limit 1")
        .bind(url)
        .fetch_one(app.state.db())
        .await
        .expect("row id")
}

async fn rows(db: &PgPool) -> i64 {
    sqlx::query_scalar("select count(*) from ap_deliveries")
        .fetch_one(db)
        .await
        .expect("count")
}

async fn set_attempts(db: &PgPool, id: Uuid, attempts: i32) {
    sqlx::query("update ap_deliveries set attempts = $2 where id = $1")
        .bind(id)
        .bind(attempts)
        .execute(db)
        .await
        .expect("attempts");
}

/// `(attempts, seconds until next attempt, last_error)` of a row.
async fn state_of(db: &PgPool, id: Uuid) -> (i32, f64, Option<String>) {
    sqlx::query_as(
        "select attempts, extract(epoch from next_attempt_at - now())::float8, last_error
           from ap_deliveries where id = $1",
    )
    .bind(id)
    .fetch_one(db)
    .await
    .expect("row")
}

fn verify(received: &Received, key_pem: &str, key_id: &str) -> Value {
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
        .expect("date, digest and covered headers pass");
    let key = PublicKey::from_pem(key_pem).expect("pem");
    ap::verify_signature(&parts, &parsed, &key).expect("signature verifies");
    assert_eq!(
        parts.header("digest").expect("digest"),
        ap::digest_header(&received.body)
    );
    assert_eq!(
        parts.header("content-type").as_deref(),
        Some("application/activity+json")
    );
    serde_json::from_slice(&received.body).expect("json body")
}

#[sqlx::test(migrations = "./migrations")]
async fn a_202_deletes_the_row_and_the_request_is_signed_by_the_account(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let ana = insert_user(&db, "ana").await;
    let actor = format!("{ORIGIN}/users/ana");
    let id = add(&app, Some(ana), &fake.url("/inbox/202?x=1"), &actor).await;
    let sent: Value = sqlx::query_scalar("select activity from ap_deliveries where id = $1")
        .bind(id)
        .fetch_one(&db)
        .await
        .expect("activity");

    assert_eq!(delivery::run_once(&app.state).await, 1);
    assert_eq!(rows(&db).await, 0);
    assert_eq!(fake.count(), 1);
    let pem = keys::user_public_pem(&app.state, ana).await.expect("pem");
    let received = fake.seen.lock().expect("lock").remove(0);
    let body = verify(&received, &pem, &format!("{actor}#main-key"));
    assert_eq!(body, sent);
    assert_eq!(received.target, "/inbox/202?x=1");

    assert_eq!(delivery::run_once(&app.state).await, 0, "nothing is left");
}

#[sqlx::test(migrations = "./migrations")]
async fn an_instance_delivery_is_signed_by_the_instance_actor(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    add(
        &app,
        None,
        &fake.url("/inbox/200"),
        &format!("{ORIGIN}/actor"),
    )
    .await;
    assert_eq!(delivery::run_once(&app.state).await, 1);
    let pem = keys::instance_public_pem(&app.state).await.expect("pem");
    let received = fake.seen.lock().expect("lock").remove(0);
    verify(&received, &pem, &format!("{ORIGIN}/actor#main-key"));
}

#[sqlx::test(migrations = "./migrations")]
async fn the_key_id_follows_the_actor_of_the_activity_not_the_current_handle(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let ana = insert_user(&db, "deleted_abc").await;
    let old = format!("{ORIGIN}/users/ana");
    add(&app, Some(ana), &fake.url("/inbox/202"), &old).await;
    assert_eq!(delivery::run_once(&app.state).await, 1);
    let pem = keys::user_public_pem(&app.state, ana).await.expect("pem");
    let received = fake.seen.lock().expect("lock").remove(0);
    verify(&received, &pem, &format!("{old}#main-key"));
}

#[sqlx::test(migrations = "./migrations")]
async fn client_errors_other_than_429_drop_the_row(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let ana = insert_user(&db, "ana").await;
    let actor = format!("{ORIGIN}/users/ana");
    for status in [400, 404, 410] {
        add(
            &app,
            Some(ana),
            &fake.url(&format!("/inbox/{status}")),
            &actor,
        )
        .await;
    }
    assert_eq!(delivery::run_once(&app.state).await, 3);
    assert_eq!(rows(&db).await, 0);
    assert_eq!(fake.count(), 3);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_429_and_a_500_back_off_and_keep_a_truncated_error(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let ana = insert_user(&db, "ana").await;
    let actor = format!("{ORIGIN}/users/ana");
    let limited = add(&app, Some(ana), &fake.url("/inbox/429"), &actor).await;
    let broken = add(&app, Some(ana), &fake.url("/inbox/500"), &actor).await;
    set_attempts(&db, broken, 2).await;

    assert_eq!(delivery::run_once(&app.state).await, 2);
    let (attempts, wait, error) = state_of(&db, limited).await;
    assert_eq!(attempts, 1);
    assert!(
        (28.0..=31.0).contains(&wait),
        "first retry after 2^1 x 15 s, got {wait}"
    );
    let error = error.expect("error stored");
    assert!(error.starts_with("HTTP 429: xxx"), "{error}");
    assert_eq!(error.chars().count(), MAX_ERROR_CHARS);

    let (attempts, wait, error) = state_of(&db, broken).await;
    assert_eq!(attempts, 3);
    assert!(
        (118.0..=121.0).contains(&wait),
        "third attempt waits 2^3 x 15 s, got {wait}"
    );
    assert!(error.expect("error").starts_with("HTTP 500: "));

    assert_eq!(
        delivery::run_once(&app.state).await,
        0,
        "rows that are not due yet stay where they are"
    );
    assert_eq!(fake.count(), 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_backoff_is_capped_at_six_hours(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let ana = insert_user(&db, "ana").await;
    let id = add(
        &app,
        Some(ana),
        &fake.url("/inbox/503"),
        &format!("{ORIGIN}/users/ana"),
    )
    .await;
    set_attempts(&db, id, 10).await;
    assert_eq!(delivery::run_once(&app.state).await, 1);
    let (attempts, wait, _) = state_of(&db, id).await;
    assert_eq!(attempts, 11);
    assert!(
        (21_590.0..=21_601.0).contains(&wait),
        "2^11 x 15 s is over 6 h, so it is 6 h: {wait}"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn the_twelfth_failure_deletes_the_row(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let ana = insert_user(&db, "ana").await;
    let actor = format!("{ORIGIN}/users/ana");
    let last = add(&app, Some(ana), &fake.url("/inbox/500"), &actor).await;
    let before = add(&app, Some(ana), &fake.url("/inbox/502"), &actor).await;
    set_attempts(&db, last, 11).await;
    set_attempts(&db, before, 10).await;

    assert_eq!(delivery::run_once(&app.state).await, 2);
    assert_eq!(rows(&db).await, 1);
    let (attempts, _, _) = state_of(&db, before).await;
    assert_eq!(attempts, 11, "the eleventh failure still retries");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_row_for_a_suspended_domain_is_deleted_unsent(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let ana = insert_user(&db, "ana").await;
    let actor = format!("{ORIGIN}/users/ana");
    add(&app, Some(ana), &fake.url("/inbox/202"), &actor).await;
    sqlx::query("insert into domain_blocks (domain, severity) values ($1, 'suspend')")
        .bind(fake.addr.to_string())
        .execute(&db)
        .await
        .expect("block");
    assert_eq!(delivery::run_once(&app.state).await, 0);
    assert_eq!(rows(&db).await, 0);
    assert_eq!(fake.count(), 0);

    sqlx::query("update domain_blocks set severity = 'silence'")
        .execute(&db)
        .await
        .expect("silence");
    add(&app, Some(ana), &fake.url("/inbox/202"), &actor).await;
    assert_eq!(delivery::run_once(&app.state).await, 1);
    assert_eq!(fake.count(), 1, "a silence block does not stop deliveries");
}

#[sqlx::test(migrations = "./migrations")]
async fn two_concurrent_passes_never_send_a_row_twice(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let ana = insert_user(&db, "ana").await;
    let actor = format!("{ORIGIN}/users/ana");
    for _ in 0..80 {
        add(&app, Some(ana), &fake.url("/slow"), &actor).await;
    }
    let (a, b) = tokio::join!(
        delivery::run_once(&app.state),
        delivery::run_once(&app.state)
    );
    assert_eq!(a + b, 80, "every row was claimed by exactly one pass");
    assert!(a <= 50 && b <= 50, "a pass takes at most a batch: {a}, {b}");
    assert_eq!(fake.count(), 80, "every row was sent exactly once");
    assert_eq!(rows(&db).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_batch_is_capped_at_fifty_rows(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let ana = insert_user(&db, "ana").await;
    let actor = format!("{ORIGIN}/users/ana");
    for _ in 0..55 {
        add(&app, Some(ana), &fake.url("/inbox/202"), &actor).await;
    }
    assert_eq!(delivery::run_once(&app.state).await, 50);
    assert_eq!(rows(&db).await, 5);
    assert_eq!(delivery::run_once(&app.state).await, 5);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_claimed_row_is_leased_while_it_is_in_flight(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let ana = insert_user(&db, "ana").await;
    let id = add(
        &app,
        Some(ana),
        &fake.url("/slow"),
        &format!("{ORIGIN}/users/ana"),
    )
    .await;
    let pass = tokio::spawn({
        let state = app.state.clone();
        async move { delivery::run_once(&state).await }
    });
    tokio::time::sleep(Duration::from_millis(100)).await;
    let (attempts, wait, _) = state_of(&db, id).await;
    assert_eq!(attempts, 0);
    assert!(
        (55.0..=61.0).contains(&wait),
        "claimed rows sit out a 60 s lease, got {wait}"
    );
    assert_eq!(
        delivery::run_once(&app.state).await,
        0,
        "so a second pass leaves it alone"
    );
    assert_eq!(pass.await.expect("pass"), 1);
    assert_eq!(rows(&db).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn an_unreachable_inbox_records_the_error(db: PgPool) {
    let app = app_with(db.clone(), open);
    let ana = insert_user(&db, "ana").await;
    let id = add(
        &app,
        Some(ana),
        "http://127.0.0.1:1/inbox",
        &format!("{ORIGIN}/users/ana"),
    )
    .await;
    assert_eq!(delivery::run_once(&app.state).await, 1);
    let (attempts, wait, error) = state_of(&db, id).await;
    assert_eq!(attempts, 1);
    assert!((28.0..=31.0).contains(&wait), "{wait}");
    let error = error.expect("error stored");
    assert!(error.starts_with("transport error"), "{error}");
}

#[sqlx::test(migrations = "./migrations")]
async fn an_address_the_client_refuses_is_a_failed_attempt_with_a_reason(db: PgPool) {
    let app = app(db.clone());
    let fake = Fake::start().await;
    let ana = insert_user(&db, "ana").await;
    let id = add(
        &app,
        Some(ana),
        &fake.url("/inbox/202").replacen("http://", "https://", 1),
        &format!("{ORIGIN}/users/ana"),
    )
    .await;
    assert_eq!(delivery::run_once(&app.state).await, 1);
    let (attempts, _, error) = state_of(&db, id).await;
    assert_eq!(attempts, 1);
    let error = error.expect("error stored");
    assert!(
        error.contains("not allowed") || error.contains("blocked"),
        "{error}"
    );
    assert_eq!(fake.count(), 0, "nothing reached the private address");
}

#[sqlx::test(migrations = "./migrations")]
async fn the_worker_loop_delivers_and_stops_on_the_token(db: PgPool) {
    let app = app_with(db.clone(), open);
    let fake = Fake::start().await;
    let ana = insert_user(&db, "ana").await;
    add(
        &app,
        Some(ana),
        &fake.url("/inbox/202"),
        &format!("{ORIGIN}/users/ana"),
    )
    .await;
    let stop = tokio_util::sync::CancellationToken::new();
    let worker = tokio::spawn(delivery::run(app.state.clone(), stop.clone()));
    for _ in 0..50 {
        if rows(&db).await == 0 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(rows(&db).await, 0, "the first tick delivers");
    stop.cancel();
    tokio::time::timeout(Duration::from_secs(5), worker)
        .await
        .expect("stops")
        .expect("no panic");
}

#[sqlx::test(migrations = "./migrations")]
async fn an_inbox_url_is_normalised_so_a_domain_block_always_matches_it(db: PgPool) {
    let app = app_with(db.clone(), open);
    let signer = insert_user(&db, "ana").await;
    for url in [
        "HTTPS://Example.COM:443/inbox",
        "https://example.com?x=1",
        "https://example.com./inbox",
    ] {
        deliveries::enqueue(app.state.db(), Some(signer), url, &json!({"type": "Like"}))
            .await
            .expect("enqueue");
    }
    for url in ["ftp://example.com/inbox", "javascript:alert(1)", "nope"] {
        deliveries::enqueue(app.state.db(), Some(signer), url, &json!({}))
            .await
            .expect("enqueue");
    }
    let stored: Vec<(String, Option<String>)> =
        sqlx::query_as("select inbox_url, domain from ap_deliveries order by inbox_url")
            .fetch_all(&db)
            .await
            .expect("rows");
    let domains: Vec<Option<&str>> = stored.iter().map(|(_, d)| d.as_deref()).collect();
    assert_eq!(domains, [Some("example.com"); 3], "{stored:?}");
    assert!(
        stored
            .iter()
            .all(|(url, _)| url.starts_with("https://example.com/"))
    );

    sqlx::query("insert into domain_blocks (domain, severity) values ('example.com', 'suspend')")
        .execute(&db)
        .await
        .expect("block");
    assert_eq!(delivery::run_once(&app.state).await, 0);
    assert_eq!(rows(&db).await, 0, "every variant was dropped unsent");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_block_covers_the_subdomains_of_a_domain(db: PgPool) {
    use tuita_server::db::federation::block;
    use tuita_server::dto::DomainBlockSeverity;

    let app = app_with(db.clone(), open);
    let ana = insert_user(&db, "ana").await;
    let actor = format!("{ORIGIN}/users/ana");
    sqlx::query("insert into domain_blocks (domain, severity) values ('evil.invalid', 'suspend')")
        .execute(&db)
        .await
        .expect("block");
    for inbox in [
        "https://evil.invalid/inbox",
        "https://sub.evil.invalid/inbox",
    ] {
        add(&app, Some(ana), inbox, &actor).await;
    }
    assert_eq!(
        delivery::run_once(&app.state).await,
        0,
        "nothing is sent to a blocked domain or its subdomains"
    );
    assert_eq!(rows(&db).await, 0);

    // The block itself deletes what is queued for the domain and below it.
    sqlx::query("delete from domain_blocks")
        .execute(&db)
        .await
        .expect("unblock");
    for inbox in [
        "https://a.b.evil.invalid/inbox",
        "https://evil.invalid/inbox",
        "https://notevil.invalid/inbox",
    ] {
        add(&app, Some(ana), inbox, &actor).await;
    }
    let mut conn = db.acquire().await.expect("conn");
    block(
        &mut conn,
        ana,
        "evil.invalid",
        DomainBlockSeverity::Suspend,
        "",
    )
    .await
    .expect("block");
    let left: Vec<String> = sqlx::query_scalar("select inbox_url from ap_deliveries")
        .fetch_all(&db)
        .await
        .expect("rows");
    assert_eq!(left, ["https://notevil.invalid/inbox"]);
}
