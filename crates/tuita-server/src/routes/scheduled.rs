//! Scheduled posts: list, create, move, publish now, delete.

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::auth::{ApiJson, CurrentUser};
use crate::cursor::parse_id;
use crate::db::posts::load_one;
use crate::db::scheduled;
use crate::db::sessions::ClientMeta;
use crate::dto::{OkDto, PostDto, RescheduleRequest, ScheduleRequest, ScheduledPostDto};
use crate::error::AppError;
use crate::state::AppState;

const NOT_FOUND: &str = "Scheduled post not found.";

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/scheduled", get(list).post(create))
        .route(
            "/scheduled/{id}",
            axum::routing::patch(reschedule).delete(remove),
        )
        .route("/scheduled/{id}/publish", post(publish))
}

async fn list(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
) -> Result<Json<Vec<ScheduledPostDto>>, AppError> {
    Ok(Json(scheduled::list(state.db(), auth.user.id).await?))
}

async fn create(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    meta: ClientMeta,
    ApiJson(req): ApiJson<ScheduleRequest>,
) -> Result<Json<ScheduledPostDto>, AppError> {
    state.limits().check_schedule(auth.user.id)?;
    Ok(Json(
        scheduled::create(&state, &auth.user, meta.client, req).await?,
    ))
}

async fn reschedule(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    ApiJson(req): ApiJson<RescheduleRequest>,
) -> Result<Json<ScheduledPostDto>, AppError> {
    let id = parse_id(&id, NOT_FOUND)?;
    Ok(Json(
        scheduled::reschedule(state.db(), auth.user.id, id, req.publish_at).await?,
    ))
}

async fn publish(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<PostDto>, AppError> {
    let id = parse_id(&id, NOT_FOUND)?;
    state.limits().check_post(auth.user.id)?;
    let post = scheduled::publish_now(&state, &auth.user, id).await?;
    let post = load_one(state.db(), auth.user.id, post, None)
        .await?
        .ok_or(AppError::Missing(NOT_FOUND))?;
    Ok(Json(post))
}

async fn remove(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<OkDto>, AppError> {
    let id = parse_id(&id, NOT_FOUND)?;
    scheduled::delete(state.db(), auth.user.id, id).await?;
    Ok(Json(OkDto { ok: true }))
}
