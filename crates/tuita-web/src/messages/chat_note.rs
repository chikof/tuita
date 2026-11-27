//! The line under a conversation's header.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::{icons::Icon, l10n::t, messages::safety_dialog::SAFETY_ID, ui::icon::icon};

/// Every variant of the note, hidden. `messages.js` shows the one `conversationState` picks:
/// `[data-note="sealed"]` (end-to-end encrypted, with the safety number button once both keys are
/// known), `[data-note="remote"]` and `[data-note="unencrypted"]`.
#[component]
pub(crate) async fn chat_note(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        <div data-c="chat-note">
            <p class="note sealed" data-note="sealed" hidden="">
                icon(name: Icon::Lock, size: 16)
                <span>(t!(e2ee_note_sealed))</span>
                <button
                    type="button"
                    class="link"
                    command="show-modal"
                    commandfor=(SAFETY_ID)
                    data-safety=""
                    hidden=""
                >
                    (t!(e2ee_safety_open))
                </button>
            </p>
            <p class="note plain" data-note="remote" hidden="">
                icon(name: Icon::WarningDiamond, size: 16)
                <span>(t!(e2ee_note_remote))</span>
            </p>
            <p class="note plain" data-note="unencrypted" hidden="">
                icon(name: Icon::WarningDiamond, size: 16)
                <span>(t!(e2ee_note_plain))</span>
            </p>
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn every_variant_is_there_and_hidden_for_the_island_to_pick() {
        let html = render("en", |cx| view! { chat_note() }.boxed());
        for kind in ["sealed", "remote", "unencrypted"] {
            assert!(
                html.contains(&format!(r#"data-note="{kind}" hidden"#)),
                "{kind}: {html}"
            );
        }
        assert!(html.contains("End-to-end encrypted"), "{html}");
        assert!(html.contains(r#"commandfor="safety-number""#), "{html}");
    }
}
