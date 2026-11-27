//! `SECRET_KEY`: the ActivityPub signing keys are sealed in the database and still sign.

mod common;

use std::io;
use std::sync::{Arc, Mutex, PoisonError};

use sqlx::PgPool;
use tuita_core::ap::PublicKey;
use tuita_server::federation::FedError;
use tuita_server::federation::keys::{self, SigningKey};
use tuita_server::{Config, ConfigError};
use uuid::Uuid;

use common::{App, app_with, insert_user};

const KEY_A: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
const KEY_B: &str = "ffeeddccbbaa99887766554433221100ffeeddccbbaa99887766554433221100";

fn keyed(key: &'static str) -> impl Fn(&str) -> Option<String> {
    move |var| (var == "SECRET_KEY").then(|| key.to_owned())
}

fn config(key: Option<&'static str>) -> Result<Config, ConfigError> {
    Config::from_lookup(|var| match var {
        "DATABASE_URL" => Some("postgres://unused".to_owned()),
        "SECRET_KEY" => key.map(str::to_owned),
        _ => None,
    })
}

async fn user_id(db: &PgPool, handle: &str) -> Uuid {
    sqlx::query_scalar("select id from users where handle = $1")
        .bind(handle)
        .fetch_one(db)
        .await
        .expect("user id")
}

async fn stored_private(db: &PgPool, handle: &str) -> String {
    sqlx::query_scalar("select private_key_pem from users where handle = $1")
        .bind(handle)
        .fetch_one(db)
        .await
        .expect("private key")
}

async fn stored_public(db: &PgPool, handle: &str) -> String {
    sqlx::query_scalar("select public_key_pem from users where handle = $1")
        .bind(handle)
        .fetch_one(db)
        .await
        .expect("public key")
}

async fn stored_instance_private(db: &PgPool) -> String {
    sqlx::query_scalar(
        "select value->>'privatePem' from instance_settings where key = 'instance_actor'",
    )
    .fetch_one(db)
    .await
    .expect("instance key")
}

/// A signature made with `key` that verifies under `public_pem`.
fn signs(key: &SigningKey, public_pem: &str) -> bool {
    let message = b"(request-target): post /users/ada/inbox";
    let signature = key.private.sign_sha256(message).expect("sign");
    PublicKey::from_pem(public_pem)
        .expect("public key")
        .verify_sha256(message, &signature)
}

fn is_plain_pem(text: &str) -> bool {
    text.contains("PRIVATE KEY")
}

#[sqlx::test(migrations = "./migrations")]
async fn sign_up_stores_the_private_key_sealed_and_it_still_signs(db: PgPool) {
    let app = app_with(db.clone(), keyed(KEY_A));
    app.browser().signup("ada").await;

    let stored = stored_private(&db, "ada").await;
    assert!(stored.starts_with("enc:v1:"), "{stored}");
    assert!(!is_plain_pem(&stored), "plaintext PEM text in the column");
    let sealed_in_sql: bool = sqlx::query_scalar(
        "select private_key_pem like 'enc:v1:%' from users where handle = 'ada'",
    )
    .fetch_one(&db)
    .await
    .expect("sql check");
    assert!(sealed_in_sql);

    let id = user_id(&db, "ada").await;
    let key = keys::user_signing_key(&app.state, id).await.expect("key");
    assert!(signs(&key, &stored_public(&db, "ada").await));
}

#[sqlx::test(migrations = "./migrations")]
async fn lazily_made_keys_are_sealed_too(db: PgPool) {
    let app = app_with(db.clone(), keyed(KEY_A));
    let id = insert_user(&db, "bob").await;

    let key = keys::user_signing_key(&app.state, id).await.expect("key");
    let stored = stored_private(&db, "bob").await;
    assert!(stored.starts_with("enc:v1:"), "{stored}");
    assert!(signs(&key, &stored_public(&db, "bob").await));

    let instance = keys::instance_signing_key(&app.state)
        .await
        .expect("instance");
    let stored = stored_instance_private(&db).await;
    assert!(stored.starts_with("enc:v1:"), "{stored}");
    let public = keys::instance_public_pem(&app.state).await.expect("public");
    assert!(signs(&instance, &public));
}

/// A database that predates `SECRET_KEY`: two accounts and the instance actor, all plaintext.
async fn plaintext_database(db: &PgPool) -> App {
    let app = app_with(db.clone(), |_| None);
    app.browser().signup("ada").await;
    app.browser().signup("bob").await;
    keys::instance_public_pem(&app.state)
        .await
        .expect("instance key");
    assert!(is_plain_pem(&stored_private(db, "ada").await));
    assert!(is_plain_pem(&stored_instance_private(db).await));
    app
}

#[sqlx::test(migrations = "./migrations")]
async fn existing_plaintext_keys_are_sealed_once_and_keep_signing(db: PgPool) {
    let plain = plaintext_database(&db).await;
    let ada = user_id(&db, "ada").await;
    let before = keys::user_signing_key(&plain.state, ada)
        .await
        .expect("key");
    let fingerprint = before
        .private
        .public_key()
        .fingerprint()
        .expect("fingerprint");
    let instance_before = keys::instance_signing_key(&plain.state)
        .await
        .expect("instance");
    let instance_print = instance_before
        .private
        .public_key()
        .fingerprint()
        .expect("fingerprint");

    let sealed = app_with(db.clone(), keyed(KEY_A));
    keys::protect_at_rest(&db, sealed.state.config())
        .await
        .expect("first run");
    let (ada_row, bob_row, instance_row) = (
        stored_private(&db, "ada").await,
        stored_private(&db, "bob").await,
        stored_instance_private(&db).await,
    );
    for stored in [&ada_row, &bob_row, &instance_row] {
        assert!(stored.starts_with("enc:v1:"), "{stored}");
        assert!(!is_plain_pem(stored));
    }

    keys::protect_at_rest(&db, sealed.state.config())
        .await
        .expect("second run");
    assert_eq!(
        stored_private(&db, "ada").await,
        ada_row,
        "second run rewrote a row"
    );
    assert_eq!(stored_private(&db, "bob").await, bob_row);
    assert_eq!(stored_instance_private(&db).await, instance_row);

    let after = keys::user_signing_key(&sealed.state, ada)
        .await
        .expect("key");
    assert_eq!(
        after
            .private
            .public_key()
            .fingerprint()
            .expect("fingerprint"),
        fingerprint,
        "same key, now read from a sealed row"
    );
    assert!(signs(&after, &stored_public(&db, "ada").await));
    let instance = keys::instance_signing_key(&sealed.state)
        .await
        .expect("instance");
    assert_eq!(
        instance
            .private
            .public_key()
            .fingerprint()
            .expect("fingerprint"),
        instance_print
    );
    let public = keys::instance_public_pem(&sealed.state)
        .await
        .expect("public");
    assert!(signs(&instance, &public));
}

#[sqlx::test(migrations = "./migrations")]
async fn two_processes_sealing_at_once_converge(db: PgPool) {
    let plain = plaintext_database(&db).await;
    let ada = user_id(&db, "ada").await;
    let print = keys::user_signing_key(&plain.state, ada)
        .await
        .expect("key")
        .private
        .public_key()
        .fingerprint()
        .expect("fingerprint");

    let one = app_with(db.clone(), keyed(KEY_A));
    let two = app_with(db.clone(), keyed(KEY_A));
    let (a, b) = tokio::join!(
        keys::protect_at_rest(&db, one.state.config()),
        keys::protect_at_rest(&db, two.state.config()),
    );
    a.expect("first process");
    b.expect("second process");

    let key = keys::user_signing_key(&one.state, ada).await.expect("key");
    assert_eq!(
        key.private.public_key().fingerprint().expect("fingerprint"),
        print
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_sealed_key_copied_to_another_row_does_not_open(db: PgPool) {
    let app = app_with(db.clone(), keyed(KEY_A));
    app.browser().signup("ada").await;
    app.browser().signup("bob").await;
    let ada_sealed = stored_private(&db, "ada").await;
    sqlx::query("update users set private_key_pem = $1 where handle = 'bob'")
        .bind(&ada_sealed)
        .execute(&db)
        .await
        .expect("copy");

    let bob = user_id(&db, "bob").await;
    let err = keys::user_signing_key(&app.state, bob)
        .await
        .expect_err("a moved ciphertext must not open");
    assert!(format!("{err:?}").contains("SECRET_KEY"), "{err:?}");
    let ada = user_id(&db, "ada").await;
    keys::user_signing_key(&app.state, ada)
        .await
        .expect("ada still signs");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_missing_or_wrong_key_stops_startup_and_names_the_variable(db: PgPool) {
    let app = app_with(db.clone(), keyed(KEY_A));
    app.browser().signup("ada").await;
    keys::instance_public_pem(&app.state)
        .await
        .expect("instance key");
    let ada_row = stored_private(&db, "ada").await;
    let instance_row = stored_instance_private(&db).await;

    for (case, key) in [("missing", None), ("wrong", Some(KEY_B))] {
        let config = config(key).expect("valid config");
        let err = keys::protect_at_rest(&db, &config)
            .await
            .expect_err("startup must fail");
        assert!(err.to_string().contains("SECRET_KEY"), "{case}: {err:#}");
    }
    assert_eq!(
        stored_private(&db, "ada").await,
        ada_row,
        "nothing regenerated or rewritten"
    );
    assert_eq!(stored_instance_private(&db).await, instance_row);
}

#[derive(Clone, Default)]
struct Logs(Arc<Mutex<Vec<u8>>>);

impl io::Write for Logs {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Logs {
    fn text(&self) -> String {
        let bytes = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn without_a_key_nothing_changes_but_one_warning(db: PgPool) {
    let logs = Logs::default();
    let writer = logs.clone();
    let _guard = tracing::subscriber::set_default(
        tracing_subscriber::fmt()
            .with_writer(move || writer.clone())
            .with_ansi(false)
            .finish(),
    );

    let app = plaintext_database(&db).await;
    let before = stored_private(&db, "ada").await;
    let instance_before = stored_instance_private(&db).await;
    keys::protect_at_rest(&db, app.state.config())
        .await
        .expect("startup without a key");

    assert_eq!(stored_private(&db, "ada").await, before);
    assert_eq!(stored_instance_private(&db).await, instance_before);
    let ada = user_id(&db, "ada").await;
    let key = keys::user_signing_key(&app.state, ada).await.expect("key");
    assert!(signs(&key, &stored_public(&db, "ada").await));
    assert_eq!(
        logs.text().matches("SECRET_KEY is not set").count(),
        1,
        "{}",
        logs.text()
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn with_a_key_there_is_no_missing_key_warning(db: PgPool) {
    let logs = Logs::default();
    let writer = logs.clone();
    let _guard = tracing::subscriber::set_default(
        tracing_subscriber::fmt()
            .with_writer(move || writer.clone())
            .with_ansi(false)
            .finish(),
    );
    let app = app_with(db.clone(), keyed(KEY_A));
    keys::protect_at_rest(&db, app.state.config())
        .await
        .expect("startup with a key");
    assert!(
        !logs.text().contains("SECRET_KEY is not set"),
        "{}",
        logs.text()
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn an_erased_account_is_never_given_a_new_key(db: PgPool) {
    let app = app_with(db.clone(), keyed(KEY_A));
    let id = insert_user(&db, "gone").await;
    sqlx::query("update users set deleted_at = now() where id = $1")
        .bind(id)
        .execute(&db)
        .await
        .expect("erase");

    let err = keys::user_signing_key(&app.state, id)
        .await
        .expect_err("no key for an erased account");
    assert!(matches!(err, FedError::NotFound), "{err:?}");
    let stored: Option<String> =
        sqlx::query_scalar("select private_key_pem from users where id = $1")
            .bind(id)
            .fetch_one(&db)
            .await
            .expect("row");
    assert_eq!(stored, None);
}
