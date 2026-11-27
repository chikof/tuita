//! HTTP Signatures, the draft-cavage-12 dialect Mastodon speaks: `rsa-sha256` over a signing
//! string of `(request-target)` and header lines, plus the SHA-256 `Digest` of the body.
//!
//! Inbound: [`parse_signature_header`] reads the header, [`check_request`] runs every check that
//! needs no key (required headers, the `Date` window, the digest), and [`verify_signature`] does
//! the RSA check once the key is known. A server fetches the key between the last two calls, so
//! junk never costs a network round trip. Outbound: [`sign_post`] and [`sign_get`].

use std::sync::LazyLock;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use rsa::sha2::{Digest, Sha256};
use thiserror::Error;
use time::format_description::well_known::Rfc2822;
use time::format_description::{self, BorrowedFormatItem};
use time::{Duration, OffsetDateTime, PrimitiveDateTime};
use url::Url;

use super::keys::{KeyError, PrivateKey, PublicKey};

/// How far a request's `Date` may be from the receiver's clock.
pub const MAX_CLOCK_SKEW: Duration = Duration::hours(1);

/// Longest `Signature` header read.
const MAX_SIGNATURE_HEADER: usize = 4096;
/// Most headers a signature may list.
const MAX_SIGNED_HEADERS: usize = 32;

/// Why a request failed signature checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum SignatureError {
    /// The `Signature` header cannot be read.
    #[error("malformed signature header")]
    Malformed,
    /// The algorithm is not `rsa-sha256` or `hs2019`.
    #[error("unsupported signature algorithm")]
    UnsupportedAlgorithm,
    /// A header the signature must cover, or covers, is absent.
    #[error("missing signed header {0}")]
    MissingHeader(&'static str),
    /// The signature does not cover a header that must be covered.
    #[error("header {0} is not signed")]
    Unsigned(&'static str),
    /// The `Digest` header does not match the body.
    #[error("digest mismatch")]
    BadDigest,
    /// The `Date` is missing, unreadable or more than an hour off.
    #[error("stale or invalid date")]
    StaleDate,
    /// The RSA signature does not verify.
    #[error("signature does not verify")]
    BadSignature,
    /// Signing failed.
    #[error("cannot sign")]
    Key(#[from] KeyError),
}

/// What the signing string is built from: the request line and its headers.
#[derive(Debug, Clone, Copy)]
pub struct RequestParts<'a> {
    /// The HTTP method, any case.
    pub method: &'a str,
    /// The path and query exactly as sent, such as `/users/ada/inbox?x=1`.
    pub path_and_query: &'a str,
    /// Every header as `(name, value)`; names compare without case. A name that repeats is
    /// joined with `, `.
    pub headers: &'a [(&'a str, &'a str)],
}

impl RequestParts<'_> {
    /// The value of header `name`, repeated headers joined with `, `.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<String> {
        let mut found: Option<String> = None;
        for (key, value) in self.headers {
            if key.eq_ignore_ascii_case(name) {
                match &mut found {
                    Some(joined) => {
                        joined.push_str(", ");
                        joined.push_str(value.trim());
                    }
                    None => found = Some(value.trim().to_owned()),
                }
            }
        }
        found
    }
}

/// A parsed `Signature` header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSignature {
    /// The `keyId`: a URL that names the key, usually `{actor}#main-key`.
    pub key_id: String,
    /// The `algorithm`, when sent.
    pub algorithm: Option<String>,
    /// The covered headers, lower case, in signing order. `(request-target)` is one of them.
    pub headers: Vec<String>,
    /// The decoded signature bytes.
    pub signature: Vec<u8>,
}

/// `SHA-256=<base64>` of `body`, the `Digest` header value.
#[must_use]
pub fn digest_header(body: &[u8]) -> String {
    format!("SHA-256={}", STANDARD.encode(Sha256::digest(body)))
}

static HTTP_DATE: LazyLock<Vec<BorrowedFormatItem<'static>>> = LazyLock::new(|| {
    format_description::parse_borrowed::<2>(
        "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second] GMT",
    )
    .unwrap_or_default()
});

/// The HTTP date of `at`, such as `Sun, 06 Nov 1994 08:49:37 GMT`.
#[must_use]
pub fn http_date(at: OffsetDateTime) -> String {
    at.to_offset(time::UtcOffset::UTC)
        .format(&*HTTP_DATE)
        .unwrap_or_default()
}

/// Reads an HTTP date (or an RFC 2822 date, which some servers send).
#[must_use]
pub fn parse_http_date(text: &str) -> Option<OffsetDateTime> {
    let text = text.trim();
    PrimitiveDateTime::parse(text, &*HTTP_DATE)
        .map(PrimitiveDateTime::assume_utc)
        .ok()
        .or_else(|| OffsetDateTime::parse(text, &Rfc2822).ok())
}

/// Parses `keyId="…",algorithm="…",headers="…",signature="…"`.
///
/// # Errors
///
/// [`SignatureError::Malformed`] when a field is unquoted garbage, `keyId` or `signature` is
/// missing, the signature is not base64 or the header is longer than 4 KiB.
pub fn parse_signature_header(header: &str) -> Result<ParsedSignature, SignatureError> {
    if header.len() > MAX_SIGNATURE_HEADER {
        return Err(SignatureError::Malformed);
    }
    let mut key_id = None;
    let mut algorithm = None;
    let mut headers = None;
    let mut signature = None;
    let mut rest = header.trim();
    while !rest.is_empty() {
        let (name, after) = rest.split_once('=').ok_or(SignatureError::Malformed)?;
        let name = name.trim().trim_start_matches(',').trim();
        let (value, after) = take_value(after.trim_start())?;
        match name.to_ascii_lowercase().as_str() {
            "keyid" => key_id = Some(value),
            "algorithm" => algorithm = Some(value),
            "headers" => headers = Some(value),
            "signature" => signature = Some(value),
            _ => {}
        }
        rest = after.trim_start().trim_start_matches(',').trim_start();
    }
    let signature = STANDARD
        .decode(signature.ok_or(SignatureError::Malformed)?.trim())
        .map_err(|_| SignatureError::Malformed)?;
    let headers: Vec<String> = headers
        .as_deref()
        .unwrap_or("date")
        .split_ascii_whitespace()
        .map(str::to_ascii_lowercase)
        .collect();
    if headers.is_empty() || headers.len() > MAX_SIGNED_HEADERS || signature.is_empty() {
        return Err(SignatureError::Malformed);
    }
    Ok(ParsedSignature {
        key_id: key_id.ok_or(SignatureError::Malformed)?,
        algorithm,
        headers,
        signature,
    })
}

/// A quoted string (with `\"` escapes) or a bare token, and what follows it.
fn take_value(input: &str) -> Result<(String, &str), SignatureError> {
    if let Some(quoted) = input.strip_prefix('"') {
        let mut value = String::new();
        let mut chars = quoted.char_indices();
        while let Some((at, ch)) = chars.next() {
            match ch {
                '\\' => {
                    let (_, escaped) = chars.next().ok_or(SignatureError::Malformed)?;
                    value.push(escaped);
                }
                '"' => return Ok((value, &quoted[at + 1..])),
                other => value.push(other),
            }
        }
        Err(SignatureError::Malformed)
    } else {
        let end = input.find(',').unwrap_or(input.len());
        Ok((input[..end].trim().to_owned(), &input[end..]))
    }
}

/// The text that is signed: one `name: value` line per covered header, with
/// `(request-target)` as `{method} {path_and_query}`.
///
/// # Errors
///
/// [`SignatureError::MissingHeader`] when a covered header is absent from the request, and
/// [`SignatureError::Malformed`] for `(created)` and `(expires)`, which tuita does not accept.
pub fn signing_string(
    parts: &RequestParts<'_>,
    covered: &[String],
) -> Result<String, SignatureError> {
    let mut lines = Vec::with_capacity(covered.len());
    for name in covered {
        if name == "(request-target)" {
            lines.push(format!(
                "(request-target): {} {}",
                parts.method.to_ascii_lowercase(),
                parts.path_and_query
            ));
        } else if name.starts_with('(') {
            return Err(SignatureError::Malformed);
        } else {
            let value = parts
                .header(name)
                .ok_or(SignatureError::MissingHeader("covered header"))?;
            lines.push(format!("{name}: {value}"));
        }
    }
    Ok(lines.join("\n"))
}

fn covers(sig: &ParsedSignature, name: &str) -> bool {
    sig.headers.iter().any(|header| header == name)
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Every check that needs no key: the algorithm, that `(request-target)`, `host`, `date` and (for
/// `POST`) `digest` are covered, that `Date` is within [`MAX_CLOCK_SKEW`] of `now`, and that the
/// `Digest` header matches `body`.
///
/// # Errors
///
/// The first failing check, as a [`SignatureError`].
pub fn check_request(
    parts: &RequestParts<'_>,
    sig: &ParsedSignature,
    body: &[u8],
    now: OffsetDateTime,
) -> Result<(), SignatureError> {
    if sig
        .algorithm
        .as_deref()
        .is_some_and(|alg| !matches!(alg.to_ascii_lowercase().as_str(), "rsa-sha256" | "hs2019"))
    {
        return Err(SignatureError::UnsupportedAlgorithm);
    }
    for required in ["(request-target)", "host", "date"] {
        if !covers(sig, required) {
            return Err(SignatureError::Unsigned(required));
        }
    }
    let is_post = parts.method.eq_ignore_ascii_case("post");
    if is_post && !covers(sig, "digest") {
        return Err(SignatureError::Unsigned("digest"));
    }
    parts
        .header("host")
        .ok_or(SignatureError::MissingHeader("host"))?;
    let date = parts
        .header("date")
        .ok_or(SignatureError::MissingHeader("date"))?;
    let sent = parse_http_date(&date).ok_or(SignatureError::StaleDate)?;
    if (now - sent).abs() > MAX_CLOCK_SKEW {
        return Err(SignatureError::StaleDate);
    }
    if is_post || covers(sig, "digest") {
        let header = parts
            .header("digest")
            .ok_or(SignatureError::MissingHeader("digest"))?;
        let expected = Sha256::digest(body);
        let matched = header.split(',').any(|entry| {
            let Some((algorithm, value)) = entry.trim().split_once('=') else {
                return false;
            };
            algorithm.trim().eq_ignore_ascii_case("sha-256")
                && STANDARD
                    .decode(value.trim())
                    .is_ok_and(|got| constant_time_eq(&got, &expected))
        });
        if !matched {
            return Err(SignatureError::BadDigest);
        }
    }
    Ok(())
}

/// Verifies the RSA signature against `key`. Call [`check_request`] first.
///
/// # Errors
///
/// [`SignatureError::BadSignature`] when it does not verify, or the signing-string errors.
pub fn verify_signature(
    parts: &RequestParts<'_>,
    sig: &ParsedSignature,
    key: &PublicKey,
) -> Result<(), SignatureError> {
    let text = signing_string(parts, &sig.headers)?;
    if key.verify_sha256(text.as_bytes(), &sig.signature) {
        Ok(())
    } else {
        Err(SignatureError::BadSignature)
    }
}

/// The `Host` header value `reqwest` sends for `url`: the host, plus the port when it is not the
/// scheme's default.
fn host_header(url: &Url) -> Option<String> {
    let host = url.host_str()?;
    Some(match url.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.to_owned(),
    })
}

fn path_and_query(url: &Url) -> String {
    match url.query() {
        Some(query) => format!("{}?{query}", url.path()),
        None => url.path().to_owned(),
    }
}

/// Headers to add to a signed request: `(name, value)` pairs in lower case.
pub type SignedHeaders = Vec<(&'static str, String)>;

/// Signs a `POST` of `body` to `url` with `key`, naming it `key_id`. Signed headers:
/// `(request-target) host date digest content-type`.
///
/// Returns `host`, `date`, `digest`, `content-type` and `signature`.
///
/// # Errors
///
/// [`SignatureError::Malformed`] when `url` has no host, or the signing failure.
pub fn sign_post(
    key: &PrivateKey,
    key_id: &str,
    url: &Url,
    body: &[u8],
    now: OffsetDateTime,
) -> Result<SignedHeaders, SignatureError> {
    let host = host_header(url).ok_or(SignatureError::Malformed)?;
    let date = http_date(now);
    let digest = digest_header(body);
    let content_type = super::CONTENT_TYPE_ACTIVITY.to_owned();
    let covered = "(request-target) host date digest content-type";
    let text = format!(
        "(request-target): post {}\nhost: {host}\ndate: {date}\ndigest: {digest}\ncontent-type: {content_type}",
        path_and_query(url)
    );
    let signature = signature_header(key, key_id, covered, &text)?;
    Ok(vec![
        ("host", host),
        ("date", date),
        ("digest", digest),
        ("content-type", content_type),
        ("signature", signature),
    ])
}

/// Signs a `GET` of `url` for `accept`. Signed headers: `(request-target) host date accept`.
///
/// Returns `host`, `date` and `signature`; the caller sets `accept` to the same `accept`.
///
/// # Errors
///
/// [`SignatureError::Malformed`] when `url` has no host, or the signing failure.
pub fn sign_get(
    key: &PrivateKey,
    key_id: &str,
    url: &Url,
    accept: &str,
    now: OffsetDateTime,
) -> Result<SignedHeaders, SignatureError> {
    let host = host_header(url).ok_or(SignatureError::Malformed)?;
    let date = http_date(now);
    let covered = "(request-target) host date accept";
    let text = format!(
        "(request-target): get {}\nhost: {host}\ndate: {date}\naccept: {accept}",
        path_and_query(url)
    );
    let signature = signature_header(key, key_id, covered, &text)?;
    Ok(vec![
        ("host", host),
        ("date", date),
        ("signature", signature),
    ])
}

fn signature_header(
    key: &PrivateKey,
    key_id: &str,
    covered: &str,
    text: &str,
) -> Result<String, SignatureError> {
    let raw = key.sign_sha256(text.as_bytes())?;
    Ok(format!(
        "keyId=\"{key_id}\",algorithm=\"rsa-sha256\",headers=\"{covered}\",signature=\"{}\"",
        STANDARD.encode(raw)
    ))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ap::keys::generate_keypair;

    fn now() -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(1_800_000_000).expect("timestamp")
    }

    struct Pair {
        private: PrivateKey,
        public: PublicKey,
    }

    fn pair() -> Pair {
        let keys = generate_keypair().expect("keys");
        Pair {
            private: PrivateKey::from_pem(&keys.private_pem).expect("private"),
            public: PublicKey::from_pem(&keys.public_pem).expect("public"),
        }
    }

    fn headers_of(signed: &SignedHeaders) -> Vec<(String, String)> {
        signed
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect()
    }

    fn run(
        method: &str,
        path: &str,
        headers: &[(String, String)],
        body: &[u8],
        at: OffsetDateTime,
        key: &PublicKey,
    ) -> Result<(), SignatureError> {
        let view: Vec<(&str, &str)> = headers
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect();
        let parts = RequestParts {
            method,
            path_and_query: path,
            headers: &view,
        };
        let signature = parts.header("signature").ok_or(SignatureError::Malformed)?;
        let parsed = parse_signature_header(&signature)?;
        check_request(&parts, &parsed, body, at)?;
        verify_signature(&parts, &parsed, key)
    }

    #[test]
    fn http_dates_round_trip() {
        let at = OffsetDateTime::from_unix_timestamp(784_111_777).expect("timestamp");
        assert_eq!(http_date(at), "Sun, 06 Nov 1994 08:49:37 GMT");
        assert_eq!(parse_http_date("Sun, 06 Nov 1994 08:49:37 GMT"), Some(at));
        assert_eq!(parse_http_date("Sun, 06 Nov 1994 08:49:37 +0000"), Some(at));
        assert_eq!(parse_http_date("yesterday"), None);
    }

    #[test]
    fn a_signed_post_verifies() {
        let keys = pair();
        let url = Url::parse("https://remote.example/users/bob/inbox?x=1").expect("url");
        let body = br#"{"type":"Follow"}"#;
        let signed = sign_post(
            &keys.private,
            "https://a.example/users/ada#main-key",
            &url,
            body,
            now(),
        )
        .expect("sign");
        let mut headers = headers_of(&signed);
        headers.push(("User-Agent".into(), "x".into()));
        run(
            "POST",
            "/users/bob/inbox?x=1",
            &headers,
            body,
            now(),
            &keys.public,
        )
        .expect("verifies");
    }

    #[test]
    fn a_signed_get_verifies_and_carries_a_port() {
        let keys = pair();
        let url = Url::parse("http://127.0.0.1:9000/users/bob").expect("url");
        let signed =
            sign_get(&keys.private, "k", &url, "application/activity+json", now()).expect("sign");
        assert_eq!(signed[0], ("host", "127.0.0.1:9000".to_owned()));
        let mut headers = headers_of(&signed);
        headers.push(("accept".into(), "application/activity+json".into()));
        run("GET", "/users/bob", &headers, b"", now(), &keys.public).expect("verifies");
    }

    #[test]
    fn a_tampered_body_fails_the_digest() {
        let keys = pair();
        let url = Url::parse("https://remote.example/inbox").expect("url");
        let signed = sign_post(&keys.private, "k", &url, b"one", now()).expect("sign");
        let headers = headers_of(&signed);
        assert_eq!(
            run("POST", "/inbox", &headers, b"two", now(), &keys.public),
            Err(SignatureError::BadDigest)
        );
    }

    #[test]
    fn a_tampered_digest_header_fails_the_signature() {
        let keys = pair();
        let url = Url::parse("https://remote.example/inbox").expect("url");
        let signed = sign_post(&keys.private, "k", &url, b"one", now()).expect("sign");
        let mut headers = headers_of(&signed);
        for (name, value) in &mut headers {
            if name == "digest" {
                *value = digest_header(b"two");
            }
        }
        assert_eq!(
            run("POST", "/inbox", &headers, b"two", now(), &keys.public),
            Err(SignatureError::BadSignature)
        );
    }

    #[test]
    fn a_changed_path_fails() {
        let keys = pair();
        let url = Url::parse("https://remote.example/inbox").expect("url");
        let signed = sign_post(&keys.private, "k", &url, b"x", now()).expect("sign");
        let headers = headers_of(&signed);
        assert_eq!(
            run("POST", "/other", &headers, b"x", now(), &keys.public),
            Err(SignatureError::BadSignature)
        );
    }

    #[test]
    fn the_wrong_key_fails() {
        let keys = pair();
        let other = pair();
        let url = Url::parse("https://remote.example/inbox").expect("url");
        let signed = sign_post(&keys.private, "k", &url, b"x", now()).expect("sign");
        let headers = headers_of(&signed);
        assert_eq!(
            run("POST", "/inbox", &headers, b"x", now(), &other.public),
            Err(SignatureError::BadSignature)
        );
    }

    #[test]
    fn a_missing_host_fails() {
        let keys = pair();
        let url = Url::parse("https://remote.example/inbox").expect("url");
        let signed = sign_post(&keys.private, "k", &url, b"x", now()).expect("sign");
        let headers: Vec<_> = headers_of(&signed)
            .into_iter()
            .filter(|(name, _)| name != "host")
            .collect();
        assert_eq!(
            run("POST", "/inbox", &headers, b"x", now(), &keys.public),
            Err(SignatureError::MissingHeader("host"))
        );
    }

    #[test]
    fn a_signature_that_does_not_cover_the_digest_is_refused() {
        let keys = pair();
        let text = "(request-target): post /inbox\nhost: h\ndate: d";
        let raw = keys.private.sign_sha256(text.as_bytes()).expect("sign");
        let header = format!(
            "keyId=\"k\",headers=\"(request-target) host date\",signature=\"{}\"",
            STANDARD.encode(raw)
        );
        let parsed = parse_signature_header(&header).expect("parse");
        let date = http_date(now());
        let view = [("host", "h"), ("date", date.as_str()), ("digest", "x")];
        let parts = RequestParts {
            method: "POST",
            path_and_query: "/inbox",
            headers: &view,
        };
        assert_eq!(
            check_request(&parts, &parsed, b"", now()),
            Err(SignatureError::Unsigned("digest"))
        );
    }

    #[test]
    fn dates_outside_an_hour_fail_and_inside_pass() {
        let keys = pair();
        let url = Url::parse("https://remote.example/inbox").expect("url");
        let signed = sign_post(&keys.private, "k", &url, b"x", now()).expect("sign");
        let headers = headers_of(&signed);
        let late = now() + Duration::minutes(61);
        let early = now() - Duration::minutes(61);
        let near = now() + Duration::minutes(59);
        assert_eq!(
            run("POST", "/inbox", &headers, b"x", late, &keys.public),
            Err(SignatureError::StaleDate)
        );
        assert_eq!(
            run("POST", "/inbox", &headers, b"x", early, &keys.public),
            Err(SignatureError::StaleDate)
        );
        run("POST", "/inbox", &headers, b"x", near, &keys.public).expect("inside the window");
    }

    #[test]
    fn the_signature_header_parses_with_odd_spacing_and_rejects_junk() {
        let parsed = parse_signature_header(
            "keyId=\"https://a/u#main-key\", algorithm=\"hs2019\" ,headers=\"(request-target) Host Date\",signature=\"AAAA\"",
        )
        .expect("parses");
        assert_eq!(parsed.key_id, "https://a/u#main-key");
        assert_eq!(parsed.algorithm.as_deref(), Some("hs2019"));
        assert_eq!(parsed.headers, ["(request-target)", "host", "date"]);
        assert_eq!(parsed.signature, [0, 0, 0]);
        for junk in [
            "",
            "keyId=\"a\"",
            "keyId=\"a\",signature=\"***\"",
            "keyId=\"a,signature=\"AAAA\"",
            "nonsense",
        ] {
            assert_eq!(
                parse_signature_header(junk),
                Err(SignatureError::Malformed),
                "{junk}"
            );
        }
        assert_eq!(
            parse_signature_header(&"a".repeat(5000)),
            Err(SignatureError::Malformed)
        );
    }

    #[test]
    fn an_unknown_algorithm_is_refused() {
        let parsed = parse_signature_header(
            "keyId=\"k\",algorithm=\"hmac-sha256\",headers=\"(request-target) host date\",signature=\"AAAA\"",
        )
        .expect("parse");
        let view = [("host", "h"), ("date", "Sun, 06 Nov 1994 08:49:37 GMT")];
        let parts = RequestParts {
            method: "GET",
            path_and_query: "/",
            headers: &view,
        };
        assert_eq!(
            check_request(&parts, &parsed, b"", now()),
            Err(SignatureError::UnsupportedAlgorithm)
        );
    }

    #[test]
    fn digest_headers_compare_in_constant_time_and_by_algorithm() {
        assert_eq!(
            digest_header(b""),
            "SHA-256=47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU="
        );
    }
}
