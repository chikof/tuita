//! Settings: the section list at `/settings` and one page per section at `/settings/{section}`.
//!
//! Every section renders on the server from the JSON API. A control that saves does so through a
//! procedure in [`actions`] that calls the API, with an optimistic signal in the browser that rolls
//! back and toasts the API's message on failure. A list that changes (sessions, passkeys, tokens,
//! blocked accounts, ...) is a shard that fetches again when its `rev` signal moves.

pub(crate) mod about;
pub(crate) mod account;
pub(crate) mod actions;
pub(crate) mod affiliation;
pub(crate) mod appearance;
pub(crate) mod bots;
pub(crate) mod card;
pub(crate) mod contact;
pub(crate) mod desktop_notifications;
pub(crate) mod device;
pub(crate) mod image_field;
pub(crate) mod index;
pub(crate) mod ink_picker;
pub(crate) mod language;
pub(crate) mod muted_words;
pub(crate) mod notifications;
pub(crate) mod page;
pub(crate) mod passkeys;
pub(crate) mod password_form;
pub(crate) mod privacy;
pub(crate) mod profile;
pub(crate) mod relation_list;
pub(crate) mod sessions;
pub(crate) mod shape_picker;
pub(crate) mod switch;
pub(crate) mod token_secret;
pub(crate) mod tokens;

use std::str::FromStr;

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::{l10n::t, session};

/// One page of the settings, in the order the list shows them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Section {
    Account,
    Profile,
    Privacy,
    Encryption,
    Notifications,
    Devices,
    Passkeys,
    Affiliation,
    Bots,
    Appearance,
    Language,
    About,
}

/// The URL segment is not a section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("not a settings section")]
pub(crate) struct NotASection;

impl Section {
    /// Every section, in list order.
    pub(crate) const ALL: [Self; 12] = [
        Self::Account,
        Self::Profile,
        Self::Privacy,
        Self::Encryption,
        Self::Notifications,
        Self::Devices,
        Self::Passkeys,
        Self::Affiliation,
        Self::Bots,
        Self::Appearance,
        Self::Language,
        Self::About,
    ];

    /// The URL segment: `/settings/<slug>`.
    pub(crate) const fn slug(self) -> &'static str {
        match self {
            Self::Account => "account",
            Self::Profile => "profile",
            Self::Privacy => "privacy",
            Self::Encryption => "encryption",
            Self::Notifications => "notifications",
            Self::Devices => "devices",
            Self::Passkeys => "passkeys",
            Self::Affiliation => "affiliation",
            Self::Bots => "bots",
            Self::Appearance => "appearance",
            Self::Language => "language",
            Self::About => "about",
        }
    }

    /// The heading of the section's page and its row in the list.
    pub(crate) fn title(self, cx: &Cx) -> String {
        match self {
            Self::Account => t!(settings_account),
            Self::Profile => t!(settings_profile),
            Self::Privacy => t!(settings_privacy),
            Self::Encryption => t!(e2ee_settings),
            Self::Notifications => t!(settings_notifications),
            Self::Devices => t!(settings_devices),
            Self::Passkeys => t!(settings_passkeys),
            Self::Affiliation => t!(settings_affiliation),
            Self::Bots => t!(settings_bots),
            Self::Appearance => t!(settings_appearance),
            Self::Language => t!(settings_language),
            Self::About => t!(settings_about),
        }
        .resolve(cx)
        .into_owned()
    }

    /// The sentence under the title in the list.
    pub(crate) fn description(self, cx: &Cx) -> String {
        match self {
            Self::Account => t!(settings_account_desc),
            Self::Profile => t!(settings_profile_desc),
            Self::Privacy => t!(settings_privacy_desc),
            Self::Encryption => t!(e2ee_settings_desc),
            Self::Notifications => t!(settings_notifications_desc),
            Self::Devices => t!(settings_devices_desc),
            Self::Passkeys => t!(settings_passkeys_desc),
            Self::Affiliation => t!(settings_affiliation_desc),
            Self::Bots => t!(settings_bots_desc),
            Self::Appearance => t!(settings_appearance_desc),
            Self::Language => t!(settings_language_desc),
            Self::About => t!(settings_about_desc),
        }
        .resolve(cx)
        .into_owned()
    }
}

impl std::fmt::Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.slug())
    }
}

impl FromStr for Section {
    type Err = NotASection;

    fn from_str(text: &str) -> Result<Self, NotASection> {
        Self::ALL
            .into_iter()
            .find(|section| section.slug() == text)
            .ok_or(NotASection)
    }
}

/// The body of `/settings/{section}`: the section's page.
#[component]
pub(crate) async fn section_page(cx: &Cx, section: Section) -> Result<impl View> {
    let me = &session::member(cx).await?.me;
    Ok(view! {
        match section {
            Section::Account => {
                account::account(me: me)
            }
            Section::Profile => {
                profile::profile(me: me)
            }
            Section::Privacy => {
                privacy::privacy(me: me)
            }
            Section::Encryption => {
                page::settings_page(
                    title: &Section::Encryption.title(cx),
                    crate::messages::encryption::encryption_section()
                )
            }
            Section::Notifications => {
                notifications::notifications(me: me)
            }
            Section::Devices => {
                sessions::devices()
            }
            Section::Passkeys => {
                passkeys::passkeys()
            }
            Section::Affiliation => {
                affiliation::affiliation()
            }
            Section::Bots => {
                bots::bots(me: me)
            }
            Section::Appearance => {
                appearance::appearance(me: me)
            }
            Section::Language => {
                language::language()
            }
            Section::About => {
                about::about()
            }
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn every_section_round_trips_through_its_slug() {
        for section in Section::ALL {
            assert_eq!(section.slug().parse::<Section>(), Ok(section));
        }
        assert_eq!("secrets".parse::<Section>(), Err(NotASection));
        assert_eq!("".parse::<Section>(), Err(NotASection));
    }

    #[test]
    fn the_list_keeps_its_order() {
        let slugs: Vec<_> = Section::ALL.iter().map(|s| s.slug()).collect();
        assert_eq!(
            slugs,
            [
                "account",
                "profile",
                "privacy",
                "encryption",
                "notifications",
                "devices",
                "passkeys",
                "affiliation",
                "bots",
                "appearance",
                "language",
                "about"
            ]
        );
    }
}
