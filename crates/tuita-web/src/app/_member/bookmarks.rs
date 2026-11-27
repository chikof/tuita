//! `/bookmarks`.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::page,
    view::{View, view},
};

use crate::{feed::bookmarks::bookmarks, l10n::t, shell};

async fn title(cx: &Cx) -> Result<Metadata> {
    let text = t!(history_title).resolve(cx).into_owned();
    shell::titled(cx, &text).await
}

#[page(with(Metadata::from_fn(async |cx| title(cx).await)))]
async fn bookmarks_page(cx: &Cx) -> Result<impl View> {
    Ok(view! { bookmarks() })
}
