//! Number, date and handle formatting.
//!
//! The functions ending in `_in` are pure: they take the [`Locale`] and, where time matters, the
//! instants. The ones that take a `&Cx` read the request locale and are one-liners over them.
//!
//! Numbers and dates come from ICU4X, which the browser's `Intl` mirrors for the twelve shipped
//! languages. Dates are rendered in UTC: the server does not know the viewer's time zone.
//! `time.js` redraws every `<time data-rel>` in the browser's own zone once the page is live.

use fixed_decimal::{Decimal, SignedRoundingMode, UnsignedRoundingMode};
use icu_calendar::{Date, Gregorian};
use icu_datetime::{
    input::{DateTime, Time},
    pattern::{DateTimePattern, FixedCalendarDateTimeNames},
};
use icu_decimal::{CompactDecimalFormatter, DecimalFormatter, options::GroupingStrategy};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use topcoat::{
    Result,
    context::Cx,
    l10n::{Locale, locale},
    view::{View, component, view},
};
use writeable::TryWriteable;

use crate::l10n::{strip_isolates, t};

/// `1200` as `1.2K`, in the language of the request. See [`compact_in`].
#[must_use]
pub(crate) fn compact(cx: &Cx, n: i64) -> String {
    compact_in(locale(cx), n)
}

/// `1200` as `1.2K`: `Intl.NumberFormat(locale, { notation: 'compact', maximumFractionDigits: 1 })`.
///
/// Rounds half away from zero to one fraction digit of the compact figure and shows no trailing
/// zero, so `999_999` is `1M` and `12_345` is `12.3K`.
#[must_use]
pub(crate) fn compact_in(locale: &Locale, n: i64) -> String {
    let icu = locale.clone().into_icu();
    let Ok(formatter) = CompactDecimalFormatter::try_new_short((&icu).into(), Default::default())
    else {
        return n.to_string();
    };
    let half_expand = SignedRoundingMode::Unsigned(UnsignedRoundingMode::HalfExpand);
    let mut value = Decimal::from(n);
    // Rounding can carry into the next unit (`999_999` to `1_000_000`), which changes the exponent
    // and so the digit that is kept; the second pass settles that.
    for _ in 0..2 {
        let exponent = formatter.compact_exponent_for_magnitude(value.nonzero_magnitude_start());
        let rounded = value
            .clone()
            .rounded_with_mode(i16::from(exponent) - 1, half_expand);
        let settled =
            formatter.compact_exponent_for_magnitude(rounded.nonzero_magnitude_start()) == exponent;
        value = rounded;
        if settled {
            break;
        }
    }
    let exponent = formatter.compact_exponent_for_magnitude(value.nonzero_magnitude_start());
    if exponent == 0 {
        // Below the first compact unit the number only gets its grouping separators.
        return match DecimalFormatter::try_new((&icu).into(), GroupingStrategy::Min2.into()) {
            Ok(plain) => {
                value.absolute.trim_end();
                plain.format(&value).to_string()
            }
            Err(_) => n.to_string(),
        };
    }
    value.multiply_pow10(-i16::from(exponent));
    value.absolute.trim_end();
    match formatter.format_with_exponent(&value, exponent) {
        Ok(shown) => shown.to_string(),
        Err(_) => n.to_string(),
    }
}

/// The instant of an RFC 3339 timestamp, or `None` when `ts` is not one.
#[must_use]
pub(crate) fn parse_ts(ts: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(ts, &Rfc3339).ok()
}

/// `now`, `5m`, `3h`, `2d`, then a short date, for the timestamp `ts` (RFC 3339).
///
/// A `ts` that does not parse is returned as it is.
#[must_use]
pub(crate) fn rel_label(cx: &Cx, ts: &str) -> String {
    match parse_ts(ts) {
        Some(at) => rel_label_in(locale(cx), at, OffsetDateTime::now_utc()),
        None => ts.to_owned(),
    }
}

/// [`rel_label`] for the instant `at`, seen from `now`: the same steps as the page's relative-time label
/// (under 45 s is `now`, then minutes, hours and days up to a week, then a date with the year only
/// when it is not the current one).
#[must_use]
pub(crate) fn rel_label_in(locale: &Locale, at: OffsetDateTime, now: OffsetDateTime) -> String {
    let seconds = (now - at).as_seconds_f64();
    let unit = |n: f64| n.floor() as i64;
    let text = if seconds < 45.0 {
        t!(time_now).resolve_in(locale)
    } else if seconds < 3600.0 {
        t!(time_minutes, n: unit(seconds / 60.0)).resolve_in(locale)
    } else if seconds < 86_400.0 {
        t!(time_hours, n: unit(seconds / 3600.0)).resolve_in(locale)
    } else if seconds < 7.0 * 86_400.0 {
        t!(time_days, n: unit(seconds / 86_400.0)).resolve_in(locale)
    } else {
        return short_date_in(locale, at, at.year() != now.year());
    };
    strip_isolates(&text)
}

/// The relative time as `<time datetime="<ts>" data-rel title="<full time>">text</time>`.
/// `time.js` keeps the text current.
///
/// A `ts` that does not parse renders as plain text without `datetime` or `title`, which
/// `time.js` leaves alone.
#[component]
pub(crate) async fn rel_time(cx: &Cx, ts: &str) -> Result<impl View> {
    let locale = locale(cx);
    let at = parse_ts(ts);
    let text = at.map_or_else(
        || ts.to_owned(),
        |at| rel_label_in(locale, at, OffsetDateTime::now_utc()),
    );
    let title = at.map(|at| full_time_in(locale, at));
    let datetime = at.map(|_| ts);
    Ok(view! { <time datetime=(datetime) data-rel="" title=(title)>(text)</time> })
}

/// `Nov 12, 2026, 3:04 PM`, in the language of the request. See [`full_time_in`].
#[must_use]
pub(crate) fn full_time(cx: &Cx, ts: &str) -> String {
    parse_ts(ts).map_or_else(|| ts.to_owned(), |at| full_time_in(locale(cx), at))
}

/// The date and time of `at` (UTC): `toLocaleString(locale, { hour: 'numeric', minute: '2-digit',
/// month: 'short', day: 'numeric', year: 'numeric' })`.
#[must_use]
pub(crate) fn full_time_in(locale: &Locale, at: OffsetDateTime) -> String {
    render(locale, |p| p.full, at)
}

/// `Nov 12` (`Nov 12, 2025` with `with_year`): `toLocaleDateString` with a short month and a day.
#[must_use]
pub(crate) fn short_date_in(locale: &Locale, at: OffsetDateTime, with_year: bool) -> String {
    if with_year {
        render(locale, |p| p.short_year, at)
    } else {
        render(locale, |p| p.short, at)
    }
}

/// `November 2026`, in the language of the request. See [`month_year_in`].
#[must_use]
pub(crate) fn month_year(cx: &Cx, ts: &str) -> String {
    parse_ts(ts).map_or_else(|| ts.to_owned(), |at| month_year_in(locale(cx), at))
}

/// The long month and the year of `at` (UTC): `toLocaleDateString(locale, { month: 'long', year:
/// 'numeric' })`.
#[must_use]
pub(crate) fn month_year_in(locale: &Locale, at: OffsetDateTime) -> String {
    render(locale, |p| p.month_year, at)
}

/// UTS 35 patterns that give what a browser's `Intl.DateTimeFormat` gives for one language.
///
/// ICU4X formats by length (`short`, `medium`, `long`) only, and for some languages `medium` is
/// numeric (`12.11.2026` in German), which is not what `{ month: 'short' }` gives. `hour:
/// 'numeric'` is a bare hour in 24-hour languages here (`9:07`), where ICU4X pads it. Pattern text
/// uses plain spaces because browsers turn U+202F into one. `LLLL` is the stand-alone month name
/// (`listopad`, not `listopada`).
struct Patterns {
    /// Hour, minute and the date with a short month, day and year.
    full: &'static str,
    /// Short month and day.
    short: &'static str,
    /// Short month, day and year.
    short_year: &'static str,
    /// Long month and year.
    month_year: &'static str,
}

const ENGLISH: Patterns = Patterns {
    full: "MMM d, y, h:mm a",
    short: "MMM d",
    short_year: "MMM d, y",
    month_year: "LLLL y",
};

/// The patterns of the shipped languages, by language code. Anything else reads as English.
const PATTERNS: [(&str, Patterns); 11] = [
    (
        "pl",
        Patterns {
            full: "d MMM y, H:mm",
            short: "d MMM",
            short_year: "d MMM y",
            month_year: "LLLL y",
        },
    ),
    (
        "de",
        Patterns {
            full: "d. MMM y, H:mm",
            short: "d. MMM",
            short_year: "d. MMM y",
            month_year: "LLLL y",
        },
    ),
    (
        "es",
        Patterns {
            full: "d MMM y, H:mm",
            short: "d MMM",
            short_year: "d MMM y",
            month_year: "LLLL 'de' y",
        },
    ),
    (
        "fr",
        Patterns {
            full: "d MMM y, H:mm",
            short: "d MMM",
            short_year: "d MMM y",
            month_year: "LLLL y",
        },
    ),
    (
        "pt",
        Patterns {
            full: "d 'de' MMM 'de' y, H:mm",
            short: "d 'de' MMM",
            short_year: "d 'de' MMM 'de' y",
            month_year: "LLLL 'de' y",
        },
    ),
    (
        "uk",
        Patterns {
            full: "d MMM y 'р'., H:mm",
            short: "d MMM",
            short_year: "d MMM y 'р'.",
            month_year: "LLLL y 'р'.",
        },
    ),
    (
        "it",
        Patterns {
            full: "d MMM y, H:mm",
            short: "d MMM",
            short_year: "d MMM y",
            month_year: "LLLL y",
        },
    ),
    (
        "nl",
        Patterns {
            full: "d MMM y, H:mm",
            short: "d MMM",
            short_year: "d MMM y",
            month_year: "LLLL y",
        },
    ),
    (
        "tr",
        Patterns {
            full: "d MMM y H:mm",
            short: "d MMM",
            short_year: "d MMM y",
            month_year: "LLLL y",
        },
    ),
    (
        "cs",
        Patterns {
            full: "d. M. y H:mm",
            short: "d. M.",
            short_year: "d. M. y",
            month_year: "LLLL y",
        },
    ),
    (
        "ja",
        Patterns {
            full: "y年M月d日 H:mm",
            short: "M月d日",
            short_year: "y年M月d日",
            month_year: "y年M月",
        },
    ),
];

/// `at` (UTC) in the pattern `pick` takes from the language's [`Patterns`]. Falls back to the
/// RFC 3339 text when ICU has no data for the locale.
fn render(locale: &Locale, pick: impl Fn(&Patterns) -> &'static str, at: OffsetDateTime) -> String {
    let language = locale.id().language();
    let patterns = PATTERNS
        .iter()
        .find(|(code, _)| language.as_str() == *code)
        .map_or(&ENGLISH, |(_, patterns)| patterns);
    format_pattern(locale, pick(patterns), at)
        .unwrap_or_else(|| at.format(&Rfc3339).unwrap_or_default())
}

pub(crate) fn format_pattern(locale: &Locale, pattern: &str, at: OffsetDateTime) -> Option<String> {
    let icu = locale.clone().into_icu();
    let mut names = FixedCalendarDateTimeNames::<Gregorian>::try_new((&icu).into()).ok()?;
    let pattern: DateTimePattern = pattern.parse().ok()?;
    let input = icu_date_time(at)?;
    let shown = names.include_for_pattern(&pattern).ok()?.format(&input);
    Some(match shown.try_write_to_string() {
        Ok(text) | Err((_, text)) => text.into_owned(),
    })
}

fn icu_date_time(at: OffsetDateTime) -> Option<DateTime<Gregorian>> {
    let date = Date::try_new_gregorian(at.year(), u8::from(at.month()), at.day()).ok()?;
    let time = Time::try_new(at.hour(), at.minute(), at.second(), 0).ok()?;
    Some(DateTime { date, time })
}

/// `@ada`, or `@ada@remote.example` for remote accounts.
#[must_use]
pub(crate) fn handle_of(handle: &str, domain: Option<&str>) -> String {
    match domain {
        Some(domain) if !domain.is_empty() => format!("@{handle}@{domain}"),
        _ => format!("@{handle}"),
    }
}

/// `ada`, or `ada@remote.example`: how API paths and message routes name an account.
#[must_use]
pub(crate) fn acct_of(handle: &str, domain: Option<&str>) -> String {
    match domain {
        Some(domain) if !domain.is_empty() => format!("{handle}@{domain}"),
        _ => handle.to_owned(),
    }
}

/// The host of an `http(s)` URL, with its port, or an empty string when it is not one.
#[must_use]
pub(crate) fn host_of(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .filter(|parsed| matches!(parsed.scheme(), "http" | "https"))
        .and_then(|parsed| {
            let host = parsed.host_str()?;
            Some(match parsed.port() {
                Some(port) => format!("{host}:{port}"),
                None => host.to_owned(),
            })
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod test {
    use super::*;
    use topcoat::view::ViewExt;

    use crate::locale::locale_of;

    /// What `Intl.NumberFormat(locale, { notation: 'compact', maximumFractionDigits: 1 })` gives
    /// in Chromium's ICU for `0 5 999 1000 1200 9999 12345 99999 999999 1234567 12345678 999999999
    /// 1500000000` (generated with Node 24, ICU 78).
    const COMPACT: [(&str, [&str; 13]); 12] = [
        (
            "en",
            [
                "0", "5", "999", "1K", "1.2K", "10K", "12.3K", "100K", "1M", "1.2M", "12.3M", "1B",
                "1.5B",
            ],
        ),
        (
            "pl",
            [
                "0",
                "5",
                "999",
                "1\u{a0}tys.",
                "1,2\u{a0}tys.",
                "10\u{a0}tys.",
                "12,3\u{a0}tys.",
                "100\u{a0}tys.",
                "1\u{a0}mln",
                "1,2\u{a0}mln",
                "12,3\u{a0}mln",
                "1\u{a0}mld",
                "1,5\u{a0}mld",
            ],
        ),
        (
            "de",
            [
                "0",
                "5",
                "999",
                "1000",
                "1200",
                "9999",
                "12.345",
                "99.999",
                "999.999",
                "1,2\u{a0}Mio.",
                "12,3\u{a0}Mio.",
                "1\u{a0}Mrd.",
                "1,5\u{a0}Mrd.",
            ],
        ),
        (
            "es",
            [
                "0",
                "5",
                "999",
                "1\u{a0}mil",
                "1,2\u{a0}mil",
                "10\u{a0}mil",
                "12,3\u{a0}mil",
                "100\u{a0}mil",
                "1\u{a0}M",
                "1,2\u{a0}M",
                "12,3\u{a0}M",
                "1000\u{a0}M",
                "1500\u{a0}M",
            ],
        ),
        (
            "fr",
            [
                "0",
                "5",
                "999",
                "1\u{a0}k",
                "1,2\u{a0}k",
                "10\u{a0}k",
                "12,3\u{a0}k",
                "100\u{a0}k",
                "1\u{a0}M",
                "1,2\u{a0}M",
                "12,3\u{a0}M",
                "1\u{a0}Md",
                "1,5\u{a0}Md",
            ],
        ),
        (
            "pt",
            [
                "0",
                "5",
                "999",
                "1\u{a0}mil",
                "1,2\u{a0}mil",
                "10\u{a0}mil",
                "12,3\u{a0}mil",
                "100\u{a0}mil",
                "1\u{a0}mi",
                "1,2\u{a0}mi",
                "12,3\u{a0}mi",
                "1\u{a0}bi",
                "1,5\u{a0}bi",
            ],
        ),
        (
            "uk",
            [
                "0",
                "5",
                "999",
                "1\u{a0}тис.",
                "1,2\u{a0}тис.",
                "10\u{a0}тис.",
                "12,3\u{a0}тис.",
                "100\u{a0}тис.",
                "1\u{a0}млн",
                "1,2\u{a0}млн",
                "12,3\u{a0}млн",
                "1\u{a0}млрд",
                "1,5\u{a0}млрд",
            ],
        ),
        (
            "it",
            [
                "0",
                "5",
                "999",
                "1K",
                "1,2K",
                "10K",
                "12,3K",
                "100K",
                "1\u{a0}Mln",
                "1,2\u{a0}Mln",
                "12,3\u{a0}Mln",
                "1\u{a0}Mld",
                "1,5\u{a0}Mld",
            ],
        ),
        (
            "nl",
            [
                "0",
                "5",
                "999",
                "1K",
                "1,2K",
                "10K",
                "12,3K",
                "100K",
                "1\u{a0}mln.",
                "1,2\u{a0}mln.",
                "12,3\u{a0}mln.",
                "1\u{a0}mld.",
                "1,5\u{a0}mld.",
            ],
        ),
        (
            "tr",
            [
                "0",
                "5",
                "999",
                "1\u{a0}B",
                "1,2\u{a0}B",
                "10\u{a0}B",
                "12,3\u{a0}B",
                "100\u{a0}B",
                "1\u{a0}Mn",
                "1,2\u{a0}Mn",
                "12,3\u{a0}Mn",
                "1\u{a0}Mr",
                "1,5\u{a0}Mr",
            ],
        ),
        (
            "cs",
            [
                "0",
                "5",
                "999",
                "1\u{a0}tis.",
                "1,2\u{a0}tis.",
                "10\u{a0}tis.",
                "12,3\u{a0}tis.",
                "100\u{a0}tis.",
                "1\u{a0}mil.",
                "1,2\u{a0}mil.",
                "12,3\u{a0}mil.",
                "1\u{a0}mld.",
                "1,5\u{a0}mld.",
            ],
        ),
        (
            "ja",
            [
                "0",
                "5",
                "999",
                "1000",
                "1200",
                "9999",
                "1.2万",
                "10万",
                "100万",
                "123.5万",
                "1234.6万",
                "10億",
                "15億",
            ],
        ),
    ];
    const COMPACT_INPUTS: [i64; 13] = [
        0,
        5,
        999,
        1000,
        1200,
        9999,
        12_345,
        99_999,
        999_999,
        1_234_567,
        12_345_678,
        999_999_999,
        1_500_000_000,
    ];

    /// `toLocaleString` / `toLocaleDateString` in UTC for the instants below: full time, short
    /// month and day, short month day and year, long month and year.
    const DATES: [(&str, &str, &str, &str, &str, &str); 36] = [
        (
            "en",
            "2026-11-12T15:04:05Z",
            "Nov 12, 2026, 3:04 PM",
            "Nov 12",
            "Nov 12, 2026",
            "November 2026",
        ),
        (
            "en",
            "2026-03-05T09:07:00Z",
            "Mar 5, 2026, 9:07 AM",
            "Mar 5",
            "Mar 5, 2026",
            "March 2026",
        ),
        (
            "en",
            "2026-01-01T00:00:00Z",
            "Jan 1, 2026, 12:00 AM",
            "Jan 1",
            "Jan 1, 2026",
            "January 2026",
        ),
        (
            "pl",
            "2026-11-12T15:04:05Z",
            "12 lis 2026, 15:04",
            "12 lis",
            "12 lis 2026",
            "listopad 2026",
        ),
        (
            "pl",
            "2026-03-05T09:07:00Z",
            "5 mar 2026, 9:07",
            "5 mar",
            "5 mar 2026",
            "marzec 2026",
        ),
        (
            "pl",
            "2026-01-01T00:00:00Z",
            "1 sty 2026, 0:00",
            "1 sty",
            "1 sty 2026",
            "styczeń 2026",
        ),
        (
            "de",
            "2026-11-12T15:04:05Z",
            "12. Nov. 2026, 15:04",
            "12. Nov.",
            "12. Nov. 2026",
            "November 2026",
        ),
        (
            "de",
            "2026-03-05T09:07:00Z",
            "5. März 2026, 9:07",
            "5. März",
            "5. März 2026",
            "März 2026",
        ),
        (
            "de",
            "2026-01-01T00:00:00Z",
            "1. Jan. 2026, 0:00",
            "1. Jan.",
            "1. Jan. 2026",
            "Januar 2026",
        ),
        (
            "es",
            "2026-11-12T15:04:05Z",
            "12 nov 2026, 15:04",
            "12 nov",
            "12 nov 2026",
            "noviembre de 2026",
        ),
        (
            "es",
            "2026-03-05T09:07:00Z",
            "5 mar 2026, 9:07",
            "5 mar",
            "5 mar 2026",
            "marzo de 2026",
        ),
        (
            "es",
            "2026-01-01T00:00:00Z",
            "1 ene 2026, 0:00",
            "1 ene",
            "1 ene 2026",
            "enero de 2026",
        ),
        (
            "fr",
            "2026-11-12T15:04:05Z",
            "12 nov. 2026, 15:04",
            "12 nov.",
            "12 nov. 2026",
            "novembre 2026",
        ),
        (
            "fr",
            "2026-03-05T09:07:00Z",
            "5 mars 2026, 9:07",
            "5 mars",
            "5 mars 2026",
            "mars 2026",
        ),
        (
            "fr",
            "2026-01-01T00:00:00Z",
            "1 janv. 2026, 0:00",
            "1 janv.",
            "1 janv. 2026",
            "janvier 2026",
        ),
        (
            "pt",
            "2026-11-12T15:04:05Z",
            "12 de nov. de 2026, 15:04",
            "12 de nov.",
            "12 de nov. de 2026",
            "novembro de 2026",
        ),
        (
            "pt",
            "2026-03-05T09:07:00Z",
            "5 de mar. de 2026, 9:07",
            "5 de mar.",
            "5 de mar. de 2026",
            "março de 2026",
        ),
        (
            "pt",
            "2026-01-01T00:00:00Z",
            "1 de jan. de 2026, 0:00",
            "1 de jan.",
            "1 de jan. de 2026",
            "janeiro de 2026",
        ),
        (
            "uk",
            "2026-11-12T15:04:05Z",
            "12 лист. 2026 р., 15:04",
            "12 лист.",
            "12 лист. 2026 р.",
            "листопад 2026 р.",
        ),
        (
            "uk",
            "2026-03-05T09:07:00Z",
            "5 бер. 2026 р., 9:07",
            "5 бер.",
            "5 бер. 2026 р.",
            "березень 2026 р.",
        ),
        (
            "uk",
            "2026-01-01T00:00:00Z",
            "1 січ. 2026 р., 0:00",
            "1 січ.",
            "1 січ. 2026 р.",
            "січень 2026 р.",
        ),
        (
            "it",
            "2026-11-12T15:04:05Z",
            "12 nov 2026, 15:04",
            "12 nov",
            "12 nov 2026",
            "novembre 2026",
        ),
        (
            "it",
            "2026-03-05T09:07:00Z",
            "5 mar 2026, 9:07",
            "5 mar",
            "5 mar 2026",
            "marzo 2026",
        ),
        (
            "it",
            "2026-01-01T00:00:00Z",
            "1 gen 2026, 0:00",
            "1 gen",
            "1 gen 2026",
            "gennaio 2026",
        ),
        (
            "nl",
            "2026-11-12T15:04:05Z",
            "12 nov 2026, 15:04",
            "12 nov",
            "12 nov 2026",
            "november 2026",
        ),
        (
            "nl",
            "2026-03-05T09:07:00Z",
            "5 mrt 2026, 9:07",
            "5 mrt",
            "5 mrt 2026",
            "maart 2026",
        ),
        (
            "nl",
            "2026-01-01T00:00:00Z",
            "1 jan 2026, 0:00",
            "1 jan",
            "1 jan 2026",
            "januari 2026",
        ),
        (
            "tr",
            "2026-11-12T15:04:05Z",
            "12 Kas 2026 15:04",
            "12 Kas",
            "12 Kas 2026",
            "Kasım 2026",
        ),
        (
            "tr",
            "2026-03-05T09:07:00Z",
            "5 Mar 2026 9:07",
            "5 Mar",
            "5 Mar 2026",
            "Mart 2026",
        ),
        (
            "tr",
            "2026-01-01T00:00:00Z",
            "1 Oca 2026 0:00",
            "1 Oca",
            "1 Oca 2026",
            "Ocak 2026",
        ),
        (
            "cs",
            "2026-11-12T15:04:05Z",
            "12. 11. 2026 15:04",
            "12. 11.",
            "12. 11. 2026",
            "listopad 2026",
        ),
        (
            "cs",
            "2026-03-05T09:07:00Z",
            "5. 3. 2026 9:07",
            "5. 3.",
            "5. 3. 2026",
            "březen 2026",
        ),
        (
            "cs",
            "2026-01-01T00:00:00Z",
            "1. 1. 2026 0:00",
            "1. 1.",
            "1. 1. 2026",
            "leden 2026",
        ),
        (
            "ja",
            "2026-11-12T15:04:05Z",
            "2026年11月12日 15:04",
            "11月12日",
            "2026年11月12日",
            "2026年11月",
        ),
        (
            "ja",
            "2026-03-05T09:07:00Z",
            "2026年3月5日 9:07",
            "3月5日",
            "2026年3月5日",
            "2026年3月",
        ),
        (
            "ja",
            "2026-01-01T00:00:00Z",
            "2026年1月1日 0:00",
            "1月1日",
            "2026年1月1日",
            "2026年1月",
        ),
    ];

    #[test]
    fn compact_numbers_match_intl_in_every_language() {
        let mut wrong = Vec::new();
        for (code, expected) in COMPACT {
            let locale = locale_of(code);
            for (input, want) in COMPACT_INPUTS.iter().zip(expected) {
                let got = compact_in(&locale, *input);
                if got != want {
                    wrong.push(format!("{code} {input}: {got:?}, Intl {want:?}"));
                }
            }
        }
        assert!(wrong.is_empty(), "{wrong:#?}");
    }

    #[test]
    fn dates_match_intl_in_every_language() {
        let mut wrong = Vec::new();
        for (code, iso, full, short, with_year, month_year) in DATES {
            let locale = locale_of(code);
            let at = parse_ts(iso).expect("rfc 3339");
            for (what, got, want) in [
                ("full", full_time_in(&locale, at), full),
                ("short", short_date_in(&locale, at, false), short),
                (
                    "short with year",
                    short_date_in(&locale, at, true),
                    with_year,
                ),
                ("month and year", month_year_in(&locale, at), month_year),
            ] {
                if got != want {
                    wrong.push(format!("{code} {iso} {what}: {got:?}, Intl {want:?}"));
                }
            }
        }
        assert!(wrong.is_empty(), "{wrong:#?}");
    }

    #[test]
    fn handles_and_accts_name_local_and_remote_accounts() {
        assert_eq!(handle_of("ada", None), "@ada");
        assert_eq!(handle_of("bo", Some("x.example")), "@bo@x.example");
        assert_eq!(acct_of("ada", None), "ada");
        assert_eq!(acct_of("bo", Some("x.example")), "bo@x.example");
        assert_eq!(acct_of("bo", Some("x.example:8080")), "bo@x.example:8080");
    }

    #[test]
    fn host_of_keeps_the_port_and_refuses_other_schemes() {
        assert_eq!(host_of("https://x.example:8080/@bo/1"), "x.example:8080");
        assert_eq!(host_of("http://x.example/a"), "x.example");
        assert_eq!(host_of("javascript:alert(1)"), "");
        assert_eq!(host_of("not a url"), "");
        assert_eq!(host_of(""), "");
    }

    fn label(code: &str, at: &str, now: &str) -> String {
        rel_label_in(
            &locale_of(code),
            parse_ts(at).expect("rfc 3339"),
            parse_ts(now).expect("rfc 3339"),
        )
    }

    #[test]
    fn recent_times_are_shortened() {
        let now = "2026-11-09T12:00:00Z";
        assert_eq!(label("en", "2026-11-09T11:59:30Z", now), "now");
        assert_eq!(label("en", "2026-11-09T11:55:00Z", now), "5m");
        assert_eq!(label("en", "2026-11-09T09:00:00Z", now), "3h");
        assert_eq!(label("en", "2026-11-07T12:00:00Z", now), "2d");
        assert_eq!(label("de", "2026-11-09T11:55:00Z", now), "5 Min.");
        assert_eq!(label("ja", "2026-11-09T11:55:00Z", now), "5分");
    }

    #[test]
    fn the_steps_change_at_their_thresholds() {
        let now = "2026-11-09T12:00:00Z";
        assert_eq!(label("en", "2026-11-09T11:59:16Z", now), "now");
        assert_eq!(label("en", "2026-11-09T11:59:15Z", now), "0m");
        assert_eq!(label("en", "2026-11-09T11:00:01Z", now), "59m");
        assert_eq!(label("en", "2026-11-09T11:00:00Z", now), "1h");
        assert_eq!(label("en", "2026-11-08T12:00:01Z", now), "23h");
        assert_eq!(label("en", "2026-11-08T12:00:00Z", now), "1d");
        assert_eq!(label("en", "2026-11-02T12:00:01Z", now), "6d");
    }

    #[test]
    fn a_week_or_more_gets_a_date_with_the_year_only_when_it_is_not_this_one() {
        let now = "2026-11-09T12:00:00Z";
        assert_eq!(label("en", "2026-11-02T12:00:00Z", now), "Nov 2");
        assert_eq!(label("en", "2025-11-02T12:00:00Z", now), "Nov 2, 2025");
        assert_eq!(label("de", "2026-11-02T12:00:00Z", now), "2. Nov.");
        assert_eq!(label("de", "2025-11-02T12:00:00Z", now), "2. Nov. 2025");
        assert_eq!(label("ja", "2026-11-02T12:00:00Z", now), "11月2日");
        assert_eq!(label("ja", "2025-11-02T12:00:00Z", now), "2025年11月2日");
    }

    #[test]
    fn a_time_in_the_future_reads_now() {
        assert_eq!(
            label("en", "2026-11-09T12:05:00Z", "2026-11-09T12:00:00Z"),
            "now"
        );
    }

    #[test]
    fn every_language_formats_every_month_without_panicking() {
        for code in tuita_core::locale::LANGS {
            let locale = locale_of(code);
            for month in 1..=12 {
                let at = parse_ts(&format!("2026-{month:02}-15T08:30:00Z")).expect("rfc 3339");
                let shown = [
                    month_year_in(&locale, at),
                    full_time_in(&locale, at),
                    short_date_in(&locale, at, true),
                    short_date_in(&locale, at, false),
                ];
                for text in shown {
                    assert!(
                        !text.contains("2026-"),
                        "{code} month {month}: {text:?} fell back to RFC 3339"
                    );
                }
            }
        }
    }

    #[test]
    fn a_language_without_patterns_uses_the_english_layout() {
        let locale = locale_of("zh");
        let at = parse_ts("2026-11-12T15:04:05Z").expect("rfc 3339");
        assert_eq!(short_date_in(&locale, at, true), "11月 12, 2026");
    }

    #[test]
    fn negative_and_zero_counts_are_formatted_too() {
        let en = locale_of("en");
        assert_eq!(compact_in(&en, -1200), "-1.2K");
        assert_eq!(compact_in(&en, 0), "0");
    }

    #[test]
    fn a_bad_timestamp_is_not_a_time() {
        assert!(parse_ts("yesterday").is_none());
        assert!(parse_ts("2026-11-12").is_none());
    }

    #[test]
    fn the_component_renders_a_time_element_the_island_keeps_current() {
        let html = crate::testing::render("en", |cx| {
            view! { rel_time(ts: "2020-01-02T03:04:05Z") }.boxed()
        });
        assert_eq!(
            html,
            r#"<time datetime="2020-01-02T03:04:05Z" data-rel="" title="Jan 2, 2020, 3:04 AM">Jan 2, 2020</time>"#
        );
    }

    #[test]
    fn the_component_leaves_a_bad_timestamp_as_plain_text() {
        let html = crate::testing::render("en", |cx| view! { rel_time(ts: "yesterday") }.boxed());
        assert_eq!(html, r#"<time data-rel="">yesterday</time>"#);
    }
}
