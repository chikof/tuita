//! A dithered hairline with an optional centred label.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::ui::mask::{Axis, ramp_mask};

/// A `role="separator"` rule that fades out at both ends. `vertical` runs it top to bottom;
/// `label` sits between two rules.
#[component]
pub(crate) async fn divider(
    cx: &Cx,
    #[default] vertical: bool,
    #[default]
    #[into]
    label: Option<&str>,
) -> Result<impl View> {
    let axis = if vertical { Axis::Y } else { Axis::X };
    let style = format!(
        "--fade-in:{};--fade-out:{}",
        ramp_mask(0.0, 1.0, 12, axis),
        ramp_mask(1.0, 0.0, 12, axis)
    );
    Ok(view! {
        <div
            class=(if vertical { "divider vertical" } else { "divider" })
            role="separator"
            aria-orientation=(if vertical { "vertical" } else { "horizontal" })
            style=(style)
            data-c="divider"
        >
            <span class="rule"></span>
            if let Some(label) = label {
                <span class="label">(label)</span>
                <span class="rule"></span>
            }
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn a_horizontal_divider_is_a_separator_with_one_rule() {
        let html = render("en", |cx| view! { divider() }.boxed());
        assert!(html.contains(r#"role="separator""#), "{html}");
        assert!(html.contains(r#"aria-orientation="horizontal""#), "{html}");
        assert_eq!(html.matches(r#"class="rule""#).count(), 1);
    }

    #[test]
    fn a_labelled_vertical_divider_has_two_rules_around_the_label() {
        let html = render("en", |cx| {
            view! { divider(vertical: true, label: "or") }.boxed()
        });
        assert!(html.contains(r#"class="divider vertical""#), "{html}");
        assert!(html.contains(r#"aria-orientation="vertical""#), "{html}");
        assert!(html.contains(r#"<span class="label">or</span>"#), "{html}");
        assert_eq!(html.matches(r#"class="rule""#).count(), 2);
    }
}
