//! The desktop notification permission.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};

use crate::{
    l10n::t,
    settings::card::setting_card,
    ui::button::{ButtonVariant, button},
};

/// The permission state and a button to ask for it.
///
/// `stream.js` runs it (`[data-island="desktop-notifications"]`): it sets `data-state`, hides the
/// button once granted or when the browser cannot notify, shows the `data-when` parts that match
/// the state and writes the localized "Permission: ..." text into `[data-state-label]`.
#[component]
pub(crate) async fn desktop_notifications(cx: &Cx) -> Result<impl View> {
    let title = t!(settings_desktop_title).resolve(cx).into_owned();
    let description = t!(settings_desktop_desc).resolve(cx).into_owned();
    Ok(view! {
        setting_card(
            title: &title,
            description: &description,
            <div
                data-c="desktop-notifications"
                data-island="desktop-notifications"
                data-state="default"
            >
                <div class="row" data-when="default granted denied">
                    <p class="state" data-state-label=""></p>
                    button(
                        variant: ButtonVariant::Outline,
                        attrs: attributes! { data-action="enable" },
                        (t!(settings_desktop_enable))
                    )
                </div>
                <p class="note" data-when="unsupported" hidden="">
                    (t!(settings_desktop_unsupported))
                </p>
            </div>
        )
    })
}
