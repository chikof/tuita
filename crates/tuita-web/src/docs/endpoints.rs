//! The routes shown in the developer docs and the scope a token needs for each.

use topcoat::context::Cx;
use tuita_server::dto::TokenScope;

use crate::l10n::t;

/// Roots (under `/api/v1`) a token is refused on, whatever its scopes: they need a real sign-in.
const COOKIE_ONLY: [&str; 6] = [
    "/tokens",
    "/admin",
    "/accounts",
    "/auth",
    "/passkeys",
    "/me/sessions",
];

/// Exact paths a token is refused on.
const COOKIE_ONLY_EXACT: [&str; 2] = ["/me/password", "/me/export"];

/// `path` is `root` or something below it (`/tokens` and `/tokens/abc`, not `/tokensmith`).
fn under(path: &str, root: &str) -> bool {
    path.strip_prefix(root)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

/// The scope a token needs for a request, or `None` when tokens are refused on that route.
#[must_use]
pub(crate) fn scope_for(method: &str, path: &str) -> Option<TokenScope> {
    let verb = method.to_ascii_uppercase();
    if COOKIE_ONLY.iter().any(|root| under(path, root))
        || COOKIE_ONLY_EXACT.contains(&path)
        || (verb == "DELETE" && path == "/me")
    {
        return None;
    }
    if under(path, "/conversations") {
        return Some(TokenScope::Messages);
    }
    Some(if verb == "GET" || verb == "HEAD" {
        TokenScope::Read
    } else {
        TokenScope::Write
    })
}

/// One documented route: its verb, its path with the placeholders the server uses, and what it
/// does (the localized sentence).
pub(crate) struct Endpoint {
    pub(crate) method: &'static str,
    pub(crate) path: &'static str,
    pub(crate) what: fn(&Cx) -> String,
}

macro_rules! endpoint {
    ($method:literal, $path:literal, $key:ident) => {
        Endpoint {
            method: $method,
            path: $path,
            what: |cx| t!($key).resolve(cx).into_owned(),
        }
    };
}

/// The routes worth showing, in display order. The scope column is computed by [`scope_for`], so
/// the table cannot drift from the rules.
pub(crate) const ENDPOINTS: [Endpoint; 22] = [
    endpoint!("GET", "/me", dev_ep_me),
    endpoint!("PATCH", "/me", dev_ep_patch_me),
    endpoint!("PATCH", "/me/bot", dev_ep_bot),
    endpoint!("GET", "/timelines/following", dev_ep_following),
    endpoint!("GET", "/timelines/foryou", dev_ep_foryou),
    endpoint!("GET", "/notifications", dev_ep_notifications),
    endpoint!("GET", "/posts/{id}", dev_ep_post),
    endpoint!("GET", "/users/{handle}", dev_ep_user),
    endpoint!("GET", "/users/{handle}/posts", dev_ep_user_posts),
    endpoint!("GET", "/search", dev_ep_search),
    endpoint!("GET", "/trends", dev_ep_trends),
    endpoint!("GET", "/stream", dev_ep_stream),
    endpoint!("POST", "/posts", dev_ep_create),
    endpoint!("POST", "/posts/thread", dev_ep_thread),
    endpoint!("DELETE", "/posts/{id}", dev_ep_delete),
    endpoint!("POST", "/posts/{id}/like", dev_ep_like),
    endpoint!("POST", "/users/{handle}/follow", dev_ep_follow),
    endpoint!("POST", "/media", dev_ep_media),
    endpoint!("POST", "/scheduled", dev_ep_schedule),
    endpoint!("POST", "/reports", dev_ep_report),
    endpoint!("GET", "/conversations", dev_ep_conversations),
    endpoint!("POST", "/conversations/{handle}", dev_ep_message),
];

/// The endpoints a token holding `scope` can call.
#[cfg(test)]
#[must_use]
pub(crate) fn endpoints_for(scope: TokenScope) -> Vec<&'static Endpoint> {
    ENDPOINTS
        .iter()
        .filter(|endpoint| scope_for(endpoint.method, endpoint.path) == Some(scope))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reads_need_read_writes_need_write_conversations_need_messages() {
        assert_eq!(
            scope_for("GET", "/timelines/following"),
            Some(TokenScope::Read)
        );
        assert_eq!(scope_for("GET", "/stream"), Some(TokenScope::Read));
        assert_eq!(scope_for("POST", "/posts"), Some(TokenScope::Write));
        assert_eq!(scope_for("PATCH", "/me/bot"), Some(TokenScope::Write));
        assert_eq!(scope_for("DELETE", "/posts/42"), Some(TokenScope::Write));
        assert_eq!(
            scope_for("GET", "/conversations"),
            Some(TokenScope::Messages)
        );
        assert_eq!(
            scope_for("POST", "/conversations/ada"),
            Some(TokenScope::Messages)
        );
    }

    #[test]
    fn the_cookie_only_routes_are_refused_for_every_verb() {
        for path in [
            "/tokens",
            "/tokens/abc",
            "/admin/stats",
            "/accounts",
            "/accounts/switch",
            "/auth/login",
            "/passkeys",
            "/passkeys/register",
            "/me/password",
            "/me/sessions",
            "/me/sessions/abc",
            "/me/export",
        ] {
            for method in ["GET", "POST", "DELETE"] {
                assert_eq!(scope_for(method, path), None, "{method} {path}");
            }
        }
        assert_eq!(scope_for("DELETE", "/me"), None);
    }

    #[test]
    fn similar_prefixes_are_not_mistaken_for_refused_ones() {
        assert_eq!(scope_for("GET", "/me"), Some(TokenScope::Read));
        assert_eq!(scope_for("GET", "/administrators"), Some(TokenScope::Read));
        assert_eq!(scope_for("GET", "/tokensmith"), Some(TokenScope::Read));
    }

    #[test]
    fn every_documented_endpoint_is_callable_with_some_scope() {
        for Endpoint { method, path, .. } in &ENDPOINTS {
            assert!(scope_for(method, path).is_some(), "{method} {path}");
        }
    }

    #[test]
    fn the_endpoints_split_by_scope_without_overlap() {
        let total = [TokenScope::Read, TokenScope::Write, TokenScope::Messages]
            .into_iter()
            .map(|scope| endpoints_for(scope).len())
            .sum::<usize>();
        assert_eq!(total, ENDPOINTS.len());
        let messages: Vec<_> = endpoints_for(TokenScope::Messages)
            .iter()
            .map(|endpoint| endpoint.path)
            .collect();
        assert_eq!(messages, ["/conversations", "/conversations/{handle}"]);
    }
}
