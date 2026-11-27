//! One row of the conversation list.

use topcoat::{
    Result,
    context::Cx,
    view::{View, class, component, view},
};
use tuita_server::dto::ConversationDto;

use crate::{
    fmt::{acct_of, full_time, handle_of, rel_label},
    icons::Icon,
    l10n::{strip_isolates, t},
    paths,
    post::rich_text::{around, emote_text, user_name},
    ui::{
        avatar::{AvatarShape, avatar},
        badge::badge,
        icon::icon,
    },
};

/// The link to the conversation with the person: avatar, name, handle, time, the newest message
/// (or the sealed placeholder) and the unread count.
///
/// `active` is the open conversation: its row never shows a count, because the open thread marks
/// its messages read as they arrive. A sealed newest message carries what `e2ee.js` needs to open
/// it (`[data-preview]`, see there); the row says "Encrypted message" until it does. A page that
/// already knows the text (the design gallery) passes it as `opened`.
#[component]
pub(crate) async fn conversation_row(
    cx: &Cx,
    conversation: &ConversationDto,
    #[default] active: bool,
    #[default]
    #[into]
    opened: Option<&str>,
) -> Result<impl View> {
    let user = &conversation.user;
    let last = &conversation.last;
    let acct = acct_of(&user.handle, user.domain.as_deref());
    let sealed = last.ciphertext.is_some();
    let unread = if active {
        0
    } else {
        conversation.unread.max(0)
    };
    let (before, after) = if last.mine {
        around(&t!(messages_you_prefix, text: "\u{0}").resolve(cx))
    } else {
        (String::new(), String::new())
    };
    let created = last
        .created_at
        .format(&time::format_description::well_known::Rfc3339)?;
    let when = rel_label(cx, &created);
    let full = full_time(cx, &created);
    let shape = AvatarShape::from_name(user.avatar_shape.as_str()).unwrap_or_default();
    let unread_label = strip_isolates(&t!(messages_unread, count: unread).resolve(cx));
    let placeholder = t!(e2ee_preview).resolve(cx).into_owned();
    let message_id = last.id.to_string();
    let user_id = user.id.to_string();
    Ok(view! {
        <a
            class=(class!("row", "wash", "unread" if unread > 0, "active" if active))
            href=(paths::messages(&acct))
            aria-current=(active.then_some("page"))
            data-c="conversation-row"
            data-id=(&user_id)
            data-acct=(&acct)
            data-name=(&user.display_name)
        >
            avatar(
                seed: &user.handle,
                src: user.avatar_url.as_deref(),
                alt: "",
                shape: shape,
                ink: user.ink.as_str(),
                size: 48
            )
            <span class="text">
                <span class="head">
                    user_name(
                        name: &user.display_name,
                        verified: user.verified,
                        bot: user.bot,
                        emojis: &user.emojis
                    )
                    <span class="dim handle">
                        (handle_of(&user.handle, user.domain.as_deref()))
                    </span>
                    <time
                        class="dim time"
                        datetime=(&created)
                        data-rel=""
                        title=(&full)
                    >
                        (&when)
                    </time>
                </span>
                if sealed {
                    <span
                        class="preview"
                        data-preview=(last.ciphertext.as_deref())
                        data-peer-id=(&user_id)
                        data-peer-key=(conversation.peer_key.as_deref())
                        data-mid=(&message_id)
                        data-mine=(last.mine.to_string())
                    >
                        icon(name: Icon::Lock, size: 14)
                        (&before)
                        <span data-ph="" hidden=(opened.is_some())>(&placeholder)</span>
                        <span data-text="" hidden=(opened.is_none())>
                            (opened.unwrap_or_default())
                        </span>
                        (&after)
                    </span>
                } else {
                    <span class="preview">
                        (&before)
                        emote_text(text: &last.text, emojis: &last.emojis)
                        (&after)
                    </span>
                }
            </span>
            if unread > 0 {
                badge(count: unread.unsigned_abs(), label: unread_label.as_str())
            }
        </a>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;
    use tuita_server::dto::{ConversationDto, MessageDto};
    use uuid::Uuid;

    use super::*;
    use crate::{post::testing::sample_user, testing::render};

    fn row(ciphertext: Option<&str>, mine: bool, unread: i64) -> ConversationDto {
        let created_at = time::OffsetDateTime::UNIX_EPOCH;
        ConversationDto {
            user: sample_user("ada"),
            unread,
            last: MessageDto {
                id: Uuid::from_u128(9),
                text: if ciphertext.is_some() {
                    String::new()
                } else {
                    "see you".into()
                },
                ciphertext: ciphertext.map(str::to_owned),
                created_at,
                mine,
                read_at: None,
                emojis: Vec::new(),
            },
            peer_key: ciphertext.map(|_| "KEY".to_owned()),
        }
    }

    #[test]
    fn a_plain_row_links_to_the_conversation_and_counts_unread() {
        let html = render("en", |cx| {
            let conversation = row(None, false, 3);
            view! { conversation_row(conversation: &conversation) }.boxed()
        });
        assert!(html.contains(r#"href="/messages/ada""#), "{html}");
        assert!(html.contains("see you"), "{html}");
        assert!(html.contains(r#"class="row wash unread""#), "{html}");
        assert!(html.contains("3 unread"), "{html}");
        assert!(!html.contains("data-preview"), "{html}");
    }

    #[test]
    fn the_open_conversation_shows_no_count_and_is_the_current_page() {
        let html = render("en", |cx| {
            let conversation = row(None, false, 3);
            view! { conversation_row(conversation: &conversation, active: true) }.boxed()
        });
        assert!(html.contains(r#"aria-current="page""#), "{html}");
        assert!(!html.contains("unread"), "{html}");
    }

    #[test]
    fn a_sealed_row_says_encrypted_and_carries_what_the_island_opens() {
        let html = render("en", |cx| {
            let conversation = row(Some("v1.a.b.c.d.e"), true, 0);
            view! { conversation_row(conversation: &conversation) }.boxed()
        });
        assert!(html.contains(r#"data-preview="v1.a.b.c.d.e""#), "{html}");
        assert!(html.contains(r#"data-peer-key="KEY""#), "{html}");
        assert!(html.contains("Encrypted message"), "{html}");
        assert!(html.contains("You: "), "{html}");
        assert!(!html.contains("see you"), "{html}");
    }
}
