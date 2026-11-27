//! `/settings` and `/settings/{section}`: two routes instead of an optional parameter.

mod section;

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::page,
    view::{View, view},
};

use crate::{l10n::t, session, settings::index::index, shell};

#[page(
    with(Metadata::from_fn(
        async |cx| {
            let title = t!(nav_settings).resolve(cx).into_owned();
            shell::titled(cx, &title).await
        },
    ))
)]
async fn settings_page(cx: &Cx) -> Result<impl View> {
    let viewer = session::member(cx).await?;
    Ok(view! { index(me: &viewer.me) })
}
