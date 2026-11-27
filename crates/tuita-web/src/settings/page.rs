//! The frame of a section page: a back bar and a stack of cards.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, component, view},
};

use crate::shell::page_bar::page_bar;

/// The page bar with a back button, then the cards.
#[component]
pub(crate) async fn settings_page(
    cx: &Cx,
    #[into] title: String,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        page_bar(title: &title, back: true)
        <div class="stack" data-c="settings-page">(child)</div>
    })
}
