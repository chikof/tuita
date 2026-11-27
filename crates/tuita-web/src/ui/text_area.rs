//! A labelled multi-line input that grows with its text.

use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, View, class, component, element_id, view},
};

use crate::ui::text_field::described_by;

/// A `<label>` and a `<textarea>` of at least `rows` lines, with an `error` (red, wins) or a `hint`
/// under it. The textarea is `aria-invalid` and `aria-describedby` the message.
///
/// It grows with its text up to 60vh through CSS (`field-sizing: content`); no script is needed. `value` is the initial text. For text the browser changes, add the bind and input
/// handlers through `attrs`, which go on the `<textarea>` (`name`, `maxlength`, `:value=$(..)`,
/// `@input=$(..)`, ...). The id is generated per component instance; give a looped field a
/// `#[key]`.
#[component]
pub(crate) async fn text_area(
    cx: &Cx,
    label: &str,
    #[default] value: &str,
    #[default]
    #[into]
    error: Option<&str>,
    #[default]
    #[into]
    hint: Option<&str>,
    #[default(3)] rows: u32,
    #[default] attrs: Attributes,
) -> Result<impl View> {
    let id = element_id(cx);
    let described = described_by(&id, error, hint);
    Ok(view! {
        <div
            class=(class!("field", "invalid" if error.is_some()))
            style=(format!("--rows:{rows}"))
            data-c="text-area"
        >
            <label for=(&id)>(label)</label>
            <textarea
                (attrs)
                id=(&id)
                rows=(rows)
                aria-invalid=(error.map(|_| "true"))
                aria-describedby=(described)
            >
                (value)
            </textarea>
            if let Some(error) = error {
                <p id=(format!("{id}-error")) class="message error">(error)</p>
            } else if let Some(hint) = hint {
                <p id=(format!("{id}-hint")) class="message">(hint)</p>
            }
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::{ViewExt, attributes};

    use super::*;
    use crate::testing::{attr_values, render};

    #[test]
    fn the_label_points_at_a_textarea_with_its_rows_and_text() {
        let html = render("en", |cx| {
            view! {
                text_area(
                    label: "Bio",
                    value: "hello",
                    rows: 5,
                    attrs: attributes! { name="bio" }
                )
            }
            .boxed()
        });
        let id = attr_values(&html, "for").pop().expect("label for");
        assert!(html.contains(&format!(r#"id="{id}""#)), "{html}");
        assert!(
            html.contains(r#"rows="5""#) && html.contains(r#"name="bio""#),
            "{html}"
        );
        assert!(html.contains(">hello</textarea>"), "{html}");
        assert!(html.contains(r#"class="field""#), "{html}");
    }

    #[test]
    fn an_error_wins_over_the_hint() {
        let html = render("en", |cx| {
            view! { text_area(label: "Bio", error: "too long", hint: "tip") }.boxed()
        });
        let id = attr_values(&html, "for").pop().expect("label for");
        assert!(html.contains(r#"class="field invalid""#), "{html}");
        assert!(html.contains(r#"aria-invalid="true""#), "{html}");
        assert!(
            html.contains(&format!(r#"aria-describedby="{id}-error""#)),
            "{html}"
        );
        assert!(
            html.contains(">too long</p>") && !html.contains("tip"),
            "{html}"
        );
    }
}
