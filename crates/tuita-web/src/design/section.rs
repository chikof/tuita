//! A titled gallery section and a labelled row of items.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, component, view},
};

/// `<section id>` with a heading and an optional mono note. The e2e anchor test looks for
/// `section#<id>` for every entry of the component index.
#[component]
pub(crate) async fn section(
    cx: &Cx,
    id: &str,
    title: &str,
    #[default]
    #[into]
    note: Option<&str>,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <section id=(id) class="section" data-c="design-section">
            <header>
                <h2>(title)</h2>
                if let Some(note) = note {
                    <p class="mono-tag">(note)</p>
                }
            </header>
            <div class="content">(child)</div>
        </section>
    })
}

/// A row of items under an optional mono label.
#[component]
pub(crate) async fn row(
    cx: &Cx,
    #[default]
    #[into]
    label: Option<&str>,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <div class="row" data-c="design-row">
            if let Some(label) = label {
                <span class="mono-tag">(label)</span>
            }
            <div class="items">(child)</div>
        </div>
    })
}
