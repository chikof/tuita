//! Post search syntax.
//!
//! A search is words separated by spaces. Every part must match:
//!
//! - `word` and `"exact phrase"`: the post text contains it, ignoring case.
//! - `-word` and `-"exact phrase"`: the post text does not contain it.
//! - `from:handle`: written by that handle. `to:handle`: a reply to that handle, or a post that
//!   mentions it.
//! - `has:media`, `has:image`, `has:video`, `has:poll`, `has:links`. The plural
//!   forms `images` and `videos`, and `link`, work too.
//! - `since:YYYY-MM-DD` and `until:YYYY-MM-DD`: posted on or after, and on or before, that day.
//!
//! A leading `@` on `from:` and `to:` values is dropped. An operator with a value it does not
//! understand (`has:cats`, `since:soon`) is ignored. A negated operator (`-from:ana`) is a plain
//! excluded word. [`parse_query`] never fails; the server turns the [`Query`] into SQL.
//!
//! ```
//! use tuita_core::search::{Has, parse_query};
//!
//! let query = parse_query(r#"cats -dogs "good boy" from:@ana has:media since:2024-01-31"#);
//! assert_eq!(query.terms, ["cats".into(), "good boy".into()]);
//! assert_eq!(query.excluded, ["dogs".into()]);
//! assert_eq!(query.from, ["ana".into()]);
//! assert_eq!(query.has, [Has::Media]);
//! assert!(query.has_operators);
//! ```

use time::{Date, Month};

use crate::text::is_js_space;

/// The longest search text; the server cuts the rest.
pub const MAX_QUERY_LEN: usize = 100;

/// What kind of attachment `has:` asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Has {
    /// An image or a video (`has:media`).
    Media,
    /// An image (`has:image`).
    Image,
    /// A video (`has:video`).
    Video,
    /// A poll (`has:poll`).
    Poll,
    /// A link: the text contains `http` (`has:links`).
    Links,
}

/// A parsed search. Every part must match, so an empty query matches every post.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Query {
    /// Words and phrases the post text must contain, ignoring case.
    pub terms: Vec<Box<str>>,
    /// Words and phrases the post text must not contain.
    pub excluded: Vec<Box<str>>,
    /// Handles the author must equal, ignoring case. Two different ones match nothing.
    pub from: Vec<Box<str>>,
    /// Handles the post must reply to or mention, ignoring case.
    pub to: Vec<Box<str>>,
    /// Attachments the post must have.
    pub has: Vec<Has>,
    /// The first day to include, in UTC.
    pub since: Option<Date>,
    /// The last day to include, in UTC: posts before the start of the following day.
    pub until: Option<Date>,
    /// Whether the text used any operator or exclusion. Callers then leave matching people out of
    /// the results.
    pub has_operators: bool,
}

/// Parses search text. It is trimmed, and cut to [`MAX_QUERY_LEN`] characters first.
#[must_use]
pub fn parse_query(input: &str) -> Query {
    let input = input.trim();
    let end = input
        .char_indices()
        .nth(MAX_QUERY_LEN)
        .map_or(input.len(), |(i, _)| i);
    let mut query = Query::default();
    for token in tokens(&input[..end]) {
        let (negated, token) = match token.strip_prefix('-') {
            Some(rest) if !rest.is_empty() => (true, rest),
            _ => (false, token),
        };
        if !negated && let Some((key, value)) = operator(token) {
            query.has_operators = true;
            query.apply(key, value.strip_prefix('@').unwrap_or(value));
            continue;
        }
        let term = token.strip_prefix('"').unwrap_or(token);
        let term = term.strip_suffix('"').unwrap_or(term);
        if term.is_empty() {
            continue;
        }
        if negated {
            query.has_operators = true;
            query.excluded.push(term.into());
        } else {
            query.terms.push(term.into());
        }
    }
    query
}

impl Query {
    fn apply(&mut self, key: Operator, value: &str) {
        match key {
            Operator::From => self.from.push(value.into()),
            Operator::To => self.to.push(value.into()),
            Operator::Has => self.has.extend(has(value)),
            Operator::Since => {
                self.since = self.since.max(date(value));
            }
            Operator::Until => {
                // Two bounds both apply, so the earlier `until` wins.
                self.until = match (self.until, date(value)) {
                    (Some(a), Some(b)) => Some(a.min(b)),
                    (a, b) => a.or(b),
                };
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Operator {
    From,
    To,
    Has,
    Since,
    Until,
}

/// Splits `key:value` for the five operators. The key ignores case; the value is not empty.
fn operator(token: &str) -> Option<(Operator, &str)> {
    let (key, value) = token.split_once(':')?;
    if value.is_empty() {
        return None;
    }
    let key = [
        ("from", Operator::From),
        ("to", Operator::To),
        ("has", Operator::Has),
        ("since", Operator::Since),
        ("until", Operator::Until),
    ]
    .into_iter()
    .find(|(name, _)| key.eq_ignore_ascii_case(name))?
    .1;
    Some((key, value))
}

fn has(value: &str) -> Option<Has> {
    Some(match value.to_lowercase().as_str() {
        "media" => Has::Media,
        "image" | "images" => Has::Image,
        "video" | "videos" => Has::Video,
        "poll" => Has::Poll,
        "link" | "links" => Has::Links,
        _ => return None,
    })
}

/// A calendar date written `YYYY-MM-DD`. An impossible date such as `2024-02-30` is ignored.
fn date(value: &str) -> Option<Date> {
    let b = value.as_bytes();
    let digits = |range: std::ops::Range<usize>| -> Option<u32> {
        let part = b.get(range)?;
        part.iter()
            .all(u8::is_ascii_digit)
            .then(|| part.iter().fold(0, |n, d| n * 10 + u32::from(d - b'0')))
    };
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let (year, month, day) = (digits(0..4)?, digits(5..7)?, digits(8..10)?);
    let month = Month::try_from(u8::try_from(month).ok()?).ok()?;
    Date::from_calendar_date(i32::try_from(year).ok()?, month, u8::try_from(day).ok()?).ok()
}

/// Splits on whitespace, keeping `"quoted phrases"` (with an optional leading `-`) whole.
fn tokens(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = text.trim_start_matches(is_js_space);
    while !rest.is_empty() {
        let len = quoted_len(rest).unwrap_or_else(|| rest.find(is_js_space).unwrap_or(rest.len()));
        out.push(&rest[..len]);
        rest = rest[len..].trim_start_matches(is_js_space);
    }
    out
}

/// The length of a leading `-"phrase"` or `"phrase"`: a quote, at least one non-quote character,
/// and a closing quote.
fn quoted_len(text: &str) -> Option<usize> {
    let open = usize::from(text.starts_with('-'));
    let body = text[open..].strip_prefix('"')?;
    let close = body.find('"').filter(|&n| n > 0)?;
    Some(open + 1 + close + 1)
}

/// Escapes `\`, `%` and `_` so the text matches literally inside a `LIKE '%...%'` pattern with
/// the default `\` escape.
#[must_use]
pub fn like_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(c, '\\' | '%' | '_') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod reference;

#[cfg(test)]
mod test {
    use super::*;

    fn date_text(d: Option<Date>) -> String {
        d.map_or_else(String::new, |d| d.to_string())
    }

    /// The query in the form of the recorded expectations.
    fn show(query: &Query) -> String {
        let list = |items: &[Box<str>]| items.iter().map(|s| &**s).collect::<Vec<_>>().join("|");
        let has: Vec<_> = query
            .has
            .iter()
            .map(|h| match h {
                Has::Media => "media",
                Has::Image => "image",
                Has::Video => "video",
                Has::Poll => "poll",
                Has::Links => "links",
            })
            .collect();
        format!(
            "terms={};ex={};from={};to={};has={};since={};until={};ops={}",
            list(&query.terms),
            list(&query.excluded),
            list(&query.from),
            list(&query.to),
            has.join("|"),
            date_text(query.since),
            date_text(query.until),
            u8::from(query.has_operators),
        )
    }

    #[test]
    fn parses_reference_queries() {
        let wrong: Vec<_> = reference::CASES
            .iter()
            .filter(|(text, expected)| show(&parse_query(text)) != *expected)
            .map(|(text, expected)| (text, expected, show(&parse_query(text))))
            .collect();
        assert!(wrong.is_empty(), "{wrong:#?}");
    }

    #[test]
    fn parses_every_operator() {
        let q =
            parse_query(r#"hello "a b" -no -"c d" from:ana to:@bo has:media has:poll has:links"#);
        assert_eq!(q.terms, ["hello".into(), "a b".into()]);
        assert_eq!(q.excluded, ["no".into(), "c d".into()]);
        assert_eq!(q.from, ["ana".into()]);
        assert_eq!(q.to, ["bo".into()]);
        assert_eq!(q.has, [Has::Media, Has::Poll, Has::Links]);
        let q = parse_query("since:2024-01-31 until:2024-02-01 has:image has:videos has:link");
        assert_eq!(
            q.since,
            Date::from_calendar_date(2024, Month::January, 31).ok()
        );
        assert_eq!(
            q.until,
            Date::from_calendar_date(2024, Month::February, 1).ok()
        );
        assert_eq!(q.has, [Has::Image, Has::Video, Has::Links]);
        assert!(q.has_operators);
    }

    #[test]
    fn plain_words_are_not_operators() {
        let q = parse_query("just some words");
        assert_eq!(q.terms, ["just".into(), "some".into(), "words".into()]);
        assert!(!q.has_operators);
    }

    #[test]
    fn operator_keys_ignore_case() {
        let q = parse_query("FROM:Ana Has:Media SINCE:2024-03-04");
        assert_eq!(q.from, ["Ana".into()]);
        assert_eq!(q.has, [Has::Media]);
        assert!(q.since.is_some());
    }

    #[test]
    fn unknown_operator_values_are_ignored_but_count() {
        let q = parse_query("has:cats since:soon until:2024-13-01 since:2023-02-29");
        assert_eq!(
            q,
            Query {
                has_operators: true,
                ..Query::default()
            }
        );
    }

    #[test]
    fn negated_operator_is_an_excluded_word() {
        let q = parse_query("-from:ana");
        assert_eq!(q.excluded, ["from:ana".into()]);
        assert!(q.from.is_empty());
    }

    #[test]
    fn bare_minus_and_empty_quotes() {
        assert_eq!(parse_query("-").terms, ["-".into()]);
        assert_eq!(parse_query(r#""""#), Query::default());
        assert_eq!(parse_query(r#"a "b"#).terms, ["a".into(), "b".into()]);
    }

    #[test]
    fn repeated_bounds_keep_the_tightest() {
        let q = parse_query("since:2024-01-01 since:2024-03-01 until:2024-05-01 until:2024-04-01");
        assert_eq!(
            q.since,
            Date::from_calendar_date(2024, Month::March, 1).ok()
        );
        assert_eq!(
            q.until,
            Date::from_calendar_date(2024, Month::April, 1).ok()
        );
    }

    #[test]
    fn cuts_long_input_to_100_characters() {
        let q = parse_query(&format!("{} tail", "a".repeat(98)));
        assert_eq!(q.terms.len(), 2);
        assert_eq!(
            q.terms[1].len(),
            1,
            "cut after one character of the second word"
        );
        assert_eq!(parse_query(&"é".repeat(150)).terms[0].chars().count(), 100);
    }

    #[test]
    fn empty_text_matches_everything() {
        assert_eq!(parse_query("  "), Query::default());
    }

    #[test]
    fn escapes_like_patterns() {
        assert_eq!(like_escape(r"100%_a\b"), r"100\%\_a\\b");
    }
}
