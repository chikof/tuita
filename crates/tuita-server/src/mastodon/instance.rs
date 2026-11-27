//! Instance: `GET /api/v1/instance`, `GET /api/v2/instance` and `GET /api/v1/custom_emojis`.
//!
//! `GET /api/v1/instance` is tuita's own route (`routes/instance.rs`: the web client reads it
//! before sign-in) and one object serves both kinds of client: [`v1_object`] adds the fields of
//! Mastodon's v1 instance to tuita's. Where the two share a name, the object keeps both readable:
//!
//! - `version` is `4.3.0 (compatible; tuita X.Y.Z)`, the form Mastodon clients parse; the plain
//!   version is `tuitaVersion`.
//! - `stats` holds tuita's `users` and `posts` and Mastodon's `user_count`, `status_count` and
//!   `domain_count`.
//! - `rules` is tuita's array of sentences, except for a request that an OAuth token
//!   authenticated, which gets Mastodon's `[{id, text, hint}]`. Mastodon clients that ask without
//!   a token should read the rules from `GET /api/v2/instance`.
//!
//! The limits in `configuration` are the ones the server enforces: the post length, the media per
//! post, the upload sizes and the poll rules.

use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::Value;
use tuita_core::text::{LINK_LEN, MAX_POST_LEN};

use super::dto::{
    self, AccountsConfig, ContactV2, CustomEmoji, InstanceConfigurationV1, InstanceConfigurationV2,
    InstanceStatsV1, InstanceThumbnail, InstanceUrlsV1, InstanceUrlsV2, InstanceUsage,
    InstanceUsers, InstanceV1, InstanceV2, MediaConfig, PollsConfig, RegistrationsV2, Rule,
    StatusesConfig, TranslationConfig,
};
use crate::config::Config;
use crate::db::emoji;
use crate::db::media::MAX_IMAGES;
use crate::db::polls::{MAX_HOURS, MAX_OPTION_CHARS, MAX_OPTIONS};
use crate::db::{accounts, settings};
use crate::dto::{InstanceDto, SignupsMode};
use crate::error::AppError;
use crate::media::{MAX_ALT_CHARS, MAX_IMAGE_BYTES, MAX_PIXELS};
use crate::state::AppState;

/// The Mastodon version tuita claims to speak.
const MASTODON_VERSION: &str = "4.3.0";
const MIB: u64 = 1024 * 1024;
/// Highest video frame rate clients are told about; tuita does not transcode video.
const VIDEO_FRAME_RATE: u32 = 60;
/// Largest video in pixels clients are told about (1920 by 1200).
const VIDEO_MATRIX: u64 = 2_304_000;
const HOUR: u32 = 3600;

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/custom_emojis", get(custom_emojis))
}

pub(super) fn router_v2() -> Router<AppState> {
    Router::new().route("/instance", get(v2))
}

/// `GET /api/v1/custom_emojis`: the local emotes, by category and shortcode.
async fn custom_emojis(State(state): State<AppState>) -> Result<Json<Vec<CustomEmoji>>, AppError> {
    let emojis = emoji::list_local(state.db()).await?;
    Ok(Json(dto::custom_emojis(state.config(), &emojis)))
}

/// The version string Mastodon clients read.
fn version() -> String {
    format!(
        "{MASTODON_VERSION} (compatible; tuita {})",
        env!("CARGO_PKG_VERSION")
    )
}

/// `ws://` or `wss://` plus the host of `PUBLIC_URL`: where the streaming API is.
fn streaming_origin(config: &Config) -> String {
    let origin = &*config.public_url;
    match origin.split_once("://") {
        Some(("https", rest)) => format!("wss://{rest}"),
        Some(("http", rest)) => format!("ws://{rest}"),
        _ => origin.to_owned(),
    }
}

/// The contact setting when it is an address, else nothing: Mastodon's `email` is not a URL.
fn email_of(contact: &str) -> String {
    let contact = contact.trim();
    if contact.contains('@') && !contact.contains("://") && !contact.contains(char::is_whitespace) {
        contact.to_owned()
    } else {
        String::new()
    }
}

pub(super) fn rules_of(rules: &[String]) -> Vec<Rule> {
    rules
        .iter()
        .enumerate()
        .map(|(index, text)| Rule {
            id: (index + 1).to_string(),
            text: text.clone(),
            hint: String::new(),
        })
        .collect()
}

fn statuses_config() -> StatusesConfig {
    StatusesConfig {
        max_characters: u32::try_from(MAX_POST_LEN).unwrap_or(u32::MAX),
        max_media_attachments: u32::try_from(MAX_IMAGES).unwrap_or(u32::MAX),
        characters_reserved_per_url: u32::try_from(LINK_LEN).unwrap_or(u32::MAX),
    }
}

fn media_config(config: &Config) -> MediaConfig {
    MediaConfig {
        supported_mime_types: [
            "image/jpeg",
            "image/png",
            "image/webp",
            "image/gif",
            "video/mp4",
            "video/webm",
            "video/quicktime",
        ]
        .map(str::to_owned)
        .to_vec(),
        description_limit: u32::try_from(MAX_ALT_CHARS).unwrap_or(u32::MAX),
        image_size_limit: MAX_IMAGE_BYTES as u64,
        image_matrix_limit: MAX_PIXELS,
        video_size_limit: u64::from(config.max_video_mb.get()) * MIB,
        video_frame_rate_limit: VIDEO_FRAME_RATE,
        video_matrix_limit: VIDEO_MATRIX,
    }
}

fn polls_config() -> PollsConfig {
    PollsConfig {
        max_options: u32::try_from(MAX_OPTIONS).unwrap_or(u32::MAX),
        max_characters_per_option: u32::try_from(MAX_OPTION_CHARS).unwrap_or(u32::MAX),
        min_expiration: HOUR,
        max_expiration: MAX_HOURS * HOUR,
    }
}

fn icon(config: &Config) -> String {
    format!("{}/icon-512.png", config.public_url)
}

/// Whether sign-ups are open right now: the mode says so, or no admin exists yet. The one
/// connection it takes is back in the pool when it returns, so callers hold nothing while it waits.
pub(crate) async fn registrations_open(
    state: &AppState,
    signups: SignupsMode,
) -> Result<bool, AppError> {
    if signups == SignupsMode::Open {
        return Ok(true);
    }
    let mut conn = state.db().acquire().await?;
    Ok(!accounts::has_admin(&mut conn).await?)
}

/// How long [`numbers`] keeps what it counted.
const NUMBERS_TTL: Duration = Duration::from_secs(60);

/// The two counts of the instance routes that scan whole tables, and when they were counted.
/// One per [`AppState`]; `GET /instance` is public, so each request must not run them again.
#[derive(Debug, Default)]
pub(crate) struct NumbersCache(Mutex<Option<(Instant, Numbers)>>);

/// Other instances known, and accounts active in the last 30 days.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Numbers {
    domains: i64,
    active_month: i64,
}

/// [`Numbers`], counted at most once a minute. Nothing is held across the queries: concurrent
/// first requests may each count once, which costs a few queries and never a pool connection
/// that waits for another.
async fn numbers(state: &AppState) -> Result<Numbers, AppError> {
    let cache = state.numbers_cache();
    if let Some((at, found)) = *cache.0.lock().unwrap_or_else(PoisonError::into_inner)
        && at.elapsed() < NUMBERS_TTL
    {
        return Ok(found);
    }
    let found = Numbers {
        domains: count_domains(state).await?,
        active_month: count_active(state).await?,
    };
    *cache.0.lock().unwrap_or_else(PoisonError::into_inner) = Some((Instant::now(), found));
    Ok(found)
}

/// Other instances this one has accounts from.
async fn count_domains(state: &AppState) -> Result<i64, AppError> {
    Ok(
        sqlx::query_scalar("select count(distinct domain) from users where domain is not null")
            .fetch_one(state.db())
            .await?,
    )
}

/// The body of `GET /api/v1/instance`: tuita's `dto` with Mastodon's v1 fields added, see the
/// module docs. `oauth` is true for a request that an OAuth token authenticated.
///
/// # Errors
///
/// Database failures.
pub(crate) async fn v1_object(
    state: &AppState,
    dto: &InstanceDto,
    oauth: bool,
) -> Result<Value, AppError> {
    let config = state.config();
    let mastodon = InstanceV1 {
        uri: dto.domain.clone(),
        title: dto.name.clone(),
        short_description: dto.description.clone(),
        description: dto.description.clone(),
        email: email_of(&dto.contact),
        version: version(),
        urls: InstanceUrlsV1 {
            streaming_api: streaming_origin(config),
        },
        stats: InstanceStatsV1 {
            user_count: dto.stats.users,
            status_count: dto.stats.posts,
            domain_count: numbers(state).await?.domains,
        },
        thumbnail: Some(icon(config)),
        languages: vec!["en".to_owned()],
        registrations: dto.registrations_open,
        approval_required: false,
        invites_enabled: dto.signups == SignupsMode::Invite,
        configuration: InstanceConfigurationV1 {
            statuses: statuses_config(),
            media_attachments: media_config(config),
            polls: polls_config(),
        },
        contact_account: None,
        rules: rules_of(&dto.rules),
    };
    let (Ok(Value::Object(mut object)), Ok(Value::Object(extra))) =
        (serde_json::to_value(dto), serde_json::to_value(&mastodon))
    else {
        return Err(AppError::Internal(anyhow::anyhow!(
            "an instance is not a JSON object"
        )));
    };
    object.insert(
        "tuitaVersion".to_owned(),
        Value::String(env!("CARGO_PKG_VERSION").to_owned()),
    );
    for (key, value) in extra {
        match (key.as_str(), object.get_mut(&key)) {
            ("rules", Some(_)) if !oauth => {}
            ("stats", Some(Value::Object(stats))) => {
                if let Value::Object(more) = value {
                    stats.extend(more);
                }
            }
            _ => {
                object.insert(key, value);
            }
        }
    }
    Ok(Value::Object(object))
}

/// Accounts that signed in during the last 30 days, by session or token.
async fn count_active(state: &AppState) -> Result<i64, AppError> {
    Ok(sqlx::query_scalar(
        "select count(*) from (
            select user_id from sessions where last_used_at > now() - interval '30 days'
            union
            select user_id from api_tokens
             where user_id is not null and last_used_at > now() - interval '30 days'
         ) active",
    )
    .fetch_one(state.db())
    .await?)
}

/// `GET /api/v2/instance`.
async fn v2(State(state): State<AppState>) -> Result<Json<InstanceV2>, AppError> {
    let config = state.config();
    let settings = settings::load(state.db()).await?;
    let open = registrations_open(&state, settings.signups).await?;
    Ok(Json(InstanceV2 {
        domain: config.domain().to_owned(),
        title: settings.name,
        version: version(),
        source_url: config.source_url.as_deref().unwrap_or_default().to_owned(),
        description: settings.description,
        usage: InstanceUsage {
            users: InstanceUsers {
                active_month: numbers(&state).await?.active_month,
            },
        },
        thumbnail: InstanceThumbnail { url: icon(config) },
        languages: vec!["en".to_owned()],
        configuration: InstanceConfigurationV2 {
            urls: InstanceUrlsV2 {
                streaming: streaming_origin(config),
            },
            accounts: AccountsConfig {
                max_featured_tags: 0,
                max_pinned_statuses: 1,
            },
            statuses: statuses_config(),
            media_attachments: media_config(config),
            polls: polls_config(),
            translation: TranslationConfig { enabled: false },
        },
        registrations: RegistrationsV2 {
            enabled: open,
            approval_required: false,
            message: None,
            url: None,
        },
        contact: ContactV2 {
            email: email_of(&settings.contact),
            account: None,
        },
        rules: rules_of(&settings.rules),
    }))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn the_streaming_origin_follows_the_public_scheme() {
        let config = |url: &str| {
            let url = url.to_owned();
            Config::from_lookup(move |var| match var {
                "DATABASE_URL" => Some("postgres://unused".to_owned()),
                "PUBLIC_URL" => Some(url.clone()),
                _ => None,
            })
            .expect("config")
        };
        assert_eq!(
            streaming_origin(&config("https://social.example")),
            "wss://social.example"
        );
        assert_eq!(
            streaming_origin(&config("http://localhost:8082")),
            "ws://localhost:8082"
        );
    }

    #[test]
    fn only_an_address_is_an_email() {
        assert_eq!(email_of("admin@example.org"), "admin@example.org");
        assert_eq!(email_of("https://example.org/@admin"), "");
        assert_eq!(email_of("Ada Lovelace @ home"), "");
        assert_eq!(email_of(""), "");
    }

    #[test]
    fn the_version_has_the_form_mastodon_clients_parse() {
        let version = version();
        assert!(
            version.starts_with("4.3.0 (compatible; tuita "),
            "{version}"
        );
        assert!(version.ends_with(')'), "{version}");
    }

    #[test]
    fn rules_are_numbered_from_one() {
        let rules = rules_of(&["Be kind.".to_owned(), "No spam.".to_owned()]);
        assert_eq!(rules[1].id, "2");
        assert_eq!(rules[1].text, "No spam.");
    }
}
