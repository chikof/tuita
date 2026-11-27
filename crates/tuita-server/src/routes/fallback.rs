//! The gate in front of the web client: backend paths that fell through answer a JSON 404,
//! ActivityPub clients asking for `/@handle` get the actor, everything else reaches the web service.

use axum::extract::{Request, State};
use axum::http::Method;
use axum::http::header::ACCEPT;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::error::AppError;
use crate::state::AppState;

/// Paths served by the backend, never by the web client. A request under one of them that
/// reaches the fallback gets a JSON 404.
const BACKEND_PREFIXES: [&str; 10] = [
    "/api",
    "/media",
    "/.well-known",
    "/users",
    "/inbox",
    "/nodeinfo",
    "/actor",
    "/oauth",
    "/healthz",
    "/readyz",
];

/// Pages of the web client below a backend prefix: the OAuth consent card, which posts to `POST /oauth/authorize`.
const WEB_PAGES: [&str; 1] = ["/oauth/authorize"];

/// Wraps the web service: see the module docs.
pub(crate) async fn gate(State(_state): State<AppState>, request: Request, next: Next) -> Response {
    let path = request.uri().path();
    let is_get = matches!(*request.method(), Method::GET | Method::HEAD);
    if is_backend_path(path) || (!is_get && WEB_PAGES.contains(&path)) {
        return AppError::NotFound.into_response();
    }
    if is_get {
        let accept = request
            .headers()
            .get(ACCEPT)
            .and_then(|accept| accept.to_str().ok());
        if let Some(response) = super::ap::negotiate::respond(path, accept) {
            return response;
        }
    }
    next.run(request).await
}

fn is_backend_path(path: &str) -> bool {
    let under = |prefix: &str| {
        path.strip_prefix(prefix)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    };
    if WEB_PAGES.contains(&path) {
        return false;
    }
    BACKEND_PREFIXES.iter().any(|prefix| under(prefix))
        || (path.starts_with("/u/") && path.ends_with("/rss.xml"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn backend_paths_are_segment_aware() {
        for path in [
            "/api",
            "/api/v1/me",
            "/users/ada",
            "/u/ada/rss.xml",
            "/healthz",
        ] {
            assert!(is_backend_path(path), "{path}");
        }
        for path in ["/", "/apix", "/@ada", "/@ada/123", "/design", "/u/ada"] {
            assert!(!is_backend_path(path), "{path}");
        }
    }
}
