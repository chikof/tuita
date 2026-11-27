//! A conversation's messages.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, class, component, view},
};
use tuita_server::dto::{MessageDto, UserDto};
use uuid::Uuid;

use crate::{
    fmt::{full_time, handle_of},
    icons::Icon,
    l10n::{strip_isolates, t},
    messages::bubbles::{Bubble, group_bubbles, seen_id},
    post::rich_text::{rich_text, segments_of, user_name},
    ui::{
        avatar::{AvatarShape, avatar},
        button::ButtonSize,
        icon::icon,
        icon_button::icon_button,
    },
};

/// A sealed message as a browser holding the keys shows it. A page that knows the outcome up
/// front (the design gallery) passes it to render the bubble opened or with its placeholder;
/// in the app `e2ee.js` does this.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Line<'a> {
    /// The opened text.
    Text(&'a str),
    /// Sealed to a key this browser no longer has.
    OwnKey,
    /// Sealed with a key the viewer has not trusted.
    PeerKey,
    /// Could not be opened.
    Unreadable,
}

/// One bubble and its time.
///
/// The `li` carries `data-id`, `data-mine`, `data-at` and `data-read` (the instant the other side
/// read it, else empty) so `messages.js` can group and mark seen when more arrive. A sealed
/// message is `p.bubble[data-ciphertext]` holding the lock, the placeholder and an empty
/// `[data-text]` that `e2ee.js` fills; a plaintext one is drawn here.
#[component]
pub(crate) async fn chat_item(
    cx: &Cx,
    bubble: &Bubble<'_>,
    #[default] seen: bool,
    #[default] plain_mark: bool,
    #[default] line: Option<Line<'_>>,
) -> Result<impl View> {
    let message = bubble.message;
    let id = message.id.to_string();
    let created = message
        .created_at
        .format(&time::format_description::well_known::Rfc3339)?;
    let read = match message.read_at {
        Some(at) => at.format(&time::format_description::well_known::Rfc3339)?,
        None => String::new(),
    };
    let clock = full_time(cx, &created);
    let segments = segments_of(&message.text);
    let placeholder = match line {
        Some(Line::OwnKey) => t!(e2ee_line_own_key).resolve(cx).into_owned(),
        Some(Line::PeerKey) => t!(e2ee_line_peer_key).resolve(cx).into_owned(),
        Some(Line::Unreadable) => t!(e2ee_line_unreadable).resolve(cx).into_owned(),
        _ => t!(e2ee_line_sealed).resolve(cx).into_owned(),
    };
    let opened = match line {
        Some(Line::Text(text)) => Some(text),
        _ => None,
    };
    let opened_segments = segments_of(opened.unwrap_or_default());
    let marked = t!(e2ee_plain_mark).resolve(cx).into_owned();
    let delete = t!(chat_delete).resolve(cx).into_owned();
    let seen_text = t!(chat_seen).resolve(cx).into_owned();
    let sealed = message.ciphertext.is_some();
    Ok(view! {
        <li
            class=(class!("item", "mine" if message.mine, "first" if bubble.first))
            data-id=(&id)
            data-mine=(message.mine.to_string())
            data-at=(&created)
            data-read=(&read)
        >
            <div class="line">
                if sealed {
                    <p
                        class=(class!("bubble", "hidden" if opened.is_none()))
                        data-ciphertext=(message.ciphertext.as_deref())
                        data-mid=(&id)
                        data-mine=(message.mine.to_string())
                    >
                        <span data-lock="" hidden=(opened.is_some())>
                            icon(name: Icon::Lock, size: 14)
                        </span>
                        <span data-ph="" hidden=(opened.is_some())>(&placeholder)</span>
                        <span
                            class="rich"
                            translate="no"
                            data-text=""
                            hidden=(opened.is_none())
                        >
                            rich_text(
                                segments: &opened_segments,
                                emojis: &message.emojis
                            )
                        </span>
                    </p>
                } else {
                    <p class="bubble">
                        rich_text(segments: &segments, emojis: &message.emojis)
                    </p>
                }
                if plain_mark && !sealed {
                    <span class="plain">
                        icon(
                            name: Icon::WarningDiamond,
                            size: 14,
                            label: Some(marked.as_str())
                        )
                    </span>
                }
                if message.mine {
                    <span class="del">
                        icon_button(
                            glyph: Icon::Trash,
                            label: &delete,
                            size: ButtonSize::Sm,
                            attrs: attributes! { data-act="chat-delete" }
                        )
                    </span>
                }
            </div>
            <p class="meta" hidden=(!bubble.last)>
                <time datetime=(&created) data-clock="">(&clock)</time>
                <span class="seen" hidden=(!seen)>
                    icon(name: Icon::CheckDouble, size: 14)
                    (&seen_text)
                </span>
            </p>
        </li>
    })
}

/// The items of `messages` (oldest first), grouped into runs. `messages.js` fetches the same
/// markup for new and older messages.
#[component]
pub(crate) async fn chat_items(
    cx: &Cx,
    messages: &[MessageDto],
    #[default] plain_mark: bool,
    #[default] lines: &[(Uuid, Line<'_>)],
) -> Result<impl View> {
    let bubbles = group_bubbles(messages);
    let seen: Option<Uuid> = seen_id(messages);
    Ok(view! {
        for bubble in &bubbles {
            chat_item(
                bubble: bubble,
                seen: seen == Some(bubble.message.id),
                plain_mark: plain_mark,
                line: lines
                    .iter()
                    .find(|(id, _)| *id == bubble.message.id)
                    .map(|(_, line)| *line)
            )
        }
    })
}

/// The thread: a "load earlier" link while `older` (the URL of the next older page) is set, the
/// intro of an empty conversation, and the list of bubbles. `plain_mark` flags messages that were
/// not sent encrypted although the other person is local.
#[component]
pub(crate) async fn chat_thread(
    cx: &Cx,
    user: &UserDto,
    messages: &[MessageDto],
    #[default]
    #[into]
    older: Option<&str>,
    #[default] plain_mark: bool,
    #[default] lines: &[(Uuid, Line<'_>)],
) -> Result<impl View> {
    let label = strip_isolates(&t!(chat_label, name: user.display_name.as_str()).resolve(cx));
    let shape = AvatarShape::from_name(user.avatar_shape.as_str()).unwrap_or_default();
    let empty = messages.is_empty() && older.is_none();
    Ok(view! {
        <div class="thread" data-c="chat-thread">
            if let Some(older) = older {
                <div class="older">
                    <a
                        class="btn outline sm wash"
                        data-c="button"
                        href=(older)
                        data-older=""
                    >
                        <span class="label">(t!(chat_load_older))</span>
                    </a>
                </div>
            }
            if empty {
                <div class="intro">
                    avatar(
                        seed: &user.handle,
                        src: user.avatar_url.as_deref(),
                        alt: "",
                        shape: shape,
                        ink: user.ink.as_str(),
                        size: 64
                    )
                    user_name(
                        name: &user.display_name,
                        verified: user.verified,
                        bot: user.bot,
                        emojis: &user.emojis
                    )
                    <span class="dim">
                        (handle_of(&user.handle, user.domain.as_deref()))
                    </span>
                    <p class="dim">(t!(chat_intro))</p>
                </div>
            }
            <ol class="list" aria-label=(&label)>
                chat_items(messages: messages, plain_mark: plain_mark, lines: lines)
            </ol>
        </div>
    })
}
