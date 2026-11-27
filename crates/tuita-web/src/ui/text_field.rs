//! A labelled single-line input.

use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, View, class, component, element_id, view},
};

/// The id the message under a field has: the error wins over the hint.
pub(crate) fn described_by(id: &str, error: Option<&str>, hint: Option<&str>) -> Option<String> {
    match (error, hint) {
        (Some(_), _) => Some(format!("{id}-error")),
        (None, Some(_)) => Some(format!("{id}-hint")),
        (None, None) => None,
    }
}

/// A `<label>` and an `<input>`, with an `error` (red, wins) or a `hint` under it. The input is
/// `aria-invalid` and `aria-describedby` the message.
///
/// `value` is the initial value. For a value the browser changes, add the bind and input handlers
/// through `attrs`, which go on the `<input>` (`name`, `type`, `autocomplete`, `required`,
/// `:value=$(..)`, `@input=$(..)`, ...). The id is generated per component instance; give a
/// looped field a `#[key]`.
#[component]
pub(crate) async fn text_field(
    cx: &Cx,
    label: &str,
    #[default] value: &str,
    #[default]
    #[into]
    error: Option<&str>,
    #[default]
    #[into]
    hint: Option<&str>,
    #[default] attrs: Attributes,
) -> Result<impl View> {
    let id = element_id(cx);
    let described = described_by(&id, error, hint);
    Ok(view! {
        <div class=(class!("field", "invalid" if error.is_some())) data-c="text-field">
            <label for=(&id)>(label)</label>
            <input
                (attrs)
                id=(&id)
                value=(value)
                aria-invalid=(error.map(|_| "true"))
                aria-describedby=(described)
            >
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
    fn the_label_points_at_the_input() {
        let html = render("en", |cx| {
            view! {
                text_field(
                    label: "Handle",
                    value: "ada",
                    attrs: attributes! { name="handle" }
                )
            }
            .boxed()
        });
        let id = attr_values(&html, "for").pop().expect("label for");
        assert!(html.contains(&format!(r#"id="{id}""#)), "{html}");
        assert!(
            html.contains(r#"<label for=""#) && html.contains(">Handle</label>"),
            "{html}"
        );
        assert!(
            html.contains(r#"value="ada""#) && html.contains(r#"name="handle""#),
            "{html}"
        );
        assert!(
            !html.contains("aria-invalid") && !html.contains("aria-describedby"),
            "{html}"
        );
        assert!(html.contains(r#"class="field""#), "{html}");
    }

    #[test]
    fn an_error_marks_the_input_invalid_and_describes_it() {
        let html = render("en", |cx| {
            view! {
                text_field(label: "Handle", error: "taken", hint: "3 to 15 characters")
            }
            .boxed()
        });
        let id = attr_values(&html, "for").pop().expect("label for");
        assert!(html.contains(r#"class="field invalid""#), "{html}");
        assert!(html.contains(r#"aria-invalid="true""#), "{html}");
        assert!(
            html.contains(&format!(r#"aria-describedby="{id}-error""#)),
            "{html}"
        );
        assert!(
            html.contains(&format!(
                r#"<p id="{id}-error" class="message error">taken</p>"#
            )),
            "{html}"
        );
        assert!(!html.contains("3 to 15"), "{html}");
    }

    #[test]
    fn a_hint_describes_the_input_without_marking_it_invalid() {
        let html = render("en", |cx| {
            view! { text_field(label: "Handle", hint: "tip") }.boxed()
        });
        let id = attr_values(&html, "for").pop().expect("label for");
        assert!(
            html.contains(&format!(r#"aria-describedby="{id}-hint""#)),
            "{html}"
        );
        assert!(html.contains(r#"class="message">tip</p>"#), "{html}");
        assert!(!html.contains("aria-invalid"), "{html}");
    }
}
