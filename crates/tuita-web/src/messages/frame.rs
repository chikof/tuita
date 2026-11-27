//! The two-pane frame of `/messages` and `/messages/{peer}`: the conversation list beside the open thread.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, attributes, class, component, view},
};
use tuita_server::dto::{ConversationDto, Page};

use crate::{
    api,
    fmt::acct_of,
    icons::Icon,
    l10n::t,
    messages::{
        conversation_row::conversation_row,
        people::{PickMode, people_dialog},
    },
    shell::page_bar::{Heading, page_bar},
    ui::{
        button::{ButtonVariant, button},
        empty_state::empty_state,
        icon::icon,
        icon_button::icon_button,
    },
};

/// The id of the new-message dialog.
pub(crate) const NEW_ID: &str = "new-message";

/// One page of the conversation list; `before` is the cursor of the page.
///
/// # Errors
///
/// Fails when the API does.
pub(crate) async fn load_list(cx: &Cx, before: Option<&str>) -> Result<Page<ConversationDto>> {
    Ok(api::get(
        cx,
        &format!("/conversations{}", api::query(&[("before", before)])),
    )
    .await?)
}

/// Whether the row is the conversation named by `open` (an account name, any case).
fn is_open(conversation: &ConversationDto, open: Option<&str>) -> bool {
    open.is_some_and(|open| {
        let user = &conversation.user;
        acct_of(&user.handle, user.domain.as_deref()).eq_ignore_ascii_case(open)
    })
}

/// The rows of `list` and, while there is more, the link `infinite.js` follows (`?before=`; it
/// names the open conversation so its row stays marked). This is also the whole answer to a
/// fragment request.
#[component]
pub(crate) async fn conversation_rows(
    cx: &Cx,
    list: &Page<ConversationDto>,
    #[default]
    #[into]
    open: Option<&str>,
) -> Result<impl View> {
    let more = list.next.as_deref().map(|next| {
        format!(
            "/messages{}",
            api::query(&[("before", Some(next)), ("open", open)])
        )
    });
    Ok(view! {
        for conversation in &list.items {
            conversation_row(
                conversation: conversation,
                active: is_open(conversation, open)
            )
        }
        if let Some(more) = more {
            <a class="more" data-more="" href=(more)>(t!(loading))</a>
        }
    })
}

/// The pane of `/messages`: pick a conversation or start one.
#[component]
pub(crate) async fn start_pane(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        <div class="start" data-c="chat-start">
            <span class="art" aria-hidden="true">icon(name: Icon::Mail, size: 40)</span>
            <h2>(t!(chat_start_title))</h2>
            <p>(t!(chat_start_body))</p>
            button(
                attrs: attributes! { command="show-modal" commandfor=(NEW_ID) },
                (t!(chat_new))
            )
        </div>
    })
}

/// The list column (search, rows, an empty note) and the `child` pane next to it. `open` is the
/// account of the open conversation; `me` is the viewer's id.
///
/// The list is `[data-island="chat-list"]`: `messages.js` filters it, refreshes it when a message
/// arrives, and `e2ee.js` opens the previews of sealed rows. On a narrow column only one of list
/// and pane shows (CSS).
#[component]
pub(crate) async fn messages_frame(
    cx: &Cx,
    list: &Page<ConversationDto>,
    me: &str,
    #[default]
    #[into]
    open: Option<&str>,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let new_label = t!(messages_new).resolve(cx).into_owned();
    let new_title = t!(messages_new).resolve(cx).into_owned();
    let empty_new = t!(messages_new).resolve(cx).into_owned();
    let search = t!(chat_search).resolve(cx).into_owned();
    let empty_title = t!(messages_empty_title).resolve(cx).into_owned();
    let empty_body = t!(messages_empty_body).resolve(cx).into_owned();
    let empty = list.items.is_empty() && list.next.is_none();
    let heading = if open.is_some() {
        Heading::H2
    } else {
        Heading::H1
    };
    let label = t!(messages_label).resolve(cx).into_owned();
    let title = t!(messages_title).resolve(cx).into_owned();
    Ok(view! {
        <div class=(class!("messages", "open" if open.is_some())) data-c="messages">
            <div class="split">
                <section
                    class="list"
                    aria-label=(&label)
                    data-island="chat-list"
                    data-stream="message"
                    data-e2ee-previews=""
                    data-me=(me)
                    data-open=(open)
                >
                    page_bar(
                        title: &title,
                        heading: heading,
                        actions: Some(
                            view! {
                                icon_button(
                                    glyph: Icon::Plus,
                                    label: &new_label,
                                    attrs: attributes! { command="show-modal" commandfor=(NEW_ID) }
                                )
                            }.into(

                            ),
                        )
                    )
                    <div class="search">
                        icon(name: Icon::Search, size: 18)
                        <input
                            type="search"
                            placeholder=(&search)
                            aria-label=(&search)
                            autocomplete="off"
                            data-search=""
                        >
                    </div>
                    <div class="rows">conversation_rows(list: list, open: open)</div>
                    if empty {
                        <div class="narrow-only">
                            empty_state(
                                context: "messages",
                                title: &empty_title,
                                body: empty_body.as_str(),
                                action: Some(
                                    view! {
                                        button(
                                            variant: ButtonVariant::Solid,
                                            attrs: attributes! { command="show-modal" commandfor=(NEW_ID) },
                                            (&empty_new)
                                        )
                                    }.into(

                                    ),
                                )
                            )
                        </div>
                        <p class="note split-only">(&empty_body)</p>
                    }
                    <p class="note" role="status" data-search-none="" hidden="">
                        (t!(chat_search_none))
                    </p>
                </section>
                <section class="pane">(child)</section>
            </div>
        </div>
        people_dialog(id: NEW_ID, title: &new_title, mode: PickMode::New, me: me)
    })
}
