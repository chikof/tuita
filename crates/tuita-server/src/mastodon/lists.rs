//! Lists: `GET|POST /lists`, `GET|PUT|DELETE /lists/{id}` and `GET|POST|DELETE /lists/{id}/accounts`.
//!
//! `GET|POST /lists` and `GET|DELETE /lists/{id}` share their path and method with tuita's own
//! list routes, which register them with [`super::by_credential`]; the handlers here that those
//! routes name are `pub(crate)`. A Mastodon list is always private to its owner, so the lists made
//! here are `private` in tuita, and another account's list is `404` even when tuita would show it.
//! The rules of tuita's lists apply as they are: 50 lists, 500 members, no blocked accounts, no
//! words the filter refuses. `replies_policy` and `exclusive` are accepted and not kept: a list
//! always answers `replies_policy: "list"` and `exclusive: false`.

use std::collections::HashSet;

use axum::extract::{Path, RawQuery, State};
use axum::response::Response;
use axum::routing::{get, put};
use axum::{Json, Router};
use serde_json::{Value, json};
use uuid::Uuid;

use super::accounts::{paged, render};
use super::dto::{self, List};
use super::page::Paging;
use super::params::{Form, Params};
use crate::auth::{Auth, CurrentUser};
use crate::db::graph::blocked_between;
use crate::db::lists::{self, MAX_MEMBERS};
use crate::db::mastodon_accounts::{Bounds, EdgeSet, edge_accounts, visible_accounts};
use crate::db::settings;
use crate::dto::ListDto;
use crate::error::AppError;
use crate::state::AppState;

const NOT_FOUND: &str = "Record not found";
/// Longest list title, in characters (tuita's own limit).
const MAX_TITLE: usize = 40;
/// Most account ids one request adds or removes.
const MAX_IDS: usize = 50;

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/lists/{id}", put(update)).route(
        "/lists/{id}/accounts",
        get(accounts).post(add_accounts).delete(remove_accounts),
    )
}

/// The list `raw` names, when the caller owns it.
async fn owned(state: &AppState, auth: &Auth, raw: &str) -> Result<ListDto, AppError> {
    let id = Uuid::parse_str(raw).map_err(|_| AppError::Missing(NOT_FOUND))?;
    lists::find(state.db(), auth.user.id, id)
        .await?
        .filter(|list| list.is_owner)
        .ok_or(AppError::Missing(NOT_FOUND))
}

/// A trimmed title of 1 to 40 characters that the word filter allows.
async fn title(state: &AppState, params: &Params) -> Result<Option<String>, AppError> {
    let Some(raw) = params.get("title") else {
        return Ok(None);
    };
    let title = raw.trim();
    if title.is_empty() {
        return Err(AppError::unprocessable(
            "Validation failed: Title can\u{2019}t be blank",
        ));
    }
    if title.chars().count() > MAX_TITLE {
        return Err(AppError::unprocessable(format!(
            "Validation failed: Title is too long (maximum is {MAX_TITLE} characters)"
        )));
    }
    let filter = settings::content_filter(state.db()).await?;
    if filter.blocks_text(title) {
        return Err(AppError::unprocessable(
            "Validation failed: That list name isn\u{2019}t allowed.",
        ));
    }
    Ok(Some(title.to_owned()))
}

fn check_policy(params: &Params) -> Result<(), AppError> {
    match params.get("replies_policy") {
        None | Some("list" | "followed" | "none") => Ok(()),
        Some(_) => Err(AppError::unprocessable(
            "Validation failed: Replies policy is not included in the list",
        )),
    }
}

/// `GET /lists`: the caller's lists.
pub(crate) async fn mine(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
) -> Result<Json<Vec<List>>, AppError> {
    let owned = lists::owned(state.db(), auth.user.id, None).await?;
    Ok(Json(owned.iter().map(dto::list).collect()))
}

/// `POST /lists`.
pub(crate) async fn create(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    form: Form,
) -> Result<Json<List>, AppError> {
    check_policy(&form.params)?;
    let title = title(&state, &form.params)
        .await?
        .ok_or_else(|| AppError::unprocessable("Validation failed: Title can\u{2019}t be blank"))?;
    state.limits().check_lists(auth.user.id)?;
    let list = lists::create(state.db(), auth.user.id, &title, "", true)
        .await
        .map_err(unprocessable)?;
    Ok(Json(dto::list(&list)))
}

/// `GET /lists/{id}`.
pub(crate) async fn show(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<List>, AppError> {
    Ok(Json(dto::list(&owned(&state, &auth, &id).await?)))
}

async fn update(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    form: Form,
) -> Result<Json<List>, AppError> {
    let list = owned(&state, &auth, &id).await?;
    check_policy(&form.params)?;
    let title = title(&state, &form.params).await?;
    lists::update(state.db(), list.id, title.as_deref(), None, None).await?;
    Ok(Json(dto::list(&owned(&state, &auth, &id).await?)))
}

/// `DELETE /lists/{id}`.
pub(crate) async fn remove(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let list = owned(&state, &auth, &id).await?;
    lists::delete(state.db(), list.id).await?;
    Ok(Json(json!({})))
}

/// Validation of a list answers `400` in tuita's API; Mastodon clients expect `422`.
fn unprocessable(err: AppError) -> AppError {
    match err {
        AppError::BadRequest(message) => AppError::Unprocessable(message),
        other => other,
    }
}

async fn accounts(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    RawQuery(query): RawQuery,
    paging: Paging<Uuid>,
) -> Result<Response, AppError> {
    let list = owned(&state, &auth, &id).await?;
    // `limit=0` is Mastodon's "all of them".
    if Params::from_query(query.as_deref()).int("limit") == Some(0) {
        let bounds = Bounds {
            upper: None,
            lower: None,
            ascending: false,
            fetch: MAX_MEMBERS,
        };
        let rows = edge_accounts(state.db(), EdgeSet::ListMembers, list.id, bounds).await?;
        return Ok(paged(&render(&state, &rows).await?, None));
    }
    let bounds = Bounds {
        upper: paging.upper().copied(),
        lower: paging.lower().copied(),
        ascending: paging.ascending(),
        fetch: paging.fetch(),
    };
    let rows = edge_accounts(state.db(), EdgeSet::ListMembers, list.id, bounds).await?;
    let slice = paging.trim(rows);
    let link = paging.link(
        state.config(),
        slice.rows.first().map(|row| row.id),
        slice.rows.last().map(|row| row.id),
        slice.more,
    );
    Ok(paged(&render(&state, &slice.rows).await?, link))
}

/// The distinct ids in `account_ids[]`, at most [`MAX_IDS`]; an id that is no UUID names nobody.
fn account_ids(params: &Params) -> Result<Vec<Uuid>, AppError> {
    let mut seen = HashSet::new();
    let mut ids = Vec::new();
    for raw in params.list("account_ids") {
        let id = Uuid::parse_str(raw).map_err(|_| AppError::Missing(NOT_FOUND))?;
        if seen.insert(id) {
            ids.push(id);
        }
    }
    if ids.is_empty() {
        return Err(AppError::bad_request(
            "param is missing or the value is empty: account_ids",
        ));
    }
    if ids.len() > MAX_IDS {
        return Err(AppError::unprocessable(format!(
            "Validation failed: at most {MAX_IDS} accounts at a time"
        )));
    }
    Ok(ids)
}

async fn add_accounts(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    form: Form,
) -> Result<Json<Value>, AppError> {
    let list = owned(&state, &auth, &id).await?;
    let ids = account_ids(&form.params)?;
    let rows = visible_accounts(state.db(), &ids, auth.user.is_staff()).await?;
    if rows.len() != ids.len() {
        return Err(AppError::Missing(NOT_FOUND));
    }
    for row in &rows {
        if blocked_between(state.db(), auth.user.id, row.id).await? {
            return Err(AppError::forbidden("You can\u{2019}t add this account."));
        }
    }
    for id in ids {
        lists::add_member(state.db(), list.id, id)
            .await
            .map_err(unprocessable)?;
    }
    Ok(Json(json!({})))
}

async fn remove_accounts(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    form: Form,
) -> Result<Json<Value>, AppError> {
    let list = owned(&state, &auth, &id).await?;
    for id in account_ids(&form.params)? {
        lists::remove_member(state.db(), list.id, id).await?;
    }
    Ok(Json(json!({})))
}
