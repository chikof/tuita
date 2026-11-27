//! A labelled native `<select>`.

use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, View, component, element_id, view},
};

use crate::{icons::Icon, ui::icon::icon};

/// One `<option>` of a [`select`].
#[derive(Clone, Copy, Debug)]
pub(crate) struct SelectOption<'a> {
    pub(crate) value: &'a str,
    pub(crate) label: &'a str,
}

impl<'a> SelectOption<'a> {
    #[must_use]
    pub(crate) const fn new(value: &'a str, label: &'a str) -> Self {
        Self { value, label }
    }
}

/// A `<label>` and a `<select>` with a drawn arrow and an optional `hint` under it. `value` is
/// the selected option's value. `attrs` go on the `<select>` (`name`, `disabled`, `@change=$(..)`,
/// ...). The id is generated per component instance.
#[component]
pub(crate) async fn select(
    cx: &Cx,
    label: &str,
    options: Vec<SelectOption<'_>>,
    #[default] value: &str,
    #[default]
    #[into]
    hint: Option<&str>,
    #[default] attrs: Attributes,
) -> Result<impl View> {
    let id = element_id(cx);
    Ok(view! {
        <div class="field" data-c="select">
            <label for=(&id)>(label)</label>
            <div class="box">
                <select
                    (attrs)
                    id=(&id)
                    aria-describedby=(hint.map(|_| format!("{id}-hint")))
                >
                    for option in options {
                        <option value=(option.value) selected=(option.value == value)>
                            (option.label)
                        </option>
                    }
                </select>
                <span class="arrow" aria-hidden="true">
                    icon(name: Icon::ChevronDown, size: 20)
                </span>
            </div>
            if let Some(hint) = hint {
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
    fn the_label_names_the_select_and_the_value_is_selected() {
        let html = render("en", |cx| {
            let options = vec![
                SelectOption::new("en", "English"),
                SelectOption::new("pl", "Polski"),
            ];
            view! {
                select(
                    label: "Language",
                    options: options,
                    value: "pl",
                    attrs: attributes! { name="language" }
                )
            }
            .boxed()
        });
        let id = attr_values(&html, "for").pop().expect("label for");
        assert!(html.contains(&format!(r#"id="{id}""#)), "{html}");
        assert!(html.contains(r#"name="language""#), "{html}");
        assert!(
            html.contains(r#"<option value="en">English</option>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<option value="pl" selected="">Polski</option>"#),
            "{html}"
        );
        assert!(!html.contains("aria-describedby"), "{html}");
    }

    #[test]
    fn a_hint_describes_the_select() {
        let html = render("en", |cx| {
            view! {
                select(
                    label: "L",
                    options: vec![SelectOption::new("a", "A")],
                    hint: "Pick one"
                )
            }
            .boxed()
        });
        assert!(html.contains(r#"class="message">Pick one</p>"#), "{html}");
        assert!(html.contains("aria-describedby"), "{html}");
    }
}
