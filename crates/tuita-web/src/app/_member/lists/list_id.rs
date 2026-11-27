//! `/lists/{list_id}`.

mod members;

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::{module_param, page},
    view::{View, view},
};

use crate::{
    api::{self, enc},
    l10n::t,
    lists::pages::list_posts,
    shell,
};
use tuita_server::dto::ListDto;

module_param!(pub(crate) list_id);

/// The tab title: the list's name, or "Lists" when it cannot be read.
pub(super) async fn title(cx: &Cx, id: &str) -> Result<Metadata> {
    let name = match api::get::<ListDto>(cx, &format!("/lists/{}", enc(id))).await {
        Ok(list) => list.name,
        Err(_) => t!(lists_title).resolve(cx).into_owned(),
    };
    shell::titled(cx, &name).await
}

#[page(with(Metadata::from_fn(async |cx| title(cx, list_id(cx)).await)))]
async fn list_page(cx: &Cx) -> Result<impl View> {
    Ok(view! { list_posts(id: list_id(cx)) })
}
