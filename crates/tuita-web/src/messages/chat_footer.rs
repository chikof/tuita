//! The composer of a conversation and what replaces it.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};

use crate::{
    icons::Icon,
    l10n::{strip_isolates, t},
    messages::{
        e2ee_notice::{Tone, e2ee_notice},
        e2ee_prompt::{PromptKind, e2ee_prompt},
    },
    ui::{
        button::{ButtonSize, ButtonVariant, button},
        icon::icon,
    },
};

/// The longest message the API takes.
pub(crate) const MAX: u32 = 2000;

/// The footer with every variant. `messages.js` computes `conversationState` and shows the right
/// one: `[data-block="setup"|"unlock"]` are the passphrase prompts, `[data-block="changed"]` the
/// key-change warning (with the two safety numbers in `[data-old]` and `[data-new]`), and
/// `[data-block="server|peer-off|self-off|downgrade|error"]` plain reasons. The form is disabled
/// until a state lets a message go out.
///
/// `handle` is `@ada`, `name` the display name, `blocked_reason` what the server said. `numbers`
/// (trusted, served) fills the key-change warning for a page that knows them; the app has
/// `messages.js` compute them.
#[component]
pub(crate) async fn chat_footer(
    cx: &Cx,
    handle: &str,
    name: &str,
    #[default]
    #[into]
    blocked_reason: Option<&str>,
    #[default] numbers: Option<(&str, &str)>,
) -> Result<impl View> {
    let strip = |text: std::borrow::Cow<'_, str>| strip_isolates(&text);
    let peer_off = strip(t!(e2ee_block_peer_off, handle: handle).resolve(cx));
    let downgrade = strip(t!(e2ee_block_downgrade, handle: handle).resolve(cx));
    let changed_title = strip(t!(e2ee_changed_title, name: name).resolve(cx));
    let placeholder = t!(chat_placeholder).resolve(cx).into_owned();
    let label = t!(chat_input_label).resolve(cx).into_owned();
    let send = t!(chat_send).resolve(cx).into_owned();
    Ok(view! {
        <div class="composer" data-c="chat-footer">
            <div data-block="setup" hidden="">e2ee_prompt(kind: PromptKind::Setup)</div>
            <div data-block="unlock" hidden="">
                e2ee_prompt(kind: PromptKind::Unlock)
            </div>
            <div class="rest">
                e2ee_notice(
                    tone: Tone::Warning,
                    title: changed_title.as_str(),
                    attrs: attributes! { data-block="changed" hidden="" },
                    actions: Some(
                        view! {
                            button(
                                size: ButtonSize::Sm,
                                attrs: attributes! { data-act="chat-trust" },
                                (t!(e2ee_trust))
                            )
                        }.into(

                        ),
                    ),
                    <p>(t!(e2ee_changed_body))</p>
                    <dl data-numbers="" hidden=(numbers.is_none())>
                        <dt>(t!(e2ee_changed_old))</dt>
                        <dd data-old="">(numbers.map_or("", |(old, _)| old))</dd>
                        <dt>(t!(e2ee_changed_new))</dt>
                        <dd data-new="">(numbers.map_or("", |(_, new)| new))</dd>
                    </dl>
                )
                <p class="note" role="status" data-block="server" hidden="">
                    (blocked_reason.unwrap_or_default())
                </p>
                <p class="note" role="status" data-block="peer-off" hidden="">
                    (&peer_off)
                </p>
                <p class="note" role="status" data-block="self-off" hidden="">
                    (t!(e2ee_block_self_off))
                </p>
                <p class="note" role="status" data-block="downgrade" hidden="">
                    (&downgrade)
                </p>
                <div data-block="error" hidden="">
                    <p class="note" role="status">(t!(e2ee_block_error))</p>
                    button(
                        size: ButtonSize::Sm,
                        variant: ButtonVariant::Outline,
                        attrs: attributes! { data-act="e2ee-retry" },
                        (t!(retry))
                    )
                </div>
                <form data-chat-form="">
                    <input
                        type="text"
                        name="text"
                        maxlength=(MAX)
                        disabled=""
                        placeholder=(&placeholder)
                        aria-label=(&label)
                        autocomplete="off"
                    >
                    <button
                        type="submit"
                        class="icon-button ghost md wash"
                        aria-label=(&send)
                        data-c="icon-button"
                        disabled=""
                    >
                        icon(name: Icon::Send, size: 24)
                    </button>
                </form>
            </div>
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn the_form_starts_disabled_and_every_block_is_there_hidden() {
        let html = render("en", |cx| {
            view! {
                chat_footer(
                    handle: "@ada",
                    name: "Ada",
                    blocked_reason: "You blocked them."
                )
            }
            .boxed()
        });
        assert!(html.contains(r#"aria-label="Message""#), "{html}");
        assert!(html.contains(r#"aria-label="Send""#), "{html}");
        for kind in [
            "setup",
            "unlock",
            "changed",
            "server",
            "peer-off",
            "self-off",
            "downgrade",
            "error",
        ] {
            assert!(
                html.contains(&format!(r#"data-block="{kind}""#)),
                "{kind}: {html}"
            );
        }
        assert!(
            html.contains("@ada hasn’t turned on encrypted messages yet."),
            "{html}"
        );
        assert!(html.contains("You blocked them."), "{html}");
        assert!(html.contains("Ada’s encryption key changed"), "{html}");
        assert!(html.contains("Trust new key"), "{html}");
    }
}
