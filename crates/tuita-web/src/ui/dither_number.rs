//! A number that counts up to its value.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::ui::{dither_text::dither_text, number_format::NumberFormat};

/// `value` in `format`. The server renders the final figure; `dither.js` counts the visible digits
/// up from 0 (each tick closes a ninth of the gap) unless the page reduces motion
/// (`data-island="dither-number"`). A screen reader gets the plain figure at once. `dither` fills
/// the digits through a dither gradient, for large pixel numbers.
#[component]
pub(crate) async fn dither_number(
    cx: &Cx,
    value: u64,
    #[default] format: NumberFormat,
    #[default] dither: bool,
) -> Result<impl View> {
    let text = format.apply(cx, value);
    Ok(view! {
        <span
            class="number"
            data-island="dither-number"
            data-value=(value)
            data-format=(format.name())
            data-c="dither-number"
        >
            <span aria-hidden="true">
                if dither {
                    dither_text(<span data-shown="">(&text)</span>)
                } else {
                    <span data-shown="">(&text)</span>
                }
            </span>
            <span class="sr-only">(&text)</span>
        </span>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn the_figure_is_there_for_everyone_and_the_island_animates_a_copy() {
        let html = render("en", |cx| view! { dither_number(value: 1234) }.boxed());
        assert!(html.contains(r#"data-value="1234""#), "{html}");
        assert!(
            html.contains(r#"<span data-shown="">1234</span>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<span class="sr-only">1234</span>"#),
            "{html}"
        );
        assert!(!html.contains(r#"class="text""#), "{html}");
    }

    #[test]
    fn compact_and_dither_are_options() {
        let html = render("en", |cx| {
            view! {
                dither_number(value: 1234, format: NumberFormat::Compact, dither: true)
            }
            .boxed()
        });
        assert!(html.contains(r#"data-format="compact""#), "{html}");
        assert!(html.contains("1.2K"), "{html}");
        assert!(html.contains(r#"data-island="dither-text""#), "{html}");
    }
}
