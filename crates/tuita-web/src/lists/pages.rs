//! The pages of lists: `/lists`, `/lists/{id}` (the posts) and `/lists/{id}/members`.

use topcoat::{
    Result,
    context::Cx,
    router::query_params,
    view::{View, attributes, component, view},
};
use tuita_server::dto::{ListDto, Page, PostDto, UserDto};

use super::{
    list_header::{ListTab, list_header},
    list_row::list_row,
    member_row::member_row,
};
use crate::{
    api::{self, ApiError, enc},
    feed::list::{more_href, post_items, post_list},
    fragment,
    icons::Icon,
    l10n::t,
    shell::page_bar::page_bar,
    ui::{button::button, empty_state::empty_state, icon_button::icon_button},
};

#[query_params(error = bad_request)]
struct PageQuery {
    before: Option<String>,
}

/// `/lists`: the viewer's lists with a button that opens the editor.
#[component]
pub(crate) async fn lists(cx: &Cx) -> Result<impl View> {
    let items = api::get::<Vec<ListDto>>(cx, "/lists").await?;
    let title = t!(lists_title).resolve(cx).into_owned();
    let new_label = t!(lists_new).resolve(cx).into_owned();
    let label = t!(lists_label).resolve(cx).into_owned();
    let empty_title = t!(lists_empty_title).resolve(cx).into_owned();
    let empty_body = t!(lists_empty_body).resolve(cx).into_owned();
    Ok(view! {
        page_bar(
            title: &title,
            actions: Some(
                view! {
                    icon_button(
                        glyph: Icon::Plus,
                        label: &new_label,
                        attrs: attributes! { data-act="list-new" }
                    )
                }.into(

                ),
            )
        )
        if items.is_empty() {
            empty_state(
                context: "lists",
                title: &empty_title,
                body: empty_body.as_str(),
                action: Some(
                    view! {
                        button(
                            attrs: attributes! { data-act="list-new" },
                            (t!(lists_new))
                        )
                    }.into(

                    ),
                )
            )
        } else {
            <div class="list" role="list" aria-label=(&label) data-c="post-list">
                #[key(list.id.as_bytes())]
                for list in &items {
                    <div role="listitem">list_row(list: list)</div>
                }
            </div>
        }
    })
}

/// What a list page shows when its list cannot be opened: the API's message under a back bar.
#[component]
async fn list_error(cx: &Cx, id: &str, error: &ApiError) -> Result<impl View> {
    let title = t!(lists_title).resolve(cx).into_owned();
    let empty_title = t!(lists_error_title).resolve(cx).into_owned();
    let context = format!("list-error:{id}");
    Ok(view! {
        (error.status)
        page_bar(back: true, title: &title)
        empty_state(
            context: &context,
            title: &empty_title,
            body: error.message.as_str()
        )
    })
}

/// `GET /lists/{id}`.
async fn open(cx: &Cx, id: &str) -> Result<ListDto, ApiError> {
    api::get::<ListDto>(cx, &format!("/lists/{}", enc(id))).await
}

/// One page of `what` (`timeline` or `members`) of list `id`.
async fn page_of<T: serde::de::DeserializeOwned>(
    cx: &Cx,
    id: &str,
    what: &str,
    before: Option<&str>,
) -> Result<Page<T>, ApiError> {
    let path = format!(
        "/lists/{}/{what}{}",
        enc(id),
        api::query(&[("before", before)])
    );
    api::get::<Page<T>>(cx, &path).await
}

/// `/lists/{id}`: the header and the posts of the list's members, paged.
#[component]
pub(crate) async fn list_posts(cx: &Cx, id: &str) -> Result<impl View> {
    let query = query_params::<PageQuery>(cx)?;
    let fragment = fragment::requested(cx);
    let loaded = async {
        let list = if fragment {
            None
        } else {
            Some(open(cx, id).await?)
        };
        let page = page_of::<PostDto>(cx, id, "timeline", query.before.as_deref()).await?;
        Ok::<_, ApiError>((list, page))
    }
    .await;
    let more = loaded.as_ref().ok().and_then(|(_, page)| {
        page.next
            .as_deref()
            .map(|cursor| more_href(cx, &[], cursor))
    });
    let label = t!(lists_posts_label).resolve(cx).into_owned();
    let empty_title = t!(lists_posts_empty_title).resolve(cx).into_owned();
    let empty_body = t!(lists_posts_empty_body).resolve(cx).into_owned();
    let context = format!("list-posts:{id}");
    Ok(view! {
        if let Err(error) = &loaded {
            list_error(id: id, error: error)
        } else if let Ok((list, page)) = &loaded {
            if fragment {
                post_items(posts: &page.items, more: more.as_deref())
            } else {
                if let Some(list) = list {
                    list_header(list: list, tab: ListTab::Posts)
                }
                post_list(
                    label: &label,
                    posts: &page.items,
                    more: more.as_deref(),
                    empty: Some(
                        view! {
                            empty_state(
                                context: &context,
                                title: &empty_title,
                                body: empty_body.as_str()
                            )
                        }.into(

                        ),
                    )
                )
            }
        }
    })
}

/// `/lists/{id}/members`: the header and the members, paged; the owner can remove them.
#[component]
pub(crate) async fn list_members(cx: &Cx, id: &str) -> Result<impl View> {
    let query = query_params::<PageQuery>(cx)?;
    let fragment = fragment::requested(cx);
    let loaded = async {
        let list = open(cx, id).await?;
        let page = page_of::<UserDto>(cx, id, "members", query.before.as_deref()).await?;
        Ok::<_, ApiError>((list, page))
    }
    .await;
    let more = loaded.as_ref().ok().and_then(|(_, page)| {
        page.next
            .as_deref()
            .map(|cursor| more_href(cx, &[], cursor))
    });
    let label = t!(lists_members_label).resolve(cx).into_owned();
    let empty_title = t!(lists_members_empty_title).resolve(cx).into_owned();
    let empty_body = t!(lists_members_empty_body).resolve(cx).into_owned();
    let context = format!("list-members:{id}");
    Ok(view! {
        if let Err(error) = &loaded {
            list_error(id: id, error: error)
        } else if let Ok((list, page)) = &loaded {
            if !fragment {
                list_header(list: list, tab: ListTab::Members)
            }
            if fragment {
                #[key(user.id.as_bytes())]
                for user in &page.items {
                    <div role="listitem">
                        member_row(list_id: id, user: user, owner: list.is_owner)
                    </div>
                }
                if let Some(href) = &more {
                    <div role="listitem" class="more" data-more-host="">
                        <a data-more="" href=(href)>(t!(loading))</a>
                    </div>
                }
            } else {
                <div
                    class="list"
                    role="list"
                    aria-label=(&label)
                    data-c="post-list"
                    data-list-members=""
                    data-list-id=(id)
                    data-owner=(&list.owner.handle)
                    data-count=(list.members.to_string())
                >
                    #[key(user.id.as_bytes())]
                    for user in &page.items {
                        <div role="listitem">
                            member_row(list_id: id, user: user, owner: list.is_owner)
                        </div>
                    }
                    if let Some(href) = &more {
                        <div role="listitem" class="more" data-more-host="">
                            <a data-more="" href=(href)>(t!(loading))</a>
                        </div>
                    }
                </div>
                if page.items.is_empty() && more.is_none() {
                    <div data-empty="">
                        empty_state(
                            context: &context,
                            title: &empty_title,
                            body: empty_body.as_str()
                        )
                    </div>
                }
            }
        }
    })
}
