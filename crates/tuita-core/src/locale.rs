//! Which interface language to suggest.
//!
//! The visitor's country (when a trusted proxy supplies it) decides first, then the browser's
//! `Accept-Language`, then English.
//!
//! ```
//! use tuita_core::locale::detect_language;
//!
//! assert_eq!(detect_language(Some("JP"), Some("de")), "ja");
//! assert_eq!(detect_language(None, Some("pl-PL,en;q=0.5")), "pl");
//! assert_eq!(detect_language(None, None), "en");
//! ```

/// The interface languages shipped; the first is the fallback.
pub const LANGS: [&str; 12] = [
    "en", "pl", "de", "es", "fr", "pt", "uk", "it", "nl", "tr", "cs", "ja",
];

/// The main shipped language of a country (upper-case ISO 3166 code), if it has one.
#[must_use]
fn country_language(country: &str) -> Option<&'static str> {
    Some(match country {
        "PL" => "pl",
        "DE" | "AT" | "CH" | "LI" => "de",
        "ES" | "MX" | "AR" | "CO" | "CL" | "PE" | "VE" | "EC" | "GT" | "CU" | "BO" | "DO"
        | "HN" | "PY" | "SV" | "NI" | "CR" | "PA" | "UY" | "GQ" => "es",
        "FR" | "BE" | "LU" | "MC" | "SN" | "CI" | "ML" | "BF" | "NE" | "TG" | "BJ" | "GA"
        | "CG" | "CD" | "CM" | "MG" | "HT" => "fr",
        "PT" | "BR" | "AO" | "MZ" | "CV" | "GW" | "ST" | "TL" => "pt",
        "UA" => "uk",
        "IT" | "SM" | "VA" => "it",
        "NL" | "SR" | "AW" | "CW" | "SX" => "nl",
        "TR" | "CY" => "tr",
        "CZ" => "cs",
        "JP" => "ja",
        _ => return None,
    })
}

/// The first shipped language in an `Accept-Language` header, highest `q` first.
///
/// A tag matches on its base (`pl-PL` is `pl`); `q=0` and a `q` that is not a number drop it.
#[must_use]
fn accepted_language(header: &str) -> Option<&'static str> {
    let mut wanted: Vec<(&str, f32)> = header
        .split(',')
        .filter_map(|part| {
            let mut pieces = part.split(';');
            let tag = pieces.next()?.trim();
            let q = match pieces.find_map(|param| param.trim().strip_prefix("q=")) {
                Some(q) => q.trim().parse::<f32>().ok()?,
                None => 1.0,
            };
            (!tag.is_empty() && q > 0.0).then_some((tag, q))
        })
        .collect();
    wanted.sort_by(|a, b| b.1.total_cmp(&a.1));
    wanted.iter().find_map(|(tag, _)| {
        let base = tag.split('-').next()?;
        LANGS
            .into_iter()
            .find(|lang| lang.eq_ignore_ascii_case(base))
    })
}

/// The language to suggest to someone who has not picked one: their `country` first, then
/// their `accept_language` header, then `en`. Always one of [`LANGS`].
#[must_use]
pub fn detect_language(country: Option<&str>, accept_language: Option<&str>) -> &'static str {
    country
        .and_then(country_language)
        .or_else(|| accept_language.and_then(accepted_language))
        .unwrap_or("en")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn country_beats_accept_language() {
        assert_eq!(detect_language(Some("JP"), Some("de,en;q=0.5")), "ja");
        assert_eq!(detect_language(Some("BR"), None), "pt");
    }

    #[test]
    fn unmapped_country_falls_to_accept_language() {
        assert_eq!(detect_language(Some("US"), Some("fr")), "fr");
    }

    #[test]
    fn q_orders_the_tags() {
        assert_eq!(
            detect_language(None, Some("de;q=0.4,fr;q=0.9,en;q=0.5")),
            "fr"
        );
        assert_eq!(detect_language(None, Some("xx,pl;q=0.2")), "pl");
        assert_eq!(detect_language(None, Some("de, es")), "de");
    }

    #[test]
    fn base_tag_matches() {
        assert_eq!(detect_language(None, Some("pl-PL,en;q=0.5")), "pl");
        assert_eq!(detect_language(None, Some("ja")), "ja");
        assert_eq!(detect_language(None, Some("UK-ua")), "uk");
        assert_eq!(detect_language(None, Some("pt-BR")), "pt");
    }

    #[test]
    fn zero_q_is_ignored() {
        assert_eq!(detect_language(None, Some("de;q=0,fr;q=0.1")), "fr");
        assert_eq!(detect_language(None, Some("de;q=0")), "en");
        assert_eq!(detect_language(None, Some("de;q=abc")), "en");
    }

    #[test]
    fn unknown_gives_english() {
        assert_eq!(detect_language(None, Some("zh-CN,ko;q=0.8,*;q=0.1")), "en");
        assert_eq!(detect_language(Some("US"), Some("")), "en");
        assert_eq!(detect_language(None, None), "en");
    }

    #[test]
    fn every_country_target_is_shipped() {
        for a in b'A'..=b'Z' {
            for b in b'A'..=b'Z' {
                let code = String::from_utf8(vec![a, b]).expect("ascii");
                if let Some(lang) = country_language(&code) {
                    assert!(LANGS.contains(&lang), "{code} -> {lang}");
                }
            }
        }
    }
}
