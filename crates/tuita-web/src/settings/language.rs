//! `/settings/language`.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::{
    auth::language_picker::language_picker,
    l10n::t,
    settings::{card::setting_card, page::settings_page},
};

/// The language select; choosing one saves it on the account and in the cookie and reloads.
#[component]
pub(crate) async fn language(cx: &Cx) -> Result<impl View> {
    let title = t!(settings_language).resolve(cx).into_owned();
    let note = t!(settings_language_note).resolve(cx).into_owned();
    Ok(view! {
        settings_page(
            title: &title,
            setting_card(title: &title, description: &note, language_picker())
        )
    })
}
