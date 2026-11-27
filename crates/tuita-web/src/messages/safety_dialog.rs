//! The safety number of a conversation.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::{l10n::t, ui::dialog::dialog};

/// The id of the dialog; the "Safety number" button of the chat note opens it.
pub(crate) const SAFETY_ID: &str = "safety-number";

/// `<dialog id="safety-number">`. `messages.js` fills the fifteen groups of `ol.number` (each
/// side computes the number from the two public keys in the browser).
#[component]
pub(crate) async fn safety_dialog(cx: &Cx, name: &str) -> Result<impl View> {
    let title = t!(e2ee_safety_title).resolve(cx).into_owned();
    Ok(view! {
        dialog(
            id: SAFETY_ID,
            title: &title,
            <div data-c="safety-dialog">
                <p class="body">(t!(e2ee_safety_body, name: name))</p>
                <ol class="number" aria-label=(&title)></ol>
            </div>
        )
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn the_dialog_names_the_person_and_waits_for_the_number() {
        let html = render("en", |cx| view! { safety_dialog(name: "Ada") }.boxed());
        assert!(html.contains(r#"<dialog id="safety-number""#), "{html}");
        assert!(html.contains("Compare this number with"), "{html}");
        assert!(
            html.contains(r#"<ol class="number" aria-label="Safety number">"#),
            "{html}"
        );
    }
}
