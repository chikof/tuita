//! A four-column grid of cards.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, component, view},
};

/// A grid of four columns (two under 760px) with 120px rows, for [`card`](super::card::card)s
/// that span columns and rows.
#[component]
pub(crate) async fn bento(cx: &Cx, #[default] child: Child<'_>) -> Result<impl View> {
    Ok(view! { <div class="bento" data-c="bento">(child)</div> })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn it_wraps_its_cards() {
        let html = render("en", |cx| view! { bento(<i>"a"</i>) }.boxed());
        assert!(
            html.contains(r#"<div class="bento" data-c="bento"><i>a</i></div>"#),
            "{html}"
        );
    }
}
