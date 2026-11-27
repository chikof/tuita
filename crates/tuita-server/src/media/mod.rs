//! Uploaded media: validation, re-encoding without metadata, and storage in the
//! [`MediaStore`] (`DATA_DIR/media`, an S3-compatible bucket, or memory in tests).
//!
//! A stored object's key is `{id}.{ext}`; the `media` row's `path` is that key. An image is
//! put from memory. A video streams to a temp file under the OS temp dir (with the size cap),
//! then uploads. A guard deletes whatever object a failed request left behind.

mod scrub;
mod store;
mod transcode;
mod video;

use axum::extract::multipart::MultipartError;
use axum::http::StatusCode;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::dto::MediaKind;
use crate::error::AppError;
use crate::state::AppState;

pub(crate) use store::ObjectGuard;
pub use store::{MediaStore, Object, ObjectMeta, StoreError};
pub(crate) use transcode::{MAX_PIXELS, Target};
pub(crate) use video::{receive as receive_video, store_bytes as store_video_bytes};

/// Largest image upload, in bytes.
pub(crate) const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024;
/// Longest alt text, in characters.
pub(crate) const MAX_ALT_CHARS: usize = 1000;
/// The answer to a file that is no image or video the server takes.
pub(crate) const NOT_MEDIA: &str =
    "That file isn’t an image or video tuita can use (PNG, JPEG, GIF, WebP, MP4, WebM or MOV).";
/// The answer to an image over [`MAX_IMAGE_BYTES`].
pub(crate) const IMAGE_TOO_LARGE: &str = "Image is too large (max 10 MB).";

pub(crate) fn internal(err: impl Into<anyhow::Error>) -> AppError {
    AppError::Internal(err.into())
}

/// The answer to a video over `MAX_VIDEO_MB`.
pub(crate) fn video_too_large(state: &AppState) -> AppError {
    AppError::PayloadTooLarge(
        format!("Videos can be up to {} MB.", state.config().max_video_mb).into(),
    )
}

/// Maps a failed multipart read: a body past the route limit is `413`, anything else is a
/// broken upload.
// By value so it plugs into `map_err`.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn multipart_error(err: MultipartError) -> AppError {
    if err.status() == StatusCode::PAYLOAD_TOO_LARGE {
        AppError::PayloadTooLarge("Request too large.".into())
    } else {
        AppError::bad_request("The upload was interrupted.")
    }
}

/// An object in the store whose `media` row is not inserted yet.
#[derive(Debug)]
pub(crate) struct NewMedia {
    pub(crate) id: Uuid,
    pub(crate) kind: MediaKind,
    pub(crate) mime: &'static str,
    /// The object's key, `{id}.{ext}`.
    pub(crate) name: String,
    pub(crate) bytes: i64,
    pub(crate) width: Option<i32>,
    pub(crate) height: Option<i32>,
    pub(crate) guard: ObjectGuard,
}

impl From<transcode::Failure> for AppError {
    fn from(failure: transcode::Failure) -> Self {
        match failure {
            transcode::Failure::NotAnImage => Self::bad_request(NOT_MEDIA),
            transcode::Failure::BadCrop => Self::bad_request("Pick the area to crop."),
            transcode::Failure::Encode(err) => internal(err),
        }
    }
}

/// Decodes and re-encodes `bytes` for `target` on a blocking thread and stores the result.
pub(crate) async fn prepare_image(
    state: &AppState,
    bytes: Vec<u8>,
    target: Target,
) -> Result<NewMedia, AppError> {
    let encoded = tokio::task::spawn_blocking(move || transcode::process(&bytes, target))
        .await
        .map_err(internal)??;
    let id = Uuid::now_v7();
    let name = format!("{id}.{}", encoded.stored.ext());
    let len = encoded.bytes.len();
    state
        .media()
        .put(&name, encoded.bytes)
        .await
        .map_err(internal)?;
    Ok(NewMedia {
        id,
        kind: MediaKind::Image,
        mime: encoded.stored.mime(),
        guard: state.media().guard(&name),
        name,
        bytes: i64::try_from(len).unwrap_or(i64::MAX),
        width: i32::try_from(encoded.width).ok(),
        height: i32::try_from(encoded.height).ok(),
    })
}

/// Inserts the `media` row of `media`.
pub(crate) async fn insert(
    conn: &mut PgConnection,
    owner: Uuid,
    alt: &str,
    poster: Option<Uuid>,
    media: &NewMedia,
) -> Result<(), AppError> {
    sqlx::query(
        "insert into media (id, owner_id, kind, mime, path, bytes, width, height, alt, poster_id)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(media.id)
    .bind(owner)
    .bind(media.kind.as_str())
    .bind(media.mime)
    .bind(&media.name)
    .bind(media.bytes)
    .bind(media.width)
    .bind(media.height)
    .bind(alt)
    .bind(poster)
    .execute(conn)
    .await?;
    Ok(())
}

/// Re-encodes `bytes`, an image fetched for a link card, stores it in the media store and
/// inserts a `media` row owned by `owner` (the post's author). Returns the media id.
///
/// # Errors
///
/// `400` when `bytes` is not a PNG, JPEG, GIF or WebP image; I/O and database failures.
pub async fn store_card_image(
    state: &AppState,
    owner: Uuid,
    bytes: Vec<u8>,
) -> Result<Uuid, AppError> {
    let mut media = prepare_image(state, bytes, Target::Post).await?;
    let mut conn = state.db().acquire().await?;
    insert(&mut conn, owner, "", None, &media).await?;
    media.guard.keep();
    Ok(media.id)
}
