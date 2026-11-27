//! Media: `POST /api/v2/media` and `PUT /api/v1/media/{id}`.
//!
//! The upload is tuita's own pipeline ([`crate::routes::media::receive`]): the same size caps,
//! content sniffing, metadata stripping and re-encoding, the same per-account upload limits and
//! the same body ceiling. `file` is the upload, `description` the alt text and `thumbnail` a
//! video's poster; `focus` is ignored. The answer is `200` with the finished attachment, as
//! processing is synchronous. `purpose` is ignored: a Mastodon upload is always a post attachment.

use axum::extract::multipart::MultipartRejection;
use axum::extract::{Multipart, Path, State};
use axum::http::HeaderMap;
use axum::routing::{post, put};
use axum::{Json, Router};
use uuid::Uuid;

use super::ctx::Ctx;
use super::dto::{self, MediaAttachment};
use super::params::Form;
use crate::auth::CurrentUser;
use crate::db::media as rows;
use crate::error::AppError;
use crate::media::MAX_ALT_CHARS;
use crate::routes::media::{receive, upload_limit};
use crate::state::AppState;

const MISSING: &str = "Upload not found.";

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/media/{id}", put(update))
}

pub(super) fn router_v2(state: &AppState) -> Router<AppState> {
    Router::new().route("/media", post(upload).layer(upload_limit(state)))
}

async fn upload(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    headers: HeaderMap,
    parts: Result<Multipart, MultipartRejection>,
) -> Result<Json<MediaAttachment>, AppError> {
    let media = receive(&state, auth.user.id, &headers, parts, true).await?;
    Ok(Json(dto::media_attachment(
        &media,
        &Ctx::new(state.config()),
    )))
}

async fn update(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    Form { params, .. }: Form,
) -> Result<Json<MediaAttachment>, AppError> {
    let id = Uuid::parse_str(&id).map_err(|_| AppError::Missing(MISSING))?;
    let alt = params.get("description").map(str::trim);
    if alt.is_some_and(|alt| alt.chars().count() > MAX_ALT_CHARS) {
        return Err(AppError::unprocessable(
            "Validation failed: Description is too long.",
        ));
    }
    let updated = sqlx::query(
        "update media set alt = coalesce($3, alt)
          where id = $1 and owner_id = $2 and post_id is null
            and not exists (select 1 from post_media pm where pm.media_id = media.id)",
    )
    .bind(id)
    .bind(auth.user.id)
    .bind(alt)
    .execute(state.db())
    .await?
    .rows_affected();
    if updated == 0 {
        return Err(AppError::Missing(MISSING));
    }
    let media = rows::find(state.db(), id)
        .await?
        .ok_or(AppError::Missing(MISSING))?;
    Ok(Json(dto::media_attachment(
        &media,
        &Ctx::new(state.config()),
    )))
}
