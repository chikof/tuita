//! `/login`.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::page,
    view::{View, view},
};

use crate::auth::login::{login_form, title};

#[page(with(Metadata::from_fn(async |cx| title(cx).await)))]
async fn login_page(cx: &Cx) -> Result<impl View> {
    crate::session::leave_sign_in_pages(cx).await?;
    Ok(view! { login_form() })
}
