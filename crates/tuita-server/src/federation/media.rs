//! Copies the pictures and videos of remote accounts and posts into local storage, so readers
//! never contact another server.
//!
//! Every file goes through [`crate::net::SafeClient`] with the upload caps of the local API
//! (10 MB per image, `MAX_VIDEO_MB` per video) and, for images, the same decode and re-encode
//! that strips metadata. A file that fails any step is skipped and logged; callers never fail
//! on it.

use std::io::Cursor;
use std::time::Duration;

use futures_util::future::join_all;
use tokio::sync::Semaphore;
use tuita_core::ap::{self, Document};
use uuid::Uuid;

use crate::db::remote::insert_media;
use crate::dto::CropRect;
use crate::media::{
    MAX_ALT_CHARS, MAX_IMAGE_BYTES, NewMedia, Target, prepare_image, store_video_bytes,
};
use crate::net::{Fetch, Overflow};
use crate::state::AppState;

/// Most pictures of one post.
pub const MAX_ATTACHMENTS: usize = 4;
/// Longest an image download may take.
const IMAGE_TIMEOUT: Duration = Duration::from_secs(20);
/// Longest a video download may take.
const VIDEO_TIMEOUT: Duration = Duration::from_secs(90);
/// Most remote files downloaded and decoded at the same time, across the whole process. One
/// decode may allocate up to 256 MB and a video is buffered whole, so a sender that mentions a
/// local user in a flood of notes with large pictures cannot make the server hold them all at
/// once; the rest wait here.
pub const MAX_CONCURRENT_COPIES: usize = 4;
static COPIES: Semaphore = Semaphore::const_new(MAX_CONCURRENT_COPIES);

/// What a copied file becomes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Use {
    Avatar,
    Banner,
    Attachment,
}

/// Copies the avatar at `url` for remote account `owner`. `None` when anything fails.
pub async fn copy_avatar(state: &AppState, owner: Uuid, url: &str) -> Option<Uuid> {
    copy_image(state, owner, url, Use::Avatar, "").await
}

/// Copies the banner at `url` for remote account `owner`. `None` when anything fails.
pub async fn copy_banner(state: &AppState, owner: Uuid, url: &str) -> Option<Uuid> {
    copy_image(state, owner, url, Use::Banner, "").await
}

/// Copies up to [`MAX_ATTACHMENTS`] images of a post, or its first video, for remote account
/// `owner`; returns the stored uploads in order. A post holds images or one video: the first
/// usable attachment decides. Files that cannot be copied are left out.
pub async fn copy_attachments(state: &AppState, owner: Uuid, docs: &[Document]) -> Vec<Uuid> {
    let mut wanted: Vec<&Document> = Vec::new();
    let mut video = false;
    for doc in docs {
        let Some(url) = doc.url.as_deref() else {
            continue;
        };
        if wanted.iter().any(|seen| seen.url.as_deref() == Some(url)) {
            continue;
        }
        let kind = declared_video(doc);
        if kind == Declared::Audio {
            continue;
        }
        if kind == Declared::Video {
            if wanted.is_empty() {
                video = true;
                wanted.push(doc);
            }
            break;
        }
        wanted.push(doc);
        if wanted.len() == MAX_ATTACHMENTS {
            break;
        }
    }
    if video {
        let mut ids = Vec::new();
        if let Some(doc) = wanted.first()
            && let Some(id) = copy_video(state, owner, doc).await
        {
            ids.push(id);
        }
        return ids;
    }
    let copies = wanted.into_iter().map(|doc| async move {
        let url = doc.url.as_deref()?;
        copy_image(state, owner, url, Use::Attachment, &alt_of(doc)).await
    });
    join_all(copies).await.into_iter().flatten().collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Declared {
    Image,
    Video,
    Audio,
}

fn declared_video(doc: &Document) -> Declared {
    let media_type = doc.media_type.as_deref().unwrap_or("").to_ascii_lowercase();
    if media_type.starts_with("video/") || doc.kind == "Video" {
        Declared::Video
    } else if media_type.starts_with("audio/") || doc.kind == "Audio" {
        Declared::Audio
    } else {
        Declared::Image
    }
}

fn alt_of(doc: &Document) -> String {
    let alt = ap::strip_controls(doc.name.as_deref().unwrap_or(""));
    let alt = alt.trim();
    match alt.char_indices().nth(MAX_ALT_CHARS) {
        Some((end, _)) => alt.get(..end).unwrap_or(alt),
        None => alt,
    }
    .to_owned()
}

/// Downloads `url` after the checks every copy needs. `None` is logged.
pub(crate) async fn download(
    state: &AppState,
    url: &str,
    accept: &'static str,
    max_bytes: usize,
    timeout: Duration,
) -> Option<Vec<u8>> {
    if !ap::is_http_url(url) {
        tracing::debug!(url, "not copying media: not an http(s) address");
        return None;
    }
    if ap::url_authority(url).as_deref() == Some(state.config().domain()) {
        tracing::debug!(url, "not copying media: it claims to be ours");
        return None;
    }
    let fetched = state
        .safe_client()
        .get(
            url,
            Fetch {
                accept,
                max_bytes,
                timeout,
                overflow: Overflow::Fail,
            },
        )
        .await;
    match fetched {
        Ok(fetched) => Some(fetched.body),
        Err(err) => {
            tracing::debug!(url, error = %err, "cannot copy remote media");
            None
        }
    }
}

async fn copy_image(
    state: &AppState,
    owner: Uuid,
    url: &str,
    purpose: Use,
    alt: &str,
) -> Option<Uuid> {
    let _permit = COPIES.acquire().await.ok()?;
    let bytes = Box::pin(download(
        state,
        url,
        "image/*",
        MAX_IMAGE_BYTES,
        IMAGE_TIMEOUT,
    ))
    .await?;
    // A post attachment may be a video the author did not label as one.
    if purpose == Use::Attachment && image::guess_format(&bytes).is_err() {
        return store_video(state, owner, alt, bytes).await;
    }
    let target = match purpose {
        Use::Attachment => Target::Post,
        Use::Avatar => Target::Avatar(centered(&bytes, 1, 1)),
        Use::Banner => Target::Banner(centered(&bytes, 3, 1)),
    };
    match prepare_image(state, bytes, target).await {
        Ok(media) => insert(state, owner, alt, media).await,
        Err(err) => {
            tracing::debug!(url, error = %err, "remote image is not usable");
            None
        }
    }
}

fn video_cap(state: &AppState) -> usize {
    usize::from(state.config().max_video_mb.get()) * 1024 * 1024
}

async fn copy_video(state: &AppState, owner: Uuid, doc: &Document) -> Option<Uuid> {
    let _permit = COPIES.acquire().await.ok()?;
    let url = doc.url.as_deref()?;
    let bytes = Box::pin(download(
        state,
        url,
        "video/*",
        video_cap(state),
        VIDEO_TIMEOUT,
    ))
    .await?;
    store_video(state, owner, &alt_of(doc), bytes).await
}

async fn store_video(state: &AppState, owner: Uuid, alt: &str, bytes: Vec<u8>) -> Option<Uuid> {
    match store_video_bytes(state, bytes).await {
        Ok(media) => insert(state, owner, alt, media).await,
        Err(err) => {
            tracing::debug!(error = %err, "remote video is not usable");
            None
        }
    }
}

async fn insert(state: &AppState, owner: Uuid, alt: &str, mut media: NewMedia) -> Option<Uuid> {
    match insert_media(state.db(), owner, alt, &media).await {
        Ok(()) => {
            media.guard.keep();
            Some(media.id)
        }
        Err(err) => {
            tracing::warn!(error = %err, "cannot store a copied upload");
            None
        }
    }
}

/// The largest centered crop of `bytes`' image with aspect `w`:`h`. `None` when the header does
/// not parse, which the re-encode then refuses.
fn centered(bytes: &[u8], w: u32, h: u32) -> Option<CropRect> {
    let (width, height) = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .into_dimensions()
        .ok()?;
    crop_for(width, height, w, h)
}

fn crop_for(width: u32, height: u32, w: u32, h: u32) -> Option<CropRect> {
    if width == 0 || height == 0 {
        return None;
    }
    // The widest `w:h` box that fits: compare width * h with height * w without overflow.
    let (crop_w, crop_h) = if u64::from(width) * u64::from(h) >= u64::from(height) * u64::from(w) {
        let crop_w = u32::try_from(u64::from(height) * u64::from(w) / u64::from(h)).ok()?;
        (crop_w.max(1), height)
    } else {
        let crop_h = u32::try_from(u64::from(width) * u64::from(h) / u64::from(w)).ok()?;
        (width, crop_h.max(1))
    };
    Some(CropRect {
        x: (width - crop_w.min(width)) / 2,
        y: (height - crop_h.min(height)) / 2,
        w: crop_w.min(width),
        h: crop_h.min(height),
        rotate: 0,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn crops_are_centered_and_inside_the_image() {
        assert_eq!(
            crop_for(800, 600, 1, 1),
            Some(CropRect {
                x: 100,
                y: 0,
                w: 600,
                h: 600,
                rotate: 0
            })
        );
        assert_eq!(
            crop_for(300, 300, 3, 1),
            Some(CropRect {
                x: 0,
                y: 100,
                w: 300,
                h: 100,
                rotate: 0
            })
        );
        assert_eq!(crop_for(0, 5, 1, 1), None);
        let tiny = crop_for(1, 1, 3, 1).expect("crop");
        assert!(tiny.w >= 1 && tiny.h >= 1 && tiny.x + tiny.w <= 1 && tiny.y + tiny.h <= 1);
    }
}
