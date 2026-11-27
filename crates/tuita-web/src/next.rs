//! Where to go after signing in: the `?next=` rule.

use url::Url;

use crate::api::enc;

/// The longest `?next=` value taken seriously.
const LONGEST: usize = 2000;
/// A made-up origin: a path is only ever asked whether it stays on its own origin.
const HERE: &str = "http://next.invalid";

/// A backslash (a slash to browsers), whitespace or a control character.
fn unsafe_char(c: char) -> bool {
    c == '\\' || c <= ' ' || c == '\u{7f}' || c == '\u{a0}'
}

/// An encoded slash or backslash in the path (a query holds them legitimately, e.g. a
/// `redirect_uri`): no return path needs one, and some stacks decode it late.
fn has_encoded_slash(raw: &str) -> bool {
    let path = raw.split(['?', '#']).next().unwrap_or(raw).as_bytes();
    path.windows(3)
        .any(|w| w[0] == b'%' && matches!(w[1..], [b'2', b'f' | b'F'] | [b'5', b'c' | b'C']))
}

/// The same-origin path (with query and fragment) a `?next=` asks to return to, or `None`.
///
/// Only a path that starts with a single `/` passes: no scheme, no host, no `//` or `/\`, no
/// whitespace or control character, and no encoded slash in the path. The check runs again on
/// the normalised result, because dot segments collapse: `/.//host` becomes `//host`.
#[must_use]
pub(crate) fn safe_next(raw: Option<&str>) -> Option<String> {
    let raw = raw?;
    if raw.is_empty() || raw.len() > LONGEST || !raw.starts_with('/') {
        return None;
    }
    if raw.starts_with("//") || has_encoded_slash(raw) || raw.chars().any(unsafe_char) {
        return None;
    }
    let base = Url::parse(HERE).ok()?;
    let url = base.join(raw).ok()?;
    if url.origin() != base.origin() {
        return None;
    }
    let mut path = url.path().to_owned();
    if let Some(query) = url.query() {
        path.push('?');
        path.push_str(query);
    }
    if let Some(fragment) = url.fragment() {
        path.push('#');
        path.push_str(fragment);
    }
    if !path.starts_with('/') || path.starts_with("//") || path.starts_with("/\\") {
        return None;
    }
    (base.join(&path).ok()?.origin() == base.origin()).then_some(path)
}

/// `/login`, remembering where the visitor was going. Sign-in pages and `/` are not remembered.
#[must_use]
pub(crate) fn login_url(path: &str, query: Option<&str>) -> String {
    if matches!(path, "/" | "/login" | "/signup") {
        return "/login".to_owned();
    }
    let mut here = path.to_owned();
    if let Some(query) = query.filter(|q| !q.is_empty()) {
        here.push('?');
        here.push_str(query);
    }
    match safe_next(Some(&here)) {
        Some(target) => format!("/login?next={}", enc(&target)),
        None => "/login".to_owned(),
    }
}

/// A link between the two sign-in pages that keeps `?add` and `?next`.
#[must_use]
pub(crate) fn auth_link(path: &str, add: bool, next: Option<&str>) -> String {
    let mut query = Vec::new();
    if add {
        query.push("add=1".to_owned());
    }
    if let Some(next) = next {
        query.push(format!("next={}", enc(next)));
    }
    if query.is_empty() {
        path.to_owned()
    } else {
        format!("{path}?{}", query.join("&"))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn keeps_a_same_origin_path_with_its_query() {
        for kept in [
            "/oauth/authorize?client_id=a&state=b%26c",
            "/@ada/post#top",
            "/oauth/authorize?redirect_uri=https%3A%2F%2Fc.example%2Fcb&scope=read%20write",
        ] {
            assert_eq!(safe_next(Some(kept)).as_deref(), Some(kept));
        }
    }

    #[test]
    fn refuses_anything_that_could_leave_the_origin() {
        let long = format!("/{}", "a".repeat(2000));
        for raw in [
            "",
            "oauth/authorize",
            "https://evil.example/",
            "//evil.example",
            "/\\evil.example",
            "/\t/evil.example",
            "/\n/evil.example",
            "/.//evil.example/x",
            "/..//evil.example",
            "/%2e//evil.example",
            "/%2E%2E//evil.example",
            "/a/..//evil.example",
            "/\\\\evil.example",
            "/%5Cevil.example",
            "/%5cevil.example",
            "/%2F/evil.example",
            "/. /evil.example",
            "/ /evil.example",
            "/.\t/\t/evil.example",
            "/\r/evil.example",
            "javascript:alert(1)",
            "/a\\b",
            long.as_str(),
        ] {
            assert_eq!(safe_next(Some(raw)), None, "{raw:?}");
        }
        assert_eq!(safe_next(None), None);
    }

    #[test]
    fn login_url_carries_the_page_the_visitor_was_going_to() {
        assert_eq!(
            login_url("/oauth/authorize", Some("client_id=a&scope=read+write")),
            "/login?next=%2Foauth%2Fauthorize%3Fclient_id%3Da%26scope%3Dread%2Bwrite"
        );
        assert_eq!(login_url("/settings", None), "/login?next=%2Fsettings");
    }

    #[test]
    fn login_url_does_not_remember_the_sign_in_pages_or_home() {
        for path in ["/", "/login", "/signup"] {
            assert_eq!(login_url(path, Some("add=1")), "/login");
        }
    }

    #[test]
    fn auth_links_keep_add_and_next() {
        assert_eq!(auth_link("/signup", false, None), "/signup");
        assert_eq!(
            auth_link("/login", true, Some("/a?b=c")),
            "/login?add=1&next=%2Fa%3Fb%3Dc"
        );
    }
}
