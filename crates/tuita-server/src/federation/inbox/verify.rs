//! HTTP signature verification of a delivery, and the binding of the signing key to the actor.

use time::OffsetDateTime;
use tracing::debug;
use tuita_core::ap::{
    self, Activity, ActivityKind, Actor, ParsedSignature, PublicKey, RequestParts, check_request,
    parse_signature_header, verify_signature,
};

use super::{Incoming, same_id, without_fragment};
use crate::db::remote;
use crate::db::users::UserRow;
use crate::error::AppError;
use crate::federation::{FedError, lookup};
use crate::state::AppState;

const MISSING: &str = "Missing signature.";
const INVALID: &str = "Invalid signature.";
const UNKNOWN_KEY: &str = "Unknown signing key.";

/// What verification decided.
pub(super) enum Verdict {
    /// The signature holds and the key belongs to the activity's actor.
    Actor(Box<UserRow>),
    /// Answer `202` and change nothing: a blocked instance, or the farewell of an account that is
    /// gone.
    Drop,
}

/// Verifies `request` against the actor of `activity`.
///
/// Nothing is fetched for a `suspend`-blocked instance, a request that fails the key-free
/// checks, or a key that is not on the actor's instance. Nothing is stored until a signature
/// holds (see [`resolve`]).
pub(super) async fn verify(
    state: &AppState,
    request: &Incoming<'_>,
    activity: &Activity,
) -> Result<Verdict, AppError> {
    let parts = RequestParts {
        method: "POST",
        path_and_query: request.path_and_query,
        headers: request.headers,
    };
    let header = parts.header("signature").ok_or(AppError::Denied(MISSING))?;
    let sig = parse_signature_header(&header).map_err(|_| AppError::Denied(INVALID))?;
    let key_url = without_fragment(&sig.key_id)
        .filter(|url| ap::is_http_url(url))
        .ok_or(AppError::Denied(INVALID))?;
    let key_actor = signing_actor(key_url, &activity.actor);

    for url in [key_actor.as_str(), activity.actor.as_str()] {
        if let Some(domain) = ap::url_authority(url)
            && remote::domain_suspended(state.db(), &domain).await?
        {
            return Ok(Verdict::Drop);
        }
    }
    check_request(&parts, &sig, request.body, OffsetDateTime::now_utc()).map_err(|err| {
        debug!(%err, "inbox request refused");
        AppError::Denied(INVALID)
    })?;
    if !ap::same_authority(&key_actor, &activity.actor) || !host_is_ours(state, &parts) {
        return Err(AppError::Denied(INVALID));
    }

    match resolve(state, &key_actor, activity, &parts, &sig).await {
        Ok(row) => bind(row, activity),
        Err(Reject::Invalid) => Err(AppError::Denied(INVALID)),
        Err(Reject::Fed(err)) => unresolved(&err, activity, &key_actor),
        Err(Reject::App(err)) => Err(err),
    }
}

/// Whether the signed `Host` is this instance's own authority (`PUBLIC_URL`). A delivery signed
/// for another host is a replay of something that was sent elsewhere.
fn host_is_ours(state: &AppState, parts: &RequestParts<'_>) -> bool {
    let public = state.config().public_url.as_ref();
    let scheme = if public.starts_with("http://") {
        "http"
    } else {
        "https"
    };
    let Some(host) = parts.header("host") else {
        return false;
    };
    ap::url_authority(&format!("{scheme}://{host}")) == ap::url_authority(public)
}

/// Why [`resolve`] found no actor.
enum Reject {
    /// The signature does not hold.
    Invalid,
    /// The key could not be resolved.
    Fed(FedError),
    /// A failure on our side.
    App(AppError),
}

impl From<AppError> for Reject {
    fn from(err: AppError) -> Self {
        Self::App(err)
    }
}

impl From<FedError> for Reject {
    fn from(err: FedError) -> Self {
        match err {
            FedError::Internal(cause) => Self::App(AppError::Internal(cause)),
            other => Self::Fed(other),
        }
    }
}

/// The account whose key signed the request, or why there is none.
///
/// Nothing is stored before a signature holds: an actor this server does not know is fetched
/// and checked in memory, its key verifies the request, and only then does the account get
/// created and its images copied. A known actor whose signature fails is fetched again once per
/// five minutes in case it rotated its key, and what that fetch says is stored only if the
/// signature holds with the new key.
async fn resolve(
    state: &AppState,
    key_actor: &str,
    activity: &Activity,
    parts: &RequestParts<'_>,
    sig: &ParsedSignature,
) -> Result<UserRow, Reject> {
    if let Some(row) = lookup::known_actor(state, key_actor).await? {
        if signature_holds(state, &row, parts, sig).await? {
            return Ok(row);
        }
        if state.limits().check_key_refresh(key_actor).is_err() {
            return Err(Reject::Invalid);
        }
        let fresh = lookup::fetch_current(state, &row).await?;
        if !key_verifies(&fresh, parts, sig) {
            return Err(Reject::Invalid);
        }
        return Ok(lookup::adopt(state, &fresh).await?);
    }
    let candidate = lookup::fetch_candidate(state, key_actor).await?;
    if !same_id(&candidate.id, &activity.actor) || !key_verifies(&candidate, parts, sig) {
        return Err(Reject::Invalid);
    }
    Ok(lookup::adopt(state, &candidate).await?)
}

/// Whether the key `actor` publishes verifies the signature.
fn key_verifies(actor: &Actor, parts: &RequestParts<'_>, sig: &ParsedSignature) -> bool {
    actor.public_key.as_ref().is_some_and(|key| {
        PublicKey::from_pem(&key.public_key_pem)
            .is_ok_and(|key| verify_signature(parts, sig, &key).is_ok())
    })
}

/// The actor a `keyId` belongs to. Mastodon's is `{actor}#main-key`; GoToSocial's is
/// `{actor}/main-key`, a sub-path of the actor that is not itself an actor. A key URL equal to
/// or below the activity's actor therefore names that actor; any other URL is fetched as it is.
/// The row that is found must still be the activity's actor ([`bind`]), so a key URL below
/// someone else's actor gains nothing.
fn signing_actor(key_url: String, actor: &str) -> String {
    let Some(actor) = without_fragment(actor) else {
        return key_url;
    };
    let below = key_url
        .strip_prefix(actor.as_str())
        .is_some_and(|rest| rest.starts_with('/'));
    if below { actor } else { key_url }
}

/// The actor of a verified request must be the one the activity names.
fn bind(row: UserRow, activity: &Activity) -> Result<Verdict, AppError> {
    match row.ap_id.as_deref() {
        Some(id) if same_id(id, &activity.actor) => Ok(Verdict::Actor(Box::new(row))),
        _ => Err(AppError::Denied(INVALID)),
    }
}

/// The key could not be resolved. An account that deleted itself signs its `Delete` with a key
/// nobody can fetch any more: that is answered `202`. Anything else is an unknown key.
fn unresolved(err: &FedError, activity: &Activity, key_actor: &str) -> Result<Verdict, AppError> {
    debug!(reason = %err, key = key_actor, "signing key not resolved");
    if matches!(err, FedError::NotFound) && is_farewell(activity, key_actor) {
        return Ok(Verdict::Drop);
    }
    Err(AppError::Denied(UNKNOWN_KEY))
}

/// A `Delete` whose actor and object are the account that owns the key.
fn is_farewell(activity: &Activity, key_actor: &str) -> bool {
    activity.kind == ActivityKind::Delete
        && same_id(&activity.actor, key_actor)
        && activity
            .object_id()
            .is_some_and(|object| same_id(object, key_actor))
}

/// Whether the stored key of `row` verifies the signature.
async fn signature_holds(
    state: &AppState,
    row: &UserRow,
    parts: &RequestParts<'_>,
    sig: &ParsedSignature,
) -> Result<bool, AppError> {
    let Some(pem) = remote::public_key_pem(state.db(), row.id).await? else {
        return Ok(false);
    };
    Ok(PublicKey::from_pem(&pem).is_ok_and(|key| verify_signature(parts, sig, &key).is_ok()))
}
