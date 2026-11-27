//! `/tokens`: personal API tokens for scripts and bots. Cookie-only: the bearer gate refuses
//! these paths, and `session_id()` refuses again.

use axum::extract::{Path, State};
use axum::routing::{delete, get};
use axum::{Json, Router};

use super::auth::text_field;
use crate::auth::{ApiJson, CurrentUser};
use crate::cursor::parse_id;
use crate::db::tokens;
use crate::dto::{CreateTokenRequest, CreatedTokenDto, OkDto, TokenDto, TokenScope};
use crate::error::AppError;
use crate::state::AppState;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/tokens", get(list).post(create))
        .route("/tokens/{id}", delete(remove))
}

async fn list(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
) -> Result<Json<Vec<TokenDto>>, AppError> {
    auth.session_id()?;
    Ok(Json(tokens::list(state.db(), auth.user.id).await?))
}

async fn create(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiJson(body): ApiJson<CreateTokenRequest>,
) -> Result<Json<CreatedTokenDto>, AppError> {
    auth.session_id()?;
    state.limits().check_token_create(auth.user.id)?;
    let name = text_field(&body.name, "Name", 1, 40)?;
    let mut scopes: Vec<TokenScope> = Vec::new();
    for scope in body.scopes.unwrap_or_else(|| vec![TokenScope::Read]) {
        if !scopes.contains(&scope) {
            scopes.push(scope);
        }
    }
    if scopes.is_empty() {
        return Err(AppError::bad_request("Pick at least one scope."));
    }
    Ok(Json(
        tokens::create(state.db(), auth.user.id, &name, &scopes).await?,
    ))
}

async fn remove(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<OkDto>, AppError> {
    auth.session_id()?;
    let id = parse_id(&id, "Token not found.")?;
    if !tokens::delete(state.db(), auth.user.id, id).await? {
        return Err(AppError::Missing("Token not found."));
    }
    Ok(Json(OkDto { ok: true }))
}
