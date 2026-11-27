//! Reading the address of a remote object: which instance a URL belongs to, and whether it is
//! one tuita may link to.

use url::Url;

/// Longest URL accepted from a remote server.
pub const MAX_URL_LEN: usize = 2048;

fn parse(url: &str) -> Option<Url> {
    if url.len() > MAX_URL_LEN || url.chars().any(char::is_control) {
        return None;
    }
    let parsed = Url::parse(url).ok()?;
    let plain = matches!(parsed.scheme(), "http" | "https")
        && parsed.host_str().is_some()
        && parsed.username().is_empty()
        && parsed.password().is_none();
    plain.then_some(parsed)
}

/// Whether `url` is an absolute `http` or `https` URL with a host, no credentials, no control
/// character (a NUL cannot be stored, and the others are never part of an address) and at most
/// [`MAX_URL_LEN`] bytes. Anything shown or followed as a link has to pass this.
///
/// ```
/// use tuita_core::ap::is_http_url;
///
/// assert!(is_http_url("https://a.example/x"));
/// assert!(!is_http_url("javascript:alert(1)"));
/// assert!(!is_http_url("https://user:pw@a.example/"));
/// assert!(!is_http_url("https://a.example/x\u{0}y"));
/// ```
#[must_use]
pub fn is_http_url(url: &str) -> bool {
    parse(url).is_some()
}

/// The instance a URL belongs to: lower-case host without a trailing dot, plus `:port` unless it
/// is the scheme's default. `None` for anything that is not an [`is_http_url`].
///
/// ```
/// use tuita_core::ap::url_authority;
///
/// assert_eq!(url_authority("https://Social.Example/users/ada").as_deref(), Some("social.example"));
/// assert_eq!(url_authority("http://127.0.0.1:8081/x").as_deref(), Some("127.0.0.1:8081"));
/// assert_eq!(url_authority("https://a.example:443/x").as_deref(), Some("a.example"));
/// assert_eq!(url_authority("https://A.example./x").as_deref(), Some("a.example"));
/// ```
#[must_use]
pub fn url_authority(url: &str) -> Option<String> {
    let parsed = parse(url)?;
    let host = parsed
        .host_str()?
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if host.is_empty() {
        return None;
    }
    Some(match parsed.port() {
        Some(port) => format!("{host}:{port}"),
        None => host,
    })
}

/// `url` as one canonical string: lower-case host without a trailing dot, no default port. `None`
/// for anything that is not an [`is_http_url`]. Equal addresses give equal strings, so a stored
/// inbox URL and its domain agree.
///
/// ```
/// use tuita_core::ap::normalize_url;
///
/// assert_eq!(
///     normalize_url("HTTPS://Social.Example.:443/inbox?x=1").as_deref(),
///     Some("https://social.example/inbox?x=1")
/// );
/// assert_eq!(normalize_url("ftp://a.example/"), None);
/// ```
#[must_use]
pub fn normalize_url(url: &str) -> Option<String> {
    let mut parsed = parse(url)?;
    let host = parsed.host_str()?.trim_end_matches('.').to_owned();
    if host.is_empty() || parsed.set_host(Some(&host)).is_err() {
        return None;
    }
    Some(parsed.into())
}

/// [`normalize_url`] for a `url` on the same instance as `owner`; `None` otherwise. What a remote
/// actor says about its own endpoints goes through this before it is stored.
#[must_use]
pub fn normalize_url_of(owner: &str, url: &str) -> Option<String> {
    if same_authority(owner, url) {
        normalize_url(url)
    } else {
        None
    }
}

/// Whether both URLs name the same instance (see [`url_authority`]).
#[must_use]
pub fn same_authority(a: &str, b: &str) -> bool {
    match (url_authority(a), url_authority(b)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn authorities_compare_by_host_and_port() {
        assert!(same_authority("https://a.example/x", "https://A.example/y"));
        assert!(!same_authority(
            "https://a.example/x",
            "https://a.example:8443/y"
        ));
        assert!(!same_authority("https://a.example/x", "ftp://a.example/y"));
        assert!(!same_authority("nope", "nope"));
    }

    #[test]
    fn hostile_urls_are_refused() {
        for url in [
            "javascript:alert(1)",
            "data:text/html,hi",
            "//a.example/x",
            "https://u@a.example/",
        ] {
            assert!(!is_http_url(url), "{url}");
        }
        assert!(!is_http_url(&format!(
            "https://a.example/{}",
            "x".repeat(MAX_URL_LEN)
        )));
    }
}
