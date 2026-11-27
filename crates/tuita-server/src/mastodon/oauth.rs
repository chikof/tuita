//! OAuth2 for Mastodon clients.
//!
//! A client registers itself (`POST /api/v1/apps`), sends the user to `/oauth/authorize` (the
//! web client's consent card), and trades the code it gets back for a token at `POST /oauth/token`.
//!
//! | Route | Who | What |
//! |---|---|---|
//! | `POST /api/v1/apps` | anyone, rate limited by address | registers an app, answers its client id and secret once |
//! | `GET /api/v1/apps/verify_credentials` | any OAuth token | the app behind the token |
//! | `GET /api/v1/oauth/authorization` | the web client, session cookie only | validates an authorization request and answers what the card shows |
//! | `GET /oauth/authorize` | a browser | the web client page |
//! | `POST /oauth/authorize` | the web client, session cookie only, CSRF guard | the user's answer; makes the code |
//! | `POST /oauth/token` | the client | `authorization_code` (PKCE `S256`, or the client secret) and `client_credentials` |
//! | `POST /oauth/revoke` | the client | deletes a token of the client |
//! | `GET /.well-known/oauth-authorization-server` | anyone | server metadata (RFC 8414) |
//!
//! The rules that matter for safety:
//!
//! - A redirect URI is stored as registered and compared by exact string equality, at the
//!   authorization request and again at the exchange. The answer is only ever redirected to a
//!   registered URI, and errors of the request itself are shown on the card, never redirected.
//! - Scopes: `read`, `write`, `follow` (stored as `write`) and `push` (accepted, dropped). A granular scope
//!   (`read:statuses`) or an unknown word is `invalid_scope`, and a request may not ask for more
//!   than the app registered.
//! - PKCE: a code made with a challenge is only redeemed with the matching `S256` verifier, so
//!   a client cannot be downgraded; `plain` is refused. A code made without one needs the client secret.
//! - Codes and secrets are stored as SHA-256 and compared in constant time; the code is single
//!   use and lives 10 minutes ([`crate::db::oauth`]).
//! - Nothing here logs a code, a secret or a token.

use std::borrow::Cow;
use std::net::{IpAddr, Ipv4Addr};

use axum::extract::{DefaultBodyLimit, State};
use axum::http::header::{AUTHORIZATION, CACHE_CONTROL, PRAGMA, WWW_AUTHENTICATE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::middleware::from_fn_with_state;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest as _, Sha256};
use url::{Host, Url};

use super::dto::{Application, CredentialApplication, Token};
use super::params::{Form, Params};
use crate::auth::bearer::{AppAuth, REFUSED, TokenAuth};
use crate::auth::{ApiJson, ApiQuery, CurrentUser, csrf};
use crate::db::oauth::{self, App, CodeExchange, Exchanged, NewApp};
use crate::db::sessions::ClientMeta;
use crate::dto::{OAuthAuthorizeDto, OAuthAuthorizeRequest, OAuthConsentDto, TokenScope};
use crate::error::AppError;
use crate::state::AppState;

/// The redirect URI that means: show the code to the user instead of sending it anywhere.
const OOB: &str = "urn:ietf:wg:oauth:2.0:oob";
/// The most redirect URIs one app may register.
const MAX_URIS: usize = 10;
/// The longest redirect URI.
const MAX_URI_LEN: usize = 2000;
/// The longest app name.
const MAX_NAME: usize = 100;
/// The longest `state` a request may carry.
const MAX_STATE: usize = 2000;
/// The longest authorization code we hash; real ones are 43 characters.
const MAX_CODE: usize = 200;
/// The largest body of an OAuth request.
const MAX_BODY: usize = 64 * 1024;
/// Schemes a redirect URI may never use: the browser would run or show what the client controls.
const BLOCKED_SCHEMES: [&str; 8] = [
    "javascript",
    "data",
    "file",
    "vbscript",
    "blob",
    "about",
    "view-source",
    "filesystem",
];
/// The scope words, in the order they are listed.
const SCOPE_WORDS: [&str; 4] = ["read", "write", "follow", "push"];

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/apps", post(register))
        .route("/apps/verify_credentials", get(verify_credentials))
        .route("/oauth/authorization", get(consent))
}

/// The routes outside the API prefix. Only `/oauth/authorize` gets the CSRF guard (it works
/// on the session cookie) and no CORS; the token, revocation and metadata endpoints are called
/// from other origins by design and use no cookies.
pub(super) fn outside(state: &AppState) -> Router<AppState> {
    let authorize = Router::new()
        .route("/oauth/authorize", post(authorize))
        // `GET /oauth/authorize` is the consent card: the web client's page.
        .layer(from_fn_with_state(state.clone(), csrf::guard));
    let open = Router::new()
        .route("/oauth/token", post(token))
        .route("/oauth/revoke", post(revoke))
        .route("/.well-known/oauth-authorization-server", get(metadata))
        .layer(super::cors());
    Router::new()
        .merge(authorize)
        .merge(open)
        .layer(DefaultBodyLimit::max(MAX_BODY))
}

// ------------------------------------------------------------------ errors

/// What an OAuth endpoint answers with when it refuses.
#[derive(Debug)]
enum OAuthError {
    /// An error of the protocol: `{"error": "<code>", "error_description": "<text>"}`, never cached.
    Protocol {
        status: StatusCode,
        code: &'static str,
        description: Cow<'static, str>,
    },
    /// Anything else: the ordinary error body.
    App(AppError),
}

impl From<AppError> for OAuthError {
    fn from(err: AppError) -> Self {
        Self::App(err)
    }
}

impl OAuthError {
    fn bad(code: &'static str, description: impl Into<Cow<'static, str>>) -> Self {
        Self::Protocol {
            status: StatusCode::BAD_REQUEST,
            code,
            description: description.into(),
        }
    }

    fn invalid_request(description: impl Into<Cow<'static, str>>) -> Self {
        Self::bad("invalid_request", description)
    }

    fn invalid_grant(description: impl Into<Cow<'static, str>>) -> Self {
        Self::bad("invalid_grant", description)
    }

    fn invalid_scope(error: &ScopeError) -> Self {
        Self::bad("invalid_scope", error.describe())
    }

    /// `401`, as RFC 6749 5.2 asks when the client could not be authenticated.
    fn invalid_client(description: impl Into<Cow<'static, str>>) -> Self {
        Self::Protocol {
            status: StatusCode::UNAUTHORIZED,
            code: "invalid_client",
            description: description.into(),
        }
    }
}

impl IntoResponse for OAuthError {
    fn into_response(self) -> Response {
        match self {
            Self::App(err) => err.into_response(),
            Self::Protocol {
                status,
                code,
                description,
            } => {
                let mut response = (
                    status,
                    Json(json!({"error": code, "error_description": description})),
                )
                    .into_response();
                no_store(response.headers_mut());
                if status == StatusCode::UNAUTHORIZED {
                    response.headers_mut().insert(
                        WWW_AUTHENTICATE,
                        HeaderValue::from_static("Basic realm=\"tuita\""),
                    );
                }
                response
            }
        }
    }
}

fn no_store(headers: &mut HeaderMap) {
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));
}

// ------------------------------------------------------------------ scopes

#[derive(Debug, PartialEq, Eq)]
enum ScopeError {
    Granular(String),
    Unknown(String),
    NotRegistered(String),
    PushOnly,
}

impl ScopeError {
    fn describe(&self) -> String {
        match self {
            Self::Granular(word) => {
                format!(
                    "The scope \u{201c}{word}\u{201d} is not supported. Use read, write or follow."
                )
            }
            Self::Unknown(word) => format!("The scope \u{201c}{word}\u{201d} is not known."),
            Self::NotRegistered(word) => {
                format!("The app did not register for the scope \u{201c}{word}\u{201d}.")
            }
            Self::PushOnly => {
                "The scope push alone grants nothing; there is no Web Push.".to_owned()
            }
        }
    }
}

/// The distinct scope words in `text`, in the order written.
fn scope_words(text: &str) -> Result<Vec<&'static str>, ScopeError> {
    let mut out: Vec<&'static str> = Vec::new();
    for word in text.split_whitespace() {
        if word.contains(':') {
            return Err(ScopeError::Granular(word.to_owned()));
        }
        let known = SCOPE_WORDS
            .iter()
            .find(|known| **known == word)
            .ok_or_else(|| ScopeError::Unknown(word.to_owned()))?;
        if !out.contains(known) {
            out.push(known);
        }
    }
    Ok(out)
}

/// The scopes an app registers: distinct known words in the canonical order; `read` for none.
fn registered_scopes(text: &str) -> Result<Vec<String>, ScopeError> {
    let mut words = scope_words(text)?;
    if words.is_empty() {
        words.push("read");
    }
    Ok(SCOPE_WORDS
        .iter()
        .filter(|word| words.contains(word))
        .map(|word| (*word).to_owned())
        .collect())
}

/// What a request for `text` is granted, given what the app `registered`: `read` and `write`
/// (`follow` is `write`; `push` is dropped). Asking for a word the app did not register is an error.
fn granted_scopes(text: &str, registered: &[String]) -> Result<Vec<TokenScope>, ScopeError> {
    let mut words = scope_words(text)?;
    if words.is_empty() {
        words.push("read");
    }
    if let Some(word) = words
        .iter()
        .find(|word| !registered.iter().any(|known| known == **word))
    {
        return Err(ScopeError::NotRegistered((*word).to_owned()));
    }
    let mut granted = Vec::new();
    if words.contains(&"read") {
        granted.push(TokenScope::Read);
    }
    if words.contains(&"write") || words.contains(&"follow") {
        granted.push(TokenScope::Write);
    }
    if granted.is_empty() {
        return Err(ScopeError::PushOnly);
    }
    Ok(granted)
}

fn scope_text(scopes: &[TokenScope]) -> String {
    scopes
        .iter()
        .map(|scope| scope.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

// ------------------------------------------------------------------ redirect URIs and PKCE

/// Why a redirect URI is refused, or `Ok`. Allowed: `https`, `http` to `localhost`, `127.0.0.1`
/// or `[::1]`, the out-of-band URI, and an app's own scheme (`app://callback`). Never allowed: a
/// fragment, credentials in the URI, whitespace, `http` to another host, and the schemes in
/// [`BLOCKED_SCHEMES`].
fn check_redirect_uri(uri: &str) -> Result<(), &'static str> {
    if uri == OOB {
        return Ok(());
    }
    if uri.is_empty() || uri.len() > MAX_URI_LEN {
        return Err("must be between 1 and 2000 characters");
    }
    if uri.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err("must not contain whitespace or control characters");
    }
    let url = Url::parse(uri).map_err(|_| "must be an absolute URI")?;
    if url.fragment().is_some() {
        return Err("must not have a fragment");
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("must not contain credentials");
    }
    match url.scheme() {
        "https" => url.host().map(|_| ()).ok_or("must name a host"),
        "http" => match url.host() {
            Some(Host::Domain("localhost")) => Ok(()),
            Some(Host::Ipv4(ip)) if ip == Ipv4Addr::LOCALHOST => Ok(()),
            Some(Host::Ipv6(ip)) if ip.is_loopback() => Ok(()),
            _ => Err("must use https, or http only for localhost"),
        },
        scheme if BLOCKED_SCHEMES.contains(&scheme) => Err("must not use this scheme"),
        scheme => {
            if uri[scheme.len()..].starts_with("://") {
                Ok(())
            } else {
                Err("must be https, or an app scheme written as scheme://")
            }
        }
    }
}

/// Where the answer goes, for the user to read: `host[:port]` of an `http(s)` URI, `scheme://host`
/// of an app's own scheme, empty for the out-of-band URI.
fn redirect_host(uri: &str) -> String {
    if uri == OOB {
        return String::new();
    }
    let Ok(url) = Url::parse(uri) else {
        return uri.to_owned();
    };
    let host = url.host_str().unwrap_or_default();
    match (url.scheme(), url.port()) {
        ("http" | "https", Some(port)) => format!("{host}:{port}"),
        ("http" | "https", None) => host.to_owned(),
        (scheme, _) => format!("{scheme}://{host}"),
    }
}

/// `base` with `pairs` added to its query string, encoded. An existing query is kept.
fn redirect_with(base: &str, pairs: &[(&str, &str)]) -> Result<String, AppError> {
    let mut url = Url::parse(base).map_err(|err| {
        AppError::Internal(anyhow::Error::new(err).context("a stored redirect URI does not parse"))
    })?;
    url.query_pairs_mut().extend_pairs(pairs);
    Ok(url.into())
}

/// The PKCE challenge a verifier makes (`S256`: base64url of its SHA-256), or `None` when the
/// verifier does not have the shape of RFC 7636 4.1: 43 to 128 unreserved characters.
fn s256(verifier: &str) -> Option<String> {
    let shaped = (43..=128).contains(&verifier.len())
        && verifier
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~'));
    shaped.then(|| URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())))
}

/// Whether `challenge` can be an `S256` challenge: 43 base64url characters.
fn is_challenge(challenge: &str) -> bool {
    challenge.len() == 43
        && challenge
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}

// ------------------------------------------------------------------ the authorization request

/// The parameters of an authorization request, from the query of the consent call or the
/// body of the answer.
struct Request<'a> {
    client_id: Option<&'a str>,
    redirect_uri: Option<&'a str>,
    scope: Option<&'a str>,
    state: Option<&'a str>,
    response_type: Option<&'a str>,
    code_challenge: Option<&'a str>,
    code_challenge_method: Option<&'a str>,
}

/// An authorization request that passed every check.
struct Checked {
    app: App,
    redirect_uri: String,
    scopes: Vec<TokenScope>,
    challenge: Option<String>,
}

fn present(value: Option<&str>) -> Option<&str> {
    value.filter(|value| !value.is_empty())
}

/// Checks a request in the order that keeps errors safe: the client and the redirect URI first
/// (nothing may be redirected before both are known good), then the rest. Every refusal is
/// shown to the user; none is redirected.
async fn check_request(state: &AppState, request: &Request<'_>) -> Result<Checked, OAuthError> {
    let client_id = present(request.client_id)
        .ok_or_else(|| OAuthError::invalid_request("client_id is required."))?;
    let app = oauth::app_by_client_id(state.db(), client_id)
        .await?
        .ok_or_else(|| OAuthError::bad("invalid_client", "The client is not known."))?;
    let redirect_uri = match present(request.redirect_uri) {
        Some(uri) if app.redirect_uris.iter().any(|registered| registered == uri) => uri.to_owned(),
        Some(_) => {
            return Err(OAuthError::invalid_request(
                "The redirect URI is not registered for this app.",
            ));
        }
        None => match app.redirect_uris.as_slice() {
            [only] => only.clone(),
            _ => return Err(OAuthError::invalid_request("redirect_uri is required.")),
        },
    };
    match present(request.response_type) {
        Some("code") => {}
        Some(_) => {
            return Err(OAuthError::bad(
                "unsupported_response_type",
                "Only response_type=code is supported.",
            ));
        }
        None => return Err(OAuthError::invalid_request("response_type is required.")),
    }
    let scopes = granted_scopes(request.scope.unwrap_or_default(), &app.scopes)
        .map_err(|err| OAuthError::invalid_scope(&err))?;
    if request.state.is_some_and(|value| value.len() > MAX_STATE) {
        return Err(OAuthError::invalid_request("state is too long."));
    }
    let challenge = match (
        present(request.code_challenge),
        present(request.code_challenge_method),
    ) {
        (Some(challenge), Some("S256")) if is_challenge(challenge) => Some(challenge.to_owned()),
        (Some(_), Some("S256")) => {
            return Err(OAuthError::invalid_request(
                "code_challenge is not a valid S256 challenge.",
            ));
        }
        (Some(_), _) => {
            return Err(OAuthError::invalid_request(
                "code_challenge_method must be S256.",
            ));
        }
        (None, Some(_)) => {
            return Err(OAuthError::invalid_request(
                "code_challenge_method needs a code_challenge.",
            ));
        }
        (None, None) => None,
    };
    Ok(Checked {
        app,
        redirect_uri,
        scopes,
        challenge,
    })
}

// ------------------------------------------------------------------ the web client's two calls

#[derive(Deserialize)]
struct ConsentQuery {
    client_id: Option<String>,
    redirect_uri: Option<String>,
    scope: Option<String>,
    response_type: Option<String>,
    code_challenge: Option<String>,
    code_challenge_method: Option<String>,
    state: Option<String>,
}

/// `GET /api/v1/oauth/authorization`: validates what the consent card is about to ask and says
/// what to show. Session cookie only: the bearer gate refuses tokens on this path.
async fn consent(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiQuery(query): ApiQuery<ConsentQuery>,
) -> Result<Json<OAuthConsentDto>, OAuthError> {
    auth.session_id()?;
    let checked = check_request(
        &state,
        &Request {
            client_id: query.client_id.as_deref(),
            redirect_uri: query.redirect_uri.as_deref(),
            scope: query.scope.as_deref(),
            state: query.state.as_deref(),
            response_type: query.response_type.as_deref(),
            code_challenge: query.code_challenge.as_deref(),
            code_challenge_method: query.code_challenge_method.as_deref(),
        },
    )
    .await?;
    Ok(Json(OAuthConsentDto {
        app_name: checked.app.name,
        website: checked.app.website,
        scopes: checked.scopes,
        redirect_host: redirect_host(&checked.redirect_uri),
        oob: checked.redirect_uri == OOB,
        redirect_uri: checked.redirect_uri,
        pkce: checked.challenge.is_some(),
    }))
}

/// `POST /oauth/authorize`: the user's answer, from the web client with the session cookie.
///
/// The request is validated again, so only a registered redirect URI is ever answered, whatever
/// the card showed. An `Authorization` header is refused: a token can neither approve an app nor
/// be turned into a code. The CSRF guard is part of the route, see [`outside`].
async fn authorize(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    headers: HeaderMap,
    ApiJson(body): ApiJson<OAuthAuthorizeRequest>,
) -> Result<Json<OAuthAuthorizeDto>, OAuthError> {
    if headers.contains_key(AUTHORIZATION) {
        return Err(AppError::forbidden(REFUSED).into());
    }
    auth.session_id()?;
    state.limits().check_oauth_authorize(auth.user.id)?;
    let checked = check_request(
        &state,
        &Request {
            client_id: Some(&body.client_id),
            redirect_uri: body.redirect_uri.as_deref(),
            scope: body.scope.as_deref(),
            state: body.state.as_deref(),
            response_type: body.response_type.as_deref(),
            code_challenge: body.code_challenge.as_deref(),
            code_challenge_method: body.code_challenge_method.as_deref(),
        },
    )
    .await?;
    let oob = checked.redirect_uri == OOB;
    let echoed_state = present(body.state.as_deref());
    if !body.approve {
        if oob {
            return Ok(Json(OAuthAuthorizeDto {
                redirect: None,
                code: None,
            }));
        }
        let mut pairs = vec![
            ("error", "access_denied"),
            (
                "error_description",
                "The resource owner or authorization server denied the request.",
            ),
        ];
        pairs.extend(echoed_state.map(|value| ("state", value)));
        return Ok(Json(OAuthAuthorizeDto {
            redirect: Some(redirect_with(&checked.redirect_uri, &pairs)?),
            code: None,
        }));
    }
    let code = oauth::create_code(
        state.db(),
        checked.app.id,
        auth.user.id,
        &checked.scopes,
        &checked.redirect_uri,
        checked.challenge.as_deref(),
    )
    .await?;
    if oob {
        return Ok(Json(OAuthAuthorizeDto {
            redirect: None,
            code: Some(code),
        }));
    }
    let mut pairs = vec![("code", code.as_str())];
    pairs.extend(echoed_state.map(|value| ("state", value)));
    Ok(Json(OAuthAuthorizeDto {
        redirect: Some(redirect_with(&checked.redirect_uri, &pairs)?),
        code: None,
    }))
}

// ------------------------------------------------------------------ apps

fn application(app: &App) -> Application {
    Application {
        id: app.id,
        name: app.name.clone(),
        website: app.website.clone(),
        scopes: app.scopes.clone(),
        redirect_uris: app.redirect_uris.clone(),
        redirect_uri: app.redirect_uris.join("\n"),
        vapid_key: String::new(),
    }
}

fn ip_of(meta: &ClientMeta) -> IpAddr {
    meta.ip.unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
}

fn invalid(what: &str) -> AppError {
    AppError::unprocessable(format!("Validation failed: {what}"))
}

/// `POST /api/v1/apps`: `client_name`, `redirect_uris` (a list, or one string with several
/// URIs separated by spaces or newlines), `scopes` (default `read`) and `website`, as JSON or a form.
async fn register(
    State(state): State<AppState>,
    meta: ClientMeta,
    Form { params, .. }: Form,
) -> Result<Json<CredentialApplication>, AppError> {
    state.limits().check_oauth_app(ip_of(&meta))?;
    let name = params.get("client_name").map(str::trim).unwrap_or_default();
    if name.is_empty() {
        return Err(invalid("Name can't be blank"));
    }
    if name.chars().count() > MAX_NAME || name.chars().any(char::is_control) {
        return Err(invalid("Name is invalid"));
    }
    let mut uris: Vec<String> = Vec::new();
    for uri in params
        .list("redirect_uris")
        .iter()
        .flat_map(|value| value.split_whitespace())
    {
        if !uris.iter().any(|known| known == uri) {
            uris.push(uri.to_owned());
        }
    }
    if uris.is_empty() {
        return Err(invalid("Redirect URI can't be blank"));
    }
    if uris.len() > MAX_URIS {
        return Err(invalid("Redirect URI has too many entries"));
    }
    for uri in &uris {
        check_redirect_uri(uri).map_err(|why| invalid(&format!("Redirect URI {why}")))?;
    }
    let scopes = registered_scopes(&params.list("scopes").join(" "))
        .map_err(|err| invalid(&format!("Scopes: {}", err.describe())))?;
    let website = website(&params)?;
    let (app, secret) = oauth::create_app(
        state.db(),
        &NewApp {
            name,
            redirect_uris: &uris,
            scopes: &scopes,
            website: website.as_deref(),
        },
    )
    .await?;
    Ok(Json(CredentialApplication {
        application: application(&app),
        client_id: app.client_id,
        client_secret: secret,
        client_secret_expires_at: 0,
    }))
}

fn website(params: &Params) -> Result<Option<String>, AppError> {
    let Some(site) = params
        .get("website")
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return Ok(None);
    };
    let fine = site.len() <= MAX_URI_LEN
        && Url::parse(site)
            .is_ok_and(|url| matches!(url.scheme(), "http" | "https") && url.host().is_some());
    if fine {
        Ok(Some(site.to_owned()))
    } else {
        Err(invalid("Website is invalid"))
    }
}

/// `GET /api/v1/apps/verify_credentials`: the app behind the token. Any OAuth token will do,
/// a `client_credentials` one included; the bearer gate lets both through here.
async fn verify_credentials(
    State(state): State<AppState>,
    user_token: Option<Extension<TokenAuth>>,
    app_token: Option<Extension<AppAuth>>,
) -> Result<Json<Application>, AppError> {
    let app = app_token
        .map(|Extension(token)| token.app_id)
        .or_else(|| user_token.and_then(|Extension(token)| token.app_id))
        .ok_or(AppError::Denied("The access token is invalid."))?;
    let app = oauth::app_by_id(state.db(), app)
        .await?
        .ok_or(AppError::Missing("App not found."))?;
    Ok(Json(application(&app)))
}

// ------------------------------------------------------------------ the token endpoint

/// The client id and secret a request carries: HTTP Basic if present, else the body fields.
/// An empty secret is none.
fn client_credentials(headers: &HeaderMap, params: &Params) -> (Option<String>, Option<String>) {
    let (id, secret) = basic_credentials(headers).map_or_else(
        || {
            (
                params.get("client_id").map(str::to_owned),
                params.get("client_secret").map(str::to_owned),
            )
        },
        |(id, secret)| (Some(id), Some(secret)),
    );
    (id, secret.filter(|secret| !secret.is_empty()))
}

fn basic_credentials(headers: &HeaderMap) -> Option<(String, String)> {
    let value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let (scheme, rest) = value.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("basic") {
        return None;
    }
    let decoded = String::from_utf8(STANDARD.decode(rest.trim()).ok()?).ok()?;
    let (id, secret) = decoded.split_once(':')?;
    Some((id.to_owned(), secret.to_owned()))
}

/// The app a request is from. A secret that is present must be right, whether or not the
/// grant needs it. Returns whether a secret was present and right.
async fn authenticate(
    state: &AppState,
    id: Option<&str>,
    secret: Option<&str>,
) -> Result<(App, bool), OAuthError> {
    let id = present(id).ok_or_else(|| OAuthError::invalid_request("client_id is required."))?;
    let app = oauth::app_by_client_id(state.db(), id)
        .await?
        .ok_or_else(|| OAuthError::invalid_client("The client is not known."))?;
    match secret {
        Some(secret) if app.secret_matches(secret) => Ok((app, true)),
        Some(_) => Err(OAuthError::invalid_client("The client secret is wrong.")),
        None => Ok((app, false)),
    }
}

fn token_response(issued: &oauth::Issued) -> Response {
    let mut response = Json(Token {
        access_token: issued.token.clone(),
        token_type: "Bearer",
        scope: scope_text(&issued.scopes),
        created_at: issued.created_at.unix_timestamp(),
    })
    .into_response();
    no_store(response.headers_mut());
    response
}

/// `POST /oauth/token`: `authorization_code` and `client_credentials`.
async fn token(
    State(state): State<AppState>,
    meta: ClientMeta,
    headers: HeaderMap,
    Form { params, .. }: Form,
) -> Result<Response, OAuthError> {
    state.limits().check_oauth_token(ip_of(&meta))?;
    let (id, secret) = client_credentials(&headers, &params);
    let grant_type = present(params.get("grant_type"))
        .ok_or_else(|| OAuthError::invalid_request("grant_type is required."))?;
    let (app, secret_verified) = authenticate(&state, id.as_deref(), secret.as_deref()).await?;
    match grant_type {
        "authorization_code" => {
            let code = present(params.get("code"))
                .ok_or_else(|| OAuthError::invalid_request("code is required."))?;
            if code.len() > MAX_CODE {
                return Err(OAuthError::invalid_grant(
                    "The authorization code is not valid.",
                ));
            }
            let verifier_challenge =
                match present(params.get("code_verifier")) {
                    Some(verifier) => Some(s256(verifier).ok_or_else(|| {
                        OAuthError::invalid_grant("The code verifier is not valid.")
                    })?),
                    None => None,
                };
            let exchanged = oauth::exchange(
                state.db(),
                &CodeExchange {
                    app: &app,
                    code,
                    redirect_uri: present(params.get("redirect_uri")),
                    verifier_challenge: verifier_challenge.as_deref(),
                    secret_verified,
                },
            )
            .await?;
            match exchanged {
                Exchanged::Issued(issued) => Ok(token_response(&issued)),
                Exchanged::Refused(why) => Err(OAuthError::invalid_grant(why)),
            }
        }
        "client_credentials" => {
            if !secret_verified {
                return Err(OAuthError::invalid_client(
                    "client_credentials needs the client secret.",
                ));
            }
            let scopes = granted_scopes(params.get("scope").unwrap_or_default(), &app.scopes)
                .map_err(|err| OAuthError::invalid_scope(&err))?;
            let issued = oauth::issue_app_token(state.db(), &app, scopes).await?;
            Ok(token_response(&issued))
        }
        _ => Err(OAuthError::bad(
            "unsupported_grant_type",
            "Only authorization_code and client_credentials are supported.",
        )),
    }
}

/// `POST /oauth/revoke`: deletes the token `token` when it belongs to the client. Answers `{}`
/// whether or not there was one (RFC 7009), so it cannot be used to probe for tokens.
async fn revoke(
    State(state): State<AppState>,
    meta: ClientMeta,
    headers: HeaderMap,
    Form { params, .. }: Form,
) -> Result<Response, OAuthError> {
    state.limits().check_oauth_token(ip_of(&meta))?;
    let (id, secret) = client_credentials(&headers, &params);
    let (app, _) = authenticate(&state, id.as_deref(), secret.as_deref()).await?;
    let token = present(params.get("token"))
        .ok_or_else(|| OAuthError::invalid_request("token is required."))?;
    oauth::revoke(state.db(), app.id, token).await?;
    let mut response = Json(json!({})).into_response();
    no_store(response.headers_mut());
    Ok(response)
}

/// `GET /.well-known/oauth-authorization-server` (RFC 8414).
async fn metadata(State(state): State<AppState>) -> Response {
    let base = &state.config().public_url;
    let mut response = Json(json!({
        "issuer": base,
        "authorization_endpoint": format!("{base}/oauth/authorize"),
        "token_endpoint": format!("{base}/oauth/token"),
        "revocation_endpoint": format!("{base}/oauth/revoke"),
        "app_registration_endpoint": format!("{base}/api/v1/apps"),
        "scopes_supported": SCOPE_WORDS,
        "response_types_supported": ["code"],
        "response_modes_supported": ["query"],
        "grant_types_supported": ["authorization_code", "client_credentials"],
        "token_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post", "none"],
        "code_challenge_methods_supported": ["S256"],
    }))
    .into_response();
    response.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=3600"),
    );
    response
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn scopes_are_registered_in_canonical_order_without_duplicates() {
        assert_eq!(
            registered_scopes("push write read read").expect("valid"),
            ["read", "write", "push"]
        );
        assert_eq!(registered_scopes("").expect("default"), ["read"]);
        assert_eq!(
            registered_scopes("read:statuses"),
            Err(ScopeError::Granular("read:statuses".to_owned()))
        );
        assert_eq!(
            registered_scopes("read admin"),
            Err(ScopeError::Unknown("admin".to_owned()))
        );
    }

    #[test]
    fn a_request_is_granted_read_and_write_and_never_more_than_registered() {
        let all: Vec<String> = ["read", "write", "follow", "push"]
            .map(str::to_owned)
            .to_vec();
        assert_eq!(
            granted_scopes("read write follow push", &all),
            Ok(vec![TokenScope::Read, TokenScope::Write])
        );
        assert_eq!(granted_scopes("follow", &all), Ok(vec![TokenScope::Write]));
        assert_eq!(granted_scopes("", &all), Ok(vec![TokenScope::Read]));
        assert_eq!(granted_scopes("push", &all), Err(ScopeError::PushOnly));
        assert_eq!(
            granted_scopes("write:statuses", &all),
            Err(ScopeError::Granular("write:statuses".to_owned()))
        );
        let read_only = vec!["read".to_owned()];
        assert_eq!(
            granted_scopes("read write", &read_only),
            Err(ScopeError::NotRegistered("write".to_owned()))
        );
        assert_eq!(
            granted_scopes("follow", &["write".to_owned()]),
            Err(ScopeError::NotRegistered("follow".to_owned()))
        );
        assert_eq!(
            granted_scopes("messages", &all),
            Err(ScopeError::Unknown("messages".to_owned()))
        );
    }

    #[test]
    fn redirect_uris_follow_the_registration_rules() {
        for good in [
            "https://phanpy.social/",
            "https://example.com/cb?x=1",
            "http://localhost:3000/cb",
            "http://127.0.0.1/cb",
            "http://[::1]:8080/cb",
            "urn:ietf:wg:oauth:2.0:oob",
            "tusky://oauth-callback",
            "org.joinmastodon.app://callback/",
            "ivory:///request_token",
        ] {
            assert_eq!(check_redirect_uri(good), Ok(()), "{good}");
        }
        for bad in [
            "",
            "not a uri",
            "/relative",
            "http://example.com/cb",
            "http://localhost.evil.example/cb",
            "http://127.0.0.2/cb",
            "https://example.com/cb#frag",
            "https://user:pass@example.com/cb",
            "javascript://%0aalert(1)",
            "JAVASCRIPT://x",
            "data://text/html,hi",
            "file:///etc/passwd",
            "vbscript://x",
            "blob://x",
            "about://blank",
            "javascript:alert(1)",
            "com.example.app:/callback",
            "https://example.com/a b",
            "https://example.com/a\nb",
        ] {
            assert!(check_redirect_uri(bad).is_err(), "{bad}");
        }
        assert!(check_redirect_uri(&format!("https://e.example/{}", "a".repeat(2000))).is_err());
    }

    #[test]
    fn the_card_shows_where_the_code_goes() {
        assert_eq!(redirect_host("https://phanpy.social/cb"), "phanpy.social");
        assert_eq!(redirect_host("http://localhost:3000/cb"), "localhost:3000");
        assert_eq!(
            redirect_host("tusky://oauth-callback"),
            "tusky://oauth-callback"
        );
        assert_eq!(redirect_host(OOB), "");
    }

    #[test]
    fn the_answer_url_is_encoded_and_keeps_the_registered_query() {
        assert_eq!(
            redirect_with(
                "https://app.example/cb?tenant=a%20b",
                &[("code", "abc"), ("state", "x y&z=1#2")]
            )
            .expect("url"),
            "https://app.example/cb?tenant=a%20b&code=abc&state=x+y%26z%3D1%232"
        );
        assert_eq!(
            redirect_with("tusky://oauth-callback", &[("code", "abc")]).expect("url"),
            "tusky://oauth-callback?code=abc"
        );
        assert!(redirect_with("not a url", &[]).is_err());
    }

    #[test]
    fn s256_matches_the_rfc_7636_example() {
        assert_eq!(
            s256("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk").as_deref(),
            Some("E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM")
        );
        assert!(is_challenge("E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"));
        assert_eq!(s256("short"), None);
        assert_eq!(s256(&"a".repeat(129)), None);
        assert_eq!(s256(&format!("{}!", "a".repeat(43))), None);
        assert!(!is_challenge("E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-c="));
        assert!(!is_challenge("tooshort"));
    }

    #[test]
    fn basic_credentials_are_read_when_the_scheme_is_basic() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Basic {}", STANDARD.encode("id:se:cret")))
                .expect("ascii"),
        );
        assert_eq!(
            basic_credentials(&headers),
            Some(("id".to_owned(), "se:cret".to_owned()))
        );
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer abc"));
        assert_eq!(basic_credentials(&headers), None);
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Basic !!!"));
        assert_eq!(basic_credentials(&headers), None);
    }

    #[test]
    fn an_empty_secret_is_no_secret_and_the_header_wins() {
        let params = Params::from_query(Some("client_id=body&client_secret="));
        assert_eq!(
            client_credentials(&HeaderMap::new(), &params),
            (Some("body".to_owned()), None)
        );
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Basic {}", STANDARD.encode("header:s")))
                .expect("ascii"),
        );
        assert_eq!(
            client_credentials(&headers, &params),
            (Some("header".to_owned()), Some("s".to_owned()))
        );
    }
}
