//! `/design`: the component gallery. No guard, no shell.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::page,
    view::{View, view},
};

use crate::design::page::gallery;

#[page(with(Metadata::new().title("Design system · tuita")))]
async fn design_page(cx: &Cx) -> Result<impl View> {
    Ok(view! { gallery() })
}
