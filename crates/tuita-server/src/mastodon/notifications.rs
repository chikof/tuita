//! Notifications: `GET /notifications`, `GET /notifications/{id}`, `POST /notifications/clear`
//! and `POST /notifications/{id}/dismiss`.
//!
//! The list shares its path with tuita's grouped list and is registered there with
//! [`super::by_credential`]. Mastodon lists each notification, so this reads stored rows
//! ([`crate::db::mastodon_activity`]), not tuita's groups. A tuita reply, mention and quote are
//! `mention`; like is `favourite`, repost `reblog`; affiliations have no Mastodon type and never
//! appear. `types[]` and `exclude_types[]` take the Mastodon names, other names match nothing.
//! `dismiss` and `clear` delete the rows, as Mastodon does.

use axum::extract::{Path, RawQuery, State};
use axum::http::HeaderValue;
use axum::response::Response;
use axum::routing::post;
use axum::{Json, Router};
use uuid::Uuid;

use super::ctx::Ctx;
use super::dto::{self, Notification};
use super::page::Paging;
use super::params::Params;
use super::timelines::{id_range, respond};
use crate::auth::CurrentUser;
use crate::db::mastodon_activity::{self as rows, NotificationFilter};
use crate::db::notifications::publish_badges;
use crate::dto::{ReadEventDto, ReadKind, StreamEvent};
use crate::error::AppError;
use crate::state::AppState;

const MISSING: &str = "Notification not found.";

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/notifications/clear", post(clear))
        .route("/notifications/{id}", axum::routing::get(show))
        .route("/notifications/{id}/dismiss", post(dismiss))
}

/// The tuita kinds behind a Mastodon notification type.
fn stored_kinds(kind: &str) -> &'static [&'static str] {
    match kind {
        "favourite" => &["like"],
        "reblog" => &["repost"],
        "mention" => &["mention", "reply", "quote"],
        "follow" => &["follow"],
        "follow_request" => &["follow_request"],
        "poll" => &["poll"],
        _ => &[],
    }
}

fn kinds_of(types: &[String]) -> Vec<String> {
    types
        .iter()
        .flat_map(|kind| stored_kinds(kind))
        .map(|kind| (*kind).to_owned())
        .collect()
}

async fn render(
    state: &AppState,
    viewer: Uuid,
    rows_in: &[rows::NotificationRow],
) -> Result<Vec<Notification>, AppError> {
    let resolved = rows::resolve(state.db(), viewer, rows_in).await?;
    let posts: Vec<&crate::dto::PostDto> = resolved
        .iter()
        .filter_map(|row| row.post.as_ref())
        .collect();
    let actors: Vec<Uuid> = resolved.iter().map(|row| row.actor.id).collect();
    let ctx = Ctx::load(state, &posts, &actors).await?;
    Ok(resolved
        .iter()
        .filter_map(|row| {
            dto::notification(
                row.id,
                row.kind,
                row.created_at,
                &row.actor,
                row.post.as_ref(),
                &ctx,
            )
        })
        .collect())
}

/// One page of the viewer's notifications for the request's `types[]`, `exclude_types[]` and
/// `account_id`, with the `Link` header: what `GET /api/v1/notifications` and
/// `GET /api/v2/notifications` both answer.
pub(super) async fn page(
    state: &AppState,
    viewer: Uuid,
    paging: &Paging<Uuid>,
    query: Option<&str>,
) -> Result<(Option<HeaderValue>, Vec<Notification>), AppError> {
    let params = Params::from_query(query);
    let filter = NotificationFilter {
        include: params.has("types").then(|| kinds_of(params.list("types"))),
        exclude: kinds_of(params.list("exclude_types")),
        account: match params.get("account_id").map(str::trim) {
            None | Some("") => None,
            Some(raw) => {
                Some(Uuid::parse_str(raw).map_err(|_| AppError::Missing("Record not found."))?)
            }
        },
    };
    let fetched = rows::notification_rows(
        state.db(),
        viewer,
        &filter,
        id_range(paging),
        paging.fetch(),
    )
    .await?;
    let slice = paging.trim(fetched);
    let link = paging.link(
        state.config(),
        slice.rows.first().map(|row| row.id),
        slice.rows.last().map(|row| row.id),
        slice.more,
    );
    Ok((link, render(state, viewer, &slice.rows).await?))
}

/// `GET /notifications` for an OAuth token.
pub(crate) async fn list(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    paging: Paging<Uuid>,
    RawQuery(query): RawQuery,
) -> Result<Response, AppError> {
    let (link, notifications) = page(&state, auth.user.id, &paging, query.as_deref()).await?;
    Ok(respond(link, &notifications))
}

async fn show(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Notification>, AppError> {
    let id = Uuid::parse_str(&id).map_err(|_| AppError::Missing(MISSING))?;
    let row = rows::notification_row(state.db(), auth.user.id, id)
        .await?
        .ok_or(AppError::Missing(MISSING))?;
    render(&state, auth.user.id, &[row])
        .await?
        .pop()
        .map(Json)
        .ok_or(AppError::Missing(MISSING))
}

async fn changed(state: &AppState, user: Uuid) {
    state.hub().publish(
        user,
        StreamEvent::Read(ReadEventDto {
            kind: ReadKind::Notifications,
        }),
    );
    publish_badges(state, user).await;
}

async fn clear(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
) -> Result<Json<serde_json::Value>, AppError> {
    rows::clear(state.db(), auth.user.id).await?;
    changed(&state, auth.user.id).await;
    Ok(Json(serde_json::json!({})))
}

async fn dismiss(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = Uuid::parse_str(&id).map_err(|_| AppError::Missing(MISSING))?;
    if !rows::dismiss(state.db(), auth.user.id, id).await? {
        return Err(AppError::Missing(MISSING));
    }
    changed(&state, auth.user.id).await;
    Ok(Json(serde_json::json!({})))
}
