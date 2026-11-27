//! A solid ink area cut through a Bayer mask.

use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, View, class, component, view},
};

use crate::ui::mask::level_mask_n;

/// The CSS `mask-image` value for `level` over an `n` by `n` grid. A 4 by 4 mask in whole quarters
/// is one of the page-wide `--l0`..`--l4` custom properties, which keeps the markup short.
fn mask_of(level: f32, n: u32) -> String {
    let lit = (level.clamp(0.0, 1.0) * 16.0).round() as u32;
    if n == 4 && lit.is_multiple_of(4) {
        format!("var(--l{})", lit / 4)
    } else {
        level_mask_n(level, n as usize)
    }
}

/// An `aria-hidden` block of `level` (0 to 1) share of ink, dithered on an `n` by `n` grid (2, 4
/// or 8). It sizes from its parent: give it a `class` (merged) or a parent that lays it out.
/// `--fill` overrides the ink.
#[component]
pub(crate) async fn dither_fill(
    cx: &Cx,
    level: f32,
    #[default(4)] n: u32,
    #[default] mut attrs: Attributes,
) -> Result<impl View> {
    let mask = mask_of(level, n);
    Ok(view! {
        <div
            class=(class!("fill", attrs.remove("class")))
            aria-hidden="true"
            style=(format!("mask-image:{mask};-webkit-mask-image:{mask};--n:{n}"))
            data-c="dither-fill"
            (attrs)
        ></div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::{ViewExt, attributes};

    use super::*;
    use crate::testing::render;

    #[test]
    fn whole_quarters_use_the_page_wide_masks() {
        let html = render("en", |cx| view! { dither_fill(level: 0.5) }.boxed());
        assert!(html.contains(r#"class="fill""#), "{html}");
        assert!(html.contains("mask-image:var(--l2)"), "{html}");
        assert!(html.contains("--n:4"), "{html}");
        assert!(html.contains(r#"aria-hidden="true""#), "{html}");
    }

    #[test]
    fn other_levels_and_sizes_carry_their_own_mask() {
        let eighth = render("en", |cx| view! { dither_fill(level: 0.125) }.boxed());
        assert!(eighth.contains("mask-image:url("), "{eighth}");
        let wide = render("en", |cx| view! { dither_fill(level: 0.5, n: 8) }.boxed());
        assert!(
            wide.contains("mask-image:url(") && wide.contains("--n:8"),
            "{wide}"
        );
    }

    #[test]
    fn a_class_is_merged() {
        let html = render("en", |cx| {
            view! { dither_fill(level: 1.0, attrs: attributes! { class="veil" }) }.boxed()
        });
        assert!(html.contains(r#"class="fill veil""#), "{html}");
    }
}
