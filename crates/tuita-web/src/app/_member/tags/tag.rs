//! `/tags/{tag}`.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::{module_param, page},
    view::{View, view},
};

use crate::{
    feed::tags::{label, tag_posts},
    shell,
};

module_param!(pub(crate) tag);

#[page(with(Metadata::from_fn(async |cx| shell::titled(cx, &label(tag(cx))).await)))]
async fn tag_page(cx: &Cx) -> Result<impl View> {
    Ok(view! { tag_posts(tag: tag(cx)) })
}
