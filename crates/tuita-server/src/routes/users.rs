//! Profiles, the follow, block and mute actions, the lists around them and suggestions.

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::{ApiQuery, CurrentUser};
use crate::cursor::PageQuery;
use crate::db::graph::{self, Edges};
use crate::db::notifications::deliver;
use crate::db::users::{self, UserRow};
use crate::dto::{KnownFollowersDto, MediaKind, Page, PostDto, ProfileTab, UserDto};
use crate::error::AppError;
use crate::federation::lookup::{find_user, forbid_suspended};
use crate::mastodon;
use crate::state::AppState;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/users/{handle}", get(profile))
        .route("/users/{handle}/posts", get(posts))
        .route("/users/{handle}/followers", get(followers))
        .route("/users/{handle}/following", get(following))
        .route("/users/{handle}/followed_by_known", get(followed_by_known))
        .route("/users/{handle}/follow", post(follow).delete(unfollow))
        .route("/users/{handle}/block", post(block).delete(unblock))
        .route("/users/{handle}/mute", post(mute).delete(unmute))
        .route("/me/blocks", get(blocks))
        .route("/me/mutes", get(mutes))
        .route(
            "/suggestions",
            get(mastodon::by_credential(
                suggestions,
                mastodon::misc::suggestions,
            )),
        )
}

/// The query of list routes that take only `before` and `limit`.
#[derive(Deserialize)]
struct ListQuery {
    before: Option<String>,
    limit: Option<u32>,
}

impl From<ListQuery> for PageQuery {
    fn from(query: ListQuery) -> Self {
        Self {
            before: query.before,
            limit: query.limit,
        }
    }
}

#[derive(Deserialize)]
struct TabQuery {
    tab: Option<String>,
    kind: Option<String>,
    before: Option<String>,
    limit: Option<u32>,
}

#[derive(Deserialize)]
struct SizeQuery {
    limit: Option<u32>,
}

/// Default 3, at most 20; a missing or zero `limit` means the default.
fn small_limit(raw: Option<u32>) -> i64 {
    match raw {
        None | Some(0) => 3,
        Some(n) => i64::from(n.min(20)),
    }
}

async fn profile(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(handle): Path<String>,
) -> Result<Json<UserDto>, AppError> {
    let target = find_user(&state, &auth.user, &handle).await?;
    Ok(Json(users::dto(state.db(), auth.user.id, &target).await?))
}

async fn posts(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(handle): Path<String>,
    ApiQuery(query): ApiQuery<TabQuery>,
) -> Result<Json<Page<PostDto>>, AppError> {
    let viewer = auth.user.id;
    let target = find_user(&state, &auth.user, &handle).await?;
    let paging = PageQuery {
        before: query.before,
        limit: query.limit,
    };
    let limit = paging.limit()?;
    if graph::blocked_between(state.db(), viewer, target.id).await? {
        return Ok(Json(Page {
            items: Vec::new(),
            next: None,
        }));
    }
    let tab = match query.tab.as_deref() {
        None => ProfileTab::Posts,
        Some(raw) => ProfileTab::parse(raw).ok_or_else(|| AppError::bad_request("Unknown tab."))?,
    };
    let kind = match query.kind.as_deref() {
        None => None,
        Some("photo") => Some(MediaKind::Image),
        Some("video") => Some(MediaKind::Video),
        Some(_) => return Err(AppError::bad_request("Unknown kind.")),
    };
    Ok(Json(
        graph::profile_posts(state.db(), viewer, &target, tab, kind, &paging, limit).await?,
    ))
}

async fn followers(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(handle): Path<String>,
    ApiQuery(query): ApiQuery<ListQuery>,
) -> Result<Json<Page<UserDto>>, AppError> {
    let target = find_user(&state, &auth.user, &handle).await?;
    edge_list(
        &state,
        &auth.user,
        Edges::Followers,
        target.id,
        query.into(),
    )
    .await
}

async fn following(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(handle): Path<String>,
    ApiQuery(query): ApiQuery<ListQuery>,
) -> Result<Json<Page<UserDto>>, AppError> {
    let target = find_user(&state, &auth.user, &handle).await?;
    edge_list(
        &state,
        &auth.user,
        Edges::Following,
        target.id,
        query.into(),
    )
    .await
}

async fn blocks(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiQuery(query): ApiQuery<ListQuery>,
) -> Result<Json<Page<UserDto>>, AppError> {
    edge_list(
        &state,
        &auth.user,
        Edges::Blocks,
        auth.user.id,
        query.into(),
    )
    .await
}

async fn mutes(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiQuery(query): ApiQuery<ListQuery>,
) -> Result<Json<Page<UserDto>>, AppError> {
    edge_list(&state, &auth.user, Edges::Mutes, auth.user.id, query.into()).await
}

async fn edge_list(
    state: &AppState,
    viewer: &UserRow,
    edges: Edges,
    subject: Uuid,
    query: PageQuery,
) -> Result<Json<Page<UserDto>>, AppError> {
    let limit = query.limit()?;
    let before = query.edge_cursor()?;
    Ok(Json(
        graph::edge_users(state.db(), viewer.id, edges, subject, before, limit).await?,
    ))
}

async fn followed_by_known(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(handle): Path<String>,
    ApiQuery(query): ApiQuery<SizeQuery>,
) -> Result<Json<KnownFollowersDto>, AppError> {
    let target = find_user(&state, &auth.user, &handle).await?;
    if target.id == auth.user.id {
        return Ok(Json(KnownFollowersDto {
            users: Vec::new(),
            count: 0,
        }));
    }
    let limit = small_limit(query.limit);
    Ok(Json(
        graph::known_followers(state.db(), auth.user.id, target.id, limit).await?,
    ))
}

async fn suggestions(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiQuery(query): ApiQuery<SizeQuery>,
) -> Result<Json<Vec<UserDto>>, AppError> {
    let limit = small_limit(query.limit);
    Ok(Json(
        graph::suggestions(state.db(), auth.user.id, limit).await?,
    ))
}

/// The card of `id` as `viewer` sees it after an action changed counters or relations.
async fn fresh(state: &AppState, viewer: Uuid, id: Uuid) -> Result<Json<UserDto>, AppError> {
    let row = users::by_id(state.db(), id)
        .await?
        .ok_or(AppError::Missing("User not found."))?;
    Ok(Json(users::dto(state.db(), viewer, &row).await?))
}

async fn follow(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(handle): Path<String>,
) -> Result<Json<UserDto>, AppError> {
    let me = auth.user.id;
    forbid_suspended(&state, &handle, "You can’t follow this account.").await?;
    let target = find_user(&state, &auth.user, &handle).await?;
    if target.id == me {
        return Err(AppError::bad_request("You can’t follow yourself."));
    }
    if graph::blocked_between(state.db(), me, target.id).await? {
        return Err(AppError::forbidden("You can’t follow this account."));
    }
    state.limits().check_follow(me)?;
    let pending = graph::follow(state.db(), state.config(), me, target.id).await?;
    deliver(&state, pending).await;
    fresh(&state, me, target.id).await
}

async fn unfollow(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(handle): Path<String>,
) -> Result<Json<UserDto>, AppError> {
    let target = find_user(&state, &auth.user, &handle).await?;
    graph::unfollow(state.db(), state.config(), auth.user.id, target.id).await?;
    fresh(&state, auth.user.id, target.id).await
}

async fn block(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(handle): Path<String>,
) -> Result<Json<UserDto>, AppError> {
    let target = find_user(&state, &auth.user, &handle).await?;
    if target.id == auth.user.id {
        return Err(AppError::bad_request("You can’t block yourself."));
    }
    graph::block(state.db(), state.config(), auth.user.id, target.id).await?;
    fresh(&state, auth.user.id, target.id).await
}

async fn unblock(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(handle): Path<String>,
) -> Result<Json<UserDto>, AppError> {
    let target = find_user(&state, &auth.user, &handle).await?;
    graph::unblock(state.db(), auth.user.id, target.id).await?;
    fresh(&state, auth.user.id, target.id).await
}

async fn mute(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(handle): Path<String>,
) -> Result<Json<UserDto>, AppError> {
    let target = find_user(&state, &auth.user, &handle).await?;
    if target.id == auth.user.id {
        return Err(AppError::bad_request("You can’t mute yourself."));
    }
    graph::mute(state.db(), auth.user.id, target.id).await?;
    fresh(&state, auth.user.id, target.id).await
}

async fn unmute(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(handle): Path<String>,
) -> Result<Json<UserDto>, AppError> {
    let target = find_user(&state, &auth.user, &handle).await?;
    graph::unmute(state.db(), auth.user.id, target.id).await?;
    fresh(&state, auth.user.id, target.id).await
}
