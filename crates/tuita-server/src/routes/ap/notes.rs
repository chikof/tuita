//! `GET /users/{handle}/statuses/{id}` and `.../activity`: the `Note` of a public or unlisted
//! post and the `Create` that carries it. Followers-only and direct posts are not served here.

use axum::Router;
use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::get;
use uuid::Uuid;

use super::activity_json;
use crate::db::ap::LocalPost;
use crate::error::AppError;
use crate::federation::render;
use crate::state::AppState;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/users/{handle}/statuses/{id}", get(note))
        .route("/users/{handle}/statuses/{id}/activity", get(activity))
}

/// The post `handle` and `id` name, when anyone may read it.
async fn open_post(state: &AppState, handle: &str, id: &str) -> Result<LocalPost, AppError> {
    let id = Uuid::parse_str(id).map_err(|_| AppError::NotFound)?;
    render::load(state, id)
        .await?
        .filter(|post| post.is_open() && post.author.handle.eq_ignore_ascii_case(handle))
        .ok_or(AppError::NotFound)
}

async fn note(
    State(state): State<AppState>,
    Path((handle, id)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let post = open_post(&state, &handle, &id).await?;
    Ok(activity_json(&render::note_document(state.config(), &post)))
}

async fn activity(
    State(state): State<AppState>,
    Path((handle, id)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let post = open_post(&state, &handle, &id).await?;
    Ok(activity_json(&render::create_value(state.config(), &post)))
}
