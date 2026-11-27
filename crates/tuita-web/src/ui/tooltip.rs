//! A text hint on hover and focus.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, component, view},
};

/// Wraps one control and shows `text` above it after half a second of hover or keyboard focus.
/// The caller wires the description: put `aria-describedby=(id)` on the control. `id` must be
/// unique on the page. The show and hide are CSS (`:hover`, `:focus-within`).
#[component]
pub(crate) async fn tooltip(
    cx: &Cx,
    id: &str,
    text: &str,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <span class="host" role="presentation" data-c="tooltip">
            (child)
            <span id=(id) role="tooltip" class="tip">(text)</span>
        </span>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn the_tip_has_the_id_the_control_describes_itself_by() {
        let html = render("en", |cx| {
            view! {
                tooltip(
                    id: "tip-copy",
                    text: "Copy link",
                    <button aria-describedby="tip-copy">"Copy"</button>
                )
            }
            .boxed()
        });
        assert!(
            html.contains(r#"<span id="tip-copy" role="tooltip" class="tip">Copy link</span>"#),
            "{html}"
        );
        assert!(html.contains(r#"aria-describedby="tip-copy""#), "{html}");
    }
}
