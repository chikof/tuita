//! Mastodon pagination: `max_id`, `since_id`, `min_id` and `limit` in, a `Link` header out.
//!
//! A list handler takes [`Paging`], runs its query with the numbers the helper gives, and
//! answers with the `Link` header:
//!
//! ```ignore
//! async fn home(State(state): State<AppState>, paging: Paging<Uuid>) -> Result<Response, AppError> {
//!     // `ids`: newest first, or oldest first when `paging.ascending()`, at most `paging.fetch()`.
//!     let ids = query(paging.upper(), paging.lower(), paging.ascending(), paging.fetch()).await?;
//!     let slice = paging.trim(ids);
//!     let link = paging.link(state.config(), slice.rows.first(), slice.rows.last(), slice.more);
//!     // build the statuses from `slice.rows`, then `(link, Json(statuses))`
//! }
//! ```
//!
//! The cursors are Mastodon ids, which here are UUIDv7 strings (`Paging<Uuid>`), or opaque
//! strings for lists that page by their own record (`Paging<String>`: bookmarks, favourites).
//! `max_id` returns rows older than the id; `since_id` the newest rows newer than the id (the
//! rows nearest the newest end, a gap may remain); `min_id` the rows immediately newer than the id.
//! Rows are always answered newest first.

use std::fmt::Display;
use std::str::FromStr;

use axum::extract::{FromRequestParts, OriginalUri};
use axum::http::HeaderValue;
use axum::http::request::Parts;
use url::{Url, form_urlencoded};

use crate::config::Config;
use crate::error::AppError;

/// Rows a list returns when the client sends no `limit`.
pub const DEFAULT_LIMIT: u32 = 20;
/// The most rows a list returns, whatever `limit` asks (Mastodon allows 80 on a few lists; the
/// subset keeps 40 everywhere).
pub const MAX_LIMIT: u32 = 40;

/// The parsed paging parameters of one request. `C` is the cursor type: [`uuid::Uuid`] for
/// ids, `String` for opaque record cursors.
#[derive(Clone, Debug)]
pub struct Paging<C> {
    /// Rows older than this.
    pub max_id: Option<C>,
    /// Rows newer than this, nearest the newest end.
    pub since_id: Option<C>,
    /// Rows immediately newer than this.
    pub min_id: Option<C>,
    requested: Option<u32>,
    path: String,
    rest: Vec<(String, String)>,
}

impl<C: FromStr + Display> Paging<C> {
    /// Reads the parameters from a query string. `path` is the public path of the route, for
    /// the links. Query pairs other than the four paging ones are kept and repeated in the links.
    ///
    /// # Errors
    ///
    /// `400` when a cursor does not parse as `C`.
    pub fn parse(path: &str, query: Option<&str>) -> Result<Self, AppError> {
        let mut paging = Self {
            max_id: None,
            since_id: None,
            min_id: None,
            requested: None,
            path: path.to_owned(),
            rest: Vec::new(),
        };
        for (key, value) in form_urlencoded::parse(query.unwrap_or_default().as_bytes()) {
            let cursor = |value: &str| {
                if value.is_empty() {
                    Ok(None)
                } else {
                    value
                        .parse::<C>()
                        .map(Some)
                        .map_err(|_| AppError::bad_request("Invalid pagination cursor."))
                }
            };
            match &*key {
                "max_id" => paging.max_id = cursor(&value)?,
                "since_id" => paging.since_id = cursor(&value)?,
                "min_id" => paging.min_id = cursor(&value)?,
                "limit" => paging.requested = value.parse().ok(),
                _ => paging.rest.push((key.into_owned(), value.into_owned())),
            }
        }
        Ok(paging)
    }

    /// The number of rows to return: the client's `limit`, [`DEFAULT_LIMIT`] without one, at
    /// most [`MAX_LIMIT`] and at least 1.
    #[must_use]
    pub fn limit(&self) -> u32 {
        self.requested.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
    }

    /// How many rows the query asks for: one more than [`Self::limit`], so [`Self::trim`] can
    /// tell whether more exist.
    #[must_use]
    pub fn fetch(&self) -> i64 {
        i64::from(self.limit()) + 1
    }

    /// The exclusive upper bound of the rows: `max_id`.
    #[must_use]
    pub fn upper(&self) -> Option<&C> {
        self.max_id.as_ref()
    }

    /// The exclusive lower bound of the rows: `min_id` if sent, else `since_id`.
    #[must_use]
    pub fn lower(&self) -> Option<&C> {
        self.min_id.as_ref().or(self.since_id.as_ref())
    }

    /// Whether the query reads the rows nearest the lower bound first (`min_id`), so it must
    /// `order by id asc`. Otherwise it reads newest first.
    #[must_use]
    pub fn ascending(&self) -> bool {
        self.min_id.is_some()
    }

    /// Cuts the extra row [`Self::fetch`] asked for and returns the rows newest first. `rows`
    /// are in the query's own order: newest first, or oldest first when [`Self::ascending`].
    #[must_use]
    pub fn trim<T>(&self, mut rows: Vec<T>) -> Slice<T> {
        let more = rows.len() > self.limit() as usize;
        rows.truncate(self.limit() as usize);
        if self.ascending() {
            rows.reverse();
        }
        Slice { rows, more }
    }

    /// The `Link` header for a trimmed page, or `None` for an empty one. `newest` and `oldest`
    /// are the cursors of the first and the last row, taken before rows the viewer may not
    /// see were dropped. `more` is [`Slice::more`].
    ///
    /// `next` (older rows) is sent when more rows exist; for a `min_id` page older rows exist by
    /// definition. `prev` (newer rows) is sent whenever the page has rows.
    #[must_use]
    pub fn link<K: Display>(
        &self,
        config: &Config,
        newest: Option<K>,
        oldest: Option<K>,
        more: bool,
    ) -> Option<HeaderValue> {
        let (newest, oldest) = (newest?, oldest?);
        let url = |cursor: &str, id: &K| {
            let mut url = Url::parse(&format!("{}{}", config.public_url, self.path)).ok()?;
            {
                let mut pairs = url.query_pairs_mut();
                for (key, value) in &self.rest {
                    pairs.append_pair(key, value);
                }
                if let Some(limit) = self.requested {
                    pairs.append_pair("limit", &limit.to_string());
                }
                pairs.append_pair(cursor, &id.to_string());
            }
            Some(url)
        };
        let mut links = Vec::new();
        if more || self.ascending() {
            links.push(format!("<{}>; rel=\"next\"", url("max_id", &oldest)?));
        }
        links.push(format!("<{}>; rel=\"prev\"", url("min_id", &newest)?));
        HeaderValue::from_str(&links.join(", ")).ok()
    }
}

/// Rows of one page after [`Paging::trim`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slice<T> {
    /// The rows, newest first.
    pub rows: Vec<T>,
    /// The query found rows beyond the limit, in the direction it was reading.
    pub more: bool,
}

impl<C, S> FromRequestParts<S> for Paging<C>
where
    C: FromStr + Display,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, AppError> {
        let path = parts
            .extensions
            .get::<OriginalUri>()
            .map_or_else(|| parts.uri.path(), |original| original.0.path())
            .to_owned();
        Self::parse(&path, parts.uri.query())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use uuid::Uuid;

    fn config() -> Config {
        Config::from_lookup(|var| match var {
            "DATABASE_URL" => Some("postgres://unused".to_owned()),
            "PUBLIC_URL" => Some("https://social.example".to_owned()),
            _ => None,
        })
        .expect("config")
    }

    const OLD: &str = "0198aaaa-0000-7000-8000-000000000001";
    const NEW: &str = "0198aaaa-0000-7000-8000-000000000009";

    #[test]
    fn limits_default_and_clamp() {
        let none = Paging::<Uuid>::parse("/api/v1/x", None).expect("parse");
        assert_eq!((none.limit(), none.fetch()), (20, 21));
        let big = Paging::<Uuid>::parse("/api/v1/x", Some("limit=500")).expect("parse");
        assert_eq!(big.limit(), 40);
        let zero = Paging::<Uuid>::parse("/api/v1/x", Some("limit=0")).expect("parse");
        assert_eq!(zero.limit(), 1);
        let junk = Paging::<Uuid>::parse("/api/v1/x", Some("limit=abc")).expect("parse");
        assert_eq!(junk.limit(), 20);
    }

    #[test]
    fn cursors_are_checked_and_bounds_follow_mastodon() {
        assert!(Paging::<Uuid>::parse("/x", Some("max_id=1234")).is_err());
        let both = Paging::<Uuid>::parse("/x", Some(&format!("max_id={NEW}&since_id={OLD}")))
            .expect("parse");
        assert_eq!(both.upper().map(ToString::to_string).as_deref(), Some(NEW));
        assert_eq!(both.lower().map(ToString::to_string).as_deref(), Some(OLD));
        assert!(!both.ascending());
        let min = Paging::<Uuid>::parse("/x", Some(&format!("min_id={OLD}"))).expect("parse");
        assert!(min.ascending());
        // Opaque cursors take any text; an empty value is no cursor.
        let opaque = Paging::<String>::parse("/x", Some("max_id=abc&since_id=")).expect("parse");
        assert_eq!(opaque.max_id.as_deref(), Some("abc"));
        assert_eq!(opaque.since_id, None);
    }

    #[test]
    fn trim_cuts_the_probe_row_and_flips_a_min_id_page() {
        let desc = Paging::<String>::parse("/x", Some("limit=2")).expect("parse");
        let slice = desc.trim(vec![9, 8, 7]);
        assert_eq!(
            slice,
            Slice {
                rows: vec![9, 8],
                more: true
            }
        );
        let slice = desc.trim(vec![9, 8]);
        assert_eq!(
            slice,
            Slice {
                rows: vec![9, 8],
                more: false
            }
        );
        let asc = Paging::<String>::parse("/x", Some("min_id=a&limit=2")).expect("parse");
        let slice = asc.trim(vec![7, 8, 9]);
        assert_eq!(
            slice,
            Slice {
                rows: vec![8, 7],
                more: true
            }
        );
    }

    #[test]
    fn the_link_header_has_absolute_urls_and_keeps_other_parameters() {
        let config = config();
        let paging = Paging::<Uuid>::parse(
            "/api/v1/notifications",
            Some("types%5B%5D=mention&exclude_types%5B%5D=follow&limit=5"),
        )
        .expect("parse");
        let link = paging
            .link(&config, Some(NEW), Some(OLD), true)
            .expect("link");
        assert_eq!(
            link.to_str().expect("ascii"),
            format!(
                "<https://social.example/api/v1/notifications?types%5B%5D=mention&exclude_types%5B%5D=follow&limit=5&max_id={OLD}>; rel=\"next\", \
                 <https://social.example/api/v1/notifications?types%5B%5D=mention&exclude_types%5B%5D=follow&limit=5&min_id={NEW}>; rel=\"prev\""
            )
        );
        let last = paging
            .link(&config, Some(NEW), Some(OLD), false)
            .expect("link");
        assert!(!last.to_str().expect("ascii").contains("rel=\"next\""));
        assert!(last.to_str().expect("ascii").contains("rel=\"prev\""));
        assert!(paging.link::<&str>(&config, None, None, false).is_none());
    }

    #[test]
    fn a_min_id_page_always_links_older_rows() {
        let config = config();
        let paging =
            Paging::<Uuid>::parse("/api/v1/timelines/home", Some(&format!("min_id={OLD}")))
                .expect("parse");
        let link = paging
            .link(&config, Some(NEW), Some(OLD), false)
            .expect("link");
        assert!(link.to_str().expect("ascii").contains("rel=\"next\""));
    }
}
