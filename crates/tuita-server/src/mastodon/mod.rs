//! The Mastodon-compatible client API: OAuth2, and the v1 and v2 routes Mastodon clients call.
//!
//! One file per area, each with a `router()` that `router` merges. The v1 routers hold paths
//! relative to `/api/v1` and sit behind the same layers as tuita's own API (CSRF guard, bearer
//! gate, body limit, CORS); the v2 routers (`router_v2`) do the same below `/api/v2`. The
//! OAuth endpoints outside the API prefix are `outside_api`.
//!
//! | File | Owns |
//! |---|---|
//! | `oauth` | `/apps`, `/oauth/authorization`, and outside the API `/oauth/authorize`, `/oauth/token`, `/oauth/revoke`, `/.well-known/oauth-authorization-server` |
//! | `accounts`, `instance`, `search`, `lists` | account, instance, search and list routes |
//! | `statuses`, `timelines`, `notifications`, `media`, `polls`, `conversations` | posting, reading, notifications, uploads, polls and direct messages |
//! | `misc` | the start-up and screen routes: preferences, blocks, mutes, filters, announcements, trends, suggestions, rules and grouped notifications |
//! | `streaming` | the websocket |
//! | `dto` | every JSON shape and the mappers from tuita values |
//! | `ctx` | what the mappers need besides their input, loaded in batches |
//! | `page` | `max_id`/`min_id`/`since_id`/`limit` and the `Link` header |
//! | `params` | query, form and JSON parameters as one flat map |
//!
//! Where a Mastodon route shares its path and method with a tuita route (`GET /instance`,
//! `GET /notifications`, `/lists`, `/bookmarks`, `/conversations`, `/announcements`,
//! `/suggestions`), register it once with
//! [`by_credential`]: the request of an OAuth token gets the Mastodon handler, everything else
//! keeps the tuita one.

use std::pin::Pin;
use std::time::Duration;

use axum::Router;
use axum::extract::Request;
use axum::handler::Handler;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE, LINK};
use axum::http::{HeaderName, Method};
use axum::response::Response;
use tower_http::cors::{Any, CorsLayer};

use crate::auth::bearer::TokenAuth;
use crate::state::AppState;

mod accounts;
pub(crate) mod conversations;
pub mod ctx;
pub mod dto;
pub(crate) mod instance;
pub(crate) mod lists;
mod media;
pub(crate) mod misc;
pub(crate) mod notifications;
mod oauth;
pub mod page;
pub mod params;
mod polls;
mod search;
pub(crate) mod statuses;
mod streaming;
pub(crate) mod timelines;

/// The routes below `/api/v1`. [`crate::routes::api`] merges them with tuita's own.
pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .merge(oauth::router())
        .merge(accounts::router())
        .merge(instance::router())
        .merge(lists::router())
        .merge(statuses::router())
        .merge(timelines::router())
        .merge(notifications::router())
        .merge(media::router())
        .merge(polls::router())
        .merge(conversations::router())
        .merge(streaming::router())
        .merge(misc::router())
}

/// The routes below `/api/v2`: the instance, search, media, filter, suggestion and notification routes.
pub(crate) fn router_v2(state: &AppState) -> Router<AppState> {
    Router::new()
        .merge(instance::router_v2())
        .merge(search::router_v2())
        .merge(media::router_v2(state))
        .merge(misc::router_v2())
}

/// The OAuth routes outside the API prefix: the authorization page and its answer, the token
/// and revocation endpoints and the server metadata.
pub(crate) fn outside_api(state: &AppState) -> Router<AppState> {
    oauth::outside(state)
}

/// The CORS layer of the Mastodon API: any origin, no credentials, the headers clients send
/// (`Authorization`, `Content-Type`, `Idempotency-Key`) and the `Link` header exposed for
/// pagination. It answers `OPTIONS` preflights itself.
pub fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            AUTHORIZATION,
            CONTENT_TYPE,
            HeaderName::from_static("idempotency-key"),
        ])
        .expose_headers([LINK])
        .max_age(Duration::from_secs(7200))
}

/// A handler that picks between a tuita handler and a Mastodon handler for the same route, by
/// the credential of the request. See [`by_credential`].
#[derive(Clone, Copy, Debug)]
pub struct Dispatch<Tuita, Mastodon> {
    tuita: Tuita,
    mastodon: Mastodon,
}

/// Routes a request that an OAuth token authenticated to `mastodon`, and every other request
/// (a session, a personal token, no sign-in) to `tuita`.
///
/// Use it where a Mastodon route shares method and path with a tuita route, which axum would
/// refuse to register twice: `.route("/notifications", get(by_credential(list, mastodon_list)))`.
/// Both handlers are ordinary axum handlers with their own extractors. The decision reads the
/// [`TokenAuth`] the bearer gate leaves, so the route must sit behind `bearer::gate`, as every
/// API route does.
#[must_use]
pub const fn by_credential<Tuita, Mastodon>(
    tuita: Tuita,
    mastodon: Mastodon,
) -> Dispatch<Tuita, Mastodon> {
    Dispatch { tuita, mastodon }
}

impl<Tuita, Mastodon, TT, TM> Handler<(TT, TM), AppState> for Dispatch<Tuita, Mastodon>
where
    Tuita: Handler<TT, AppState>,
    Mastodon: Handler<TM, AppState>,
    TT: 'static,
    TM: 'static,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    fn call(self, request: Request, state: AppState) -> Self::Future {
        let oauth = request
            .extensions()
            .get::<TokenAuth>()
            .is_some_and(|auth| auth.app_id.is_some());
        if oauth {
            Box::pin(self.mastodon.call(request, state))
        } else {
            Box::pin(self.tuita.call(request, state))
        }
    }
}

/// Which methods an [`ALLOWED`] row admits.
const ANY: &str = "*";

/// The routes an OAuth token (a token with `app_id`) may call below `/api/v1`: the Mastodon API
/// and nothing else. The bearer gate refuses every other path with `403` for such a token, so a
/// `read` or `write` grant never reaches tuita-only routes (`/conversations/{handle}`, `/me/bot`,
/// `/reports`, `/users/...`, `/posts`, ...). Personal tokens and cookies are not affected.
///
/// Each row is `(methods, pattern)`: `methods` is `"*"` or names joined by `|` (`HEAD` counts as
/// `GET`); `pattern` is the path below `/api/v1` without the leading slash, where `*` is exactly
/// one segment and a final `**` is the rest of the path (at least one segment). Routes that
/// tuita and Mastodon both serve (`/notifications`, `/lists`, `/bookmarks`, `/conversations`) are
/// listed with the Mastodon shapes only; `by_credential` hands the request to the Mastodon
/// handler. Everything below `/api/v2` is Mastodon and needs no row.
///
/// Add a row here when a slice registers a Mastodon route: a forgotten row answers `403`, and
/// `tests/oauth.rs` (`every_registered_mastodon_route_is_open_to_oauth_tokens`) fails.
pub const ALLOWED: &[(&str, &str)] = &[
    ("POST", "apps"),
    ("GET", "apps/verify_credentials"),
    (ANY, "accounts/**"),
    ("POST", "statuses"),
    (ANY, "statuses/**"),
    ("GET", "timelines/home"),
    ("GET", "timelines/public"),
    ("GET", "timelines/tag/*"),
    ("GET", "timelines/list/*"),
    ("GET", "notifications"),
    ("GET", "notifications/*"),
    ("POST", "notifications/clear"),
    ("POST", "notifications/*/dismiss"),
    ("GET|POST", "lists"),
    ("GET|PUT|DELETE", "lists/*"),
    ("GET|POST|DELETE", "lists/*/accounts"),
    ("GET", "bookmarks"),
    ("GET", "favourites"),
    ("GET", "conversations"),
    ("DELETE", "conversations/*"),
    ("POST", "conversations/*/read"),
    ("PUT", "media/*"),
    ("GET", "polls/*"),
    ("POST", "polls/*/votes"),
    ("GET", "instance"),
    ("GET", "instance/**"),
    ("GET", "custom_emojis"),
    ("GET", "streaming"),
    ("GET", "streaming/**"),
    (ANY, "scheduled_statuses"),
    (ANY, "scheduled_statuses/*"),
    ("GET", "preferences"),
    (ANY, "markers"),
    (ANY, "filters"),
    (ANY, "filters/**"),
    ("GET", "followed_tags"),
    (ANY, "follow_requests"),
    (ANY, "follow_requests/**"),
    ("GET", "blocks"),
    ("GET", "mutes"),
    ("GET|POST", "announcements"),
    (ANY, "announcements/**"),
    ("GET", "trends"),
    ("GET", "trends/**"),
    ("GET|DELETE", "suggestions"),
    ("DELETE", "suggestions/*"),
    ("GET", "endorsements"),
    (ANY, "domain_blocks"),
];

/// Whether an OAuth token may call `method path` below `/api/v1` (`v2` for `/api/v2`). See
/// [`ALLOWED`]. Fails closed: a path no row names is refused.
#[must_use]
pub fn oauth_allowed(method: &Method, path: &str, v2: bool) -> bool {
    if v2 {
        return true;
    }
    let path = path.strip_prefix("/api/v1").unwrap_or(path);
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let method = if *method == Method::HEAD {
        "GET"
    } else {
        method.as_str()
    };
    ALLOWED.iter().any(|(methods, pattern)| {
        (*methods == ANY || methods.split('|').any(|name| name == method))
            && pattern_matches(pattern, &segments)
    })
}

fn pattern_matches(pattern: &str, segments: &[&str]) -> bool {
    let mut rest = segments.iter();
    for part in pattern.split('/') {
        match (part, rest.next()) {
            ("**", Some(_)) => return true,
            ("*", Some(segment)) if !segment.is_empty() => {}
            (literal, Some(segment)) if literal == *segment && literal != "*" => {}
            _ => return false,
        }
    }
    rest.next().is_none()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn the_allowlist_admits_the_mastodon_shapes_and_nothing_else() {
        let id = "0198aaaa-0000-7000-8000-000000000001";
        for (method, path) in [
            (Method::GET, "/apps/verify_credentials".to_owned()),
            (Method::GET, "/accounts/verify_credentials".to_owned()),
            (Method::POST, format!("/accounts/{id}/follow")),
            (Method::GET, format!("/accounts/{id}/statuses")),
            (Method::PATCH, "/accounts/update_credentials".to_owned()),
            (Method::POST, "/statuses".to_owned()),
            (Method::PUT, format!("/statuses/{id}")),
            (Method::POST, format!("/statuses/{id}/favourite")),
            (Method::GET, "/timelines/home".to_owned()),
            (Method::GET, "/timelines/tag/rust".to_owned()),
            (Method::HEAD, "/timelines/public".to_owned()),
            (Method::GET, "/notifications".to_owned()),
            (Method::POST, "/notifications/clear".to_owned()),
            (Method::POST, format!("/notifications/{id}/dismiss")),
            (Method::GET, "/conversations".to_owned()),
            (Method::DELETE, format!("/conversations/{id}")),
            (Method::POST, format!("/conversations/{id}/read")),
            (Method::PUT, format!("/lists/{id}")),
            (Method::POST, format!("/lists/{id}/accounts")),
            (Method::PUT, format!("/media/{id}")),
            (Method::POST, format!("/polls/{id}/votes")),
            (Method::GET, "/api/v1/instance".to_owned()),
            (Method::GET, "/streaming".to_owned()),
            (Method::GET, "/preferences".to_owned()),
            (Method::GET, "/followed_tags".to_owned()),
            (Method::GET, "/follow_requests".to_owned()),
            (Method::GET, "/blocks".to_owned()),
            (Method::GET, "/mutes".to_owned()),
            (Method::GET, "/filters".to_owned()),
            (Method::GET, "/announcements".to_owned()),
            (Method::GET, "/trends/tags".to_owned()),
            (Method::GET, "/trends/statuses".to_owned()),
            (Method::GET, "/trends/links".to_owned()),
            (Method::GET, "/suggestions".to_owned()),
            (Method::GET, "/accounts/familiar_followers".to_owned()),
            (Method::GET, "/endorsements".to_owned()),
            (Method::GET, "/domain_blocks".to_owned()),
            (Method::GET, "/instance/rules".to_owned()),
        ] {
            assert!(oauth_allowed(&method, &path, false), "{method} {path}");
        }
        for (method, path) in [
            (Method::GET, "/me".to_owned()),
            (Method::POST, "/me/bot".to_owned()),
            (Method::PATCH, "/me".to_owned()),
            (Method::GET, "/conversations/ada".to_owned()),
            (Method::POST, "/conversations/ada".to_owned()),
            (Method::DELETE, "/conversations/ada/messages/1".to_owned()),
            (Method::POST, "/reports".to_owned()),
            (Method::GET, "/users/ada".to_owned()),
            (Method::POST, "/posts".to_owned()),
            (Method::POST, "/affiliations".to_owned()),
            (Method::PATCH, format!("/lists/{id}")),
            (Method::POST, format!("/lists/{id}/members/ada")),
            (Method::GET, format!("/lists/{id}/timeline")),
            (Method::POST, "/notifications/read".to_owned()),
            (Method::GET, "/timelines/following".to_owned()),
            (Method::POST, "/media".to_owned()),
            (Method::PATCH, format!("/media/{id}")),
            (Method::GET, "/accounts".to_owned()),
            (Method::GET, "/tokens".to_owned()),
            (Method::POST, "/instance".to_owned()),
            (Method::GET, "/".to_owned()),
            (Method::GET, "".to_owned()),
            (Method::GET, "/apps".to_owned()),
        ] {
            assert!(!oauth_allowed(&method, &path, false), "{method} {path}");
        }
        // Everything below /api/v2 is Mastodon.
        assert!(oauth_allowed(&Method::POST, "/media", true));
    }
}
