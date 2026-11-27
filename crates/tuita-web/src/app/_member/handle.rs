//! `/@{handle}`: a root parameter whose type keeps the leading `@` and the handle pattern.
//! A mixed `@{handle}` segment is impossible in module routing, so the module is `handle`.
//!
//! A local account is loaded with the page. A remote one (`@ada@remote.example`) is looked up by
//! the API while the request waits; when that fails the page says why and offers a retry.

mod followers;
mod following;
mod post_id;

use topcoat::{
    Result,
    context::{Cx, memoize},
    metadata::Metadata,
    router::{module_param, page, query_params},
    view::{View, ViewExt, view},
};
use tuita_server::dto::UserDto;

use crate::{
    api::{self, enc},
    fmt::handle_of,
    fragment,
    l10n::{strip_isolates, t},
    paths::Handle,
    profile::{
        load_user::{self, Profile},
        profile_view::{MediaFilter, profile_fragment, profile_view, tab_of},
        remote_lookup::remote_page,
    },
    shell,
};

module_param!(pub(crate) handle: Handle else not_found);

/// The account of the page: `GET /users/<acct>`, once per request. A failed lookup of a remote
/// account is a [`Profile::Failed`] the page explains; any other failure is an error of the page.
#[memoize(as_ref)]
pub(crate) async fn profile(cx: &Cx) -> Result<Profile> {
    let acct = handle(cx)?.acct();
    let result = api::get::<UserDto>(cx, &format!("/users/{}", enc(acct))).await;
    Ok(load_user::settle(acct, result)?)
}

/// The tab title: `Name (@handle) · tuita`, or the handle while the account is unknown.
pub(crate) async fn title(cx: &Cx) -> Result<Metadata> {
    let Ok(profile) = profile(cx).await else {
        return Ok(Metadata::new().title(t!(app_name).resolve(cx).into_owned()));
    };
    let text = match profile {
        Profile::Found(user) => {
            let handle = strip_isolates(&handle_of(&user.handle, user.domain.as_deref()));
            format!("{} ({handle})", user.display_name)
        }
        Profile::Failed(_) => handle(cx)?.to_string(),
    };
    shell::titled(cx, &text).await
}

#[query_params(error = bad_request)]
struct ProfileQuery {
    tab: Option<String>,
    media: Option<String>,
    before: Option<String>,
}

#[page(with(Metadata::from_fn(async |cx| title(cx).await)))]
async fn profile_page(cx: &Cx) -> Result<impl View> {
    let query = query_params::<ProfileQuery>(cx)?;
    let tab = tab_of(query.tab.as_deref());
    let kind = MediaFilter::parse(query.media.as_deref());
    let handle = handle(cx)?;
    if fragment::requested(cx) {
        return Ok(view! {
            profile_fragment(
                acct: handle.acct(),
                tab: tab,
                kind: kind,
                before: query.before.as_deref()
            )
        }
        .boxed());
    }
    let page = match profile(cx).await? {
        Profile::Found(user) => view! {
            profile_view(
                user: user,
                tab: tab,
                kind: kind,
                before: query.before.as_deref()
            )
        }
        .boxed(),
        Profile::Failed(failure) => {
            let text = handle.to_string();
            view! { remote_page(handle: &text, failure: failure) }.boxed()
        }
    };
    Ok(page)
}
