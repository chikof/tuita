//! Invite codes for instances that take sign-ups by invite only.

use sqlx::{FromRow, PgConnection, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::audit::{self, Entry};
use crate::dto::InviteDto;
use crate::error::AppError;

/// Letters and digits without the confusable `0 O 1 I`. 32 symbols, so a random byte maps
/// onto them without bias.
const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
/// Longest code a sign-up form may send; longer is certainly not one of ours.
const MAX_CODE_LEN: usize = 32;
/// Most invites the admin list holds.
const LISTED: i64 = 100;
/// Longest expiry, in days.
const MAX_DAYS: i32 = 365;
/// Tries to find a code nobody has.
const TRIES: usize = 5;

/// The form a code is stored in: trimmed and upper-case. `None` when it cannot be a code.
#[must_use]
pub fn normalize(raw: &str) -> Option<String> {
    let code = raw.trim().to_ascii_uppercase();
    (!code.is_empty() && code.len() <= MAX_CODE_LEN).then_some(code)
}

fn new_code() -> String {
    let mut bytes = [0_u8; 10];
    rand::fill(&mut bytes);
    let mut code = String::with_capacity(11);
    for (index, byte) in bytes.iter().enumerate() {
        if index == 5 {
            code.push('-');
        }
        code.push(char::from(ALPHABET[usize::from(byte % 32)]));
    }
    code
}

/// Whether `code` (already normalised) can still be used: not revoked, not expired, not used up.
///
/// # Errors
///
/// Database failures.
pub async fn is_usable(db: &PgPool, code: &str) -> Result<bool, AppError> {
    Ok(sqlx::query_scalar(
        "select exists (select 1 from invites
                         where code = $1 and revoked_at is null and uses < max_uses
                           and (expires_at is null or expires_at > now()))",
    )
    .bind(code)
    .fetch_one(db)
    .await?)
}

/// Uses one slot of `code` (already normalised). `false` when it is unknown, revoked, expired
/// or used up. The check and the increment are one statement, so two sign-ups cannot both take
/// the last slot.
///
/// # Errors
///
/// Database failures.
pub async fn consume(conn: &mut PgConnection, code: &str) -> Result<bool, AppError> {
    let used = sqlx::query(
        "update invites set uses = uses + 1
          where code = $1 and revoked_at is null and uses < max_uses
            and (expires_at is null or expires_at > now())",
    )
    .bind(code)
    .execute(conn)
    .await?
    .rows_affected();
    Ok(used > 0)
}

#[derive(FromRow)]
struct Row {
    code: String,
    note: String,
    max_uses: i32,
    uses: i32,
    expires_at: Option<OffsetDateTime>,
    revoked: bool,
    created_at: OffsetDateTime,
    used_by: Vec<String>,
}

impl From<Row> for InviteDto {
    fn from(row: Row) -> Self {
        Self {
            code: row.code,
            note: row.note,
            max_uses: row.max_uses,
            uses: row.uses,
            expires_at: row.expires_at,
            revoked: row.revoked,
            created_at: row.created_at,
            used_by: row.used_by,
        }
    }
}

const SELECT: &str = "select i.code, i.note, i.max_uses, i.uses, i.expires_at,
        i.revoked_at is not null as revoked, i.created_at,
        array(select u.handle from users u
               where u.invited_with = i.code and u.deleted_at is null
               order by u.created_at limit 20) as used_by
     from invites i";

/// The newest hundred invites.
///
/// # Errors
///
/// Database failures.
pub async fn list(db: &PgPool) -> Result<Vec<InviteDto>, AppError> {
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{SELECT} order by i.created_at desc, i.code limit {LISTED}"
    )))
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

/// A validated invite.
#[derive(Debug)]
pub struct New {
    /// How many accounts may use it.
    pub max_uses: i32,
    /// Days until it expires; 0 for never.
    pub days: i32,
    /// A note for the admin.
    pub note: String,
}

/// Makes an invite and returns it.
///
/// # Errors
///
/// Database failures.
pub async fn create(conn: &mut PgConnection, actor: Uuid, new: New) -> Result<InviteDto, AppError> {
    for _ in 0..TRIES {
        let code = new_code();
        let inserted = sqlx::query(
            "insert into invites (code, created_by, note, max_uses, expires_at)
             values ($1, $2, $3, $4,
                     case when $5 > 0 then now() + make_interval(days => $5) end)
             on conflict (code) do nothing",
        )
        .bind(&code)
        .bind(actor)
        .bind(&new.note)
        .bind(new.max_uses.clamp(1, 1000))
        .bind(new.days.min(MAX_DAYS))
        .execute(&mut *conn)
        .await?
        .rows_affected();
        if inserted == 0 {
            continue;
        }
        let row: Row = sqlx::query_as(sqlx::AssertSqlSafe(format!("{SELECT} where i.code = $1")))
            .bind(&code)
            .fetch_one(&mut *conn)
            .await?;
        let uses = if new.max_uses == 1 { "use" } else { "uses" };
        audit::record(
            conn,
            Entry::new(Some(actor), "invite.create")
                .target(code)
                .text(format!("{} {uses}", new.max_uses.clamp(1, 1000))),
        )
        .await?;
        return Ok(row.into());
    }
    Err(AppError::Internal(anyhow::anyhow!(
        "no free invite code after {TRIES} tries"
    )))
}

/// Revokes an invite. `404 Invite not found.` when there is none.
///
/// # Errors
///
/// The `404`; database failures.
pub async fn revoke(conn: &mut PgConnection, actor: Uuid, code: &str) -> Result<(), AppError> {
    let revoked =
        sqlx::query("update invites set revoked_at = coalesce(revoked_at, now()) where code = $1")
            .bind(code)
            .execute(&mut *conn)
            .await?
            .rows_affected();
    if revoked == 0 {
        return Err(AppError::Missing("Invite not found."));
    }
    audit::record(conn, Entry::new(Some(actor), "invite.revoke").target(code)).await
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn codes_have_two_groups_of_five() {
        let code = new_code();
        assert_eq!(code.len(), 11);
        assert_eq!(code.as_bytes()[5], b'-');
        assert!(
            code.bytes()
                .filter(|b| *b != b'-')
                .all(|b| ALPHABET.contains(&b))
        );
        assert_ne!(code, new_code());
    }

    #[test]
    fn codes_are_compared_in_upper_case() {
        assert_eq!(normalize("  ab3de-fgh2j ").as_deref(), Some("AB3DE-FGH2J"));
        assert_eq!(normalize("   "), None);
        assert_eq!(normalize(&"x".repeat(33)), None);
    }
}
