//! Actor keys: one RSA-2048 pair per local account and one for the instance actor.
//!
//! A pair is made at sign-up. Accounts that predate federation get theirs on first use: one
//! generation at a time per process ([`Generator`]), so concurrent first uses run one RSA
//! keygen and the rest read its result; the pair is stored with `update ... where
//! private_key_pem is null`, which keeps several processes converging on one stored key.
//!
//! With `SECRET_KEY` set, the private halves are stored sealed (`enc:v1:...`, see `secrets.rs`) and
//! [`protect_at_rest`] seals the plaintext rows of an older database at startup. Changing
//! `SECRET_KEY` afterwards is not supported: startup refuses to run against keys it cannot open.

use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Context as _;
use serde_json::{Value, json};
use sqlx::{PgPool, Row as _};
use tuita_core::ap::{self, KeyPair, PrivateKey};
use uuid::Uuid;

use super::FedError;
use super::ids;
use crate::config::Config;
use crate::secrets::{self, SecretError, SecretKey};
use crate::state::AppState;

/// `instance_settings` key of the instance actor's pair.
const INSTANCE_KEY: &str = "instance_actor";

/// A key that signs, and the `keyId` that names its public half.
#[derive(Clone, Debug)]
pub struct SigningKey {
    /// What a signature's `keyId` says.
    pub key_id: String,
    /// The private key.
    pub private: PrivateKey,
}

/// Serialises first-use key generation and counts the pairs it made.
#[derive(Debug, Default)]
pub struct Generator {
    one_at_a_time: tokio::sync::Mutex<()>,
    made: AtomicUsize,
}

impl Generator {
    /// How many pairs first uses generated in this process.
    #[must_use]
    pub fn made(&self) -> usize {
        self.made.load(Ordering::Relaxed)
    }

    async fn generate(&self) -> Result<KeyPair, FedError> {
        self.made.fetch_add(1, Ordering::Relaxed);
        new_pair().await
    }
}

/// Generates a pair on a blocking thread.
///
/// # Errors
///
/// [`FedError::Internal`] when generation fails.
pub async fn new_pair() -> Result<KeyPair, FedError> {
    tokio::task::spawn_blocking(ap::generate_keypair)
        .await
        .map_err(|err| FedError::Internal(err.into()))?
        .map_err(|err| FedError::Internal(err.into()))
}

fn parse(pem: &str) -> Result<PrivateKey, FedError> {
    PrivateKey::from_pem(pem).map_err(|err| FedError::Internal(err.into()))
}

/// The plaintext of a stored private key; `context` names its row.
fn open_private(state: &AppState, context: &str, stored: &str) -> Result<String, FedError> {
    secrets::open(state.config().secret_key.as_deref(), context, stored)
        .map_err(|err| FedError::Internal(err.into()))
}

/// The stored pair of local account `user`, making it when there is none. An erased account
/// keeps no key it did not already have: it is never given a new one.
async fn user_pair(state: &AppState, user: Uuid) -> Result<(String, String, String), FedError> {
    let read = || async {
        sqlx::query(
            "select handle, public_key_pem, private_key_pem, deleted_at is not null as gone
               from users where id = $1 and domain is null",
        )
        .bind(user)
        .fetch_optional(state.db())
        .await
    };
    let row = read().await?.ok_or(FedError::NotFound)?;
    let handle: String = row.try_get("handle")?;
    let public: Option<String> = row.try_get("public_key_pem")?;
    let private: Option<String> = row.try_get("private_key_pem")?;
    if let (Some(public), Some(private)) = (public, private) {
        return Ok((handle, public, private));
    }
    let _alone = state.keygen().one_at_a_time.lock().await;
    let row = read().await?.ok_or(FedError::NotFound)?;
    let public: Option<String> = row.try_get("public_key_pem")?;
    let private: Option<String> = row.try_get("private_key_pem")?;
    if let (Some(public), Some(private)) = (public, private) {
        return Ok((handle, public, private));
    }
    if row.try_get::<bool, _>("gone")? {
        return Err(FedError::NotFound);
    }
    let pair = state.keygen().generate().await?;
    sqlx::query(
        "update users set public_key_pem = $2, private_key_pem = $3
          where id = $1 and private_key_pem is null and deleted_at is null",
    )
    .bind(user)
    .bind(&pair.public_pem)
    .bind(secrets::store(
        state.config().secret_key.as_deref(),
        &secrets::user_context(user),
        &pair.private_pem,
    ))
    .execute(state.db())
    .await?;
    let row = read().await?.ok_or(FedError::NotFound)?;
    let public: Option<String> = row.try_get("public_key_pem")?;
    let private: Option<String> = row.try_get("private_key_pem")?;
    match (public, private) {
        (Some(public), Some(private)) => Ok((handle, public, private)),
        _ => Err(FedError::Internal(anyhow::anyhow!("key was not stored"))),
    }
}

/// The signing key of local account `user`.
///
/// # Errors
///
/// [`FedError::NotFound`] for no such local account; [`FedError::Internal`] for database or key
/// failures.
pub async fn user_signing_key(state: &AppState, user: Uuid) -> Result<SigningKey, FedError> {
    let (handle, _, private) = user_pair(state, user).await?;
    Ok(SigningKey {
        key_id: ids::key_id(&ids::actor_id(state.config(), &handle)),
        private: parse(&open_private(
            state,
            &secrets::user_context(user),
            &private,
        )?)?,
    })
}

/// The public key (PEM) of local account `user`.
///
/// # Errors
///
/// As [`user_signing_key`].
pub async fn user_public_pem(state: &AppState, user: Uuid) -> Result<String, FedError> {
    Ok(user_pair(state, user).await?.1)
}

/// The instance actor's pair as `(public, private)` PEM, made on first use. Racing first uses
/// converge: the insert does nothing when a pair is stored, and everyone reads that one.
async fn instance_pair(state: &AppState) -> Result<(String, String), FedError> {
    let read = || async {
        sqlx::query_scalar::<_, Value>("select value from instance_settings where key = $1")
            .bind(INSTANCE_KEY)
            .fetch_optional(state.db())
            .await
    };
    let mut stored = read().await?;
    if stored.is_none() {
        let _alone = state.keygen().one_at_a_time.lock().await;
        stored = read().await?;
        if stored.is_none() {
            let pair = state.keygen().generate().await?;
            sqlx::query(
                "insert into instance_settings (key, value) values ($1, $2) on conflict do nothing",
            )
            .bind(INSTANCE_KEY)
            .bind(json!({
                "publicPem": pair.public_pem,
                "privatePem": secrets::store(
                    state.config().secret_key.as_deref(),
                    secrets::INSTANCE_CONTEXT,
                    &pair.private_pem,
                ),
            }))
            .execute(state.db())
            .await?;
            stored = read().await?;
        }
    }
    let value = stored.ok_or_else(|| FedError::Internal(anyhow::anyhow!("no instance key")))?;
    let text = |name: &str| value.get(name).and_then(Value::as_str).map(str::to_owned);
    match (text("publicPem"), text("privatePem")) {
        (Some(public), Some(private)) => Ok((public, private)),
        _ => Err(FedError::Internal(anyhow::anyhow!(
            "malformed instance key"
        ))),
    }
}

/// The key that signs fetches: the instance actor's.
///
/// # Errors
///
/// [`FedError::Internal`] for database or key failures.
pub async fn instance_signing_key(state: &AppState) -> Result<SigningKey, FedError> {
    let (_, private) = instance_pair(state).await?;
    Ok(SigningKey {
        key_id: ids::key_id(&ids::instance_actor_id(state.config())),
        private: parse(&open_private(state, secrets::INSTANCE_CONTEXT, &private)?)?,
    })
}

/// The instance actor's public key (PEM).
///
/// # Errors
///
/// As [`instance_signing_key`].
pub async fn instance_public_pem(state: &AppState) -> Result<String, FedError> {
    Ok(instance_pair(state).await?.0)
}

/// SHA-256 of the instance actor's public key, lower-case hex: what the admin compares.
///
/// # Errors
///
/// As [`instance_signing_key`].
pub async fn instance_fingerprint(state: &AppState) -> Result<String, FedError> {
    let pem = instance_public_pem(state).await?;
    ap::fingerprint(&pem).map_err(|err| FedError::Internal(err.into()))
}

/// How many plaintext keys one startup batch seals.
const BATCH: i64 = 100;

/// Startup step for `SECRET_KEY`: makes the stored private keys agree with the configuration.
///
/// With a key, it checks that the key opens the keys already sealed, then seals every plaintext
/// private key (accounts and the instance actor) in batches. Each row is rewritten only while it
/// still holds the plaintext this run read, so a second run, or a second process, changes
/// nothing. Without a key it warns, and refuses to continue when a sealed key is stored: keys
/// are never regenerated.
///
/// # Errors
///
/// A message naming `SECRET_KEY` when the key is missing or does not open a stored key; database
/// failures.
pub async fn protect_at_rest(db: &PgPool, config: &Config) -> anyhow::Result<()> {
    let Some(key) = config.secret_key.as_deref() else {
        if any_sealed(db).await? {
            return Err(SecretError::MissingKey.into());
        }
        tracing::warn!(
            "SECRET_KEY is not set: ActivityPub signing keys are stored unencrypted in the database"
        );
        return Ok(());
    };
    check_key(db, key).await?;
    let accounts = seal_users(db, key).await?;
    let instance = seal_instance(db, key).await?;
    if accounts + instance > 0 {
        tracing::info!(accounts, instance, "sealed stored ActivityPub signing keys");
    }
    Ok(())
}

async fn any_sealed(db: &PgPool) -> anyhow::Result<bool> {
    sqlx::query_scalar(
        "select exists (select 1 from users where private_key_pem like 'enc:%')
             or exists (select 1 from instance_settings
                         where key = $1 and value->>'privatePem' like 'enc:%')",
    )
    .bind(INSTANCE_KEY)
    .fetch_one(db)
    .await
    .context("cannot look for sealed signing keys")
}

/// Opens one sealed account key and the sealed instance key, when there are any: a wrong
/// `SECRET_KEY` fails here, once, instead of at the first signature.
async fn check_key(db: &PgPool, key: &SecretKey) -> anyhow::Result<()> {
    let account: Option<(Uuid, String)> = sqlx::query_as(
        "select id, private_key_pem from users where private_key_pem like 'enc:%'
          order by id limit 1",
    )
    .fetch_optional(db)
    .await
    .context("cannot read a sealed account key")?;
    if let Some((id, stored)) = account {
        secrets::open(Some(key), &secrets::user_context(id), &stored)?;
    }
    let instance: Option<String> =
        sqlx::query_scalar("select value->>'privatePem' from instance_settings where key = $1")
            .bind(INSTANCE_KEY)
            .fetch_optional(db)
            .await
            .context("cannot read the sealed instance key")?
            .flatten();
    if let Some(stored) = instance.filter(|stored| secrets::is_sealed(stored)) {
        secrets::open(Some(key), secrets::INSTANCE_CONTEXT, &stored)?;
    }
    Ok(())
}

/// Seals the plaintext private keys of accounts, a batch at a time in id order; returns how
/// many rows this run changed.
async fn seal_users(db: &PgPool, key: &SecretKey) -> anyhow::Result<u64> {
    let mut sealed = 0;
    let mut after = Uuid::nil();
    loop {
        let batch: Vec<(Uuid, String)> = sqlx::query_as(
            "select id, private_key_pem from users
              where id > $1 and private_key_pem is not null and private_key_pem not like 'enc:%'
              order by id limit $2",
        )
        .bind(after)
        .bind(BATCH)
        .fetch_all(db)
        .await
        .context("cannot read plaintext account keys")?;
        let Some((last, _)) = batch.last() else {
            return Ok(sealed);
        };
        after = *last;
        for (id, plain) in &batch {
            sealed += sqlx::query(
                "update users set private_key_pem = $2 where id = $1 and private_key_pem = $3",
            )
            .bind(id)
            .bind(secrets::seal(key, &secrets::user_context(*id), plain))
            .bind(plain)
            .execute(db)
            .await
            .context("cannot seal an account key")?
            .rows_affected();
        }
    }
}

/// Seals the instance actor's private key when it is plaintext; returns how many rows changed.
async fn seal_instance(db: &PgPool, key: &SecretKey) -> anyhow::Result<u64> {
    let stored: Option<String> =
        sqlx::query_scalar("select value->>'privatePem' from instance_settings where key = $1")
            .bind(INSTANCE_KEY)
            .fetch_optional(db)
            .await
            .context("cannot read the instance key")?
            .flatten();
    let Some(plain) = stored.filter(|stored| !secrets::is_sealed(stored)) else {
        return Ok(0);
    };
    Ok(sqlx::query(
        "update instance_settings set value = jsonb_set(value, '{privatePem}', to_jsonb($2::text))
          where key = $1 and value->>'privatePem' = $3",
    )
    .bind(INSTANCE_KEY)
    .bind(secrets::seal(key, secrets::INSTANCE_CONTEXT, &plain))
    .bind(&plain)
    .execute(db)
    .await
    .context("cannot seal the instance key")?
    .rows_affected())
}
