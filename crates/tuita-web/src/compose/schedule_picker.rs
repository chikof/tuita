//! The schedule picker of the composer.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};

use crate::{
    icons::Icon,
    l10n::t,
    ui::{
        button::{ButtonSize, ButtonVariant, button},
        popover::popover,
        text_field::text_field,
    },
};

/// A calendar button that opens a panel of quick times and a custom date and time.
///
/// `composer.js` fills `[data-role="presets"]` each time the panel opens (the quick times depend
/// on the clock; each preset is a clone of `template[data-tpl="preset"]` with its `[data-role="name"]`
/// and `[data-role="when"]` set), sets the zone hint and the `min` and `max` of the
/// `datetime-local` input, validates a pick against the server's window, shows
/// `[data-role="failure"]` for a pick outside it and shows `[data-act="schedule-clear"]` while a time
/// is set. The panel has no `<form>`: it sits inside the composer's form, so Enter and the Set
/// button (`[data-act="schedule-set"]`) are handled by the island. `id` names the popover.
#[component]
pub(crate) async fn schedule_picker(cx: &Cx, id: &str) -> Result<impl View> {
    let title = t!(schedule_title).resolve(cx).into_owned();
    let custom = t!(schedule_custom).resolve(cx).into_owned();
    Ok(view! {
        popover(
            id: id,
            label: &title,
            glyph: Icon::Calendar,
            <div class="picker" data-c="schedule-picker" data-role="schedule-panel">
                <h3>(&title)</h3>
                <div class="presets" data-role="presets"></div>
                <div class="custom" data-role="custom">
                    text_field(
                        label: &custom,
                        hint: "",
                        attrs: attributes! { type="datetime-local" data-field="custom" }
                    )
                    button(
                        size: ButtonSize::Sm,
                        disabled: true,
                        attrs: attributes! { data-act="schedule-set" },
                        (t!(schedule_set))
                    )
                </div>
                <p class="failure" role="alert" data-role="failure" hidden=""></p>
                <span data-role="clear" hidden="">
                    button(
                        size: ButtonSize::Sm,
                        variant: ButtonVariant::Ghost,
                        attrs: attributes! { data-act="schedule-clear" },
                        (t!(schedule_clear))
                    )
                </span>
                <a class="link" href="/scheduled">(t!(schedule_view))</a>
                <template data-tpl="preset">
                    button(
                        size: ButtonSize::Sm,
                        variant: ButtonVariant::Ghost,
                        <span data-role="name"></span>
                        <span class="when" data-role="when"></span>
                    )
                </template>
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
    fn the_panel_has_presets_a_custom_field_and_a_link_but_no_form() {
        let html = render("en", |cx| view! { schedule_picker(id: "s1") }.boxed());
        assert!(html.contains(r#"popovertarget="s1-panel""#), "{html}");
        assert!(html.contains(r#"aria-label="Schedule post""#), "{html}");
        assert!(html.contains(r#"type="datetime-local""#), "{html}");
        assert!(html.contains("Pick a date and time"), "{html}");
        assert!(html.contains(r#"href="/scheduled""#), "{html}");
        assert!(html.contains(r#"data-role="presets""#), "{html}");
        assert!(html.contains(r#"data-tpl="preset""#), "{html}");
        assert!(
            !html.contains("<form"),
            "a nested form breaks the composer: {html}"
        );
    }
}
