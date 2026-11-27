//! The followers and following lists of an account.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_server::dto::{Page, UserDto};

use crate::{
    api::{self, enc},
    fmt::{acct_of, handle_of},
    l10n::{strip_isolates, t},
    paths,
    profile::list::{infinite_list, next_href, user_items},
    shell::page_bar::page_bar,
    ui::{
        empty_state::empty_state,
        link_tabs::{LinkTab, link_tabs},
    },
};

/// Which of the two lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Which {
    Followers,
    Following,
}

impl Which {
    const fn name(self) -> &'static str {
        match self {
            Self::Followers => "followers",
            Self::Following => "following",
        }
    }
}

/// The labels of the two tabs.
struct TabLabels {
    followers: String,
    following: String,
}

/// The two tabs, each a link to its list.
fn tab_items<'a>(acct: &str, labels: &'a TabLabels) -> Vec<LinkTab<'a>> {
    vec![
        LinkTab {
            id: "followers",
            label: &labels.followers,
            href: paths::followers(acct),
        },
        LinkTab {
            id: "following",
            label: &labels.following,
            href: paths::following(acct),
        },
    ]
}

/// `GET /users/<acct>/followers|following`, the page after `before` when given.
async fn fetch(cx: &Cx, acct: &str, list: Which, before: Option<&str>) -> Result<Page<UserDto>> {
    let query = api::query(&[("before", before)]);
    Ok(
        api::get::<Page<UserDto>>(cx, &format!("/users/{}/{}{query}", enc(acct), list.name()))
            .await?,
    )
}

/// A page of people for `infinite.js`: the rows and the link to the next page, nothing else.
#[component]
pub(crate) async fn follow_fragment(
    cx: &Cx,
    acct: &str,
    list: Which,
    before: Option<&str>,
) -> Result<impl View> {
    let page = fetch(cx, acct, list, before).await?;
    let more = page.next.as_deref().map(|cursor| next_href(cx, cursor));
    Ok(view! { user_items(users: &page.items, more: more.as_deref()) })
}

/// The bar with its two tabs and the first page of `list` for `user`.
#[component]
pub(crate) async fn follow_list(
    cx: &Cx,
    user: &UserDto,
    list: Which,
    before: Option<&str>,
) -> Result<impl View> {
    let acct = acct_of(&user.handle, user.domain.as_deref());
    let handle = handle_of(&user.handle, user.domain.as_deref());
    let page = fetch(cx, &acct, list, before).await?;
    let more = page.next.as_deref().map(|cursor| next_href(cx, cursor));
    let labels = TabLabels {
        followers: t!(follows_tab_followers).resolve(cx).into_owned(),
        following: t!(follows_tab_following).resolve(cx).into_owned(),
    };
    let tabs_label = t!(follows_tabs).resolve(cx).into_owned();
    let label = match list {
        Which::Followers => t!(follows_label_followers).resolve(cx).into_owned(),
        Which::Following => t!(follows_label_following).resolve(cx).into_owned(),
    };
    let empty_title = match list {
        Which::Followers => t!(follows_empty_followers_title).resolve(cx).into_owned(),
        Which::Following => t!(follows_empty_following_title).resolve(cx).into_owned(),
    };
    let empty_body = match list {
        Which::Followers => t!(follows_empty_followers_body).resolve(cx).into_owned(),
        Which::Following => t!(follows_empty_following_body).resolve(cx).into_owned(),
    };
    let context = format!("{}:{}", list.name(), user.handle);
    let sub = strip_isolates(&handle);
    Ok(view! {
        page_bar(
            back: true,
            title: &user.display_name,
            sub: sub.as_str(),
            below: Some(
                view! {
                    link_tabs(
                        items: tab_items(&acct, &labels),
                        selected: list.name(),
                        label: &tabs_label
                    )
                }.into(

                ),
            )
        )
        infinite_list(
            label: &label,
            user_items(users: &page.items, more: more.as_deref())
        )
        if page.items.is_empty() {
            empty_state(
                context: &context,
                title: &empty_title,
                body: empty_body.as_str()
            )
        }
    })
}
