//! A titled surface a group of fields or a list sits in.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, component, element_id, view},
};

/// A bordered section named by its title, with an optional description under it.
#[component]
pub(crate) async fn setting_card(
    cx: &Cx,
    title: &str,
    #[default]
    #[into]
    description: Option<&str>,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let id = element_id(cx);
    Ok(view! {
        <section
            class="card"
            aria-labelledby=(format!("{id}-title"))
            data-c="admin-setting-card"
        >
            <h2 id=(format!("{id}-title"))>(title)</h2>
            if let Some(description) = description {
                <p class="description">(description)</p>
            }
            (child)
        </section>
    })
}
