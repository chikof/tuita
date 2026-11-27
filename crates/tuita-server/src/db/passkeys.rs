//! Passkeys (WebAuthn credentials) and the challenges in flight.
//!
//! A passkey row holds the serialised webauthn-rs `Passkey`. A challenge row holds the
//! serialised ceremony state and is deleted when it is used, so it works once.

use serde_json::Value;
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

use super::users::{COLS, UserRow};
use crate::dto::PasskeyDto;
use crate::error::AppError;

/// Passkeys one account may hold.
pub const MAX_PASSKEYS: i64 = 20;
/// Minutes a challenge stays valid.
const CHALLENGE_MINUTES: i32 = 5;

/// The `400` for a challenge that is unknown, used, expired or another user's.
pub const EXPIRED: &str = "This passkey request expired. Please try again.";

#[derive(FromRow)]
struct Row {
    id: Uuid,
    name: String,
    created_at: OffsetDateTime,
    last_used_at: Option<OffsetDateTime>,
}

impl From<Row> for PasskeyDto {
    fn from(row: Row) -> Self {
        Self {
            id: row.id,
            name: row.name,
            created_at: row.created_at,
            last_used_at: row.last_used_at,
        }
    }
}

/// The passkeys of `user`, newest first.
///
/// # Errors
///
/// Database failures.
pub async fn list(db: &PgPool, user: Uuid) -> Result<Vec<PasskeyDto>, AppError> {
    let rows: Vec<Row> = sqlx::query_as(
        "select id, name, created_at, last_used_at from passkeys
          where user_id = $1 order by created_at desc, id desc",
    )
    .bind(user)
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

/// The credential ids of `user`'s passkeys, so a registration can exclude them.
///
/// # Errors
///
/// Database failures.
pub async fn credential_ids(db: &PgPool, user: Uuid) -> Result<Vec<Vec<u8>>, AppError> {
    Ok(
        sqlx::query_scalar("select cred_id from passkeys where user_id = $1")
            .bind(user)
            .fetch_all(db)
            .await?,
    )
}

/// The id a user's authenticator stores as the user handle.
///
/// # Errors
///
/// Database failures.
pub async fn webauthn_id(db: &PgPool, user: Uuid) -> Result<Uuid, AppError> {
    Ok(
        sqlx::query_scalar("select webauthn_id from users where id = $1")
            .bind(user)
            .fetch_one(db)
            .await?,
    )
}

/// Stores the state of a ceremony that starts now. `user` is the account registering a
/// passkey, or `None` for a sign-in. Returns the challenge id.
///
/// # Errors
///
/// Database failures.
pub async fn save_challenge(
    db: &PgPool,
    user: Option<Uuid>,
    state: &Value,
) -> Result<Uuid, AppError> {
    let id = Uuid::now_v7();
    sqlx::query(
        "insert into webauthn_challenges (id, user_id, state, expires_at)
         values ($1, $2, $3, now() + make_interval(mins => $4))",
    )
    .bind(id)
    .bind(user)
    .bind(state)
    .bind(CHALLENGE_MINUTES)
    .execute(db)
    .await?;
    Ok(id)
}

/// Uses up challenge `id`: returns its state and deletes it. `None` when it is unknown, already
/// used, expired, or made for another `user` (`None` matches sign-in challenges only).
///
/// # Errors
///
/// Database failures.
pub async fn take_challenge(
    db: &PgPool,
    id: Uuid,
    user: Option<Uuid>,
) -> Result<Option<Value>, AppError> {
    Ok(sqlx::query_scalar(
        "delete from webauthn_challenges
          where id = $1 and user_id is not distinct from $2 and expires_at > now()
          returning state",
    )
    .bind(id)
    .bind(user)
    .fetch_optional(db)
    .await?)
}

/// Removes challenges that expired unused. Returns how many.
///
/// # Errors
///
/// Database failures.
pub async fn expire_challenges(db: &PgPool) -> Result<u64, AppError> {
    Ok(
        sqlx::query("delete from webauthn_challenges where expires_at < now()")
            .execute(db)
            .await?
            .rows_affected(),
    )
}

/// Stores a new passkey for `user`.
///
/// # Errors
///
/// `400 You can have up to 20 passkeys.`; `409 That passkey is already registered.`; database
/// failures.
pub async fn insert(
    db: &PgPool,
    user: Uuid,
    passkey: &Passkey,
    name: &str,
) -> Result<PasskeyDto, AppError> {
    let state = serde_json::to_value(passkey).map_err(|err| AppError::Internal(err.into()))?;
    let mut tx = db.begin().await?;
    // Two registrations at once take turns on the owner's row, so the cap holds.
    sqlx::query("select id from users where id = $1 for update")
        .bind(user)
        .execute(&mut *tx)
        .await?;
    let held: i64 = sqlx::query_scalar("select count(*) from passkeys where user_id = $1")
        .bind(user)
        .fetch_one(&mut *tx)
        .await?;
    if held >= MAX_PASSKEYS {
        return Err(AppError::bad_request("You can have up to 20 passkeys."));
    }
    let inserted: Result<Row, sqlx::Error> = sqlx::query_as(
        "insert into passkeys (id, user_id, cred_id, passkey, name)
         values ($1, $2, $3, $4, $5)
         returning id, name, created_at, last_used_at",
    )
    .bind(Uuid::now_v7())
    .bind(user)
    .bind(passkey.cred_id().as_slice())
    .bind(state)
    .bind(name)
    .fetch_one(&mut *tx)
    .await;
    let row = match inserted {
        Ok(row) => row,
        Err(err)
            if err
                .as_database_error()
                .and_then(|db| db.code())
                .is_some_and(|code| code == "23505") =>
        {
            return Err(AppError::conflict("That passkey is already registered."));
        }
        Err(err) => return Err(err.into()),
    };
    tx.commit().await?;
    Ok(row.into())
}

/// Renames a passkey of `user`.
///
/// # Errors
///
/// Database failures.
pub async fn rename(
    db: &PgPool,
    user: Uuid,
    id: Uuid,
    name: &str,
) -> Result<Option<PasskeyDto>, AppError> {
    let row: Option<Row> = sqlx::query_as(
        "update passkeys set name = $3 where id = $1 and user_id = $2
         returning id, name, created_at, last_used_at",
    )
    .bind(id)
    .bind(user)
    .bind(name)
    .fetch_optional(db)
    .await?;
    Ok(row.map(Into::into))
}

/// Deletes a passkey of `user`. `false` when there was none.
///
/// # Errors
///
/// Database failures.
pub async fn delete(db: &PgPool, user: Uuid, id: Uuid) -> Result<bool, AppError> {
    Ok(
        sqlx::query("delete from passkeys where id = $1 and user_id = $2")
            .bind(id)
            .bind(user)
            .execute(db)
            .await?
            .rows_affected()
            > 0,
    )
}

/// A passkey found by its credential id, with its owner.
#[derive(FromRow)]
pub struct Found {
    /// The row id.
    pub passkey_id: Uuid,
    /// The stored passkey.
    pub passkey: sqlx::types::Json<Passkey>,
    /// The owner.
    #[sqlx(flatten)]
    pub user: UserRow,
}

/// The passkey with credential id `cred_id` whose owner's `webauthn_id` is `webauthn_id`, when
/// the owner is a live local account.
///
/// # Errors
///
/// Database failures.
pub async fn find(
    db: &PgPool,
    cred_id: &[u8],
    webauthn_id: Uuid,
) -> Result<Option<Found>, AppError> {
    let sql = format!(
        "select p.id as passkey_id, p.passkey, {COLS}
           from passkeys p join users u on u.id = p.user_id
          where p.cred_id = $1 and u.webauthn_id = $2 and u.deleted_at is null and u.domain is null"
    );
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(cred_id)
        .bind(webauthn_id)
        .fetch_optional(db)
        .await?)
}

/// Records a sign-in with passkey `id`: the time and the counters the ceremony updated.
///
/// # Errors
///
/// Database failures.
pub async fn record_use(db: &PgPool, id: Uuid, passkey: &Passkey) -> Result<(), AppError> {
    let state = serde_json::to_value(passkey).map_err(|err| AppError::Internal(err.into()))?;
    sqlx::query("update passkeys set passkey = $2, last_used_at = now() where id = $1")
        .bind(id)
        .bind(state)
        .execute(db)
        .await?;
    Ok(())
}
