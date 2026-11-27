//! What a search query asks the server to fetch from another instance.

/// Whether `q` is a handle on another instance (`@ada@remote.example`, `ada@remote.example`) or
/// the link of a post. The server fetches those instead of searching local text, which can fail
/// without an error, so an empty answer says the account or post may not be reachable.
pub(crate) fn is_lookup(q: &str) -> bool {
    let text = q.trim();
    is_account(text) || is_post_url(text)
}

/// `@?[A-Za-z0-9_]{1,15}@<host>[:<port>]`.
fn is_account(text: &str) -> bool {
    let text = text.strip_prefix('@').unwrap_or(text);
    let Some((local, host)) = text.split_once('@') else {
        return false;
    };
    let (name, port) = match host.split_once(':') {
        Some((name, port)) => (name, Some(port)),
        None => (host, None),
    };
    (1..=15).contains(&local.len())
        && local
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        && !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
        && port.is_none_or(|port| {
            (1..=5).contains(&port.len()) && port.bytes().all(|b| b.is_ascii_digit())
        })
}

/// `http(s)://<host>/<something>` with no whitespace.
fn is_post_url(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let Some(rest) = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
    else {
        return false;
    };
    let Some((host, path)) = rest.split_once('/') else {
        return false;
    };
    !host.is_empty()
        && !host.contains(['?', '#'])
        && !host.contains(char::is_whitespace)
        && !path.is_empty()
        && !path.contains(char::is_whitespace)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn handles_on_other_instances_are_lookups_with_or_without_the_leading_at() {
        assert!(is_lookup("@ada@remote.example"));
        assert!(is_lookup(" ada@remote.example:8443 "));
    }

    #[test]
    fn the_link_of_a_post_is_a_lookup() {
        assert!(is_lookup("https://remote.example/@ada/112233"));
    }

    #[test]
    fn everything_else_is_left_to_the_search() {
        for q in [
            "ada",
            "@ada",
            "from:ada@remote.example",
            "see https://a.example/x",
            "https://a.example",
            "",
            "@abcdefghijklmnop@remote.example",
            "ada@remote.example:123456",
        ] {
            assert!(!is_lookup(q), "{q}");
        }
    }
}
