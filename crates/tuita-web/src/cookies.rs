//! Reading request cookies and writing the few cookies the web client owns.

use topcoat::{
    context::{Cx, context},
    router::{HeaderValue, header, request::headers, response::response_headers},
};

use crate::api::Web;

/// The value of cookie `name` in the request, if present and not empty.
pub(crate) fn get<'a>(cx: &'a Cx, name: &str) -> Option<&'a str> {
    headers(cx)
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|line| line.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, value)| *key == name && !value.is_empty())
        .map(|(_, value)| value)
}

/// Appends `Set-Cookie: name=value` for the whole site: `HttpOnly`, `SameSite=Lax`, `Secure`
/// when the instance runs on https. `max_age` is in seconds; `None` makes a session cookie.
///
/// `name` and `value` must be cookie-safe (ASCII letters, digits, `-`, `_`, `.`). A value that
/// is not is dropped instead of corrupting the header.
pub(crate) fn set(cx: &Cx, name: &str, value: &str, max_age: Option<u32>) {
    let mut line = format!("{name}={value}; Path=/; SameSite=Lax; HttpOnly");
    if let Some(age) = max_age {
        line.push_str(&format!("; Max-Age={age}"));
    }
    if context::<Web>(cx).state.config().secure_cookies {
        line.push_str("; Secure");
    }
    match HeaderValue::from_str(&line) {
        Ok(value) => {
            response_headers(cx).append(header::SET_COOKIE, value);
        }
        Err(error) => tracing::warn!(%error, name, "cookie value is not a valid header"),
    }
}

/// Expires cookie `name` in the browser.
pub(crate) fn expire(cx: &Cx, name: &str) {
    set(cx, name, "", Some(0));
}
