//! The inbox: what other servers deliver to `/inbox` and `/users/{handle}/inbox`.
//!
//! [`receive`] runs the whole pipeline for one delivery and answers `Ok(())` for `202 Accepted`
//! (dropped and ignored activities included):
//!
//! 1. The body is parsed with depth and size guards (`400 Invalid JSON.`).
//! 2. `verify` checks the HTTP signature and binds the key to the activity's actor. A `suspend`
//!    domain is dropped before any fetch; nothing is changed before the signature holds.
//! 3. The verified instance's rate limit is counted, then the activity id is recorded in
//!    `ap_seen`, so a replay changes nothing. A failure on our side forgets the id again, so the
//!    sender's retry is processed.
//! 4. `dispatch` applies the activity. Every authorship rule lives there and in
//!    [`crate::federation::ingest`]: a remote account can only create, change and delete its own
//!    objects, and can never name a local id as its own.
//!
//! Shared-inbox deliveries are processed once: addressees come from the activity itself.

mod dispatch;
mod follow;
mod interact;
mod report;
mod verify;

use tracing::debug;
use tuita_core::ap::{self, Activity};
use url::Url;

use super::FedError;
use crate::db::inbound;
use crate::db::users::UserRow;
use crate::error::AppError;
use crate::state::AppState;

/// Largest delivery read, in bytes.
pub const MAX_BODY_BYTES: usize = 1024 * 1024;

/// One delivery as the route received it.
#[derive(Clone, Copy, Debug)]
pub struct Incoming<'a> {
    /// The request target as sent, such as `/users/ada/inbox`. The signature covers it.
    pub path_and_query: &'a str,
    /// Every header as `(name, value)`; `host` is the `Host` the sender addressed.
    pub headers: &'a [(&'a str, &'a str)],
    /// The exact body bytes. The `Digest` header is checked against these.
    pub body: &'a [u8],
}

/// Runs the pipeline for one delivery.
///
/// # Errors
///
/// `401` `Missing signature.`, `Invalid signature.` or `Unknown signing key.`; `400`
/// `Invalid JSON.`; `429` when the verified instance is over its limit; `500` for failures on
/// our side (the activity is not remembered, so a retry works).
pub async fn receive(state: &AppState, request: &Incoming<'_>) -> Result<(), AppError> {
    let value = ap::parse_document(request.body, MAX_BODY_BYTES)
        .map_err(|_| AppError::bad_request("Invalid JSON."))?;
    let activity: Activity =
        serde_json::from_value(value).map_err(|_| AppError::bad_request("Invalid JSON."))?;
    let actor = match verify::verify(state, request, &activity).await? {
        verify::Verdict::Drop => return Ok(()),
        verify::Verdict::Actor(actor) => *actor,
    };
    if let Some(domain) = actor.domain.as_deref() {
        state.limits().check_inbox_domain(domain)?;
    }
    let key = replay_key(&activity, &actor);
    if let Some(key) = key
        && !inbound::mark_seen(state.db(), key).await?
    {
        debug!(activity = key, "replayed activity dropped");
        return Ok(());
    }
    match dispatch::dispatch(state, &actor, &activity).await {
        Ok(()) => Ok(()),
        Err(err) => {
            if let Some(key) = key
                && let Err(unmark) = inbound::unmark_seen(state.db(), key).await
            {
                tracing::warn!(error = %unmark, "cannot forget an activity that failed");
            }
            Err(err)
        }
    }
}

/// The id that dedupes `activity`: its own id, when it is a URL on the actor's instance. An id
/// on another instance could be a victim's real activity id, so it is processed without
/// dedupe.
fn replay_key<'a>(activity: &'a Activity, actor: &UserRow) -> Option<&'a str> {
    let id = activity.id.as_deref()?;
    let origin = actor.ap_id.as_deref()?;
    (ap::is_http_url(id) && ap::same_authority(id, origin)).then_some(id)
}

/// `id` without its fragment, normalised by the URL parser.
fn without_fragment(id: &str) -> Option<String> {
    let mut url = Url::parse(id).ok()?;
    url.set_fragment(None);
    Some(url.into())
}

/// Whether two actor or object ids name the same thing: equal, or equal once parsed and
/// stripped of a fragment.
fn same_id(a: &str, b: &str) -> bool {
    a == b || matches!((without_fragment(a), without_fragment(b)), (Some(a), Some(b)) if a == b)
}

/// Turns the outcome of a lookup or ingest call into "carry on": a [`FedError`] that only says
/// the object is unusable (not found, unreadable, unreachable) is logged and dropped, because the
/// sender cannot fix it by retrying; only a failure on our side is returned.
fn soft<T>(what: &str, result: Result<T, FedError>) -> Result<Option<T>, AppError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(FedError::Internal(cause)) => Err(AppError::Internal(cause)),
        Err(other) => {
            debug!(what, reason = %other, "activity dropped");
            Ok(None)
        }
    }
}

/// `id` when it is short enough and a URL: safe to store as an activity id.
fn clean_id(id: Option<&str>) -> Option<&str> {
    id.filter(|id| ap::is_http_url(id))
}
