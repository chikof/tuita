//! The one error type handlers return.

use axum::Json;
use axum::http::header::RETRY_AFTER;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::dto::ErrorBody;

/// Errors handlers can return. Rendered as `{"error": "<message>"}`.
#[derive(Debug, Error)]
pub enum AppError {
    /// The request is malformed or breaks a rule: `400 Bad Request`.
    #[error("{0}")]
    BadRequest(Box<str>),
    /// No session: `401 Unauthorized`.
    #[error("sign in required")]
    Unauthorized,
    /// Wrong handle or password at login: `401 Unauthorized`.
    #[error("Wrong handle or password.")]
    BadCredentials,
    /// A credential other than a password was refused (an API token, a passkey): `401
    /// Unauthorized` with a message that says which.
    #[error("{0}")]
    Denied(&'static str),
    /// The caller may not do this: `403 Forbidden`.
    #[error("{0}")]
    Forbidden(Box<str>),
    /// No such route: `404 Not Found`.
    #[error("not found")]
    NotFound,
    /// The named resource does not exist or is hidden from the caller: `404 Not Found`
    /// with a message that says which ("Post not found.").
    #[error("{0}")]
    Missing(&'static str),
    /// The request collides with existing state: `409 Conflict`.
    #[error("{0}")]
    Conflict(Box<str>),
    /// The request is well formed but a value is refused: `422 Unprocessable Entity`. The
    /// Mastodon API answers validation failures with it.
    #[error("{0}")]
    Unprocessable(Box<str>),
    /// The body is too large: `413 Payload Too Large`.
    #[error("{0}")]
    PayloadTooLarge(Box<str>),
    /// The path exists but not for this method: `405 Method Not Allowed`.
    #[error("method not allowed")]
    MethodNotAllowed,
    /// The body is not JSON: `415 Unsupported Media Type`.
    #[error("Expected JSON.")]
    UnsupportedMediaType,
    /// An upload of a kind the server does not take: `415` with a message that says which.
    #[error("{0}")]
    UnsupportedFile(&'static str),
    /// Another server could not be reached or sent nothing usable: `502 Bad Gateway`.
    #[error("{0}")]
    BadGateway(Box<str>),
    /// A rate limit was hit: `429` with `Retry-After`.
    #[error("Too many requests. Try again in a few minutes.")]
    TooManyRequests {
        /// Seconds until the caller may try again.
        retry_after_secs: u32,
    },
    /// Anything unexpected: `500`. The cause is logged, never sent to the client.
    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}
// 24: tag word + the largest payload, Box<str> or &'static str (16); the data-carrying variants leave no niche for the tag.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<AppError>() == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<Option<AppError>>() == std::mem::size_of::<AppError>());

impl AppError {
    /// A `400` with `message`.
    #[must_use]
    pub fn bad_request(message: impl Into<Box<str>>) -> Self {
        Self::BadRequest(message.into())
    }

    /// A `403` with `message`.
    #[must_use]
    pub fn forbidden(message: impl Into<Box<str>>) -> Self {
        Self::Forbidden(message.into())
    }

    /// A `409` with `message`.
    #[must_use]
    pub fn conflict(message: impl Into<Box<str>>) -> Self {
        Self::Conflict(message.into())
    }

    /// A `422` with `message`.
    #[must_use]
    pub fn unprocessable(message: impl Into<Box<str>>) -> Self {
        Self::Unprocessable(message.into())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self::Internal(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized | Self::BadCredentials | Self::Denied(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound | Self::Missing(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Unprocessable(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            Self::UnsupportedMediaType | Self::UnsupportedFile(_) => {
                StatusCode::UNSUPPORTED_MEDIA_TYPE
            }
            Self::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            Self::BadGateway(_) => StatusCode::BAD_GATEWAY,
            Self::TooManyRequests { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal(cause) => {
                tracing::error!(error = ?cause, "internal error");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        let retry_after = match &self {
            Self::TooManyRequests { retry_after_secs } => Some(*retry_after_secs),
            _ => None,
        };
        let body = ErrorBody {
            error: self.to_string(),
        };
        let mut response = (status, Json(body)).into_response();
        if let Some(secs) = retry_after {
            response
                .headers_mut()
                .insert(RETRY_AFTER, HeaderValue::from(secs));
        }
        response
    }
}
