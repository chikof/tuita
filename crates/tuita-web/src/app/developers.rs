//! `/developers`.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::page,
    view::{View, view},
};

use crate::docs::developers::{developers, title};

#[page(with(Metadata::from_fn(async |cx| title(cx).await)))]
async fn developers_page(cx: &Cx) -> Result<impl View> {
    Ok(view! { developers() })
}
