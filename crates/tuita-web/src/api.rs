//! The in-process client of the JSON API: pages read and change data by calling the backend
//! router with the browser's own cookies and headers, so every rule stays in one place.

use std::net::SocketAddr;

use axum::{
    Router,
    body::{Body, to_bytes},
    extract::ConnectInfo,
    http::{Method, Request, Response, StatusCode, header},
};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;
use topcoat::{
    context::{Cx, context},
    router::{
        HeaderValue,
        request::{extensions, headers},
        response::response_headers,
    },
};
use tower::ServiceExt;
use tuita_server::AppState;

/// Largest API answer a page reads.
const MAX_BODY: usize = 16 * 1024 * 1024;

/// Request headers copied from the browser's request to the backend.
const FORWARDED: [&str; 7] = [
    "cookie",
    "origin",
    "user-agent",
    "accept-language",
    "x-forwarded-for",
    "cf-ipcountry",
    "x-request-id",
];

/// The application state and the backend router, installed once in the router context.
pub(crate) struct Web {
    pub(crate) state: AppState,
    pub(crate) backend: Router,
}

/// A non-2xx answer of the API, or a failure to reach it.
#[derive(Debug, Clone, Error)]
#[error("api {status}: {message}")]
pub(crate) struct ApiError {
    pub(crate) status: StatusCode,
    /// The API's `error` text, else the reason phrase.
    pub(crate) message: String,
    /// Seconds from `Retry-After` on a 429.
    pub(crate) retry_after: Option<u64>,
}

impl ApiError {
    /// A failure on our side of the call (bad body, unreadable answer).
    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
            retry_after: None,
        }
    }
}

/// Everything except `A-Za-z0-9 - _ . ! ~ * ' ( )`: what `encodeURIComponent` leaves alone.
const COMPONENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'!')
    .remove(b'~')
    .remove(b'*')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')');

/// `encodeURIComponent`.
pub(crate) fn enc(segment: &str) -> String {
    utf8_percent_encode(segment, COMPONENT).to_string()
}

/// Everything except `A-Za-z0-9 * - . _`: what `URLSearchParams` leaves alone.
const FORM: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'*')
    .remove(b'-')
    .remove(b'.')
    .remove(b'_');

/// `?a=1&b=2` for the entries that are present and not empty, or an empty string; the same
/// serialisation as `URLSearchParams`.
pub(crate) fn query(pairs: &[(&str, Option<&str>)]) -> String {
    let form = |text: &str| {
        utf8_percent_encode(text, FORM)
            .to_string()
            .replace("%20", "+")
    };
    let text = pairs
        .iter()
        .filter_map(|(key, value)| match value {
            Some(value) if !value.is_empty() => Some(format!("{}={}", form(key), form(value))),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("&");
    if text.is_empty() {
        text
    } else {
        format!("?{text}")
    }
}

fn web(cx: &Cx) -> &Web {
    context::<Web>(cx)
}

/// Calls `method` on `/api/v1{path}` and returns the raw answer, whatever its status.
///
/// # Errors
///
/// Fails only when the request cannot be built or the backend does not answer.
pub(crate) async fn send_raw(
    cx: &Cx,
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
) -> Result<Response<axum::body::Bytes>, ApiError> {
    send_raw_at(cx, method, &format!("/api/v1{path}"), body).await
}

/// Like [`send_raw`] for a backend route outside `/api/v1`; `uri` is the absolute path, e.g.
/// `/oauth/authorize`.
///
/// # Errors
///
/// Fails only when the request cannot be built or the backend does not answer.
pub(crate) async fn send_raw_at(
    cx: &Cx,
    method: Method,
    uri: &str,
    body: Option<serde_json::Value>,
) -> Result<Response<axum::body::Bytes>, ApiError> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::ACCEPT, "application/json");
    let incoming = headers(cx);
    for name in FORWARDED {
        for value in incoming.get_all(name) {
            builder = builder.header(name, value);
        }
    }
    let request = match body {
        Some(body) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string())),
        None => builder.body(Body::empty()),
    };
    let mut request = request.map_err(|error| ApiError::internal(error.to_string()))?;
    if let Some(info) = extensions(cx).get::<ConnectInfo<SocketAddr>>() {
        request.extensions_mut().insert(*info);
    }
    // A task of its own: the handler's future is large and a page render is already deep, so
    // polling it here would stack the two and overflow a 2 MiB worker stack in debug builds.
    let backend = web(cx).backend.clone();
    let response = tokio::spawn(async move { backend.oneshot(request).await })
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?
        .map_err(|error| match error {})?;
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, MAX_BODY)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Response::from_parts(parts, bytes))
}

/// The `Set-Cookie` headers of `response`, appended to the page's own response.
pub(crate) fn forward_cookies(cx: &Cx, response: &Response<axum::body::Bytes>) {
    for value in response.headers().get_all(header::SET_COOKIE) {
        response_headers(cx).append(header::SET_COOKIE, value.clone());
    }
}

/// Like [`send_raw`], but a non-2xx answer is an [`ApiError`].
///
/// # Errors
///
/// An [`ApiError`] for a non-2xx status or a failed call.
pub(crate) async fn send(
    cx: &Cx,
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
) -> Result<Response<axum::body::Bytes>, ApiError> {
    let response = send_raw(cx, method, path, body).await?;
    if response.status().is_success() {
        return Ok(response);
    }
    Err(error_of(&response))
}

fn error_of(response: &Response<axum::body::Bytes>) -> ApiError {
    let status = response.status();
    let message = serde_json::from_slice::<serde_json::Value>(response.body())
        .ok()
        .and_then(|value| value.get("error")?.as_str().map(str::to_owned))
        .filter(|text| !text.is_empty())
        .or_else(|| status.canonical_reason().map(str::to_owned))
        .unwrap_or_else(|| format!("HTTP {}", status.as_u16()));
    let retry_after = (status == StatusCode::TOO_MANY_REQUESTS)
        .then(|| {
            response
                .headers()
                .get(header::RETRY_AFTER)
                .and_then(|value: &HeaderValue| value.to_str().ok())
                .and_then(|value| value.trim().parse().ok())
        })
        .flatten();
    ApiError {
        status,
        message,
        retry_after,
    }
}

fn decode<T: DeserializeOwned>(response: &Response<axum::body::Bytes>) -> Result<T, ApiError> {
    let body = response.body();
    let json: &[u8] = if body.is_empty() { b"null" } else { body };
    serde_json::from_slice(json).map_err(|error| ApiError::internal(error.to_string()))
}

/// `GET /api/v1{path}`; `path` may carry a query string (see [`query`]).
///
/// # Errors
///
/// An [`ApiError`] for a non-2xx status or an answer that is not a `T`.
pub(crate) async fn get<T: DeserializeOwned>(cx: &Cx, path: &str) -> Result<T, ApiError> {
    decode(&send(cx, Method::GET, path, None).await?)
}

/// `POST /api/v1{path}` with a JSON body.
///
/// # Errors
///
/// An [`ApiError`] for a non-2xx status or an answer that is not a `T`.
pub(crate) async fn post<B: Serialize + ?Sized, T: DeserializeOwned>(
    cx: &Cx,
    path: &str,
    body: &B,
) -> Result<T, ApiError> {
    with_body(cx, Method::POST, path, body).await
}

/// `PATCH /api/v1{path}` with a JSON body.
///
/// # Errors
///
/// An [`ApiError`] for a non-2xx status or an answer that is not a `T`.
pub(crate) async fn patch<B: Serialize + ?Sized, T: DeserializeOwned>(
    cx: &Cx,
    path: &str,
    body: &B,
) -> Result<T, ApiError> {
    with_body(cx, Method::PATCH, path, body).await
}

/// `DELETE /api/v1{path}`.
///
/// # Errors
///
/// An [`ApiError`] for a non-2xx status or an answer that is not a `T`.
pub(crate) async fn delete<T: DeserializeOwned>(cx: &Cx, path: &str) -> Result<T, ApiError> {
    decode(&send(cx, Method::DELETE, path, None).await?)
}

/// `POST {path}` with a JSON body on a backend route outside `/api/v1`, e.g. `/oauth/authorize`.
///
/// # Errors
///
/// An [`ApiError`] for a non-2xx status or an answer that is not a `T`.
pub(crate) async fn post_root<B: Serialize + ?Sized, T: DeserializeOwned>(
    cx: &Cx,
    path: &str,
    body: &B,
) -> Result<T, ApiError> {
    let body = serde_json::to_value(body).map_err(|error| ApiError::internal(error.to_string()))?;
    let response = send_raw_at(cx, Method::POST, path, Some(body)).await?;
    if !response.status().is_success() {
        return Err(error_of(&response));
    }
    decode(&response)
}

async fn with_body<B: Serialize + ?Sized, T: DeserializeOwned>(
    cx: &Cx,
    method: Method,
    path: &str,
    body: &B,
) -> Result<T, ApiError> {
    let body = serde_json::to_value(body).map_err(|error| ApiError::internal(error.to_string()))?;
    decode(&send(cx, method, path, Some(body)).await?)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn enc_matches_encode_uri_component() {
        assert_eq!(enc("a b/c?d&é"), "a%20b%2Fc%3Fd%26%C3%A9");
        assert_eq!(enc("!~*'()-_."), "!~*'()-_.");
        assert_eq!(enc("@ada@x.example:8080"), "%40ada%40x.example%3A8080");
    }

    #[test]
    fn query_skips_absent_and_empty_values() {
        assert_eq!(query(&[("a", None), ("b", Some(""))]), "");
        assert_eq!(
            query(&[("q", Some("a b&c")), ("before", None), ("n", Some("1"))]),
            "?q=a+b%26c&n=1"
        );
    }

    #[test]
    fn error_bodies_name_the_api_message() {
        let mut response = Response::new(axum::body::Bytes::from_static(br#"{"error":"nope"}"#));
        *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
        response
            .headers_mut()
            .insert(header::RETRY_AFTER, HeaderValue::from_static("7"));
        let error = error_of(&response);
        assert_eq!(error.message, "nope");
        assert_eq!(error.retry_after, Some(7));

        let mut plain = Response::new(axum::body::Bytes::from_static(b"<html>"));
        *plain.status_mut() = StatusCode::BAD_GATEWAY;
        assert_eq!(error_of(&plain).message, "Bad Gateway");
    }
}
