//! `/admin`: the overview tab. The other tabs are `/admin/{tab}`: two routes instead of an
//! optional parameter.

mod tab;

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::page,
    view::{View, view},
};

use crate::admin::{
    page::{admin_page, title},
    rules::AdminTab,
};

#[page(with(Metadata::from_fn(async |cx| title(cx, AdminTab::Overview).await)))]
async fn admin_overview_page(cx: &Cx) -> Result<impl View> {
    Ok(view! { admin_page(tab: AdminTab::Overview) })
}
