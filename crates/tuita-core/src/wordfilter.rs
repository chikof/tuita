//! Content filter for handles, names, bios, posts and list names. It blocks only the words an
//! admin added; nothing is blocked out of the box.
//!
//! A blocked term matches anywhere inside a word. Text is normalised first: lower case, accents
//! stripped, look-alike letters mapped, leetspeak decoded (`g1zm0`), `CamelCase` split, and runs
//! of one or two letter fragments joined (`g.i.z`). An allow list names the innocent words a
//! blocked term would otherwise catch.
//!
//! ```
//! use tuita_core::wordfilter::{FilterConfig, Target, find_blocked};
//!
//! assert!(find_blocked("g1zm0", Target::Text, &FilterConfig::default()).is_none());
//! let config = FilterConfig::from_settings("gizmo", "", "");
//! let hit = find_blocked("g1zm0", Target::Text, &config).expect("blocked");
//! assert_eq!(hit.term, "gizmo");
//! ```

use unicode_normalization::UnicodeNormalization;

/// What kind of string is being checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Target {
    /// A name, bio, post or list name.
    #[default]
    Text,
    /// A handle. Also checks every fragment squashed together, which catches `gi_zmo`. Display
    /// names skip this, where "Gi Zmo" would otherwise read as "gizmo".
    Handle,
}

/// What the admin set: blocked terms, allowed exceptions and reserved handles. Terms are lower
/// case.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterConfig {
    /// Blocked terms. A word that contains one (letters and digits only) is blocked.
    pub extra_blocked: Vec<Box<str>>,
    /// Innocent words, exempt from every blocked term.
    pub extra_allowed: Vec<Box<str>>,
    /// Reserved handles, without a leading `@`.
    pub extra_reserved: Vec<Box<str>>,
}

impl FilterConfig {
    /// Builds a config from the three free-text admin settings: one term per line or comma
    /// separated, trimmed and lower-cased. Empty entries are dropped.
    #[must_use]
    pub fn from_settings(blocked: &str, allowed: &str, reserved: &str) -> Self {
        Self {
            extra_blocked: entries(blocked, false),
            extra_allowed: entries(allowed, false),
            extra_reserved: entries(reserved, true),
        }
    }
}

fn entries(text: &str, strip_at: bool) -> Vec<Box<str>> {
    text.split(['\n', ','])
        .filter_map(|entry| {
            let entry = entry.trim().to_lowercase();
            let entry = if strip_at {
                entry.strip_prefix('@').unwrap_or(&entry)
            } else {
                &entry
            };
            (!entry.is_empty()).then(|| entry.into())
        })
        .collect()
}

/// The first match found by [`find_blocked`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit<'a> {
    /// The blocked term that matched.
    pub term: &'a str,
}

/// Handles nobody can register: system accounts and role names. Not a word blacklist.
const RESERVED: &[&str] = &[
    "admin",
    "administrator",
    "admins",
    "root",
    "system",
    "sysadmin",
    "moderator",
    "moderators",
    "mod",
    "mods",
    "staff",
    "support",
    "help",
    "helpdesk",
    "security",
    "abuse",
    "official",
    "team",
    "owner",
    "operator",
    "webmaster",
    "postmaster",
    "hostmaster",
    "noreply",
    "info",
    "contact",
    "legal",
    "privacy",
    "terms",
    "api",
    "www",
    "mail",
    "email",
    "null",
    "undefined",
    "anonymous",
    "everyone",
    "here",
    "all",
    "deleted",
    "suspended",
    "verified",
    "trust",
    "safety",
    "trustandsafety",
    "twitter",
    "x",
    "tuita",
];

/// Returns the first blocked term in `text`, or `None` when it is clean or no term is set.
#[must_use]
pub fn find_blocked<'a>(text: &str, target: Target, config: &'a FilterConfig) -> Option<Hit<'a>> {
    if text.is_empty() {
        return None;
    }
    // A term with no letters or digits would match every word; skip it.
    let custom: Vec<(String, &str)> = config
        .extra_blocked
        .iter()
        .map(|term| {
            let needle = term
                .chars()
                .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
            (needle.collect::<String>(), &**term)
        })
        .filter(|(needle, _)| !needle.is_empty())
        .collect();
    if custom.is_empty() {
        return None;
    }
    let mut seen: Vec<String> = Vec::new();
    for leet in [false, true] {
        for word in words(text, leet, target == Target::Handle) {
            if seen.contains(&word) {
                continue;
            }
            if !config.extra_allowed.iter().any(|a| **a == *word)
                && let Some((_, term)) = custom.iter().find(|(needle, _)| word.contains(needle))
            {
                return Some(Hit { term });
            }
            seen.push(word);
        }
    }
    None
}

/// Reports whether nobody may register `handle`: a built-in or admin-reserved word, compared
/// without case, digits and underscores (`Admin_01` is reserved).
#[must_use]
pub fn is_reserved_handle(handle: &str, config: &FilterConfig) -> bool {
    let handle = handle.to_lowercase();
    let bare: String = handle
        .chars()
        .filter(|c| !c.is_ascii_digit() && *c != '_')
        .collect();
    let reserved =
        |h: &str| RESERVED.contains(&h) || config.extra_reserved.iter().any(|r| **r == *h);
    reserved(&handle) || reserved(&bare)
}

// ---------------------------------------------------------------- normalisation

/// Maps one non-ASCII character that looks like a Latin letter to that letter.
const fn homoglyph(c: char) -> Option<&'static str> {
    Some(match c {
        '\u{430}' | '\u{3b1}' => "a",
        '\u{435}' => "e",
        '\u{43e}' | '\u{3bf}' => "o",
        '\u{440}' => "p",
        '\u{441}' | '\u{3f2}' => "c",
        '\u{443}' => "y",
        '\u{445}' => "x",
        '\u{456}' | '\u{131}' => "i",
        '\u{458}' => "j",
        '\u{455}' => "s",
        '\u{501}' => "d",
        '\u{261}' => "g",
        '\u{2113}' => "l",
        '\u{3bd}' => "v",
        'κ' => "k", // Greek kappa, a literal: its escape trips the spell checker
        '\u{3c4}' => "t",
        '\u{df}' => "ss",
        _ => return None,
    })
}

/// Maps a digit or symbol to the letter it stands in for (`4` is `a`, `$` is `s`).
const fn leet(c: char) -> Option<char> {
    Some(match c {
        '0' => 'o',
        '1' | '!' | '|' => 'i',
        '2' => 'z',
        '3' | '\u{20ac}' => 'e',
        '4' | '@' => 'a',
        '5' | '$' => 's',
        '6' | '9' => 'g',
        '7' | '+' => 't',
        '8' => 'b',
        '\u{a3}' => 'l',
        _ => return None,
    })
}

/// NFKD, accents and zero-width characters removed, look-alike letters mapped to Latin.
fn base_normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.nfkd() {
        let accent = ('\u{300}'..='\u{36f}').contains(&c);
        let zero_width = matches!(
            c,
            '\u{200b}'..='\u{200f}' | '\u{2060}' | '\u{feff}' | '\u{ad}'
        );
        if accent || zero_width {
            continue;
        }
        let mapped = if c.is_ascii() {
            None
        } else {
            single_lowercase(c).and_then(homoglyph)
        };
        match mapped {
            Some(latin) => out.push_str(latin),
            None => out.push(c),
        }
    }
    out
}

fn single_lowercase(c: char) -> Option<char> {
    let mut lower = c.to_lowercase();
    let first = lower.next()?;
    lower.next().is_none().then_some(first)
}

/// The words of a text: `CamelCase` split, optionally leetspeak decoded, split on anything that
/// is not a letter, runs of one or two letter fragments joined (`f.u.c.k` is `fuck`). With
/// `join_all`, every fragment squashed together is one more word.
fn words(text: &str, decode_leet: bool, join_all: bool) -> Vec<String> {
    let base = base_normalize(text);
    let mut spaced = String::with_capacity(base.len() + 8);
    let mut prev_lower = false;
    for c in base.chars() {
        if prev_lower && c.is_ascii_uppercase() {
            spaced.push(' ');
        }
        prev_lower = c.is_ascii_lowercase();
        spaced.push(if decode_leet { leet(c).unwrap_or(c) } else { c });
    }
    let lower = spaced.to_lowercase();
    let raw: Vec<&str> = lower
        .split(|c: char| !c.is_ascii_lowercase())
        .filter(|w| !w.is_empty())
        .collect();
    let mut out: Vec<String> = Vec::new();
    let mut run = String::new();
    for word in &raw {
        if word.len() <= 2 {
            run.push_str(word);
            continue;
        }
        if !run.is_empty() {
            out.push(std::mem::take(&mut run));
        }
        out.push((*word).to_owned());
    }
    if !run.is_empty() {
        out.push(run);
    }
    if join_all && raw.len() > 1 {
        let joined = raw.concat();
        if !out.contains(&joined) {
            out.push(joined);
        }
    }
    out
}

#[cfg(test)]
mod test {
    use super::*;

    fn blocked(text: &str, target: Target, config: &FilterConfig) -> String {
        find_blocked(text, target, config).map_or_else(String::new, |hit| hit.term.to_owned())
    }

    fn gizmo() -> FilterConfig {
        FilterConfig::from_settings("gizmo", "", "")
    }

    #[test]
    fn empty_config_blocks_nothing() {
        let config = FilterConfig::default();
        for text in ["f4gg0t", "what the fuck", "kkk", "Scunthorpe", "gizmo"] {
            assert_eq!(blocked(text, Target::Text, &config), "", "{text}");
            assert_eq!(blocked(text, Target::Handle, &config), "", "{text}");
        }
    }

    #[test]
    fn admin_term_matches_variants() {
        let config = gizmo();
        for text in [
            "g1zm0",
            "gizmooo",
            "GIZMO",
            "a gizmo here",
            "gízmo",
            "GizmoMe",
            "g-i-z-m-o",
        ] {
            assert_eq!(blocked(text, Target::Text, &config), "gizmo", "{text}");
        }
        assert_eq!(blocked("gizmos", Target::Text, &config), "gizmo");
        assert_eq!(blocked("hello", Target::Text, &config), "");
    }

    #[test]
    fn handles_also_match_squashed_fragments() {
        let config = gizmo();
        assert_eq!(blocked("gi_zmo", Target::Handle, &config), "gizmo");
        assert_eq!(blocked("Gi Zmo", Target::Text, &config), "");
    }

    #[test]
    fn parses_admin_settings() {
        let config =
            FilterConfig::from_settings("Spoon, b4d-w0rd\nplonk\n", "spooner", "@Boss, ceo\n");
        let list = |v: &[Box<str>]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(list(&config.extra_blocked), ["spoon", "b4d-w0rd", "plonk"]);
        assert_eq!(list(&config.extra_allowed), ["spooner"]);
        assert_eq!(list(&config.extra_reserved), ["boss", "ceo"]);
    }

    #[test]
    fn allowed_word_is_exempt() {
        let config = FilterConfig::from_settings("spoon", "spooner", "");
        assert_eq!(blocked("a spoon!", Target::Text, &config), "spoon");
        assert_eq!(blocked("PlonkSpoon", Target::Text, &config), "spoon");
        assert_eq!(blocked("Spooner", Target::Text, &config), "");
    }

    #[test]
    fn ignores_admin_term_without_letters() {
        let config = FilterConfig::from_settings("!!!", "", "");
        assert_eq!(blocked("hello", Target::Text, &config), "");
    }

    #[test]
    fn reserves_handles() {
        let config = FilterConfig::from_settings("", "", "@Boss, ceo\n");
        for reserved in [
            "boss",
            "Boss_1",
            "ceo",
            "CEO99",
            "admin",
            "Admin_01",
            "a_d_m_i_n",
            "x",
            "x1",
            "tuita",
            "Tuita_1",
        ] {
            assert!(is_reserved_handle(reserved, &config), "{reserved}");
        }
        for free in ["bossy", "alice", "tuitaa"] {
            assert!(!is_reserved_handle(free, &config), "{free}");
        }
        assert!(!is_reserved_handle("boss", &FilterConfig::default()));
    }
}
