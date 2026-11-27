//! The compose dialog the shell opens.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use super::composer::composer;
use crate::{
    l10n::t,
    ui::dialog::{DialogVariant, dialog},
};

/// The id of the compose dialog: `composer.js` opens `dialog#compose`.
pub(crate) const ID: &str = "compose";

/// A modal dialog holding a [`composer`] with no fixed target.
///
/// It is rendered once on every page of a signed-in visitor (`shell::globals`). `composer.js`
/// opens it on a click of `[data-compose]` (`new`, `reply` or `quote` with `data-post-id`),
/// fills the target into the box, retitles it (`#compose-title`) and calls `showModal()`.
#[component]
pub(crate) async fn compose_dialog(cx: &Cx) -> Result<impl View> {
    let title = t!(composer_title).resolve(cx).into_owned();
    Ok(view! {
        dialog(
            id: ID,
            title: &title,
            variant: DialogVariant::Dialog,
            composer(in_dialog: true, autofocus: true)
        )
    })
}
