//! The `tuita` mark and wordmark.

use topcoat::{
    Result,
    context::Cx,
    view::{View, class, component, view},
};

use crate::l10n::t;

/// The four-cell mark `size` px wide and, with `wordmark`, the app name beside it. Without it the
/// name stays as screen-reader text.
#[component]
pub(crate) async fn brand(
    cx: &Cx,
    #[default(34)] size: u32,
    #[default(true)] wordmark: bool,
) -> Result<impl View> {
    Ok(view! {
        <span class="brand" style=(format!("--mark:{size}px")) data-c="brand">
            <span class="mark" aria-hidden="true">
                <i></i>
                <i></i>
                <i></i>
                <i></i>
            </span>
            <span class=(class!("word", "sr-only" if !wordmark))>(t!(app_name))</span>
        </span>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn the_name_is_visible_with_the_wordmark() {
        let html = render("en", |cx| view! { brand() }.boxed());
        assert!(
            html.contains(r#"<span class="word">tuita</span>"#),
            "{html}"
        );
        assert!(html.contains("--mark:34px"), "{html}");
    }

    #[test]
    fn the_name_is_screen_reader_only_without_it() {
        let html = render("en", |cx| {
            view! { brand(size: 20, wordmark: false) }.boxed()
        });
        assert!(
            html.contains(r#"<span class="word sr-only">tuita</span>"#),
            "{html}"
        );
        assert!(html.contains("--mark:20px"), "{html}");
    }
}
