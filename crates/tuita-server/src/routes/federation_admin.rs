//! `/admin/federation...`: domain blocks, the delivery queue and the instance actor's key.
//! Admin only and cookie only (the bearer gate refuses tokens on every `/admin` path). The SQL
//! and the audit rows are in `db::federation`.

use axum::extract::{Path, State};
use axum::routing::{get, patch, post};
use axum::{Json, Router};

use crate::auth::{Admin, ApiJson};
use crate::db::federation as db;
use crate::dto::{
    CreateDomainBlockRequest, DomainBlockDto, FederationDto, InstanceActorDto, OkDto,
    PatchDomainBlockRequest, RetryDto,
};
use crate::error::AppError;
use crate::federation::{ids, keys};
use crate::state::AppState;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/federation", get(overview))
        .route("/admin/federation/blocks", post(create_block))
        .route(
            "/admin/federation/blocks/{domain}",
            patch(patch_block).delete(delete_block),
        )
        .route("/admin/federation/retry", post(retry))
}

/// The domain a path names; a segment that cannot be a domain names no block.
fn path_domain(segment: &str) -> Result<String, AppError> {
    db::parse_domain(segment).ok_or(AppError::Missing("Block not found."))
}

async fn overview(
    State(state): State<AppState>,
    Admin(_): Admin,
) -> Result<Json<FederationDto>, AppError> {
    let config = state.config();
    let actor_id = ids::instance_actor_id(config);
    let (known_domains, remote_accounts, remote_posts) = db::known(state.db()).await?;
    Ok(Json(FederationDto {
        domain: config.domain().to_owned(),
        queue: db::queue(state.db()).await?,
        actor: InstanceActorDto {
            key_id: ids::key_id(&actor_id),
            id: actor_id,
            fingerprint: keys::instance_fingerprint(&state).await?,
            public_key_pem: keys::instance_public_pem(&state).await?,
        },
        blocks: db::blocks(state.db()).await?,
        known_domains,
        remote_accounts,
        remote_posts,
    }))
}

async fn create_block(
    State(state): State<AppState>,
    Admin(auth): Admin,
    ApiJson(body): ApiJson<CreateDomainBlockRequest>,
) -> Result<Json<DomainBlockDto>, AppError> {
    let domain = db::new_block_domain(&body.domain, state.config().domain())?;
    let reason = db::check_reason(&body.reason)?;
    let mut tx = state.db().begin().await?;
    let block = db::block(&mut tx, auth.user.id, &domain, body.severity, &reason).await?;
    tx.commit().await?;
    Ok(Json(block))
}

async fn patch_block(
    State(state): State<AppState>,
    Admin(auth): Admin,
    Path(domain): Path<String>,
    ApiJson(body): ApiJson<PatchDomainBlockRequest>,
) -> Result<Json<DomainBlockDto>, AppError> {
    let domain = path_domain(&domain)?;
    let reason = body.reason.as_deref().map(db::check_reason).transpose()?;
    let mut tx = state.db().begin().await?;
    let block = db::patch_block(
        &mut tx,
        auth.user.id,
        &domain,
        body.severity,
        reason.as_deref(),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(block))
}

async fn delete_block(
    State(state): State<AppState>,
    Admin(auth): Admin,
    Path(domain): Path<String>,
) -> Result<Json<OkDto>, AppError> {
    let domain = path_domain(&domain)?;
    let mut tx = state.db().begin().await?;
    db::unblock(&mut tx, auth.user.id, &domain).await?;
    tx.commit().await?;
    Ok(Json(OkDto { ok: true }))
}

async fn retry(
    State(state): State<AppState>,
    Admin(auth): Admin,
) -> Result<Json<RetryDto>, AppError> {
    let mut tx = state.db().begin().await?;
    let requeued = db::retry_now(&mut tx, auth.user.id).await?;
    tx.commit().await?;
    Ok(Json(RetryDto { requeued }))
}
