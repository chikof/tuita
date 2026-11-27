//! Announcements: banners the admin puts up for everyone.

use sqlx::{FromRow, PgConnection, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::audit::{self, Entry};
use crate::dto::{AdminAnnouncementDto, AnnouncementDto, AnnouncementTone};
use crate::error::AppError;

/// Most announcements everyone sees at once.
const SHOWN: i64 = 3;
/// Most announcements the admin list holds.
const LISTED: i64 = 50;
/// Longest expiry, in days.
pub const MAX_DAYS: i32 = 365;

#[derive(FromRow)]
struct Row {
    id: Uuid,
    text: String,
    tone: String,
    link: String,
    active: bool,
    created_at: OffsetDateTime,
    expires_at: Option<OffsetDateTime>,
}

fn tone(raw: &str) -> AnnouncementTone {
    AnnouncementTone::parse(raw).unwrap_or(AnnouncementTone::Info)
}

impl From<Row> for AdminAnnouncementDto {
    fn from(row: Row) -> Self {
        Self {
            id: row.id,
            tone: tone(&row.tone),
            text: row.text,
            link: row.link,
            active: row.active,
            created_at: row.created_at,
            expires_at: row.expires_at,
        }
    }
}

const COLS: &str = "id, text, tone, link, active, created_at, expires_at";

/// The announcements everyone sees: active and not expired, newest first.
///
/// # Errors
///
/// Database failures.
pub async fn shown(db: &PgPool) -> Result<Vec<AnnouncementDto>, AppError> {
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "select {COLS} from announcements
          where active and (expires_at is null or expires_at > now())
          order by id desc limit {SHOWN}"
    )))
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| AnnouncementDto {
            id: row.id,
            tone: tone(&row.tone),
            text: row.text,
            link: row.link,
            created_at: row.created_at,
        })
        .collect())
}

/// The admin list: the newest fifty, ended ones included.
///
/// # Errors
///
/// Database failures.
pub async fn list(db: &PgPool) -> Result<Vec<AdminAnnouncementDto>, AppError> {
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "select {COLS} from announcements order by id desc limit {LISTED}"
    )))
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

/// A validated announcement.
#[derive(Debug)]
pub struct New {
    /// The text.
    pub text: String,
    /// The colour.
    pub tone: AnnouncementTone,
    /// The link, or empty.
    pub link: String,
    /// Days until it expires; 0 for never.
    pub days: i32,
}

/// Puts an announcement up.
///
/// # Errors
///
/// Database failures.
pub async fn create(
    conn: &mut PgConnection,
    actor: Uuid,
    new: New,
) -> Result<AdminAnnouncementDto, AppError> {
    let id = Uuid::now_v7();
    let row: Row = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "insert into announcements (id, text, tone, link, created_by, expires_at)
         values ($1, $2, $3, $4, $5, case when $6 > 0 then now() + make_interval(days => $6) end)
         returning {COLS}"
    )))
    .bind(id)
    .bind(&new.text)
    .bind(new.tone.as_str())
    .bind(&new.link)
    .bind(actor)
    .bind(new.days.min(MAX_DAYS))
    .fetch_one(&mut *conn)
    .await?;
    let summary: String = new.text.chars().take(80).collect();
    audit::record(
        conn,
        Entry::new(Some(actor), "announcement.create")
            .target(format!("#{id}"))
            .text(summary),
    )
    .await?;
    Ok(row.into())
}

/// Ends an announcement. `404 Announcement not found.` when there is none.
///
/// # Errors
///
/// The `404`; database failures.
pub async fn end(conn: &mut PgConnection, actor: Uuid, id: Uuid) -> Result<(), AppError> {
    let ended = sqlx::query("update announcements set active = false where id = $1")
        .bind(id)
        .execute(&mut *conn)
        .await?
        .rows_affected();
    if ended == 0 {
        return Err(AppError::Missing("Announcement not found."));
    }
    audit::record(
        conn,
        Entry::new(Some(actor), "announcement.end").target(format!("#{id}")),
    )
    .await
}
