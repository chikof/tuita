//! Liveness and readiness probes.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::state::AppState;

/// Liveness: the process is up and the event loop answers. Never checks dependencies.
pub(crate) async fn healthz() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

/// Readiness: `200` while serving and the database answers within one second;
/// `503` before startup, while draining, or when Postgres is down.
pub(crate) async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    if state.is_healthy().await {
        (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "ready" })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "status": "not ready" })),
        )
    }
}
