//! The like heart.
//!
//! The pixel-dithered fill is CSS: the silhouette mask is `--heart-shape` in `post.css` (the
//! heart of pixelarticons on the 2 px grid) cut by the page-wide Bayer
//! levels `--l1` to `--l4`.

use topcoat::{
    Result,
    context::Cx,
    view::{View, class, component, view},
};

use crate::{icons::Icon, ui::icon::icon};

/// An outline heart; `filled` shows the dithered fill, `pop` plays the four-step fill animation.
#[component]
pub(crate) async fn heart(cx: &Cx, filled: bool, #[default] pop: bool) -> Result<impl View> {
    Ok(view! {
        <span class=(class!("heart", "filled" if filled, "pop" if pop)) data-c="heart">
            icon(name: Icon::Heart, size: 20)
            <i aria-hidden="true"></i>
        </span>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    /// The lit cell ranges per row of the heart,.
    const ROWS: [&[(u32, u32)]; 10] = [
        &[(2, 3), (7, 8)],
        &[(1, 4), (6, 9)],
        &[(0, 10)],
        &[(0, 10)],
        &[(0, 10)],
        &[(1, 9)],
        &[(2, 8)],
        &[(3, 7)],
        &[(4, 6)],
        &[(5, 5)],
    ];

    #[test]
    fn the_stylesheet_holds_the_silhouette_of_heart_ts() {
        let mut path = String::new();
        for (row, ranges) in ROWS.iter().enumerate() {
            for (from, to) in *ranges {
                let width = 2 * (to - from + 1);
                path.push_str(&format!(
                    "M{} {}h{width}v2h-{width}z",
                    1 + 2 * from,
                    2 + 2 * row
                ));
            }
        }
        let css = include_str!("../../assets/css/post.css");
        let encoded = path.replace(' ', "%20");
        assert!(
            css.contains(&encoded),
            "post.css lacks the heart path {encoded}"
        );
    }

    #[test]
    fn the_flags_become_classes() {
        let on = render("en", |cx| view! { heart(filled: true, pop: true) }.boxed());
        assert!(on.contains(r#"class="heart filled pop""#), "{on}");
        let off = render("en", |cx| view! { heart(filled: false) }.boxed());
        assert!(off.contains(r#"class="heart""#), "{off}");
    }
}
