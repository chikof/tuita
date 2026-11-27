//! The poll editor of the composer.

use topcoat::{
    Result,
    context::Cx,
    runtime::signal,
    view::{View, attributes, component, view},
};

use super::{MAX_OPTION_LENGTH, MAX_POLL_OPTIONS, POLL_HOURS};
use crate::{
    icons::Icon,
    l10n::t,
    ui::{
        button::{ButtonSize, ButtonVariant, button},
        icon_button::icon_button,
        segmented::segmented,
        switch::switch,
        tabs::Choice,
        text_field::text_field,
    },
};

/// The answer fields, the length picker and the "several answers" switch.
///
/// It is rendered hidden with every answer slot; `composer.js` shows it with the poll button and
/// reads it when the post is sent: the answers from the `[data-field="option"]` inputs, the length
/// from the selected `[role="tab"]` of the segmented control (its id ends in `-seg-<hours>`) and
/// "several answers" from `[data-role="multiple"][aria-checked]`. Answers three and four stay
/// hidden until the add button (`[data-act="add-option"]`) shows them; their remove buttons are
/// `[data-act="remove-option"][data-option]`. `[data-act="remove-poll"]` closes the editor.
#[component]
pub(crate) async fn poll_editor(cx: &Cx) -> Result<impl View> {
    let hours = signal(cx, || "24".to_owned());
    let multiple = signal(cx, || false);
    let lengths: Vec<(String, String)> = POLL_HOURS
        .iter()
        .map(|&h| {
            let label = if h < 24 {
                t!(poll_length_hours, n: h)
            } else {
                t!(poll_length_days, n: h / 24)
            };
            (h.to_string(), label.resolve(cx).into_owned())
        })
        .collect();
    let length_label = t!(poll_length).resolve(cx).into_owned();
    let multiple_label = t!(poll_multiple_label).resolve(cx).into_owned();
    let options: Vec<(usize, String, String)> = (0..MAX_POLL_OPTIONS)
        .map(|i| {
            let n = i + 1;
            let label = if i < 2 {
                t!(poll_choice, n: n)
            } else {
                t!(poll_choice_optional, n: n)
            };
            let remove = t!(poll_remove_choice, n: n);
            (
                i,
                label.resolve(cx).into_owned(),
                remove.resolve(cx).into_owned(),
            )
        })
        .collect();
    let max_length = MAX_OPTION_LENGTH.to_string();
    Ok(view! {
        <div class="poll" data-c="poll-editor" data-role="poll" hidden="">
            #[key(i)]
            for (i, label, remove) in options {
                <div class="option" data-option=(i) hidden=(i >= 2)>
                    text_field(
                        label: &label,
                        attrs: attributes! {
                            maxlength=(&max_length)
                            autocomplete="off"
                            data-field="option"
                        }
                    )
                    if i >= 2 {
                        icon_button(
                            glyph: Icon::Close,
                            size: ButtonSize::Sm,
                            label: &remove,
                            attrs: attributes! { data-act="remove-option" data-option=(i) }
                        )
                    }
                </div>
            }
            <div data-role="add-option">
                button(
                    size: ButtonSize::Sm,
                    variant: ButtonVariant::Ghost,
                    attrs: attributes! { data-act="add-option" },
                    (t!(poll_add_choice))
                )
            </div>
            segmented(
                items: lengths
                    .iter()
                    .map(|(id, label)| Choice::new(id, label))
                    .collect(),
                value: &hours,
                label: &length_label
            )
            <div class="row">
                switch(
                    label: &multiple_label,
                    checked: &multiple,
                    attrs: attributes! { data-role="multiple" }
                )
                button(
                    size: ButtonSize::Sm,
                    variant: ButtonVariant::Ghost,
                    attrs: attributes! { data-act="remove-poll" },
                    (t!(poll_remove))
                )
            </div>
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::{l10n::strip_isolates, testing::render};

    #[test]
    fn it_holds_every_answer_slot_a_length_picker_and_the_switch() {
        let html = strip_isolates(&render("en", |cx| view! { poll_editor() }.boxed()));
        assert!(html.contains(r#"data-role="poll""#), "{html}");
        assert_eq!(html.matches(r#"data-field="option""#).count(), 4, "{html}");
        assert!(html.contains("Choice 1"), "{html}");
        assert!(html.contains("Choice 3 (optional)"), "{html}");
        assert_eq!(
            html.matches(r#"data-act="remove-option""#).count(),
            2,
            "{html}"
        );
        assert!(html.contains(r#"aria-label="Poll length""#), "{html}");
        assert_eq!(html.matches(r#"role="tab""#).count(), 5, "{html}");
        assert!(
            html.contains(r#"role="switch""#) && html.contains("Allow multiple choices"),
            "{html}"
        );
        assert!(html.contains("Remove poll"), "{html}");
    }

    #[test]
    fn answers_three_and_four_start_hidden() {
        let html = render("en", |cx| view! { poll_editor() }.boxed());
        assert_eq!(
            html.matches(r#"class="option" data-option="2" hidden"#)
                .count()
                + html
                    .matches(r#"class="option" data-option="3" hidden"#)
                    .count(),
            2,
            "{html}"
        );
    }
}
