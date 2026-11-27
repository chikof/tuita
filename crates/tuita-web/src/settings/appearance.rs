//! `/settings/appearance`: theme, ink, contrast, text size and motion.
//!
//! These are device preferences. Nothing goes to the server: every control carries `data-pref`
//! and `prefs.js` stores the choice in `localStorage` and applies it at once.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_server::dto::MeDto;

use crate::{
    l10n::t,
    post::rich_text::user_name,
    settings::{card::setting_card, ink_picker::pref_ink_picker, page::settings_page},
    shell::parts::account_avatar,
};

/// A tablist of buttons for one preference. `prefs.js` marks the stored one `aria-selected`; the
/// segmented control's own styles draw it.
#[component]
async fn pref_segmented(
    cx: &Cx,
    pref: &str,
    items: Vec<(String, String)>,
    label: &str,
) -> Result<impl View> {
    Ok(view! {
        <div class="control" data-c="pref-control">
            <span class="mono-tag" aria-hidden="true">(label)</span>
            <div
                class="segmented"
                role="tablist"
                aria-label=(label)
                data-roving="tablist"
                data-c="segmented"
            >
                for (id, text) in items {
                    <button
                        type="button"
                        role="tab"
                        class="segment"
                        aria-selected="false"
                        data-pref=(pref)
                        data-value=(&id)
                    >
                        <span class="pill" aria-hidden="true"></span>
                        <span class="text">(text)</span>
                    </button>
                }
            </div>
        </div>
    })
}

/// The theme, reading and preview cards.
#[component]
pub(crate) async fn appearance(cx: &Cx, me: &MeDto) -> Result<impl View> {
    let user = &me.user;
    let title = t!(settings_appearance).resolve(cx).into_owned();
    let system = t!(settings_theme_system).resolve(cx).into_owned();
    let light = t!(settings_theme_light).resolve(cx).into_owned();
    let dark = t!(settings_theme_dark).resolve(cx).into_owned();
    let modes = pairs([("system", system), ("light", light), ("dark", dark)]);
    let normal = t!(settings_contrast_normal).resolve(cx).into_owned();
    let more = t!(settings_contrast_more).resolve(cx).into_owned();
    let contrasts = pairs([("normal", normal), ("more", more)]);
    let size_items: Vec<(String, String)> = ["0.9", "1", "1.1", "1.25"]
        .into_iter()
        .map(|id| {
            let percent = (id.parse::<f64>().unwrap_or(1.0) * 100.0).round();
            (id.to_owned(), format!("{percent}%"))
        })
        .collect();
    let motion_system = t!(settings_motion_system).resolve(cx).into_owned();
    let reduce = t!(settings_motion_reduce).resolve(cx).into_owned();
    let motions = pairs([("system", motion_system), ("reduce", reduce)]);
    let mode_label = t!(settings_theme_mode).resolve(cx).into_owned();
    let ink_label = t!(settings_theme_ink).resolve(cx).into_owned();
    let contrast_label = t!(settings_theme_contrast).resolve(cx).into_owned();
    let size_label = t!(settings_theme_text_size).resolve(cx).into_owned();
    let motion_label = t!(settings_theme_motion).resolve(cx).into_owned();
    Ok(view! {
        settings_page(
            title: &title,
            setting_card(
                title: t!(settings_theme_title).resolve(cx).into_owned(),
                description: t!(settings_appearance_note).resolve(cx).into_owned(),
                pref_segmented(pref: "mode", items: modes, label: &mode_label)
                pref_ink_picker(label: &ink_label)
                pref_segmented(
                    pref: "contrast",
                    items: contrasts,
                    label: &contrast_label
                )
            )
            setting_card(
                title: t!(settings_reading_title).resolve(cx).into_owned(),
                pref_segmented(pref: "textScale", items: size_items, label: &size_label)
                pref_segmented(pref: "motion", items: motions, label: &motion_label)
            )
            setting_card(
                title: t!(settings_preview_title).resolve(cx).into_owned(),
                <div class="sample" data-c="appearance-sample">
                    <span aria-hidden="true">
                        account_avatar(
                            handle: &user.handle,
                            src: user.avatar_url.as_deref(),
                            shape: user.avatar_shape,
                            ink: user.ink,
                            size: 40
                        )
                    </span>
                    <div class="body">
                        <p>
                            user_name(name: &user.display_name, emojis: &user.emojis)
                            <span class="handle">
                                "@"
                                (&user.handle)
                            </span>
                        </p>
                        <p>(t!(settings_preview_text))</p>
                    </div>
                </div>
            )
        )
    })
}

/// Owned `(id, label)` pairs for [`pref_segmented`].
fn pairs<const N: usize>(items: [(&str, String); N]) -> Vec<(String, String)> {
    items
        .into_iter()
        .map(|(id, label)| (id.to_owned(), label))
        .collect()
}
