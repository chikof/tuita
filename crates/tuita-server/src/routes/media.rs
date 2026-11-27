//! `POST /api/v1/media`, `PATCH /api/v1/media/{id}`, `DELETE /api/v1/me/{avatar,banner}` and
//! the public `GET /media/{id}.{ext}`.

use axum::extract::multipart::{Field, MultipartRejection};
use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::HeaderMap;
use axum::http::header::CONTENT_LENGTH;
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use uuid::Uuid;

use super::media_file;
use crate::auth::{ApiJson, CurrentUser};
use crate::cursor::parse_id;
use crate::db::media as rows;
use crate::db::users;
use crate::dto::{CropRect, MeDto, MediaDto, MediaPurpose, PatchMediaRequest};
use crate::error::AppError;
use crate::federation::outbound;
use crate::media::{
    IMAGE_TOO_LARGE, MAX_ALT_CHARS, MAX_IMAGE_BYTES, NOT_MEDIA, NewMedia, Target, insert,
    multipart_error, prepare_image, receive_video, video_too_large,
};
use crate::state::AppState;

const MIB: u64 = 1024 * 1024;
/// Most bytes of a text field (`alt`, `purpose`, `crop`).
const FIELD_CAP: usize = 16 * 1024;
/// Most fields in one upload.
const MAX_FIELDS: usize = 16;
/// What an upload may carry around its file: a poster and the small fields.
const SLACK: u64 = MAX_IMAGE_BYTES as u64 + 64 * 1024;
const ALT_TOO_LONG: &str = "Alt text must be at most 1000 characters.";

/// Axum's own ceiling for the upload routes: the largest video plus the poster and small fields.
/// The exact caps are still enforced while streaming.
pub(crate) fn upload_limit(state: &AppState) -> DefaultBodyLimit {
    DefaultBodyLimit::max((usize::from(state.config().max_video_mb.get()) + 11) * 1024 * 1024)
}

/// The upload routes under `/api/v1`.
pub(super) fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/media", post(upload).layer(upload_limit(state)))
        .route("/media/{id}", patch(patch_media))
        .route("/me/avatar", delete(clear_avatar))
        .route("/me/banner", delete(clear_banner))
}

/// The file route outside `/api/v1`.
pub(super) fn files() -> Router<AppState> {
    Router::new().route("/media/{name}", get(media_file::file))
}

/// The fields of an upload, as far as they were read.
#[derive(Default)]
struct Form {
    purpose: Option<MediaPurpose>,
    crop: Option<CropRect>,
    alt: String,
    image: Option<Vec<u8>>,
    video: Option<NewMedia>,
    poster: Option<Vec<u8>>,
}

async fn upload(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    headers: HeaderMap,
    parts: Result<Multipart, MultipartRejection>,
) -> Result<Json<MediaDto>, AppError> {
    Ok(Json(
        receive(&state, auth.user.id, &headers, parts, false).await?,
    ))
}

/// Reads, checks and stores one upload for `user`: the whole pipeline of `POST /media`. The
/// Mastodon `POST /api/v2/media` calls it with `post_only`, which ignores `purpose` (a
/// Mastodon client never sets an avatar through it). The Mastodon field names `description` and
/// `thumbnail` are `alt` and `poster`.
pub(crate) async fn receive(
    state: &AppState,
    user: Uuid,
    headers: &HeaderMap,
    parts: Result<Multipart, MultipartRejection>,
    post_only: bool,
) -> Result<MediaDto, AppError> {
    let parts = parts.map_err(|_| AppError::bad_request(NOT_MEDIA))?;
    let length = headers
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());
    let form = read_form(state, user, length, parts).await?;

    let alt = form.alt.trim();
    if alt.chars().count() > MAX_ALT_CHARS {
        return Err(AppError::bad_request(ALT_TOO_LONG));
    }
    let purpose = if post_only {
        MediaPurpose::Post
    } else {
        form.purpose.unwrap_or(MediaPurpose::Post)
    };
    let id = match (form.image, form.video) {
        (Some(bytes), None) => {
            let target = match purpose {
                MediaPurpose::Post => Target::Post,
                MediaPurpose::Avatar => Target::Avatar(form.crop),
                MediaPurpose::Banner => Target::Banner(form.crop),
            };
            store_image(state, user, alt, purpose, bytes, target).await?
        }
        (None, Some(video)) => {
            if purpose != MediaPurpose::Post {
                return Err(AppError::bad_request("Videos can’t be avatars or banners."));
            }
            store_video(state, user, alt, video, form.poster).await?
        }
        _ => return Err(AppError::bad_request(NOT_MEDIA)),
    };
    let media = rows::find(state.db(), id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(media)
}

/// Reads the multipart body field by field. Every field is read through a size cap, so no
/// part of the body is buffered or skipped without a limit.
async fn read_form(
    state: &AppState,
    user: Uuid,
    length: Option<u64>,
    mut parts: Multipart,
) -> Result<Form, AppError> {
    let mut form = Form::default();
    for _ in 0..MAX_FIELDS {
        let Some(mut field) = parts.next_field().await.map_err(multipart_error)? else {
            return Ok(form);
        };
        let name = field.name().unwrap_or_default().to_owned();
        match name.as_str() {
            "file" => {
                if form.image.is_some() || form.video.is_some() {
                    return Err(AppError::bad_request(NOT_MEDIA));
                }
                read_file(state, user, length, &mut field, &mut form).await?;
            }
            "poster" | "thumbnail" => {
                if !declared_image(&field) {
                    return Err(AppError::bad_request(NOT_MEDIA));
                }
                let bytes = read_capped(&mut field, MAX_IMAGE_BYTES, image_too_large).await?;
                form.poster = Some(bytes);
            }
            "alt" | "description" => {
                form.alt = read_text(&mut field, || AppError::bad_request(ALT_TOO_LONG)).await?
            }
            "purpose" => {
                let text = read_text(&mut field, bad_purpose).await?;
                form.purpose = Some(MediaPurpose::parse(text.trim()).ok_or_else(bad_purpose)?);
            }
            "crop" => {
                let text = read_text(&mut field, bad_crop).await?;
                form.crop = Some(serde_json::from_str(&text).map_err(|_| bad_crop())?);
            }
            _ => {
                read_capped(&mut field, FIELD_CAP, || AppError::bad_request(NOT_MEDIA)).await?;
            }
        }
    }
    Err(AppError::bad_request("The upload has too many fields."))
}

/// Reads the `file` part by its declared type: images into memory up to 10 MiB, videos to disk
/// up to `MAX_VIDEO_MB`. The content decides the real type later.
async fn read_file(
    state: &AppState,
    user: Uuid,
    length: Option<u64>,
    field: &mut Field<'_>,
    form: &mut Form,
) -> Result<(), AppError> {
    let declared = declared_type(field);
    match declared.as_str() {
        "image/jpeg" | "image/png" | "image/webp" | "image/gif" => {
            if length.is_some_and(|len| len > MAX_IMAGE_BYTES as u64 + SLACK) {
                return Err(image_too_large());
            }
            state.limits().check_upload(user)?;
            form.image = Some(read_capped(field, MAX_IMAGE_BYTES, image_too_large).await?);
        }
        "video/mp4" | "video/webm" | "video/quicktime" => {
            let cap = u64::from(state.config().max_video_mb.get()) * MIB;
            if length.is_some_and(|len| len > cap + SLACK) {
                return Err(video_too_large(state));
            }
            state.limits().check_video(user)?;
            form.video = Some(receive_video(state, field, cap).await?);
        }
        other if other.starts_with("video/") => {
            return Err(AppError::UnsupportedFile(
                "Videos must be MP4, WebM or MOV.",
            ));
        }
        _ => return Err(AppError::bad_request(NOT_MEDIA)),
    }
    Ok(())
}

fn declared_type(field: &Field<'_>) -> String {
    field
        .content_type()
        .and_then(|declared| declared.split(';').next())
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

fn declared_image(field: &Field<'_>) -> bool {
    matches!(
        declared_type(field).as_str(),
        "image/jpeg" | "image/png" | "image/webp" | "image/gif"
    )
}

fn image_too_large() -> AppError {
    AppError::PayloadTooLarge(IMAGE_TOO_LARGE.into())
}

fn bad_purpose() -> AppError {
    AppError::bad_request("Unknown upload purpose.")
}

fn bad_crop() -> AppError {
    AppError::bad_request("Pick the area to crop.")
}

/// Reads `field` into memory, failing with `over()` once it passes `cap` bytes.
async fn read_capped(
    field: &mut Field<'_>,
    cap: usize,
    over: impl Fn() -> AppError,
) -> Result<Vec<u8>, AppError> {
    let mut buf = Vec::new();
    while let Some(chunk) = field.chunk().await.map_err(multipart_error)? {
        if buf.len() + chunk.len() > cap {
            return Err(over());
        }
        buf.extend_from_slice(&chunk);
    }
    Ok(buf)
}

async fn read_text(field: &mut Field<'_>, over: impl Fn() -> AppError) -> Result<String, AppError> {
    let bytes = read_capped(field, FIELD_CAP, over).await?;
    String::from_utf8(bytes)
        .map_err(|_| AppError::bad_request("The upload has an unreadable field."))
}

/// The column of `users` that holds an avatar or banner upload.
fn profile_column(purpose: MediaPurpose) -> Option<&'static str> {
    match purpose {
        MediaPurpose::Post => None,
        MediaPurpose::Avatar => Some("avatar_id"),
        MediaPurpose::Banner => Some("banner_id"),
    }
}

async fn store_image(
    state: &AppState,
    user: Uuid,
    alt: &str,
    purpose: MediaPurpose,
    bytes: Vec<u8>,
    target: Target,
) -> Result<Uuid, AppError> {
    let mut media = prepare_image(state, bytes, target).await?;
    let mut tx = state.db().begin().await?;
    insert(&mut tx, user, alt, None, &media).await?;
    if let Some(column) = profile_column(purpose) {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "update users set {column} = $1 where id = $2"
        )))
        .bind(media.id)
        .bind(user)
        .execute(&mut *tx)
        .await?;
        outbound::profile_updated(&mut tx, state.config(), user).await?;
    }
    tx.commit().await?;
    media.guard.keep();
    Ok(media.id)
}

async fn store_video(
    state: &AppState,
    user: Uuid,
    alt: &str,
    mut video: NewMedia,
    poster: Option<Vec<u8>>,
) -> Result<Uuid, AppError> {
    let mut poster = match poster {
        Some(bytes) => Some(prepare_image(state, bytes, Target::Post).await?),
        None => None,
    };
    let mut tx = state.db().begin().await?;
    if let Some(poster) = &poster {
        insert(&mut tx, user, "", None, poster).await?;
    }
    insert(&mut tx, user, alt, poster.as_ref().map(|p| p.id), &video).await?;
    tx.commit().await?;
    video.guard.keep();
    if let Some(poster) = &mut poster {
        poster.guard.keep();
    }
    Ok(video.id)
}

async fn patch_media(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    ApiJson(body): ApiJson<PatchMediaRequest>,
) -> Result<Json<MediaDto>, AppError> {
    let id = parse_id(&id, "Upload not found.")?;
    let alt = body.alt.trim();
    if alt.chars().count() > MAX_ALT_CHARS {
        return Err(AppError::bad_request(ALT_TOO_LONG));
    }
    let updated = sqlx::query(
        "update media set alt = $3
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
        return Err(AppError::Missing("Upload not found."));
    }
    let media = rows::find(state.db(), id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(media))
}

async fn clear_avatar(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
) -> Result<Json<MeDto>, AppError> {
    clear_profile_media(&state, auth.user.id, MediaPurpose::Avatar).await
}

async fn clear_banner(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
) -> Result<Json<MeDto>, AppError> {
    clear_profile_media(&state, auth.user.id, MediaPurpose::Banner).await
}

/// Unsets the account's avatar or banner. The upload stays until the cleanup worker reaps it.
async fn clear_profile_media(
    state: &AppState,
    user: Uuid,
    purpose: MediaPurpose,
) -> Result<Json<MeDto>, AppError> {
    if let Some(column) = profile_column(purpose) {
        let mut tx = state.db().begin().await?;
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "update users set {column} = null where id = $1"
        )))
        .bind(user)
        .execute(&mut *tx)
        .await?;
        outbound::profile_updated(&mut tx, state.config(), user).await?;
        tx.commit().await?;
    }
    let row = users::by_id(state.db(), user)
        .await?
        .ok_or(AppError::Unauthorized)?;
    Ok(Json(
        users::me_dto(state.db(), &row, state.config().domain()).await?,
    ))
}
