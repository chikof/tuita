//! Posts: create, thread, edit, open a thread, delete, react, vote, hide, pin, and the lists of
//! who reacted or quoted.

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::auth::{ApiJson, ApiQuery, CurrentUser};
use crate::cursor::{PageQuery, parse_id};
use crate::db::post_actions::{self, Reaction, ReactorList};
use crate::db::posts::load_one;
use crate::db::sessions::ClientMeta;
use crate::db::{compose, polls};
use crate::dto::{
    CreatePostRequest, CreateThreadRequest, EditPostRequest, OkDto, Page, PostDto, PostEditsDto,
    ThreadDto, ThreadPostsDto, UserDto, VoteRequest,
};
use crate::error::AppError;
use crate::state::AppState;

const NOT_FOUND: &str = "Post not found.";

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/posts", post(create))
        .route("/posts/thread", post(create_thread))
        .route("/posts/{id}", get(thread).patch(edit).delete(remove))
        .route("/posts/{id}/edits", get(edits))
        .route("/posts/{id}/vote", post(vote))
        .route("/posts/{id}/hide", post(hide).delete(unhide))
        .route("/posts/{id}/like", post(like).delete(unlike))
        .route("/posts/{id}/repost", post(repost).delete(unrepost))
        .route("/posts/{id}/bookmark", post(bookmark).delete(unbookmark))
        .route("/posts/{id}/pin", post(pin).delete(unpin))
        .route("/posts/{id}/likes", get(likes))
        .route("/posts/{id}/reposts", get(reposts))
        .route("/posts/{id}/quotes", get(quotes))
}

async fn create(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    meta: ClientMeta,
    ApiJson(req): ApiJson<CreatePostRequest>,
) -> Result<Json<PostDto>, AppError> {
    state.limits().check_post(auth.user.id)?;
    let id = compose::create(&state, &auth.user, meta.client, req).await?;
    let post = load_one(state.db(), auth.user.id, id, None)
        .await?
        .ok_or(AppError::Missing(NOT_FOUND))?;
    Ok(Json(post))
}

async fn create_thread(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    meta: ClientMeta,
    ApiJson(req): ApiJson<CreateThreadRequest>,
) -> Result<Json<ThreadPostsDto>, AppError> {
    for _ in &req.posts {
        state.limits().check_post(auth.user.id)?;
    }
    let ids = compose::create_thread(&state, &auth.user, meta.client, req.posts).await?;
    let posts = crate::db::posts::load(state.db(), auth.user.id, &ids, None).await?;
    Ok(Json(ThreadPostsDto { posts }))
}

async fn edit(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    ApiJson(req): ApiJson<EditPostRequest>,
) -> Result<Json<PostDto>, AppError> {
    let id = parse_id(&id, NOT_FOUND)?;
    state.limits().check_edit(auth.user.id)?;
    compose::edit(&state, &auth.user, id, req).await?;
    let post = load_one(state.db(), auth.user.id, id, Some(auth.user.id))
        .await?
        .ok_or(AppError::Missing(NOT_FOUND))?;
    Ok(Json(post))
}

async fn edits(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<PostEditsDto>, AppError> {
    let id = parse_id(&id, NOT_FOUND)?;
    load_one(state.db(), auth.user.id, id, None)
        .await?
        .ok_or(AppError::Missing(NOT_FOUND))?;
    Ok(Json(compose::edits(state.db(), id).await?))
}

async fn vote(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    ApiJson(req): ApiJson<VoteRequest>,
) -> Result<Json<PostDto>, AppError> {
    let id = parse_id(&id, NOT_FOUND)?;
    let author = polls::vote(state.db(), state.config(), auth.user.id, id, &req.choices).await?;
    let post = load_one(state.db(), auth.user.id, id, Some(author))
        .await?
        .ok_or(AppError::Missing(NOT_FOUND))?;
    Ok(Json(post))
}

async fn set_hidden(
    state: AppState,
    auth: CurrentUser,
    id: &str,
    hidden: bool,
) -> Result<Json<PostDto>, AppError> {
    let id = parse_id(id, NOT_FOUND)?;
    let viewer = auth.0.user.id;
    compose::set_hidden(state.db(), viewer, id, hidden).await?;
    let post = load_one(state.db(), viewer, id, Some(viewer))
        .await?
        .ok_or(AppError::Missing(NOT_FOUND))?;
    Ok(Json(post))
}

async fn hide(
    State(state): State<AppState>,
    auth: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<PostDto>, AppError> {
    set_hidden(state, auth, &id, true).await
}

async fn unhide(
    State(state): State<AppState>,
    auth: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<PostDto>, AppError> {
    set_hidden(state, auth, &id, false).await
}

async fn thread(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<ThreadDto>, AppError> {
    let id = parse_id(&id, NOT_FOUND)?;
    Ok(Json(
        post_actions::thread(state.db(), auth.user.id, id).await?,
    ))
}

async fn remove(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<OkDto>, AppError> {
    let id = parse_id(&id, NOT_FOUND)?;
    post_actions::delete(&state, &auth.user, id).await?;
    Ok(Json(OkDto { ok: true }))
}

async fn react(
    state: AppState,
    auth: CurrentUser,
    id: &str,
    reaction: Reaction,
    add: bool,
) -> Result<Json<PostDto>, AppError> {
    let id = parse_id(id, NOT_FOUND)?;
    let post = Box::pin(post_actions::react(
        &state,
        auth.0.user.id,
        id,
        reaction,
        add,
    ))
    .await?;
    Ok(Json(post))
}

async fn like(
    State(state): State<AppState>,
    auth: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<PostDto>, AppError> {
    react(state, auth, &id, Reaction::Like, true).await
}

async fn unlike(
    State(state): State<AppState>,
    auth: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<PostDto>, AppError> {
    react(state, auth, &id, Reaction::Like, false).await
}

async fn repost(
    State(state): State<AppState>,
    auth: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<PostDto>, AppError> {
    react(state, auth, &id, Reaction::Repost, true).await
}

async fn unrepost(
    State(state): State<AppState>,
    auth: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<PostDto>, AppError> {
    react(state, auth, &id, Reaction::Repost, false).await
}

async fn bookmark(
    State(state): State<AppState>,
    auth: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<PostDto>, AppError> {
    react(state, auth, &id, Reaction::Bookmark, true).await
}

async fn unbookmark(
    State(state): State<AppState>,
    auth: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<PostDto>, AppError> {
    react(state, auth, &id, Reaction::Bookmark, false).await
}

async fn pin(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<OkDto>, AppError> {
    let id = parse_id(&id, NOT_FOUND)?;
    post_actions::pin(state.db(), auth.user.id, id).await?;
    Ok(Json(OkDto { ok: true }))
}

async fn unpin(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
) -> Result<Json<OkDto>, AppError> {
    post_actions::unpin(state.db(), auth.user.id).await?;
    Ok(Json(OkDto { ok: true }))
}

async fn reactors(
    state: AppState,
    auth: CurrentUser,
    id: &str,
    query: PageQuery,
    list: ReactorList,
) -> Result<Json<Page<UserDto>>, AppError> {
    let id = parse_id(id, NOT_FOUND)?;
    let page = post_actions::reactors(
        state.db(),
        auth.0.user.id,
        id,
        list,
        query.edge_cursor()?,
        query.limit()?,
    )
    .await?;
    Ok(Json(page))
}

async fn likes(
    State(state): State<AppState>,
    auth: CurrentUser,
    Path(id): Path<String>,
    ApiQuery(query): ApiQuery<PageQuery>,
) -> Result<Json<Page<UserDto>>, AppError> {
    reactors(state, auth, &id, query, ReactorList::Likes).await
}

async fn reposts(
    State(state): State<AppState>,
    auth: CurrentUser,
    Path(id): Path<String>,
    ApiQuery(query): ApiQuery<PageQuery>,
) -> Result<Json<Page<UserDto>>, AppError> {
    reactors(state, auth, &id, query, ReactorList::Reposts).await
}

async fn quotes(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    ApiQuery(query): ApiQuery<PageQuery>,
) -> Result<Json<Page<PostDto>>, AppError> {
    let id = parse_id(&id, NOT_FOUND)?;
    let page = post_actions::quotes(
        state.db(),
        auth.user.id,
        id,
        query.id_cursor()?,
        query.limit()?,
    )
    .await?;
    Ok(Json(page))
}
