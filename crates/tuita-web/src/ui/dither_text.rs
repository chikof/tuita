//! Text filled through a dither gradient.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, component, view},
};

use crate::ui::mask::{Axis, ramp_mask};

/// Mask rows the server draws: one per 2px of a 48px line. `dither.js` redraws the mask for the
/// real height.
const ROWS: usize = 24;

/// Pixel-font text whose ink thins from tone `from` at the top to `to` at the bottom, through a
/// Bayer ramp (`dither.js` sizes the ramp to the text: `data-island="dither-text"`). Use it for
/// decoration; the text stays real text.
#[component]
pub(crate) async fn dither_text(
    cx: &Cx,
    #[default(1.0)] from: f32,
    #[default(0.45)] to: f32,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let mask = ramp_mask(f64::from(from), f64::from(to), ROWS, Axis::Y);
    Ok(view! {
        <span
            class="text"
            data-island="dither-text"
            data-from=(from)
            data-to=(to)
            data-rows=(ROWS)
            style=(format!("mask-image:{mask};-webkit-mask-image:{mask};--rows:{ROWS}"))
            data-c="dither-text"
        >
            (child)
        </span>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn it_wraps_its_content_in_a_masked_span() {
        let html = render("en", |cx| view! { dither_text("404") }.boxed());
        assert!(html.contains(r#"<span class="text""#), "{html}");
        assert!(html.contains(">404</span>"), "{html}");
        assert!(html.contains("mask-image:url("), "{html}");
        assert!(html.contains(r#"data-from="1""#), "{html}");
        assert!(html.contains(r#"data-to="0.45""#), "{html}");
        assert!(html.contains("--rows:24"), "{html}");
    }

    #[test]
    fn the_ramp_runs_between_the_given_tones() {
        let html = render("en", |cx| {
            view! { dither_text(from: 1.0, to: 0.5, "x") }.boxed()
        });
        assert!(html.contains(r#"data-to="0.5""#), "{html}");
    }
}
