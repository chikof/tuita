//! The public federation routes outside `/api/v1`: WebFinger and friends, the actor and its
//! collections, notes, the `/@handle` negotiation and `/oembed`. The JSON comes from
//! [`crate::federation::render`]. The inbox is [`inbox`]'s.

mod actor;
mod discovery;
mod embeds;
mod inbox;
pub(super) mod negotiate;
mod notes;

use axum::Router;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::Value;

use crate::db::ap::local_user;
use crate::db::users::UserRow;
use crate::error::AppError;
use crate::state::AppState;

/// `Content-Type` of ActivityPub documents.
const ACTIVITY_JSON: HeaderValue =
    HeaderValue::from_static("application/activity+json; charset=utf-8");
/// `Cache-Control` of public documents that change rarely.
const CACHE_PUBLIC: HeaderValue = HeaderValue::from_static("public, max-age=180");

/// The routes outside `/api/v1`.
pub(super) fn router() -> Router<AppState> {
    Router::new()
        .merge(discovery::router())
        .merge(actor::router())
        .merge(notes::router())
        .merge(embeds::oembed_router())
        .merge(inbox::router())
}

/// The route that lives under `/api/v1`: the Mastodon status JSON.
pub(super) fn api_router() -> Router<AppState> {
    embeds::status_router()
}

/// A `200` with `doc` as `application/activity+json`, cacheable for three minutes.
fn activity_json(doc: &Value) -> Response {
    activity_json_with(StatusCode::OK, doc, true)
}

fn activity_json_with(status: StatusCode, doc: &Value, cache: bool) -> Response {
    let body = serde_json::to_vec(doc).unwrap_or_default();
    let mut response = (status, body).into_response();
    let headers = response.headers_mut();
    headers.insert(CONTENT_TYPE, ACTIVITY_JSON);
    if cache {
        headers.insert(CACHE_CONTROL, CACHE_PUBLIC);
    }
    response
}

/// The account the public routes may show: local, not suspended, not deleted.
async fn live_user(state: &AppState, handle: &str) -> Result<UserRow, AppError> {
    local_user(state.db(), handle)
        .await?
        .filter(|user| user.suspended_at.is_none() && user.deleted_at.is_none())
        .ok_or(AppError::NotFound)
}
