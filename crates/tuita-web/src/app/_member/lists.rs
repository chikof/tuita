//! `/lists`, `/lists/{list_id}`, `/lists/{list_id}/members`.

mod list_id;

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::page,
    view::{View, view},
};

use crate::{l10n::t, lists::pages::lists, shell};

async fn title(cx: &Cx) -> Result<Metadata> {
    let text = t!(lists_title).resolve(cx).into_owned();
    shell::titled(cx, &text).await
}

#[page(with(Metadata::from_fn(async |cx| title(cx).await)))]
async fn lists_page(cx: &Cx) -> Result<impl View> {
    Ok(view! { lists() })
}
