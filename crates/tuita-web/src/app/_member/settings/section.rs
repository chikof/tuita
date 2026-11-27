//! `/settings/{section}`.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::{module_param, page},
    view::{View, view},
};

use crate::{
    settings::{Section, section_page},
    shell,
};

module_param!(pub(crate) section: Section else not_found);

#[page(
    with(Metadata::from_fn(
        async |cx| {
            let title = section(cx)?.title(cx);
            shell::titled(cx, &title).await
        },
    ))
)]
async fn settings_section_page(cx: &Cx) -> Result<impl View> {
    let section = *section(cx)?;
    Ok(view! { section_page(section: section) })
}
