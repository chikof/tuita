//! Federation groundwork: actor keys (sign-up and lazy backfill) and signed fetches.

mod common;

use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, Uri};
use axum::routing::get;
use serde_json::json;
use sqlx::PgPool;
use time::OffsetDateTime;
use tokio::net::TcpListener;
use tuita_core::ap::{self, RequestParts};
use tuita_server::federation::{FedError, fetch, keys};

use common::{app_with, insert_user};

type Seen = Arc<Mutex<Option<(String, Vec<(String, String)>)>>>;

async fn actor(
    State(seen): State<Seen>,
    uri: Uri,
    headers: HeaderMap,
) -> axum::Json<serde_json::Value> {
    let pairs = headers
        .iter()
        .map(|(name, value)| {
            (
                name.to_string(),
                value.to_str().unwrap_or_default().to_owned(),
            )
        })
        .collect();
    *seen.lock().expect("lock") = Some((uri.to_string(), pairs));
    axum::Json(json!({"id": "http://x/actor", "type": "Person"}))
}

fn open(var: &str) -> Option<String> {
    matches!(
        var,
        "FEDERATION_ALLOW_HTTP" | "FEDERATION_ALLOW_PRIVATE_NETS"
    )
    .then(|| "1".to_owned())
}

#[sqlx::test(migrations = "./migrations")]
async fn a_fetch_is_signed_by_the_instance_actor_and_verifies(db: PgPool) {
    let app = app_with(db, open);
    let seen: Seen = Arc::default();
    let router = Router::new()
        .route("/users/bob", get(actor))
        .with_state(seen.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });

    let got = fetch::fetch_json(&app.state, &format!("http://{addr}/users/bob?x=1"))
        .await
        .expect("fetched");
    assert_eq!(got["type"], "Person");

    let (target, headers) = seen.lock().expect("lock").clone().expect("request seen");
    let view: Vec<(&str, &str)> = headers
        .iter()
        .map(|(n, v)| (n.as_str(), v.as_str()))
        .collect();
    let parts = RequestParts {
        method: "GET",
        path_and_query: &target,
        headers: &view,
    };
    let header = parts.header("signature").expect("signature header");
    let parsed = ap::parse_signature_header(&header).expect("parses");
    assert_eq!(
        parsed.key_id,
        format!("{}/actor#main-key", app.state.config().public_url)
    );
    assert_eq!(
        parts
            .header("accept")
            .as_deref()
            .map(|a| a.starts_with("application/activity+json")),
        Some(true)
    );
    ap::check_request(&parts, &parsed, b"", OffsetDateTime::now_utc()).expect("checks");
    let pem = keys::instance_public_pem(&app.state).await.expect("pem");
    ap::verify_signature(
        &parts,
        &parsed,
        &ap::PublicKey::from_pem(&pem).expect("key"),
    )
    .expect("the signature verifies");
}

#[sqlx::test(migrations = "./migrations")]
async fn fetch_errors_use_the_documented_variants(db: PgPool) {
    let app = app_with(db, open);
    let router = Router::new().route("/html", get(|| async { "<html>no</html>" }));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    let missing = fetch::fetch_json(&app.state, &format!("http://{addr}/nope")).await;
    assert!(matches!(missing, Err(FedError::NotFound)), "{missing:?}");
    let junk = fetch::fetch_json(&app.state, &format!("http://{addr}/html")).await;
    assert!(matches!(junk, Err(FedError::Unreadable(_))), "{junk:?}");
    let strict = app_with(app.state.db().clone(), |_| None);
    let refused = fetch::fetch_json(&strict.state, &format!("http://{addr}/html")).await;
    assert!(
        matches!(refused, Err(FedError::Unreachable(_))),
        "{refused:?}"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn eight_first_uses_of_a_key_store_one(db: PgPool) {
    let app = app_with(db.clone(), |_| None);
    let user = insert_user(&db, "ada").await;
    let mut tasks = Vec::new();
    for _ in 0..8 {
        let state = app.state.clone();
        tasks.push(tokio::spawn(async move {
            let key = keys::user_signing_key(&state, user).await.expect("key");
            key.private.public_key().fingerprint().expect("fingerprint")
        }));
    }
    let mut prints = Vec::new();
    for task in tasks {
        prints.push(task.await.expect("task"));
    }
    prints.dedup();
    assert_eq!(
        prints.len(),
        1,
        "every caller signs with the stored key: {prints:?}"
    );
    let pem = keys::user_public_pem(&app.state, user).await.expect("pem");
    assert_eq!(ap::fingerprint(&pem).expect("fingerprint"), prints[0]);
    let key = keys::user_signing_key(&app.state, user).await.expect("key");
    assert_eq!(
        key.key_id,
        format!("{}/users/ada#main-key", app.state.config().public_url)
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn eight_first_uses_of_the_instance_key_store_one(db: PgPool) {
    let app = app_with(db.clone(), |_| None);
    let mut tasks = Vec::new();
    for _ in 0..8 {
        let state = app.state.clone();
        tasks.push(tokio::spawn(async move {
            keys::instance_fingerprint(&state)
                .await
                .expect("fingerprint")
        }));
    }
    let mut prints = Vec::new();
    for task in tasks {
        prints.push(task.await.expect("task"));
    }
    prints.dedup();
    assert_eq!(prints.len(), 1, "{prints:?}");
    let rows: i64 =
        sqlx::query_scalar("select count(*) from instance_settings where key = 'instance_actor'")
            .fetch_one(&db)
            .await
            .expect("count");
    assert_eq!(rows, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn sign_up_stores_a_key_pair(db: PgPool) {
    let app = app_with(db.clone(), |_| None);
    let mut browser = app.browser();
    browser.signup("ada").await;
    let (public, private): (Option<String>, Option<String>) =
        sqlx::query_as("select public_key_pem, private_key_pem from users where handle = 'ada'")
            .fetch_one(&db)
            .await
            .expect("row");
    assert!(
        public
            .expect("public")
            .starts_with("-----BEGIN PUBLIC KEY-----")
    );
    assert!(
        private
            .expect("private")
            .starts_with("-----BEGIN PRIVATE KEY-----")
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn deliveries_dedupe_per_inbox(db: PgPool) {
    let mut conn = db.acquire().await.expect("conn");
    let activity = json!({"type": "Create"});
    let added = tuita_server::db::deliveries::enqueue_many(
        &mut conn,
        None,
        [
            "https://a.example/inbox",
            "https://b.example/inbox",
            "https://a.example/inbox",
        ],
        &activity,
    )
    .await
    .expect("enqueue");
    assert_eq!(added, 2);
    let domains: Vec<String> =
        sqlx::query_scalar("select domain from ap_deliveries order by domain")
            .fetch_all(&db)
            .await
            .expect("rows");
    assert_eq!(domains, ["a.example", "b.example"]);
}

#[sqlx::test(migrations = false)]
async fn migration_0004_refuses_to_adopt_a_local_user_named_tuita(db: PgPool) {
    tuita_server::MIGRATOR
        .run_to(3, &db)
        .await
        .expect("migrations before 0004");
    sqlx::query("insert into users (id, handle, display_name) values (gen_random_uuid(), 'Tuita', 'a person')")
        .execute(&db)
        .await
        .expect("user");
    let err = tuita_server::MIGRATOR
        .run_to(4, &db)
        .await
        .expect_err("the handle is taken");
    assert!(err.to_string().contains("reserved handle"), "{err}");
    let accounts: i64 =
        sqlx::query_scalar("select count(*) from users where lower(handle) = 'tuita'")
            .fetch_one(&db)
            .await
            .expect("count");
    assert_eq!(accounts, 1, "nothing was adopted or added");
}

#[sqlx::test(migrations = "./migrations")]
async fn eight_concurrent_first_uses_run_one_key_generation(db: PgPool) {
    let app = app_with(db.clone(), |_| None);
    let user = insert_user(&db, "ada").await;
    let mut tasks = Vec::new();
    for _ in 0..8 {
        let state = app.state.clone();
        tasks.push(tokio::spawn(async move {
            keys::user_public_pem(&state, user).await.expect("pem")
        }));
    }
    for task in tasks {
        task.await.expect("task");
    }
    assert_eq!(app.state.keygen().made(), 1);

    // A different account is another generation; a stored key is never generated again.
    let bob = insert_user(&db, "bob").await;
    keys::user_public_pem(&app.state, bob).await.expect("pem");
    keys::user_public_pem(&app.state, bob).await.expect("pem");
    assert_eq!(app.state.keygen().made(), 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn eight_concurrent_first_uses_of_the_instance_key_run_one_generation(db: PgPool) {
    let app = app_with(db, |_| None);
    let mut tasks = Vec::new();
    for _ in 0..8 {
        let state = app.state.clone();
        tasks.push(tokio::spawn(async move {
            keys::instance_public_pem(&state).await.expect("pem")
        }));
    }
    for task in tasks {
        task.await.expect("task");
    }
    assert_eq!(app.state.keygen().made(), 1);
}
