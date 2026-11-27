//! One list of the index.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_server::dto::ListDto;

use super::member_count;
use crate::{
    icons::Icon,
    l10n::t,
    paths,
    ui::{
        dither_field::{FieldKind, dither_field},
        icon::icon,
        tag::tag,
    },
};

/// A link to a list: a dithered tile seeded by its id, the name, a lock for a private list and the
/// description (or the member count when it has none).
#[component]
pub(crate) async fn list_row(cx: &Cx, list: &ListDto) -> Result<impl View> {
    let count = member_count(cx, list);
    let id = list.id.to_string();
    let subtitle = if list.description.is_empty() {
        count.clone()
    } else {
        list.description.clone()
    };
    Ok(view! {
        <a class="row" data-c="list-row" href=(paths::list(&id))>
            <span class="art" aria-hidden="true">
                dither_field(
                    field: FieldKind::Seeded,
                    seed: id.as_str(),
                    cell: 4,
                    opacity: 0.6
                )
            </span>
            <span class="text">
                <span class="name">
                    <b>(&list.name)</b>
                    if list.private {
                        icon(name: Icon::Lock, size: 14)
                        tag((t!(lists_private)))
                    }
                </span>
                <span class="dim">(&subtitle)</span>
            </span>
            if !list.description.is_empty() {
                <span class="dim count">(&count)</span>
            }
        </a>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::{lists::testing::sample_list, testing::render};

    #[test]
    fn a_row_links_to_the_list_and_shows_the_count_without_a_description() {
        let list = sample_list("Friends", false);
        let html = render("en", |cx| view! { list_row(list: &list) }.boxed());
        assert!(html.contains(r#"href="/lists/"#), "{html}");
        assert!(html.contains("<b>Friends</b>"), "{html}");
        assert!(html.contains("2 members"), "{html}");
        assert!(!html.contains("Private"), "{html}");
    }

    #[test]
    fn a_private_list_with_a_description_shows_both_and_the_count_apart() {
        let mut list = sample_list("Close", true);
        list.description = "People I trust".to_owned();
        let html = render("en", |cx| view! { list_row(list: &list) }.boxed());
        assert!(html.contains(">Private<"), "{html}");
        assert!(
            html.contains(r#"<span class="dim">People I trust</span>"#),
            "{html}"
        );
        assert!(html.contains(r#"<span class="dim count">"#), "{html}");
    }
}
