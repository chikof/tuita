//! The delivery queue: one `ap_deliveries` row per activity per inbox.

use std::collections::HashSet;

use serde_json::Value;
use sqlx::PgExecutor;
use tuita_core::ap;
use uuid::Uuid;

use crate::error::AppError;

/// Queues `activity` for `inbox_url`, signed by local account `signer` (the instance actor when
/// `None`). Returns whether a row was added.
///
/// The URL is stored in its normalised form (lower-case scheme and host, no default port, no
/// trailing dot), so the `domain` column, which the queue view groups by and a domain block
/// deletes by, is the same string [`ap::url_authority`] gives. An inbox that is not an http(s)
/// URL is not queued.
///
/// # Errors
///
/// Database failures.
pub async fn enqueue<'e>(
    db: impl PgExecutor<'e>,
    signer: Option<Uuid>,
    inbox_url: &str,
    activity: &Value,
) -> Result<bool, AppError> {
    let Some(inbox_url) = ap::normalize_url(inbox_url) else {
        tracing::warn!(
            inbox_url,
            "not queueing a delivery to an address that is not http(s)"
        );
        return Ok(false);
    };
    sqlx::query(
        "insert into ap_deliveries (id, inbox_url, activity, signer_id) values ($1, $2, $3, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(inbox_url)
    .bind(activity)
    .bind(signer)
    .execute(db)
    .await?;
    Ok(true)
}

/// Queues `activity` once for each distinct inbox of `inboxes`. Returns how many rows it added.
///
/// # Errors
///
/// Database failures.
pub async fn enqueue_many<'a>(
    conn: &mut sqlx::PgConnection,
    signer: Option<Uuid>,
    inboxes: impl IntoIterator<Item = &'a str>,
    activity: &Value,
) -> Result<usize, AppError> {
    let mut seen = HashSet::new();
    let mut added = 0;
    for inbox in inboxes {
        let Some(inbox) = ap::normalize_url(inbox) else {
            continue;
        };
        if seen.insert(inbox.clone()) && enqueue(&mut *conn, signer, &inbox, activity).await? {
            added += 1;
        }
    }
    Ok(added)
}
