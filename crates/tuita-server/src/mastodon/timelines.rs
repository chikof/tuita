//! Timelines and record lists: `GET /timelines/home|public|tag/{tag}|list/{id}`,
//! `GET /bookmarks` and `GET /favourites`.
//!
//! Every route needs a signed-in account, like tuita's own timelines. The queries are
//! [`crate::db::mastodon_feeds`]: tuita's following, list, tag and for-you queries with `max_id`,
//! `min_id` and `since_id` bounds, so every block, mute, word filter and silenced-domain rule
//! applies alike. What tuita's data cannot do:
//!
//! - `home` has no replies and no `exclude_replies`-style options, as tuita's following feed.
//! - `public` and `tag` list top-level public posts; `local`, `remote` and `only_media` work.
//!   The tag timeline reads its one tag; `any[]`, `all[]` and `none[]` are ignored.
//! - Bookmarks and favourites page by their own record: the cursors are opaque strings.

use axum::Json;
use axum::Router;
use axum::extract::{Path, RawQuery, State};
use axum::http::header::LINK;
use axum::http::{HeaderMap, HeaderValue};
use axum::response::{IntoResponse as _, Response};
use axum::routing::get;
use serde::Serialize;
use uuid::Uuid;

use super::page::Paging;
use super::params::Params;
use super::statuses::render_many;
use crate::auth::CurrentUser;
use crate::cursor::EdgeCursor;
use crate::db::mastodon_feeds::{self as feeds, Entry, PublicScope, Range, Record};
use crate::db::users::UserRow;
use crate::db::{lists, posts};
use crate::error::AppError;
use crate::state::AppState;

/// Longest hashtag a timeline accepts, in bytes.
const MAX_TAG: usize = 100;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/timelines/home", get(home))
        .route("/timelines/public", get(public))
        .route("/timelines/tag/{tag}", get(tag))
        .route("/timelines/list/{id}", get(list))
        .route("/favourites", get(favourites))
}

/// The bounds of a page of ids.
pub(super) fn id_range(paging: &Paging<Uuid>) -> Range<Uuid> {
    Range {
        upper: paging.upper().copied(),
        lower: paging.lower().copied(),
        ascending: paging.ascending(),
    }
}

/// The bounds of a page of records.
pub(super) fn edge_range(paging: &Paging<String>) -> Result<Range<EdgeCursor>, AppError> {
    let parse = |cursor: Option<&String>| cursor.map(|raw| EdgeCursor::parse(raw)).transpose();
    Ok(Range {
        upper: parse(paging.upper())?,
        lower: parse(paging.lower())?,
        ascending: paging.ascending(),
    })
}

/// `body` as JSON with the `Link` header, when there is one.
pub(super) fn respond<T: Serialize>(link: Option<HeaderValue>, body: &T) -> Response {
    let mut headers = HeaderMap::new();
    if let Some(link) = link {
        headers.insert(LINK, link);
    }
    (headers, Json(body)).into_response()
}

async fn entries_page(
    state: &AppState,
    viewer: &UserRow,
    paging: &Paging<Uuid>,
    entries: Vec<Entry>,
) -> Result<Response, AppError> {
    let slice = paging.trim(entries);
    let link = paging.link(
        state.config(),
        slice.rows.first().map(|entry| entry.key),
        slice.rows.last().map(|entry| entry.key),
        slice.more,
    );
    let posts = feeds::posts_of(state.db(), viewer.id, &slice.rows).await?;
    Ok(respond(link, &render_many(state, viewer, posts).await?))
}

async fn ids_page(
    state: &AppState,
    viewer: &UserRow,
    paging: &Paging<Uuid>,
    ids: Vec<Uuid>,
) -> Result<Response, AppError> {
    let slice = paging.trim(ids);
    let link = paging.link(
        state.config(),
        slice.rows.first().copied(),
        slice.rows.last().copied(),
        slice.more,
    );
    let posts = posts::load(state.db(), viewer.id, &slice.rows, None).await?;
    Ok(respond(link, &render_many(state, viewer, posts).await?))
}

async fn home(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    paging: Paging<Uuid>,
) -> Result<Response, AppError> {
    let entries = feeds::home(state.db(), auth.user.id, id_range(&paging), paging.fetch()).await?;
    entries_page(&state, &auth.user, &paging, entries).await
}

async fn public(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    paging: Paging<Uuid>,
    RawQuery(query): RawQuery,
) -> Result<Response, AppError> {
    let params = Params::from_query(query.as_deref());
    let scope = PublicScope {
        local: params.flag("local") == Some(true),
        remote: params.flag("remote") == Some(true),
        only_media: params.flag("only_media") == Some(true),
        tag: None,
    };
    let ids = feeds::public(
        state.db(),
        auth.user.id,
        &scope,
        id_range(&paging),
        paging.fetch(),
    )
    .await?;
    ids_page(&state, &auth.user, &paging, ids).await
}

async fn tag(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(tag): Path<String>,
    paging: Paging<Uuid>,
    RawQuery(query): RawQuery,
) -> Result<Response, AppError> {
    let tag = tag.trim_start_matches('#').to_lowercase();
    if tag.is_empty() || tag.len() > MAX_TAG {
        return Err(AppError::Missing("Record not found."));
    }
    let params = Params::from_query(query.as_deref());
    let scope = PublicScope {
        local: params.flag("local") == Some(true),
        remote: params.flag("remote") == Some(true),
        only_media: params.flag("only_media") == Some(true),
        tag: Some(tag),
    };
    let ids = feeds::public(
        state.db(),
        auth.user.id,
        &scope,
        id_range(&paging),
        paging.fetch(),
    )
    .await?;
    ids_page(&state, &auth.user, &paging, ids).await
}

async fn list(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    paging: Paging<Uuid>,
) -> Result<Response, AppError> {
    let id = Uuid::parse_str(&id).map_err(|_| AppError::Missing("List not found."))?;
    let owned = lists::find(state.db(), auth.user.id, id)
        .await?
        .filter(|list| list.is_owner)
        .ok_or(AppError::Missing("List not found."))?;
    let entries = feeds::list(
        state.db(),
        auth.user.id,
        owned.id,
        id_range(&paging),
        paging.fetch(),
    )
    .await?;
    entries_page(&state, &auth.user, &paging, entries).await
}

async fn record_page(
    state: &AppState,
    viewer: &UserRow,
    paging: &Paging<String>,
    record: Record,
) -> Result<Response, AppError> {
    let rows = feeds::records(
        state.db(),
        viewer.id,
        record,
        edge_range(paging)?,
        paging.fetch(),
    )
    .await?;
    let slice = paging.trim(rows);
    let cursor = |row: &(Uuid, time::OffsetDateTime)| EdgeCursor::new(row.1, row.0).encode();
    let link = paging.link(
        state.config(),
        slice.rows.first().map(cursor),
        slice.rows.last().map(cursor),
        slice.more,
    );
    let ids: Vec<Uuid> = slice.rows.iter().map(|row| row.0).collect();
    let posts = posts::load(state.db(), viewer.id, &ids, None).await?;
    Ok(respond(link, &render_many(state, viewer, posts).await?))
}

/// `GET /bookmarks` for an OAuth token; tuita's own handler answers every other credential.
pub(crate) async fn bookmarks(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    paging: Paging<String>,
) -> Result<Response, AppError> {
    record_page(&state, &auth.user, &paging, Record::Bookmarks).await
}

async fn favourites(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    paging: Paging<String>,
) -> Result<Response, AppError> {
    record_page(&state, &auth.user, &paging, Record::Likes).await
}
