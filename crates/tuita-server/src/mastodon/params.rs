//! The parameters of a Mastodon request.
//!
//! Mastodon clients send the same parameters as a query string, a form (`application/x-www-form-urlencoded`
//! or `multipart/form-data`) or JSON, and Rails merges them. [`Form`] does the same and flattens
//! every shape into one [`Params`] map, so a handler reads `params.get("status")` whatever the client sent:
//!
//! - `key[]=a&key[]=b` and `{"key": ["a", "b"]}` are the list `key` = `[a, b]`.
//! - `poll[expires_in]=300` and `{"poll": {"expires_in": 300}}` are the key `poll[expires_in]`.
//! - `poll[options][]=a` and `{"poll": {"options": ["a"]}}` are the list `poll[options]`.
//! - `fields_attributes[0][name]=x` and `{"fields_attributes": [{"name": "x"}]}` are the key
//!   `fields_attributes[0][name]`.
//! - JSON numbers and booleans become their text; JSON `null` is absent.
//!
//! Files of a multipart body are kept apart in [`Form::files`].

use std::collections::BTreeMap;

use axum::body::Bytes;
use axum::extract::{FromRequest, Multipart, Request};
use axum::http::header::CONTENT_TYPE;
use serde_json::Value;
use url::form_urlencoded;

use crate::error::AppError;
use crate::media::multipart_error;

/// Request parameters, flattened; see the module docs for the key forms.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Params(BTreeMap<String, Vec<String>>);

impl Params {
    /// The first value of `key`.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key)?.first().map(String::as_str)
    }

    /// Every value of `key`; empty when it is absent.
    #[must_use]
    pub fn list(&self, key: &str) -> &[String] {
        self.0.get(key).map_or(&[], Vec::as_slice)
    }

    /// Whether `key` was sent at all.
    #[must_use]
    pub fn has(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    /// The first value of `key` as a flag: `true`, `1`, `on`, `yes` and `t` are true; `false`,
    /// `0`, `off`, `no`, `f` and the empty string are false (Rails' rule); anything else, and an
    /// absent key, is `None`.
    #[must_use]
    pub fn flag(&self, key: &str) -> Option<bool> {
        match self.get(key)?.to_ascii_lowercase().as_str() {
            "true" | "1" | "on" | "yes" | "t" => Some(true),
            "false" | "0" | "off" | "no" | "f" | "" => Some(false),
            _ => None,
        }
    }

    /// The first value of `key` as an integer; `None` when absent or not a number.
    #[must_use]
    pub fn int(&self, key: &str) -> Option<i64> {
        self.get(key)?.trim().parse().ok()
    }

    /// Adds a value. A key written with a trailing `[]` is the key without it.
    pub fn push(&mut self, key: &str, value: String) {
        self.0
            .entry(key.strip_suffix("[]").unwrap_or(key).to_owned())
            .or_default()
            .push(value);
    }

    /// Replaces what `other` names, keeps the rest.
    fn merge(&mut self, other: Self) {
        self.0.extend(other.0);
    }

    /// The parameters of a query string.
    #[must_use]
    pub fn from_query(query: Option<&str>) -> Self {
        let mut params = Self::default();
        for (key, value) in form_urlencoded::parse(query.unwrap_or_default().as_bytes()) {
            params.push(&key, value.into_owned());
        }
        params
    }

    /// The parameters of a JSON object; `None` when `value` is not an object.
    #[must_use]
    pub fn from_json(value: &Value) -> Option<Self> {
        let Value::Object(map) = value else {
            return None;
        };
        let mut params = Self::default();
        for (key, value) in map {
            flatten(&mut params, key, value);
        }
        Some(params)
    }
}

fn flatten(params: &mut Params, key: &str, value: &Value) {
    match value {
        Value::Null => {}
        Value::Bool(flag) => params.push(key, flag.to_string()),
        Value::Number(number) => params.push(key, number.to_string()),
        Value::String(text) => params.push(key, text.clone()),
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                match item {
                    Value::Object(_) | Value::Array(_) => {
                        flatten(params, &format!("{key}[{index}]"), item);
                    }
                    _ => flatten(params, key, item),
                }
            }
        }
        Value::Object(map) => {
            for (inner, item) in map {
                flatten(params, &format!("{key}[{inner}]"), item);
            }
        }
    }
}

/// A file part of a multipart body.
#[derive(Clone, Debug)]
pub struct Upload {
    /// The form field the file came in.
    pub field: String,
    /// The file name the client gave.
    pub file_name: String,
    /// The part's `Content-Type`, when it had one.
    pub content_type: Option<String>,
    /// The content.
    pub bytes: Bytes,
}

/// The parameters of a request, from its query string and its body, whatever the body's format.
/// A key in the body replaces the same key of the query.
///
/// The body is read whole, within the route's `DefaultBodyLimit`: a route that takes uploads
/// raises that limit itself.
#[derive(Clone, Debug, Default)]
pub struct Form {
    /// The flattened parameters.
    pub params: Params,
    /// File parts, in order. Empty unless the body was `multipart/form-data`.
    pub files: Vec<Upload>,
}

impl<S: Send + Sync> FromRequest<S> for Form {
    type Rejection = AppError;

    async fn from_request(request: Request, state: &S) -> Result<Self, AppError> {
        let mut params = Params::from_query(request.uri().query());
        let content_type = request
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.trim().to_ascii_lowercase())
            .unwrap_or_default();
        if content_type.starts_with("multipart/form-data") {
            let mut multipart = Multipart::from_request(request, state)
                .await
                .map_err(|_| AppError::bad_request("Invalid form."))?;
            let mut files = Vec::new();
            while let Some(field) = multipart.next_field().await.map_err(multipart_error)? {
                let name = field.name().unwrap_or_default().to_owned();
                let file_name = field.file_name().map(str::to_owned);
                let part_type = field.content_type().map(str::to_owned);
                let bytes = field.bytes().await.map_err(multipart_error)?;
                match file_name {
                    Some(file_name) => files.push(Upload {
                        field: name,
                        file_name,
                        content_type: part_type,
                        bytes,
                    }),
                    None => params.push(&name, String::from_utf8_lossy(&bytes).into_owned()),
                }
            }
            return Ok(Self { params, files });
        }
        let bytes = Bytes::from_request(request, state)
            .await
            .map_err(|_| AppError::PayloadTooLarge("Request too large.".into()))?;
        if bytes.is_empty() {
            return Ok(Self {
                params,
                files: Vec::new(),
            });
        }
        let body = if content_type.starts_with("application/json") || content_type.contains("+json")
        {
            let value: Value = serde_json::from_slice(&bytes)
                .map_err(|_| AppError::bad_request("Invalid JSON."))?;
            Params::from_json(&value).ok_or_else(|| AppError::bad_request("Invalid JSON."))?
        } else if content_type.starts_with("application/x-www-form-urlencoded") {
            Params::from_query(std::str::from_utf8(&bytes).ok())
        } else {
            return Err(AppError::UnsupportedMediaType);
        };
        params.merge(body);
        Ok(Self {
            params,
            files: Vec::new(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use axum::body::Body;
    use serde_json::json;

    fn request(content_type: Option<&str>, uri: &str, body: &str) -> Request {
        let mut builder = Request::builder().method("POST").uri(uri);
        if let Some(content_type) = content_type {
            builder = builder.header(CONTENT_TYPE, content_type);
        }
        builder.body(Body::from(body.to_owned())).expect("request")
    }

    #[test]
    fn a_form_and_json_give_the_same_keys() {
        let form = Params::from_query(Some(
            "status=hi&poll%5Boptions%5D%5B%5D=a&poll%5Boptions%5D%5B%5D=b&poll%5Bexpires_in%5D=300\
             &media_ids%5B%5D=1&media_ids%5B%5D=2&fields_attributes%5B0%5D%5Bname%5D=site",
        ));
        let json = Params::from_json(&json!({
            "status": "hi",
            "poll": {"options": ["a", "b"], "expires_in": 300},
            "media_ids": ["1", "2"],
            "fields_attributes": [{"name": "site"}],
            "ignored": null,
        }))
        .expect("object");
        assert_eq!(form, json);
        assert_eq!(json.get("status"), Some("hi"));
        assert_eq!(json.list("poll[options]"), ["a", "b"]);
        assert_eq!(json.int("poll[expires_in]"), Some(300));
        assert_eq!(json.list("media_ids"), ["1", "2"]);
        assert_eq!(json.get("fields_attributes[0][name]"), Some("site"));
        assert!(!json.has("ignored"));
    }

    #[test]
    fn flags_follow_rails() {
        let params = Params::from_query(Some("a=true&b=0&c=maybe&d=&e=ON"));
        assert_eq!(params.flag("a"), Some(true));
        assert_eq!(params.flag("b"), Some(false));
        assert_eq!(params.flag("c"), None);
        assert_eq!(params.flag("d"), Some(false));
        assert_eq!(params.flag("e"), Some(true));
        assert_eq!(params.flag("missing"), None);
        let json = Params::from_json(&json!({"sensitive": true, "limit": 5})).expect("object");
        assert_eq!(json.flag("sensitive"), Some(true));
        assert_eq!(json.int("limit"), Some(5));
    }

    #[tokio::test]
    async fn the_body_replaces_the_query_for_every_format() {
        let json = request(
            Some("application/json; charset=utf-8"),
            "/x?a=query&b=query",
            r#"{"a": "body"}"#,
        );
        let form = Form::from_request(json, &()).await.expect("json");
        assert_eq!(form.params.get("a"), Some("body"));
        assert_eq!(form.params.get("b"), Some("query"));

        let urlencoded = request(
            Some("application/x-www-form-urlencoded"),
            "/x",
            "a=1&a=2&c%5B%5D=3",
        );
        let form = Form::from_request(urlencoded, &()).await.expect("form");
        assert_eq!(form.params.list("a"), ["1", "2"]);
        assert_eq!(form.params.list("c"), ["3"]);

        let boundary = "xyz";
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"description\"\r\n\r\nalt text\r\n\
             --{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"a.png\"\r\n\
             Content-Type: image/png\r\n\r\nPNGDATA\r\n--{boundary}--\r\n"
        );
        let multipart = request(
            Some(&format!("multipart/form-data; boundary={boundary}")),
            "/x",
            &body,
        );
        let form = Form::from_request(multipart, &()).await.expect("multipart");
        assert_eq!(form.params.get("description"), Some("alt text"));
        assert_eq!(form.files.len(), 1);
        assert_eq!(form.files[0].field, "file");
        assert_eq!(form.files[0].file_name, "a.png");
        assert_eq!(form.files[0].content_type.as_deref(), Some("image/png"));
        assert_eq!(&form.files[0].bytes[..], b"PNGDATA");
    }

    #[tokio::test]
    async fn bad_bodies_are_refused() {
        let bad_json = request(Some("application/json"), "/x", "{nope");
        assert!(matches!(
            Form::from_request(bad_json, &()).await,
            Err(AppError::BadRequest(_))
        ));
        let array = request(Some("application/json"), "/x", "[1]");
        assert!(matches!(
            Form::from_request(array, &()).await,
            Err(AppError::BadRequest(_))
        ));
        let odd = request(Some("text/plain"), "/x", "a=1");
        assert!(matches!(
            Form::from_request(odd, &()).await,
            Err(AppError::UnsupportedMediaType)
        ));
        let empty = request(None, "/x?q=1", "");
        let form = Form::from_request(empty, &()).await.expect("empty body");
        assert_eq!(form.params.get("q"), Some("1"));
    }
}
