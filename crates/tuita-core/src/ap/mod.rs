//! ActivityPub, the pure half: object types, HTTP signatures, keys and HTML conversion. No I/O.
//!
//! JSON-LD is read as compact JSON, as Mastodon does. Reading is lenient (see [`OneOrMany`] and
//! [`IdOrObject`]); writing produces the shapes Mastodon, Misskey and GoToSocial accept.
//!
//! - [`Actor`], [`Note`], [`Collection`], [`Tombstone`], [`Document`], [`Tag`]: objects.
//! - [`Activity`], [`ActivityKind`]: the `Create`, `Follow`, `Like`, ... envelope.
//! - [`signature`]: sign and verify requests (draft-cavage-12, `rsa-sha256`).
//! - [`keys`]: RSA-2048 key pairs.
//! - [`html_to_text`], [`rich_text_to_html`]: content in both directions.
//! - [`parse_document`]: bounded JSON parsing of a received body.

mod activity;
mod html;
pub mod keys;
mod object;
mod origin;
pub mod signature;
mod value;

use serde::Serialize;
use serde_json::{Value, json};
use thiserror::Error;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

pub use activity::{Activity, ActivityKind};
pub use html::{
    MentionUrl, TagUrl, escape_html, html_to_text, plain_to_html, rich_text_to_html, sanitize_html,
    strip_controls,
};
pub use keys::{KeyError, KeyPair, PrivateKey, PublicKey, fingerprint, generate_keypair};
pub use object::{
    Actor, ActorKind, Collection, Document, Endpoints, Image, Note, NoteKind, PollOption,
    PropertyValue, PublicKeyInfo, Tag, Tombstone, VoteCount,
};
pub use origin::{
    MAX_URL_LEN, is_http_url, normalize_url, normalize_url_of, same_authority, url_authority,
};
pub use signature::{
    ParsedSignature, RequestParts, SignatureError, SignedHeaders, check_request, digest_header,
    http_date, parse_http_date, parse_signature_header, sign_get, sign_post, verify_signature,
};
pub use value::{HasId, IdOrObject, OneOrMany, id_of, type_names};

/// The audience that means everyone.
pub const PUBLIC: &str = "https://www.w3.org/ns/activitystreams#Public";

/// `Content-Type` of an activity we send.
pub const CONTENT_TYPE_ACTIVITY: &str = "application/activity+json";

/// `Accept` of a fetch of an actor or object.
pub const ACCEPT_ACTIVITY: &str = "application/activity+json, application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"";

/// Whether an `Accept` or `Content-Type` value asks for ActivityPub JSON.
///
/// ```
/// use tuita_core::ap::wants_activity_json;
///
/// assert!(wants_activity_json("application/activity+json"));
/// assert!(wants_activity_json("text/html, application/ld+json; profile=\"x\""));
/// assert!(!wants_activity_json("text/html,application/xhtml+xml"));
/// ```
#[must_use]
pub fn wants_activity_json(accept: &str) -> bool {
    accept.split(',').any(|part| {
        let kind = part.split(';').next().unwrap_or("").trim();
        kind.eq_ignore_ascii_case("application/activity+json")
            || kind.eq_ignore_ascii_case("application/ld+json")
    })
}

/// The `@context` every document we send carries.
#[must_use]
pub fn context() -> Value {
    json!([
        "https://www.w3.org/ns/activitystreams",
        "https://w3id.org/security/v1",
        {
            "manuallyApprovesFollowers": "as:manuallyApprovesFollowers",
            "sensitive": "as:sensitive",
            "Hashtag": "as:Hashtag",
            "Emoji": "toot:Emoji",
            "toot": "http://joinmastodon.org/ns#",
            "discoverable": "toot:discoverable",
            "indexable": "toot:indexable",
            "PropertyValue": "schema:PropertyValue",
            "schema": "http://schema.org#",
            "value": "schema:value",
            "quoteUrl": "as:quoteUrl"
        }
    ])
}

/// Serializes `value` and adds the `@context`. Anything that is not a JSON object is returned
/// unchanged.
#[must_use]
pub fn document<T: Serialize>(value: &T) -> Value {
    match serde_json::to_value(value) {
        Ok(Value::Object(mut map)) => {
            map.insert("@context".to_owned(), context());
            Value::Object(map)
        }
        Ok(other) => other,
        Err(_) => Value::Null,
    }
}

/// RFC 3339 text of `at`, in UTC, as ActivityPub `published` fields carry it.
#[must_use]
pub fn format_time(at: OffsetDateTime) -> String {
    at.to_offset(time::UtcOffset::UTC)
        .format(&Rfc3339)
        .unwrap_or_default()
}

/// Reads an RFC 3339 time, such as `2026-11-09T10:00:00Z` or `...+02:00`.
#[must_use]
pub fn parse_time(text: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(text.trim(), &Rfc3339).ok()
}

/// Deepest nesting accepted in a received document.
pub const MAX_DEPTH: usize = 24;
/// Most JSON values accepted in a received document.
pub const MAX_NODES: usize = 20_000;

/// Why a received body was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ParseError {
    /// The body is longer than the caller allows.
    #[error("document too large")]
    TooLarge,
    /// The body is not a JSON object.
    #[error("not a JSON object")]
    Invalid,
    /// Nested deeper than [`MAX_DEPTH`].
    #[error("document nested too deeply")]
    TooDeep,
    /// More than [`MAX_NODES`] values.
    #[error("document too complex")]
    TooComplex,
}

/// Parses a received body as one JSON object, refusing bodies over `max_bytes`, nesting deeper
/// than [`MAX_DEPTH`] and more than [`MAX_NODES`] values.
///
/// ```
/// use tuita_core::ap::{ParseError, parse_document};
///
/// assert!(parse_document(br#"{"type": "Follow"}"#, 1024).is_ok());
/// assert_eq!(parse_document(b"[1]", 1024), Err(ParseError::Invalid));
/// assert_eq!(parse_document(br#"{"a": 1}"#, 3), Err(ParseError::TooLarge));
/// ```
///
/// # Errors
///
/// A [`ParseError`] naming the limit that was broken.
pub fn parse_document(bytes: &[u8], max_bytes: usize) -> Result<Value, ParseError> {
    if bytes.len() > max_bytes {
        return Err(ParseError::TooLarge);
    }
    let value: Value = serde_json::from_slice(bytes).map_err(|_| ParseError::Invalid)?;
    if !value.is_object() {
        return Err(ParseError::Invalid);
    }
    let mut nodes = 0usize;
    let mut stack = vec![(&value, 1usize)];
    while let Some((node, depth)) = stack.pop() {
        nodes += 1;
        if nodes > MAX_NODES {
            return Err(ParseError::TooComplex);
        }
        if depth > MAX_DEPTH {
            return Err(ParseError::TooDeep);
        }
        match node {
            Value::Array(items) => stack.extend(items.iter().map(|item| (item, depth + 1))),
            Value::Object(map) => stack.extend(map.values().map(|item| (item, depth + 1))),
            _ => {}
        }
    }
    Ok(value)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deep_and_wide_documents_are_refused() {
        let deep = format!("{}1{}", "[".repeat(40), "]".repeat(40));
        let nested = format!(r#"{{"a": {deep}}}"#);
        assert_eq!(
            parse_document(nested.as_bytes(), 1 << 20),
            Err(ParseError::TooDeep)
        );
        let wide = format!("{{\"a\": [{}]}}", "1,".repeat(MAX_NODES) + "1");
        assert_eq!(
            parse_document(wide.as_bytes(), 1 << 20),
            Err(ParseError::TooComplex)
        );
        let recursion_bomb = "[".repeat(100_000);
        assert_eq!(
            parse_document(recursion_bomb.as_bytes(), 1 << 20),
            Err(ParseError::Invalid)
        );
    }

    #[test]
    fn times_round_trip_in_utc() {
        let at = parse_time("2026-11-09T12:00:00+02:00").expect("time");
        assert_eq!(format_time(at), "2026-11-09T10:00:00Z");
        assert_eq!(parse_time("later"), None);
    }

    #[test]
    fn documents_carry_the_context() {
        let activity = Activity::new(
            ActivityKind::Follow,
            Some("https://a/1".into()),
            "https://a/u",
            json!("https://b/u"),
        );
        let doc = activity.to_document();
        assert_eq!(doc["@context"][0], "https://www.w3.org/ns/activitystreams");
        assert_eq!(doc["type"], "Follow");
        assert_eq!(doc["object"], "https://b/u");
    }
}
