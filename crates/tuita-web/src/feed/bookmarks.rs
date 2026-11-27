//! `/bookmarks`: saved posts, and the liked posts under a second tab. The tab is `?tab=likes`.

use topcoat::{
    Result,
    context::Cx,
    router::query_params,
    view::{View, component, view},
};
use tuita_server::dto::{Page, PostDto};

use super::list::{more_href, post_items, post_list};
use crate::{
    api::{self, enc},
    fmt::acct_of,
    fragment,
    l10n::t,
    session,
    shell::page_bar::page_bar,
    ui::{
        empty_state::empty_state,
        link_tabs::{LinkTab, link_tabs},
    },
};

#[query_params(error = bad_request)]
struct BookmarksQuery {
    tab: Option<String>,
    before: Option<String>,
}

/// The page: a bar with the two tabs and one infinite list.
#[component]
pub(crate) async fn bookmarks(cx: &Cx) -> Result<impl View> {
    let query = query_params::<BookmarksQuery>(cx)?;
    let viewer = session::member(cx).await?;
    let likes = query.tab.as_deref() == Some("likes");
    let path = if likes {
        let acct = acct_of(&viewer.me.user.handle, viewer.me.user.domain.as_deref());
        format!(
            "/users/{}/posts{}",
            enc(&acct),
            api::query(&[("tab", Some("likes")), ("before", query.before.as_deref())])
        )
    } else {
        format!(
            "/bookmarks{}",
            api::query(&[("before", query.before.as_deref())])
        )
    };
    let page = api::get::<Page<PostDto>>(cx, &path).await?;
    let keep = [("tab", likes.then_some("likes"))];
    let more = page
        .next
        .as_deref()
        .map(|cursor| more_href(cx, &keep, cursor));

    let title = t!(history_title).resolve(cx).into_owned();
    let tabs_label = t!(history_tabs).resolve(cx).into_owned();
    let saved = t!(bookmarks_title).resolve(cx).into_owned();
    let liked = t!(profile_tab_likes).resolve(cx).into_owned();
    let list_label = if likes {
        t!(history_likes_label).resolve(cx).into_owned()
    } else {
        t!(bookmarks_label).resolve(cx).into_owned()
    };
    let empty_title = if likes {
        t!(profile_empty_likes_title).resolve(cx).into_owned()
    } else {
        t!(bookmarks_empty_title).resolve(cx).into_owned()
    };
    let empty_body = if likes {
        t!(profile_empty_likes_body).resolve(cx).into_owned()
    } else {
        t!(bookmarks_empty_body).resolve(cx).into_owned()
    };
    Ok(view! {
        if fragment::requested(cx) {
            post_items(posts: &page.items, more: more.as_deref())
        } else {
            page_bar(
                title: &title,
                below: Some(
                    view! {
                        link_tabs(
                            items: vec![
                                LinkTab {
                                    id: "bookmarks",
                                    label: &saved,
                                    href: "/bookmarks".to_owned(),
                                },
                                LinkTab {
                                    id: "likes",
                                    label: &liked,
                                    href: "/bookmarks?tab=likes".to_owned(),
                                },
                            ],
                            selected: if likes { "likes" } else { "bookmarks" },
                            label: &tabs_label
                        )
                    }.into(

                    ),
                )
            )
            post_list(
                label: &list_label,
                posts: &page.items,
                more: more.as_deref(),
                empty: Some(
                    view! {
                        empty_state(
                            context: if likes { "likes" } else { "bookmarks" },
                            title: &empty_title,
                            body: empty_body.as_str()
                        )
                    }.into(

                    ),
                )
            )
        }
    })
}
