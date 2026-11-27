//! The lookups the Mastodon mappers need besides a post or a profile card: accounts with their
//! extra columns, the author of a replied-to post, the date of an account's latest status.

use std::collections::HashMap;

use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use super::users::{COLS, UserRow};
use crate::error::AppError;

/// The accounts among `ids`, deleted and suspended ones included (a status keeps its author).
/// Unknown ids are absent.
///
/// # Errors
///
/// Database failures.
pub async fn account_rows(db: &PgPool, ids: &[Uuid]) -> Result<Vec<UserRow>, AppError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let sql = format!("select {COLS} from users u where u.id = any($1)");
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(ids)
        .fetch_all(db)
        .await?)
}

/// When each account in `ids` last posted something public or unlisted. Accounts that never did
/// are absent.
///
/// # Errors
///
/// Database failures.
pub async fn last_status_times(
    db: &PgPool,
    ids: &[Uuid],
) -> Result<HashMap<Uuid, OffsetDateTime>, AppError> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows: Vec<(Uuid, Option<OffsetDateTime>)> = sqlx::query_as(
        "select a.id,
                (select p.created_at from posts p
                  where p.author_id = a.id and p.deleted_at is null
                    and p.visibility in ('public', 'unlisted')
                  order by p.id desc limit 1)
           from unnest($1::uuid[]) as a(id)",
    )
    .bind(ids)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|(id, at)| Some((id, at?)))
        .collect())
}

/// The author of each post in `ids`: `post id -> author id`. Unknown posts are absent.
///
/// # Errors
///
/// Database failures.
pub async fn post_authors(db: &PgPool, ids: &[Uuid]) -> Result<HashMap<Uuid, Uuid>, AppError> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows: Vec<(Uuid, Uuid)> =
        sqlx::query_as("select id, author_id from posts where id = any($1)")
            .bind(ids)
            .fetch_all(db)
            .await?;
    Ok(rows.into_iter().collect())
}
