//! `GET /users/{handle}` and its collections, and `GET /actor`.

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::get;
use serde::Deserialize;
use serde_json::{Value, json};
use tuita_core::ap::{self, Collection};
use uuid::Uuid;

use super::{activity_json, activity_json_with, live_user};
use crate::auth::ApiQuery;
use crate::db::ap::{load_posts, local_user, outbox_ids, outbox_total, pinned_open};
use crate::error::AppError;
use crate::federation::{ids, render};
use crate::state::AppState;

/// Posts on one outbox page.
const PAGE_SIZE: usize = 20;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/actor", get(instance_actor))
        .route("/users/{handle}", get(actor))
        .route("/users/{handle}/outbox", get(outbox))
        .route("/users/{handle}/followers", get(followers))
        .route("/users/{handle}/following", get(following))
        .route("/users/{handle}/featured", get(featured))
}

async fn instance_actor(State(state): State<AppState>) -> Result<Response, AppError> {
    Ok(activity_json(&render::instance_actor(&state).await?))
}

async fn actor(
    State(state): State<AppState>,
    Path(handle): Path<String>,
) -> Result<Response, AppError> {
    let user = local_user(state.db(), &handle)
        .await?
        .ok_or(AppError::NotFound)?;
    if user.deleted_at.is_some() {
        let tombstone = json!({
            "@context": ap::context(),
            "id": ids::actor_id(state.config(), &user.handle),
            "type": "Tombstone",
        });
        return Ok(activity_json_with(StatusCode::GONE, &tombstone, false));
    }
    if user.suspended_at.is_some() {
        return Err(AppError::NotFound);
    }
    Ok(activity_json(&render::actor(&state, &user).await?))
}

#[derive(Deserialize)]
struct OutboxQuery {
    page: Option<String>,
    before: Option<String>,
}

async fn outbox(
    State(state): State<AppState>,
    Path(handle): Path<String>,
    ApiQuery(query): ApiQuery<OutboxQuery>,
) -> Result<Response, AppError> {
    let config = state.config();
    let user = live_user(&state, &handle).await?;
    let id = ids::outbox_url(config, &user.handle);
    if query.page.as_deref() != Some("true") {
        let total = outbox_total(state.db(), user.id).await?;
        let first = format!("{id}?page=true");
        let collection = Collection::paged(&id, u64::try_from(total).unwrap_or(0), &first);
        return Ok(activity_json(&ap::document(&collection)));
    }
    let before = query
        .before
        .as_deref()
        .map(|raw| Uuid::parse_str(raw).map_err(|_| AppError::bad_request("Invalid cursor.")))
        .transpose()?;
    let mut found = outbox_ids(state.db(), user.id, before, PAGE_SIZE as i64 + 1).await?;
    let more = found.len() > PAGE_SIZE;
    found.truncate(PAGE_SIZE);
    let posts = load_posts(state.db(), config.domain(), &found).await?;
    let items: Vec<Value> = posts
        .iter()
        .map(|post| render::create_object(config, post))
        .collect();
    let page_id = match before {
        Some(before) => format!("{id}?page=true&before={before}"),
        None => format!("{id}?page=true"),
    };
    let next = more
        .then(|| {
            found
                .last()
                .map(|last| format!("{id}?page=true&before={last}"))
        })
        .flatten();
    let page = Collection::page(&page_id, &id, Vec::new(), next.as_deref());
    let mut doc = ap::document(&page);
    if let Value::Object(map) = &mut doc {
        map.insert("orderedItems".to_owned(), Value::Array(items));
    }
    Ok(activity_json(&doc))
}

async fn followers(
    State(state): State<AppState>,
    Path(handle): Path<String>,
) -> Result<Response, AppError> {
    let user = live_user(&state, &handle).await?;
    let id = ids::followers_url(state.config(), &user.handle);
    let total = u64::try_from(user.followers_count).unwrap_or(0);
    Ok(activity_json(&ap::document(&Collection::counting(
        &id, total,
    ))))
}

async fn following(
    State(state): State<AppState>,
    Path(handle): Path<String>,
) -> Result<Response, AppError> {
    let user = live_user(&state, &handle).await?;
    let id = ids::following_url(state.config(), &user.handle);
    let total = u64::try_from(user.following_count).unwrap_or(0);
    Ok(activity_json(&ap::document(&Collection::counting(
        &id, total,
    ))))
}

async fn featured(
    State(state): State<AppState>,
    Path(handle): Path<String>,
) -> Result<Response, AppError> {
    let config = state.config();
    let user = live_user(&state, &handle).await?;
    let pinned = match pinned_open(state.db(), &user).await? {
        Some(id) => load_posts(state.db(), config.domain(), &[id]).await?,
        None => Vec::new(),
    };
    let items: Vec<Value> = pinned
        .iter()
        .map(|post| render::note_object(config, post))
        .collect();
    Ok(activity_json(&json!({
        "@context": ap::context(),
        "id": ids::featured_url(config, &user.handle),
        "type": "OrderedCollection",
        "totalItems": items.len(),
        "orderedItems": items,
    })))
}
