//! Video uploads: streamed to a temp file with a size cap, identified by their magic bytes, then
//! uploaded to the media store.

use std::path::{Path, PathBuf};

use axum::extract::multipart::Field;
use tokio::io::AsyncWriteExt as _;
use uuid::Uuid;

use super::{NOT_MEDIA, NewMedia, internal, multipart_error, scrub, video_too_large};
use crate::dto::MediaKind;
use crate::error::AppError;
use crate::state::AppState;

/// Bytes of the file start that decide the type.
const HEAD: usize = 12;

/// A video container the server stores.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Container {
    Mp4,
    Webm,
    Mov,
}

impl Container {
    fn ext(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Webm => "webm",
            Self::Mov => "mov",
        }
    }

    fn mime(self) -> &'static str {
        match self {
            Self::Mp4 => "video/mp4",
            Self::Webm => "video/webm",
            Self::Mov => "video/quicktime",
        }
    }
}

/// The container `head` (the first bytes of a file) belongs to: `ftyp` for MP4 and MOV, the EBML
/// header for WebM.
pub(crate) fn sniff(head: &[u8]) -> Option<Container> {
    if head.len() >= 12 && &head[4..8] == b"ftyp" {
        return Some(if &head[8..12] == b"qt  " {
            Container::Mov
        } else {
            Container::Mp4
        });
    }
    head.starts_with(&[0x1a, 0x45, 0xdf, 0xa3])
        .then_some(Container::Webm)
}

/// A file in the OS temp dir, removed when the guard drops.
#[derive(Debug)]
struct TempFile(PathBuf);

impl TempFile {
    fn new(id: Uuid) -> Self {
        Self(std::env::temp_dir().join(format!("tuita-upload-{id}.part")))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        // The file may never have been created; a leftover in the temp dir is harmless either way.
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Streams `field` into a temp file under the OS temp dir, at most `cap` bytes, scrubs the
/// metadata out of it in place, then uploads it to the media store under a name that shows the
/// container its content has.
///
/// # Errors
///
/// `413` past `cap`, `400` when the content is not a known container, I/O and store failures.
/// The temp file never outlives the call, and no object outlives an error.
pub(crate) async fn receive(
    state: &AppState,
    field: &mut Field<'_>,
    cap: u64,
) -> Result<NewMedia, AppError> {
    let id = Uuid::now_v7();
    let temp = TempFile::new(id);
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temp.path())
        .await
        .map_err(internal)?;
    let mut head = [0_u8; HEAD];
    let mut have = 0;
    let mut size = 0_u64;
    while let Some(chunk) = field.chunk().await.map_err(multipart_error)? {
        size += chunk.len() as u64;
        if size > cap {
            return Err(video_too_large(state));
        }
        let take = (HEAD - have).min(chunk.len());
        head[have..have + take].copy_from_slice(&chunk[..take]);
        have += take;
        file.write_all(&chunk).await.map_err(internal)?;
    }
    file.flush().await.map_err(internal)?;
    drop(file);
    let container = sniff(&head[..have]).ok_or_else(|| AppError::bad_request(NOT_MEDIA))?;
    let path = temp.path().to_owned();
    tokio::task::spawn_blocking(move || scrub::scrub_file(&path, container))
        .await
        .map_err(internal)?
        .map_err(internal)?;
    let name = format!("{id}.{}", container.ext());
    state
        .media()
        .put_file(&name, temp.path())
        .await
        .map_err(internal)?;
    Ok(NewMedia {
        id,
        kind: MediaKind::Video,
        mime: container.mime(),
        guard: state.media().guard(&name),
        name,
        bytes: i64::try_from(size).unwrap_or(i64::MAX),
        width: None,
        height: None,
    })
}

/// Stores `bytes`, a video another server sent, named after the container its content shows,
/// with its metadata scrubbed. The caller checked the size.
///
/// # Errors
///
/// `400` when the content is not a known container; store failures. No object outlives an
/// error.
pub(crate) async fn store_bytes(state: &AppState, bytes: Vec<u8>) -> Result<NewMedia, AppError> {
    let container = sniff(bytes.get(..HEAD).unwrap_or(&bytes))
        .ok_or_else(|| AppError::bad_request(NOT_MEDIA))?;
    let bytes = tokio::task::spawn_blocking(move || scrub::scrub_bytes(bytes, container))
        .await
        .map_err(internal)?;
    let id = Uuid::now_v7();
    let name = format!("{id}.{}", container.ext());
    let len = bytes.len();
    state.media().put(&name, bytes).await.map_err(internal)?;
    Ok(NewMedia {
        id,
        kind: MediaKind::Video,
        mime: container.mime(),
        guard: state.media().guard(&name),
        name,
        bytes: i64::try_from(len).unwrap_or(i64::MAX),
        width: None,
        height: None,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn a_temp_file_is_removed_when_its_guard_drops_and_a_missing_one_is_fine() {
        let temp = TempFile::new(Uuid::now_v7());
        let path = temp.path().to_owned();
        assert_eq!(path.parent(), Some(std::env::temp_dir().as_path()));
        std::fs::write(&path, b"partial").expect("create");
        drop(temp);
        assert!(!path.exists());
        drop(TempFile::new(Uuid::now_v7()));
    }

    #[test]
    fn containers_are_told_apart_by_their_first_bytes() {
        assert_eq!(sniff(b"\0\0\0\x18ftypmp42\0\0\0\0"), Some(Container::Mp4));
        assert_eq!(sniff(b"\0\0\0\x14ftypqt  \0\0\0\0"), Some(Container::Mov));
        assert_eq!(sniff(&[0x1a, 0x45, 0xdf, 0xa3, 0]), Some(Container::Webm));
        assert_eq!(sniff(b"<html>"), None);
    }
}
