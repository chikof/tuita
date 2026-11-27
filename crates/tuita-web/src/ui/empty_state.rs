//! A dithered picture, a title and an optional sentence and action.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, component, view},
};

use crate::ui::dither_field::{FieldKind, dither_field};

/// An empty list or result: a field-of-dither picture seeded by `context` (the same context always
/// draws the same picture), `title`, an optional `body` and an optional `action` (usually a
/// button).
///
/// The picture is a seeded [`dither_field`] the dither island draws.
#[component]
pub(crate) async fn empty_state(
    cx: &Cx,
    context: &str,
    title: &str,
    #[default]
    #[into]
    body: Option<&str>,
    #[default] action: Option<Child<'_>>,
) -> Result<impl View> {
    Ok(view! {
        <div class="empty" data-c="empty-state">
            <div class="art">
                dither_field(
                    field: FieldKind::Seeded,
                    seed: context,
                    cell: 5,
                    opacity: 0.9
                )
            </div>
            <h3>(title)</h3>
            if let Some(body) = body {
                <p>(body)</p>
            }
            if let Some(action) = action {
                <div class="action">(action)</div>
            }
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn it_shows_a_seeded_picture_and_the_title() {
        let html = render("en", |cx| {
            view! { empty_state(context: "bookmarks", title: "Nothing here") }.boxed()
        });
        assert!(html.contains(r#"data-seed="bookmarks""#), "{html}");
        assert!(html.contains(r#"<canvas aria-hidden="true""#), "{html}");
        assert!(html.contains("<h3>Nothing here</h3>"), "{html}");
        assert!(
            !html.contains("<p>") && !html.contains("class=\"action\""),
            "{html}"
        );
    }

    #[test]
    fn body_and_action_are_optional_extras() {
        let html = render("en", |cx| {
            view! {
                empty_state(
                    context: "x",
                    title: "T",
                    body: "Try later",
                    action: Some(view! { <a href="/explore">"Explore"</a> }.into())
                )
            }
            .boxed()
        });
        assert!(html.contains("<p>Try later</p>"), "{html}");
        assert!(
            html.contains(r#"<div class="action"><a href="/explore">Explore</a></div>"#),
            "{html}"
        );
    }
}
