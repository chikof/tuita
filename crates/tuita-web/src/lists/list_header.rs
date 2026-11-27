//! The top of a list's pages: bar, details card and the Posts and
//! Members tabs.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};
use tuita_server::dto::ListDto;

use super::member_count;
use crate::{
    icons::Icon,
    l10n::{strip_isolates, t},
    paths,
    shell::page_bar::page_bar,
    ui::{
        button::{ButtonSize, ButtonVariant, button},
        dialog::dialog,
        icon::icon,
        link_tabs::{LinkTab, link_tabs},
        tag::tag,
    },
};

/// The id of the delete question.
const DELETE_ID: &str = "confirm-list-delete";

/// The tab a list page shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ListTab {
    Posts,
    Members,
}

/// The bar (`@owner · 12 members`), the card with the owner's Edit and Delete buttons and the two
/// tabs, which are links to the posts page and the members page.
#[component]
pub(crate) async fn list_header(cx: &Cx, list: &ListDto, tab: ListTab) -> Result<impl View> {
    let handle = format!("@{}", list.owner.handle);
    let sub = format!("{handle} · {}", member_count(cx, list));
    let by = strip_isolates(&t!(lists_by, handle: handle.as_str()).resolve(cx));
    let id = list.id.to_string();
    let tabs_label = t!(lists_tabs).resolve(cx).into_owned();
    let posts = t!(lists_tab_posts).resolve(cx).into_owned();
    let members = t!(lists_tab_members).resolve(cx).into_owned();
    let delete_title = t!(lists_delete_title).resolve(cx).into_owned();
    let delete_body = t!(lists_delete_body, name: list.name.as_str())
        .resolve(cx)
        .into_owned();
    Ok(view! {
        page_bar(back: true, title: &list.name, sub: sub.as_str())
        <section class="card" data-c="list-header">
            <div class="line">
                if list.private {
                    icon(name: Icon::Lock, size: 16)
                    tag((t!(lists_private)))
                }
                <span class="dim">(&by)</span>
            </div>
            if !list.description.is_empty() {
                <p>(&list.description)</p>
            }
            if list.is_owner {
                <div class="actions">
                    button(
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        attrs: attributes! { data-act="list-edit" data-list-id=(&id) },
                        (t!(lists_edit))
                    )
                    button(
                        variant: ButtonVariant::Danger,
                        size: ButtonSize::Sm,
                        attrs: attributes! { command="show-modal" commandfor=(DELETE_ID) },
                        (t!(lists_delete))
                    )
                </div>
            }
        </section>
        link_tabs(
            items: vec![
                LinkTab { id: "posts", label: &posts, href: paths::list(&id) },
                LinkTab {
                    id: "members",
                    label: &members,
                    href: paths::list_members(&id),
                },
            ],
            selected: if tab == ListTab::Members { "members" } else { "posts" },
            label: &tabs_label
        )
        if list.is_owner {
            dialog(
                id: DELETE_ID,
                title: &delete_title,
                actions: Some(
                    view! {
                        button(
                            variant: ButtonVariant::Ghost,
                            attrs: attributes! { command="close" commandfor=(DELETE_ID) },
                            (t!(cancel))
                        )
                        button(
                            variant: ButtonVariant::Danger,
                            attrs: attributes! {
                                command="close"
                                commandfor=(DELETE_ID)
                                data-act="list-delete-yes"
                                data-list-id=(&id)
                            },
                            (t!(lists_delete))
                        )
                    }.into(

                    ),
                ),
                <p>(&delete_body)</p>
            )
        }
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::{lists::testing::sample_list, testing::render};

    #[test]
    fn the_owner_gets_edit_and_delete_and_the_members_tab_is_selected() {
        let list = sample_list("Friends", false);
        let html = render("en", |cx| {
            view! { list_header(list: &list, tab: ListTab::Members) }.boxed()
        });
        assert!(html.contains("@ada · 2 members"), "{html}");
        assert!(html.contains(r#"data-act="list-edit""#), "{html}");
        assert!(html.contains(r#"data-act="list-delete-yes""#), "{html}");
        assert!(html.contains("List by"), "{html}");
        let members = html.find(">Members<").expect("the members tab");
        let tab = html[..members].rfind("<a").expect("a link");
        assert!(
            html[tab..members].contains(r#"aria-selected="true""#),
            "{html}"
        );
    }

    #[test]
    fn a_visitor_sees_no_owner_actions() {
        let mut list = sample_list("Friends", true);
        list.is_owner = false;
        let html = render("en", |cx| {
            view! { list_header(list: &list, tab: ListTab::Posts) }.boxed()
        });
        assert!(
            !html.contains("list-edit") && !html.contains("list-delete"),
            "{html}"
        );
        assert!(html.contains(">Private<"), "{html}");
    }
}
