//! The unfurl worker: fetches the first link of new and edited posts and stores a link card.
//!

use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use scraper::{Html, Selector};
use sqlx::{FromRow, PgPool};
use tokio::sync::{Semaphore, mpsc};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::db::media as media_db;
use crate::dto::{CardEventDto, LinkCardDto, StreamEvent};
use crate::error::AppError;
use crate::net::{Fetch, Overflow, SafeClient};
use crate::state::AppState;

/// Jobs the queue holds before new ones are dropped.
pub const QUEUE_CAPACITY: usize = 256;

/// Longest a page or image fetch may take, redirects and body included.
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(6);

/// Most bytes of a page that are read.
const HTML_LIMIT: usize = 1 << 20;

/// Largest preview image.
const IMAGE_LIMIT: usize = 10 << 20;

const HTML_ACCEPT: &str = "text/html,application/xhtml+xml;q=0.9,*/*;q=0.5";
const IMAGE_ACCEPT: &str = "image/avif,image/webp,image/png,image/jpeg,image/gif;q=0.9";

const TITLE_MAX: usize = 200;
const DESCRIPTION_MAX: usize = 400;
const SITE_MAX: usize = 80;
const LARGE_IMAGE_WIDTH: f64 = 600.0;

/// One link to preview.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Job {
    /// The post that holds the link.
    pub post_id: Uuid,
    /// Its author, who gets the `card` event.
    pub author_id: Uuid,
    /// The first link of the post.
    pub url: String,
}
// 56: two ids (16 each) and a `String` (24).
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<Job>() == 56);

/// Previews fetched at the same time.
pub const CONCURRENCY: usize = 4;

/// Runs until `stop` is cancelled or the queue closes, with up to [`CONCURRENCY`] fetches in
/// flight. Fetches under way are dropped on cancel. Jobs for one URL run one after another, so the
/// second finds the first one's row and fetches nothing.
pub async fn run(state: AppState, mut jobs: mpsc::Receiver<Job>, stop: CancellationToken) {
    let slots = Arc::new(Semaphore::new(CONCURRENCY));
    let by_url: Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>> = Arc::default();
    let mut running = JoinSet::new();
    loop {
        let slot = tokio::select! {
            () = stop.cancelled() => return,
            slot = slots.clone().acquire_owned() => match slot {
                Ok(slot) => slot,
                Err(_) => return,
            },
        };
        let job = tokio::select! {
            () = stop.cancelled() => return,
            job = jobs.recv() => match job {
                Some(job) => job,
                None => return,
            },
        };
        while running.try_join_next().is_some() {}
        let state = state.clone();
        let lock = {
            let mut map = by_url.lock().unwrap_or_else(PoisonError::into_inner);
            map.retain(|_, lock| Arc::strong_count(lock) > 1);
            map.entry(job.url.clone()).or_default().clone()
        };
        running.spawn(async move {
            let _slot = slot;
            let _one_at_a_time = lock.lock().await;
            if let Err(err) = process_job(&state, state.safe_client(), &job).await {
                tracing::warn!(url = %job.url, error = %err, "link card not stored");
            }
        });
    }
}

/// Builds the card for `job.url` unless a fresh row exists, stores it, and sends the `card`
/// event to the author when there is a card. Returns that card.
///
/// A row is fresh for seven days, or six hours when the last fetch failed. A page that cannot
/// be fetched, is not HTML or has no title is stored as a failure, not returned as an error.
///
/// # Errors
///
/// Database failures and failures to store the preview image.
pub async fn process_job(
    state: &AppState,
    client: &SafeClient,
    job: &Job,
) -> Result<Option<LinkCardDto>, AppError> {
    let db = state.db();
    let row = match fresh_row(db, &job.url).await? {
        Some(row) => row,
        None => refresh(state, client, job).await?,
    };
    let card = row.into_card(&job.url);
    if let Some(card) = &card {
        state.hub().publish(
            job.author_id,
            StreamEvent::Card(Box::new(CardEventDto {
                post_id: job.post_id,
                card: card.clone(),
            })),
        );
    }
    Ok(card)
}

/// A `link_cards` row with its image's file name.
#[derive(Debug, FromRow)]
struct CardRow {
    title: String,
    description: String,
    site: String,
    large: bool,
    ok: bool,
    path: Option<String>,
}

impl CardRow {
    fn failed() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            site: String::new(),
            large: false,
            ok: false,
            path: None,
        }
    }

    fn into_card(self, url: &str) -> Option<LinkCardDto> {
        self.ok.then(|| LinkCardDto {
            url: url.to_owned(),
            title: self.title,
            description: self.description,
            site: self.site,
            image: self.path.as_deref().map(media_db::url),
            large: self.large,
        })
    }
}

async fn fresh_row(db: &PgPool, url: &str) -> Result<Option<CardRow>, AppError> {
    let row = sqlx::query_as(
        "select c.title, c.description, c.site, c.large, c.ok, m.path
           from link_cards c left join media m on m.id = c.image_id
          where c.url = $1
            and c.fetched_at > now() - case when c.ok then interval '7 days' else interval '6 hours' end",
    )
    .bind(url)
    .fetch_optional(db)
    .await?;
    Ok(row)
}

/// What a page said about itself, and the id of its stored image.
struct Built {
    meta: Meta,
    site: String,
    image: Option<Uuid>,
}

async fn refresh(state: &AppState, client: &SafeClient, job: &Job) -> Result<CardRow, AppError> {
    let db = state.db();
    let mut row = CardRow::failed();
    let mut image_id = None;
    if let Some(built) = build(state, client, job).await? {
        row = CardRow {
            title: built.meta.title,
            description: built.meta.description,
            site: built.site,
            large: built.meta.large,
            ok: true,
            path: None,
        };
        if let Some(image) = built.image {
            row.path = sqlx::query_scalar("select path from media where id = $1")
                .bind(image)
                .fetch_optional(db)
                .await?;
            image_id = row.path.is_some().then_some(image);
        }
    }
    sqlx::query(
        "insert into link_cards (url, title, description, site, image_id, large, ok, fetched_at)
         values ($1, $2, $3, $4, $5, $6, $7, now())
         on conflict (url) do update set title = excluded.title,
             description = excluded.description, site = excluded.site,
             image_id = excluded.image_id, large = excluded.large, ok = excluded.ok,
             fetched_at = excluded.fetched_at",
    )
    .bind(&job.url)
    .bind(&row.title)
    .bind(&row.description)
    .bind(&row.site)
    .bind(image_id)
    .bind(row.large)
    .bind(row.ok)
    .execute(db)
    .await?;
    Ok(row)
}

/// Fetches and parses the page and its image. `None` for every fetch or parse failure.
async fn build(
    state: &AppState,
    client: &SafeClient,
    job: &Job,
) -> Result<Option<Built>, AppError> {
    let page = Fetch {
        accept: HTML_ACCEPT,
        max_bytes: HTML_LIMIT,
        timeout: FETCH_TIMEOUT,
        overflow: Overflow::Truncate,
    };
    let fetched = match client.get(&job.url, page).await {
        Ok(fetched) => fetched,
        Err(err) => {
            tracing::warn!(url = %job.url, error = %err, "link preview fetch failed");
            return Ok(None);
        }
    };
    let kind = fetched.content_type.to_ascii_lowercase();
    if !kind.contains("html") && !kind.contains("xml") {
        return Ok(None);
    }
    let meta = parse_meta(&String::from_utf8_lossy(&fetched.body));
    if meta.title.is_empty() {
        return Ok(None);
    }
    let site = if meta.site.is_empty() {
        let host = fetched.url.host_str().unwrap_or_default();
        host.strip_prefix("www.").unwrap_or(host).to_owned()
    } else {
        meta.site.clone()
    };
    let image = match meta.image.as_str() {
        "" => None,
        src => match fetched.url.join(src) {
            Ok(image_url) => fetch_image(state, client, job.author_id, image_url.as_str()).await?,
            Err(_) => None,
        },
    };
    Ok(Some(Built { meta, site, image }))
}

/// A preview image that cannot be fetched or decoded only costs the card its image.
async fn fetch_image(
    state: &AppState,
    client: &SafeClient,
    owner: Uuid,
    url: &str,
) -> Result<Option<Uuid>, AppError> {
    let fetch = Fetch {
        accept: IMAGE_ACCEPT,
        max_bytes: IMAGE_LIMIT,
        timeout: FETCH_TIMEOUT,
        overflow: Overflow::Fail,
    };
    let fetched = match client.get(url, fetch).await {
        Ok(fetched) => fetched,
        Err(err) => {
            tracing::warn!(url, error = %err, "link preview image fetch failed");
            return Ok(None);
        }
    };
    match crate::media::store_card_image(state, owner, fetched.body).await {
        Ok(id) => Ok(Some(id)),
        Err(AppError::BadRequest(reason)) => {
            tracing::warn!(url, %reason, "link preview image refused");
            Ok(None)
        }
        Err(err) => Err(err),
    }
}

/// The fields of a page that make a card.
#[derive(Debug, Default, PartialEq, Eq)]
struct Meta {
    title: String,
    description: String,
    image: String,
    site: String,
    large: bool,
}

static META: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("meta").expect("`meta` is a valid selector"));
static TITLE: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("title").expect("`title` is a valid selector"));

/// Collapses runs of whitespace and cuts to `max` characters.
fn tidy(text: &str, max: usize) -> String {
    let mut out = String::new();
    for word in text.split_whitespace() {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
    }
    out.chars().take(max).collect()
}

/// Reads Open Graph, Twitter card and plain meta tags. The first tag of a name wins.
fn parse_meta(html: &str) -> Meta {
    let doc = Html::parse_document(html);
    let mut tags: HashMap<String, String> = HashMap::new();
    for element in doc.select(&META) {
        let tag = element.value();
        let key = tag
            .attr("property")
            .or_else(|| tag.attr("name"))
            .or_else(|| tag.attr("itemprop"))
            .map(|key| key.trim().to_ascii_lowercase());
        if let (Some(key), Some(content)) = (key, tag.attr("content"))
            && !key.is_empty()
        {
            tags.entry(key).or_insert_with(|| tidy(content, usize::MAX));
        }
    }
    let first = |names: &[&str]| {
        names
            .iter()
            .filter_map(|name| tags.get(*name))
            .find(|value| !value.is_empty())
            .cloned()
            .unwrap_or_default()
    };
    let mut title = first(&["og:title", "twitter:title"]);
    if title.is_empty()
        && let Some(element) = doc.select(&TITLE).next()
    {
        title = element.text().collect();
    }
    let wide = tags
        .get("og:image:width")
        .and_then(|width| width.parse::<f64>().ok())
        .is_some_and(|width| width >= LARGE_IMAGE_WIDTH);
    Meta {
        title: tidy(&title, TITLE_MAX),
        description: tidy(
            &first(&["og:description", "twitter:description", "description"]),
            DESCRIPTION_MAX,
        ),
        image: first(&[
            "og:image:secure_url",
            "og:image",
            "twitter:image",
            "twitter:image:src",
        ]),
        site: tidy(&first(&["og:site_name", "application-name"]), SITE_MAX),
        large: wide
            || tags
                .get("twitter:card")
                .is_some_and(|card| card.contains("large")),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reads_open_graph_and_decodes_entities() {
        let meta = parse_meta(
            r#"<html><head><title>Fallback</title>
            <meta property="og:title" content="Fish &amp; Chips &#x1F41F;">
            <meta property="og:description" content="  line one
               line   two ">
            <meta property="og:image" content="/img/a.png">
            <meta property="og:site_name" content="Chippy">
            <meta property="og:image:width" content="1200">
            </head></html>"#,
        );
        assert_eq!(
            meta,
            Meta {
                title: "Fish & Chips \u{1f41f}".to_owned(),
                description: "line one line two".to_owned(),
                image: "/img/a.png".to_owned(),
                site: "Chippy".to_owned(),
                large: true,
            }
        );
    }

    #[test]
    fn falls_back_to_twitter_then_title_tag() {
        let twitter = parse_meta(
            r#"<meta name="twitter:title" content="T"><meta name="twitter:card" content="summary_large_image">
               <meta name="twitter:image:src" content="x.jpg"><meta name="description" content="D">
               <meta name="application-name" content="App">"#,
        );
        assert_eq!(twitter.title, "T");
        assert_eq!(twitter.description, "D");
        assert_eq!(twitter.image, "x.jpg");
        assert_eq!(twitter.site, "App");
        assert!(twitter.large);
        let plain = parse_meta("<title> Just \n a title </title>");
        assert_eq!(plain.title, "Just a title");
        assert!(!plain.large);
        assert!(parse_meta("<p>nothing</p>").title.is_empty());
    }

    #[test]
    fn first_tag_wins_and_empty_values_fall_through() {
        let meta = parse_meta(
            r#"<meta property="og:title" content=""><meta name="twitter:title" content="Second">
               <meta property="og:image" content="one.png"><meta property="og:image" content="two.png">
               <meta property="og:image:width" content="600px">"#,
        );
        assert_eq!(meta.title, "Second");
        assert_eq!(meta.image, "one.png");
        assert!(!meta.large);
    }

    #[test]
    fn cuts_long_fields() {
        let long = "é".repeat(500);
        let meta = parse_meta(&format!(
            r#"<meta property="og:title" content="{long}"><meta property="og:description" content="{long}">
               <meta property="og:site_name" content="{long}">"#
        ));
        assert_eq!(meta.title.chars().count(), TITLE_MAX);
        assert_eq!(meta.description.chars().count(), DESCRIPTION_MAX);
        assert_eq!(meta.site.chars().count(), SITE_MAX);
    }
}
