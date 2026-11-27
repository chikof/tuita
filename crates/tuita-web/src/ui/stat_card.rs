//! A card with one number.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, class, component, view},
};

use crate::ui::{
    card::{CardLayout, card},
    dither_number::dither_number,
    number_format::NumberFormat,
};

/// A card with a `label`, a counting `value`, and an optional `sub` line. `href` makes the whole
/// card a link. `hero` puts a large dithered number on a solid panel, for a card that also carries
/// a backdrop `field` (a [`dither_field`](super::dither_field::dither_field)): real text never
/// sits on dither. `span`, `rows` and `entrance` are the card's.
#[component]
pub(crate) async fn stat_card(
    cx: &Cx,
    label: &str,
    value: u64,
    #[default]
    #[into]
    sub: Option<&str>,
    #[default]
    #[into]
    href: Option<&str>,
    #[default] format: NumberFormat,
    #[default]
    #[into]
    entrance: Option<u32>,
    #[default(1)] span: u8,
    #[default(1)] rows: u8,
    #[default] hero: bool,
    #[default] field: Option<Child<'_>>,
) -> Result<impl View> {
    Ok(view! {
        card(
            span: span,
            rows: rows,
            entrance: entrance,
            interactive: href.is_some(),
            tight: !hero,
            layout: CardLayout::Spread,
            if let Some(field) = field {
                (field)
            }
            <div class=(class!("body", "hero" if hero)) data-c="stat-card">
                <p class="label">
                    if let Some(href) = href {
                        <a class="stretch" href=(href)>(label)</a>
                    } else {
                        (label)
                    }
                </p>
                <p class="value">
                    dither_number(value: value, format: format, dither: hero)
                </p>
                if let Some(sub) = sub {
                    <p class="sub">(sub)</p>
                }
            </div>
        )
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn it_shows_the_label_the_number_and_the_sub_line() {
        let html = render("en", |cx| {
            view! { stat_card(label: "Users", value: 1234, sub: "this week") }.boxed()
        });
        assert!(html.contains(r#"<p class="label">Users</p>"#), "{html}");
        assert!(html.contains(r#"data-value="1234""#), "{html}");
        assert!(html.contains(r#"<p class="sub">this week</p>"#), "{html}");
        assert!(html.contains(r#"class="card spread tight""#), "{html}");
        assert!(!html.contains("hero"), "{html}");
    }

    #[test]
    fn a_link_stretches_over_the_card_and_hero_puts_the_number_on_a_panel() {
        let html = render("en", |cx| {
            view! {
                stat_card(
                    label: "Users",
                    value: 7,
                    href: "/admin/users",
                    hero: true,
                    span: 2
                )
            }
            .boxed()
        });
        assert!(
            html.contains(r#"<a class="stretch" href="/admin/users">Users</a>"#),
            "{html}"
        );
        assert!(html.contains(r#"class="body hero""#), "{html}");
        assert!(
            html.contains("wash") && html.contains("interactive"),
            "{html}"
        );
        assert!(html.contains("--span:2"), "{html}");
        assert!(html.contains(r#"data-island="dither-text""#), "{html}");
        assert!(!html.contains("tight"), "{html}");
    }
}
