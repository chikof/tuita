//! Database access for reports.

use sqlx::PgPool;
use uuid::Uuid;

use super::visible_post;
use crate::dto::ReportReason;
use crate::error::AppError;

/// The author of a post that is not deleted and that `viewer` may open. `None` for any other, so
/// a post the viewer cannot see answers like a missing one.
///
/// # Errors
///
/// Database failures.
pub async fn post_author(db: &PgPool, viewer: Uuid, post: Uuid) -> Result<Option<Uuid>, AppError> {
    let sql = format!(
        "select p.author_id from posts p where p.id = $1 and p.deleted_at is null and {}",
        visible_post("p", "$2")
    );
    Ok(sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .bind(post)
        .bind(viewer)
        .fetch_optional(db)
        .await?)
}

/// Files a report of `target` (and `post`, when given) by `reporter`. Returns `true` when the
/// reporter already has an open report on the same target and post; nothing is added then.
///
/// # Errors
///
/// Database failures.
pub async fn create(
    db: &PgPool,
    reporter: Uuid,
    target: Uuid,
    post: Option<Uuid>,
    reason: ReportReason,
    note: &str,
) -> Result<bool, AppError> {
    let duplicate: bool = sqlx::query_scalar(
        "select exists (select 1 from reports
            where reporter_id = $1 and target_id = $2 and post_id is not distinct from $3
              and status = 'open')",
    )
    .bind(reporter)
    .bind(target)
    .bind(post)
    .fetch_one(db)
    .await?;
    if !duplicate {
        sqlx::query(
            "insert into reports (id, reporter_id, target_id, post_id, reason, note)
             values ($1, $2, $3, $4, $5, $6)",
        )
        .bind(Uuid::now_v7())
        .bind(reporter)
        .bind(target)
        .bind(post)
        .bind(reason.as_str())
        .bind(note)
        .execute(db)
        .await?;
    }
    Ok(duplicate)
}
