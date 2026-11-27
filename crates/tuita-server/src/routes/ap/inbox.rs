//! `POST /users/{handle}/inbox` and `POST /inbox`: deliveries from other servers. The pipeline is
//! [`crate::federation::inbox`]; this module reads the request and counts the address limit.

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::routing::post;

use super::live_user;
use crate::db::sessions::ClientMeta;
use crate::error::AppError;
use crate::federation::inbox::{self, Incoming, MAX_BODY_BYTES};
use crate::state::AppState;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/users/{handle}/inbox", post(personal))
        .route("/inbox", post(shared))
}

async fn personal(
    State(state): State<AppState>,
    meta: ClientMeta,
    Path(handle): Path<String>,
    uri: Uri,
    headers: HeaderMap,
    body: Body,
) -> Result<StatusCode, AppError> {
    count_address(&state, &meta)?;
    live_user(&state, &handle).await?;
    Box::pin(deliver(&state, &uri, &headers, body)).await
}

async fn shared(
    State(state): State<AppState>,
    meta: ClientMeta,
    uri: Uri,
    headers: HeaderMap,
    body: Body,
) -> Result<StatusCode, AppError> {
    count_address(&state, &meta)?;
    Box::pin(deliver(&state, &uri, &headers, body)).await
}

fn count_address(state: &AppState, meta: &ClientMeta) -> Result<(), AppError> {
    state
        .limits()
        .check_inbox(meta.ip.unwrap_or(std::net::Ipv4Addr::UNSPECIFIED.into()))
}

/// Reads the body (at most 1 MiB) and runs the pipeline; every outcome that is not an error is
/// `202` with an empty body.
async fn deliver(
    state: &AppState,
    uri: &Uri,
    headers: &HeaderMap,
    body: Body,
) -> Result<StatusCode, AppError> {
    let body = to_bytes(body, MAX_BODY_BYTES)
        .await
        .map_err(|_| AppError::PayloadTooLarge("Body too large.".into()))?;
    let authority = uri
        .authority()
        .map(|authority| authority.as_str().to_owned());
    let mut view: Vec<(&str, &str)> = headers
        .iter()
        .filter_map(|(name, value)| Some((name.as_str(), value.to_str().ok()?)))
        .collect();
    // HTTP/2 carries the host in the URI, not in a header.
    if !headers.contains_key("host")
        && let Some(authority) = &authority
    {
        view.push(("host", authority));
    }
    let path_and_query = uri
        .path_and_query()
        .map_or_else(|| uri.path(), |target| target.as_str());
    Box::pin(inbox::receive(
        state,
        &Incoming {
            path_and_query,
            headers: &view,
            body: &body,
        },
    ))
    .await?;
    Ok(StatusCode::ACCEPTED)
}
