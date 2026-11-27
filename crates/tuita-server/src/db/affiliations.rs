//! Affiliations: an organization invites accounts, optionally with a label such as
//! "Engineer". The invitation only shows once the account accepts, and either side may end it.
//! An account has at most one active affiliation.

use sqlx::{FromRow, PgConnection, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::emoji;
use super::notifications::{self, Pending};
use super::users::{COLS, UserRow, dtos, user_ref};
use crate::cursor::Limit;
use crate::dto::{
    AffiliateDto, AffiliationDto, AffiliationStatus, AffiliationsDto, NotificationKindDto, Page,
};
use crate::error::AppError;

/// Longest label, in characters.
pub const MAX_LABEL_CHARS: usize = 30;

#[derive(FromRow)]
struct Row {
    #[sqlx(rename = "aff_id")]
    id: Uuid,
    #[sqlx(rename = "aff_status")]
    status: String,
    #[sqlx(rename = "aff_label")]
    label: String,
    #[sqlx(rename = "aff_created_at")]
    created_at: OffsetDateTime,
    #[sqlx(flatten)]
    other: UserRow,
}

impl From<Row> for AffiliationDto {
    fn from(row: Row) -> Self {
        Self {
            id: row.id,
            status: AffiliationStatus::parse(&row.status).unwrap_or(AffiliationStatus::Pending),
            label: row.label,
            created_at: row.created_at,
            user: user_ref(&row.other),
        }
    }
}

/// Affiliations of one side. `side` is the column of the account to show (`user_id` for the
/// members of an organization, `org_id` for the organizations of a member) and `mine` the other.
async fn rows(
    db: &PgPool,
    side: &str,
    mine: &str,
    me: Uuid,
    status: Option<&str>,
) -> Result<Vec<AffiliationDto>, AppError> {
    let sql = format!(
        "select a.id as aff_id, a.status as aff_status, a.label as aff_label,
                a.created_at as aff_created_at, {COLS}
           from affiliations a join users u on u.id = a.{side}
          where a.{mine} = $1 and ($2::text is null or a.status = $2)
          order by (a.status = 'active') desc, a.created_at desc, a.id desc"
    );
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(me)
        .bind(status)
        .fetch_all(db)
        .await?;
    let mut items: Vec<AffiliationDto> = rows.into_iter().map(Into::into).collect();
    let mut cards: Vec<_> = items.iter_mut().map(|item| &mut item.user).collect();
    emoji::hydrate(db, &mut cards).await?;
    Ok(items)
}

/// What `GET /affiliations` shows `me`.
///
/// # Errors
///
/// Database failures.
pub async fn overview(db: &PgPool, me: &UserRow) -> Result<AffiliationsDto, AppError> {
    let is_org = me.is_org.unwrap_or(false);
    let members = if is_org {
        rows(db, "user_id", "org_id", me.id, None).await?
    } else {
        Vec::new()
    };
    let current = rows(db, "org_id", "user_id", me.id, Some("active"))
        .await?
        .into_iter()
        .next();
    let invites = rows(db, "org_id", "user_id", me.id, Some("pending")).await?;
    Ok(AffiliationsDto {
        org: is_org,
        members,
        current,
        invites,
    })
}

/// An organization invites an account. Returns the notification to deliver after the commit.
///
/// # Errors
///
/// `409` when the account is already affiliated with, or already invited by, the organization;
/// database failures.
pub async fn invite(
    conn: &mut PgConnection,
    org: Uuid,
    user: Uuid,
    label: &str,
) -> Result<Option<Pending>, AppError> {
    let existing: Option<String> =
        sqlx::query_scalar("select status from affiliations where org_id = $1 and user_id = $2")
            .bind(org)
            .bind(user)
            .fetch_optional(&mut *conn)
            .await?;
    match existing.as_deref() {
        Some("active") => {
            return Err(AppError::conflict(
                "They\u{2019}re already affiliated with you.",
            ));
        }
        Some(_) => {
            return Err(AppError::conflict(
                "They already have an invitation from you.",
            ));
        }
        None => {}
    }
    sqlx::query(
        "insert into affiliations (id, org_id, user_id, label, status) values ($1, $2, $3, $4, 'pending')",
    )
    .bind(Uuid::now_v7())
    .bind(org)
    .bind(user)
    .bind(label)
    .execute(&mut *conn)
    .await
    .map_err(|err| {
        if err
            .as_database_error()
            .and_then(|db| db.code())
            .is_some_and(|code| code == "23505")
        {
            AppError::conflict("They already have an invitation from you.")
        } else {
            err.into()
        }
    })?;
    notifications::notify(conn, user, org, NotificationKindDto::Affiliation, None).await
}

/// The account accepts an invitation, replacing its current affiliation.
///
/// # Errors
///
/// `404 Invitation not found.`; database failures.
pub async fn accept(db: &PgPool, user: Uuid, id: Uuid) -> Result<(), AppError> {
    let mut tx = db.begin().await?;
    // Two accepts at once take turns on the account's row, so the one-active rule holds.
    sqlx::query("select id from users where id = $1 for update")
        .bind(user)
        .execute(&mut *tx)
        .await?;
    let pending: Option<Uuid> = sqlx::query_scalar(
        "select id from affiliations where id = $1 and user_id = $2 and status = 'pending' for update",
    )
    .bind(id)
    .bind(user)
    .fetch_optional(&mut *tx)
    .await?;
    if pending.is_none() {
        return Err(AppError::Missing("Invitation not found."));
    }
    sqlx::query("delete from affiliations where user_id = $1 and status = 'active'")
        .bind(user)
        .execute(&mut *tx)
        .await?;
    sqlx::query("update affiliations set status = 'active', accepted_at = now() where id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

/// The organization changes the label of one of its affiliations.
///
/// # Errors
///
/// `404 Affiliation not found.`; database failures.
pub async fn relabel(db: &PgPool, org: Uuid, id: Uuid, label: &str) -> Result<(), AppError> {
    let changed = sqlx::query("update affiliations set label = $3 where id = $1 and org_id = $2")
        .bind(id)
        .bind(org)
        .bind(label)
        .execute(db)
        .await?
        .rows_affected();
    if changed == 0 {
        return Err(AppError::Missing("Affiliation not found."));
    }
    Ok(())
}

/// Either side ends an affiliation or withdraws an invitation.
///
/// # Errors
///
/// `404 Affiliation not found.`; database failures.
pub async fn remove(db: &PgPool, me: Uuid, id: Uuid) -> Result<(), AppError> {
    let gone =
        sqlx::query("delete from affiliations where id = $1 and (org_id = $2 or user_id = $2)")
            .bind(id)
            .bind(me)
            .execute(db)
            .await?
            .rows_affected();
    if gone == 0 {
        return Err(AppError::Missing("Affiliation not found."));
    }
    Ok(())
}

#[derive(FromRow)]
struct AffiliateRow {
    label: String,
    #[sqlx(flatten)]
    user: UserRow,
}

/// The accounts affiliated with `org`, most recent first. `offset` is the page cursor.
///
/// # Errors
///
/// Database failures.
pub async fn affiliates(
    db: &PgPool,
    viewer: Uuid,
    org: Uuid,
    offset: i64,
    limit: Limit,
) -> Result<Page<AffiliateDto>, AppError> {
    let sql = format!(
        "select a.label, {COLS}
           from affiliations a join users u on u.id = a.user_id
          where a.org_id = $1 and a.status = 'active'
            and u.suspended_at is null and u.deleted_at is null
          order by a.accepted_at desc, a.id desc limit $2 offset $3"
    );
    let rows: Vec<AffiliateRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(org)
        .bind(limit.fetch())
        .bind(offset)
        .fetch_all(db)
        .await?;
    let more = rows.len() > limit.size();
    let rows: Vec<AffiliateRow> = rows.into_iter().take(limit.size()).collect();
    let users: Vec<UserRow> = rows.iter().map(|row| row.user.clone()).collect();
    let cards = dtos(db, viewer, &users).await?;
    let items = cards
        .into_iter()
        .zip(rows)
        .map(|(user, row)| AffiliateDto {
            user,
            label: row.label,
        })
        .collect();
    Ok(Page {
        items,
        next: more.then(|| offset.saturating_add(i64::from(limit.get())).to_string()),
    })
}
