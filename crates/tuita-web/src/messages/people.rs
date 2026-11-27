//! The people picker of the new-message and share-to-chat dialogs.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};
use tuita_server::dto::UserDto;

use crate::{
    fmt::{acct_of, handle_of},
    l10n::t,
    post::rich_text::user_name,
    ui::{
        avatar::{AvatarShape, avatar},
        dialog::dialog,
        text_field::text_field,
    },
};

/// What picking a person does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PickMode {
    /// Opens the conversation.
    New,
    /// Sends the shared post to them.
    Share,
}

impl PickMode {
    const fn name(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Share => "share",
        }
    }
}

/// A dialog with a search field and the people found. `messages.js` (`[data-island="people-picker"]`)
/// searches through `/_tuita/chat-people` as the person types and handles the pick. `me` is the
/// viewer's id, which a share needs to seal the message.
#[component]
pub(crate) async fn people_dialog(
    cx: &Cx,
    id: &str,
    title: &str,
    mode: PickMode,
    me: &str,
) -> Result<impl View> {
    let search = t!(messages_new_search).resolve(cx).into_owned();
    let results = t!(messages_new_results).resolve(cx).into_owned();
    Ok(view! {
        dialog(
            id: id,
            title: title,
            <div
                data-c="people-picker"
                data-island="people-picker"
                data-mode=(mode.name())
                data-me=(me)
            >
                text_field(
                    label: &search,
                    attrs: attributes! { type="search" autocomplete="off" }
                )
                <p class="note error" role="alert" data-error="" hidden=""></p>
                <ul class="results" aria-label=(&results)></ul>
            </div>
        )
    })
}

/// The rows of the people found: one button each (`data-pick`, `data-acct`, `data-id`,
/// `data-handle`), or a note that nobody matched.
#[component]
pub(crate) async fn people_rows(cx: &Cx, users: &[UserDto]) -> Result<impl View> {
    Ok(view! {
        for user in users {
            <li>
                <button
                    type="button"
                    class="pick"
                    data-pick=""
                    data-acct=(acct_of(&user.handle, user.domain.as_deref()))
                    data-id=(user.id.to_string())
                    data-handle=(handle_of(&user.handle, user.domain.as_deref()))
                >
                    avatar(
                        seed: &user.handle,
                        src: user.avatar_url.as_deref(),
                        alt: "",
                        shape: AvatarShape::from_name(user.avatar_shape.as_str()).unwrap_or_default(

                        ),
                        ink: user.ink.as_str(),
                        size: 40
                    )
                    <span class="who">
                        user_name(
                            name: &user.display_name,
                            verified: user.verified,
                            bot: user.bot,
                            emojis: &user.emojis
                        )
                        <span class="dim">
                            (handle_of(&user.handle, user.domain.as_deref()))
                        </span>
                    </span>
                </button>
            </li>
        }
        if users.is_empty() {
            <li class="note" role="status">(t!(messages_new_none))</li>
        }
    })
}
