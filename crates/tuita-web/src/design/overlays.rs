//! Gallery sections: dialog, sheet, toast.
//!
//! The demo buttons use the native invoker attributes; the toast buttons are plain
//! `[data-design-act="toast"]` hooks handled by the gallery script (`design-parts/core.js`), which
//! dispatches the `tuita-toast` event `toast.js` listens for.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};

use super::section::{row, section};
use crate::ui::{
    button::{ButtonVariant, button},
    dialog::dialog,
    sheet::sheet,
};

#[component]
pub(crate) async fn overlays(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        section(
            id: "dialog",
            title: "Dialog",
            note: "native dialog; 50% checker backdrop; bottom sheet under 600px; Esc closes",
            row(
                button(
                    variant: ButtonVariant::Outline,
                    attrs: attributes! { command="show-modal" commandfor="core-dialog" },
                    "Open dialog"
                )
            )
            dialog(
                id: "core-dialog",
                title: "Delete this post?",
                actions: Some(
                    view! {
                        button(
                            variant: ButtonVariant::Ghost,
                            attrs: attributes! { command="close" commandfor="core-dialog" },
                            "Cancel"
                        )
                        button(
                            variant: ButtonVariant::Danger,
                            attrs: attributes! { command="close" commandfor="core-dialog" },
                            "Delete"
                        )
                    }.into(

                    ),
                ),
                <p>"This cannot be undone. Replies stay in place."</p>
            )
        )
        section(
            id: "sheet",
            title: "Sheet",
            note: "docks to the right edge; bottom sheet with drag handle under 600px",
            row(
                button(
                    variant: ButtonVariant::Outline,
                    attrs: attributes! { command="show-modal" commandfor="core-sheet" },
                    "Open sheet"
                )
            )
            sheet(
                id: "core-sheet",
                title: "Filters",
                <p>"Sheets hold longer forms and settings."</p>
            )
        )
        section(
            id: "toast",
            title: "Toast",
            note: "bottom-left stack; 5 s auto-dismiss; draining line; optional action",
            row(
                button(
                    variant: ButtonVariant::Outline,
                    attrs: attributes! {
                        data-design-act="toast"
                        data-message="Saved changes"
                        data-counter=""
                    },
                    "Show toast"
                )
                button(
                    variant: ButtonVariant::Outline,
                    attrs: attributes! {
                        data-design-act="toast"
                        data-message="Post deleted"
                        data-label="Undo"
                        data-href="#toast-undo"
                    },
                    "Toast with action"
                )
            )
        )
    })
}
