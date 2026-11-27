//! The framed block every setting sits in.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, component, element_id, view},
};

/// A `<section>` named by its `<h2>` with an optional sentence under it (empty for none) and the
/// content. Text props are owned, so a caller can pass a fresh `t!(..).resolve(cx).into_owned()`.
///
/// W4c's encryption section renders the same markup by hand; keep the two in step.
#[component]
pub(crate) async fn setting_card(
    cx: &Cx,
    #[into] title: String,
    #[default]
    #[into]
    description: String,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let id = element_id(cx);
    Ok(view! {
        <section
            class="card"
            aria-labelledby=(format!("{id}-title"))
            data-c="setting-card"
        >
            <h2 id=(format!("{id}-title"))>(title)</h2>
            if !description.is_empty() {
                <p class="description">(description)</p>
            }
            (child)
        </section>
    })
}
