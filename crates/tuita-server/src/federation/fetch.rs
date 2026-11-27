//! Signed `GET`s of remote actors and objects.

use std::time::Duration;

use serde_json::Value;
use time::OffsetDateTime;
use tuita_core::ap::{self, ACCEPT_ACTIVITY};
use url::Url;

use super::FedError;
use super::keys::instance_signing_key;
use crate::net::{Fetch, NetError, Overflow};
use crate::state::AppState;

/// Largest document read.
pub const MAX_DOCUMENT_BYTES: usize = 2 * 1024 * 1024;
/// Longest a whole fetch may take.
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

/// A fetched document and where it came from.
#[derive(Clone, Debug)]
pub struct Document {
    /// The URL the body came from, after redirects. An object's `id` must belong to its host.
    pub url: String,
    /// The parsed JSON object.
    pub json: Value,
}

/// The name of the server behind `url`, for messages.
fn server_of(url: &str) -> Box<str> {
    ap::url_authority(url)
        .unwrap_or_else(|| "that server".to_owned())
        .into()
}

fn map_error(err: &NetError, url: &str) -> FedError {
    match err {
        NetError::Status(404 | 410) => FedError::NotFound,
        NetError::TooLarge => FedError::Unreadable(server_of(url)),
        _ => FedError::Unreachable(server_of(url)),
    }
}

/// Fetches `url` as ActivityPub JSON, signed by the instance actor (`keyId` `{PUBLIC_URL}/actor#main-key`).
///
/// # Errors
///
/// [`FedError::NotFound`] for `404` or `410`; [`FedError::Unreachable`] for a refused address,
/// transport failure, timeout or other status; [`FedError::Unreadable`] for a body that is too
/// large or is not a JSON object.
pub async fn fetch_document(state: &AppState, url: &str) -> Result<Document, FedError> {
    let key = instance_signing_key(state).await?;
    let parsed = Url::parse(url).map_err(|_| FedError::Unreadable(server_of(url)))?;
    let signed = ap::sign_get(
        &key.private,
        &key.key_id,
        &parsed,
        ACCEPT_ACTIVITY,
        OffsetDateTime::now_utc(),
    )
    .map_err(|err| FedError::Internal(err.into()))?;
    // `host` is signed, and `reqwest` sends the same value itself.
    let headers: Vec<(&str, String)> = signed
        .into_iter()
        .filter(|(name, _)| *name != "host")
        .collect();
    let fetched = state
        .safe_client()
        .get_with(
            url,
            Fetch {
                accept: ACCEPT_ACTIVITY,
                max_bytes: MAX_DOCUMENT_BYTES,
                timeout: FETCH_TIMEOUT,
                overflow: Overflow::Fail,
            },
            &headers,
        )
        .await
        .map_err(|err| map_error(&err, url))?;
    let json = ap::parse_document(&fetched.body, MAX_DOCUMENT_BYTES)
        .map_err(|_| FedError::Unreadable(server_of(fetched.url.as_str())))?;
    Ok(Document {
        url: fetched.url.to_string(),
        json,
    })
}

/// Like [`fetch_document`], keeping only the JSON.
///
/// # Errors
///
/// As [`fetch_document`].
pub async fn fetch_json(state: &AppState, url: &str) -> Result<Value, FedError> {
    Ok(fetch_document(state, url).await?.json)
}
