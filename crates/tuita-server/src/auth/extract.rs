//! Request extractors: the signed-in user, client metadata, and JSON/query parsing that fails
//! with this API's error bodies.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::Json;
use axum::body::Bytes;
use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::extract::{ConnectInfo, FromRequest, FromRequestParts, Query, Request};
use axum::http::header::USER_AGENT;
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use serde::de::DeserializeOwned;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::bearer::TokenAuth;
use super::{Tokens, hash_token};
use crate::db::sessions::{self, ClientMeta};
use crate::db::users::UserRow;
use crate::error::AppError;
use crate::state::AppState;

/// How stale `last_used_at` may get before a request refreshes it.
const TOUCH_AFTER: Duration = Duration::hours(1);
/// Longest stored `User-Agent`.
const MAX_USER_AGENT: usize = 300;

/// What authenticated a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Credential {
    /// A login session (a cookie).
    Session(Uuid),
    /// An API token (`Authorization: Bearer`).
    Token(Uuid),
}

/// The account behind a request and what it used to sign in.
pub struct Auth {
    /// The signed-in account.
    pub user: UserRow,
    /// The session or token that authenticated the request.
    pub credential: Credential,
    /// The OAuth app, when the request carried an OAuth token; `None` for a session and for a
    /// personal token. Routes that answer in the Mastodon shape check this.
    pub app: Option<Uuid>,
}

impl Auth {
    /// The session behind the request. Routes that act on sessions or account security call
    /// this, so even a token the path rules let through cannot reach them.
    ///
    /// # Errors
    ///
    /// `403` when an API token authenticated the request.
    pub fn session_id(&self) -> Result<Uuid, AppError> {
        match self.credential {
            Credential::Session(id) => Ok(id),
            Credential::Token(_) => Err(AppError::forbidden(super::bearer::REFUSED)),
        }
    }

    /// The OAuth app the request's token was issued to, if any.
    #[must_use]
    pub fn oauth_app(&self) -> Option<Uuid> {
        self.app
    }
}

/// The signed-in account, or nobody.
pub struct MaybeUser(pub Option<Auth>);

/// The signed-in account; `401 sign in required` otherwise.
pub struct CurrentUser(pub Auth);

/// A moderator or admin; `401` when signed out and `403 staff only` for regular accounts.
pub struct Staff(pub Auth);

/// An admin; `401` when signed out and `403 admin only` for everyone else.
pub struct Admin(pub Auth);

impl FromRequestParts<AppState> for MaybeUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        // A bearer token the gate accepted identifies the request; cookies play no part then.
        if let Some(token) = parts.extensions.get::<TokenAuth>() {
            return Ok(Self(Some(Auth {
                user: token.user.clone(),
                credential: Credential::Token(token.token_id),
                app: token.app_id,
            })));
        }
        let Some(token) = Tokens::from_headers(&parts.headers).active else {
            return Ok(Self(None));
        };
        let Some(found) = sessions::find(state.db(), &hash_token(&token)).await? else {
            return Ok(Self(None));
        };
        if OffsetDateTime::now_utc() - found.last_used_at > TOUCH_AFTER {
            let meta = client_meta(parts, state);
            if let Err(err) = sessions::touch(state.db(), found.session_id, &meta).await {
                tracing::warn!(error = %err, "cannot refresh a session");
            }
        }
        Ok(Self(Some(Auth {
            user: found.user,
            credential: Credential::Session(found.session_id),
            app: None,
        })))
    }
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        MaybeUser::from_request_parts(parts, state)
            .await?
            .0
            .map(Self)
            .ok_or(AppError::Unauthorized)
    }
}

impl FromRequestParts<AppState> for Staff {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let CurrentUser(auth) = CurrentUser::from_request_parts(parts, state).await?;
        if auth.user.is_staff() {
            Ok(Self(auth))
        } else {
            Err(AppError::forbidden("staff only"))
        }
    }
}

impl FromRequestParts<AppState> for Admin {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let CurrentUser(auth) = CurrentUser::from_request_parts(parts, state).await?;
        if auth.user.role() == crate::dto::Role::Admin {
            Ok(Self(auth))
        } else {
            Err(AppError::forbidden("admin only"))
        }
    }
}

impl FromRequestParts<AppState> for ClientMeta {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(client_meta(parts, state))
    }
}

/// The last hop of the last `X-Forwarded-For` header line: the one the trusted proxy added.
///
/// A proxy may append its hop to the client's own line or add a second line; either way the
/// proxy's value is last, and everything before it is client-supplied and spoofable.
fn forwarded_for(headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get_all("x-forwarded-for")
        .iter()
        .next_back()
        .and_then(|value| value.to_str().ok())
        .and_then(|list| list.rsplit(',').next())
        .and_then(|hop| hop.trim().parse::<IpAddr>().ok())
}

/// The client address and device facts of a request.
///
/// The address is the last `X-Forwarded-For` hop when `TRUST_PROXY` is set, else the peer
/// address from `ConnectInfo`. Without either (in-process tests) it is `0.0.0.0`. The country
/// comes from `CF-IPCountry`, only behind a trusted proxy. Both headers are as trustworthy as
/// the proxy: it must overwrite `CF-IPCountry` and overwrite or append to `X-Forwarded-For`.
fn client_meta(parts: &Parts, state: &AppState) -> ClientMeta {
    let trust = state.config().trust_proxy;
    let forwarded = trust.then(|| forwarded_for(&parts.headers)).flatten();
    let peer = parts
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| addr.ip());
    let ip = forwarded
        .or(peer)
        .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    let country = trust
        .then(|| {
            parts
                .headers
                .get("cf-ipcountry")
                .and_then(|value| value.to_str().ok())
                .map(str::to_ascii_uppercase)
                .filter(|code| {
                    code.len() == 2
                        && code.bytes().all(|b| b.is_ascii_uppercase())
                        && !matches!(code.as_str(), "XX" | "T1")
                })
        })
        .flatten();
    let user_agent: String = parts
        .headers
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .chars()
        .take(MAX_USER_AGENT)
        .collect();
    // The native app says what it is in `tuita-client`; everything else is read from the UA.
    let declared_android = parts
        .headers
        .get("tuita-client")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("android"));
    ClientMeta {
        ip: Some(ip),
        client: Some(if declared_android {
            "android-app"
        } else {
            client_of(&user_agent)
        }),
        user_agent,
        country,
    }
}

// The device family of a `User-Agent`.
#[must_use]
fn client_of(user_agent: &str) -> &'static str {
    let has = |needle: &str| user_agent.contains(needle);
    if !has("Mozilla/") {
        "api"
    } else if has("iPhone") || has("iPod") {
        "iphone"
    } else if has("iPad") || (has("Macintosh") && has("Mobile/")) {
        "ipad"
    } else if has("Android") {
        "android"
    } else if has("CrOS") {
        "chromeos"
    } else if has("Windows") {
        "windows"
    } else if has("Macintosh") || has("Mac OS X") {
        "mac"
    } else if has("Linux") || has("X11") {
        "linux"
    } else {
        "web"
    }
}

/// JSON body that fails with this API's error bodies: `415 Expected JSON.`, `413 Request too
/// large.`, otherwise `400 Invalid JSON.`.
pub struct ApiJson<T>(pub T);

impl<T, S> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(request: Request, state: &S) -> Result<Self, AppError> {
        match Json::<T>::from_request(request, state).await {
            Ok(Json(value)) => Ok(Self(value)),
            Err(rejection) => Err(json_error(&rejection)),
        }
    }
}

fn json_error(rejection: &JsonRejection) -> AppError {
    match rejection.status() {
        StatusCode::UNSUPPORTED_MEDIA_TYPE => AppError::UnsupportedMediaType,
        StatusCode::PAYLOAD_TOO_LARGE => AppError::PayloadTooLarge("Request too large.".into()),
        _ => AppError::bad_request("Invalid JSON."),
    }
}

/// Like [`ApiJson`], but an empty body means `T::default()`.
pub struct ApiJsonOrDefault<T>(pub T);

impl<T, S> FromRequest<S> for ApiJsonOrDefault<T>
where
    T: DeserializeOwned + Default,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(request: Request, state: &S) -> Result<Self, AppError> {
        let bytes = Bytes::from_request(request, state)
            .await
            .map_err(|_| AppError::PayloadTooLarge("Request too large.".into()))?;
        if bytes.is_empty() {
            return Ok(Self(T::default()));
        }
        serde_json::from_slice(&bytes)
            .map(Self)
            .map_err(|_| AppError::bad_request("Invalid JSON."))
    }
}

/// Query string that fails with `400 Invalid query.` as a JSON error body.
pub struct ApiQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for ApiQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        Query::<T>::from_request_parts(parts, state)
            .await
            .map(|Query(value)| Self(value))
            .map_err(|_: QueryRejection| AppError::bad_request("Invalid query."))
    }
}

#[cfg(test)]
mod test {
    use axum::http::{HeaderMap, HeaderValue};

    use super::{IpAddr, client_of, forwarded_for};

    #[test]
    fn device_families() {
        let ua = |s: &str| client_of(s);
        assert_eq!(ua("curl/8.0"), "api");
        assert_eq!(ua("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0)"), "iphone");
        assert_eq!(
            ua("Mozilla/5.0 (Macintosh; Intel Mac OS X) Mobile/15E148"),
            "ipad"
        );
        assert_eq!(ua("Mozilla/5.0 (Linux; Android 14)"), "android");
        assert_eq!(ua("Mozilla/5.0 (X11; Linux x86_64)"), "linux");
        assert_eq!(ua("Mozilla/5.0 (Windows NT 10.0)"), "windows");
        assert_eq!(ua("Mozilla/5.0 (Macintosh; Intel Mac OS X 14)"), "mac");
        assert_eq!(ua("Mozilla/5.0 (something)"), "web");
    }

    #[test]
    fn the_proxy_hop_is_the_last_value_of_the_last_line() {
        let headers = |lines: &[&str]| {
            let mut map = HeaderMap::new();
            for line in lines {
                map.append(
                    "x-forwarded-for",
                    HeaderValue::from_str(line).expect("header"),
                );
            }
            map
        };
        let ip = |text: &str| text.parse::<IpAddr>().ok();
        assert_eq!(forwarded_for(&headers(&[])), None);
        assert_eq!(forwarded_for(&headers(&["203.0.113.9"])), ip("203.0.113.9"));
        assert_eq!(
            forwarded_for(&headers(&["6.6.6.6, 7.7.7.7, 203.0.113.9"])),
            ip("203.0.113.9"),
            "an appending proxy"
        );
        assert_eq!(
            forwarded_for(&headers(&["6.6.6.6", "203.0.113.9"])),
            ip("203.0.113.9"),
            "a proxy that adds its own line"
        );
        assert_eq!(
            forwarded_for(&headers(&["6.6.6.6", "7.7.7.7, 2001:db8::1"])),
            ip("2001:db8::1")
        );
        assert_eq!(forwarded_for(&headers(&["6.6.6.6", "not-an-ip"])), None);
    }
}
