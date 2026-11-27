//! An on/off setting that saves itself.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, signal},
    view::{View, attributes, component, element_id, view},
};

use crate::{settings::actions::save_toggle, ui::switch::switch};

/// A switch named `label` with `hint` under it. A click flips it at once and saves `field` (see
/// [`save_toggle`]); a failed save flips it back and toasts the API's message.
#[component]
pub(crate) async fn setting_switch(
    cx: &Cx,
    field: &str,
    label: &str,
    hint: &str,
    on: bool,
) -> Result<impl View> {
    let checked = signal(cx, move || on);
    let saved = signal(cx, move || on);
    let id = element_id(cx);
    Ok(view! {
        <div
            class="row"
            data-c="setting-switch"
            @click=$(async |_e: Event| {
                let next = checked.get();
                if next != saved.get() {
                    saved.set(next);
                    let done = save_toggle(field.to_owned(), next).await;
                    if done.is_ok() {
                        let msg = done.unwrap();
                        raw!(
                            "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                            msg.is_empty(),
                        )
                    } else {
                        checked.set(!next);
                        saved.set(!next);
                        let msg = done.unwrap_err();
                        raw!(
                            "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                            msg.is_empty(),
                        )
                    };
                }
            })
        >
            switch(
                label: label,
                checked: &checked,
                attrs: attributes! { aria-describedby=(format!("{id}-hint")) }
            )
            <p id=(format!("{id}-hint")) class="hint">(hint)</p>
        </div>
    })
}
