//! The flows that change who is signed in or in which language: login, signup, logout, account
//! switching and the language choice.
//!
//! Each is a procedure: the browser calls it, the procedure calls the JSON API in-process and
//! appends the API answer's `Set-Cookie` headers to its own response, so the browser ends up with
//! the new session. The inner `Err` is the API's message for the form or a toast. The caller then
//! navigates itself (`location.assign`), because the page it is on belongs to the old session.

use axum::http::Method;
use serde_json::json;
use topcoat::{Result, context::Cx, runtime::procedure};
use uuid::Uuid;

use crate::{api, cookies, locale, next};

/// Where a visitor goes after signing in: the safe `next` path, else home.
fn arrival(next_path: &str) -> String {
    next::safe_next(Some(next_path)).unwrap_or_else(|| "/".to_owned())
}

/// Calls `POST path` and forwards the cookies it sets. `Err` is the API's message.
async fn post_with_cookies(
    cx: &Cx,
    path: &str,
    body: serde_json::Value,
) -> std::result::Result<(), String> {
    match api::send(cx, Method::POST, path, Some(body)).await {
        Ok(response) => {
            api::forward_cookies(cx, &response);
            Ok(())
        }
        Err(error) => Err(error.message),
    }
}

/// `POST /auth/login`. Answers the path to open next.
#[procedure]
pub(crate) async fn sign_in(
    cx: &Cx,
    handle: String,
    password: String,
    add: bool,
    next_path: String,
) -> Result<Result<String, String>> {
    let body = json!({
        "handle": handle.trim().trim_start_matches('@'),
        "password": password,
        "add": add,
    });
    Ok(post_with_cookies(cx, "/auth/login", body)
        .await
        .map(|()| arrival(&next_path)))
}

/// `POST /auth/signup`. Answers the path to open next.
#[procedure]
pub(crate) async fn sign_up(
    cx: &Cx,
    handle: String,
    display_name: String,
    password: String,
    invite: String,
    add: bool,
    next_path: String,
) -> Result<Result<String, String>> {
    let bare = handle.trim().trim_start_matches('@').to_owned();
    let name = display_name.trim();
    let invite = invite.trim();
    let language = locale::resolve(cx).await?;
    let body = json!({
        "handle": bare,
        "displayName": if name.is_empty() { bare.as_str() } else { name },
        "password": password,
        "acceptTerms": true,
        "add": add,
        "language": language,
        "invite": if invite.is_empty() { None } else { Some(invite) },
    });
    Ok(post_with_cookies(cx, "/auth/signup", body)
        .await
        .map(|()| arrival(&next_path)))
}

/// `POST /auth/logout`: signs out the active account, or every account on this browser. `true`
/// when done.
#[procedure]
pub(crate) async fn sign_out(cx: &Cx, all: bool) -> Result<Result<bool, String>> {
    Ok(post_with_cookies(cx, "/auth/logout", json!({ "all": all }))
        .await
        .map(|()| true))
}

/// `POST /accounts/switch`: makes another signed-in account the active one.
#[procedure]
pub(crate) async fn switch_account(cx: &Cx, id: String) -> Result<Result<bool, String>> {
    let Ok(id) = id.parse::<Uuid>() else {
        return Ok(Err("Unknown account".to_owned()));
    };
    Ok(
        post_with_cookies(cx, "/accounts/switch", json!({ "id": id }))
            .await
            .map(|()| true),
    )
}

/// The language choice: saved on the account when signed in (`PATCH /me`), and in the
/// `tuita_lang` cookie either way so the next render of this browser is in it.
#[procedure]
pub(crate) async fn choose_language(cx: &Cx, code: String) -> Result<Result<bool, String>> {
    let Some(code) = locale::known(&code) else {
        return Ok(Err("Unknown language".to_owned()));
    };
    if crate::session::me(cx).await?.is_some()
        && let Err(error) =
            api::send(cx, Method::PATCH, "/me", Some(json!({ "language": code }))).await
    {
        return Ok(Err(error.message));
    }
    cookies::set(cx, locale::COOKIE, code, Some(locale::COOKIE_MAX_AGE));
    Ok(Ok(true))
}
