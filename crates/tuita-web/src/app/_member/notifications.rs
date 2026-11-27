//! `/notifications`.
//!
//! The list is plain markup, so it can be refreshed by `notifications.js`: the island marks the
//! notifications read after the page shows (the CSRF guard of the API wants an `Origin`, which a
//! page navigation does not carry) and, on a live `notification` event, fetches the first page
//! again as a fragment and swaps the list.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::{page, query_params},
    view::{View, ViewExt, view},
};
use tuita_server::dto::{NotificationGroupDto, Page};

use crate::{
    api::{self},
    fragment,
    l10n::t,
    notifications::list::notification_items,
    profile::list::{infinite_list, next_href},
    shell,
    shell::page_bar::page_bar,
    ui::{
        empty_state::empty_state,
        link_tabs::{LinkTab, link_tabs},
    },
};

#[query_params(error = bad_request)]
struct NotificationsQuery {
    filter: Option<String>,
    before: Option<String>,
}

#[page(
    with(Metadata::from_fn(
        async |cx| {
            let title = t!(nav_notifications).resolve(cx).into_owned();
            shell::titled(cx, &title).await
        },
    ))
)]
async fn notifications_page(cx: &Cx) -> Result<impl View> {
    let query = query_params::<NotificationsQuery>(cx)?;
    let mentions = query.filter.as_deref() == Some("mentions");
    let path = format!(
        "/notifications{}",
        api::query(&[
            ("filter", mentions.then_some("mentions")),
            ("before", query.before.as_deref()),
        ])
    );
    let page = api::get::<Page<NotificationGroupDto>>(cx, &path).await?;
    let more = page.next.as_deref().map(|cursor| next_href(cx, cursor));
    if fragment::requested(cx) {
        return Ok(view! {
            notification_items(groups: &page.items, more: more.as_deref())
        }
        .boxed());
    }
    let title = t!(nav_notifications).resolve(cx).into_owned();
    let all = t!(notif_tab_all).resolve(cx).into_owned();
    let only = t!(notif_tab_mentions).resolve(cx).into_owned();
    let tabs_label = t!(notif_filter).resolve(cx).into_owned();
    let (empty_title, empty_body) = if mentions {
        (
            t!(notif_empty_mentions_title).resolve(cx).into_owned(),
            t!(notif_empty_mentions_body).resolve(cx).into_owned(),
        )
    } else {
        (
            t!(notif_empty_all_title).resolve(cx).into_owned(),
            t!(notif_empty_all_body).resolve(cx).into_owned(),
        )
    };
    let context = if mentions {
        "mentions"
    } else {
        "notifications"
    };
    let empty = page.items.is_empty();
    Ok(view! {
        page_bar(
            title: &title,
            below: Some(
                view! {
                    link_tabs(
                        items: vec![
                            LinkTab {
                                id: "all",
                                label: &all,
                                href: "/notifications".to_owned(),
                            },
                            LinkTab {
                                id: "mentions",
                                label: &only,
                                href: "/notifications?filter=mentions".to_owned(),
                            },
                        ],
                        selected: if mentions { "mentions" } else { "all" },
                        label: &tabs_label
                    )
                }.into(

                ),
            )
        )
        <div
            data-island="notifications"
            data-stream="notification"
            data-filter=(if mentions { "mentions" } else { "all" })
        >
            infinite_list(
                label: &title,
                notification_items(groups: &page.items, more: more.as_deref())
            )
            if empty {
                <div data-empty="">
                    empty_state(
                        context: context,
                        title: &empty_title,
                        body: empty_body.as_str()
                    )
                </div>
            }
        </div>
    }
    .boxed())
}
