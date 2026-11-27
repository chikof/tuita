//! The branded 404.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};

use crate::{
    l10n::t,
    ui::{
        button::{ButtonVariant, button},
        dither_text::dither_text,
    },
};

/// "Nothing here": a large 404, a sentence and a button that goes back in history.
#[component]
pub(crate) async fn not_found(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        <div class="notfound" data-c="not-found">
            <div class="digits">dither_text(from: 1.0, to: 0.5, "404")</div>
            <h2>(t!(not_found_title))</h2>
            <p>(t!(not_found_body))</p>
            button(
                variant: ButtonVariant::Outline,
                attrs: attributes! { @click="() => history.back()" },
                (t!(not_found_back))
            )
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn the_page_says_404_and_offers_a_way_back() {
        let html = render("en", |cx| view! { not_found() }.boxed());
        assert!(
            html.contains(r#"<div class="digits"><span class="text""#),
            "{html}"
        );
        assert!(html.contains(">404</span>"), "{html}");
        assert!(html.contains("<h2>"), "{html}");
        assert!(html.contains(r#"class="btn outline md wash""#), "{html}");
        assert!(
            html.contains(r#"<span class="label">Go back</span>"#),
            "{html}"
        );
        assert!(html.contains("data-topcoat-on:click"), "{html}");
    }
}
