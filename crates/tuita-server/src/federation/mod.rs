//! ActivityPub, the server half: ids of local objects, actor keys, signed fetches and the
//! errors remote lookups give.
//!
//! The pure parts (types, signatures, HTML) are in [`tuita_core::ap`]. Everything that reaches
//! another server goes through [`crate::net::SafeClient`].

pub mod fetch;
pub mod ids;
pub mod inbox;
pub mod ingest;
pub mod keys;
pub mod lookup;
pub mod mastodon;
pub mod media;
pub mod outbound;
pub mod render;

use crate::error::AppError;
use thiserror::Error;

/// Why a remote lookup or fetch failed. Each variant has the message the API shows.
#[derive(Debug, Error)]
pub enum FedError {
    /// The account or object does not exist, or is hidden from the caller: `404`.
    #[error("User not found.")]
    NotFound,
    /// The server could not be reached or refused us: `502`.
    #[error("Couldn’t reach {0} right now.")]
    Unreachable(Box<str>),
    /// The server answered with something that is not a usable actor or object: `502`.
    #[error("{0} sent something tuita can’t read.")]
    Unreadable(Box<str>),
    /// The caller used up its remote lookups: `429`.
    #[error("Too many requests. Try again in a few minutes.")]
    RateLimited {
        /// Seconds until the caller may try again.
        retry_after_secs: u32,
    },
    /// Anything unexpected on our side: `500`.
    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}
// 24: the tag plus the largest payload, a `Box<str>` (16); `anyhow::Error` is one pointer.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<FedError>() == 24);

impl From<sqlx::Error> for FedError {
    fn from(err: sqlx::Error) -> Self {
        Self::Internal(err.into())
    }
}

impl From<FedError> for AppError {
    fn from(err: FedError) -> Self {
        match err {
            FedError::NotFound => Self::Missing("User not found."),
            FedError::Unreachable(_) | FedError::Unreadable(_) => {
                Self::BadGateway(err.to_string().into())
            }
            FedError::RateLimited { retry_after_secs } => {
                Self::TooManyRequests { retry_after_secs }
            }
            FedError::Internal(cause) => Self::Internal(cause),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn errors_map_to_the_documented_statuses_and_messages() {
        use axum::response::IntoResponse as _;
        let reach: AppError = FedError::Unreachable("b.example".into()).into();
        assert_eq!(reach.to_string(), "Couldn’t reach b.example right now.");
        assert_eq!(reach.into_response().status(), 502);
        let read: AppError = FedError::Unreadable("b.example".into()).into();
        assert_eq!(
            read.to_string(),
            "b.example sent something tuita can’t read."
        );
        assert_eq!(read.into_response().status(), 502);
        let missing: AppError = FedError::NotFound.into();
        assert_eq!(missing.into_response().status(), 404);
    }
}
