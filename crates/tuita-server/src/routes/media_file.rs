//! `GET` and `HEAD /media/{id}.{ext}`: a stored file streamed from the media store, with one
//! byte range (`206`), `If-None-Match` and `If-Modified-Since` (`304`).

use std::ops::Range;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::extract::{Path, Request, State};
use axum::http::header::{
    ACCEPT_RANGES, CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE,
    ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED, RANGE,
};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode};
use axum::response::Response;
use uuid::Uuid;

use crate::error::AppError;
use crate::media::{ObjectMeta, StoreError};
use crate::state::AppState;

const IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// What a `Range` header asks for, given the object's size.
#[derive(Debug, PartialEq, Eq)]
enum Wanted {
    /// No usable range: send the whole object. Multiple ranges and a malformed header land
    /// here, as RFC 9110 allows.
    Whole,
    /// Send these bytes (end exclusive) as `206`.
    Part(Range<u64>),
    /// The range starts past the end: `416`.
    Unsatisfiable,
}

/// Reads the one range of `bytes=a-b`, `bytes=a-` or `bytes=-n` against an object of `size`.
fn wanted(header: &str, size: u64) -> Wanted {
    let Some(spec) = header.trim().strip_prefix("bytes=") else {
        return Wanted::Whole;
    };
    if spec.contains(',') {
        return Wanted::Whole;
    }
    let Some((first, last)) = spec.trim().split_once('-') else {
        return Wanted::Whole;
    };
    let (first, last) = (first.trim(), last.trim());
    let range = if first.is_empty() {
        let Ok(suffix) = last.parse::<u64>() else {
            return Wanted::Whole;
        };
        if suffix == 0 {
            return Wanted::Unsatisfiable;
        }
        size.saturating_sub(suffix)..size
    } else {
        let Ok(start) = first.parse::<u64>() else {
            return Wanted::Whole;
        };
        let end = if last.is_empty() {
            size
        } else {
            match last.parse::<u64>() {
                Ok(last) if last >= start => last.saturating_add(1).min(size),
                _ => return Wanted::Whole,
            }
        };
        start..end
    };
    if range.start >= size {
        Wanted::Unsatisfiable
    } else {
        Wanted::Part(range)
    }
}

/// Whether the request's validators say the client already has this object.
fn not_modified(headers: &HeaderMap, meta: &ObjectMeta) -> bool {
    if let Some(tags) = headers.get(IF_NONE_MATCH).and_then(|v| v.to_str().ok()) {
        let weak = |tag: &str| tag.trim().trim_start_matches("W/").to_owned();
        let ours = meta.etag.as_deref().map(weak);
        return tags
            .split(',')
            .any(|tag| tag.trim() == "*" || ours.as_deref() == Some(weak(tag).as_str()));
    }
    headers
        .get(IF_MODIFIED_SINCE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| httpdate::parse_http_date(v).ok())
        .is_some_and(|since| whole_seconds(meta.modified) <= since)
}

/// HTTP dates have no sub-second part.
fn whole_seconds(time: SystemTime) -> SystemTime {
    let secs = time.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    UNIX_EPOCH + Duration::from_secs(secs)
}

fn store_error(err: StoreError) -> AppError {
    match err {
        StoreError::NotFound(_) | StoreError::BadKey(_) => AppError::NotFound,
        other => AppError::Internal(other.into()),
    }
}

/// The headers every answer carries; `status` decides which more.
fn answer(status: StatusCode, mime: &str, meta: &ObjectMeta, body: Body) -> Response {
    let mut response = Response::new(body);
    *response.status_mut() = status;
    let headers = response.headers_mut();
    if let Ok(mime) = HeaderValue::from_str(mime) {
        headers.insert(CONTENT_TYPE, mime);
    }
    headers.insert(ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    if let Ok(modified) = HeaderValue::from_str(&httpdate::fmt_http_date(meta.modified)) {
        headers.insert(LAST_MODIFIED, modified);
    }
    if let Some(etag) = meta
        .etag
        .as_deref()
        .and_then(|tag| HeaderValue::from_str(tag).ok())
    {
        headers.insert(ETAG, etag);
    }
    if status.is_success() || status == StatusCode::NOT_MODIFIED {
        headers.insert(CACHE_CONTROL, HeaderValue::from_static(IMMUTABLE));
        headers.insert(CONTENT_DISPOSITION, HeaderValue::from_static("inline"));
    }
    response
}

fn with_length(mut response: Response, len: u64) -> Response {
    response
        .headers_mut()
        .insert(CONTENT_LENGTH, HeaderValue::from(len));
    response
}

/// `GET /media/{id}` or `/media/{id}.{ext}`: the stored file, with ranges. The row decides the
/// key and the type, never the request.
pub(super) async fn file(
    State(state): State<AppState>,
    Path(name): Path<String>,
    request: Request,
) -> Result<Response, AppError> {
    let (id, ext) = match name.split_once('.') {
        Some((id, ext)) => (id, Some(ext)),
        None => (name.as_str(), None),
    };
    if id.len() != 36 {
        return Err(AppError::NotFound);
    }
    let id = Uuid::parse_str(id).map_err(|_| AppError::NotFound)?;
    let (path, mime): (String, String) =
        sqlx::query_as("select path, mime from media where id = $1")
            .bind(id)
            .fetch_optional(state.db())
            .await?
            .ok_or(AppError::NotFound)?;
    if ext.is_some_and(|ext| path.rsplit_once('.').map(|(_, stored)| stored) != Some(ext)) {
        return Err(AppError::NotFound);
    }

    let store = state.media();
    let headers = request.headers();
    let head_only = request.method() == Method::HEAD;
    let range_header = headers.get(RANGE).and_then(|v| v.to_str().ok());
    let conditional =
        headers.contains_key(IF_NONE_MATCH) || headers.contains_key(IF_MODIFIED_SINCE);
    if !(head_only || range_header.is_some() || conditional) {
        let object = store.get(&path, None).await.map_err(store_error)?;
        let len = object.range.end - object.range.start;
        let response = answer(
            StatusCode::OK,
            &mime,
            &object.meta,
            Body::from_stream(object.body),
        );
        return Ok(with_length(response, len));
    }

    let meta = store.head(&path).await.map_err(store_error)?;
    if not_modified(headers, &meta) {
        return Ok(answer(
            StatusCode::NOT_MODIFIED,
            &mime,
            &meta,
            Body::empty(),
        ));
    }
    let part = match range_header.map_or(Wanted::Whole, |header| wanted(header, meta.size)) {
        Wanted::Unsatisfiable => {
            let mut response = answer(
                StatusCode::RANGE_NOT_SATISFIABLE,
                &mime,
                &meta,
                Body::empty(),
            );
            if let Ok(value) = HeaderValue::from_str(&format!("bytes */{}", meta.size)) {
                response.headers_mut().insert(CONTENT_RANGE, value);
            }
            return Ok(response);
        }
        Wanted::Part(range) => Some(range),
        Wanted::Whole => None,
    };
    let served = part.clone().unwrap_or(0..meta.size);
    let status = if part.is_some() {
        StatusCode::PARTIAL_CONTENT
    } else {
        StatusCode::OK
    };
    let body = if head_only {
        Body::empty()
    } else {
        Body::from_stream(store.get(&path, part).await.map_err(store_error)?.body)
    };
    let mut response = answer(status, &mime, &meta, body);
    if status == StatusCode::PARTIAL_CONTENT {
        let value = format!("bytes {}-{}/{}", served.start, served.end - 1, meta.size);
        if let Ok(value) = HeaderValue::from_str(&value) {
            response.headers_mut().insert(CONTENT_RANGE, value);
        }
    }
    Ok(with_length(response, served.end - served.start))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn one_range_is_read_against_the_size() {
        assert_eq!(wanted("bytes=0-99", 1000), Wanted::Part(0..100));
        assert_eq!(wanted("bytes=900-", 1000), Wanted::Part(900..1000));
        assert_eq!(wanted("bytes=-100", 1000), Wanted::Part(900..1000));
        assert_eq!(wanted("bytes=-5000", 1000), Wanted::Part(0..1000));
        assert_eq!(wanted("bytes=990-5000", 1000), Wanted::Part(990..1000));
        assert_eq!(wanted("bytes=0-0", 1000), Wanted::Part(0..1));
    }

    #[test]
    fn a_range_past_the_end_is_unsatisfiable() {
        assert_eq!(wanted("bytes=1000-", 1000), Wanted::Unsatisfiable);
        assert_eq!(wanted("bytes=2000-3000", 1000), Wanted::Unsatisfiable);
        assert_eq!(wanted("bytes=-0", 1000), Wanted::Unsatisfiable);
    }

    #[test]
    fn multiple_and_malformed_ranges_send_everything() {
        for header in [
            "bytes=0-1,5-6",
            "items=0-1",
            "bytes=a-b",
            "bytes=5-2",
            "bytes=",
            "bytes",
        ] {
            assert_eq!(wanted(header, 1000), Wanted::Whole, "{header}");
        }
    }

    fn meta(etag: Option<&str>) -> ObjectMeta {
        ObjectMeta {
            size: 10,
            modified: UNIX_EPOCH + Duration::from_secs(1_700_000_000) + Duration::from_millis(250),
            etag: etag.map(Into::into),
        }
    }

    fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.insert(*name, HeaderValue::from_str(value).expect("header value"));
        }
        map
    }

    #[test]
    fn validators_decide_304() {
        let tagged = meta(Some("\"abc\""));
        assert!(not_modified(
            &headers(&[("if-none-match", "\"abc\"")]),
            &tagged
        ));
        assert!(not_modified(
            &headers(&[("if-none-match", "\"x\", W/\"abc\"")]),
            &tagged
        ));
        assert!(not_modified(&headers(&[("if-none-match", "*")]), &tagged));
        assert!(!not_modified(
            &headers(&[("if-none-match", "\"x\"")]),
            &tagged
        ));
        assert!(!not_modified(&headers(&[]), &tagged));

        let date = httpdate::fmt_http_date(tagged.modified);
        assert!(not_modified(
            &headers(&[("if-modified-since", &date)]),
            &tagged
        ));
        let before = httpdate::fmt_http_date(tagged.modified - Duration::from_secs(60));
        assert!(!not_modified(
            &headers(&[("if-modified-since", &before)]),
            &tagged
        ));
        // The entity tag wins when both are sent.
        assert!(!not_modified(
            &headers(&[("if-none-match", "\"x\""), ("if-modified-since", &date)]),
            &tagged
        ));
    }
}
