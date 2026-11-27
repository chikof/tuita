//! Fragment routes of the post area. No layout; a guest gets an empty `401` from the `/_tuita`
//! layer and an API failure becomes an empty answer with the API's status.
//!
//! Every answer is HTML an island inserts into the page. Topcoat does not hydrate inserted
//! markup, so none of it holds signals or event handlers; `post.js`, `lightbox.js` and
//! `hovercard.js` drive it through data attributes.

use topcoat::{
    Result,
    context::Cx,
    router::{StatusCode, path_param, query_params, route},
    view::{ViewExt, ViewHandle, view},
};
use tuita_server::dto::{ListDto, Page, PostDto, PostEditsDto, ThreadDto, UserDto};

use crate::{
    api::{self, enc},
    post::{
        dialogs::{
            ListItems, edit_dialog, history_dialog, list_picker_dialog, post_list_dialog,
            post_list_items, report_dialog,
        },
        lightbox::lightbox,
        link_card::link_card,
        poll::poll,
        post_card::{ThreadLine, post_card},
        post_thread::ListKind,
        user_hover_card::user_hover_card,
    },
};

path_param!(acct);
path_param!(post_id);
path_param!(kind);

#[query_params(error = bad_request)]
struct LightboxQuery {
    index: Option<usize>,
}

#[query_params(error = bad_request)]
struct CardQuery {
    main: Option<String>,
    thread: Option<String>,
    hide_reply: Option<String>,
}

#[query_params(error = bad_request)]
struct PageQuery {
    before: Option<String>,
}

/// The post `id` and what surrounds it.
async fn thread(cx: &Cx) -> Result<ThreadDto> {
    Ok(api::get::<ThreadDto>(cx, &format!("/posts/{}", enc(post_id(cx)))).await?)
}

#[route(GET "/_tuita/hovercard/{acct}")]
async fn hovercard(cx: &Cx) -> Result<ViewHandle> {
    let user = api::get::<UserDto>(cx, &format!("/users/{}", enc(acct(cx)))).await?;
    Box::pin(view! { user_hover_card(user: &user) }.first()).await
}

/// The link preview of a post, or nothing while it has none (`stream.js` fills the card slot
/// of a post whose preview finished loading).
#[route(GET "/_tuita/card/{post_id}")]
async fn card(cx: &Cx) -> Result<ViewHandle> {
    let post = thread(cx).await?.post;
    match post.card {
        Some(card) => view! { link_card(card: &card) }.first().await,
        None => view! { (StatusCode::NO_CONTENT) }.first().await,
    }
}

#[route(GET "/_tuita/lightbox/{post_id}")]
async fn lightbox_panel(cx: &Cx) -> Result<ViewHandle> {
    let query = query_params::<LightboxQuery>(cx)?;
    let thread = thread(cx).await?;
    if thread.post.media.is_empty() {
        return Err(topcoat::router::error::not_found().into());
    }
    Box::pin(view! { lightbox(thread: &thread, index: query.index.unwrap_or(0)) }.first()).await
}

/// A fresh copy of a card, for after an edit, a pin or a hide: `main`, `thread` (`up` or `down`)
/// and `hide_reply` repeat how the old card was drawn.
#[route(GET "/_tuita/post/{post_id}")]
async fn refresh(cx: &Cx) -> Result<ViewHandle> {
    let query = query_params::<CardQuery>(cx)?;
    let post = thread(cx).await?.post;
    let line = match query.thread.as_deref() {
        Some("up") => Some(ThreadLine::Up),
        Some("down") => Some(ThreadLine::Down),
        _ => None,
    };
    Box::pin(
        view! {
            post_card(
                post: &post,
                main: query.main.is_some(),
                thread: line,
                hide_reply_context: query.hide_reply.is_some()
            )
        }
        .first(),
    )
    .await
}

/// The poll of a post, drawn for the viewer; `post.js` swaps it in after a vote.
#[route(GET "/_tuita/poll/{post_id}")]
async fn poll_of(cx: &Cx) -> Result<ViewHandle> {
    let post = thread(cx).await?.post;
    Box::pin(view! { poll(post: &post) }.first()).await
}

#[route(GET "/_tuita/dialog/edit/{post_id}")]
async fn edit(cx: &Cx) -> Result<ViewHandle> {
    let post = thread(cx).await?.post;
    Box::pin(view! { edit_dialog(post: &post) }.first()).await
}

#[route(GET "/_tuita/dialog/history/{post_id}")]
async fn history(cx: &Cx) -> Result<ViewHandle> {
    let post = thread(cx).await?.post;
    let edits = api::get::<PostEditsDto>(cx, &format!("/posts/{}/edits", enc(post_id(cx)))).await?;
    Box::pin(view! { history_dialog(post: &post, edits: &edits) }.first()).await
}

#[route(GET "/_tuita/dialog/report/{post_id}")]
async fn report(cx: &Cx) -> Result<ViewHandle> {
    let post = thread(cx).await?.post;
    Box::pin(view! { report_dialog(post: Some(&post)) }.first()).await
}

#[route(GET "/_tuita/dialog/report-user/{acct}")]
async fn report_user(cx: &Cx) -> Result<ViewHandle> {
    let handle = acct(cx).to_owned();
    Box::pin(view! { report_dialog(handle: Some(&handle)) }.first()).await
}

#[route(GET "/_tuita/dialog/lists/{acct}")]
async fn lists(cx: &Cx) -> Result<ViewHandle> {
    let query = api::query(&[("member", Some(acct(cx)))]);
    let lists = api::get::<Vec<ListDto>>(cx, &format!("/lists{query}")).await?;
    let account = acct(cx).to_owned();
    Box::pin(view! { list_picker_dialog(acct: &account, lists: &lists) }.first()).await
}

/// One page of who liked, reposted or quoted a post. With `before` it is only the entries and the
/// link to the next page, for `infinite.js`; without it the whole dialog.
#[route(GET "/_tuita/dialog/post-list/{post_id}/{kind}")]
async fn post_list(cx: &Cx) -> Result<ViewHandle> {
    let query = query_params::<PageQuery>(cx)?;
    let kind = match kind(cx) {
        "likes" => ListKind::Likes,
        "reposts" => ListKind::Reposts,
        "quotes" => ListKind::Quotes,
        _ => return Err(topcoat::router::error::not_found().into()),
    };
    let id = post_id(cx).to_owned();
    let path = format!(
        "/posts/{}/{}{}",
        enc(&id),
        kind.name(),
        api::query(&[("before", query.before.as_deref())])
    );
    let (people, posts, next): (Vec<UserDto>, Vec<PostDto>, Option<String>) =
        if matches!(kind, ListKind::Quotes) {
            let page = api::get::<Page<PostDto>>(cx, &path).await?;
            (Vec::new(), page.items, page.next)
        } else {
            let page = api::get::<Page<UserDto>>(cx, &path).await?;
            (page.items, Vec::new(), page.next)
        };
    let more = next.map(|cursor| {
        format!(
            "/_tuita/dialog/post-list/{}/{}{}",
            enc(&id),
            kind.name(),
            api::query(&[("before", Some(&cursor))])
        )
    });
    let items = if matches!(kind, ListKind::Quotes) {
        ListItems::Posts(&posts)
    } else {
        ListItems::People(&people)
    };
    if query.before.is_some() {
        Box::pin(view! { post_list_items(items: items, more: more.as_deref()) }.first()).await
    } else {
        Box::pin(
            view! {
                post_list_dialog(
                    post_id: &id,
                    kind: kind,
                    items: items,
                    more: more.as_deref()
                )
            }
            .first(),
        )
        .await
    }
}
