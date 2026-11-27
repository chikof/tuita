//! The dialog that shows a new token once.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal},
    view::{View, attributes, component, view},
};

use crate::{
    l10n::t,
    ui::{
        button::{ButtonVariant, button},
        dialog::dialog,
    },
};

/// The id of the dialog; open it with `showModal()` after setting the `secret` signal.
pub(crate) const DIALOG: &str = "token-secret";

/// The warning, the token in a read-only box and Copy and Done buttons. `id` names the dialog
/// (default [`DIALOG`]); give another one when the page holds a second element with that id.
///
/// `settings.js` copies the box on `[data-copy]`, selects it on focus and blanks it when the dialog
/// closes. The secret lives only in the browser's `secret` signal; the page never renders it.
#[component]
pub(crate) async fn token_secret(
    cx: &Cx,
    secret: &Signal<String>,
    #[default]
    #[into]
    id: Option<&str>,
) -> Result<impl View> {
    let dialog_id = id.unwrap_or(DIALOG);
    let title = t!(token_secret_title).resolve(cx).into_owned();
    let box_id = format!("{dialog_id}-value");
    let copy_target = format!("#{box_id}");
    Ok(view! {
        dialog(
            id: dialog_id,
            title: &title,
            actions: Some(
                view! {
                    button(
                        variant: ButtonVariant::Outline,
                        attrs: attributes! { data-copy=(copy_target) },
                        (t!(token_copy))
                    )
                    button(
                        attrs: attributes! {
                            command="close"
                            commandfor=(dialog_id)
                            @click=$(|_e: Event| secret.set("".to_owned()))
                        },
                        (t!(token_done))
                    )
                }.into(

                ),
            ),
            <div class="body" data-c="token-secret">
                <p class="warning" role="note">(t!(token_secret_warning))</p>
                <label for=(&box_id) class="label">(t!(token_secret_label))</label>
                <textarea
                    id=(&box_id)
                    class="secret"
                    readonly=(true)
                    rows="2"
                    spellcheck="false"
                    autocomplete="off"
                    data-select=""
                    :value=$(secret.get())
                ></textarea>
            </div>
        )
    })
}
