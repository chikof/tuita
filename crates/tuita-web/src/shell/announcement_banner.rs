//! The instance's announcements above the page.

use topcoat::{
    Result,
    context::{Cx, memoize},
    view::{View, component, view},
};
use tuita_server::dto::{AnnouncementDto, AnnouncementTone};

use crate::{
    api,
    icons::Icon,
    l10n::t,
    ui::{icon::icon, icon_button::icon_button, mask::level_mask},
};

/// `GET /announcements`: the banners everyone sees, newest first. A failure shows none: the
/// banner is a courtesy and the page works without it.
#[memoize(as_ref)]
pub(crate) async fn announcements(cx: &Cx) -> Result<Vec<AnnouncementDto>> {
    Ok(api::get::<Vec<AnnouncementDto>>(cx, "/announcements")
        .await
        .unwrap_or_default())
}

/// An `https://` link leaves the app; a `/path` stays in it.
pub(crate) fn is_external(link: &str) -> bool {
    let lower = link.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

/// The icon of a tone.
const fn tone_icon(tone: AnnouncementTone) -> Icon {
    match tone {
        AnnouncementTone::Info => Icon::InfoBox,
        AnnouncementTone::Success => Icon::Check,
        AnnouncementTone::Warning => Icon::WarningDiamond,
    }
}

/// A warning interrupts a screen reader; the other tones wait for a pause.
const fn role_of(tone: AnnouncementTone) -> &'static str {
    match tone {
        AnnouncementTone::Warning => "alert",
        _ => "status",
    }
}

/// The dither stripe at the banner's left edge: a lighter mask for calmer tones, none (solid) for
/// a warning.
fn stripe_style(tone: AnnouncementTone) -> String {
    let mask = match tone {
        AnnouncementTone::Info => level_mask(0.25),
        AnnouncementTone::Success => level_mask(0.5),
        AnnouncementTone::Warning => return String::new(),
    };
    format!("mask-image:{mask};-webkit-mask-image:{mask}")
}

/// The banners the instance runs, or nothing when there are none.
///
/// The server renders every active announcement. `shell.js` removes those this browser dismissed
/// (`localStorage` `tuita:ann:<id>`) when the page loads and handles the close buttons
/// (`[data-dismiss]`); the markup it relies on is `[data-island="announcements"]` with
/// `.banner[data-announcement-id]` children.
#[component]
pub(crate) async fn announcement_banner(cx: &Cx) -> Result<impl View> {
    let items = announcements(cx).await?;
    Ok(view! { announcement_banner_view(items: items) })
}

/// [`announcement_banner`] for known `items`.
#[component]
pub(crate) async fn announcement_banner_view(
    cx: &Cx,
    items: &[AnnouncementDto],
) -> Result<impl View> {
    let region = t!(announce_region).resolve(cx).into_owned();
    let dismiss = t!(announce_dismiss).resolve(cx).into_owned();
    let banners: Vec<_> = items
        .iter()
        .map(|item| {
            let tone = match item.tone {
                AnnouncementTone::Info => t!(announce_tone_info).resolve(cx).into_owned(),
                AnnouncementTone::Success => t!(announce_tone_success).resolve(cx).into_owned(),
                AnnouncementTone::Warning => t!(announce_tone_warning).resolve(cx).into_owned(),
            };
            (item, format!("{tone}: "), stripe_style(item.tone))
        })
        .collect();
    Ok(view! {
        if !banners.is_empty() {
            <section
                class="banners"
                aria-label=(region)
                data-island="announcements"
                data-c="announcement-banner"
            >
                for (item, tone, stripe) in banners {
                    <div
                        class=(format!("banner {}", item.tone.as_str()))
                        role=(role_of(item.tone))
                        data-announcement-id=(item.id.to_string())
                    >
                        <i class="stripe" aria-hidden="true" style=(stripe)></i>
                        <span class="icon">
                            icon(name: tone_icon(item.tone), size: 20)
                        </span>
                        <p>
                            <span class="sr-only">(tone)</span>
                            (&item.text)
                            if !item.link.is_empty() {
                                <a
                                    href=(&item.link)
                                    target=(is_external(&item.link).then_some("_blank"))
                                    rel=(is_external(&item.link).then_some(
                                        "noopener noreferrer",
                                    ))
                                >
                                    (t!(announce_more))
                                </a>
                            }
                        </p>
                        icon_button(
                            glyph: Icon::Close,
                            size: crate::ui::button::ButtonSize::Sm,
                            label: &dismiss,
                            attrs: topcoat::view::attributes! { data-dismiss="" }
                        )
                    </div>
                }
            </section>
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn an_absolute_link_leaves_the_app_and_a_path_stays() {
        assert!(is_external("https://example.org/news"));
        assert!(is_external("http://example.org"));
        assert!(is_external("HTTPS://example.org"));
        assert!(!is_external("/terms"));
        assert!(!is_external(""));
    }

    #[test]
    fn a_warning_stripe_is_solid() {
        assert!(stripe_style(AnnouncementTone::Warning).is_empty());
        assert!(stripe_style(AnnouncementTone::Info).starts_with("mask-image:url("));
        assert_ne!(
            stripe_style(AnnouncementTone::Info),
            stripe_style(AnnouncementTone::Success)
        );
    }
}
