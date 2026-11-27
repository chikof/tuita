//! `/lists/{list_id}/members`.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::page,
    view::{View, view},
};

use super::{list_id, title};
use crate::lists::pages::list_members;

#[page(with(Metadata::from_fn(async |cx| title(cx, list_id(cx)).await)))]
async fn list_members_page(cx: &Cx) -> Result<impl View> {
    Ok(view! { list_members(id: list_id(cx)) })
}
