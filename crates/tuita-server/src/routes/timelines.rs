//! Timelines, the new-posts counter and bookmarks.

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::{ApiQuery, CurrentUser};
use crate::cursor::{Limit, PageQuery, parse_offset};
use crate::db::timeline;
use crate::dto::{Feed, NewCountDto, Page, PostDto};
use crate::error::AppError;
use crate::mastodon::{self, by_credential};
use crate::state::AppState;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/timelines/following", get(following))
        .route("/timelines/foryou", get(foryou))
        .route("/timelines/{feed}/new", get(new_count))
        .route(
            "/bookmarks",
            get(by_credential(bookmarks, mastodon::timelines::bookmarks)),
        )
}

async fn following(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiQuery(query): ApiQuery<PageQuery>,
) -> Result<Json<Page<PostDto>>, AppError> {
    let limit = query.limit()?;
    let before = query.id_cursor()?;
    Ok(Json(
        timeline::following(state.db(), auth.user.id, before, limit).await?,
    ))
}

/// Query of `GET /timelines/foryou`: the For you feed pages by offset, not by `before`.
#[derive(Debug, Default, Deserialize)]
struct ForYouQuery {
    cursor: Option<String>,
    limit: Option<u32>,
}

async fn foryou(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiQuery(query): ApiQuery<ForYouQuery>,
) -> Result<Json<Page<PostDto>>, AppError> {
    let limit = Limit::new(query.limit)?;
    let offset = parse_offset(query.cursor.as_deref())?;
    Ok(Json(
        timeline::foryou(state.db(), auth.user.id, offset, limit).await?,
    ))
}

#[derive(Debug, Default, Deserialize)]
struct SinceQuery {
    since: Option<String>,
}

async fn new_count(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(feed): Path<String>,
    ApiQuery(query): ApiQuery<SinceQuery>,
) -> Result<Json<NewCountDto>, AppError> {
    let feed = Feed::parse(&feed).ok_or_else(|| AppError::bad_request("Unknown timeline."))?;
    let since = match query.since.as_deref() {
        None => Uuid::nil(),
        Some(raw) => Uuid::parse_str(raw).map_err(|_| AppError::bad_request("Invalid since."))?,
    };
    let count = timeline::new_count(state.db(), auth.user.id, feed, since).await?;
    Ok(Json(NewCountDto { count }))
}

async fn bookmarks(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiQuery(query): ApiQuery<PageQuery>,
) -> Result<Json<Page<PostDto>>, AppError> {
    let limit = query.limit()?;
    let before = query.edge_cursor()?;
    Ok(Json(
        timeline::bookmarks(state.db(), auth.user.id, before, limit).await?,
    ))
}
