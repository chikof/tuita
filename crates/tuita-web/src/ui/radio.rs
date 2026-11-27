//! A radio button with a drawn octagon.

use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, View, component, view},
};

/// A native `<input type="radio" name=(group) value=(value)>` inside a `<label>` named `label`.
/// Radios with the same `group` are one choice. `checked` is the initial state; `attrs` go on the
/// `<input>` (`disabled`, `@change=$(..)`, ...).
#[component]
pub(crate) async fn radio(
    cx: &Cx,
    group: &str,
    value: &str,
    label: &str,
    #[default] checked: bool,
    #[default] attrs: Attributes,
) -> Result<impl View> {
    Ok(view! {
        <label class="radio" data-c="radio">
            <input (attrs) type="radio" name=(group) value=(value) checked=(checked)>
            <span class="box" aria-hidden="true"><i></i></span>
            <span class="text">(label)</span>
        </label>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn it_is_a_labelled_radio_in_its_group() {
        let html = render("en", |cx| {
            view! {
                radio(group: "feed", value: "foryou", label: "For you", checked: true)
            }
            .boxed()
        });
        assert!(html.starts_with(r#"<label class="radio""#), "{html}");
        assert!(html.contains(r#"type="radio""#), "{html}");
        assert!(
            html.contains(r#"name="feed""#) && html.contains(r#"value="foryou""#),
            "{html}"
        );
        assert!(html.contains("checked"), "{html}");
        assert!(
            html.contains(r#"<span class="text">For you</span>"#),
            "{html}"
        );
    }

    #[test]
    fn it_starts_unchecked() {
        let html = render("en", |cx| {
            view! { radio(group: "g", value: "a", label: "A") }.boxed()
        });
        assert!(!html.contains("checked"), "{html}");
    }
}
