//! The toast stack.
//!
//! The server renders the empty region. `toast.js` adds a `div.toast[role=status]` for every
//! `tuita-toast` event and for every `output[data-toast]` element in the page.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::{l10n::t, ui::mask::level_mask};

/// The `#toaster` live region, fixed at the bottom of the viewport. `--m25` and `--m50` are the
/// dither masks the buttons that `toast.js` builds inherit.
#[component]
pub(crate) async fn toaster(cx: &Cx) -> Result<impl View> {
    let style = format!("--m25:{};--m50:{}", level_mask(0.25), level_mask(0.5));
    Ok(view! {
        <section
            id="toaster"
            class="toaster"
            data-c="toaster"
            aria-label=(t!(notifications))
            style=(style)
        ></section>
    })
}
