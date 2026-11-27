//! Custom emotes: the public list and the staff routes that add, recategorise and remove them.
//!
//! `GET /emojis` is public. `GET`/`POST /admin/emojis` and `PATCH`/`DELETE
//! /admin/emojis/{shortcode}` need a moderator or admin; every write is audited.

use axum::extract::multipart::{Field, MultipartRejection};
use axum::extract::{Multipart, Path, State};
use axum::routing::{get, patch};
use axum::{Json, Router};

use crate::auth::{ApiJson, Staff};
use crate::db::emoji;
use crate::dto::{EmojiDto, OkDto, PatchEmojiRequest};
use crate::error::AppError;
use crate::media::{NOT_MEDIA, Target, insert, multipart_error, prepare_image};
use crate::state::AppState;

/// Largest emote file, in bytes.
const MAX_FILE_BYTES: usize = 256 * 1024;
/// Most bytes of a text field.
const FIELD_CAP: usize = 4 * 1024;
/// Most fields in one upload.
const MAX_FIELDS: usize = 8;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/emojis", get(list))
        .route("/admin/emojis", get(admin_list).post(create))
        .route("/admin/emojis/{shortcode}", patch(update).delete(remove))
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<EmojiDto>>, AppError> {
    Ok(Json(emoji::list_local(state.db()).await?))
}

async fn admin_list(
    State(state): State<AppState>,
    Staff(_): Staff,
) -> Result<Json<Vec<EmojiDto>>, AppError> {
    Ok(Json(emoji::list_local(state.db()).await?))
}

#[derive(Default)]
struct Form {
    shortcode: String,
    category: Option<String>,
    file: Option<Vec<u8>>,
}

async fn create(
    State(state): State<AppState>,
    Staff(auth): Staff,
    parts: Result<Multipart, MultipartRejection>,
) -> Result<Json<EmojiDto>, AppError> {
    let parts = parts.map_err(|_| AppError::bad_request(NOT_MEDIA))?;
    let form = read_form(parts).await?;
    let shortcode = form.shortcode.trim();
    emoji::validate_shortcode(shortcode)?;
    emoji::clean_category(form.category.as_deref())?;
    let bytes = form.file.ok_or_else(|| AppError::bad_request(NOT_MEDIA))?;

    let mut media = prepare_image(&state, bytes, Target::Emote).await?;
    let mut tx = state.db().begin().await?;
    insert(
        &mut tx,
        auth.user.id,
        &format!(":{shortcode}:"),
        None,
        &media,
    )
    .await?;
    let created = emoji::create_local(
        &mut tx,
        auth.user.id,
        shortcode,
        form.category.as_deref(),
        media.id,
        &media.name,
    )
    .await?;
    tx.commit().await?;
    media.guard.keep();
    Ok(Json(created))
}

async fn update(
    State(state): State<AppState>,
    Staff(auth): Staff,
    Path(shortcode): Path<String>,
    ApiJson(body): ApiJson<PatchEmojiRequest>,
) -> Result<Json<EmojiDto>, AppError> {
    let mut tx = state.db().begin().await?;
    let updated =
        emoji::set_category(&mut tx, auth.user.id, &shortcode, body.category.as_deref()).await?;
    tx.commit().await?;
    Ok(Json(updated))
}

async fn remove(
    State(state): State<AppState>,
    Staff(auth): Staff,
    Path(shortcode): Path<String>,
) -> Result<Json<OkDto>, AppError> {
    let mut tx = state.db().begin().await?;
    let path = emoji::delete_local(&mut tx, auth.user.id, &shortcode).await?;
    tx.commit().await?;
    if let Err(err) = state.media().delete(&path).await {
        tracing::warn!(error = %err, path, "cannot remove the file of a deleted emote");
    }
    Ok(Json(OkDto { ok: true }))
}

/// Reads the multipart body field by field, each through a size cap.
async fn read_form(mut parts: Multipart) -> Result<Form, AppError> {
    let mut form = Form::default();
    for _ in 0..MAX_FIELDS {
        let Some(mut field) = parts.next_field().await.map_err(multipart_error)? else {
            return Ok(form);
        };
        let name = field.name().unwrap_or_default().to_owned();
        match name.as_str() {
            "file" => {
                if form.file.is_some() {
                    return Err(AppError::bad_request(NOT_MEDIA));
                }
                form.file = Some(read_capped(&mut field, MAX_FILE_BYTES, too_large).await?);
            }
            "shortcode" => form.shortcode = read_text(&mut field).await?,
            "category" => form.category = Some(read_text(&mut field).await?),
            _ => {
                read_capped(&mut field, FIELD_CAP, bad_field).await?;
            }
        }
    }
    Err(AppError::bad_request("The upload has too many fields."))
}

fn too_large() -> AppError {
    AppError::PayloadTooLarge("An emote can be up to 256 KB.".into())
}

fn bad_field() -> AppError {
    AppError::bad_request("The upload has an unreadable field.")
}

async fn read_text(field: &mut Field<'_>) -> Result<String, AppError> {
    let bytes = read_capped(field, FIELD_CAP, bad_field).await?;
    String::from_utf8(bytes).map_err(|_| bad_field())
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
