//! Which language a request is rendered in.
//!
//! The order is the signed-in account's language, the `tuita_lang` cookie, the instance's guess
//! from the request (`detected_language`), then English. Only the twelve shipped languages count.

use topcoat::{context::Cx, l10n::Locale};

use crate::{cookies, session};

/// Cookie that remembers the language chosen in this browser.
pub(crate) const COOKIE: &str = "tuita_lang";
/// One year, in seconds.
pub(crate) const COOKIE_MAX_AGE: u32 = 31_536_000;

/// An interface language and its name in that language.
pub(crate) struct Language {
    pub(crate) code: &'static str,
    pub(crate) name: &'static str,
}

/// The twelve interface languages, each named in itself. The codes are `tuita_core::locale::LANGS`.
pub(crate) const LANGUAGES: [Language; 12] = [
    Language {
        code: "en",
        name: "English",
    },
    Language {
        code: "pl",
        name: "Polski",
    },
    Language {
        code: "de",
        name: "Deutsch",
    },
    Language {
        code: "es",
        name: "Español",
    },
    Language {
        code: "fr",
        name: "Français",
    },
    Language {
        code: "pt",
        name: "Português",
    },
    Language {
        code: "uk",
        name: "Українська",
    },
    Language {
        code: "it",
        name: "Italiano",
    },
    Language {
        code: "nl",
        name: "Nederlands",
    },
    Language {
        code: "tr",
        name: "Türkçe",
    },
    Language {
        code: "cs",
        name: "Čeština",
    },
    Language {
        code: "ja",
        name: "日本語",
    },
];

/// The shipped language `code` names, if it is one.
#[must_use]
pub(crate) fn known(code: &str) -> Option<&'static str> {
    LANGUAGES
        .iter()
        .find(|language| language.code == code)
        .map(|language| language.code)
}

/// The language of this request, by the order in the module docs.
///
/// # Errors
///
/// Fails when the viewer or instance cannot be loaded.
pub(crate) async fn resolve(cx: &Cx) -> topcoat::Result<&'static str> {
    if let Some(viewer) = session::viewer(cx).await?
        && let Some(code) = viewer.me.settings.language.as_deref().and_then(known)
    {
        return Ok(code);
    }
    if let Some(code) = cookies::get(cx, COOKIE).and_then(known) {
        return Ok(code);
    }
    // The instance row is a hint only: a page still renders in English when it cannot be read.
    if let Ok(instance) = session::instance(cx).await
        && let Some(code) = known(&instance.detected_language)
    {
        return Ok(code);
    }
    Ok("en")
}

/// `code` as a Topcoat locale; English for anything unknown.
#[must_use]
pub(crate) fn locale_of(code: &str) -> Locale {
    code.parse().unwrap_or(topcoat::l10n::locale!("en"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lists_the_twelve_languages_each_named_in_itself() {
        let codes: Vec<_> = LANGUAGES.iter().map(|l| l.code).collect();
        assert_eq!(codes, tuita_core::locale::LANGS);
        let names: Vec<_> = LANGUAGES.iter().map(|l| l.name).collect();
        assert_eq!(
            names,
            [
                "English",
                "Polski",
                "Deutsch",
                "Español",
                "Français",
                "Português",
                "Українська",
                "Italiano",
                "Nederlands",
                "Türkçe",
                "Čeština",
                "日本語"
            ]
        );
    }

    #[test]
    fn unknown_codes_are_not_languages() {
        assert_eq!(known("pl"), Some("pl"));
        assert_eq!(known("pl-PL"), None);
        assert_eq!(known("zh"), None);
        assert_eq!(known(""), None);
    }

    #[test]
    fn every_language_is_a_valid_locale() {
        for language in &LANGUAGES {
            assert_eq!(locale_of(language.code).to_string(), language.code);
        }
    }
}
