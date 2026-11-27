//! Fragment routes of the messages area. No layout; the `/_tuita` layer gives a guest an empty
//! `401` and turns an API failure into an empty answer with its status.

use topcoat::{
    Result,
    context::Cx,
    router::{query_params, route},
    view::{ViewExt, ViewHandle, view},
};
use tuita_server::dto::SearchDto;

use crate::{api, messages::people::people_rows, session};

#[query_params(error = bad_request)]
struct PeopleQuery {
    q: Option<String>,
}

/// The local people matching `q`, other than the viewer, as `<li>` rows for a picker. Remote
/// accounts are left out: they have no keys, and the picker is for starting an encrypted chat.
#[route(GET "/_tuita/chat-people")]
async fn chat_people(cx: &Cx) -> Result<ViewHandle> {
    let query = query_params::<PeopleQuery>(cx)?;
    let text = query.q.as_deref().map(str::trim).unwrap_or_default();
    if text.is_empty() {
        return view! { "" }.first().await;
    }
    let me = session::member(cx).await?.me.user.id;
    let found = api::get::<SearchDto>(
        cx,
        &format!(
            "/search{}",
            api::query(&[("q", Some(text)), ("type", Some("people"))])
        ),
    )
    .await?;
    let users: Vec<_> = found
        .users
        .into_iter()
        .filter(|user| user.domain.is_none() && user.id != me)
        .collect();
    view! { people_rows(users: &users) }.first().await
}
