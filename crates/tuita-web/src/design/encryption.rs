//! Gallery sections: end-to-end encrypted messages.
//!
//! The chat note and the chat footer render every variant hidden for `messages.js`; here each
//! demo reveals the one it shows with the `data-design-note` and `data-design-block` hooks of
//! `assets/css/design-parts/media.css`. A sealed thread is drawn with the lines a browser
//! holding the keys would show (`Line`), so nothing here needs keys.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};
use uuid::Uuid;

use super::{
    media::parts::{foot, frame},
    sample,
    section::{row, section},
};
use crate::{
    messages::{
        chat_footer::chat_footer,
        chat_note::chat_note,
        chat_thread::{Line, chat_thread},
        conversation_row::conversation_row,
        e2ee_notice::{Tone, e2ee_notice},
        e2ee_prompt::{PromptKind, e2ee_prompt},
        encryption::encryption_cards,
        safety_dialog::{SAFETY_ID, safety_dialog},
    },
    ui::button::{ButtonVariant, button},
};

const NAME: &str = "Ada Lovelace";
const HANDLE: &str = "@ada";
const BLOCKED: &str = "You can’t message people who blocked you.";

/// The lines of the sealed thread: opened, then each placeholder.
fn sealed_lines() -> [(Uuid, Line<'static>); 4] {
    [
        (
            sample::id_of("s1"),
            Line::Text("Did you see the new dither engine?"),
        ),
        (sample::id_of("s2"), Line::OwnKey),
        (sample::id_of("s3"), Line::PeerKey),
        (sample::id_of("s4"), Line::Unreadable),
    ]
}

/// The same messages once everything opens.
fn opened_lines() -> [(Uuid, Line<'static>); 4] {
    [
        (
            sample::id_of("s1"),
            Line::Text("Did you see the new dither engine?"),
        ),
        (
            sample::id_of("s2"),
            Line::Text("Yes! The Bayer masks look great at 2 px."),
        ),
        (
            sample::id_of("s3"),
            Line::Text("Nice. Photos stay untouched, right?"),
        ),
        (sample::id_of("s4"), Line::Text("Always.")),
    ]
}

#[component]
async fn prompts(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        section(
            id: "e2ee-prompt",
            title: "E2eePrompt",
            note: "setup, unlock, wrong passphrase, reset; the card replaces the composer in a sealed thread",
            frame(
                stack: true,
                e2ee_prompt(kind: PromptKind::Setup)
                e2ee_prompt(kind: PromptKind::Unlock)
            )
        )
        section(
            id: "e2ee-notice",
            title: "E2eeNotice",
            note: "info and warning tones, with the dither stripe",
            frame(
                stack: true,
                e2ee_notice(title: "Heads up", "Sealed in this browser.")
                e2ee_notice(
                    tone: Tone::Warning,
                    title: "Key changed",
                    "Compare the new number first."
                )
            )
        )
    })
}

#[component]
async fn threads(cx: &Cx) -> Result<impl View> {
    let ada = sample::user_of(&sample::ada());
    let sealed = sample::sealed_chat();
    let plain = sample::chat();
    let sealed_lines = sealed_lines();
    let opened_lines = opened_lines();
    Ok(view! {
        section(
            id: "e2ee-thread",
            title: "Encrypted thread",
            note: "sealed with every placeholder, safety number, key changed, plain with a mark, remote, unavailable",
            <div data-design-sandbox="" data-design-number=(sample::SAFETY_NEW)>
                frame(
                    attrs: attributes! { data-design-note="sealed" },
                    chat_note()
                    chat_thread(
                        user: &ada,
                        messages: &sealed,
                        lines: &sealed_lines,
                        plain_mark: true
                    )
                    foot(
                        attrs: attributes! { data-design-block="ready" },
                        chat_footer(handle: HANDLE, name: NAME)
                    )
                )
                frame(
                    attrs: attributes! { data-design-note="unencrypted" },
                    chat_note()
                    chat_thread(user: &ada, messages: &plain, plain_mark: true)
                )
                frame(
                    attrs: attributes! { data-design-note="remote" },
                    chat_note()
                    chat_thread(user: &ada, messages: &plain[..3])
                )
                frame(
                    attrs: attributes! { data-design-note="sealed" },
                    chat_note()
                    chat_thread(user: &ada, messages: &sealed, lines: &opened_lines)
                    foot(
                        attrs: attributes! { data-design-block="changed" },
                        chat_footer(
                            handle: HANDLE,
                            name: NAME,
                            numbers: Some((sample::SAFETY_OLD, sample::SAFETY_NEW))
                        )
                    )
                )
                frame(
                    foot(
                        attrs: attributes! { data-design-block="setup" },
                        chat_footer(handle: HANDLE, name: NAME)
                    )
                    foot(
                        attrs: attributes! { data-design-block="unlock" },
                        chat_footer(handle: HANDLE, name: NAME)
                    )
                )
                frame(
                    foot(
                        attrs: attributes! { data-design-block="peer-off" },
                        chat_footer(handle: HANDLE, name: NAME)
                    )
                    foot(
                        attrs: attributes! { data-design-block="downgrade" },
                        chat_footer(handle: HANDLE, name: NAME)
                    )
                    foot(
                        attrs: attributes! { data-design-block="server" },
                        chat_footer(handle: HANDLE, name: NAME, blocked_reason: BLOCKED)
                    )
                )
                row(
                    button(
                        variant: ButtonVariant::Outline,
                        attrs: attributes! { command="show-modal" commandfor=(SAFETY_ID) },
                        "Safety number dialog"
                    )
                )
                safety_dialog(name: NAME)
            </div>
        )
    })
}

#[component]
async fn rows_and_settings(cx: &Cx) -> Result<impl View> {
    let mut sealed = sample::conversations().swap_remove(0);
    sealed.peer_key = Some("sample-key".to_owned());
    sealed.last = sample::sealed_chat()[2].clone();
    let mut read = sealed.clone();
    read.unread = 0;
    let me = sample::sample_me().user.id.to_string();
    Ok(view! {
        section(
            id: "e2ee-row",
            title: "Encrypted ConversationRow",
            note: "lock mark; opened text or the encrypted placeholder",
            frame(
                conversation_row(
                    conversation: &sealed,
                    opened: "Nice. Photos stay untouched, right?"
                )
                conversation_row(conversation: &read)
            )
        )
        section(
            id: "e2ee-settings",
            title: "Encrypted messages settings",
            note: "off, locked, on; reset opens a dialog that asks for a new passphrase",
            frame(
                stack: true,
                encryption_cards(me: &me, status: "none", prefix: "e2ee-none")
            )
            frame(
                stack: true,
                encryption_cards(me: &me, status: "locked", prefix: "e2ee-locked")
            )
            frame(
                stack: true,
                encryption_cards(me: &me, status: "ready", prefix: "e2ee-ready")
            )
        )
    })
}

#[component]
pub(crate) async fn encryption(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        prompts()
        threads()
        rows_and_settings()
    })
}
