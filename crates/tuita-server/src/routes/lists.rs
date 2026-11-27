//! Lists: curated groups of accounts with their own timeline.

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use super::auth::text_field;
use crate::auth::{ApiJson, ApiQuery, Auth, CurrentUser};
use crate::cursor::{PageQuery, parse_id};
use crate::db::graph::blocked_between;
use crate::db::{lists, settings};
use crate::dto::{ListDto, ListRequest, OkDto, Page, PatchListRequest, PostDto, UserDto};
use crate::error::AppError;
use crate::federation::lookup::{find_user, find_user_with};
use crate::filter::ContentFilter;
use crate::mastodon::{self, by_credential};
use crate::state::AppState;

const NOT_FOUND: &str = "List not found.";
const OWNER_ONLY: &str = "Only the list owner can change it.";

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/lists",
            get(by_credential(mine, mastodon::lists::mine))
                .post(by_credential(create, mastodon::lists::create)),
        )
        .route(
            "/lists/{id}",
            get(by_credential(show, mastodon::lists::show))
                .patch(change)
                .delete(by_credential(remove, mastodon::lists::remove)),
        )
        .route("/lists/{id}/timeline", get(timeline))
        .route("/lists/{id}/members", get(members))
        .route(
            "/lists/{id}/members/{handle}",
            post(add_member).delete(remove_member),
        )
}

#[derive(Deserialize)]
struct MineQuery {
    member: Option<String>,
}

/// The list `id` as the caller may see it.
async fn viewable(state: &AppState, auth: &Auth, id: &str) -> Result<ListDto, AppError> {
    let id = parse_id(id, NOT_FOUND)?;
    lists::find(state.db(), auth.user.id, id)
        .await?
        .ok_or(AppError::Missing(NOT_FOUND))
}

/// The list `id`, when the caller owns it.
async fn owned(state: &AppState, auth: &Auth, id: &str) -> Result<ListDto, AppError> {
    let list = viewable(state, auth, id).await?;
    if list.is_owner {
        Ok(list)
    } else {
        Err(AppError::forbidden(OWNER_ONLY))
    }
}

/// Rejects a name or description the word filter blocks.
fn check_words(
    filter: &ContentFilter,
    name: Option<&str>,
    description: Option<&str>,
) -> Result<(), AppError> {
    let text = format!(
        "{} {}",
        name.unwrap_or_default(),
        description.unwrap_or_default()
    );
    if filter.blocks_text(&text) {
        return Err(AppError::bad_request(
            "That list name or description isn\u{2019}t allowed.",
        ));
    }
    Ok(())
}

async fn mine(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiQuery(query): ApiQuery<MineQuery>,
) -> Result<Json<Vec<ListDto>>, AppError> {
    let member = match query.member.as_deref().filter(|handle| !handle.is_empty()) {
        Some(handle) => Some(find_user(&state, &auth.user, handle).await?.id),
        None => None,
    };
    Ok(Json(lists::owned(state.db(), auth.user.id, member).await?))
}

async fn create(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiJson(body): ApiJson<ListRequest>,
) -> Result<Json<ListDto>, AppError> {
    state.limits().check_lists(auth.user.id)?;
    let name = text_field(&body.name, "List name", 1, 40)?;
    let description = text_field(&body.description, "Description", 0, 160)?;
    let filter = settings::content_filter(state.db()).await?;
    check_words(&filter, Some(&name), Some(&description))?;
    let list = lists::create(state.db(), auth.user.id, &name, &description, body.private).await?;
    Ok(Json(list))
}

async fn show(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<ListDto>, AppError> {
    Ok(Json(viewable(&state, &auth, &id).await?))
}

async fn change(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    ApiJson(body): ApiJson<PatchListRequest>,
) -> Result<Json<ListDto>, AppError> {
    let filter = settings::content_filter(state.db()).await?;
    let list = owned(&state, &auth, &id).await?;
    let name = body
        .name
        .as_deref()
        .map(|name| text_field(name, "List name", 1, 40))
        .transpose()?;
    let description = body
        .description
        .as_deref()
        .map(|description| text_field(description, "Description", 0, 160))
        .transpose()?;
    check_words(&filter, name.as_deref(), description.as_deref())?;
    lists::update(
        state.db(),
        list.id,
        name.as_deref(),
        description.as_deref(),
        body.private,
    )
    .await?;
    Ok(Json(viewable(&state, &auth, &id).await?))
}

async fn remove(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<OkDto>, AppError> {
    let list = owned(&state, &auth, &id).await?;
    lists::delete(state.db(), list.id).await?;
    Ok(Json(OkDto { ok: true }))
}

async fn timeline(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    ApiQuery(query): ApiQuery<PageQuery>,
) -> Result<Json<Page<PostDto>>, AppError> {
    let list = viewable(&state, &auth, &id).await?;
    let page = lists::timeline(
        state.db(),
        auth.user.id,
        list.id,
        query.id_cursor()?,
        query.limit()?,
    )
    .await?;
    Ok(Json(page))
}

async fn members(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    ApiQuery(query): ApiQuery<PageQuery>,
) -> Result<Json<Page<UserDto>>, AppError> {
    let list = viewable(&state, &auth, &id).await?;
    let page = lists::members(
        state.db(),
        auth.user.id,
        list.id,
        query.edge_cursor()?,
        query.limit()?,
    )
    .await?;
    Ok(Json(page))
}

async fn add_member(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path((id, handle)): Path<(String, String)>,
) -> Result<Json<OkDto>, AppError> {
    let list = owned(&state, &auth, &id).await?;
    let user = find_user(&state, &auth.user, &handle).await?;
    if blocked_between(state.db(), auth.user.id, user.id).await? {
        return Err(AppError::forbidden("You can\u{2019}t add this account."));
    }
    lists::add_member(state.db(), list.id, user.id).await?;
    Ok(Json(OkDto { ok: true }))
}

async fn remove_member(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path((id, handle)): Path<(String, String)>,
) -> Result<Json<OkDto>, AppError> {
    let list = owned(&state, &auth, &id).await?;
    // Staff view: a suspended member must stay removable.
    let user = find_user_with(&state, auth.user.id, true, &handle).await?;
    lists::remove_member(state.db(), list.id, user.id).await?;
    Ok(Json(OkDto { ok: true }))
}
