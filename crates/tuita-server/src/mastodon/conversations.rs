//! Conversations: `GET /conversations`, `POST /conversations/{id}/read` and
//! `DELETE /conversations/{id}`. A conversation is a tuita direct-message thread, and its id is
//! the id of the other account; its `last_status` is the newest message as a `direct` status.
//!
//! `GET` and `POST .../read` share their path with tuita's own routes and are registered there
//! with [`super::by_credential`]. `DELETE` has no tuita twin and is registered here under the
//! parameter name tuita's route uses, so both sit on one path.
//!
//! A sealed message has no text the server can show, so its status carries a placeholder (see
//! [`super::dto::message_status`]); read it in tuita.
//!
//! Limits of tuita's data: the list pages by `max_id` only (`min_id` and `since_id` are ignored;
//! the cursor is the id of a conversation's newest message), and deleting a conversation deletes
//! the messages the viewer sent and blanks their text, since tuita cannot hide the other
//! account's messages from one side.

use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::delete;
use axum::{Json, Router};
use uuid::Uuid;

use super::ctx::Ctx;
use super::dto::{self, Conversation};
use super::page::Paging;
use super::timelines::respond;
use crate::auth::CurrentUser;
use crate::cursor::Limit;
use crate::db::mastodon_activity::conversation_with;
use crate::db::messages;
use crate::db::notifications::publish_badges;
use crate::db::users::user_ref;
use crate::dto::{ReadEventDto, ReadKind, StreamEvent};
use crate::error::AppError;
use crate::state::AppState;

const MISSING: &str = "Conversation not found.";

pub(super) fn router() -> Router<AppState> {
    // The parameter is named `handle` because tuita's `/conversations/{handle}` is the same path.
    Router::new().route("/conversations/{handle}", delete(remove))
}

fn conversation_id(raw: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(raw).map_err(|_| AppError::Missing(MISSING))
}

/// `GET /conversations` for an OAuth token.
pub(crate) async fn list(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    paging: Paging<Uuid>,
) -> Result<Response, AppError> {
    let limit = Limit::new(Some(paging.limit())).map_err(|_| AppError::NotFound)?;
    let page =
        messages::conversations(state.db(), auth.user.id, paging.upper().copied(), limit).await?;
    let link = paging.link(
        state.config(),
        page.items.first().map(|item| item.last.id),
        page.items.last().map(|item| item.last.id),
        page.next.is_some(),
    );
    let ids: Vec<Uuid> = page.items.iter().map(|item| item.user.id).collect();
    let ctx = Ctx::for_users(&state, &ids).await?;
    let viewer = user_ref(&auth.user);
    let body: Vec<Conversation> = page
        .items
        .iter()
        .map(|item| dto::conversation(item, &viewer, &ctx))
        .collect();
    Ok(respond(link, &body))
}

/// `POST /conversations/{id}/read` for an OAuth token.
pub(crate) async fn read(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Conversation>, AppError> {
    let other = conversation_id(&id)?;
    let viewer = auth.user.id;
    let mut found = conversation_with(state.db(), viewer, other)
        .await?
        .ok_or(AppError::Missing(MISSING))?;
    if messages::mark_read(state.db(), viewer, other).await? > 0 {
        state.hub().publish(
            viewer,
            StreamEvent::Read(ReadEventDto {
                kind: ReadKind::Messages,
            }),
        );
        publish_badges(&state, viewer).await;
    }
    found.unread = 0;
    let ctx = Ctx::for_users(&state, &[other]).await?;
    Ok(Json(dto::conversation(&found, &user_ref(&auth.user), &ctx)))
}

async fn remove(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let other = conversation_id(&id)?;
    let viewer = auth.user.id;
    conversation_with(state.db(), viewer, other)
        .await?
        .ok_or(AppError::Missing(MISSING))?;
    crate::db::mastodon_activity::delete_sent(state.db(), state.config(), viewer, other).await?;
    Ok(Json(serde_json::json!({})))
}
