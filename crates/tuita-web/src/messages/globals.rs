//! The share-to-chat dialog every page of a member carries (rendered by `shell::globals`).

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::{
    l10n::t,
    messages::people::{PickMode, people_dialog},
    session,
};

/// The id of the share dialog; `messages.js` opens it for a `data-act="share-chat"` menu item.
pub(crate) const SHARE_ID: &str = "share-chat";

/// `<dialog id="share-chat">`: pick a person and the post's link is sent to them as a message.
#[component]
pub(crate) async fn messages_globals(cx: &Cx) -> Result<impl View> {
    let me = session::viewer(cx)
        .await?
        .as_ref()
        .map(|viewer| viewer.me.user.id.to_string());
    let title = t!(share_chat_title).resolve(cx).into_owned();
    Ok(view! {
        if let Some(me) = &me {
            people_dialog(id: SHARE_ID, title: &title, mode: PickMode::Share, me: me)
        }
    })
}
