//! `/terms`.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::page,
    view::{View, view},
};

use crate::docs::doc_page::{DocKind, doc_page, title};

#[page(with(Metadata::from_fn(async |cx| title(cx, DocKind::Terms).await)))]
async fn terms_page(cx: &Cx) -> Result<impl View> {
    Ok(view! { doc_page(kind: DocKind::Terms) })
}
