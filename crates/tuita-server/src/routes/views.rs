//! `POST /views`: posts the client saw for a moment.

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use crate::auth::{ApiJson, CurrentUser};
use crate::db::views;
use crate::dto::{OkDto, ViewsRequest};
use crate::error::AppError;
use crate::state::AppState;

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/views", post(record))
}

async fn record(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiJson(req): ApiJson<ViewsRequest>,
) -> Result<Json<OkDto>, AppError> {
    state.limits().check_views(auth.user.id)?;
    views::record(state.db(), auth.user.id, &req.ids).await?;
    Ok(Json(OkDto { ok: true }))
}
