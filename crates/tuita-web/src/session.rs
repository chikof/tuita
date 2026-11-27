//! Who is looking and what instance this is, loaded once per request.

use topcoat::{
    Result,
    context::{Cx, memoize},
    router::{error::see_other, query_params, request::uri},
};
use tuita_server::dto::{AccountsDto, BadgesDto, InstanceDto, MeDto, MeResponse, Role};

use crate::{api, next};

/// The signed-in account with what the shell needs around it.
#[derive(Clone, Debug)]
pub(crate) struct Viewer {
    pub(crate) me: MeDto,
    pub(crate) accounts: AccountsDto,
    pub(crate) badges: BadgesDto,
}

/// `GET /instance`.
#[memoize(as_ref)]
pub(crate) async fn instance(cx: &Cx) -> Result<InstanceDto> {
    Ok(api::get::<InstanceDto>(cx, "/instance").await?)
}

/// `GET /me`: the signed-in account, or `None` for a guest. Cheaper than [`viewer`]; the guards
/// and the locale only need this.
#[memoize(as_ref)]
pub(crate) async fn me(cx: &Cx) -> Result<Option<MeDto>> {
    match api::get::<MeResponse>(cx, "/me").await {
        Ok(MeResponse { user }) => Ok(user),
        Err(error) if error.status == axum::http::StatusCode::UNAUTHORIZED => Ok(None),
        Err(error) => Err(error.into()),
    }
}

/// The signed-in account, its other accounts and its unread counts; `None` for a guest.
#[memoize(as_ref)]
pub(crate) async fn viewer(cx: &Cx) -> Result<Option<Viewer>> {
    let Some(me) = me(cx).await? else {
        return Ok(None);
    };
    let (accounts, badges) = tokio::try_join!(
        api::get::<AccountsDto>(cx, "/accounts"),
        api::get::<BadgesDto>(cx, "/badges"),
    )?;
    Ok(Some(Viewer {
        me: me.clone(),
        accounts,
        badges,
    }))
}

/// The signed-in viewer, or a `303` to `/login?next=<this page>`.
///
/// # Errors
///
/// Redirects a guest; fails when the API does.
pub(crate) async fn member(cx: &Cx) -> Result<&Viewer> {
    match viewer(cx).await? {
        Some(viewer) => Ok(viewer),
        None => {
            let uri = uri(cx);
            Err(see_other(next::login_url(uri.path(), uri.query())).into())
        }
    }
}

/// Moderators and admins.
#[must_use]
pub(crate) fn is_staff(viewer: &Viewer) -> bool {
    viewer.me.role != Role::User
}

#[query_params(error = bad_request)]
struct SignInQuery {
    add: Option<String>,
    next: Option<String>,
}

/// For `/login` and `/signup`: a signed-in visitor without `?add` goes to `?next=` (when it is a
/// safe path) or `/` instead of seeing the form.
///
/// # Errors
///
/// A `303` for a signed-in visitor; fails when the API does.
pub(crate) async fn leave_sign_in_pages(cx: &Cx) -> Result<()> {
    let query = topcoat::router::query_params::<SignInQuery>(cx)?;
    if query.add.is_none() && me(cx).await?.is_some() {
        let target = next::safe_next(query.next.as_deref()).unwrap_or_else(|| "/".to_owned());
        return Err(see_other(target).into());
    }
    Ok(())
}
