//! The open conversation: header note, messages and composer.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_server::dto::ConversationViewDto;

use crate::{
    fmt::{acct_of, handle_of},
    l10n::t,
    messages::{
        chat_footer::chat_footer,
        chat_note::chat_note,
        chat_thread::{chat_items, chat_thread},
        safety_dialog::safety_dialog,
    },
    shell::page_bar::page_bar,
    ui::confirm_dialog::confirm_dialog,
};
use topcoat::runtime::Event;

/// The id of the dialog that asks before a message is deleted.
pub(crate) const DELETE_ID: &str = "confirm-chat-delete";

/// The answer to a fragment request: a marker with the current state, the items and, for an
/// older page, the link to the next older one. `messages.js` merges it into the thread.
#[component]
pub(crate) async fn thread_fragment(
    cx: &Cx,
    view: &ConversationViewDto,
    #[default]
    #[into]
    older: Option<&str>,
) -> Result<impl View> {
    let plain_mark = view.user.domain.is_none();
    Ok(view! {
        <div
            hidden=""
            data-view=""
            data-mode=(view.mode.as_str())
            data-peer-key=(view.peer_key.as_deref())
            data-blocked-reason=(view.blocked_reason.as_deref())
        ></div>
        chat_items(messages: &view.messages, plain_mark: plain_mark)
        if let Some(older) = older {
            <a data-older="" href=(older)></a>
        }
    })
}

/// The pane of `/messages/{peer}`. `older` is the URL of the next older page.
///
/// The thread root carries the state the server knows (`data-mode`, `data-peer-key`,
/// `data-blocked-reason`, absent when sending is allowed); `messages.js` decides from it and the
/// keys of this browser, and `e2ee.js` reads `data-peer-key` and `data-mode` to pin and open.
#[component]
pub(crate) async fn thread_pane(
    cx: &Cx,
    view: &ConversationViewDto,
    me: &str,
    #[default]
    #[into]
    older: Option<&str>,
) -> Result<impl View> {
    let user = &view.user;
    let acct = acct_of(&user.handle, user.domain.as_deref());
    let handle = handle_of(&user.handle, user.domain.as_deref());
    let sub = format!("@{}", user.handle);
    let delete_title = t!(chat_delete_title).resolve(cx).into_owned();
    let delete_body = t!(chat_delete_body).resolve(cx).into_owned();
    let delete_label = t!(chat_delete).resolve(cx).into_owned();
    Ok(view! {
        page_bar(
            back: true,
            title: &user.display_name,
            sub: sub.as_str(),
            below: Some(view! { chat_note() }.into())
        )
        <div
            class="chat"
            data-c="chat"
            data-island="chat-thread"
            data-stream="message"
            data-e2ee-lines=""
            data-me=(me)
            data-peer=(&acct)
            data-peer-id=(user.id.to_string())
            data-remote=(user.domain.is_some().to_string())
            data-name=(&user.display_name)
            data-handle=(&handle)
            data-mode=(view.mode.as_str())
            data-peer-key=(view.peer_key.as_deref())
            data-blocked-reason=(view.blocked_reason.as_deref())
        >
            chat_thread(
                user: user,
                messages: &view.messages,
                older: older,
                plain_mark: user.domain.is_none()
            )
            <div class="dock">
                chat_footer(
                    handle: &handle,
                    name: &user.display_name,
                    blocked_reason: view.blocked_reason.as_deref()
                )
            </div>
        </div>
        safety_dialog(name: &user.display_name)
        confirm_dialog(
            id: DELETE_ID,
            title: &delete_title,
            body: &delete_body,
            confirm_label: &delete_label,
            danger: true,
            action: $(|_e: Event| raw!("dispatchEvent(new Event('tuita-chat-delete'))"))
        )
    })
}
