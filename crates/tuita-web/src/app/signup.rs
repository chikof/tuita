//! `/signup`.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::page,
    view::{View, view},
};

use crate::auth::signup::{signup_form, title};

#[page(with(Metadata::from_fn(async |cx| title(cx).await)))]
async fn signup_page(cx: &Cx) -> Result<impl View> {
    crate::session::leave_sign_in_pages(cx).await?;
    Ok(view! { signup_form() })
}
