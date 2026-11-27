//! Remote lookup: `user@domain` to a stored account and a post URL to a stored post.
//!
//! An unknown account goes WebFinger, actor fetch, upsert, first page of its outbox. A known
//! account is served from the database; when its profile is older than 24 hours it is returned
//! as it is and refreshed in the background. Nothing here trusts a remote claim that it can
//! check: the actor id must be on the host that served it and on the domain the address names,
//! the key must belong to that actor, and a note's id must be on its author's host.

use std::collections::HashSet;
use std::sync::{LazyLock, Mutex, PoisonError};
use std::time::Duration;

use futures_util::FutureExt as _;
use futures_util::future::BoxFuture;
use serde::Deserialize as _;
use serde_json::Value;
use time::OffsetDateTime;
use tuita_core::ap::{
    self, Activity, ActivityKind, Actor, ActorKind, Collection, Image, Note, PublicKey,
};
use url::Url;
use uuid::Uuid;

use super::FedError;
use super::fetch::{self, Document};
use super::ingest::{self, Context, Stored, cap, internal};
use super::media::{copy_avatar, copy_banner};
use crate::config::Config;
use crate::db::remote::{self, MediaSlot, RemoteProfile, Upsert};
use crate::db::users::{self, UserRow, find_local};
use crate::error::AppError;
use crate::net::{Fetch, NetError, Overflow};
use crate::state::AppState;

/// A stored profile older than this is refreshed when it is looked up.
pub const STALE_AFTER: time::Duration = time::Duration::hours(24);
/// Most posts the first page of an outbox adds.
pub const BACKFILL_LIMIT: usize = 20;
/// Longest the backfill of one account may take, all fetches together.
const BACKFILL_BUDGET: Duration = Duration::from_secs(20);
/// Largest WebFinger answer read.
const JRD_MAX_BYTES: usize = 128 * 1024;
/// Longest WebFinger request.
const JRD_TIMEOUT: Duration = Duration::from_secs(10);
/// Longest remote handle, in characters.
const MAX_HANDLE_CHARS: usize = 64;
/// Longest domain.
const MAX_DOMAIN_LEN: usize = 255;
/// Longest display name kept, in characters.
const MAX_NAME_CHARS: usize = 100;
/// Longest bio kept, in characters.
const MAX_BIO_CHARS: usize = 500;

/// A handle as a route or a search writes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Handle {
    /// An account of this instance: `ada`, `@ada`, or either with this instance's domain.
    Local(String),
    /// An account of another instance: `ada@example.org`, with or without `@`.
    Remote {
        /// The handle.
        handle: String,
        /// The normalised domain: lower case, `host` or `host:port`.
        domain: String,
    },
}

/// Reads a handle from a `{handle}` path segment, a search query or a mention. `None` for
/// anything malformed.
///
/// ```
/// use tuita_server::federation::lookup::{Handle, parse_handle};
///
/// assert_eq!(parse_handle("@ada", "me.example"), Some(Handle::Local("ada".into())));
/// assert_eq!(parse_handle("ada@ME.example", "me.example"), Some(Handle::Local("ada".into())));
/// assert_eq!(
///     parse_handle("@bob@Social.Example", "me.example"),
///     Some(Handle::Remote { handle: "bob".into(), domain: "social.example".into() })
/// );
/// assert_eq!(parse_handle("a@b@c", "me.example"), None);
/// assert_eq!(parse_handle("bob@bad domain", "me.example"), None);
/// ```
#[must_use]
pub fn parse_handle(raw: &str, own_domain: &str) -> Option<Handle> {
    let raw = raw.strip_prefix('@').unwrap_or(raw);
    let Some((handle, domain)) = raw.split_once('@') else {
        return Some(Handle::Local(raw.to_owned()));
    };
    if handle.is_empty() || domain.contains('@') {
        return None;
    }
    let domain = normalize_domain(domain)?;
    if domain.eq_ignore_ascii_case(own_domain) {
        return Some(Handle::Local(handle.to_owned()));
    }
    valid_remote_handle(handle).then(|| Handle::Remote {
        handle: handle.to_owned(),
        domain,
    })
}

/// A remote handle: 1 to 64 of `A-Z a-z 0-9 _ . -`.
#[must_use]
pub fn valid_remote_handle(handle: &str) -> bool {
    !handle.is_empty()
        && handle.chars().count() <= MAX_HANDLE_CHARS
        && handle
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

/// `host` or `host:port` in lower case, or `None` when `raw` is not one. The default ports are
/// dropped, and IPv6 literals are refused.
fn normalize_domain(raw: &str) -> Option<String> {
    if raw.is_empty()
        || raw.len() > MAX_DOMAIN_LEN
        || !raw
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b':' | b'_'))
    {
        return None;
    }
    let url = Url::parse(&format!("http://{raw}/")).ok()?;
    if !matches!(url.path(), "/") || url.query().is_some() {
        return None;
    }
    let host = url.host_str()?.trim_end_matches('.').to_ascii_lowercase();
    if host.is_empty() {
        return None;
    }
    // The scheme is not known yet: neither default port is kept.
    let explicit = raw
        .rsplit_once(':')
        .and_then(|(_, port)| port.parse::<u16>().ok());
    Some(match explicit {
        Some(80 | 443) | None => host,
        Some(port) => format!("{host}:{port}"),
    })
}

fn map_net(err: &NetError, domain: &str) -> FedError {
    match err {
        NetError::Status(404 | 410) => FedError::NotFound,
        NetError::TooLarge => FedError::Unreadable(domain.into()),
        _ => FedError::Unreachable(domain.into()),
    }
}

fn limited(err: AppError) -> FedError {
    match err {
        AppError::TooManyRequests { retry_after_secs } => {
            FedError::RateLimited { retry_after_secs }
        }
        other => internal(other),
    }
}

// ------------------------------------------------------------------ WebFinger

/// The schemes to try for a WebFinger request, in order. Plain http only when the instance allows
/// it, and then after the one its own address uses.
fn schemes(config: &Config) -> &'static [&'static str] {
    if !config.federation_allow_http {
        &["https"]
    } else if config.public_url.starts_with("http://") {
        &["http", "https"]
    } else {
        &["https", "http"]
    }
}

/// The actor URL that `acct:{handle}@{domain}` names, through WebFinger.
///
/// The JRD is read up to 128 KiB. The `self` link must be `application/activity+json` (or
/// `ld+json`) and an `https` URL, or `http` when the instance allows plain http.
///
/// # Errors
///
/// [`FedError::NotFound`] for `404`, `410` or a JRD without an actor link;
/// [`FedError::Unreachable`] for transport failures, timeouts and refused addresses;
/// [`FedError::Unreadable`] for an answer that is not a JRD.
pub async fn webfinger(state: &AppState, handle: &str, domain: &str) -> Result<String, FedError> {
    let mut last = FedError::Unreachable(domain.into());
    for scheme in schemes(state.config()) {
        match webfinger_over(state, scheme, handle, domain).await {
            Ok(href) => return Ok(href),
            Err(err @ FedError::Unreachable(_)) => last = err,
            Err(err) => return Err(err),
        }
    }
    Err(last)
}

async fn webfinger_over(
    state: &AppState,
    scheme: &str,
    handle: &str,
    domain: &str,
) -> Result<String, FedError> {
    let url = Url::parse_with_params(
        &format!("{scheme}://{domain}/.well-known/webfinger"),
        &[("resource", format!("acct:{handle}@{domain}"))],
    )
    .map_err(|_| FedError::NotFound)?;
    let fetched = state
        .safe_client()
        .get_with(
            url.as_str(),
            Fetch {
                accept: "application/jrd+json, application/json",
                max_bytes: JRD_MAX_BYTES,
                timeout: JRD_TIMEOUT,
                overflow: Overflow::Fail,
            },
            &[],
        )
        .await
        .map_err(|err| map_net(&err, domain))?;
    let jrd = ap::parse_document(&fetched.body, JRD_MAX_BYTES)
        .map_err(|_| FedError::Unreadable(domain.into()))?;
    actor_link(&jrd, state.config().federation_allow_http).ok_or(FedError::NotFound)
}

/// The `self` link of a JRD that points at an actor.
fn actor_link(jrd: &Value, allow_http: bool) -> Option<String> {
    jrd.get("links")?
        .as_array()?
        .iter()
        .filter(|link| link.get("rel").and_then(Value::as_str) == Some("self"))
        .filter(|link| {
            link.get("type")
                .and_then(Value::as_str)
                .is_some_and(|kind| {
                    kind.starts_with("application/activity+json")
                        || kind.starts_with("application/ld+json")
                })
        })
        .filter_map(|link| link.get("href")?.as_str())
        .find(|href| ap::is_http_url(href) && (allow_http || href.starts_with("https://")))
        .map(str::to_owned)
}

// ------------------------------------------------------------------ actors

fn actor_of(doc: &Document) -> Result<Actor, FedError> {
    Actor::deserialize(&doc.json).map_err(|_| FedError::Unreadable(server_of(&doc.url)))
}

fn server_of(url: &str) -> Box<str> {
    ap::url_authority(url)
        .unwrap_or_else(|| "that server".to_owned())
        .into()
}

/// Checks that `actor`, served from `origin`, may become a remote account here, and returns it.
///
/// - Its `id` is an http(s) URL on the host that served it, and not an id of this instance.
/// - Its domain is the host of the id; with `expect` the `preferredUsername` and domain match.
/// - It is a person, service, organization or group. An `Application` (the instance actor) is
///   [`FedError::NotFound`]: it never becomes an account.
/// - It has an inbox URL on its own instance, and a `publicKey` whose owner is the actor itself
///   and whose PEM is an RSA key. The inbox, shared inbox, outbox, followers and featured URLs
///   are normalised, and the optional ones dropped unless they are on the actor's instance.
///
/// # Errors
///
/// [`FedError::Unreadable`] naming the server for any broken rule; [`FedError::NotFound`] for
/// an application actor.
pub fn check_actor(
    state: &AppState,
    mut actor: Actor,
    origin: &str,
    expect: Option<(&str, &str)>,
) -> Result<Actor, FedError> {
    let server = server_of(origin);
    let config = state.config();
    let domain =
        ap::url_authority(&actor.id).ok_or_else(|| FedError::Unreadable(server.clone()))?;
    let bound = ap::same_authority(&actor.id, origin)
        && !domain.eq_ignore_ascii_case(config.domain())
        && super::ids::local_handle_from_actor_id(config, &actor.id).is_none();
    if !bound {
        return Err(FedError::Unreadable(server));
    }
    if actor.kind == ActorKind::Application {
        return Err(FedError::NotFound);
    }
    if !valid_remote_handle(&actor.preferred_username) {
        return Err(FedError::Unreadable(server));
    }
    // Every endpoint is fetched from or posted to later, so each one is on the actor's own
    // instance and stored in its normalised form. A foreign `inbox` refuses the actor; a foreign
    // optional endpoint is dropped.
    let id = actor.id.clone();
    let Some(inbox) = ap::normalize_url_of(&id, &actor.inbox) else {
        return Err(FedError::Unreadable(server));
    };
    actor.inbox = inbox;
    let own = |url: &Option<String>| {
        url.as_deref()
            .and_then(|url| ap::normalize_url_of(&id, url))
    };
    actor.outbox = own(&actor.outbox);
    actor.followers = own(&actor.followers);
    actor.featured = own(&actor.featured);
    if let Some(endpoints) = actor.endpoints.as_mut() {
        endpoints.shared_inbox = own(&endpoints.shared_inbox);
    }
    if let Some((handle, expected_domain)) = expect
        && (!actor.preferred_username.eq_ignore_ascii_case(handle)
            || !domain.eq_ignore_ascii_case(expected_domain))
    {
        return Err(FedError::Unreadable(server));
    }
    let key_ok = actor.public_key.as_ref().is_some_and(|key| {
        key.owner == actor.id
            && (key.id.is_empty() || ap::same_authority(&key.id, &actor.id))
            && PublicKey::from_pem(&key.public_key_pem).is_ok()
    });
    if key_ok {
        Ok(actor)
    } else {
        Err(FedError::Unreadable(server))
    }
}

fn property<'a>(actor: &'a Actor, name: &str) -> Option<&'a str> {
    actor
        .attachment
        .iter()
        .find(|field| field.name.trim().eq_ignore_ascii_case(name))
        .map(|field| field.value.as_str())
}

fn http_only(url: Option<&String>) -> Option<String> {
    url.filter(|url| ap::is_http_url(url)).cloned()
}

/// The row to store for a checked `actor`.
fn profile_of(actor: &Actor, now: OffsetDateTime) -> Option<RemoteProfile> {
    let domain = ap::url_authority(&actor.id)?;
    let name: String = actor
        .name
        .as_deref()
        .unwrap_or("")
        .chars()
        .filter(|c| !c.is_control())
        .collect();
    let name = cap(name.trim(), MAX_NAME_CHARS);
    let website = property(actor, "website")
        .map(|value| ap::html_to_text(value).trim().to_owned())
        .filter(|value| ap::is_http_url(value))
        .unwrap_or_default();
    Some(RemoteProfile {
        ap_id: actor.id.clone(),
        handle: actor.preferred_username.clone(),
        display_name: if name.is_empty() {
            actor.preferred_username.clone()
        } else {
            name
        },
        domain,
        bio: actor
            .summary
            .as_deref()
            .map(|html| cap(ap::html_to_text(html).trim(), MAX_BIO_CHARS))
            .unwrap_or_default(),
        website,
        is_bot: actor.kind == ActorKind::Service,
        is_org: actor.kind == ActorKind::Organization,
        locked: actor.manually_approves_followers,
        discoverable: actor.discoverable.unwrap_or(false),
        inbox_url: actor.inbox.clone(),
        shared_inbox_url: http_only(
            actor
                .endpoints
                .as_ref()
                .and_then(|endpoints| endpoints.shared_inbox.as_ref()),
        ),
        outbox_url: http_only(actor.outbox.as_ref()),
        followers_url: http_only(actor.followers.as_ref()),
        featured_url: http_only(actor.featured.as_ref()),
        url: http_only(actor.url.as_ref()),
        public_key_pem: actor
            .public_key
            .as_ref()
            .map(|key| key.public_key_pem.clone())
            .unwrap_or_default(),
        created_at: actor
            .published
            .as_deref()
            .and_then(ap::parse_time)
            .map(|at| at.min(now)),
    })
}

/// Stores a checked `actor` as a remote account and copies its avatar and banner. Returns the
/// stored row, or `None` when this server deleted the account or blocks its instance.
///
/// An image is copied only when its URL differs from the one the stored copy came from, so a
/// profile refresh of an unchanged actor downloads nothing.
///
/// # Errors
///
/// [`FedError::Unreadable`] when another account holds the handle and cannot be replaced;
/// [`FedError::Internal`] on database failures.
pub async fn store_actor(state: &AppState, actor: &Actor) -> Result<Option<UserRow>, FedError> {
    let profile = profile_of(actor, OffsetDateTime::now_utc())
        .ok_or_else(|| FedError::Unreadable(server_of(&actor.id)))?;
    let db = state.db();
    if profile.domain.eq_ignore_ascii_case(state.config().domain()) {
        return Err(FedError::Unreadable(profile.domain.into()));
    }
    if remote::domain_suspended(db, &profile.domain)
        .await
        .map_err(internal)?
    {
        return Ok(None);
    }
    let id = match remote::upsert_actor(db, &profile).await.map_err(internal)? {
        Upsert::Stored(id) => id,
        Upsert::Gone => return Ok(None),
        Upsert::Conflict => return Err(FedError::Unreadable(profile.domain.into())),
    };
    let (avatar_src, banner_src) = remote::image_sources(db, id).await.map_err(internal)?;
    let avatar = image_slot(
        actor.icon.as_ref(),
        avatar_src.as_deref(),
        |url| async move { Box::pin(copy_avatar(state, id, &url)).await },
    );
    let banner = image_slot(
        actor.image.as_ref(),
        banner_src.as_deref(),
        |url| async move { Box::pin(copy_banner(state, id, &url)).await },
    );
    let texts = [profile.display_name.as_str(), profile.bio.as_str()];
    let emotes = Box::pin(ingest::copy_emotes(
        state,
        id,
        Some(&profile.domain),
        actor.tag.as_slice(),
        &texts,
    ));
    let (avatar, banner, ()) = tokio::join!(avatar, banner, emotes);
    remote::set_images(db, id, avatar, banner)
        .await
        .map_err(internal)?;
    users::by_id(db, id).await.map_err(internal)
}

/// What to do with one image column: nothing when the actor still names the URL the copy came
/// from, a fresh copy for a new URL (the old one stays if that fails), and a cleared column for
/// an actor without the image.
async fn image_slot<F, Fut>(image: Option<&Image>, copied_from: Option<&str>, copy: F) -> MediaSlot
where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = Option<Uuid>>,
{
    let Some(image) = image.filter(|image| ap::is_http_url(&image.url)) else {
        return MediaSlot::Clear;
    };
    if copied_from == Some(image.url.as_str()) {
        return MediaSlot::Keep;
    }
    match copy(image.url.clone()).await {
        Some(id) => MediaSlot::Set {
            id,
            src: image.url.clone(),
        },
        None => MediaSlot::Keep,
    }
}

/// Fetches the actor at `url` and checks it. Nothing is stored and no image is fetched: the
/// inbox verifies a signature against this actor's key before it may become an account.
async fn fetch_checked(
    state: &AppState,
    url: &str,
    expect: Option<(&str, &str)>,
) -> Result<Actor, FedError> {
    let doc = fetch::fetch_document(state, url).await?;
    check_actor(state, actor_of(&doc)?, &doc.url, expect)
}

/// `url` is an actor URL of another instance this server does not block. The network is not
/// touched.
async fn guard_actor_url(state: &AppState, url: &str) -> Result<(), FedError> {
    if !ap::is_http_url(url) {
        return Err(FedError::Unreadable(server_of(url)));
    }
    let own = state.config().domain();
    let domain = ap::url_authority(url).ok_or(FedError::NotFound)?;
    if domain.eq_ignore_ascii_case(own)
        || remote::domain_suspended(state.db(), &domain)
            .await
            .map_err(internal)?
    {
        return Err(FedError::NotFound);
    }
    Ok(())
}

/// The stored remote account with actor id `url`, usable and not fetched. `None` when this
/// server does not know the actor.
///
/// # Errors
///
/// [`FedError::NotFound`] for a deleted or suspended account or a `suspend`-blocked instance;
/// [`FedError::Unreadable`] for a URL that is not http(s).
pub async fn known_actor(state: &AppState, url: &str) -> Result<Option<UserRow>, FedError> {
    guard_actor_url(state, url).await?;
    remote::find_by_ap_id(state.db(), url)
        .await
        .map_err(internal)?
        .map(|row| usable(state, row))
        .transpose()
}

/// Fetches and checks the actor at `url` without storing it: the inbox verifies a signature with
/// its key first and calls [`adopt`] only when that holds.
///
/// # Errors
///
/// As [`actor_by_url`].
pub async fn fetch_candidate(state: &AppState, url: &str) -> Result<Actor, FedError> {
    guard_actor_url(state, url).await?;
    fetch_checked(state, url, None).await
}

/// Fetches the actor of `row` again without storing it, for a signature that fails against the
/// stored key. The actor must still be the same one.
///
/// # Errors
///
/// As [`actor_by_url`]; [`FedError::Unreadable`] when the document names another actor.
pub async fn fetch_current(state: &AppState, row: &UserRow) -> Result<Actor, FedError> {
    let (Some(ap_id), Some(domain)) = (row.ap_id.as_deref(), row.domain.as_deref()) else {
        return Err(FedError::NotFound);
    };
    let actor = fetch_checked(state, ap_id, Some((&row.handle, domain))).await?;
    if actor.id == ap_id {
        Ok(actor)
    } else {
        Err(FedError::Unreadable(domain.into()))
    }
}

/// Stores a checked `actor` (see [`store_actor`]) and returns the usable account.
///
/// # Errors
///
/// [`FedError::NotFound`] when this server deleted the account or blocks its instance; the
/// errors of [`store_actor`].
pub async fn adopt(state: &AppState, actor: &Actor) -> Result<UserRow, FedError> {
    let row = store_actor(state, actor).await?.ok_or(FedError::NotFound)?;
    usable(state, row)
}

/// The stored remote account with actor id `url`; an unknown one is fetched, checked and
/// stored first. Not for the inbox, which verifies before it stores: see [`fetch_candidate`].
///
/// # Errors
///
/// [`FedError::NotFound`] for an unknown, deleted, suspended or blocked account, an application
/// actor, and a `suspend`-blocked instance (without touching the network); the fetch errors of
/// [`fetch::fetch_document`]; [`FedError::Unreadable`] for an actor that breaks a rule of
/// [`check_actor`].
pub async fn actor_by_url(state: &AppState, url: &str) -> Result<UserRow, FedError> {
    if let Some(row) = known_actor(state, url).await? {
        return Ok(row);
    }
    let actor = fetch_checked(state, url, None).await?;
    adopt(state, &actor).await
}

/// `row` unless the account is deleted or suspended; stale rows are refreshed in the background.
fn usable(state: &AppState, row: UserRow) -> Result<UserRow, FedError> {
    if row.deleted_at.is_some() || row.suspended_at.is_some() {
        return Err(FedError::NotFound);
    }
    if is_stale(&row) {
        spawn_refresh(state, row.clone());
    }
    Ok(row)
}

fn is_stale(row: &UserRow) -> bool {
    row.fetched_at
        .is_none_or(|at| OffsetDateTime::now_utc() - at > STALE_AFTER)
}

/// Fetches the actor of `row` again and stores what it says now, for the background refresh.
///
/// # Errors
///
/// As [`actor_by_url`].
pub async fn refresh_actor(state: &AppState, row: &UserRow) -> Result<UserRow, FedError> {
    let actor = fetch_current(state, row).await?;
    Box::pin(store_actor(state, &actor))
        .await?
        .ok_or(FedError::NotFound)
}

// ------------------------------------------------------------------ background refresh

/// Actor ids being refreshed right now. One per process, like the server: a profile viewed by
/// twenty people at once is fetched once.
static REFRESHING: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(Mutex::default);

/// Holds an actor id in [`REFRESHING`] until dropped.
struct Refreshing(String);

impl Refreshing {
    fn claim(ap_id: &str) -> Option<Self> {
        REFRESHING
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(ap_id.to_owned())
            .then(|| Self(ap_id.to_owned()))
    }
}

impl Drop for Refreshing {
    fn drop(&mut self) {
        REFRESHING
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&self.0);
    }
}

/// Refreshes `row` in a detached task, unless one is running for it or the server is shutting
/// down. The task ends when shutdown starts. A failed refresh makes the profile due again in an
/// hour, so a dead server is not asked on every view.
fn spawn_refresh(state: &AppState, row: UserRow) {
    let Some(ap_id) = row.ap_id.clone() else {
        return;
    };
    if state.shutdown().is_cancelled() {
        return;
    }
    let Some(claim) = Refreshing::claim(&ap_id) else {
        return;
    };
    let state = state.clone();
    tokio::spawn(async move {
        let _claim = claim;
        let work = async {
            match refresh_actor(&state, &row).await {
                Ok(_) => {}
                Err(err) => {
                    tracing::debug!(actor = %ap_id, error = %err, "profile refresh failed");
                    if let Err(err) = remote::delay_refresh(state.db(), row.id).await {
                        tracing::debug!(error = %err, "cannot delay a profile refresh");
                    }
                }
            }
        };
        tokio::select! {
            () = state.shutdown().cancelled() => {}
            () = work => {}
        }
    });
}

// ------------------------------------------------------------------ resolving addresses

/// The account `handle`@`domain`: stored, or looked up now.
///
/// `domain` is another instance's, in the form [`parse_handle`] returns. A stored account is
/// returned as it is, and refreshed in the background when its profile is older than 24 hours.
/// An unknown one goes WebFinger, actor fetch, upsert and a backfill of the first outbox page
/// (see [`backfill_outbox`]). Only that last case reaches the network, and only then does
/// `viewer` spend one of its 30 lookups per 10 minutes; `None` skips the limit.
///
/// A `suspend`-blocked domain is [`FedError::NotFound`] without any network traffic, as are
/// deleted and suspended remote accounts.
///
/// # Errors
///
/// [`FedError::NotFound`], [`FedError::Unreachable`], [`FedError::Unreadable`] as the route
/// table documents, [`FedError::RateLimited`] when `viewer` is out of lookups, and
/// [`FedError::Internal`].
pub async fn resolve_acct(
    state: &AppState,
    viewer: Option<Uuid>,
    handle: &str,
    domain: &str,
) -> Result<UserRow, FedError> {
    let config = state.config();
    let Some(domain) = normalize_domain(domain) else {
        return Err(FedError::NotFound);
    };
    if !valid_remote_handle(handle)
        || domain.eq_ignore_ascii_case(config.domain())
        || (domain.contains(':') && !config.federation_allow_http)
    {
        return Err(FedError::NotFound);
    }
    let db = state.db();
    if remote::domain_suspended(db, &domain)
        .await
        .map_err(internal)?
    {
        return Err(FedError::NotFound);
    }
    if let Some(row) = remote::find(db, handle, &domain).await.map_err(internal)? {
        return usable(state, row);
    }
    if let Some(viewer) = viewer {
        state.limits().check_lookup(viewer).map_err(limited)?;
    }
    let href = webfinger(state, handle, &domain).await?;
    let doc = fetch::fetch_document(state, &href).await?;
    let actor = check_actor(state, actor_of(&doc)?, &doc.url, Some((handle, &domain)))?;
    let row = Box::pin(store_actor(state, &actor))
        .await?
        .ok_or(FedError::NotFound)?;
    if let Some(outbox) = actor.outbox.as_deref() {
        let stored = backfill_outbox(state, &row, outbox).await;
        tracing::debug!(actor = %actor.id, stored, "backfilled an outbox");
    }
    let row = users::by_id(db, row.id)
        .await
        .map_err(internal)?
        .ok_or(FedError::NotFound)?;
    usable(state, row)
}

/// The account a route's `{handle}` names, as `viewer` may see it: a local account (also when
/// the handle carries this instance's own domain) or, through [`resolve_acct`], a remote one.
/// This is the one place handles are parsed; every `{handle}` route goes through it.
///
/// # Errors
///
/// `404 User not found.` for an unknown, hidden or malformed handle, `502` when the other
/// server cannot be reached or sends something unreadable, `429` when `viewer` is out of
/// lookups, and database failures.
pub async fn find_user(state: &AppState, viewer: &UserRow, raw: &str) -> Result<UserRow, AppError> {
    find_user_with(state, viewer.id, viewer.is_staff(), raw).await
}

/// [`find_user`] for a viewer given by id and staff flag: list removal looks as staff so a
/// suspended local member stays removable.
///
/// # Errors
///
/// As [`find_user`].
pub async fn find_user_with(
    state: &AppState,
    viewer: Uuid,
    staff: bool,
    raw: &str,
) -> Result<UserRow, AppError> {
    match parse_handle(raw, state.config().domain()) {
        None => Err(AppError::Missing("User not found.")),
        Some(Handle::Local(handle)) => find_local(state.db(), &handle, staff).await,
        Some(Handle::Remote { handle, domain }) => {
            Ok(Box::pin(resolve_acct(state, Some(viewer), &handle, &domain)).await?)
        }
    }
}

/// Refuses with `403 {message}` when `raw` names an account on a `suspend`-blocked instance.
/// Actions that send something to the account (follow, message) call this before
/// [`find_user`], which answers `404` for such an account.
///
/// # Errors
///
/// `403` with `message`, or a database failure.
pub async fn forbid_suspended(
    state: &AppState,
    raw: &str,
    message: &'static str,
) -> Result<(), AppError> {
    if let Some(Handle::Remote { domain, .. }) = parse_handle(raw, state.config().domain())
        && remote::domain_suspended(state.db(), &domain).await?
    {
        return Err(AppError::forbidden(message));
    }
    Ok(())
}

// ------------------------------------------------------------------ outbox backfill

/// A candidate from an outbox page.
enum Candidate {
    Embedded(Box<Note>),
    Fetch(String),
}

fn candidate(item: &Value, origin: &str) -> Option<Candidate> {
    match item {
        Value::String(id) => Some(Candidate::Fetch(id.clone())),
        Value::Object(_) if ap::type_names(item).contains(&"Create") => {
            let activity = Activity::deserialize(item).ok()?;
            if activity.kind != ActivityKind::Create
                || (!activity.actor.is_empty() && activity.actor != origin)
            {
                return None;
            }
            match &activity.object {
                Value::String(id) => Some(Candidate::Fetch(id.clone())),
                object @ Value::Object(_) => Note::deserialize(object)
                    .ok()
                    .map(|note| Candidate::Embedded(Box::new(note))),
                _ => None,
            }
        }
        Value::Object(_) => Note::deserialize(item)
            .ok()
            .map(|note| Candidate::Embedded(Box::new(note))),
        _ => None,
    }
}

/// The items of the first page of the outbox at `url`: the collection's own items, an embedded
/// first page, or the page `first` links to (same host only).
async fn outbox_items(state: &AppState, origin: &str, url: &str) -> Option<Vec<Value>> {
    let doc = fetch::fetch_document(state, url).await.ok()?;
    let outbox = Collection::deserialize(&doc.json).ok()?;
    let own = outbox.page_items();
    if !own.is_empty() {
        return Some(own.to_vec());
    }
    if let Some(page) = outbox.first_page() {
        return Some(page.page_items().to_vec());
    }
    let first = outbox
        .first_url()
        .filter(|first| ap::same_authority(first, origin))?;
    let page = fetch::fetch_document(state, first).await.ok()?;
    let page = Collection::deserialize(&page.json).ok()?;
    Some(page.page_items().to_vec())
}

/// Stores up to [`BACKFILL_LIMIT`] public posts from the first page of the outbox at
/// `outbox_url` of remote account `author`, newest first as the outbox lists them. Items may
/// be embedded `Create(Note)` activities or ids, which are fetched one by one. The whole
/// backfill gives up after 20 seconds. Everything that fails is skipped. Returns how many posts
/// were stored.
pub async fn backfill_outbox(state: &AppState, author: &UserRow, outbox_url: &str) -> usize {
    let work = async {
        let Some(origin) = author.ap_id.as_deref() else {
            return 0;
        };
        if !ap::same_authority(outbox_url, origin) {
            return 0;
        }
        let Some(items) = outbox_items(state, origin, outbox_url).await else {
            return 0;
        };
        let mut stored = 0;
        for item in items.iter().take(BACKFILL_LIMIT) {
            let note = match candidate(item, origin) {
                Some(Candidate::Embedded(note)) => *note,
                Some(Candidate::Fetch(id)) if ap::same_authority(&id, origin) => {
                    match fetch_note_document(state, &id).await {
                        Ok(note) => note,
                        Err(err) => {
                            tracing::debug!(note = %id, error = %err, "backfill: cannot fetch a note");
                            continue;
                        }
                    }
                }
                _ => continue,
            };
            match ingest::store_note(state, author, &note, Context::Backfill).await {
                Ok(Stored::Post { created: true, .. }) => stored += 1,
                Ok(_) => {}
                Err(err) => {
                    tracing::debug!(note = %note.id, error = %err, "backfill: skipped a note")
                }
            }
        }
        stored
    };
    tokio::time::timeout(BACKFILL_BUDGET, work)
        .await
        .unwrap_or_else(|_| {
            tracing::debug!(outbox = outbox_url, "backfill ran out of time");
            0
        })
}

// ------------------------------------------------------------------ posts

/// Fetches the note at `url`; its `id` must be on the host that served it.
async fn fetch_note_document(state: &AppState, url: &str) -> Result<Note, FedError> {
    let doc = fetch::fetch_document(state, url).await?;
    let note =
        Note::deserialize(&doc.json).map_err(|_| FedError::Unreadable(server_of(&doc.url)))?;
    if !ap::same_authority(&note.id, &doc.url) {
        return Err(FedError::Unreadable(server_of(&doc.url)));
    }
    Ok(note)
}

async fn fetch_note(state: &AppState, url: &str, with_parent: bool) -> Result<Stored, FedError> {
    let note = fetch_note_document(state, url).await?;
    let attributed = note
        .attributed_to
        .as_deref()
        .ok_or_else(|| FedError::Unreadable(server_of(url)))?;
    let author = actor_by_url(state, attributed).await?;
    if with_parent {
        ingest::store_note(state, &author, &note, Context::Lookup).await
    } else {
        ingest::store_parent(state, &author, &note).await
    }
}

/// The post `url` names, fetched and stored: its `id` must be on the host that served it, and
/// its author (`attributedTo`) on the same host is resolved and stored first. A user asked for
/// it, so it is always stored (see [`Context::Lookup`]). A post already stored, by `ap_id` or
/// page URL, is returned without any request. Returns the stored post's id.
///
/// `viewer` spends one lookup of 30 per 10 minutes, only when the network is used.
///
/// # Errors
///
/// [`FedError::NotFound`] for a URL of this instance, a `suspend`-blocked instance (no network
/// traffic), a missing post or a message instead of a post; [`FedError::Unreachable`];
/// [`FedError::Unreadable`] for anything that is not a note by an author on its host;
/// [`FedError::RateLimited`].
pub async fn fetch_remote_post(
    state: &AppState,
    viewer: Option<Uuid>,
    url: &str,
) -> Result<Uuid, FedError> {
    let domain = ap::url_authority(url).ok_or(FedError::NotFound)?;
    if domain.eq_ignore_ascii_case(state.config().domain()) {
        return Err(FedError::NotFound);
    }
    let db = state.db();
    if remote::domain_suspended(db, &domain)
        .await
        .map_err(internal)?
    {
        return Err(FedError::NotFound);
    }
    if let Some(id) = remote::find_known_post(db, url, &domain)
        .await
        .map_err(internal)?
    {
        return Ok(id);
    }
    if let Some(viewer) = viewer {
        state.limits().check_lookup(viewer).map_err(limited)?;
    }
    match Box::pin(fetch_note(state, url, true)).await? {
        Stored::Post { id, .. } => Ok(id),
        Stored::Message { .. } | Stored::Dropped => Err(FedError::NotFound),
    }
}

/// Fetches and stores the post `url`, which a looked-up note replies to. Failures are logged:
/// the reply is stored without its parent.
pub(super) fn fetch_parent<'a>(state: &'a AppState, url: &'a str) -> BoxFuture<'a, ()> {
    async move {
        if let Err(err) = fetch_note(state, url, false).await {
            tracing::debug!(parent = url, error = %err, "cannot fetch the post a note replies to");
        }
    }
    .boxed()
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn addresses_parse_into_local_and_remote_handles() {
        let own = "me.example";
        assert_eq!(parse_handle("ada", own), Some(Handle::Local("ada".into())));
        assert_eq!(
            parse_handle("@ada@me.example", own),
            Some(Handle::Local("ada".into()))
        );
        assert_eq!(
            parse_handle("bob@127.0.0.1:4000", own),
            Some(Handle::Remote {
                handle: "bob".into(),
                domain: "127.0.0.1:4000".into()
            })
        );
        assert_eq!(
            parse_handle("bob@Example.ORG:443", own),
            Some(Handle::Remote {
                handle: "bob".into(),
                domain: "example.org".into()
            })
        );
        for bad in [
            "@@x.example",
            "bob@",
            "bob@x.example/path",
            "bob@x.example?q=1",
            "bob@[::1]",
            "bob@user:pw@x.example",
            "b o b@x.example",
            "bob@x.example:notaport",
        ] {
            assert_eq!(parse_handle(bad, own), None, "{bad}");
        }
    }

    #[test]
    fn remote_handles_have_a_safe_alphabet() {
        assert!(valid_remote_handle("a.b-c_9"));
        assert!(!valid_remote_handle(""));
        assert!(!valid_remote_handle("a/b"));
        assert!(!valid_remote_handle("ä"));
        assert!(!valid_remote_handle(&"a".repeat(65)));
        assert!(valid_remote_handle(&"a".repeat(64)));
    }

    #[test]
    fn the_actor_link_must_be_an_activitypub_self_link() {
        let jrd = json!({"links": [
            {"rel": "http://webfinger.net/rel/profile-page", "type": "text/html", "href": "https://a.example/@b"},
            {"rel": "self", "type": "text/html", "href": "https://a.example/html"},
            {"rel": "self", "type": "application/activity+json", "href": "ftp://a.example/u"},
            {"rel": "self", "type": "application/activity+json", "href": "http://a.example/u"},
            {"rel": "self", "type": "application/activity+json; charset=utf-8", "href": "https://a.example/users/b"}
        ]});
        assert_eq!(
            actor_link(&jrd, false).as_deref(),
            Some("https://a.example/users/b")
        );
        assert_eq!(
            actor_link(&jrd, true).as_deref(),
            Some("http://a.example/u")
        );
        assert_eq!(actor_link(&json!({"links": "no"}), true), None);
        assert_eq!(actor_link(&json!({}), true), None);
    }
}
