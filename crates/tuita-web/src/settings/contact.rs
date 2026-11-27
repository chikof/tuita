//! Linking an instance's contact string.

fn no_space(text: &str) -> bool {
    !text.is_empty() && !text.chars().any(char::is_whitespace)
}

/// `a@b.c` with no `:`, `/` or space in any part.
fn is_address(text: &str) -> bool {
    let plain = |part: &str| {
        !part.is_empty()
            && !part
                .chars()
                .any(|c| c.is_whitespace() || matches!(c, '@' | ':' | '/'))
    };
    let Some((local, domain)) = text.split_once('@') else {
        return false;
    };
    plain(local) && domain.contains('.') && domain.split('.').all(plain)
}

/// `host.tld`, `host.tld:8080` or `host.tld/path`: dotted letters, digits and dashes.
fn is_bare_domain(text: &str) -> bool {
    let (host, path) = text
        .split_once('/')
        .map_or((text, None), |(h, p)| (h, Some(p)));
    let (name, port) = host
        .split_once(':')
        .map_or((host, None), |(n, p)| (n, Some(p)));
    let labels_ok = name.contains('.')
        && name.split('.').all(|label| {
            !label.is_empty() && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        });
    labels_ok
        && port.is_none_or(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
        && path.is_none_or(no_space_or_empty)
}

fn no_space_or_empty(text: &str) -> bool {
    !text.chars().any(char::is_whitespace)
}

/// The link target for a contact string, or `None` when it is neither an address nor a URL.
#[must_use]
pub(crate) fn contact_href(contact: &str) -> Option<String> {
    let text = contact.trim();
    let lower = text.to_ascii_lowercase();
    let scheme_rest = |scheme: &str| lower.strip_prefix(scheme).map(|_| &text[scheme.len()..]);
    if let Some(rest) = scheme_rest("https://").or_else(|| scheme_rest("http://"))
        && no_space(rest)
    {
        return Some(text.to_owned());
    }
    if let Some(rest) = scheme_rest("mailto:")
        && no_space(rest)
        && rest.contains('@')
    {
        return Some(text.to_owned());
    }
    if is_address(text) {
        return Some(format!("mailto:{text}"));
    }
    is_bare_domain(text).then(|| format!("https://{text}"))
}

#[cfg(test)]
mod test {
    use super::contact_href;

    #[test]
    fn links_addresses_urls_and_bare_domains_and_leaves_other_text_alone() {
        assert_eq!(
            contact_href("ops@example.org").as_deref(),
            Some("mailto:ops@example.org")
        );
        assert_eq!(
            contact_href("https://example.org/contact").as_deref(),
            Some("https://example.org/contact")
        );
        assert_eq!(
            contact_href("example.org/help").as_deref(),
            Some("https://example.org/help")
        );
        assert_eq!(contact_href("javascript:alert(1)"), None);
        assert_eq!(contact_href("DM @ada on the other site"), None);
    }
}
