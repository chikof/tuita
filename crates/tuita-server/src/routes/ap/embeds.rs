//! Link-preview endpoints: `GET /oembed` and the Mastodon status JSON at
//! `GET /api/v1/statuses/{id}`: anonymous visitors get the public view; a signed-in request gets
//! the Mastodon status with its viewer state ([`crate::mastodon::statuses::show`]).

use axum::extract::{Path, State};
use axum::http::header::{ACCESS_CONTROL_ALLOW_ORIGIN, CACHE_CONTROL};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use tuita_core::ap::{escape_html, same_authority};
use url::Url;
use uuid::Uuid;

use crate::auth::{ApiQuery, MaybeUser};
use crate::config::Config;
use crate::db::settings;
use crate::dto::Visibility;
use crate::error::AppError;
use crate::federation::{ids, mastodon, render};
use crate::state::AppState;

/// Widest embed, in pixels.
const MAX_WIDTH: u32 = 550;
/// Narrowest embed, in pixels.
const MIN_WIDTH: u32 = 200;
/// How long consumers may keep an oEmbed answer, in seconds.
const OEMBED_CACHE_SECS: u32 = 180;

pub(super) fn oembed_router() -> Router<AppState> {
    Router::new().route("/oembed", get(oembed))
}

pub(super) fn status_router() -> Router<AppState> {
    Router::new().route("/statuses/{id}", get(status))
}

#[derive(Deserialize)]
struct OembedQuery {
    url: Option<String>,
    maxwidth: Option<u32>,
    #[serde(rename = "maxheight")]
    max_height: Option<u32>,
}

/// The handle and post id of a post URL of this instance: `/@{handle}/{id}` or
/// `/users/{handle}/statuses/{id}`.
fn post_of_url(config: &Config, url: &str) -> Option<(String, Uuid)> {
    if !same_authority(url, &config.public_url) {
        return None;
    }
    let parsed = Url::parse(url).ok()?;
    let segments: Vec<&str> = parsed.path_segments()?.collect();
    match segments.as_slice() {
        [handle, id] => Some((
            handle.strip_prefix('@')?.to_owned(),
            Uuid::parse_str(id).ok()?,
        )),
        ["users", handle, "statuses", id] => {
            Some(((*handle).to_owned(), Uuid::parse_str(id).ok()?))
        }
        _ => None,
    }
}

async fn oembed(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<OembedQuery>,
) -> Result<Response, AppError> {
    let config = state.config();
    let url = query
        .url
        .as_deref()
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .ok_or_else(|| AppError::bad_request("url is required."))?;
    let (handle, id) = post_of_url(config, url).ok_or(AppError::Missing("Not found."))?;
    let post = render::load(&state, id)
        .await?
        .filter(|post| {
            post.is_open()
                && post.visibility == Visibility::Public
                && post.author.handle.eq_ignore_ascii_case(&handle)
        })
        .ok_or(AppError::Missing("Not found."))?;
    let site = settings::load(state.db()).await?.name;
    let page = ids::note_url(config, &post.author.handle, post.id);
    let author = &post.author;
    let shown = if post.cw.is_empty() {
        post.text.clone()
    } else {
        format!("CW: {}", post.cw)
    };
    let html = format!(
        "<blockquote class=\"tuita-embed\" cite=\"{page}\"><p>{body}</p>&mdash; {name} (@{handle}) \
         <a href=\"{page}\">{date}</a></blockquote>",
        page = escape_html(&page),
        body = escape_html(&shown).replace('\n', "<br>"),
        name = escape_html(&author.display_name),
        handle = escape_html(&author.handle),
        date = post.created_at.date(),
    );
    let body = json!({
        "version": "1.0",
        "type": "rich",
        "provider_name": site,
        "provider_url": config.public_url,
        "author_name": format!("{} (@{})", author.display_name, author.handle),
        "author_url": ids::profile_url(config, &author.handle),
        "html": html,
        "width": query.maxwidth.map_or(MAX_WIDTH, |width| width.clamp(MIN_WIDTH, MAX_WIDTH)),
        "height": query.max_height,
        "cache_age": OEMBED_CACHE_SECS,
    });
    Ok((
        [
            (ACCESS_CONTROL_ALLOW_ORIGIN, "*".to_owned()),
            (
                CACHE_CONTROL,
                format!("public, max-age={OEMBED_CACHE_SECS}"),
            ),
        ],
        Json(body),
    )
        .into_response())
}

async fn status(
    State(state): State<AppState>,
    MaybeUser(viewer): MaybeUser,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let id = Uuid::parse_str(&id).map_err(|_| AppError::Missing("Post not found."))?;
    if let Some(auth) = viewer {
        return crate::mastodon::statuses::show(&state, &auth.user, id).await;
    }
    let status = mastodon::status(&state, id)
        .await?
        .ok_or(AppError::Missing("Post not found."))?;
    Ok((
        [
            (ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
            (CACHE_CONTROL, "public, max-age=60"),
        ],
        Json(status),
    )
        .into_response())
}
