//! `tuita-server`: the HTTP server, database access and ActivityPub of tuita.
//!
//! The `tuita` binary stays thin; everything testable lives here:
//!
//! - [`Config`]: settings read from environment variables.
//! - [`AppState`]: database pool, config and shared services, shared by handlers.
//! - [`backend`] / [`router`]: the HTTP routes plus middleware, usable in-process (see `tests/http.rs`).
//! - [`serve`] / [`serve_on`]: bind, serve and drain on shutdown.
//! - [`AppError`]: the one error type handlers return; renders as `{"error": "..."}`.

pub mod admin;
pub mod auth;
mod config;
pub mod cursor;
pub mod db;
pub mod dto;
mod error;
pub mod federation;
pub mod filter;
pub mod limits;
pub mod mastodon;
pub mod media;
pub mod net;
pub mod realtime;
pub mod routes;
mod secrets;
pub mod seed;
mod state;
mod storage;
pub mod workers;

use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Context as _;
use axum::Router;
use axum::http::header::{
    CONTENT_SECURITY_POLICY as CSP_HEADER, REFERRER_POLICY, STRICT_TRANSPORT_SECURITY,
    X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS,
};
use axum::http::{HeaderName, HeaderValue, Request, StatusCode};
use axum::middleware::{Next, from_fn, from_fn_with_state, map_response};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, get_service};
use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

pub use config::{Config, ConfigError, LogFormat};
pub use error::AppError;
pub use state::AppState;
pub use storage::{S3, S3Credentials, Storage};

/// Schema migrations in `migrations/`, embedded at compile time.
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// The `Content-Security-Policy` of every response. `'unsafe-eval'` is there because the web
/// client compiles its `$(...)` expressions in the browser; no inline script runs.
pub const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; img-src 'self' data: blob:; media-src 'self' blob:; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-eval'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'";

const PERMISSIONS_POLICY: HeaderName = HeaderName::from_static("permissions-policy");

/// Opens the Postgres pool. Fails fast when the database does not answer.
///
/// # Errors
///
/// Fails when Postgres cannot be reached within 5 seconds.
pub async fn connect(config: &Config) -> anyhow::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(16)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.database_url)
        .await
        .context("cannot connect to DATABASE_URL")
}

/// Applies pending migrations.
///
/// # Errors
///
/// Fails when a migration fails or the database is ahead of this binary.
pub async fn migrate(db: &PgPool) -> anyhow::Result<()> {
    MIGRATOR.run(db).await.context("migration failed")
}

/// How long a media upload may take: a slow connection sending the largest video.
const UPLOAD_TIMEOUT: Duration = Duration::from_secs(600);

/// The time limit of one request: the configured one, except for media uploads.
fn timeout_for(method: &axum::http::Method, path: &str, default: Duration) -> Duration {
    if method == axum::http::Method::POST && matches!(path, "/api/v1/media" | "/api/v2/media") {
        default.max(UPLOAD_TIMEOUT)
    } else {
        default
    }
}

/// The path as the request log shows it: the handle of `/api/v1/conversations/{handle}...` is
/// replaced by `{handle}`, because who messages whom is not for a log. Every other path is
/// logged as it is.
fn logged_path(path: &str) -> std::borrow::Cow<'_, str> {
    const PREFIX: &str = "/api/v1/conversations/";
    match path.strip_prefix(PREFIX) {
        Some(rest) if !rest.is_empty() => {
            let tail = rest.find('/').map_or("", |slash| &rest[slash..]);
            std::borrow::Cow::Owned(format!("{PREFIX}{{handle}}{tail}"))
        }
        _ => std::borrow::Cow::Borrowed(path),
    }
}

/// The backend alone: `GET /healthz`, `GET /readyz`, the JSON API under `/api/v1` and `/api/v2`,
/// and the routes outside the API prefix (media, ActivityPub, OAuth, RSS). No fallback and no
/// middleware; the web client calls this in-process.
pub fn backend(state: &AppState) -> Router {
    Router::new()
        .route("/healthz", get(routes::health::healthz))
        .route("/readyz", get(routes::health::readyz))
        .nest("/api/v1", routes::api(state))
        .nest("/api/v2", routes::api_v2(state))
        .merge(routes::outside_api(state))
        .with_state(state.clone())
}

/// Builds the router with all routes and middleware.
///
/// Routes: [`backend`], and `web` for everything it does not match. Requests that reach `web`
/// pass `routes::fallback::gate` first: a path under a backend prefix gets a JSON 404, and
/// ActivityPub clients asking for `/@handle` get the actor document.
///
/// Middleware, outermost first:
///
/// 1. Request id: reuses an incoming `x-request-id` or sets a UUID.
/// 2. Trace: one span per request carrying method, path and request id. The handle of
///    `/api/v1/conversations/{handle}` is left out of the path.
/// 3. Request id is copied to the response `x-request-id` header.
/// 4. Security headers, including [`CONTENT_SECURITY_POLICY`], and
///    `Strict-Transport-Security` when `SECURE_COOKIES` is on.
/// 5. Timeout: a request still running after the configured limit ends with
///    `408 Request Timeout` and an empty body, so clients can tell it from the 503
///    readiness answer.
///    `POST /api/v1/media` gets at least 10 minutes, so large videos can upload.
pub fn router(state: AppState, web: Router) -> Router {
    let timeout = state.config().request_timeout();
    // Only when cookies are `Secure`: the instance is then reached over https, and a plain-http
    // dev instance must not pin its host to https. No `includeSubDomains`, no preload.
    let hsts = state.config().secure_cookies;
    let api = backend(&state);
    let gated = tower::Layer::layer(&from_fn_with_state(state, routes::fallback::gate), web);

    // The consent card is the web client's page at the path of the backend's `POST /oauth/authorize`;
    // a method router without `GET` would answer 405 instead of reaching the fallback.
    api.route("/oauth/authorize", get_service(gated.clone()))
        .fallback_service(gated)
        .layer(from_fn(
            move |request: axum::extract::Request, next: Next| async move {
                let limit = timeout_for(request.method(), request.uri().path(), timeout);
                match tokio::time::timeout(limit, next.run(request)).await {
                    Ok(response) => response,
                    Err(_) => StatusCode::REQUEST_TIMEOUT.into_response(),
                }
            },
        ))
        .layer(SetResponseHeaderLayer::overriding(
            CSP_HEADER,
            HeaderValue::from_static(CONTENT_SECURITY_POLICY),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            PERMISSIONS_POLICY,
            HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
        ))
        .layer(map_response(move |mut response: Response| async move {
            if hsts {
                response.headers_mut().insert(
                    STRICT_TRANSPORT_SECURITY,
                    HeaderValue::from_static("max-age=31536000"),
                );
            }
            response
        }))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(
            TraceLayer::new_for_http()
                .on_failure(log_failure)
                .make_span_with(|request: &Request<_>| {
                    let request_id = request
                        .headers()
                        .get("x-request-id")
                        .and_then(|value| value.to_str().ok())
                        .unwrap_or_default();
                    tracing::info_span!(
                        "request",
                        method = %request.method(),
                        path = %logged_path(request.uri().path()),
                        request_id,
                    )
                }),
        )
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
}

/// Binds the configured address and serves until `shutdown` completes.
///
/// # Errors
///
/// Fails when the address cannot be bound or the server stops with an I/O error.
pub async fn serve(
    state: AppState,
    web: Router,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    let bind_addr = state.config().bind_addr;
    let listener = TcpListener::bind(bind_addr)
        .await
        .with_context(|| format!("cannot bind {bind_addr}"))?;
    serve_on(listener, state, web, shutdown).await
}

/// Serves on an already bound `listener` until `shutdown` completes.
///
/// Shutdown order: `shutdown` fires, `/readyz` turns 503, the listener stops accepting,
/// in-flight requests finish (bounded by the request timeout), then this returns.
///
/// # Errors
///
/// Fails when the listener is broken or the server stops with an I/O error.
pub async fn serve_on(
    listener: TcpListener,
    state: AppState,
    web: Router,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    if state.config().source_url.is_none() {
        tracing::warn!("SOURCE_URL is not set; AGPL requires offering the source to users");
    }
    if state.config().federation_allow_private_nets {
        tracing::warn!(
            "FEDERATION_ALLOW_PRIVATE_NETS is on: federation fetches may reach loopback and private addresses; development and tests only"
        );
    }
    let app = router(state.clone(), web);
    let local_addr = listener
        .local_addr()
        .context("listener has no local address")?;
    let stop = state.shutdown().clone();
    let mut workers = workers::spawn_workers(&state, &stop);
    state.set_ready(true);
    tracing::info!(%local_addr, "listening");
    let draining = state.clone();
    let served = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        shutdown.await;
        draining.set_ready(false);
        // Ends open event streams and workers; without it the drain would wait on them forever.
        draining.shutdown().cancel();
        tracing::info!("shutdown requested, draining");
    })
    .await;
    stop.cancel();
    while let Some(done) = workers.join_next().await {
        if let Err(err) = done {
            tracing::error!(%err, "worker failed");
        }
    }
    served.context("server error")?;
    tracing::info!("stopped");
    Ok(())
}

/// Completes on SIGINT (Ctrl-C) or, on Unix, SIGTERM (what containers and systemd send).
pub async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = signal::ctrl_c().await {
            tracing::error!(%err, "cannot listen for SIGINT");
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(err) => {
                tracing::error!(%err, "cannot listen for SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}

/// Installs the global `tracing` subscriber on stderr. Filter comes from `RUST_LOG`, default `info`.
///
/// Call once, early in `main`. A second call is ignored.
pub fn init_tracing(format: LogFormat) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr);
    // `try_init` fails only when a subscriber is already set; keep the first one.
    let _ = match format {
        LogFormat::Pretty => builder.pretty().try_init(),
        LogFormat::Json => builder.json().try_init(),
    };
}

/// Logs a failed response. A 502 is a remote server that did not answer (a federation lookup, a
/// delivery): the instance is working, so it is a warning, not an error.
fn log_failure(failure: ServerErrorsFailureClass, latency: Duration, _span: &tracing::Span) {
    match failure {
        ServerErrorsFailureClass::StatusCode(StatusCode::BAD_GATEWAY) => {
            tracing::warn!(?latency, "response failed: 502 Bad Gateway");
        }
        failure => tracing::error!(classification = %failure, ?latency, "response failed"),
    }
}

#[cfg(test)]
mod test {
    use axum::http::Method;

    use super::*;

    #[test]
    fn only_media_uploads_get_the_long_timeout() {
        let default = Duration::from_secs(30);
        assert_eq!(
            timeout_for(&Method::POST, "/api/v1/media", default),
            UPLOAD_TIMEOUT
        );
        assert_eq!(
            timeout_for(&Method::POST, "/api/v2/media", default),
            UPLOAD_TIMEOUT,
            "the Mastodon upload route"
        );
        assert_eq!(timeout_for(&Method::PUT, "/api/v2/media", default), default);
        assert_eq!(timeout_for(&Method::GET, "/media/x", default), default);
        assert_eq!(
            timeout_for(&Method::POST, "/api/v1/posts", default),
            default
        );
        assert_eq!(
            timeout_for(&Method::PATCH, "/api/v1/media/x", default),
            default
        );
    }

    #[test]
    fn the_conversation_handle_is_not_logged() {
        assert_eq!(
            logged_path("/api/v1/conversations/bob"),
            "/api/v1/conversations/{handle}"
        );
        assert_eq!(
            logged_path("/api/v1/conversations/bob/read"),
            "/api/v1/conversations/{handle}/read"
        );
        assert_eq!(
            logged_path("/api/v1/conversations/bob/messages/42"),
            "/api/v1/conversations/{handle}/messages/42"
        );
        for untouched in [
            "/api/v1/conversations",
            "/api/v1/conversations/",
            "/api/v1/users/bob",
            "/api/v1/posts",
            "/media/x.png",
        ] {
            assert_eq!(logged_path(untouched), untouched);
        }
        assert!(!logged_path("/api/v1/conversations/bob/read").contains("bob"));
    }
}
