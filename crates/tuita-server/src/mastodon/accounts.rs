//! Accounts: `GET /accounts/verify_credentials`, `PATCH /accounts/update_credentials`,
//! `GET /accounts/{id}`, `/accounts/lookup`, `/accounts/relationships`,
//! `/accounts/{id}/statuses|followers|following|lists`, and
//! `POST /accounts/{id}/follow|unfollow|block|unblock|mute|unmute`.
//!
//! Every handler is the Mastodon face of a rule tuita already has: the follow, block and mute
//! actions call the same [`crate::db::graph`] functions as `/users/{handle}/follow` and friends
//! (self-follows, blocks, the hourly follow limit, pending remote follows and their delivery),
//! the profile edit goes through the validation of `PATCH /me`, and a remote `user@domain` is
//! looked up through [`crate::federation::lookup`]. The routes need a signed-in account; a
//! request without one is `401`, as everywhere in this API.
//!
//! Ids are account UUIDs. An id that names nobody, a deleted or suspended account, an account on
//! a `suspend`-blocked instance and the `tuita` system account are all `404 Record not found`.
//!
//! Differences from Mastodon, all because tuita has no such thing: local accounts cannot be
//! `locked` (`locked` in `update_credentials` is ignored), `fields_attributes` can only set the
//! website and location, `source[...]` defaults are ignored, a follow ignores `reblogs`,
//! `notify` and `languages`, and a mute ignores `duration` and `notifications`.

use std::collections::{HashMap, HashSet};

use axum::extract::{DefaultBodyLimit, FromRequestParts, Path, RawQuery, State};
use axum::http::HeaderValue;
use axum::http::header::{CONTENT_TYPE, LINK};
use axum::http::request::Parts;
use axum::response::{IntoResponse as _, Response};
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use serde::Serialize;
use uuid::Uuid;

use super::ctx::Ctx;
use super::dto::{self, Account, Relationship};
use super::page::Paging;
use super::params::{Form, Params};
use crate::auth::{Auth, CurrentUser};
use crate::db::graph;
use crate::db::mastodon_accounts::{
    Bounds, EdgeSet, StatusFilter, edge_accounts, profile_entries, visible_account,
    visible_accounts,
};
use crate::db::notifications::deliver;
use crate::db::users::{self, UserRow, me_dto, user_ref};
use crate::db::{accounts, lists, posts, settings};
use crate::dto::{PatchMeRequest, PostDto};
use crate::error::AppError;
use crate::federation::lookup::{Handle, parse_handle, resolve_acct};
use crate::federation::outbound;
use crate::media::{IMAGE_TOO_LARGE, MAX_IMAGE_BYTES, Target, insert, prepare_image};
use crate::routes::me::validate;
use crate::state::AppState;

/// What every handler answers for an id that names no account the caller may look at.
const NOT_FOUND: &str = "Record not found";
/// Most ids `GET /accounts/relationships` answers for.
pub(super) const MAX_RELATIONSHIPS: usize = 40;
/// The body of `PATCH /accounts/update_credentials` may carry an avatar and a header.
const PROFILE_BODY: usize = 2 * MAX_IMAGE_BYTES + 256 * 1024;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/accounts/verify_credentials", get(verify_credentials))
        .route(
            "/accounts/update_credentials",
            patch(update_credentials).layer(DefaultBodyLimit::max(PROFILE_BODY)),
        )
        .route("/accounts/relationships", get(relationships))
        .route("/accounts/lookup", get(lookup))
        .route("/accounts/{id}", get(show))
        .route("/accounts/{id}/statuses", get(statuses))
        .route("/accounts/{id}/followers", get(followers))
        .route("/accounts/{id}/following", get(following))
        .route("/accounts/{id}/lists", get(lists_with))
        .route("/accounts/{id}/follow", post(follow))
        .route("/accounts/{id}/unfollow", post(unfollow))
        .route("/accounts/{id}/block", post(block))
        .route("/accounts/{id}/unblock", post(unblock))
        .route("/accounts/{id}/mute", post(mute))
        .route("/accounts/{id}/unmute", post(unmute))
}

// ------------------------------------------------------------------ shared helpers

/// A JSON array with its `Link` header, when there is one.
pub(super) fn paged<T: Serialize>(body: &[T], link: Option<HeaderValue>) -> Response {
    let mut response = Json(body).into_response();
    if let Some(link) = link {
        response.headers_mut().insert(LINK, link);
    }
    response
}

/// The account `raw` names, as the caller may look at it.
pub(super) async fn account_row(
    state: &AppState,
    viewer: &UserRow,
    raw: &str,
) -> Result<UserRow, AppError> {
    let id = Uuid::parse_str(raw).map_err(|_| AppError::Missing(NOT_FOUND))?;
    visible_account(state.db(), id, viewer.is_staff())
        .await?
        .ok_or(AppError::Missing(NOT_FOUND))
}

/// The Mastodon accounts of `rows`, in the same order, with their context loaded in one batch.
pub(super) async fn render(state: &AppState, rows: &[UserRow]) -> Result<Vec<Account>, AppError> {
    let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
    let ctx = Ctx::for_users(state, &ids).await?;
    Ok(rows
        .iter()
        .filter_map(|row| ctx.user(row.id).map(|user| dto::account(user, &ctx)))
        .collect())
}

/// How `viewer` relates to `target` right now.
async fn relationship_of(
    state: &AppState,
    viewer: Uuid,
    target: Uuid,
) -> Result<Json<Relationship>, AppError> {
    let relation = users::relations(state.db(), viewer, &[target])
        .await?
        .remove(&target)
        .unwrap_or_default();
    Ok(Json(dto::relationship(target, relation)))
}

pub(super) fn bounds(paging: &Paging<Uuid>) -> Bounds {
    Bounds {
        upper: paging.upper().copied(),
        lower: paging.lower().copied(),
        ascending: paging.ascending(),
        fetch: paging.fetch(),
    }
}

// ------------------------------------------------------------------ reading

async fn verify_credentials(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
) -> Result<Json<dto::CredentialAccount>, AppError> {
    let me = me_dto(state.db(), &auth.user, state.config().domain()).await?;
    let ctx = Ctx::for_users(&state, &[auth.user.id]).await?;
    Ok(Json(dto::credential_account(&me, 0, &ctx)))
}

async fn show(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Account>, AppError> {
    let row = account_row(&state, &auth.user, &id).await?;
    let mut accounts = render(&state, &[row]).await?;
    accounts.pop().map(Json).ok_or(AppError::Missing(NOT_FOUND))
}

async fn lookup(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    RawQuery(query): RawQuery,
) -> Result<Json<Account>, AppError> {
    let params = Params::from_query(query.as_deref());
    let acct = params.get("acct").map(str::trim).unwrap_or_default();
    if acct.is_empty() {
        return Err(AppError::bad_request(
            "param is missing or the value is empty: acct",
        ));
    }
    let row = match parse_handle(acct, state.config().domain()) {
        None => return Err(AppError::Missing(NOT_FOUND)),
        Some(Handle::Local(handle)) => {
            crate::db::users::find_local(state.db(), &handle, auth.user.is_staff())
                .await
                .map_err(|_| AppError::Missing(NOT_FOUND))?
        }
        Some(Handle::Remote { handle, domain }) => {
            Box::pin(resolve_acct(&state, Some(auth.user.id), &handle, &domain))
                .await
                .map_err(|err| match AppError::from(err) {
                    AppError::Missing(_) => AppError::Missing(NOT_FOUND),
                    other => other,
                })?
        }
    };
    let mut accounts = render(&state, &[row]).await?;
    accounts.pop().map(Json).ok_or(AppError::Missing(NOT_FOUND))
}

async fn relationships(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    RawQuery(query): RawQuery,
) -> Result<Json<Vec<Relationship>>, AppError> {
    let params = Params::from_query(query.as_deref());
    let mut seen = HashSet::new();
    let ids: Vec<Uuid> = params
        .list("id")
        .iter()
        .filter_map(|raw| Uuid::parse_str(raw).ok())
        .filter(|id| seen.insert(*id))
        .take(MAX_RELATIONSHIPS)
        .collect();
    let existing: HashSet<Uuid> = visible_accounts(state.db(), &ids, auth.user.is_staff())
        .await?
        .into_iter()
        .map(|row| row.id)
        .collect();
    let ids: Vec<Uuid> = ids.into_iter().filter(|id| existing.contains(id)).collect();
    let mut relations = users::relations(state.db(), auth.user.id, &ids).await?;
    Ok(Json(
        ids.into_iter()
            .map(|id| dto::relationship(id, relations.remove(&id).unwrap_or_default()))
            .collect(),
    ))
}

async fn statuses(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    paging: Paging<Uuid>,
    form: Form,
) -> Result<Response, AppError> {
    let db = state.db();
    let viewer = auth.user.id;
    let target = account_row(&state, &auth.user, &id).await?;
    let flag = |key: &str| form.params.flag(key).unwrap_or(false);
    if graph::blocked_between(db, viewer, target.id).await? {
        return Ok(paged::<dto::Status>(&[], None));
    }
    if flag("pinned") {
        let pinned = match target.pinned_post_id {
            Some(post) => posts::load_one(db, viewer, post, Some(target.id)).await?,
            None => None,
        };
        let posts: Vec<PostDto> = pinned
            .into_iter()
            .map(|mut post| {
                post.pinned = true;
                post
            })
            .collect();
        return Ok(paged(&render_statuses(&state, &posts).await?, None));
    }
    let entries = profile_entries(
        db,
        target.id,
        StatusFilter {
            only_media: flag("only_media"),
            exclude_replies: flag("exclude_replies"),
            exclude_reblogs: flag("exclude_reblogs"),
        },
        bounds(&paging),
    )
    .await?;
    let slice = paging.trim(entries);
    let link = paging.link(
        state.config(),
        slice.rows.first().map(|entry| entry.key),
        slice.rows.last().map(|entry| entry.key),
        slice.more,
    );
    let post_ids: Vec<Uuid> = slice.rows.iter().map(|entry| entry.post_id).collect();
    let loaded = posts::load(db, viewer, &post_ids, Some(target.id)).await?;
    let by_id: HashMap<Uuid, PostDto> = loaded.into_iter().map(|post| (post.id, post)).collect();
    let booster = user_ref(&target);
    let mut items = Vec::with_capacity(slice.rows.len());
    for entry in &slice.rows {
        let Some(mut post) = by_id.get(&entry.post_id).cloned() else {
            continue;
        };
        if entry.reposted {
            post.reposted_by = Some(booster.clone());
            post.entry_id = entry.key;
        } else {
            post.pinned = target.pinned_post_id == Some(post.id);
        }
        items.push(post);
    }
    Ok(paged(&render_statuses(&state, &items).await?, link))
}

async fn render_statuses(
    state: &AppState,
    posts: &[PostDto],
) -> Result<Vec<dto::Status>, AppError> {
    let ctx = Ctx::for_posts(state, posts).await?;
    Ok(dto::statuses(posts, &ctx))
}

async fn followers(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    paging: Paging<Uuid>,
) -> Result<Response, AppError> {
    edge_page(&state, &auth, &id, EdgeSet::Followers, &paging).await
}

async fn following(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    paging: Paging<Uuid>,
) -> Result<Response, AppError> {
    edge_page(&state, &auth, &id, EdgeSet::Following, &paging).await
}

async fn edge_page(
    state: &AppState,
    auth: &Auth,
    id: &str,
    set: EdgeSet,
    paging: &Paging<Uuid>,
) -> Result<Response, AppError> {
    let target = account_row(state, &auth.user, id).await?;
    let rows = edge_accounts(state.db(), set, target.id, bounds(paging)).await?;
    let slice = paging.trim(rows);
    let link = paging.link(
        state.config(),
        slice.rows.first().map(|row| row.id),
        slice.rows.last().map(|row| row.id),
        slice.more,
    );
    Ok(paged(&render(state, &slice.rows).await?, link))
}

/// The caller's lists that hold the account.
async fn lists_with(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<dto::List>>, AppError> {
    let target = account_row(&state, &auth.user, &id).await?;
    let owned = lists::owned(state.db(), auth.user.id, Some(target.id)).await?;
    Ok(Json(
        owned
            .iter()
            .filter(|list| list.has_member == Some(true))
            .map(dto::list)
            .collect(),
    ))
}

// ------------------------------------------------------------------ actions

async fn follow(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Relationship>, AppError> {
    let me = auth.user.id;
    let target = account_row(&state, &auth.user, &id).await?;
    if target.id == me {
        return Err(AppError::unprocessable(
            "Validation failed: You can\u{2019}t follow yourself.",
        ));
    }
    if graph::blocked_between(state.db(), me, target.id).await? {
        return Err(AppError::forbidden("You can\u{2019}t follow this account."));
    }
    state.limits().check_follow(me)?;
    let pending = graph::follow(state.db(), state.config(), me, target.id).await?;
    deliver(&state, pending).await;
    relationship_of(&state, me, target.id).await
}

async fn unfollow(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Relationship>, AppError> {
    let target = account_row(&state, &auth.user, &id).await?;
    graph::unfollow(state.db(), state.config(), auth.user.id, target.id).await?;
    relationship_of(&state, auth.user.id, target.id).await
}

async fn block(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Relationship>, AppError> {
    let target = account_row(&state, &auth.user, &id).await?;
    if target.id == auth.user.id {
        return Err(AppError::unprocessable(
            "Validation failed: You can\u{2019}t block yourself.",
        ));
    }
    graph::block(state.db(), state.config(), auth.user.id, target.id).await?;
    relationship_of(&state, auth.user.id, target.id).await
}

async fn unblock(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Relationship>, AppError> {
    let target = account_row(&state, &auth.user, &id).await?;
    graph::unblock(state.db(), auth.user.id, target.id).await?;
    relationship_of(&state, auth.user.id, target.id).await
}

async fn mute(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Relationship>, AppError> {
    let target = account_row(&state, &auth.user, &id).await?;
    if target.id == auth.user.id {
        return Err(AppError::unprocessable(
            "Validation failed: You can\u{2019}t mute yourself.",
        ));
    }
    graph::mute(state.db(), auth.user.id, target.id).await?;
    relationship_of(&state, auth.user.id, target.id).await
}

async fn unmute(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Relationship>, AppError> {
    let target = account_row(&state, &auth.user, &id).await?;
    graph::unmute(state.db(), auth.user.id, target.id).await?;
    relationship_of(&state, auth.user.id, target.id).await
}

// ------------------------------------------------------------------ editing the profile

/// Website and location from `fields_attributes[n][name|value]`. Mastodon replaces the
/// whole set, so a field the client leaves out is cleared. Nothing changes when the client sent no fields.
fn field_patch(params: &Params, patch: &mut PatchMeRequest) {
    if !(0..4).any(|n| params.has(&format!("fields_attributes[{n}][name]"))) {
        return;
    }
    let (mut website, mut location) = (String::new(), String::new());
    for n in 0..4 {
        let name = params
            .get(&format!("fields_attributes[{n}][name]"))
            .unwrap_or_default()
            .trim()
            .to_lowercase();
        let value = params
            .get(&format!("fields_attributes[{n}][value]"))
            .unwrap_or_default()
            .to_owned();
        match name.as_str() {
            "website" => website = value,
            "location" => location = value,
            _ => {}
        }
    }
    patch.website = Some(website);
    patch.location = Some(location);
}

/// Validation answers `400` in tuita's API; Mastodon clients expect `422`.
fn unprocessable(err: AppError) -> AppError {
    match err {
        AppError::BadRequest(message) => AppError::Unprocessable(message),
        other => other,
    }
}

/// Counts one upload against the caller's rate limit before the body is read, for a request
/// that carries a multipart body. `update_credentials` buffers up to [`PROFILE_BODY`], so the
/// limit must refuse first; extractors run in argument order, so this one goes before [`Form`].
struct UploadBudget;

impl FromRequestParts<AppState> for UploadBudget {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let multipart = parts
            .headers
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.trim().to_ascii_lowercase().starts_with("multipart/"));
        if multipart {
            let CurrentUser(auth) = CurrentUser::from_request_parts(parts, state).await?;
            state.limits().check_upload(auth.user.id)?;
        }
        Ok(Self)
    }
}

async fn update_credentials(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    _budget: UploadBudget,
    form: Form,
) -> Result<Json<dto::CredentialAccount>, AppError> {
    let db = state.db();
    let user = auth.user.id;
    let params = &form.params;
    let mut patch = PatchMeRequest {
        display_name: params.get("display_name").map(str::to_owned),
        bio: params.get("note").map(str::to_owned),
        discoverable: params.flag("discoverable"),
        ..PatchMeRequest::default()
    };
    field_patch(params, &mut patch);
    let filter = settings::content_filter(db).await?;
    let changes = validate(&auth.user, patch, &filter).map_err(unprocessable)?;

    let mut images: Vec<(bool, Vec<u8>)> = Vec::new();
    for file in &form.files {
        let header = match file.field.as_str() {
            "avatar" => false,
            "header" => true,
            _ => continue,
        };
        if file.bytes.len() > MAX_IMAGE_BYTES {
            return Err(AppError::PayloadTooLarge(IMAGE_TOO_LARGE.into()));
        }
        images.push((header, file.bytes.to_vec()));
    }
    let bot = params
        .flag("bot")
        .filter(|bot| *bot != auth.user.is_bot.unwrap_or(false));
    if bot.is_some() {
        state.limits().check_bot(user)?;
    }
    for (header, bytes) in images {
        store_profile_image(&state, user, bytes, header)
            .await
            .map_err(unprocessable)?;
    }
    let mut row = Box::pin(accounts::update_profile(db, state.config(), user, changes)).await?;
    if let Some(bot) = bot {
        row = accounts::set_bot(db, state.config(), user, bot, None).await?;
    }
    let me = me_dto(state.db(), &row, state.config().domain()).await?;
    let ctx = Ctx::for_users(&state, &[user]).await?;
    Ok(Json(dto::credential_account(&me, 0, &ctx)))
}

/// Replaces the avatar or the header with `bytes`, centred and scaled as the web client's crop
/// would, and tells the account's remote followers.
async fn store_profile_image(
    state: &AppState,
    user: Uuid,
    bytes: Vec<u8>,
    header: bool,
) -> Result<(), AppError> {
    let target = if header {
        Target::BannerFill
    } else {
        Target::AvatarFill
    };
    let mut media = prepare_image(state, bytes, target).await?;
    let sql = if header {
        "update users set banner_id = $1 where id = $2"
    } else {
        "update users set avatar_id = $1 where id = $2"
    };
    let mut tx = state.db().begin().await?;
    insert(&mut tx, user, "", None, &media).await?;
    sqlx::query(sql)
        .bind(media.id)
        .bind(user)
        .execute(&mut *tx)
        .await?;
    outbound::profile_updated(&mut tx, state.config(), user).await?;
    tx.commit().await?;
    media.guard.keep();
    Ok(())
}
