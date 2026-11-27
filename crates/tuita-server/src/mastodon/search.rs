//! Search: `GET /api/v2/search`.
//!
//! It runs the queries of tuita's own search: people by handle, display name and bio, posts by
//! text and the operators of [`tuita_core::search`], hashtags by prefix, each as the caller may
//! see them (blocks, mutes, silenced instances, `discoverable`). With `resolve=true` a query that
//! is a whole handle (`@bob@example.org`) or the URL of a post is looked up on its own server,
//! with the same limits as tuita's search; without `resolve` the network is never touched.
//!
//! `limit` is 1 to 40 (default 20). `offset` skips rows, up to [`MAX_OFFSET`]. Mastodon's
//! `account_id`, `exclude_unreviewed`, `min_id` and `max_id` are ignored. A handle or URL answer is
//! the whole result for the first page.

use axum::extract::{RawQuery, State};
use axum::routing::get;
use axum::{Json, Router};
use time::{OffsetDateTime, Time};
use tuita_core::search::{MAX_QUERY_LEN, parse_query};
use uuid::Uuid;

use super::accounts::render;
use super::ctx::Ctx;
use super::dto::{self, SearchResults, TagHistory};
use super::params::Params;
use crate::auth::CurrentUser;
use crate::db::mastodon_accounts::tag_usage;
use crate::db::{posts, search, users};
use crate::dto::PostDto;
use crate::error::AppError;
use crate::routes::search::lookup_hit;
use crate::state::AppState;

/// Rows a client may skip. Deeper pages cost the database more than a search is worth.
const MAX_OFFSET: i64 = 500;
/// Longest hashtag prefix.
const MAX_PREFIX_LEN: usize = 50;
/// Rows of a page unless the client asks for another number.
const DEFAULT_LIMIT: i64 = 20;
/// Most rows of a page.
const MAX_LIMIT: i64 = 40;

pub(super) fn router_v2() -> Router<AppState> {
    Router::new().route("/search", get(find))
}

/// Which kinds of result the client wants.
#[derive(Clone, Copy)]
struct Kinds {
    accounts: bool,
    statuses: bool,
    hashtags: bool,
}

impl Kinds {
    fn parse(raw: Option<&str>) -> Result<Self, AppError> {
        let only = |accounts, statuses, hashtags| Self {
            accounts,
            statuses,
            hashtags,
        };
        match raw.map(str::trim) {
            None | Some("") => Ok(only(true, true, true)),
            Some("accounts") => Ok(only(true, false, false)),
            Some("statuses") => Ok(only(false, true, false)),
            Some("hashtags") => Ok(only(false, false, true)),
            Some(_) => Err(AppError::unprocessable(
                "Validation failed: Type is not included in the list",
            )),
        }
    }
}

async fn find(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    RawQuery(query): RawQuery,
) -> Result<Json<SearchResults>, AppError> {
    let params = Params::from_query(query.as_deref());
    let q: String = params
        .get("q")
        .ok_or_else(|| AppError::bad_request("param is missing or the value is empty: q"))?
        .trim()
        .chars()
        .take(MAX_QUERY_LEN)
        .collect();
    let kinds = Kinds::parse(params.get("type"))?;
    let limit = params
        .int("limit")
        .unwrap_or(DEFAULT_LIMIT)
        .clamp(1, MAX_LIMIT);
    let offset = params.int("offset").unwrap_or(0).clamp(0, MAX_OFFSET);
    let resolve = params.flag("resolve").unwrap_or(false);
    let following = params.flag("following").unwrap_or(false);
    if q.is_empty() {
        return Ok(Json(SearchResults::default()));
    }
    let db = state.db();
    let viewer = auth.user.id;
    let mut results = SearchResults::default();

    // A whole handle or a post URL is answered by one lookup, as tuita's own search does.
    if resolve
        && offset == 0
        && (kinds.accounts || kinds.statuses)
        && let Some(found) = Box::pin(lookup_hit(&state, &auth.user, &q)).await?
    {
        if kinds.accounts {
            let ids: Vec<Uuid> = found.users.iter().map(|user| user.id).collect();
            results.accounts = accounts_by_id(&state, &ids).await?;
        }
        if kinds.statuses {
            results.statuses = statuses_of(&state, &found.posts).await?;
        }
        return Ok(Json(results));
    }

    let needle = q.strip_prefix('@').unwrap_or(&q);
    if kinds.accounts {
        let rows = search::people(db, viewer, needle, true, offset, limit).await?;
        let rows = if following {
            let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
            let relations = users::relations(db, viewer, &ids).await?;
            rows.into_iter()
                .filter(|row| relations.get(&row.id).is_some_and(|rel| rel.following))
                .collect()
        } else {
            rows
        };
        results.accounts = render(&state, &rows).await?;
    }
    if kinds.statuses {
        let ids =
            search::post_ids(db, viewer, &parse_query(&q), false, None, offset, limit).await?;
        let found = posts::load(db, viewer, &ids, None).await?;
        results.statuses = statuses_of(&state, &found).await?;
    }
    if kinds.hashtags && offset == 0 && !q.contains(char::is_whitespace) {
        let lower = q.to_lowercase();
        let prefix: String = lower
            .strip_prefix('#')
            .unwrap_or(&lower)
            .chars()
            .take(MAX_PREFIX_LEN)
            .collect();
        let found = search::hashtags(db, &prefix).await?;
        let names: Vec<String> = found
            .into_iter()
            .take(limit as usize)
            .map(|trend| trend.tag)
            .collect();
        let usage = tag_usage(db, &names).await?;
        let today = OffsetDateTime::now_utc()
            .replace_time(Time::MIDNIGHT)
            .unix_timestamp();
        results.hashtags = names
            .iter()
            .map(|name| {
                let mut tag = dto::tag(name, state.config());
                tag.history = (0..7)
                    .map(|back| {
                        let day = today - back * 86_400;
                        let (uses, accounts) = usage
                            .iter()
                            .find(|(tag, at, _, _)| tag == name && *at == day)
                            .map_or((0, 0), |(_, _, uses, accounts)| (*uses, *accounts));
                        TagHistory {
                            day: day.to_string(),
                            uses: uses.to_string(),
                            accounts: accounts.to_string(),
                        }
                    })
                    .collect();
                tag
            })
            .collect();
    }
    Ok(Json(results))
}

async fn accounts_by_id(state: &AppState, ids: &[Uuid]) -> Result<Vec<dto::Account>, AppError> {
    let ctx = Ctx::for_users(state, ids).await?;
    Ok(ids
        .iter()
        .filter_map(|id| ctx.user(*id).map(|user| dto::account(user, &ctx)))
        .collect())
}

async fn statuses_of(state: &AppState, posts: &[PostDto]) -> Result<Vec<dto::Status>, AppError> {
    let ctx = Ctx::for_posts(state, posts).await?;
    Ok(dto::statuses(posts, &ctx))
}
