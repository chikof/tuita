//! URLs of the pages every area links to, and the `/@{handle}` route parameter.
//!
//! An `acct` is how the API names an account: `ada`, or `ada@remote.example` for a remote one
//! (no leading `@`; `fmt::acct_of` builds it).

use std::{fmt, str::FromStr};

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

use crate::api::enc;

/// What a path segment may hold unescaped here: the unreserved set plus `@` and `:`, which a
/// handle needs (`/@ada@remote.example:8080`).
const SEGMENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~')
    .remove(b'@')
    .remove(b':');

fn segment(text: &str) -> String {
    utf8_percent_encode(text, SEGMENT).to_string()
}

/// `/@ada` or `/@ada@remote.example`.
#[must_use]
pub(crate) fn profile(acct: &str) -> String {
    format!("/@{}", segment(acct))
}

/// `/@ada/{post_id}`.
#[must_use]
pub(crate) fn post(acct: &str, post_id: &str) -> String {
    format!("{}/{}", profile(acct), segment(post_id))
}

/// `/@ada/followers`.
#[must_use]
pub(crate) fn followers(acct: &str) -> String {
    format!("{}/followers", profile(acct))
}

/// `/@ada/following`.
#[must_use]
pub(crate) fn following(acct: &str) -> String {
    format!("{}/following", profile(acct))
}

/// `/messages/{acct}`, the conversation with one account.
#[must_use]
pub(crate) fn messages(acct: &str) -> String {
    format!("/messages/{}", enc(acct))
}

/// `/tags/{tag}`.
#[must_use]
pub(crate) fn tag(tag: &str) -> String {
    format!("/tags/{}", enc(tag))
}

/// `/lists/{id}`.
#[must_use]
pub(crate) fn list(id: &str) -> String {
    format!("/lists/{}", segment(id))
}

/// `/lists/{id}/members`.
#[must_use]
pub(crate) fn list_members(id: &str) -> String {
    format!("{}/members", list(id))
}

/// The value of the `/@{handle}` segment, leading `@` kept: `@ada`, `@ada@remote.example:8080`.
///
/// It matches `^@[A-Za-z0-9_]{1,15}(@[A-Za-z0-9.-]+(:\d+)?)?$`; anything else is not a profile
/// URL, so the route answers 404.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Handle(String);

impl Handle {
    /// The account as the API names it: the text without the leading `@`.
    #[must_use]
    pub(crate) fn acct(&self) -> &str {
        &self.0[1..]
    }
}

/// The segment is not a handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("not a handle")]
pub(crate) struct NotAHandle;

impl FromStr for Handle {
    type Err = NotAHandle;

    fn from_str(text: &str) -> Result<Self, NotAHandle> {
        let rest = text.strip_prefix('@').ok_or(NotAHandle)?;
        let (local, host) = match rest.split_once('@') {
            Some((local, host)) => (local, Some(host)),
            None => (rest, None),
        };
        let local_ok = (1..=15).contains(&local.len())
            && local
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_');
        let host_ok = host.is_none_or(|host| {
            let (name, port) = match host.split_once(':') {
                Some((name, port)) => (name, Some(port)),
                None => (host, None),
            };
            !name.is_empty()
                && name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
                && port.is_none_or(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
        });
        if local_ok && host_ok {
            Ok(Self(text.to_owned()))
        } else {
            Err(NotAHandle)
        }
    }
}

impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn handle_accepts_local_and_remote_accounts() {
        for ok in [
            "@a",
            "@ada_9",
            "@abcdefghijklmno",
            "@ada@remote.example",
            "@ada@remote.example:8080",
            "@ada@my-host.example",
        ] {
            assert_eq!(
                ok.parse::<Handle>().map(|h| h.to_string()),
                Ok(ok.to_owned()),
                "{ok}"
            );
        }
        assert_eq!(
            "@ada@remote.example"
                .parse::<Handle>()
                .map(|h| h.acct().to_owned()),
            Ok("ada@remote.example".to_owned())
        );
    }

    #[test]
    fn handle_refuses_everything_else() {
        for bad in [
            "ada",
            "@",
            "@abcdefghijklmnop",
            "@ada!",
            "@ada@",
            "@ada@host:",
            "@ada@host:80x",
            "@ada@ho st",
            "@ada@host@x",
            "@@ada",
            "",
        ] {
            assert!(bad.parse::<Handle>().is_err(), "{bad}");
        }
    }

    #[test]
    fn profile_urls_keep_at_signs_and_encode_the_rest() {
        assert_eq!(profile("ada"), "/@ada");
        assert_eq!(
            profile("ada@remote.example:8080"),
            "/@ada@remote.example:8080"
        );
        assert_eq!(post("ada", "0190"), "/@ada/0190");
        assert_eq!(followers("ada"), "/@ada/followers");
        assert_eq!(following("ada"), "/@ada/following");
        assert_eq!(
            messages("ada@remote.example"),
            "/messages/ada%40remote.example"
        );
        assert_eq!(tag("rust lang"), "/tags/rust%20lang");
        assert_eq!(list_members("7"), "/lists/7/members");
    }
}
