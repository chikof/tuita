//! API tokens as `Authorization: Bearer tuita_...`, for scripts, bots and OAuth clients.
//!
//! [`gate`] runs in front of every `/api/v1` and `/api/v2` route:
//!
//! - No bearer: the request goes on unchanged and the cookie rules apply.
//! - A bearer that is malformed, unknown or revoked, or whose owner is suspended or deleted: `401`.
//!   It never falls back to a cookie, so a revoked token cannot be mistaken for "signed out with
//!   a cookie".
//! - A valid bearer identifies the request. Cookies sent along are ignored for identity (the
//!   CSRF guard still looks at them, see [`super::csrf`]). The account's token is refused with
//!   `403` on the cookie-only paths and when it lacks the scope the route needs, then counted
//!   against the 600 per 10 minutes of the token.
//!
//! Two kinds of token share the table: a personal token (made at `POST /tokens`) and an OAuth
//! token (`app_id` set, made at `POST /oauth/token`). They differ in three ways. An OAuth token
//! never carries `messages`, so `/conversations` needs plain `read` or `write` for it. A
//! `client_credentials` token has no user: it passes the gate only on
//! `GET /apps/verify_credentials` and leaves an [`AppAuth`]. And a handler can tell the two apart
//! through [`TokenAuth::app_id`] or [`super::Auth::oauth_app`].
//!
//! An OAuth token may only call the Mastodon API: the gate checks the request against the
//! allowlist `mastodon::ALLOWED` (and everything below `/api/v2`) and answers every other path
//! `403`, so a `read` or `write` grant never reaches tuita-only routes such as direct messages.
//! Personal tokens keep the denylist below.
//!
//! Cookie-only: `/tokens`, `/admin`, `/auth`, `/passkeys`, `/oauth` (the consent card), exactly
//! `/accounts` and `/accounts/switch`, `/me/password`, `/me/sessions`, `/me/export`,
//! `/me/message-key` (the passphrase-sealed key backup) and
//! `DELETE /me`. Scopes: `/conversations` needs `messages`, other `GET` and `HEAD` requests need
//! `read`, everything else needs `write`; `GET /apps/verify_credentials` needs none.
//!
//! The websocket stream cannot send headers from a browser, so on `GET /streaming` the token may
//! also come as the `access_token` query parameter or as the first `Sec-WebSocket-Protocol`
//! value, in that order after the header.

use axum::extract::{OriginalUri, Request, State};
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, Method};
use axum::middleware::Next;
use axum::response::{IntoResponse as _, Response};
use url::form_urlencoded;
use uuid::Uuid;

use crate::db::tokens;
use crate::db::users::UserRow;
use crate::dto::TokenScope;
use crate::error::AppError;
use crate::state::AppState;

/// The `403` message for paths a token may not use.
pub const REFUSED: &str =
    "API tokens can\u{2019}t be used for this. Sign in on the website instead.";
/// The `401` message for a bearer that does not work.
const INVALID: &str = "Invalid or revoked API token.";
/// The `403` message for a `client_credentials` token on a route that needs a user.
const NEEDS_USER: &str = "This method requires an authenticated user.";
/// The prefix the API is mounted under, when a layer sees the full path.
const API_PREFIX: &str = "/api/v1";

/// What the gate leaves in the request extensions for [`super::MaybeUser`].
#[derive(Clone)]
pub struct TokenAuth {
    /// The token's owner.
    pub user: UserRow,
    /// The token.
    pub token_id: Uuid,
    /// What it may do.
    pub scopes: Vec<TokenScope>,
    /// The OAuth app the token was issued to; `None` for a personal token.
    pub app_id: Option<Uuid>,
}

/// What the gate leaves for a `client_credentials` token, which has an app and no user.
#[derive(Clone, Debug)]
pub struct AppAuth {
    /// The app.
    pub app_id: Uuid,
    /// The token.
    pub token_id: Uuid,
    /// What it may do.
    pub scopes: Vec<TokenScope>,
}

/// The token of an `Authorization: Bearer <token>` header. `None` when the header is missing or
/// uses another scheme; `Some("")` for a bearer header without a usable token.
fn bearer(headers: &HeaderMap) -> Option<&str> {
    let raw = headers.get(AUTHORIZATION)?.as_bytes();
    let split = raw
        .iter()
        .position(u8::is_ascii_whitespace)
        .unwrap_or(raw.len());
    let (scheme, rest) = raw.split_at(split);
    scheme
        .eq_ignore_ascii_case(b"bearer")
        .then(|| std::str::from_utf8(rest).unwrap_or_default().trim())
}

/// The first two path segments below the API prefix.
fn segments(path: &str) -> (&str, Option<&str>) {
    let path = path.strip_prefix(API_PREFIX).unwrap_or(path);
    let mut parts = path.trim_start_matches('/').split('/');
    let first = parts.next().unwrap_or_default();
    (first, parts.next().filter(|part| !part.is_empty()))
}

/// The token a request carries: the bearer header, and on `GET /streaming` the `access_token`
/// query parameter or the first `Sec-WebSocket-Protocol` value.
fn token_of(
    method: &Method,
    path: &str,
    query: Option<&str>,
    headers: &HeaderMap,
) -> Option<String> {
    if let Some(token) = bearer(headers) {
        return Some(token.to_owned());
    }
    if *method != Method::GET || segments(path).0 != "streaming" {
        return None;
    }
    let from_query = query.and_then(|query| {
        form_urlencoded::parse(query.as_bytes())
            .find(|(key, _)| key == "access_token")
            .map(|(_, value)| value.into_owned())
    });
    from_query.or_else(|| {
        headers
            .get("sec-websocket-protocol")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').next())
            .map(|protocol| protocol.trim().to_owned())
    })
}

/// Whether a token may not call `method path`: account security, sessions, passkeys, tokens,
/// administration, the OAuth consent card, the data export and account deletion. Exactly
/// `/accounts` and `/accounts/switch` are the account switcher; the Mastodon routes below
/// `/accounts/` are open to tokens.
#[must_use]
pub fn cookie_only(method: &Method, path: &str) -> bool {
    match segments(path) {
        ("tokens" | "admin" | "auth" | "passkeys" | "oauth", _)
        | ("accounts", None | Some("switch")) => true,
        ("me", Some("password" | "sessions" | "export" | "message-key")) => true,
        ("me", None) => method == Method::DELETE,
        _ => false,
    }
}

/// The scope a token needs for `method path`. `oauth` says the token was issued through OAuth:
/// it never carries `messages`, so conversations need `read` or `write` like any other route.
#[must_use]
pub fn scope_needed(method: &Method, path: &str, oauth: bool) -> TokenScope {
    if !oauth && segments(path).0 == "conversations" {
        TokenScope::Messages
    } else if matches!(*method, Method::GET | Method::HEAD) {
        TokenScope::Read
    } else {
        TokenScope::Write
    }
}

/// `GET /apps/verify_credentials`: any valid OAuth token may ask which app it belongs to.
fn verifies_app(method: &Method, path: &str) -> bool {
    *method == Method::GET && segments(path) == ("apps", Some("verify_credentials"))
}

/// Middleware, see the module docs.
pub async fn gate(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    // Owned copies: the request body is not `Sync`, so the request cannot be held across awaits.
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let query = request.uri().query().map(str::to_owned);
    // A nested router sees the path without its prefix; `OriginalUri` says which API it is.
    let v2 = request
        .extensions()
        .get::<OriginalUri>()
        .is_some_and(|original| original.0.path().starts_with("/api/v2"));
    let Some(secret) = token_of(&method, &path, query.as_deref(), request.headers()) else {
        return next.run(request).await;
    };
    match authorize(&state, &method, &path, v2, &secret).await {
        Ok(Authorized::User(auth)) => {
            request.extensions_mut().insert(*auth);
            next.run(request).await
        }
        Ok(Authorized::App(auth)) => {
            request.extensions_mut().insert(auth);
            next.run(request).await
        }
        Err(err) => err.into_response(),
    }
}

enum Authorized {
    User(Box<TokenAuth>),
    App(AppAuth),
}

async fn authorize(
    state: &AppState,
    method: &Method,
    path: &str,
    v2: bool,
    secret: &str,
) -> Result<Authorized, AppError> {
    if !tokens::is_well_formed(secret) {
        return Err(AppError::Denied(INVALID));
    }
    let Some(found) = tokens::find(state.db(), secret).await? else {
        return authorize_app(state, method, path, secret).await;
    };
    if cookie_only(method, path) {
        return Err(AppError::forbidden(REFUSED));
    }
    // An OAuth token is limited to the Mastodon API; see `mastodon::ALLOWED`.
    if found.app_id.is_some() && !crate::mastodon::oauth_allowed(method, path, v2) {
        return Err(AppError::forbidden(REFUSED));
    }
    let scopes = found.scopes();
    if !verifies_app(method, path) {
        refuse_missing_scope(&scopes, scope_needed(method, path, found.app_id.is_some()))?;
    }
    state.limits().check_token(found.token_id)?;
    Ok(Authorized::User(Box::new(TokenAuth {
        user: found.user,
        token_id: found.token_id,
        scopes,
        app_id: found.app_id,
    })))
}

/// A token with no user: valid only for [`verifies_app`].
async fn authorize_app(
    state: &AppState,
    method: &Method,
    path: &str,
    secret: &str,
) -> Result<Authorized, AppError> {
    let found = tokens::find_app_only(state.db(), secret)
        .await?
        .ok_or(AppError::Denied(INVALID))?;
    if !verifies_app(method, path) {
        return Err(AppError::forbidden(NEEDS_USER));
    }
    state.limits().check_token(found.token_id)?;
    Ok(Authorized::App(AppAuth {
        app_id: found.app_id,
        token_id: found.token_id,
        scopes: found.scopes(),
    }))
}

fn refuse_missing_scope(scopes: &[TokenScope], needed: TokenScope) -> Result<(), AppError> {
    if scopes.contains(&needed) {
        Ok(())
    } else {
        Err(AppError::forbidden(format!(
            "This token doesn\u{2019}t have the \u{201c}{}\u{201d} scope.",
            needed.as_str()
        )))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use axum::http::HeaderValue;

    fn header(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(value).expect("ascii"));
        headers
    }

    #[test]
    fn only_a_bearer_header_is_a_bearer() {
        assert_eq!(bearer(&HeaderMap::new()), None);
        assert_eq!(bearer(&header("Basic dXNlcjpwYXNz")), None);
        assert_eq!(bearer(&header("Bearer tuita_abc")), Some("tuita_abc"));
        assert_eq!(bearer(&header("bearer   tuita_abc ")), Some("tuita_abc"));
        assert_eq!(bearer(&header("Bearer")), Some(""));
        assert_eq!(bearer(&header("Bearerx tuita_abc")), None);
    }

    #[test]
    fn the_websocket_token_comes_from_the_query_or_the_subprotocol_on_streaming_only() {
        let none = HeaderMap::new();
        let get = Method::GET;
        assert_eq!(
            token_of(
                &get,
                "/streaming",
                Some("stream=user&access_token=tuita_a%2Bb"),
                &none
            ),
            Some("tuita_a+b".to_owned())
        );
        let mut protocol = HeaderMap::new();
        protocol.insert(
            "sec-websocket-protocol",
            HeaderValue::from_static("tuita_abc, other"),
        );
        assert_eq!(
            token_of(&get, "/api/v1/streaming/user", None, &protocol),
            Some("tuita_abc".to_owned())
        );
        // The header wins, and no other route reads the query or the subprotocol.
        assert_eq!(
            token_of(
                &get,
                "/streaming",
                Some("access_token=q"),
                &header("Bearer h")
            ),
            Some("h".to_owned())
        );
        assert_eq!(
            token_of(&get, "/me", Some("access_token=q"), &protocol),
            None
        );
        assert_eq!(
            token_of(&Method::POST, "/streaming", Some("access_token=q"), &none),
            None
        );
    }

    #[test]
    fn cookie_only_paths() {
        let get = Method::GET;
        for path in [
            "/tokens",
            "/tokens/5",
            "/admin/stats",
            "/accounts",
            "/accounts/switch",
            "/auth/login",
            "/auth/passkey",
            "/passkeys",
            "/passkeys/register/options",
            "/me/password",
            "/me/sessions",
            "/me/sessions/7",
            "/me/export",
            "/me/message-key",
            "/oauth/authorization",
            "/api/v1/admin/users",
        ] {
            assert!(cookie_only(&get, path), "{path}");
        }
        assert!(cookie_only(&Method::DELETE, "/me"));
        for path in [
            "/me",
            "/me/bot",
            "/me/blocks",
            "/posts",
            "/conversations/ada",
            "/tokensx",
            "/accounts/verify_credentials",
            "/accounts/0198aaaa-0000-7000-8000-000000000001",
            "/accounts/relationships",
            "/apps/verify_credentials",
        ] {
            assert!(!cookie_only(&get, path), "{path}");
        }
        assert!(!cookie_only(&Method::PATCH, "/me"));
    }

    #[test]
    fn scopes_follow_the_route() {
        assert_eq!(
            scope_needed(&Method::GET, "/timelines/following", false),
            TokenScope::Read
        );
        assert_eq!(
            scope_needed(&Method::HEAD, "/posts/1", false),
            TokenScope::Read
        );
        assert_eq!(
            scope_needed(&Method::POST, "/posts", false),
            TokenScope::Write
        );
        assert_eq!(
            scope_needed(&Method::DELETE, "/me/avatar", false),
            TokenScope::Write
        );
        assert_eq!(
            scope_needed(&Method::GET, "/conversations", false),
            TokenScope::Messages
        );
        assert_eq!(
            scope_needed(&Method::POST, "/api/v1/conversations/ada", false),
            TokenScope::Messages
        );
    }

    #[test]
    fn an_oauth_token_needs_read_or_write_for_conversations() {
        assert_eq!(
            scope_needed(&Method::GET, "/conversations", true),
            TokenScope::Read
        );
        assert_eq!(
            scope_needed(&Method::POST, "/conversations/1/read", true),
            TokenScope::Write
        );
    }

    #[test]
    fn only_verify_credentials_needs_no_scope() {
        assert!(verifies_app(&Method::GET, "/apps/verify_credentials"));
        assert!(!verifies_app(&Method::POST, "/apps/verify_credentials"));
        assert!(!verifies_app(&Method::GET, "/apps"));
    }
}
