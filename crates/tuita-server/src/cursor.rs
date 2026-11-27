//! List pagination: the `before` and `limit` query parameters and the cursors behind them.
//!
//! A cursor is opaque to clients. The server hands out the cursor of the last entry as
//! `Page::next` and takes it back as `before`. Lists keyed by a v7 uuid (posts, notifications)
//! use that uuid. Lists keyed by a join row (followers, likes, bookmarks) use an [`EdgeCursor`].
//! Ranked or searched lists use a decimal offset.

use serde::Deserialize;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::dto::Page;
use crate::error::AppError;

/// Entries per page when the client sends no `limit`.
pub const DEFAULT_LIMIT: u32 = 20;
/// The most entries one page may hold.
pub const MAX_LIMIT: u32 = 40;

/// The page size: 1 to [`MAX_LIMIT`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limit(u32);

impl Limit {
    /// Validates a client-supplied size; `None` means the default.
    ///
    /// # Errors
    ///
    /// `400 Invalid limit.` when it is 0 or above [`MAX_LIMIT`].
    pub fn new(raw: Option<u32>) -> Result<Self, AppError> {
        match raw {
            None => Ok(Self(DEFAULT_LIMIT)),
            Some(n) if (1..=MAX_LIMIT).contains(&n) => Ok(Self(n)),
            Some(_) => Err(AppError::bad_request("Invalid limit.")),
        }
    }

    /// The size as the number of entries to return.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }

    /// The size as a SQL `LIMIT` that reads one extra row to learn whether a next page exists.
    #[must_use]
    pub const fn fetch(self) -> i64 {
        self.0 as i64 + 1
    }

    /// The size as an index.
    #[must_use]
    pub const fn size(self) -> usize {
        self.0 as usize
    }
}

/// The query string every list route accepts.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct PageQuery {
    /// Cursor from the previous page's `next`.
    pub before: Option<String>,
    /// Entries per page.
    pub limit: Option<u32>,
}

impl PageQuery {
    /// The page size.
    ///
    /// # Errors
    ///
    /// `400 Invalid limit.` when out of range.
    pub fn limit(&self) -> Result<Limit, AppError> {
        Limit::new(self.limit)
    }

    /// `before` as a uuid cursor.
    ///
    /// # Errors
    ///
    /// `400 Invalid cursor.` when present but not a uuid.
    pub fn id_cursor(&self) -> Result<Option<Uuid>, AppError> {
        self.before
            .as_deref()
            .map(|raw| Uuid::parse_str(raw).map_err(|_| invalid_cursor()))
            .transpose()
    }

    /// `before` as an [`EdgeCursor`].
    ///
    /// # Errors
    ///
    /// `400 Invalid cursor.` when present but malformed.
    pub fn edge_cursor(&self) -> Result<Option<EdgeCursor>, AppError> {
        self.before.as_deref().map(EdgeCursor::parse).transpose()
    }

    /// `before` as a row offset; 0 when absent.
    ///
    /// # Errors
    ///
    /// `400 Invalid cursor.` when present but not a number.
    pub fn offset(&self) -> Result<i64, AppError> {
        parse_offset(self.before.as_deref())
    }
}

/// Parses an optional decimal offset; absent means 0.
///
/// # Errors
///
/// `400 Invalid cursor.` when present but not a non-negative number.
pub fn parse_offset(raw: Option<&str>) -> Result<i64, AppError> {
    match raw {
        None => Ok(0),
        Some(raw) => raw
            .parse::<i64>()
            .ok()
            .filter(|n| *n >= 0)
            .ok_or_else(invalid_cursor),
    }
}

fn invalid_cursor() -> AppError {
    AppError::bad_request("Invalid cursor.")
}

/// Position in a list ordered by `(created_at desc, id desc)` of a join table row.
///
/// Written `<unix micros>:<uuid>`; the uuid is the id of the row's other side (the follower, the
/// liked post, ...), which breaks ties between rows created in the same microsecond.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeCursor {
    /// `created_at` of the last row of the previous page.
    pub at: OffsetDateTime,
    /// The tie-breaking id of that row.
    pub id: Uuid,
}

impl EdgeCursor {
    /// The cursor of a row.
    #[must_use]
    pub const fn new(at: OffsetDateTime, id: Uuid) -> Self {
        Self { at, id }
    }

    /// Reads `<unix micros>:<uuid>`.
    ///
    /// # Errors
    ///
    /// `400 Invalid cursor.` when malformed.
    pub fn parse(raw: &str) -> Result<Self, AppError> {
        let (micros, id) = raw.split_once(':').ok_or_else(invalid_cursor)?;
        let micros: i64 = micros.parse().map_err(|_| invalid_cursor())?;
        let at = OffsetDateTime::from_unix_timestamp_nanos(i128::from(micros) * 1000)
            .map_err(|_| invalid_cursor())?;
        let id = Uuid::parse_str(id).map_err(|_| invalid_cursor())?;
        Ok(Self { at, id })
    }

    /// The wire form, for `Page::next`.
    #[must_use]
    pub fn encode(&self) -> String {
        let micros = self.at.unix_timestamp_nanos() / 1000;
        format!("{micros}:{}", self.id)
    }
}

/// Parses a path or query value as a uuid.
///
/// # Errors
///
/// `404` with `missing` when it is not a uuid, since no row has such an id.
pub fn parse_id(raw: &str, missing: &'static str) -> Result<Uuid, AppError> {
    Uuid::parse_str(raw).map_err(|_| AppError::Missing(missing))
}

/// Builds a page from up to `limit + 1` rows read with [`Limit::fetch`]. When the extra row is
/// there it is dropped and `cursor_of` the last kept row becomes `next`.
pub fn page<T>(mut rows: Vec<T>, limit: Limit, cursor_of: impl Fn(&T) -> String) -> Page<T> {
    let next = if rows.len() > limit.size() {
        rows.truncate(limit.size());
        rows.last().map(cursor_of)
    } else {
        None
    };
    Page { items: rows, next }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn limit_bounds() {
        assert_eq!(Limit::new(None).ok().map(Limit::get), Some(20));
        assert_eq!(Limit::new(Some(40)).ok().map(Limit::get), Some(40));
        assert!(Limit::new(Some(0)).is_err());
        assert!(Limit::new(Some(41)).is_err());
    }

    #[test]
    fn edge_cursor_round_trips() {
        let at = OffsetDateTime::from_unix_timestamp_nanos(1_700_000_000_123_456_000)
            .expect("valid time");
        let cursor = EdgeCursor::new(at, Uuid::from_u128(7));
        let text = cursor.encode();
        assert_eq!(
            text,
            "1700000000123456:00000000-0000-0000-0000-000000000007"
        );
        assert_eq!(EdgeCursor::parse(&text).ok(), Some(cursor));
        assert!(EdgeCursor::parse("nope").is_err());
        assert!(EdgeCursor::parse("12:not-a-uuid").is_err());
    }

    #[test]
    fn page_cuts_the_extra_row() {
        let limit = Limit::new(Some(2)).expect("valid");
        let full = page(vec![1, 2, 3], limit, |n| n.to_string());
        assert_eq!(full.items, [1, 2]);
        assert_eq!(full.next.as_deref(), Some("2"));
        let last = page(vec![1, 2], limit, |n| n.to_string());
        assert_eq!(last.items, [1, 2]);
        assert_eq!(last.next, None);
    }

    #[test]
    fn offsets() {
        assert_eq!(parse_offset(None).ok(), Some(0));
        assert_eq!(parse_offset(Some("40")).ok(), Some(40));
        assert!(parse_offset(Some("-1")).is_err());
        assert!(parse_offset(Some("x")).is_err());
    }
}
