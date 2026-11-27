//! API tokens: personal secrets for scripts and bots, sent as `Authorization: Bearer`.
//!
//! Only the SHA-256 of a secret is stored, so a secret is shown once, when it is made.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sqlx::{FromRow, PgPool};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::users::{COLS, UserRow};
use crate::auth::hash_token;
use crate::dto::{CreatedTokenDto, TokenDto, TokenScope};
use crate::error::AppError;

/// What every secret starts with.
pub const SECRET_PREFIX: &str = "tuita_";
/// Tokens one account may hold.
pub const MAX_TOKENS: i64 = 20;
/// Characters of a secret kept to tell tokens apart: [`SECRET_PREFIX`] and 6 more.
const SHOWN_PREFIX: usize = 12;
/// How stale `last_used_at` may get before a request refreshes it.
const TOUCH_AFTER: Duration = Duration::minutes(1);

/// A new secret: [`SECRET_PREFIX`] and 32 random bytes, base64url without padding.
#[must_use]
pub fn new_secret() -> String {
    let mut bytes = [0_u8; 32];
    rand::fill(&mut bytes);
    format!("{SECRET_PREFIX}{}", URL_SAFE_NO_PAD.encode(bytes))
}

/// Whether `secret` has the shape of one of ours, so a lookup is worth a query.
#[must_use]
pub fn is_well_formed(secret: &str) -> bool {
    secret.strip_prefix(SECRET_PREFIX).is_some_and(|rest| {
        (20..=100).contains(&rest.len())
            && rest
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    })
}

/// Characters of `secret` kept in the list to tell tokens apart.
#[must_use]
pub fn shown_prefix(secret: &str) -> String {
    secret.chars().take(SHOWN_PREFIX).collect()
}

/// A token joined with its owner.
#[derive(FromRow)]
pub struct Found {
    /// The owner.
    #[sqlx(flatten)]
    pub user: UserRow,
    /// The token id.
    pub token_id: Uuid,
    /// Stored scope texts.
    pub scopes: Vec<String>,
    /// The app this OAuth token belongs to; `None` for a personal token.
    pub app_id: Option<Uuid>,
    /// The last recorded use.
    pub token_last_used_at: Option<OffsetDateTime>,
}

impl Found {
    /// The scopes this token carries.
    #[must_use]
    pub fn scopes(&self) -> Vec<TokenScope> {
        parse_scopes(&self.scopes)
    }
}

fn parse_scopes(scopes: &[String]) -> Vec<TokenScope> {
    scopes
        .iter()
        .filter_map(|scope| TokenScope::parse(scope))
        .collect()
}

/// Records a use of token `id` when the last record is over a minute old. A separate statement
/// on the pool, never inside a transaction; a failure is logged and ignored.
async fn touch(db: &PgPool, id: Uuid, last_used: Option<OffsetDateTime>) {
    if last_used.is_none_or(|at| OffsetDateTime::now_utc() - at > TOUCH_AFTER)
        && let Err(err) = sqlx::query("update api_tokens set last_used_at = now() where id = $1")
            .bind(id)
            .execute(db)
            .await
    {
        tracing::warn!(error = %err, "cannot record an API token use");
    }
}

/// The token behind `secret` and its owner. `None` when the secret is unknown or revoked, or
/// the owner is suspended, deleted or remote, or the token has no owner (see [`find_app_only`]).
///
/// Records the use when the last record is over a minute old. That is a separate statement on
/// the pool, after the lookup, never inside a transaction; a failure is logged and ignored.
///
/// # Errors
///
/// Database failures of the lookup.
pub async fn find(db: &PgPool, secret: &str) -> Result<Option<Found>, AppError> {
    let sql = format!(
        "select {COLS}, t.id as token_id, t.scopes, t.app_id, t.last_used_at as token_last_used_at
           from api_tokens t join users u on u.id = t.user_id
          where t.token_hash = $1 and u.suspended_at is null and u.deleted_at is null
            and u.domain is null"
    );
    let found: Option<Found> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(hash_token(secret).as_slice())
        .fetch_optional(db)
        .await?;
    if let Some(found) = &found {
        touch(db, found.token_id, found.token_last_used_at).await;
    }
    Ok(found)
}

/// A `client_credentials` token: it belongs to an app and has no user.
#[derive(FromRow)]
pub struct AppFound {
    /// The token id.
    pub token_id: Uuid,
    /// The app it belongs to.
    pub app_id: Uuid,
    /// Stored scope texts.
    pub scopes: Vec<String>,
    /// The last recorded use.
    pub last_used_at: Option<OffsetDateTime>,
}

impl AppFound {
    /// The scopes this token carries.
    #[must_use]
    pub fn scopes(&self) -> Vec<TokenScope> {
        parse_scopes(&self.scopes)
    }
}

/// The app-only token behind `secret`; `None` when there is none.
///
/// # Errors
///
/// Database failures of the lookup.
pub async fn find_app_only(db: &PgPool, secret: &str) -> Result<Option<AppFound>, AppError> {
    let found: Option<AppFound> = sqlx::query_as(
        "select id as token_id, app_id, scopes, last_used_at
           from api_tokens where token_hash = $1 and user_id is null and app_id is not null",
    )
    .bind(hash_token(secret).as_slice())
    .fetch_optional(db)
    .await?;
    if let Some(found) = &found {
        touch(db, found.token_id, found.last_used_at).await;
    }
    Ok(found)
}

/// Whether token `id` still works: it exists and its owner is active.
///
/// # Errors
///
/// Database failures.
pub async fn is_live(db: &PgPool, id: Uuid) -> Result<bool, AppError> {
    Ok(sqlx::query_scalar(
        "select exists (select 1 from api_tokens t join users u on u.id = t.user_id
                         where t.id = $1 and u.suspended_at is null and u.deleted_at is null)",
    )
    .bind(id)
    .fetch_one(db)
    .await?)
}

#[derive(FromRow)]
struct Row {
    id: Uuid,
    name: String,
    scopes: Vec<String>,
    prefix: String,
    created_at: OffsetDateTime,
    last_used_at: Option<OffsetDateTime>,
}

impl From<Row> for TokenDto {
    fn from(row: Row) -> Self {
        Self {
            id: row.id,
            name: row.name,
            scopes: row
                .scopes
                .iter()
                .filter_map(|scope| TokenScope::parse(scope))
                .collect(),
            prefix: row.prefix,
            created_at: row.created_at,
            last_used_at: row.last_used_at,
        }
    }
}

/// The tokens of `user`, newest first. No secrets.
///
/// # Errors
///
/// Database failures.
pub async fn list(db: &PgPool, user: Uuid) -> Result<Vec<TokenDto>, AppError> {
    let rows: Vec<Row> = sqlx::query_as(
        "select id, name, scopes, prefix, created_at, last_used_at
           from api_tokens where user_id = $1 order by id desc",
    )
    .bind(user)
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

/// Makes a token for `user`. The answer holds the secret; nothing else ever does.
///
/// # Errors
///
/// `400 You can have up to 20 tokens.`; database failures.
pub async fn create(
    db: &PgPool,
    user: Uuid,
    name: &str,
    scopes: &[TokenScope],
) -> Result<CreatedTokenDto, AppError> {
    let secret = new_secret();
    let prefix = shown_prefix(&secret);
    let scope_texts: Vec<&str> = scopes.iter().map(|scope| scope.as_str()).collect();
    let id = Uuid::now_v7();
    let mut tx = db.begin().await?;
    // Two requests creating a token at once take turns on the owner's row, so the cap holds.
    sqlx::query("select id from users where id = $1 for update")
        .bind(user)
        .execute(&mut *tx)
        .await?;
    // OAuth tokens (`app_id` set) have their own cap and do not use up the personal ones.
    let held: i64 =
        sqlx::query_scalar("select count(*) from api_tokens where user_id = $1 and app_id is null")
            .bind(user)
            .fetch_one(&mut *tx)
            .await?;
    if held >= MAX_TOKENS {
        return Err(AppError::bad_request("You can have up to 20 tokens."));
    }
    let created_at: OffsetDateTime = sqlx::query_scalar(
        "insert into api_tokens (id, user_id, name, scopes, token_hash, prefix)
         values ($1, $2, $3, $4, $5, $6) returning created_at",
    )
    .bind(id)
    .bind(user)
    .bind(name)
    .bind(&scope_texts)
    .bind(hash_token(&secret).as_slice())
    .bind(&prefix)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(CreatedTokenDto {
        token: secret,
        info: TokenDto {
            id,
            name: name.to_owned(),
            scopes: scopes.to_vec(),
            prefix,
            created_at,
            last_used_at: None,
        },
    })
}

/// Deletes token `id` of `user`. `false` when there was none.
///
/// # Errors
///
/// Database failures.
pub async fn delete(db: &PgPool, user: Uuid, id: Uuid) -> Result<bool, AppError> {
    Ok(
        sqlx::query("delete from api_tokens where id = $1 and user_id = $2")
            .bind(id)
            .bind(user)
            .execute(db)
            .await?
            .rows_affected()
            > 0,
    )
}

/// Deletes every API token of `user`, personal and OAuth, and every OAuth code issued to them.
/// Passkeys and login sessions are not touched.
///
/// # Errors
///
/// Database failures.
pub async fn delete_all(db: &PgPool, user: Uuid) -> Result<(), AppError> {
    let mut tx = db.begin().await?;
    sqlx::query("delete from oauth_codes where user_id = $1")
        .bind(user)
        .execute(&mut *tx)
        .await?;
    sqlx::query("delete from api_tokens where user_id = $1")
        .bind(user)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn secrets_are_tuita_and_43_url_safe_characters() {
        let secret = new_secret();
        assert_eq!(secret.len(), SECRET_PREFIX.len() + 43);
        assert!(is_well_formed(&secret));
        assert_ne!(secret, new_secret());
    }

    #[test]
    fn only_our_shape_is_looked_up() {
        assert!(!is_well_formed("tuita_short"));
        assert!(!is_well_formed("3tt_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
        assert!(!is_well_formed("tuita_aaaaaaaaaaaaaaaaaaaa$aaaaaaaaaaa"));
        assert!(!is_well_formed(&format!("tuita_{}", "a".repeat(101))));
    }
}
