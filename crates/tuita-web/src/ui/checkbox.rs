//! A checkbox with a drawn box.

use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, Child, View, component, view},
};

use crate::{icons::Icon, ui::icon::icon};

/// A native `<input type="checkbox">` inside a `<label>`, the box drawn by CSS and the checkmark an
/// icon. The text is `label`, or the child content for a label with links.
///
/// `checked` is the initial state. For a state the browser changes, bind it through `attrs`
/// (`:checked=$(..)`, `@change=$(..)`); `attrs` go on the `<input>` (`name`, `value`, `disabled`,
/// `required`, ...).
#[component]
pub(crate) async fn checkbox(
    cx: &Cx,
    #[default] checked: bool,
    #[default] label: &str,
    #[default] attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <label class="check" data-c="checkbox">
            <input (attrs) type="checkbox" checked=(checked)>
            <span class="box" aria-hidden="true">
                icon(name: Icon::Check, size: 16)
            </span>
            <span class="text">
                (label)
                (child)
            </span>
        </label>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::{ViewExt, attributes};

    use super::*;
    use crate::testing::render;

    #[test]
    fn the_label_wraps_the_input_and_names_it() {
        let html = render("en", |cx| {
            view! {
                checkbox(label: "Remember me", attrs: attributes! { name="remember" })
            }
            .boxed()
        });
        assert!(html.starts_with(r#"<label class="check""#), "{html}");
        assert!(
            html.contains(r#"type="checkbox""#) && html.contains(r#"name="remember""#),
            "{html}"
        );
        assert!(
            html.contains(r#"<span class="text">Remember me</span>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<span class="box" aria-hidden="true"><svg"#),
            "{html}"
        );
        assert!(!html.contains("checked"), "{html}");
    }

    #[test]
    fn checked_is_the_initial_state_and_children_replace_plain_text() {
        let html = render("en", |cx| {
            view! { checkbox(checked: true, <a href="/terms">"terms"</a>) }.boxed()
        });
        assert!(html.contains("checked"), "{html}");
        assert!(
            html.contains(r#"<span class="text"><a href="/terms">terms</a></span>"#),
            "{html}"
        );
    }
}
