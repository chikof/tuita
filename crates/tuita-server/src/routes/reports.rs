//! `POST /reports`: report a post or an account to the moderators.

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use super::auth::text_field;
use crate::auth::{ApiJson, CurrentUser};
use crate::db::reports;
use crate::dto::{ReportAckDto, ReportRequest};
use crate::error::AppError;
use crate::federation::lookup::find_user;
use crate::state::AppState;

/// Longest report note, in characters.
const MAX_NOTE_CHARS: usize = 1000;

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/reports", post(create))
}

async fn create(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiJson(body): ApiJson<ReportRequest>,
) -> Result<Json<ReportAckDto>, AppError> {
    state.limits().check_report(auth.user.id)?;
    let db = state.db();
    let note = text_field(&body.note, "Details", 0, MAX_NOTE_CHARS)?;
    let handle = body.handle.as_deref().filter(|handle| !handle.is_empty());
    let (target, post) = match (body.post_id, handle) {
        (Some(post), _) => {
            let author = reports::post_author(db, auth.user.id, post)
                .await?
                .ok_or(AppError::Missing("Post not found."))?;
            (author, Some(post))
        }
        (None, Some(handle)) => (find_user(&state, &auth.user, handle).await?.id, None),
        (None, None) => return Err(AppError::bad_request("Nothing to report.")),
    };
    if target == auth.user.id {
        return Err(AppError::bad_request("You can\u{2019}t report yourself."));
    }
    let duplicate = reports::create(db, auth.user.id, target, post, body.reason, &note).await?;
    Ok(Json(ReportAckDto {
        ok: true,
        duplicate,
    }))
}
