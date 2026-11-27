//! CSRF guard: a cookie-authenticated mutation must come from the instance's own origin.

use axum::extract::{Request, State};
use axum::http::header::{COOKIE, ORIGIN};
use axum::http::{HeaderMap, Method};
use axum::middleware::Next;
use axum::response::{IntoResponse as _, Response};

use super::{ACCOUNTS_COOKIE, SESSION_COOKIE, cookie_value};
use crate::error::AppError;
use crate::state::AppState;

/// Middleware: a request that is not `GET`, `HEAD` or `OPTIONS` and carries a session cookie
/// needs an `Origin` header equal to the `PUBLIC_URL` origin, else `403 bad origin`.
///
/// Requests without a session cookie pass: they cannot act as a signed-in user, so there is
/// nothing to forge.
pub async fn guard(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let safe = matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    if !safe && has_session_cookie(request.headers()) {
        let origin = request
            .headers()
            .get(ORIGIN)
            .and_then(|value| value.to_str().ok());
        let same = origin
            .is_some_and(|origin| origin.trim_end_matches('/') == &*state.config().public_url);
        if !same {
            return AppError::forbidden("bad origin").into_response();
        }
    }
    next.run(request).await
}

fn has_session_cookie(headers: &HeaderMap) -> bool {
    headers.get_all(COOKIE).iter().any(|value| {
        value.to_str().is_ok_and(|raw| {
            cookie_value(raw, SESSION_COOKIE).is_some()
                || cookie_value(raw, ACCOUNTS_COOKIE).is_some()
        })
    })
}
