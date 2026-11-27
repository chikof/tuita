//! `/scheduled`.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::page,
    view::{View, view},
};

use crate::{feed::scheduled::scheduled, l10n::t, shell};

async fn title(cx: &Cx) -> Result<Metadata> {
    let text = t!(scheduled_title).resolve(cx).into_owned();
    shell::titled(cx, &text).await
}

#[page(with(Metadata::from_fn(async |cx| title(cx).await)))]
async fn scheduled_page(cx: &Cx) -> Result<impl View> {
    Ok(view! { scheduled() })
}
