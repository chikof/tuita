//! Database access for message keys: each local account's public key and its passphrase-sealed
//! private-key backup. The server never opens the backup.

use std::collections::HashMap;

use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::dto::MessageKeyDto;
use crate::error::AppError;

/// A stored key. No `Debug`: the backup stays out of logs.
#[derive(FromRow)]
struct KeyRow {
    public_key: String,
    backup: String,
    created_at: OffsetDateTime,
}

impl From<KeyRow> for MessageKeyDto {
    fn from(row: KeyRow) -> Self {
        Self {
            public_key: row.public_key,
            backup: row.backup,
            created_at: row.created_at,
        }
    }
}

/// The key of `user`, if they turned encryption on.
///
/// # Errors
///
/// Database failures.
pub async fn get(db: &PgPool, user: Uuid) -> Result<Option<MessageKeyDto>, AppError> {
    let row: Option<KeyRow> = sqlx::query_as(
        "select public_key, backup, created_at from message_keys where user_id = $1",
    )
    .bind(user)
    .fetch_optional(db)
    .await?;
    Ok(row.map(Into::into))
}

/// The public keys of `users`, by account id: one query. Accounts without a key (and every
/// remote account) are absent.
///
/// # Errors
///
/// Database failures.
pub async fn get_many(db: &PgPool, users: &[Uuid]) -> Result<HashMap<Uuid, String>, AppError> {
    let rows: Vec<(Uuid, String)> =
        sqlx::query_as("select user_id, public_key from message_keys where user_id = any($1)")
            .bind(users)
            .fetch_all(db)
            .await?;
    Ok(rows.into_iter().collect())
}

/// Stores the key of `user`. An existing key is replaced only when `replace` is set; `None`
/// says it was kept. The caller has validated both values.
///
/// # Errors
///
/// Database failures.
pub async fn set(
    db: &PgPool,
    user: Uuid,
    public_key: &str,
    backup: &str,
    replace: bool,
) -> Result<Option<MessageKeyDto>, AppError> {
    let row: Option<KeyRow> = sqlx::query_as(
        "insert into message_keys (user_id, public_key, backup) values ($1, $2, $3)
         on conflict (user_id) do update
            set public_key = excluded.public_key, backup = excluded.backup,
                created_at = now(), updated_at = now()
          where $4
         returning public_key, backup, created_at",
    )
    .bind(user)
    .bind(public_key)
    .bind(backup)
    .bind(replace)
    .fetch_optional(db)
    .await?;
    Ok(row.map(Into::into))
}
