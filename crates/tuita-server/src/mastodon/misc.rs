//! The routes Mastodon clients call at start-up and on common screens: `GET /preferences`,
//! `/followed_tags`, `/follow_requests`, `/endorsements`, `/domain_blocks`, `/blocks`, `/mutes`,
//! `/filters` (v1 and v2), `/announcements`, `/trends/tags|statuses|links`, `/suggestions`
//! (v1 and v2), `/accounts/familiar_followers`, `/instance/rules` and `GET /api/v2/notifications`.
//!
//! Each route answers with tuita's own data where tuita has it, and with the empty value Mastodon
//! clients expect where tuita has no such thing:
//!
//! | Route | Data |
//! |---|---|
//! | `preferences` | Mastodon's defaults: tuita keeps no posting or reading defaults |
//! | `blocks`, `mutes` | the viewer's block and mute lists, paged by account id |
//! | `filters` | the viewer's muted words; none, `[]` |
//! | `announcements` | the shown announcements |
//! | `trends/tags` | tuita's trending hashtags with their daily use |
//! | `suggestions` | `graph::suggestions` |
//! | `accounts/familiar_followers` | followers of an account whom the viewer follows |
//! | `instance/rules` | the instance rules |
//! | `notifications` (v2) | the notification list, one group per notification |
//! | `followed_tags`, `endorsements`, `domain_blocks`, `trends/statuses`, `trends/links` | `[]`: no such data |
//! | `follow_requests` | `[]`: no account is locked, so a follow never waits for approval |
//!
//! Every route needs a sign-in and the `read` scope. `/announcements` and `/suggestions` share
//! their path with tuita's own routes and are registered there with [`super::by_credential`].

use std::collections::HashSet;

use axum::extract::{RawQuery, State};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use super::accounts::{MAX_RELATIONSHIPS, bounds, paged, render};
use super::ctx::Ctx;
use super::dto::{
    self, Account, Announcement, FamiliarFollowers, Filter, FilterV1, Preferences, Rule,
    Suggestion, Tag,
};
use super::instance::rules_of;
use super::notifications;
use super::page::Paging;
use super::params::Params;
use super::timelines::respond;
use crate::auth::CurrentUser;
use crate::db::mastodon_accounts::{EdgeSet, edge_accounts, visible_accounts};
use crate::db::mastodon_misc::tag_history;
use crate::db::{announcements, graph, search, settings};
use crate::dto::UserDto;
use crate::error::AppError;
use crate::state::AppState;

/// Trends shown when the client sends no `limit`, and the most it may ask for.
const TRENDS_DEFAULT: usize = 10;
const TRENDS_MAX: usize = 20;
/// Suggestions returned when the client sends no `limit`, and the most it may ask for.
const SUGGESTIONS_DEFAULT: i64 = 40;
const SUGGESTIONS_MAX: i64 = 80;
/// Familiar followers listed per account.
const FAMILIAR_LIMIT: i64 = 20;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/preferences", get(preferences))
        .route("/followed_tags", get(none))
        .route("/follow_requests", get(none))
        .route("/endorsements", get(none))
        .route("/domain_blocks", get(none))
        .route("/blocks", get(blocks))
        .route("/mutes", get(mutes))
        .route("/filters", get(filters_v1))
        .route("/trends/tags", get(trend_tags))
        .route("/trends/statuses", get(none))
        .route("/trends/links", get(none))
        .route("/accounts/familiar_followers", get(familiar_followers))
        .route("/instance/rules", get(rules))
}

pub(super) fn router_v2() -> Router<AppState> {
    Router::new()
        .route("/filters", get(filters_v2))
        .route("/suggestions", get(suggestions_v2))
        .route("/notifications", get(notifications_v2))
}

/// The empty list of a feature tuita does not have.
async fn none(CurrentUser(_): CurrentUser) -> Json<[(); 0]> {
    Json([])
}

async fn preferences(CurrentUser(_): CurrentUser) -> Json<Preferences> {
    Json(dto::preferences())
}

// ------------------------------------------------------------------ blocks and mutes

/// One page of the accounts the viewer blocked or muted, with the `Link` header.
async fn edge_list(
    state: &AppState,
    viewer: Uuid,
    set: EdgeSet,
    paging: &Paging<Uuid>,
) -> Result<(Vec<Account>, Option<axum::http::HeaderValue>), AppError> {
    let rows = edge_accounts(state.db(), set, viewer, bounds(paging)).await?;
    let slice = paging.trim(rows);
    let link = paging.link(
        state.config(),
        slice.rows.first().map(|row| row.id),
        slice.rows.last().map(|row| row.id),
        slice.more,
    );
    Ok((render(state, &slice.rows).await?, link))
}

async fn blocks(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    paging: Paging<Uuid>,
) -> Result<Response, AppError> {
    let (accounts, link) = edge_list(&state, auth.user.id, EdgeSet::Blocks, &paging).await?;
    Ok(paged(&accounts, link))
}

async fn mutes(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    paging: Paging<Uuid>,
) -> Result<Response, AppError> {
    let (accounts, link) = edge_list(&state, auth.user.id, EdgeSet::Mutes, &paging).await?;
    let muted: Vec<_> = accounts.into_iter().map(dto::muted_account).collect();
    Ok(paged(&muted, link))
}

// ------------------------------------------------------------------ filters

async fn filters_v1(CurrentUser(auth): CurrentUser) -> Json<Vec<FilterV1>> {
    Json(dto::filters_v1(
        auth.user.muted_words.as_deref().unwrap_or_default(),
    ))
}

async fn filters_v2(CurrentUser(auth): CurrentUser) -> Json<Vec<Filter>> {
    Json(dto::filters(
        auth.user.muted_words.as_deref().unwrap_or_default(),
    ))
}

// ------------------------------------------------------------------ announcements

/// `GET /announcements` for an OAuth token; shared with tuita's route.
pub(crate) async fn announcement_list(
    State(state): State<AppState>,
    CurrentUser(_): CurrentUser,
) -> Result<Json<Vec<Announcement>>, AppError> {
    let shown = announcements::shown(state.db()).await?;
    Ok(Json(
        shown
            .iter()
            .map(|card| dto::announcement(card, state.config()))
            .collect(),
    ))
}

// ------------------------------------------------------------------ trends

async fn trend_tags(
    State(state): State<AppState>,
    CurrentUser(_): CurrentUser,
    RawQuery(query): RawQuery,
) -> Result<Json<Vec<Tag>>, AppError> {
    let params = Params::from_query(query.as_deref());
    let count = |key: &str| usize::try_from(params.int(key).unwrap_or(0)).unwrap_or(0);
    let limit = match count("limit") {
        0 => TRENDS_DEFAULT,
        asked => asked.min(TRENDS_MAX),
    };
    let trends = search::trends(state.db()).await?;
    let page: Vec<String> = trends
        .into_iter()
        .skip(count("offset"))
        .take(limit)
        .map(|trend| trend.tag)
        .collect();
    let history = tag_history(state.db(), &page).await?;
    Ok(Json(
        page.iter()
            .map(|name| {
                let days = history.get(name).map_or(&[][..], Vec::as_slice);
                dto::trending_tag(name, days, state.config())
            })
            .collect(),
    ))
}

// ------------------------------------------------------------------ suggestions

/// The suggested accounts for `viewer`, as tuita ranks them, with the context to map them.
async fn suggested(
    state: &AppState,
    viewer: Uuid,
    query: Option<&str>,
) -> Result<Vec<Account>, AppError> {
    let limit = Params::from_query(query)
        .int("limit")
        .filter(|limit| *limit > 0)
        .map_or(SUGGESTIONS_DEFAULT, |limit| limit.min(SUGGESTIONS_MAX));
    let users: Vec<UserDto> = graph::suggestions(state.db(), viewer, limit).await?;
    let ids: Vec<Uuid> = users.iter().map(|user| user.id).collect();
    let ctx = Ctx::for_users(state, &ids).await?;
    Ok(users.iter().map(|user| dto::account(user, &ctx)).collect())
}

/// `GET /suggestions` for an OAuth token; shared with tuita's route. Mastodon's v1 answer is a
/// list of accounts.
pub(crate) async fn suggestions(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    RawQuery(query): RawQuery,
) -> Result<Json<Vec<Account>>, AppError> {
    Ok(Json(
        suggested(&state, auth.user.id, query.as_deref()).await?,
    ))
}

async fn suggestions_v2(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    RawQuery(query): RawQuery,
) -> Result<Json<Vec<Suggestion>>, AppError> {
    let accounts = suggested(&state, auth.user.id, query.as_deref()).await?;
    Ok(Json(accounts.into_iter().map(dto::suggestion).collect()))
}

// ------------------------------------------------------------------ the rest

/// For each account in `id[]`: its followers whom the viewer follows too. Unknown and hidden
/// accounts are left out; the viewer's own account has no familiar followers.
async fn familiar_followers(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    RawQuery(query): RawQuery,
) -> Result<Json<Vec<FamiliarFollowers>>, AppError> {
    let params = Params::from_query(query.as_deref());
    let mut seen = HashSet::new();
    let asked: Vec<Uuid> = params
        .list("id")
        .iter()
        .filter_map(|raw| Uuid::parse_str(raw).ok())
        .filter(|id| seen.insert(*id))
        .take(MAX_RELATIONSHIPS)
        .collect();
    let found: HashSet<Uuid> = visible_accounts(state.db(), &asked, auth.user.is_staff())
        .await?
        .into_iter()
        .map(|row| row.id)
        .collect();
    let mut known = Vec::new();
    for id in asked.into_iter().filter(|id| found.contains(id)) {
        let users = if id == auth.user.id {
            Vec::new()
        } else {
            graph::known_followers(state.db(), auth.user.id, id, FAMILIAR_LIMIT)
                .await?
                .users
        };
        known.push((id, users));
    }
    let ids: Vec<Uuid> = known
        .iter()
        .flat_map(|(_, users)| users.iter().map(|user| user.id))
        .collect();
    let ctx = Ctx::for_users(&state, &ids).await?;
    Ok(Json(
        known
            .iter()
            .map(|(id, users)| FamiliarFollowers {
                id: *id,
                accounts: users
                    .iter()
                    .map(|user| dto::account_from_ref(user, &ctx))
                    .collect(),
            })
            .collect(),
    ))
}

async fn rules(
    State(state): State<AppState>,
    CurrentUser(_): CurrentUser,
) -> Result<Json<Vec<Rule>>, AppError> {
    let loaded = settings::load(state.db()).await?;
    Ok(Json(rules_of(&loaded.rules)))
}

/// `GET /api/v2/notifications`: the same page as v1, one group per notification.
async fn notifications_v2(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    paging: Paging<Uuid>,
    RawQuery(query): RawQuery,
) -> Result<Response, AppError> {
    let (link, page) = notifications::page(&state, auth.user.id, &paging, query.as_deref()).await?;
    Ok(respond(link, &dto::grouped_notifications(page)))
}
