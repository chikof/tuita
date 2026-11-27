//! View counts: one count per viewer, post and day.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

/// Most post ids one report may carry; more are ignored.
pub const MAX_IDS: usize = 50;

/// Records that `viewer` saw `posts` today. A viewer counts once per post per day, and an author
/// never counts for their own post. Deleted and unknown posts are skipped silently.
///
/// # Errors
///
/// Database failures.
pub async fn record(db: &PgPool, viewer: Uuid, posts: &[Uuid]) -> Result<(), AppError> {
    let mut ids = posts.to_vec();
    ids.sort_unstable();
    ids.dedup();
    ids.truncate(MAX_IDS);
    if ids.is_empty() {
        return Ok(());
    }
    sqlx::query(
        "with seen as (
            insert into post_views (post_id, viewer, day)
            select p.id, 'u' || $1::text, current_date from posts p
             where p.id = any($2) and p.deleted_at is null and p.author_id <> $1
            on conflict do nothing
            returning post_id)
         update posts set views = coalesce(views, 0) + 1 where id in (select post_id from seen)",
    )
    .bind(viewer)
    .bind(&ids)
    .execute(db)
    .await?;
    Ok(())
}
