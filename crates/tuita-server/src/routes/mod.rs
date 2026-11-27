//! HTTP handlers, one module per area. [`crate::router`] wires them.
//!
//! The JSON API lives under `/api/v1`; `api` builds that router. `outside_api` holds the public
//! routes that are not JSON: media files and RSS feeds.

mod admin;
mod affiliations;
mod ap;
mod auth;
mod emojis;
pub(crate) mod fallback;
mod federation_admin;
pub mod head;
pub(crate) mod health;
mod instance;
mod lists;
pub(crate) mod me;
pub(crate) mod media;
mod media_file;
mod messages;
mod notifications;
mod passkeys;
mod posts;
mod reports;
mod rss;
mod scheduled;
pub(crate) mod search;
mod stream;
mod timelines;
mod tokens;
mod users;
mod views;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::HeaderValue;
use axum::http::header::{CACHE_CONTROL, VARY};
use axum::middleware::{from_fn_with_state, map_response};
use axum::response::Response;

use crate::auth::{bearer, csrf};
use crate::error::AppError;
use crate::mastodon;
use crate::state::AppState;

/// Largest JSON request body.
const MAX_BODY_BYTES: usize = 1024 * 1024;

/// The `/api/v1` router: every API route behind the CSRF guard, with JSON 404 and 405 answers.
/// It holds tuita's own routes and the Mastodon v1 routes.
pub(crate) fn api(state: &AppState) -> Router<AppState> {
    // Routes that take a body larger than JSON set their own `DefaultBodyLimit`, which wins
    // over this one because it sits closer to the handler.
    guarded(
        Router::new()
            .merge(instance::router())
            .merge(auth::router())
            .merge(me::router())
            .merge(posts::router())
            .merge(timelines::router())
            .merge(users::router())
            .merge(search::router())
            .merge(notifications::router())
            .merge(stream::router())
            .merge(media::router(state))
            .merge(scheduled::router())
            .merge(views::router())
            .merge(messages::router())
            .merge(lists::router())
            .merge(reports::router())
            .merge(admin::router())
            .merge(tokens::router())
            .merge(affiliations::router())
            .merge(emojis::router())
            .merge(passkeys::router())
            .merge(ap::api_router())
            .merge(federation_admin::router())
            .merge(mastodon::router()),
        state,
    )
}

/// The `/api/v2` router: the Mastodon v2 routes behind the same layers as [`api`].
pub(crate) fn api_v2(state: &AppState) -> Router<AppState> {
    guarded(mastodon::router_v2(state), state)
}

/// `router` with JSON 404 and 405 answers and the layers every API route sits behind. Outermost
/// first: CORS (it answers preflights before anything else looks at them), the private-response
/// cache headers, the CSRF guard, the bearer gate, the body limit.
fn guarded(router: Router<AppState>, state: &AppState) -> Router<AppState> {
    router
        .fallback(|| async { AppError::NotFound })
        .method_not_allowed_fallback(|| async { AppError::MethodNotAllowed })
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(from_fn_with_state(state.clone(), bearer::gate))
        .layer(from_fn_with_state(state.clone(), csrf::guard))
        .layer(map_response(private_by_default))
        .layer(mastodon::cors())
}

/// Marks an API answer `no-store` and `Vary: Cookie, Authorization` unless the handler chose a
/// `Cache-Control` itself (the anonymous public status route does). Without it a browser history
/// cache or a shared proxy could keep or replay one reader's private JSON for another.
async fn private_by_default(mut response: Response) -> Response {
    let headers = response.headers_mut();
    if !headers.contains_key(CACHE_CONTROL) {
        headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
        headers.append(VARY, HeaderValue::from_static("Cookie, Authorization"));
    }
    response
}

/// Public routes outside the API prefix: `GET /media/{id}.{ext}`, `GET /u/{handle}/rss.xml`, the
/// ActivityPub objects and the OAuth endpoints.
pub(crate) fn outside_api(state: &AppState) -> Router<AppState> {
    Router::new()
        .merge(media::files())
        .merge(rss::router())
        .merge(ap::router())
        .merge(mastodon::outside_api(state))
}
