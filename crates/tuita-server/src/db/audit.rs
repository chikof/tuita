//! The audit log: one row per staff action, written in the transaction of the action itself.

use serde_json::json;
use sqlx::{FromRow, PgConnection, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::cursor::{Limit, page};
use crate::dto::{AuditEntryDto, Page};
use crate::error::AppError;

/// Detail text of rows written by `tuita admin ...`.
pub const COMMAND_LINE: &str = "command line";

/// A row to write.
#[derive(Clone, Debug)]
pub struct Entry {
    actor: Option<Uuid>,
    action: &'static str,
    target: String,
    text: String,
    user: Option<Uuid>,
}

impl Entry {
    /// A row for `action` by `actor`; `None` is the command line or the system.
    #[must_use]
    pub fn new(actor: Option<Uuid>, action: &'static str) -> Self {
        Self {
            actor,
            action,
            target: String::new(),
            text: if actor.is_none() {
                COMMAND_LINE.to_owned()
            } else {
                String::new()
            },
            user: None,
        }
    }

    /// What it happened to: `@handle`, `#<id>`, a code.
    #[must_use]
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = target.into();
        self
    }

    /// A sentence of detail. For a null actor the detail starts with `command line`.
    #[must_use]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        if text.is_empty() {
            return self;
        }
        self.text = if self.actor.is_none() {
            format!("{COMMAND_LINE}: {text}")
        } else {
            text
        };
        self
    }

    /// The account the row is about, for the account history.
    #[must_use]
    pub fn about(mut self, user: Uuid) -> Self {
        self.user = Some(user);
        self
    }
}

/// Writes `entry`.
///
/// # Errors
///
/// Database failures.
pub async fn record(conn: &mut PgConnection, entry: Entry) -> Result<(), AppError> {
    let detail = json!({
        "text": entry.text,
        "user": entry.user.map(|id| id.to_string()),
    });
    sqlx::query(
        "insert into audit_log (id, actor_id, action, target, detail) values ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::now_v7())
    .bind(entry.actor)
    .bind(entry.action)
    .bind(entry.target)
    .bind(detail)
    .execute(conn)
    .await?;
    Ok(())
}

#[derive(FromRow)]
struct Row {
    id: Uuid,
    actor: Option<String>,
    action: String,
    target: String,
    text: Option<String>,
    created_at: OffsetDateTime,
}

impl From<Row> for AuditEntryDto {
    fn from(row: Row) -> Self {
        Self {
            id: row.id,
            actor: row.actor,
            action: row.action,
            target: row.target,
            detail: row.text.unwrap_or_default(),
            created_at: row.created_at,
        }
    }
}

const SELECT: &str =
    "select a.id, u.handle as actor, a.action, a.target, a.detail->>'text' as text,
                             a.created_at
                        from audit_log a left join users u on u.id = a.actor_id";

/// One page of the log, newest first. `before` is the id of the last row of the previous page.
///
/// # Errors
///
/// Database failures.
pub async fn list(
    db: &PgPool,
    before: Option<Uuid>,
    limit: Limit,
) -> Result<Page<AuditEntryDto>, AppError> {
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{SELECT} where ($1::uuid is null or a.id < $1) order by a.id desc limit $2"
    )))
    .bind(before)
    .bind(limit.fetch())
    .fetch_all(db)
    .await?;
    let entries = rows.into_iter().map(AuditEntryDto::from).collect();
    Ok(page(entries, limit, |entry: &AuditEntryDto| {
        entry.id.to_string()
    }))
}

/// The newest rows about `user`, newest first.
///
/// # Errors
///
/// Database failures.
pub async fn history(db: &PgPool, user: Uuid, count: i64) -> Result<Vec<AuditEntryDto>, AppError> {
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{SELECT} where a.detail->>'user' = $1 order by a.id desc limit $2"
    )))
    .bind(user.to_string())
    .bind(count)
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(AuditEntryDto::from).collect())
}
