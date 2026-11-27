//! Search, hashtag suggestions, trends and tag pages.

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use tuita_core::search::{MAX_QUERY_LEN, parse_query};
use uuid::Uuid;

use crate::auth::{ApiQuery, CurrentUser};
use crate::cursor::{PageQuery, page};
use crate::db::users::{UserRow, find_local};
use crate::db::{posts, search, users};
use crate::dto::{Page, PostDto, SearchDto, SearchType, TrendDto};
use crate::error::AppError;
use crate::federation::{FedError, ids, lookup};
use crate::state::AppState;

/// People shown above the posts of a posts search.
const PEOPLE_ABOVE_POSTS: i64 = 8;
/// Longest hashtag prefix.
const MAX_PREFIX_LEN: usize = 50;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/search", get(find))
        .route("/hashtags", get(hashtags))
        .route("/trends", get(trends))
        .route("/tags/{tag}/posts", get(tag_posts))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    before: Option<String>,
    limit: Option<u32>,
}

#[derive(Deserialize)]
struct PrefixQuery {
    prefix: Option<String>,
}

#[derive(Deserialize)]
struct ListQuery {
    before: Option<String>,
    limit: Option<u32>,
}

fn empty() -> SearchDto {
    SearchDto {
        users: Vec::new(),
        posts: Vec::new(),
        next: None,
    }
}

/// The result of a search whose whole query is a handle (`@user@domain`, `user@domain`) or the
/// URL of a post: the account or post it names. `None` for any other query. A lookup that fails
/// gives an empty result; only a spent lookup limit is an error.
pub(crate) async fn lookup_hit(
    state: &AppState,
    viewer: &UserRow,
    q: &str,
) -> Result<Option<SearchDto>, AppError> {
    let db = state.db();
    if q.chars().any(char::is_whitespace) {
        return Ok(None);
    }
    if q.starts_with("http://") || q.starts_with("https://") {
        let id = match local_post(state, q) {
            Some(id) => Some(id),
            None => match lookup::fetch_remote_post(state, Some(viewer.id), q).await {
                Ok(id) => Some(id),
                Err(err @ FedError::RateLimited { .. }) => return Err(err.into()),
                Err(err) => {
                    tracing::debug!(url = q, error = %err, "post lookup found nothing");
                    None
                }
            },
        };
        let posts = match id {
            Some(id) => posts::load(db, viewer.id, &[id], None).await?,
            None => Vec::new(),
        };
        return Ok(Some(SearchDto {
            users: Vec::new(),
            posts,
            next: None,
        }));
    }
    let own = state.config().domain();
    let handle = match lookup::parse_handle(q, own) {
        Some(lookup::Handle::Remote { handle, domain }) if domain.contains(['.', ':']) => {
            lookup::resolve_acct(state, Some(viewer.id), &handle, &domain).await
        }
        Some(lookup::Handle::Local(handle)) if q.trim_start_matches('@').contains('@') => {
            find_local(db, &handle, viewer.is_staff())
                .await
                .map_err(|_| FedError::NotFound)
        }
        _ => return Ok(None),
    };
    let users = match handle {
        Ok(row) => vec![users::dto(db, viewer.id, &row).await?],
        Err(err @ FedError::RateLimited { .. }) => return Err(err.into()),
        Err(err) => {
            tracing::debug!(query = q, error = %err, "account lookup found nothing");
            Vec::new()
        }
    };
    Ok(Some(SearchDto {
        users,
        posts: Vec::new(),
        next: None,
    }))
}

/// The id of the post of this instance that `url` names, as `/@handle/{id}` or as its note id.
fn local_post(state: &AppState, url: &str) -> Option<Uuid> {
    let config = state.config();
    if let Some((_, id)) = ids::local_post_from_note_id(config, url) {
        return Some(id);
    }
    let rest = url.strip_prefix(&*config.public_url)?.strip_prefix("/@")?;
    let (_, id) = rest.split_once('/')?;
    Uuid::parse_str(id).ok()
}

async fn find(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiQuery(query): ApiQuery<SearchQuery>,
) -> Result<Json<SearchDto>, AppError> {
    let db = state.db();
    let viewer = auth.user.id;
    let paging = PageQuery {
        before: query.before,
        limit: query.limit,
    };
    let limit = paging.limit()?;
    let q: String = query
        .q
        .as_deref()
        .unwrap_or_default()
        .trim()
        .chars()
        .take(MAX_QUERY_LEN)
        .collect();
    if q.is_empty() {
        return Ok(Json(empty()));
    }
    let kind = query
        .kind
        .as_deref()
        .and_then(SearchType::parse)
        .unwrap_or(SearchType::Posts);
    if paging.before.is_none()
        && matches!(kind, SearchType::Posts | SearchType::People)
        && let Some(found) = Box::pin(lookup_hit(&state, &auth.user, &q)).await?
    {
        return Ok(Json(found));
    }
    let needle = q.strip_prefix('@').unwrap_or(&q);

    if kind == SearchType::People {
        let offset = paging.offset()?;
        let rows = search::people(db, viewer, needle, true, offset, limit.fetch()).await?;
        let Page { items, next } = page(rows, limit, |_| {
            offset.saturating_add(i64::from(limit.get())).to_string()
        });
        return Ok(Json(SearchDto {
            users: users::dtos(db, viewer, &items).await?,
            posts: Vec::new(),
            next,
        }));
    }

    let parsed = parse_query(&q);
    let before = paging.id_cursor()?;
    let found = search::posts(
        db,
        viewer,
        &parsed,
        kind == SearchType::Media,
        before,
        limit,
    )
    .await?;
    let people = if before.is_none()
        && kind == SearchType::Posts
        && !parsed.has_operators
        && !q.starts_with('#')
    {
        let rows = search::people(db, viewer, needle, false, 0, PEOPLE_ABOVE_POSTS).await?;
        users::dtos(db, viewer, &rows).await?
    } else {
        Vec::new()
    };
    Ok(Json(SearchDto {
        users: people,
        posts: posts::load(db, viewer, &found.items, None).await?,
        next: found.next,
    }))
}

async fn hashtags(
    State(state): State<AppState>,
    CurrentUser(_auth): CurrentUser,
    ApiQuery(query): ApiQuery<PrefixQuery>,
) -> Result<Json<Vec<TrendDto>>, AppError> {
    let raw = query.prefix.unwrap_or_default().to_lowercase();
    let prefix: String = raw
        .strip_prefix('#')
        .unwrap_or(&raw)
        .chars()
        .take(MAX_PREFIX_LEN)
        .collect();
    Ok(Json(search::hashtags(state.db(), &prefix).await?))
}

async fn trends(State(state): State<AppState>) -> Result<Json<Vec<TrendDto>>, AppError> {
    Ok(Json(search::trends(state.db()).await?))
}

async fn tag_posts(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(tag): Path<String>,
    ApiQuery(query): ApiQuery<ListQuery>,
) -> Result<Json<Page<PostDto>>, AppError> {
    let paging = PageQuery {
        before: query.before,
        limit: query.limit,
    };
    let limit = paging.limit()?;
    let before = paging.id_cursor()?;
    let viewer = auth.user.id;
    let found = search::tag_posts(state.db(), viewer, &tag.to_lowercase(), before, limit).await?;
    Ok(Json(Page {
        items: posts::load(state.db(), viewer, &found.items, None).await?,
        next: found.next,
    }))
}
