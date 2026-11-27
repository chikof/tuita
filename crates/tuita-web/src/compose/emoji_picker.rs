//! The emoji picker of the composer.

use topcoat::{
    Result,
    context::Cx,
    runtime::signal,
    view::{View, component, view},
};

use super::emoji::EMOJI;
use crate::{
    icons::Icon,
    l10n::t,
    ui::{
        popover::popover,
        tabs::{Choice, tabs},
    },
};

/// A smiley button that opens a panel with the Unicode emoji and, when the instance has any, its
/// custom emotes under a second tab.
///
/// `composer.js` loads `GET /emojis` when the panel opens. With emotes it fills
/// `[data-role="custom"]` (a `<section>` with an `<h3>` per category, a `.grid` of
/// `button[data-emoji=":shortcode:"]`) and shows `[data-role="tabs"]`. Every
/// `button[data-emoji]` inserts its value into the box and closes the panel. `id` names the
/// popover.
#[component]
pub(crate) async fn emoji_picker(cx: &Cx, id: &str) -> Result<impl View> {
    let tab = signal(cx, || "emoji".to_owned());
    let label = t!(composer_emoji).resolve(cx).into_owned();
    let sets = t!(emotepick_tabs).resolve(cx).into_owned();
    let emoji_tab = t!(emotepick_tab_emoji).resolve(cx).into_owned();
    let custom_tab = t!(emotepick_tab_custom).resolve(cx).into_owned();
    Ok(view! {
        popover(
            id: id,
            label: &label,
            glyph: Icon::Smile,
            <div class="picker" data-c="emoji-picker">
                <div data-role="tabs" hidden="">
                    tabs(
                        items: vec![
                            Choice::new("emoji", &emoji_tab),
                            Choice::new("custom", &custom_tab),
                        ],
                        value: &tab,
                        label: &sets
                    )
                </div>
                <div class="scroll" data-role="unicode" :hidden=$(tab.get() != "emoji")>
                    <div class="grid">
                        for emoji in EMOJI {
                            <button type="button" aria-label=(emoji) data-emoji=(emoji)>
                                (emoji)
                            </button>
                        }
                    </div>
                </div>
                <div class="scroll" data-role="custom" :hidden=$(tab.get() != "custom")></div>
            </div>
        )
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn it_offers_the_unicode_grid_and_a_hidden_tab_list_for_custom_emotes() {
        let html = render("en", |cx| view! { emoji_picker(id: "e1") }.boxed());
        assert!(html.contains(r#"popovertarget="e1-panel""#), "{html}");
        assert!(html.contains(r#"aria-label="Add emoji""#), "{html}");
        assert_eq!(html.matches("data-emoji=").count(), 64, "{html}");
        assert!(html.contains(r#"aria-label="Emoji sets""#), "{html}");
        assert!(html.contains(r#"data-role="custom""#), "{html}");
        assert_eq!(html.matches(r#"role="tab""#).count(), 2, "{html}");
    }
}
