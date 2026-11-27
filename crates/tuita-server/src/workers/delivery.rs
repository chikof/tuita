//! The delivery worker: sends the queued `ap_deliveries` rows to other servers.
//!
//! Every 2 seconds [`run_once`] claims up to 50 due rows and delivers up to 8 at a time. A row
//! is claimed by pushing its `next_attempt_at` a lease (60 s) into the future in one statement
//! that uses `FOR UPDATE SKIP LOCKED`; the claim commits before any network I/O, so no row
//! lock is held across a request and a crash simply lets the lease run out. Concurrent workers
//! (or concurrent calls of [`run_once`]) never claim the same row.
//!
//! What a delivery does to its row:
//!
//! | answer | row |
//! |---|---|
//! | `2xx` | deleted |
//! | `4xx` except `429` | deleted, with a warning (the other server will never accept it) |
//! | `429`, `3xx`, `5xx`, transport error, refused address, timeout | `attempts + 1`, retried after `min(2^attempts × 15 s, 6 h)`, `last_error` kept; deleted once `attempts` reaches 12 |
//!
//! Rows for a domain with a `suspend` block are deleted unsent.

use std::collections::HashSet;
use std::error::Error as _;
use std::time::Duration;

use futures_util::stream::{self, StreamExt as _};
use serde_json::Value;
use sqlx::FromRow;
use time::OffsetDateTime;
use tokio_util::sync::CancellationToken;
use tuita_core::ap::{self, CONTENT_TYPE_ACTIVITY};
use url::Url;
use uuid::Uuid;

use crate::error::AppError;
use crate::federation::keys;
use crate::net::{NetError, PostRequest};
use crate::state::AppState;

/// Gap between two passes.
const INTERVAL: Duration = Duration::from_secs(2);
/// Most rows one pass claims.
pub const BATCH: i64 = 50;
/// Most deliveries in flight at once.
pub const PARALLEL: usize = 8;
/// How long a claimed row stays out of other passes' reach. Longer than the longest delivery
/// (the 10 s request timeout), so a live pass never loses its rows; a crashed one is retried
/// after it.
pub const LEASE: Duration = Duration::from_secs(60);
/// A row that has failed this many times is dropped.
pub const MAX_ATTEMPTS: i32 = 12;
/// Longest wait of one request.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// Most of a response that is read; it is only kept for the error text.
pub const MAX_RESPONSE_BYTES: usize = 64 * 1024;
/// Longest stored `last_error`, in characters.
pub const MAX_ERROR_CHARS: usize = 300;
/// Longest backoff.
const MAX_BACKOFF: Duration = Duration::from_secs(6 * 60 * 60);

/// What a delivery's answer means for its row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The other server took it: delete the row.
    Delivered,
    /// The other server refused it for good (`4xx` other than `429`): delete the row.
    Refused,
    /// Try again later: `429`, `5xx`, anything else that is not `2xx`.
    Retry,
}

/// Classifies an HTTP status. A transport failure is always [`Outcome::Retry`].
#[must_use]
pub const fn classify(status: u16) -> Outcome {
    match status {
        200..=299 => Outcome::Delivered,
        429 => Outcome::Retry,
        400..=499 => Outcome::Refused,
        _ => Outcome::Retry,
    }
}

/// How long to wait after the failure that made the row's attempt count `attempts`:
/// `min(2^attempts × 15 s, 6 h)`.
#[must_use]
pub fn next_attempt(attempts: i32) -> Duration {
    // 2^11 × 15 s is already past 6 h; the cap on the shift keeps it from overflowing.
    let shift = u32::try_from(attempts.clamp(0, 20)).unwrap_or(20);
    Duration::from_secs(15_u64 << shift).min(MAX_BACKOFF)
}

/// `text` cut to [`MAX_ERROR_CHARS`] characters.
fn truncate(text: &str) -> String {
    text.chars().take(MAX_ERROR_CHARS).collect()
}

/// `err` and its causes, `: ` apart.
fn describe(err: &NetError) -> String {
    let mut text = err.to_string();
    let mut cause = err.source();
    while let Some(inner) = cause {
        text.push_str(": ");
        text.push_str(&inner.to_string());
        cause = inner.source();
    }
    text
}

/// A claimed row.
#[derive(FromRow)]
struct Claimed {
    id: Uuid,
    inbox_url: String,
    activity: Value,
    signer_id: Option<Uuid>,
    attempts: i32,
    domain: Option<String>,
}

/// The result of one delivery, ready to apply to the row.
struct Done {
    outcome: Outcome,
    error: Option<String>,
}

/// Claims the due rows: bumps their `next_attempt_at` by [`LEASE`] and returns them.
async fn claim(state: &AppState) -> Result<Vec<Claimed>, AppError> {
    Ok(sqlx::query_as(
        "update ap_deliveries set next_attempt_at = now() + $2 * interval '1 second'
          where id in (
              select id from ap_deliveries where next_attempt_at <= now()
               order by next_attempt_at, id limit $1 for update skip locked)
          returning id, inbox_url, activity, signer_id, attempts, domain",
    )
    .bind(BATCH)
    .bind(LEASE.as_secs_f64())
    .fetch_all(state.db())
    .await?)
}

/// Deletes the claimed rows whose domain has a `suspend` block and returns the ones to send.
async fn drop_suspended(state: &AppState, rows: Vec<Claimed>) -> Result<Vec<Claimed>, AppError> {
    let domains: Vec<&str> = rows
        .iter()
        .filter_map(|row| row.domain.as_deref())
        .collect();
    if domains.is_empty() {
        return Ok(rows);
    }
    let sql = format!(
        "select d from unnest($1::text[]) d
          where exists (select 1 from domain_blocks b
                         where b.severity = 'suspend' and {})",
        crate::db::domain_in_block("d", "b.domain")
    );
    let blocked: Vec<String> = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .bind(&domains)
        .fetch_all(state.db())
        .await?;
    if blocked.is_empty() {
        return Ok(rows);
    }
    let blocked: HashSet<String> = blocked.into_iter().collect();
    let (dropped, kept): (Vec<_>, Vec<_>) = rows.into_iter().partition(|row| {
        row.domain
            .as_ref()
            .is_some_and(|domain| blocked.contains(domain))
    });
    let ids: Vec<Uuid> = dropped.iter().map(|row| row.id).collect();
    sqlx::query("delete from ap_deliveries where id = any($1)")
        .bind(&ids)
        .execute(state.db())
        .await?;
    tracing::info!(
        dropped = ids.len(),
        "dropped deliveries to blocked instances"
    );
    Ok(kept)
}

/// Sends one row. Signing failures count as a failed attempt, like a network failure.
async fn send(state: &AppState, row: &Claimed) -> Done {
    match try_send(state, row).await {
        Ok(done) => done,
        Err(error) => Done {
            outcome: Outcome::Retry,
            error: Some(truncate(&error)),
        },
    }
}

async fn try_send(state: &AppState, row: &Claimed) -> Result<Done, String> {
    let url = Url::parse(&row.inbox_url).map_err(|err| format!("bad inbox url: {err}"))?;
    let body = serde_json::to_vec(&row.activity).map_err(|err| err.to_string())?;
    let key = match row.signer_id {
        Some(user) => keys::user_signing_key(state, user).await,
        None => keys::instance_signing_key(state).await,
    }
    .map_err(|err| format!("signing key: {err}"))?;
    // The key id is the one the activity's own actor implies, so an activity of an account whose
    // handle has since changed (a deleted account's `Delete`) still names the key remote servers
    // hold.
    let key_id = match (
        row.signer_id,
        row.activity.get("actor").and_then(Value::as_str),
    ) {
        (Some(_), Some(actor)) => format!("{actor}#main-key"),
        _ => key.key_id.clone(),
    };
    let signed = {
        let (private, url, body) = (key.private, url.clone(), body.clone());
        tokio::task::spawn_blocking(move || {
            ap::sign_post(&private, &key_id, &url, &body, OffsetDateTime::now_utc())
        })
        .await
        .map_err(|err| format!("signing task: {err}"))?
        .map_err(|err| format!("signing: {err}"))?
    };
    // `reqwest` sends `Host` itself (the same value, which is what was signed) and the client
    // sets `Content-Type`.
    let headers: Vec<(&str, String)> = signed
        .into_iter()
        .filter(|(name, _)| !matches!(*name, "host" | "content-type"))
        .collect();
    let posted = state
        .safe_client()
        .post(
            &row.inbox_url,
            PostRequest {
                body,
                content_type: CONTENT_TYPE_ACTIVITY,
                headers: &headers,
                max_response_bytes: MAX_RESPONSE_BYTES,
                timeout: REQUEST_TIMEOUT,
            },
        )
        .await;
    Ok(match posted {
        Ok(posted) => {
            let outcome = classify(posted.status);
            let error = (outcome != Outcome::Delivered).then(|| {
                let text = String::from_utf8_lossy(&posted.body);
                truncate(&format!("HTTP {}: {}", posted.status, text.trim()))
            });
            Done { outcome, error }
        }
        Err(err) => Done {
            outcome: Outcome::Retry,
            error: Some(truncate(&describe(&err))),
        },
    })
}

/// Applies the result to the row.
async fn settle(state: &AppState, row: &Claimed, done: Done) -> Result<(), AppError> {
    let error = done.error.as_deref().unwrap_or_default();
    match done.outcome {
        Outcome::Delivered => {
            delete(state, row.id).await?;
        }
        Outcome::Refused => {
            tracing::warn!(inbox = %row.inbox_url, error, "inbox refused a delivery; dropped");
            delete(state, row.id).await?;
        }
        Outcome::Retry => {
            let attempts = row.attempts.saturating_add(1);
            if attempts >= MAX_ATTEMPTS {
                tracing::warn!(inbox = %row.inbox_url, attempts, error, "delivery gave up");
                delete(state, row.id).await?;
            } else {
                sqlx::query(
                    "update ap_deliveries
                        set attempts = $2, last_error = $3,
                            next_attempt_at = now() + $4 * interval '1 second'
                      where id = $1",
                )
                .bind(row.id)
                .bind(attempts)
                .bind(error)
                .bind(next_attempt(attempts).as_secs_f64())
                .execute(state.db())
                .await?;
            }
        }
    }
    Ok(())
}

async fn delete(state: &AppState, id: Uuid) -> Result<(), AppError> {
    sqlx::query("delete from ap_deliveries where id = $1")
        .bind(id)
        .execute(state.db())
        .await?;
    Ok(())
}

/// Claims one batch of due rows and delivers it, up to [`PARALLEL`] at a time. Returns how
/// many rows it handled (delivered, dropped, or scheduled for a retry).
///
/// A database failure while claiming is logged and counts as zero rows. A failure to record one
/// row's outcome is logged too; that row comes due again when its lease runs out.
pub async fn run_once(state: &AppState) -> usize {
    let rows = match claim(state).await {
        Ok(claimed) => match drop_suspended(state, claimed).await {
            Ok(rows) => rows,
            Err(err) => {
                tracing::error!(error = %err, "could not drop deliveries to blocked instances");
                return 0;
            }
        },
        Err(err) => {
            tracing::error!(error = %err, "could not claim deliveries");
            return 0;
        }
    };
    let count = rows.len();
    stream::iter(rows)
        .for_each_concurrent(PARALLEL, |row| async move {
            let done = send(state, &row).await;
            if let Err(err) = settle(state, &row, done).await {
                tracing::error!(id = %row.id, error = %err, "could not record a delivery result");
            }
        })
        .await;
    count
}

/// Runs [`run_once`] every 2 seconds until `stop` is cancelled. A pass that is under way
/// finishes first, so in-flight deliveries are not abandoned between the request and its
/// bookkeeping.
pub async fn run(state: AppState, stop: CancellationToken) {
    let mut ticker = tokio::time::interval(INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            () = stop.cancelled() => return,
            _ = ticker.tick() => {}
        }
        let handled = run_once(&state).await;
        if handled > 0 {
            tracing::debug!(handled, "delivery pass");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn statuses_classify_as_the_policy_says() {
        for status in [200, 201, 202, 204] {
            assert_eq!(classify(status), Outcome::Delivered, "{status}");
        }
        for status in [400, 401, 403, 404, 410, 422] {
            assert_eq!(classify(status), Outcome::Refused, "{status}");
        }
        for status in [301, 302, 429, 500, 502, 503] {
            assert_eq!(classify(status), Outcome::Retry, "{status}");
        }
    }

    #[test]
    fn backoff_doubles_from_thirty_seconds_and_caps_at_six_hours() {
        assert_eq!(next_attempt(1), Duration::from_secs(30));
        assert_eq!(next_attempt(2), Duration::from_secs(60));
        assert_eq!(next_attempt(5), Duration::from_secs(480));
        assert_eq!(next_attempt(10), Duration::from_secs(15_360));
        assert_eq!(next_attempt(11), Duration::from_secs(6 * 3600));
        assert_eq!(next_attempt(12), Duration::from_secs(6 * 3600));
        assert_eq!(next_attempt(i32::MAX), Duration::from_secs(6 * 3600));
    }

    #[test]
    fn errors_are_cut_to_300_characters_not_bytes() {
        let long = "é".repeat(400);
        assert_eq!(truncate(&long).chars().count(), 300);
        assert_eq!(truncate("short"), "short");
    }
}
