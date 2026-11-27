//! The group of pages that need a signed-in member. It adds no URL segment.
//!
//! A guest on `/@{handle}` or `/@{handle}/{post_id}` gets a `200` page with link-preview tags and
//! an immediate redirect to `/login`, so crawlers still see the preview. A guest anywhere else
//! gets a `303` to `/login?next=<path>`.

mod admin;
mod bookmarks;
mod explore;
mod handle;
mod lists;
mod messages;
mod notifications;
mod scheduled;
mod settings;
mod tags;

use topcoat::{
    Result,
    context::{Cx, context},
    l10n::Locale,
    router::{
        Body, Method, Next, StatusCode,
        error::see_other,
        layer,
        request::{method, uri},
        response::{IntoResponse, Response},
    },
    view::{Unescaped, ViewExt, view},
};

use crate::{api::Web, l10n::t, next::login_url, paths::Handle, session};

/// True for `/@{handle}` and `/@{handle}/{post_id}`, the two URLs people share.
fn is_shared_url(path: &str) -> bool {
    let mut segments = path.strip_prefix('/').unwrap_or(path).split('/');
    let Some(first) = segments.next() else {
        return false;
    };
    if first.parse::<Handle>().is_err() {
        return false;
    }
    match (segments.next(), segments.next()) {
        (None, _) => true,
        (Some(second), None) => !second.is_empty() && !matches!(second, "followers" | "following"),
        _ => false,
    }
}

#[layer]
async fn member_guard(cx: &Cx, body: Body, next: Next<'_>) -> Result<Response> {
    if session::me(cx).await?.is_some() {
        return next.run(cx, body).await;
    }
    let (path, query) = {
        let uri = uri(cx);
        (uri.path().to_owned(), uri.query().map(str::to_owned))
    };
    let to = login_url(&path, query.as_deref());
    if matches!(method(cx), &Method::GET | &Method::HEAD) && is_shared_url(&path) {
        return guest_preview(cx, &path, &to).await;
    }
    Err(see_other(to).into())
}

/// A bare document: the preview tags of the profile or post, a refresh to the sign-in page and a
/// visible link to the same place.
async fn guest_preview(cx: &Cx, path: &str, to: &str) -> Result<Response> {
    let web = context::<Web>(cx);
    let head = tuita_server::routes::head::head_tags(&web.state, path).await;
    let lang = context::<Locale>(cx).id().to_string();
    let refresh = format!("0; url={to}");
    let page = view! {
        <!DOCTYPE html>
        <html lang=(lang)>
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <meta http-equiv="refresh" content=(refresh)>
                (Unescaped::new_unchecked(head))
            </head>
            <body><p><a href=(to)>(t!(auth_login_title))</a></p></body>
        </html>
    };
    let mut response = Box::pin(page.first()).await?.into_response(cx)?;
    *response.status_mut() = StatusCode::OK;
    Ok(response)
}

#[cfg(test)]
mod test {
    use super::is_shared_url;

    #[test]
    fn only_profile_and_post_urls_are_previewed() {
        for yes in ["/@ada", "/@ada@remote.example", "/@ada/0190"] {
            assert!(is_shared_url(yes), "{yes}");
        }
        for no in [
            "/@ada/followers",
            "/@ada/following",
            "/@ada/1/2",
            "/explore",
            "/ada",
            "/@",
            "/",
            "/@ada/",
        ] {
            assert!(!is_shared_url(no), "{no}");
        }
    }
}
