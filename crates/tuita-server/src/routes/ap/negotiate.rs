//! Content negotiation for `/@{handle}` and `/@{handle}/{id}`: a request that asks for
//! ActivityPub JSON is sent on to the canonical `/users/...` URL; everything else is the web client's.

use axum::http::header::{LOCATION, VARY};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use tuita_core::ap::wants_activity_json;
use uuid::Uuid;

use crate::error::AppError;

/// Longest handle.
const MAX_HANDLE_LEN: usize = 15;

fn is_handle(text: &str) -> bool {
    (1..=MAX_HANDLE_LEN).contains(&text.len())
        && text.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// The answer to `GET {path}` with `accept`, when it is not the web client's:
///
/// - `/@{handle}` or `/@{handle}/{post id}` with an ActivityPub `Accept`: `302` to
///   `/users/{handle}` or `/users/{handle}/statuses/{post id}`.
/// - The same with a remote handle (`/@user@domain`): `404`. That profile is not served here,
///   and the web client's HTML would be the wrong answer to a federation client.
/// - Anything else: `None`.
pub(crate) fn respond(path: &str, accept: Option<&str>) -> Option<Response> {
    let rest = path.strip_prefix("/@")?;
    if !accept.is_some_and(wants_activity_json) {
        return None;
    }
    let (handle, id) = match rest.split_once('/') {
        None => (rest, None),
        Some((handle, id)) => (handle, Some(id)),
    };
    if handle.contains('@') {
        return Some(AppError::NotFound.into_response());
    }
    if !is_handle(handle) {
        return None;
    }
    let target = match id {
        None => format!("/users/{handle}"),
        Some(id) => format!("/users/{handle}/statuses/{}", Uuid::parse_str(id).ok()?),
    };
    let mut response = StatusCode::FOUND.into_response();
    let headers = response.headers_mut();
    headers.insert(LOCATION, HeaderValue::from_str(&target).ok()?);
    headers.insert(VARY, HeaderValue::from_static("accept"));
    Some(response)
}

#[cfg(test)]
mod test {
    use super::*;

    const AP: Option<&str> = Some("application/activity+json");
    const BROWSER: Option<&str> = Some("text/html,application/xhtml+xml");

    fn location(path: &str, accept: Option<&str>) -> Option<(u16, String)> {
        let response = respond(path, accept)?;
        let location = response
            .headers()
            .get(LOCATION)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        Some((response.status().as_u16(), location))
    }

    #[test]
    fn activity_requests_for_local_handles_redirect() {
        let id = "018f0000-0000-7000-8000-000000000001";
        assert_eq!(location("/@ada", AP), Some((302, "/users/ada".to_owned())));
        assert_eq!(
            location(
                &format!("/@ada/{id}"),
                Some("application/ld+json; profile=\"x\"")
            ),
            Some((302, format!("/users/ada/statuses/{id}")))
        );
    }

    #[test]
    fn browsers_and_other_paths_get_the_web_client() {
        assert!(respond("/@ada", BROWSER).is_none());
        assert!(respond("/@ada", None).is_none());
        assert!(respond("/explore", AP).is_none());
        assert!(respond("/@ada/not-a-uuid", AP).is_none());
        assert!(respond("/@ada/extra/segments", AP).is_none());
        assert!(respond("/@bad-handle", AP).is_none());
    }

    #[test]
    fn a_remote_handle_is_not_found_for_federation_clients() {
        let response = respond("/@ada@b.example", AP).expect("answered");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert!(respond("/@ada@b.example/018f0000-0000-7000-8000-000000000001", AP).is_some());
        assert!(respond("/@ada@b.example", BROWSER).is_none());
    }
}
