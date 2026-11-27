//! Notification list, mark-as-read and unread badges.

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use crate::auth::{ApiQuery, CurrentUser};
use crate::cursor::PageQuery;
use crate::db::notifications;
use crate::dto::{
    BadgesDto, NotificationFilter, NotificationGroupDto, OkDto, Page, ReadEventDto, ReadKind,
    StreamEvent,
};
use crate::error::AppError;
use crate::mastodon::{self, by_credential};
use crate::state::AppState;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/notifications",
            get(by_credential(list, mastodon::notifications::list)),
        )
        .route("/notifications/read", post(read))
        .route("/badges", get(badges))
}

#[derive(Debug, Default, Deserialize)]
struct ListQuery {
    filter: Option<String>,
    before: Option<String>,
    limit: Option<u32>,
}

async fn list(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiQuery(query): ApiQuery<ListQuery>,
) -> Result<Json<Page<NotificationGroupDto>>, AppError> {
    let filter = match query.filter.as_deref() {
        None => NotificationFilter::All,
        Some(raw) => NotificationFilter::parse(raw)
            .ok_or_else(|| AppError::bad_request("Invalid filter."))?,
    };
    let page = PageQuery {
        before: query.before,
        limit: query.limit,
    };
    let limit = page.limit()?;
    let before = page.id_cursor()?;
    Ok(Json(
        notifications::list(state.db(), auth.user.id, filter, before, limit).await?,
    ))
}

async fn read(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
) -> Result<Json<OkDto>, AppError> {
    notifications::mark_read(state.db(), auth.user.id).await?;
    state.hub().publish(
        auth.user.id,
        StreamEvent::Read(ReadEventDto {
            kind: ReadKind::Notifications,
        }),
    );
    notifications::publish_badges(&state, auth.user.id).await;
    Ok(Json(OkDto { ok: true }))
}

async fn badges(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
) -> Result<Json<BadgesDto>, AppError> {
    Ok(Json(
        notifications::badges(state.db(), auth.user.id, auth.user.is_staff()).await?,
    ))
}
