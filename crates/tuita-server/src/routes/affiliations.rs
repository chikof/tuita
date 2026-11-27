//! `/affiliations` and `/orgs/{handle}/affiliates`: organizations and their members.

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};

use super::auth::text_field;
use crate::auth::{ApiJson, ApiQuery, CurrentUser};
use crate::cursor::{PageQuery, parse_id};
use crate::db::affiliations::{self, MAX_LABEL_CHARS};
use crate::db::notifications::deliver;
use crate::db::{accounts, users};
use crate::dto::{
    AffiliateDto, AffiliateRequest, AffiliationsDto, OkDto, Page, PatchAffiliationRequest,
};
use crate::error::AppError;
use crate::state::AppState;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/affiliations", get(overview).post(invite))
        .route("/affiliations/{id}/accept", post(accept))
        .route(
            "/affiliations/{id}",
            axum::routing::patch(relabel).delete(remove),
        )
        .route("/orgs/{handle}/affiliates", get(affiliates))
}

const GONE: &str = "No account with that handle on this instance.";

async fn overview(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
) -> Result<Json<AffiliationsDto>, AppError> {
    Ok(Json(affiliations::overview(state.db(), &auth.user).await?))
}

async fn invite(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiJson(body): ApiJson<AffiliateRequest>,
) -> Result<Json<OkDto>, AppError> {
    if !auth.user.is_org.unwrap_or(false) {
        return Err(AppError::forbidden(
            "Only organization accounts can add affiliates. Ask an admin to mark your account as an organization.",
        ));
    }
    state.limits().check_affiliate(auth.user.id)?;
    let handle = body.handle.trim().trim_start_matches('@');
    let target = accounts::find_for_login(state.db(), handle)
        .await?
        .filter(|target| target.suspended_at.is_none())
        .ok_or(AppError::Missing(GONE))?;
    if target.id == auth.user.id {
        return Err(AppError::bad_request(
            "An organization can\u{2019}t be affiliated with itself.",
        ));
    }
    if target.is_org.unwrap_or(false) {
        return Err(AppError::bad_request(
            "Organizations can\u{2019}t be affiliated with other organizations.",
        ));
    }
    let label = text_field(
        body.label.as_deref().unwrap_or_default(),
        "Label",
        0,
        MAX_LABEL_CHARS,
    )?;
    let mut tx = state.db().begin().await?;
    let pending = affiliations::invite(&mut tx, auth.user.id, target.id, &label).await?;
    tx.commit().await?;
    deliver(&state, pending).await;
    Ok(Json(OkDto { ok: true }))
}

async fn accept(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<OkDto>, AppError> {
    let id = parse_id(&id, "Invitation not found.")?;
    affiliations::accept(state.db(), auth.user.id, id).await?;
    Ok(Json(OkDto { ok: true }))
}

async fn relabel(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    ApiJson(body): ApiJson<PatchAffiliationRequest>,
) -> Result<Json<OkDto>, AppError> {
    let id = parse_id(&id, "Affiliation not found.")?;
    let label = text_field(&body.label, "Label", 0, MAX_LABEL_CHARS)?;
    affiliations::relabel(state.db(), auth.user.id, id, &label).await?;
    Ok(Json(OkDto { ok: true }))
}

async fn remove(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<OkDto>, AppError> {
    let id = parse_id(&id, "Affiliation not found.")?;
    affiliations::remove(state.db(), auth.user.id, id).await?;
    Ok(Json(OkDto { ok: true }))
}

async fn affiliates(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(handle): Path<String>,
    ApiQuery(paging): ApiQuery<PageQuery>,
) -> Result<Json<Page<AffiliateDto>>, AppError> {
    let org = users::find_local(state.db(), &handle, auth.user.is_staff()).await?;
    Ok(Json(
        affiliations::affiliates(
            state.db(),
            auth.user.id,
            org.id,
            paging.offset()?,
            paging.limit()?,
        )
        .await?,
    ))
}
