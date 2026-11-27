//! `/admin/{tab}`: one tab of the console; an unknown name is a 404.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::{module_param, page},
    view::{View, view},
};

use crate::admin::{
    page::{admin_page, title},
    rules::AdminTab,
};

module_param!(pub(crate) tab: AdminTab else not_found);

#[page(with(Metadata::from_fn(async |cx| title(cx, *tab(cx)?).await)))]
async fn admin_tab_page(cx: &Cx) -> Result<impl View> {
    let tab = *tab(cx)?;
    Ok(view! { admin_page(tab: tab) })
}
