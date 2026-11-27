//! A bordered surface, optionally a grid tile.

use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, Child, View, class, component, view},
};

/// How a card lays out its content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum CardLayout {
    /// Top to bottom.
    #[default]
    Flow,
    /// First and last child at opposite ends.
    Spread,
}

impl CardLayout {
    const fn class(self) -> &'static str {
        match self {
            Self::Flow => "flow",
            Self::Spread => "spread",
        }
    }
}

/// A card. In a [`bento`](super::bento::bento) it spans `span` columns (1 to 4, at most 2 on a
/// phone) and `rows` rows. `interactive` lifts it on hover and washes it with dither. `tight`
/// shrinks the padding. `entrance` is the index in a staggered entrance (90 ms per step); omit it
/// for none. A `class` in `attrs` is merged; the rest of `attrs` goes on the element.
#[component]
pub(crate) async fn card(
    cx: &Cx,
    #[default] interactive: bool,
    #[default] layout: CardLayout,
    #[default] tight: bool,
    #[default(1)] span: u8,
    #[default(1)] rows: u8,
    #[default]
    #[into]
    entrance: Option<u32>,
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let classes = class!(
        "card",
        layout.class(),
        "wash" if interactive,
        "tight" if tight,
        "interactive" if interactive,
        "enter" if entrance.is_some(),
        attrs.remove("class"),
    );
    let style = format!(
        "--span:{span};--span-sm:{};--rows:{rows}{}",
        span.min(2),
        entrance.map_or_else(String::new, |step| format!(";--enter:{step}")),
    );
    Ok(view! { <div class=(classes) style=(style) data-c="card" (attrs)>(child)</div> })
}

#[cfg(test)]
mod test {
    use topcoat::view::{ViewExt, attributes};

    use super::*;
    use crate::testing::render;

    #[test]
    fn a_plain_card_is_a_flow_div() {
        let html = render("en", |cx| view! { card("hi") }.boxed());
        assert!(
            html.starts_with("<div") && html.contains(r#"class="card flow""#),
            "{html}"
        );
        assert!(html.contains("--span:1;--span-sm:1;--rows:1"), "{html}");
        assert!(html.contains(">hi</div>"), "{html}");
        assert!(!html.contains("--enter"), "{html}");
    }

    #[test]
    fn the_flags_pick_classes_and_the_grid_style() {
        let html = render("en", |cx| {
            view! {
                card(
                    interactive: true,
                    layout: CardLayout::Spread,
                    tight: true,
                    span: 3,
                    rows: 2,
                    entrance: 4u32,
                    attrs: attributes! { class="stat" id="c" },
                    "x"
                )
            }
            .boxed()
        });
        assert!(html.starts_with("<div"), "{html}");
        assert!(
            html.contains(r#"class="card spread wash tight interactive enter stat""#),
            "{html}"
        );
        assert!(
            html.contains("--span:3;--span-sm:2;--rows:2;--enter:4"),
            "{html}"
        );
        assert!(html.contains(r#"id="c""#), "{html}");
    }
}
