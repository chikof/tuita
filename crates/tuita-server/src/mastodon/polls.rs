//! Polls: `GET /polls/{id}` and `POST /polls/{id}/votes`. A poll's id is the id of its post, so
//! both routes check that the viewer may see that post first; a poll of a followers-only, direct
//! or blocked-author post answers `404`.

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;

use super::dto::{self, Poll};
use super::params::Form;
use crate::auth::CurrentUser;
use crate::config::Config;
use crate::db::{mastodon as queries, polls, posts};
use crate::dto::PostDto;
use crate::error::AppError;
use crate::state::AppState;

const MISSING: &str = "Poll not found.";
/// Most choices read from a vote; a poll has at most 4 answers.
const MAX_CHOICES: usize = 8;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/polls/{id}", get(show))
        .route("/polls/{id}/votes", post(vote))
}

fn poll_id(raw: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(raw).map_err(|_| AppError::Missing(MISSING))
}

/// The post behind poll `id` as `viewer` sees it; `404` when it has no poll or is hidden.
async fn poll_post(state: &AppState, viewer: Uuid, id: Uuid) -> Result<PostDto, AppError> {
    let author = queries::post_authors(state.db(), &[id])
        .await?
        .get(&id)
        .copied()
        .ok_or(AppError::Missing(MISSING))?;
    posts::load_one(state.db(), viewer, id, Some(author))
        .await?
        .filter(|post| post.poll.is_some())
        .ok_or(AppError::Missing(MISSING))
}

fn render(config: &Config, post: &PostDto) -> Result<Poll, AppError> {
    let emojis = dto::custom_emojis(config, &post.emojis);
    post.poll
        .as_ref()
        .map(|poll| dto::poll(post.id, poll, &emojis))
        .ok_or(AppError::Missing(MISSING))
}

async fn show(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Poll>, AppError> {
    let post = poll_post(&state, auth.user.id, poll_id(&id)?).await?;
    Ok(Json(render(state.config(), &post)?))
}

async fn vote(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    Form { params, .. }: Form,
) -> Result<Json<Poll>, AppError> {
    let id = poll_id(&id)?;
    let viewer = auth.user.id;
    poll_post(&state, viewer, id).await?;
    let raw = params.list("choices");
    if raw.is_empty() || raw.len() > MAX_CHOICES {
        return Err(AppError::unprocessable("Validation failed: Pick a choice."));
    }
    let choices = raw
        .iter()
        .map(|choice| choice.trim().parse::<u8>())
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|_| AppError::unprocessable("Validation failed: Pick a choice."))?;
    polls::vote(state.db(), state.config(), viewer, id, &choices)
        .await
        .map_err(|err| match err {
            AppError::BadRequest(message) | AppError::Conflict(message) => {
                AppError::Unprocessable(message)
            }
            other => other,
        })?;
    let post = poll_post(&state, viewer, id).await?;
    Ok(Json(render(state.config(), &post)?))
}
