//! Statuses: `POST /statuses`, `GET|PUT|DELETE /statuses/{id}`, `/context`, `/source`,
//! `/history`, `favourite`, `unfavourite`, `reblog`, `unreblog`, `bookmark`, `unbookmark`, `pin`,
//! `unpin`, `favourited_by`, `reblogged_by`, and the scheduled statuses.
//!
//! Every write goes through the functions tuita's own routes use ([`compose`], [`post_actions`],
//! [`scheduled`]), so the word filter, reply policy, rate limits, blocks, mutes, media ownership
//! and limits, quote handling and federation delivery apply the same way. Every route that takes
//! a status id first loads the status as the viewer may see it, so a followers-only, direct or
//! blocked-author post answers `404` like a missing one.
//!
//! What tuita cannot do is refused with `422`, never approximated:
//!
//! - `visibility` other than `public` (tuita's local posts are public).
//! - A poll shorter than one hour, or with `hide_totals`; a longer one runs for whole hours,
//!   rounded up.
//! - An edit that changes media, the poll or `sensitive`: tuita edits text and content warning,
//!   within 30 minutes. The same values sent back unchanged are accepted.
//! - Boosting a status that is not public or unlisted.
//!
//! `language` is accepted and ignored. Tuita has one pinned post per account, so pinning another
//! status replaces the pin.

use std::collections::{HashMap, HashSet};

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse as _, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::Serialize;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tuita_core::emoji::shortcodes;
use uuid::Uuid;

use super::ctx::Ctx;
use super::dto::{self, Account, CustomEmoji, MediaAttachment, Poll, Status};
use super::page::Paging;
use super::params::{Form, Params};
use super::timelines::{edge_range, respond};
use crate::auth::CurrentUser;
use crate::cursor::EdgeCursor;
use crate::db::emoji;
use crate::db::mastodon as queries;
use crate::db::mastodon_activity::{self as activity, Claim};
use crate::db::mastodon_feeds::{self as feeds, ReactorTable};
use crate::db::media::MAX_IMAGES;
use crate::db::post_actions::{self, Reaction};
use crate::db::sessions::ClientMeta;
use crate::db::users::{self, UserRow, user_ref};
use crate::db::{compose, posts, scheduled};
use crate::dto::{
    CreatePostRequest, EditPostRequest, EmojiDto, PollInput, PostDto, ScheduleRequest,
    ScheduledPostDto, ScheduledStatus, Visibility,
};
use crate::error::AppError;
use crate::state::AppState;

const MISSING: &str = "Post not found.";
/// Longest `Idempotency-Key` accepted, in bytes.
const MAX_KEY: usize = 255;
/// Most poll answers read before validation counts them.
const MAX_POLL_OPTIONS: usize = 10;
const SECS_PER_HOUR: i64 = 3600;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/statuses", post(create))
        // `GET /statuses/{id}` is registered with the M4 embed handler, which hands signed-in
        // requests to `show`.
        .route("/statuses/{id}", put(edit).delete(remove))
        .route("/statuses/{id}/context", get(context))
        .route("/statuses/{id}/source", get(source))
        .route("/statuses/{id}/history", get(history))
        .route("/statuses/{id}/favourite", post(favourite))
        .route("/statuses/{id}/unfavourite", post(unfavourite))
        .route("/statuses/{id}/reblog", post(reblog))
        .route("/statuses/{id}/unreblog", post(unreblog))
        .route("/statuses/{id}/bookmark", post(bookmark))
        .route("/statuses/{id}/unbookmark", post(unbookmark))
        .route("/statuses/{id}/pin", post(pin))
        .route("/statuses/{id}/unpin", post(unpin))
        .route("/statuses/{id}/favourited_by", get(favourited_by))
        .route("/statuses/{id}/reblogged_by", get(reblogged_by))
        .route("/scheduled_statuses", get(scheduled_list))
        .route(
            "/scheduled_statuses/{id}",
            get(scheduled_one)
                .put(scheduled_move)
                .delete(scheduled_remove),
        )
}

// ------------------------------------------------------------------ shared helpers

fn missing() -> AppError {
    AppError::Missing(MISSING)
}

fn parse_status_id(raw: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(raw).map_err(|_| missing())
}

/// Validation failures are `422` in Mastodon's API; tuita's rules answer `400` or `409`.
fn invalid(err: AppError) -> AppError {
    match err {
        AppError::BadRequest(message) | AppError::Conflict(message) => {
            AppError::Unprocessable(message)
        }
        other => other,
    }
}

fn unprocessable(message: &str) -> AppError {
    AppError::unprocessable(message)
}

/// Maps posts to statuses with the viewer's pin set, in a few batched queries.
///
/// # Errors
///
/// Database failures.
pub(super) async fn render_many(
    state: &AppState,
    viewer: &UserRow,
    mut posts: Vec<PostDto>,
) -> Result<Vec<Status>, AppError> {
    if posts.is_empty() {
        return Ok(Vec::new());
    }
    let pinned = activity::pinned_post(state.db(), viewer.id).await?;
    for post in &mut posts {
        post.pinned = pinned == Some(post.id) && post.author.id == viewer.id;
    }
    let ctx = Ctx::for_posts(state, &posts).await?;
    Ok(dto::statuses(&posts, &ctx))
}

/// [`render_many`] for one post.
pub(super) async fn render_one(
    state: &AppState,
    viewer: &UserRow,
    post: PostDto,
) -> Result<Status, AppError> {
    render_many(state, viewer, vec![post])
        .await?
        .pop()
        .ok_or_else(missing)
}

/// Post `id` as `viewer` may see it: a missing post, a followers-only or direct post of someone
/// else and a blocked author's post are `404`. A muted author's post stays readable.
async fn visible_post_of(state: &AppState, viewer: Uuid, id: Uuid) -> Result<PostDto, AppError> {
    let author = queries::post_authors(state.db(), &[id])
        .await?
        .get(&id)
        .copied()
        .ok_or_else(missing)?;
    posts::load_one(state.db(), viewer, id, Some(author))
        .await?
        .ok_or_else(missing)
}

/// `GET /statuses/{id}` for a signed-in request: the status with the viewer's state.
///
/// # Errors
///
/// `404` unless the viewer may see the post.
pub(crate) async fn show(
    state: &AppState,
    viewer: &UserRow,
    id: Uuid,
) -> Result<Response, AppError> {
    let post = visible_post_of(state, viewer.id, id).await?;
    Ok(Json(render_one(state, viewer, post).await?).into_response())
}

fn own(post: &PostDto, viewer: &UserRow) -> Result<(), AppError> {
    if post.author.id == viewer.id {
        Ok(())
    } else {
        Err(AppError::forbidden("This action is not allowed."))
    }
}

// ------------------------------------------------------------------ creating

fn check_visibility(params: &Params) -> Result<(), AppError> {
    match params
        .get("visibility")
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        None | Some("public") => Ok(()),
        Some("unlisted" | "private" | "direct") => Err(unprocessable(
            "Validation failed: this instance only supports public posts, set visibility to \"public\".",
        )),
        Some(_) => Err(unprocessable(
            "Validation failed: Visibility is not included in the list.",
        )),
    }
}

fn uuid_list(params: &Params, key: &str) -> Result<Vec<Uuid>, AppError> {
    let raw = params.list(key);
    if raw.len() > MAX_IMAGES {
        return Err(unprocessable(&format!(
            "Validation failed: You can attach up to {MAX_IMAGES} images."
        )));
    }
    raw.iter()
        .map(|value| {
            Uuid::parse_str(value.trim())
                .map_err(|_| unprocessable("Validation failed: Media attachment is invalid."))
        })
        .collect()
}

fn poll_input(params: &Params) -> Result<Option<PollInput>, AppError> {
    if !params.has("poll[options]") {
        return Ok(None);
    }
    let options = params.list("poll[options]");
    if options.len() > MAX_POLL_OPTIONS {
        return Err(unprocessable(
            "Validation failed: Polls need 2 to 4 choices.",
        ));
    }
    let seconds = params
        .int("poll[expires_in]")
        .ok_or_else(|| unprocessable("Validation failed: Poll duration is required."))?;
    if seconds < SECS_PER_HOUR {
        return Err(unprocessable(
            "Validation failed: Polls on this instance run for at least 1 hour.",
        ));
    }
    if params.flag("poll[hide_totals]") == Some(true) {
        return Err(unprocessable(
            "Validation failed: this instance can't hide poll totals.",
        ));
    }
    let hours = seconds.saturating_add(SECS_PER_HOUR - 1) / SECS_PER_HOUR;
    Ok(Some(PollInput {
        options: options.to_vec(),
        multiple: params.flag("poll[multiple]") == Some(true),
        duration_hours: u32::try_from(hours).unwrap_or(u32::MAX),
    }))
}

fn post_request(params: &Params) -> Result<CreatePostRequest, AppError> {
    let reply_to = match params.get("in_reply_to_id").map(str::trim) {
        None | Some("") => None,
        Some(raw) => Some(Uuid::parse_str(raw).map_err(|_| missing())?),
    };
    let quote_of = match params.get("quoted_status_id").map(str::trim) {
        None | Some("") => None,
        Some(raw) => Some(Uuid::parse_str(raw).map_err(|_| missing())?),
    };
    Ok(CreatePostRequest {
        text: params.get("status").unwrap_or_default().to_owned(),
        cw: params.get("spoiler_text").unwrap_or_default().to_owned(),
        sensitive: params.flag("sensitive") == Some(true),
        reply_to,
        quote_of,
        reply_policy: None,
        media_ids: uuid_list(params, "media_ids")?,
        poll: poll_input(params)?,
    })
}

fn idempotency_key(headers: &HeaderMap) -> Result<Option<String>, AppError> {
    let Some(value) = headers.get("idempotency-key") else {
        return Ok(None);
    };
    let key = value
        .to_str()
        .map_err(|_| unprocessable("Validation failed: Idempotency-Key is invalid."))?
        .trim();
    if key.is_empty() {
        return Ok(None);
    }
    if key.len() > MAX_KEY {
        return Err(unprocessable(
            "Validation failed: Idempotency-Key is too long.",
        ));
    }
    Ok(Some(key.to_owned()))
}

enum Made {
    Post(Uuid),
    Scheduled(Box<ScheduledPostDto>),
}

async fn make(
    state: &AppState,
    viewer: &UserRow,
    client: Option<&'static str>,
    req: CreatePostRequest,
    at: Option<OffsetDateTime>,
) -> Result<Made, AppError> {
    if let Some(publish_at) = at {
        state.limits().check_schedule(viewer.id)?;
        let entry = scheduled::create(
            state,
            viewer,
            client,
            ScheduleRequest {
                publish_at,
                post: req,
            },
        )
        .await
        .map_err(invalid)?;
        return Ok(Made::Scheduled(Box::new(entry)));
    }
    state.limits().check_post(viewer.id)?;
    let id = compose::create(state, viewer, client, req)
        .await
        .map_err(invalid)?;
    Ok(Made::Post(id))
}

async fn made_response(
    state: &AppState,
    viewer: &UserRow,
    made: Made,
) -> Result<Response, AppError> {
    match made {
        Made::Post(id) => show(state, viewer, id).await,
        Made::Scheduled(entry) => Ok(Json(scheduled_json(state, &entry)).into_response()),
    }
}

async fn create(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    meta: ClientMeta,
    headers: HeaderMap,
    Form { params, .. }: Form,
) -> Result<Response, AppError> {
    let viewer = &auth.user;
    check_visibility(&params)?;
    let req = post_request(&params)?;
    let at = match params.get("scheduled_at").map(str::trim) {
        None | Some("") => None,
        Some(raw) => Some(
            OffsetDateTime::parse(raw, &Rfc3339)
                .map_err(|_| unprocessable("Validation failed: Scheduled at is invalid."))?,
        ),
    };
    let key = idempotency_key(&headers)?;
    if let Some(key) = &key {
        match activity::claim_key(state.db(), viewer.id, key).await? {
            Claim::Fresh => {}
            Claim::Post(id) => return show(&state, viewer, id).await,
            Claim::Scheduled(id) => {
                let entry = scheduled_entry(&state, viewer, id).await?;
                return Ok(Json(scheduled_json(&state, &entry)).into_response());
            }
            Claim::Running => {
                return Err(AppError::conflict(
                    "A request with this Idempotency-Key is still running.",
                ));
            }
        }
    }
    let made = make(&state, viewer, meta.client, req, at).await;
    if let Some(key) = &key {
        let kept = match &made {
            Ok(Made::Post(id)) => {
                activity::finish_key(state.db(), viewer.id, key, Some(*id), None).await
            }
            Ok(Made::Scheduled(entry)) => {
                activity::finish_key(state.db(), viewer.id, key, None, Some(entry.id)).await
            }
            Err(_) => activity::release_key(state.db(), viewer.id, key).await,
        };
        if let Err(err) = kept {
            tracing::warn!(error = %err, "cannot update an idempotency key");
        }
    }
    made_response(&state, viewer, made?).await
}

// ------------------------------------------------------------------ editing and deleting

fn same_set(sent: &[Uuid], have: &[Uuid]) -> bool {
    let mut sent = sent.to_vec();
    let mut have = have.to_vec();
    sent.sort_unstable();
    have.sort_unstable();
    sent == have
}

/// Refuses the parts of an edit tuita cannot apply, unless they repeat what the post has.
fn check_unchanged(params: &Params, post: &PostDto) -> Result<(), AppError> {
    const FROZEN: &str =
        "Validation failed: this instance edits the text and content warning only.";
    let new_cw = params
        .get("spoiler_text")
        .map_or(post.cw.as_str(), str::trim);
    if params.has("media_ids") {
        let media: Vec<Uuid> = post.media.iter().map(|media| media.id).collect();
        if !same_set(&uuid_list(params, "media_ids")?, &media) {
            return Err(unprocessable(FROZEN));
        }
    }
    if params.has("poll[options]") {
        let same = post.poll.as_ref().is_some_and(|poll| {
            poll.options
                .iter()
                .map(|option| option.text.as_str())
                .eq(params.list("poll[options]").iter().map(String::as_str))
        });
        if !same {
            return Err(unprocessable(FROZEN));
        }
    }
    if params
        .flag("sensitive")
        .is_some_and(|sensitive| sensitive != (post.sensitive || !new_cw.is_empty()))
    {
        return Err(unprocessable(FROZEN));
    }
    Ok(())
}

async fn edit(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    Form { params, .. }: Form,
) -> Result<Json<Status>, AppError> {
    let id = parse_status_id(&id)?;
    let viewer = &auth.user;
    let post = visible_post_of(&state, viewer.id, id).await?;
    own(&post, viewer)?;
    check_unchanged(&params, &post)?;
    state.limits().check_edit(viewer.id)?;
    let req = EditPostRequest {
        text: params.get("status").map(str::to_owned),
        cw: params.get("spoiler_text").map(str::to_owned),
    };
    compose::edit(&state, viewer, id, req)
        .await
        .map_err(invalid)?;
    let post = visible_post_of(&state, viewer.id, id).await?;
    Ok(Json(render_one(&state, viewer, post).await?))
}

async fn remove(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Status>, AppError> {
    let id = parse_status_id(&id)?;
    let viewer = &auth.user;
    let post = visible_post_of(&state, viewer.id, id).await?;
    own(&post, viewer)?;
    let text = post.text.clone();
    let mut status = render_one(&state, viewer, post).await?;
    status.text = Some(text);
    post_actions::delete(&state, viewer, id).await?;
    Ok(Json(status))
}

async fn source(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = parse_status_id(&id)?;
    let post = visible_post_of(&state, auth.user.id, id).await?;
    own(&post, &auth.user)?;
    Ok(Json(serde_json::json!({
        "id": post.id,
        "text": post.text,
        "spoiler_text": post.cw,
    })))
}

#[derive(Serialize)]
struct StatusEdit {
    content: String,
    spoiler_text: String,
    sensitive: bool,
    created_at: String,
    account: Account,
    poll: Option<Poll>,
    media_attachments: Vec<MediaAttachment>,
    emojis: Vec<CustomEmoji>,
}

async fn history(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<StatusEdit>>, AppError> {
    let id = parse_status_id(&id)?;
    let post = visible_post_of(&state, auth.user.id, id).await?;
    let versions = compose::edits(state.db(), id).await?;
    let ctx = Ctx::for_posts(&state, std::slice::from_ref(&post)).await?;
    let account = dto::account_from_ref(&post.author, &ctx);
    let media: Vec<MediaAttachment> = post
        .media
        .iter()
        .map(|media| dto::media_attachment(media, &ctx))
        .collect();
    let poll_emojis = dto::custom_emojis(state.config(), &post.emojis);
    let poll = post
        .poll
        .as_ref()
        .map(|poll| dto::poll(post.id, poll, &poll_emojis));
    let texts: Vec<&str> = versions
        .edits
        .iter()
        .map(|version| (version.text.as_str(), version.cw.as_str()))
        .chain([(versions.current.text.as_str(), versions.current.cw.as_str())])
        .flat_map(|(text, cw)| [text, cw])
        .collect();
    let catalog = emoji::resolve(state.db(), post.author.domain.as_deref(), &texts).await?;
    let emojis_of = |text: &str, cw: &str| -> Vec<CustomEmoji> {
        let mut found: Vec<&EmojiDto> = Vec::new();
        for code in shortcodes(text).into_iter().chain(shortcodes(cw)) {
            let known = catalog
                .iter()
                .find(|emoji| emoji.shortcode.eq_ignore_ascii_case(code));
            if let Some(known) = known
                && !found.contains(&known)
            {
                found.push(known);
            }
        }
        found
            .into_iter()
            .map(|emoji| dto::custom_emoji(state.config(), emoji))
            .collect()
    };
    let edit = |text: &str, cw: &str, at: OffsetDateTime| StatusEdit {
        content: crate::federation::render::content_html(state.config(), text, &[]),
        spoiler_text: cw.to_owned(),
        sensitive: post.sensitive || !cw.is_empty(),
        created_at: dto::iso(at),
        account: account.clone(),
        poll: poll.clone(),
        media_attachments: media.clone(),
        emojis: emojis_of(text, cw),
    };
    let mut out: Vec<StatusEdit> = versions
        .edits
        .iter()
        .rev()
        .map(|version| edit(&version.text, &version.cw, version.at))
        .collect();
    out.push(edit(
        &versions.current.text,
        &versions.current.cw,
        versions.current.at,
    ));
    Ok(Json(out))
}

// ------------------------------------------------------------------ context

async fn context(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<dto::Context>, AppError> {
    let id = parse_status_id(&id)?;
    let viewer = &auth.user;
    let root = visible_post_of(&state, viewer.id, id).await?;
    let ids = feeds::context(state.db(), viewer.id, id, root.author.id).await?;
    let wanted: Vec<Uuid> = ids
        .ancestors
        .iter()
        .copied()
        .chain(ids.descendants.iter().map(|(id, _)| *id))
        .collect();
    let mut loaded: HashMap<Uuid, PostDto> = posts::load(state.db(), viewer.id, &wanted, None)
        .await?
        .into_iter()
        .map(|post| (post.id, post))
        .collect();
    // Ancestors stay a chain: reading up from the post, stop at the first one the viewer cannot see.
    let mut above = Vec::new();
    for ancestor in ids.ancestors.iter().rev() {
        let Some(post) = loaded.remove(ancestor) else {
            break;
        };
        above.push(post);
    }
    above.reverse();
    // Replies stay attached: one whose parent was dropped is dropped too.
    let mut by_depth = ids.descendants;
    by_depth.sort_unstable_by_key(|(id, depth)| (*depth, *id));
    let mut kept: HashSet<Uuid> = HashSet::from([id]);
    let mut below = Vec::new();
    for (reply, _) in by_depth {
        let Some(post) = loaded.remove(&reply) else {
            continue;
        };
        if post
            .reply_to
            .as_ref()
            .is_some_and(|parent| kept.contains(&parent.id))
        {
            kept.insert(reply);
            below.push(post);
        }
    }
    below.sort_unstable_by_key(|post| post.id);
    let split = above.len();
    let mut statuses =
        render_many(&state, viewer, above.into_iter().chain(below).collect()).await?;
    let descendants = statuses.split_off(split);
    Ok(Json(dto::Context {
        ancestors: statuses,
        descendants,
    }))
}

// ------------------------------------------------------------------ actions

async fn react(
    state: AppState,
    auth: CurrentUser,
    id: &str,
    reaction: Reaction,
    add: bool,
) -> Result<Json<Status>, AppError> {
    let id = parse_status_id(id)?;
    let viewer = &auth.0.user;
    if add && reaction == Reaction::Repost {
        let post = visible_post_of(&state, viewer.id, id).await?;
        if !matches!(post.visibility, Visibility::Public | Visibility::Unlisted) {
            return Err(unprocessable(
                "Validation failed: Reblog of a non-public status is not allowed.",
            ));
        }
    }
    let mut post = Box::pin(post_actions::react(&state, viewer.id, id, reaction, add)).await?;
    if add
        && reaction == Reaction::Repost
        && let Some(entry) = activity::repost_entry(state.db(), viewer.id, id).await?
    {
        post.entry_id = entry;
        post.reposted_by = Some(user_ref(viewer));
    }
    Ok(Json(render_one(&state, viewer, post).await?))
}

async fn favourite(
    State(state): State<AppState>,
    auth: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Status>, AppError> {
    Box::pin(react(state, auth, &id, Reaction::Like, true)).await
}

async fn unfavourite(
    State(state): State<AppState>,
    auth: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Status>, AppError> {
    Box::pin(react(state, auth, &id, Reaction::Like, false)).await
}

async fn reblog(
    State(state): State<AppState>,
    auth: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Status>, AppError> {
    Box::pin(react(state, auth, &id, Reaction::Repost, true)).await
}

async fn unreblog(
    State(state): State<AppState>,
    auth: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Status>, AppError> {
    Box::pin(react(state, auth, &id, Reaction::Repost, false)).await
}

async fn bookmark(
    State(state): State<AppState>,
    auth: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Status>, AppError> {
    Box::pin(react(state, auth, &id, Reaction::Bookmark, true)).await
}

async fn unbookmark(
    State(state): State<AppState>,
    auth: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Status>, AppError> {
    Box::pin(react(state, auth, &id, Reaction::Bookmark, false)).await
}

async fn pin(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Status>, AppError> {
    let id = parse_status_id(&id)?;
    let viewer = &auth.user;
    let post = visible_post_of(&state, viewer.id, id).await?;
    if post.author.id != viewer.id {
        return Err(unprocessable(
            "Validation failed: Sorry, you can only pin your own statuses.",
        ));
    }
    post_actions::pin(state.db(), viewer.id, id).await?;
    Ok(Json(render_one(&state, viewer, post).await?))
}

async fn unpin(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<Status>, AppError> {
    let id = parse_status_id(&id)?;
    let viewer = &auth.user;
    let post = visible_post_of(&state, viewer.id, id).await?;
    if post.author.id == viewer.id
        && activity::pinned_post(state.db(), viewer.id).await? == Some(id)
    {
        post_actions::unpin(state.db(), viewer.id).await?;
    }
    Ok(Json(render_one(&state, viewer, post).await?))
}

// ------------------------------------------------------------------ who reacted

async fn reactors(
    state: AppState,
    auth: CurrentUser,
    id: &str,
    paging: Paging<String>,
    table: ReactorTable,
) -> Result<Response, AppError> {
    let id = parse_status_id(id)?;
    let viewer = &auth.0.user;
    visible_post_of(&state, viewer.id, id).await?;
    let rows = feeds::reactors(
        state.db(),
        viewer.id,
        id,
        table,
        edge_range(&paging)?,
        paging.fetch(),
    )
    .await?;
    let slice = paging.trim(rows);
    let cursor = |row: &(UserRow, EdgeCursor)| row.1.encode();
    let link = paging.link(
        state.config(),
        slice.rows.first().map(cursor),
        slice.rows.last().map(cursor),
        slice.more,
    );
    let accounts: Vec<UserRow> = slice.rows.into_iter().map(|(user, _)| user).collect();
    let cards = users::dtos(state.db(), viewer.id, &accounts).await?;
    let ids: Vec<Uuid> = accounts.iter().map(|user| user.id).collect();
    let ctx = Ctx::for_users(&state, &ids).await?;
    let body: Vec<Account> = cards.iter().map(|card| dto::account(card, &ctx)).collect();
    Ok(respond(link, &body))
}

async fn favourited_by(
    State(state): State<AppState>,
    auth: CurrentUser,
    Path(id): Path<String>,
    paging: Paging<String>,
) -> Result<Response, AppError> {
    reactors(state, auth, &id, paging, ReactorTable::Likes).await
}

async fn reblogged_by(
    State(state): State<AppState>,
    auth: CurrentUser,
    Path(id): Path<String>,
    paging: Paging<String>,
) -> Result<Response, AppError> {
    reactors(state, auth, &id, paging, ReactorTable::Reposts).await
}

// ------------------------------------------------------------------ scheduled statuses

#[derive(Serialize)]
struct ScheduledPoll {
    options: Vec<String>,
    expires_in: i64,
    multiple: bool,
    hide_totals: bool,
}

#[derive(Serialize)]
struct ScheduledParams {
    text: String,
    poll: Option<ScheduledPoll>,
    media_ids: Option<Vec<Uuid>>,
    sensitive: Option<bool>,
    spoiler_text: Option<String>,
    visibility: &'static str,
    in_reply_to_id: Option<Uuid>,
    language: Option<String>,
    application_id: i64,
    scheduled_at: Option<String>,
    idempotency: Option<String>,
    with_rate_limit: bool,
}

#[derive(Serialize)]
struct ScheduledJson {
    id: Uuid,
    scheduled_at: String,
    params: ScheduledParams,
    media_attachments: Vec<MediaAttachment>,
}

fn scheduled_json(state: &AppState, entry: &ScheduledPostDto) -> ScheduledJson {
    let ctx = Ctx::new(state.config());
    ScheduledJson {
        id: entry.id,
        scheduled_at: dto::iso(entry.publish_at),
        params: ScheduledParams {
            text: entry.text.clone(),
            poll: entry.poll.as_ref().map(|poll| ScheduledPoll {
                options: poll.options.clone(),
                expires_in: i64::from(poll.duration_hours) * SECS_PER_HOUR,
                multiple: poll.multiple,
                hide_totals: false,
            }),
            media_ids: Some(entry.media.iter().map(|media| media.id).collect()),
            sensitive: Some(entry.sensitive),
            spoiler_text: Some(entry.cw.clone()),
            visibility: "public",
            in_reply_to_id: None,
            language: None,
            application_id: 0,
            scheduled_at: None,
            idempotency: None,
            with_rate_limit: false,
        },
        media_attachments: entry
            .media
            .iter()
            .map(|media| dto::media_attachment(media, &ctx))
            .collect(),
    }
}

async fn scheduled_entry(
    state: &AppState,
    viewer: &UserRow,
    id: Uuid,
) -> Result<ScheduledPostDto, AppError> {
    scheduled::list(state.db(), viewer.id)
        .await?
        .into_iter()
        .find(|entry| entry.id == id && entry.status == ScheduledStatus::Pending)
        .ok_or(AppError::Missing("Record not found."))
}

fn scheduled_id(raw: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(raw).map_err(|_| AppError::Missing("Record not found."))
}

async fn scheduled_list(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
) -> Result<Json<Vec<ScheduledJson>>, AppError> {
    let entries = scheduled::list(state.db(), auth.user.id).await?;
    Ok(Json(
        entries
            .iter()
            .filter(|entry| entry.status == ScheduledStatus::Pending)
            .map(|entry| scheduled_json(&state, entry))
            .collect(),
    ))
}

async fn scheduled_one(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<ScheduledJson>, AppError> {
    let entry = scheduled_entry(&state, &auth.user, scheduled_id(&id)?).await?;
    Ok(Json(scheduled_json(&state, &entry)))
}

async fn scheduled_move(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    Form { params, .. }: Form,
) -> Result<Json<ScheduledJson>, AppError> {
    let id = scheduled_id(&id)?;
    let at = params
        .get("scheduled_at")
        .and_then(|raw| OffsetDateTime::parse(raw.trim(), &Rfc3339).ok())
        .ok_or_else(|| unprocessable("Validation failed: Scheduled at is invalid."))?;
    let entry = scheduled::reschedule(state.db(), auth.user.id, id, at)
        .await
        .map_err(invalid)?;
    Ok(Json(scheduled_json(&state, &entry)))
}

async fn scheduled_remove(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    scheduled::delete(state.db(), auth.user.id, scheduled_id(&id)?).await?;
    Ok(Json(serde_json::json!({})))
}
