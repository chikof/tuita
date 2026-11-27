//! `/.well-known/webfinger`, `/.well-known/host-meta`, `/.well-known/nodeinfo`, `/nodeinfo/2.0`
//! and `/nodeinfo/2.1`.
//!
//! Browser clients (Mastodon web apps) read these from another origin before they sign in, so
//! the routes sit behind the CORS layer of the Mastodon API: any origin, no credentials, and
//! `OPTIONS` preflights answered by the layer.

use axum::Router;
use axum::extract::State;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::Deserialize;
use serde_json::json;
use tuita_core::ap::escape_html;

use super::{CACHE_PUBLIC, live_user};
use crate::auth::ApiQuery;
use crate::config::Config;
use crate::db::{ap::usage, settings};
use crate::dto::SignupsMode;
use crate::error::AppError;
use crate::federation::ids;
use crate::mastodon;
use crate::state::AppState;

const JRD: &str = "application/jrd+json; charset=utf-8";
const XRD: &str = "application/xrd+xml; charset=utf-8";
const NODEINFO_REL_PREFIX: &str = "http://nodeinfo.diaspora.software/ns/schema/";
const PROFILE_PAGE_REL: &str = "http://webfinger.net/rel/profile-page";
const MISSING_RESOURCE: &str = "Missing or invalid resource.";

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/.well-known/webfinger", get(webfinger))
        .route("/.well-known/host-meta", get(host_meta))
        .route("/.well-known/nodeinfo", get(nodeinfo_links))
        .route(
            "/nodeinfo/2.0",
            get(|state: State<AppState>| nodeinfo(state, "2.0")),
        )
        .route(
            "/nodeinfo/2.1",
            get(|state: State<AppState>| nodeinfo(state, "2.1")),
        )
        .layer(mastodon::cors())
}

#[derive(Deserialize)]
struct WebfingerQuery {
    resource: Option<String>,
}

/// What a WebFinger `resource` names.
enum Target {
    /// A local account.
    Account(String),
    /// The instance actor.
    Instance,
}

/// The account or instance actor `resource` names: `acct:{handle}@{domain}`, `acct:{domain}@{domain}`
/// or an actor URL of this instance.
///
/// # Errors
///
/// `400` for anything that is not shaped like one of those; `404` for another domain.
fn target(config: &Config, resource: &str) -> Result<Target, AppError> {
    if let Some(acct) = resource.strip_prefix("acct:") {
        let acct = acct.strip_prefix('@').unwrap_or(acct);
        let (name, domain) = acct
            .rsplit_once('@')
            .filter(|(name, domain)| !name.is_empty() && !domain.is_empty())
            .ok_or_else(|| AppError::bad_request(MISSING_RESOURCE))?;
        if !domain.eq_ignore_ascii_case(config.domain()) {
            return Err(AppError::NotFound);
        }
        return Ok(if name.eq_ignore_ascii_case(domain) {
            Target::Instance
        } else {
            Target::Account(name.to_owned())
        });
    }
    if resource == ids::instance_actor_id(config) {
        return Ok(Target::Instance);
    }
    ids::local_handle_from_actor_id(config, resource)
        .map(|handle| Target::Account(handle.to_owned()))
        .ok_or_else(|| AppError::bad_request(MISSING_RESOURCE))
}

fn jrd(body: &serde_json::Value) -> Response {
    (
        [(CONTENT_TYPE, JRD)],
        [(CACHE_CONTROL, CACHE_PUBLIC)],
        serde_json::to_vec(body).unwrap_or_default(),
    )
        .into_response()
}

async fn webfinger(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<WebfingerQuery>,
) -> Result<Response, AppError> {
    let config = state.config();
    let resource = query
        .resource
        .as_deref()
        .map(str::trim)
        .filter(|resource| !resource.is_empty())
        .ok_or_else(|| AppError::bad_request(MISSING_RESOURCE))?;
    let body = match target(config, resource)? {
        Target::Instance => {
            let actor = ids::instance_actor_id(config);
            json!({
                "subject": format!("acct:{0}@{0}", config.domain()),
                "aliases": [actor],
                "links": [
                    {"rel": PROFILE_PAGE_REL, "type": "text/html", "href": config.public_url},
                    {"rel": "self", "type": "application/activity+json", "href": actor},
                ],
            })
        }
        Target::Account(handle) => {
            let user = live_user(&state, &handle).await?;
            let actor = ids::actor_id(config, &user.handle);
            let profile = ids::profile_url(config, &user.handle);
            json!({
                "subject": format!("acct:{}@{}", user.handle, config.domain()),
                "aliases": [profile, actor],
                "links": [
                    {"rel": PROFILE_PAGE_REL, "type": "text/html", "href": profile},
                    {"rel": "self", "type": "application/activity+json", "href": actor},
                ],
            })
        }
    };
    Ok(jrd(&body))
}

async fn host_meta(State(state): State<AppState>) -> Response {
    let template = format!(
        "{}/.well-known/webfinger?resource={{uri}}",
        state.config().public_url
    );
    let body = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <XRD xmlns=\"http://docs.oasis-open.org/ns/xri/xrd-1.0\">\
         <Link rel=\"lrdd\" template=\"{}\"/></XRD>\n",
        escape_html(&template)
    );
    ([(CONTENT_TYPE, XRD)], [(CACHE_CONTROL, CACHE_PUBLIC)], body).into_response()
}

async fn nodeinfo_links(State(state): State<AppState>) -> Response {
    let public_url = &state.config().public_url;
    let links: Vec<_> = ["2.0", "2.1"]
        .into_iter()
        .map(|version| {
            json!({
                "rel": format!("{NODEINFO_REL_PREFIX}{version}"),
                "href": format!("{public_url}/nodeinfo/{version}"),
            })
        })
        .collect();
    (
        [(CONTENT_TYPE, "application/json")],
        [(CACHE_CONTROL, CACHE_PUBLIC)],
        serde_json::to_vec(&json!({ "links": links })).unwrap_or_default(),
    )
        .into_response()
}

/// The NodeInfo document of schema `version` (`2.0` or `2.1`). Only 2.1 has `software.repository`.
async fn nodeinfo(State(state): State<AppState>, version: &str) -> Result<Response, AppError> {
    let settings = settings::load(state.db()).await?;
    let (users, posts) = usage(state.db()).await?;
    let mut software = json!({"name": "tuita", "version": env!("CARGO_PKG_VERSION")});
    if version == "2.1"
        && let Some(source) = state.config().source_url.as_deref()
    {
        software["repository"] = json!(source);
    }
    let body = json!({
        "version": version,
        "software": software,
        "protocols": ["activitypub"],
        "services": {"inbound": [], "outbound": []},
        "openRegistrations": settings.signups == SignupsMode::Open,
        "usage": {"users": {"total": users}, "localPosts": posts},
        "metadata": {"nodeName": settings.name, "nodeDescription": settings.description},
    });
    let profile = format!("application/json; profile=\"{NODEINFO_REL_PREFIX}{version}#\"");
    Ok((
        [(CONTENT_TYPE, profile)],
        [(CACHE_CONTROL, CACHE_PUBLIC)],
        serde_json::to_vec(&body).unwrap_or_default(),
    )
        .into_response())
}
