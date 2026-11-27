//! `/@{handle}/following`.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::{page, query_params},
    view::{View, ViewExt, view},
};

use super::{handle, profile, title};
use crate::{
    fragment,
    profile::{
        follow_list::{Which, follow_fragment, follow_list},
        load_user::Profile,
        remote_lookup::remote_page,
    },
};

#[query_params(error = bad_request)]
struct ListQuery {
    before: Option<String>,
}

#[page(with(Metadata::from_fn(async |cx| title(cx).await)))]
async fn following_page(cx: &Cx) -> Result<impl View> {
    let query = query_params::<ListQuery>(cx)?;
    let handle = handle(cx)?;
    if fragment::requested(cx) {
        return Ok(view! {
            follow_fragment(
                acct: handle.acct(),
                list: Which::Following,
                before: query.before.as_deref()
            )
        }
        .boxed());
    }
    Ok(match profile(cx).await? {
        Profile::Found(user) => view! {
            follow_list(
                user: user,
                list: Which::Following,
                before: query.before.as_deref()
            )
        }
        .boxed(),
        Profile::Failed(failure) => {
            let text = handle.to_string();
            view! { remote_page(handle: &text, failure: failure) }.boxed()
        }
    })
}
