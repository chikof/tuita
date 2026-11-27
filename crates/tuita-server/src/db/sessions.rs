//! Login sessions. A session is a random token the browser holds; only its SHA-256 is stored.

use std::net::IpAddr;

use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::users::{COLS, UserRow};
use crate::error::AppError;

/// How long a session lasts.
pub const SESSION_DAYS: i64 = 30;

/// What the server learned about the client when a session started or was used.
#[derive(Clone, Debug, Default)]
pub struct ClientMeta {
    /// Client address, when known.
    pub ip: Option<IpAddr>,
    /// The `User-Agent`, cut to 300 characters.
    pub user_agent: String,
    /// Two-letter country from a trusted proxy.
    pub country: Option<String>,
    /// Device family.
    pub client: Option<&'static str>,
}

/// A session joined with its account.
#[derive(FromRow)]
pub struct SessionUser {
    /// The account.
    #[sqlx(flatten)]
    pub user: UserRow,
    /// The session id.
    pub session_id: Uuid,
    /// When the session was last used.
    pub last_used_at: OffsetDateTime,
}

/// A stored session, for the "your sessions" list.
#[derive(FromRow)]
pub struct SessionRow {
    /// Session id.
    pub id: Uuid,
    /// Whether it is the session making the request.
    pub current: bool,
    /// Creation time.
    pub created_at: OffsetDateTime,
    /// Last use.
    pub last_used_at: OffsetDateTime,
    /// End time.
    pub expires_at: OffsetDateTime,
    /// `User-Agent`.
    pub user_agent: String,
    /// Client address as text.
    pub ip: Option<String>,
    /// Country.
    pub country: Option<String>,
    /// Device family.
    pub client: Option<String>,
}

/// Starts a session for `user` that ends in [`SESSION_DAYS`] days. Returns its id.
///
/// # Errors
///
/// Database failures.
pub async fn insert(
    db: &PgPool,
    user: Uuid,
    token_hash: &[u8],
    meta: &ClientMeta,
) -> Result<Uuid, AppError> {
    let id = Uuid::now_v7();
    sqlx::query(
        "insert into sessions (id, user_id, token_hash, expires_at, user_agent, ip, country, client)
         values ($1, $2, $3, now() + make_interval(days => $4), $5, $6, $7, $8)",
    )
    .bind(id)
    .bind(user)
    .bind(token_hash)
    .bind(i32::try_from(SESSION_DAYS).unwrap_or(30))
    .bind(&meta.user_agent)
    .bind(meta.ip)
    .bind(meta.country.as_deref())
    .bind(meta.client)
    .execute(db)
    .await?;
    Ok(id)
}

/// The live session behind `token_hash` and its account. `None` when the token is unknown or
/// expired, or the account is suspended, deleted or remote.
///
/// # Errors
///
/// Database failures.
pub async fn find(db: &PgPool, token_hash: &[u8]) -> Result<Option<SessionUser>, AppError> {
    let sql = format!(
        "select {COLS}, s.id as session_id, s.last_used_at
         from sessions s join users u on u.id = s.user_id
         where s.token_hash = $1 and s.expires_at > now()
           and u.suspended_at is null and u.deleted_at is null and u.domain is null"
    );
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(token_hash)
        .fetch_optional(db)
        .await?)
}

/// Records a use of a session: time, client address and country.
///
/// # Errors
///
/// Database failures.
pub async fn touch(db: &PgPool, session: Uuid, meta: &ClientMeta) -> Result<(), AppError> {
    sqlx::query(
        "update sessions set last_used_at = now(), ip = $2, country = coalesce($3, country) where id = $1",
    )
    .bind(session)
    .bind(meta.ip)
    .bind(meta.country.as_deref())
    .execute(db)
    .await?;
    Ok(())
}

/// Ends the session behind `token_hash`.
///
/// # Errors
///
/// Database failures.
pub async fn delete_token(db: &PgPool, token_hash: &[u8]) -> Result<(), AppError> {
    sqlx::query("delete from sessions where token_hash = $1")
        .bind(token_hash)
        .execute(db)
        .await?;
    Ok(())
}

/// The live sessions of `user`, `current` first, then the most recently used.
///
/// # Errors
///
/// Database failures.
pub async fn list(db: &PgPool, user: Uuid, current: Uuid) -> Result<Vec<SessionRow>, AppError> {
    Ok(sqlx::query_as(
        "select id, id = $2 as current, created_at, last_used_at, expires_at, user_agent,
                host(ip) as ip, country, client
         from sessions where user_id = $1 and expires_at > now()
         order by (id = $2) desc, last_used_at desc",
    )
    .bind(user)
    .bind(current)
    .fetch_all(db)
    .await?)
}

/// Ends every session of `user` except `keep`. Returns how many ended.
///
/// # Errors
///
/// Database failures.
pub async fn delete_others(db: &PgPool, user: Uuid, keep: Uuid) -> Result<u64, AppError> {
    Ok(
        sqlx::query("delete from sessions where user_id = $1 and id <> $2")
            .bind(user)
            .bind(keep)
            .execute(db)
            .await?
            .rows_affected(),
    )
}

/// Ends session `id` of `user`. Returns whether it existed.
///
/// # Errors
///
/// Database failures.
pub async fn delete_one(db: &PgPool, user: Uuid, id: Uuid) -> Result<bool, AppError> {
    Ok(
        sqlx::query("delete from sessions where user_id = $1 and id = $2")
            .bind(user)
            .bind(id)
            .execute(db)
            .await?
            .rows_affected()
            > 0,
    )
}

/// Whether session `id` still works: not ended, not expired, and its account is active.
///
/// # Errors
///
/// Database failures.
pub async fn is_live(db: &PgPool, id: Uuid) -> Result<bool, AppError> {
    Ok(sqlx::query_scalar(
        "select exists (select 1 from sessions s join users u on u.id = s.user_id
            where s.id = $1 and s.expires_at > now()
              and u.suspended_at is null and u.deleted_at is null)",
    )
    .bind(id)
    .fetch_one(db)
    .await?)
}
