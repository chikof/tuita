//! Fragment requests: `infinite.js` and the other islands fetch parts of a page with the header
//! `x-tuita-fragment: 1`; the layout then renders only the page, and a page only its list items.

use topcoat::{context::Cx, router::request::headers};

/// The request header that asks for a fragment.
pub(crate) const HEADER: &str = "x-tuita-fragment";

/// True when the request carries `x-tuita-fragment: 1`.
#[must_use]
pub(crate) fn requested(cx: &Cx) -> bool {
    headers(cx)
        .get(HEADER)
        .is_some_and(|value| value.as_bytes() == b"1")
}

/// Wraps every `/_tuita/*` fragment route: a guest gets an empty `401`, and an API failure
/// inside the route becomes an empty answer with the API's status.
#[topcoat::router::layer("/_tuita")]
async fn guard(
    cx: &Cx,
    body: topcoat::router::Body,
    next: topcoat::router::Next<'_>,
) -> topcoat::Result<topcoat::router::response::Response> {
    if crate::session::me(cx).await?.is_none() {
        return Ok(empty(topcoat::router::StatusCode::UNAUTHORIZED));
    }
    match next.run(cx, body).await {
        Err(error) => match error.downcast_ref::<crate::api::ApiError>() {
            Some(api) => Ok(empty(api.status)),
            None => Err(error),
        },
        ok => ok,
    }
}

fn empty(status: topcoat::router::StatusCode) -> topcoat::router::response::Response {
    let mut response = topcoat::router::response::Response::new(topcoat::router::Body::empty());
    *response.status_mut() = status;
    response
}
