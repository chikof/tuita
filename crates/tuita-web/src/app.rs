//! The root of the module router: `/`, the root layout, the pathless edge layer.
//!
//! Every module below maps to a URL segment. Modules that
//! start with `_` are groups: they share a layer and layout but add no URL segment.

pub(crate) mod _member;
mod design;
mod developers;
mod login;
mod oauth;
mod privacy;
mod signup;
mod terms;

use topcoat::{
    Error, Result,
    asset::asset_config,
    context::{Cx, context},
    l10n::Locale,
    metadata::{self, Metadata},
    router::{
        Body, HeaderValue, LayerFn, LayerFuture, Next, RouterBuilder, Slot, StatusCode,
        error::{ForbiddenError, NotFoundError, see_other},
        header, layout, module_router, page,
        request::uri,
        response::{IntoResponse, response_headers},
    },
    view::{BoxView, Unescaped, View, ViewExt, component, error_boundary, view},
};

use crate::{
    api::{ApiError, Web},
    assets::{BOOT_JS, GEIST_FONT, GEIST_MONO_FONT, GEIST_PIXEL_FONT, SCRIPTS, STYLESHEETS},
    cookies, fragment,
    l10n::{client_strings_json, flash, t},
    locale, next, session,
    shell::{self, app_shell::app_shell, globals::globals},
    static_files,
    ui::not_found::not_found,
};

/// Cookie that carries a flash message key across a redirect.
pub(crate) const FLASH_COOKIE: &str = "tuita_flash";

/// The module router rooted here.
///
/// A root `not_found!()` conflicts with the `/@{handle}` param, so [`edge`], a pathless layer,
/// brands unmatched URLs instead.
pub(crate) fn routes() -> RouterBuilder {
    module_router!().layer(LayerFn::new(None::<&str>, edge))
}

/// Paths that keep their own cache headers and need no locale: hashed assets and the fixed files.
fn is_static(path: &str) -> bool {
    path.starts_with("/_topcoat/assets/") || static_files::PATHS.contains(&path)
}

/// Pathless layer around everything:
///
/// - sets the request locale (see `locale::resolve`),
/// - marks every dynamic response `Cache-Control: no-store` and `Vary: Cookie`,
/// - turns the router's own "no route" error into the branded 404 page.
fn edge<'a>(cx: &'a Cx, body: Body, next: Next<'a>) -> LayerFuture<'a> {
    Box::pin(async move {
        if is_static(uri(cx).path()) {
            return next.run(cx, body).await;
        }
        let code = locale::resolve(cx).await?;
        let cx = &cx.with(locale::locale_of(code));
        let out = response_headers(cx);
        out.append(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        out.append(header::VARY, HeaderValue::from_static("Cookie"));
        match next.run(cx, body).await {
            Err(error) if error.downcast_ref::<NotFoundError>().is_some() => {
                let page = view! { root_layout(slot: Slot::new(view! { not_found() })) };
                let mut response = Box::pin(page.first()).await?.into_response(cx)?;
                *response.status_mut() = StatusCode::NOT_FOUND;
                Ok(response)
            }
            other => other,
        }
    })
}

/// A page body for a route whose owner has not written it yet.
#[component]
pub(crate) async fn placeholder(cx: &Cx) -> Result<impl View> {
    Ok(view! { <h1>(t!(not_found_title))</h1> })
}

/// The `@font-face` rules, pointing at the hashed font assets.
fn font_faces(cx: &Cx) -> String {
    let config = asset_config(cx);
    let face = |family: &str, asset, weight: &str| {
        format!(
            "@font-face{{font-family:'{family}';src:url('{}') format('woff2');font-weight:{weight};font-style:normal;font-display:swap}}",
            config.resolve(cx, asset)
        )
    };
    [
        face("Geist", GEIST_FONT, "100 900"),
        face("Geist Mono", GEIST_MONO_FONT, "100 900"),
        face("Geist Pixel", GEIST_PIXEL_FONT, "400"),
    ]
    .concat()
}

/// Turns an error raised by a page into a branded page, or hands it on.
///
/// - an API `401` becomes a redirect to `/login`,
/// - a `404` (API or router) becomes the branded not-found page,
/// - other API failures and `403` become a short error page with their status.
pub(crate) fn error_page(cx: &Cx, error: Error) -> Result<BoxView<'_>> {
    if let Some(api) = error.downcast_ref::<ApiError>() {
        return match api.status {
            StatusCode::UNAUTHORIZED => {
                let uri = uri(cx);
                Err(see_other(next::login_url(uri.path(), uri.query())).into())
            }
            StatusCode::NOT_FOUND => Ok(not_found_view(cx)),
            status => Ok(status_view(cx, status, api.message.clone())),
        };
    }
    if error.downcast_ref::<NotFoundError>().is_some() {
        return Ok(not_found_view(cx));
    }
    if error.downcast_ref::<ForbiddenError>().is_some() {
        return Ok(status_view(
            cx,
            StatusCode::FORBIDDEN,
            t!(error_title).resolve(cx).into_owned(),
        ));
    }
    Err(error)
}

fn not_found_view(cx: &Cx) -> BoxView<'_> {
    view! {
        (StatusCode::NOT_FOUND)
        not_found()
    }
    .boxed()
}

fn status_view(cx: &Cx, status: StatusCode, message: String) -> BoxView<'_> {
    view! {
        (status)
        <main class="error-page" data-c="error-page">
            <h1>(t!(error_title))</h1>
            <p>(message)</p>
        </main>
    }
    .boxed()
}

/// The `<head>` of every document.
#[component]
async fn document_head(
    cx: &Cx,
    #[default] title: Option<&metadata::Text>,
    head: &str,
    fonts: &str,
    strings: &str,
) -> Result<impl View> {
    Ok(view! {
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            if let Some(title) = title {
                <title>(title)</title>
            }
            (Unescaped::new_unchecked(head.to_owned()))
            <link rel="icon" href="/favicon.svg" type="image/svg+xml">
            <link rel="apple-touch-icon" href="/icon-192.png">
            <link rel="manifest" href="/manifest.webmanifest">
            <meta name="theme-color" content="#0b0b0d">
            <link
                rel="preload"
                href=(GEIST_FONT)
                as="font"
                type="font/woff2"
                crossorigin=""
            >
            <style>(Unescaped::new_unchecked(fonts.to_owned()))</style>
            <script src=(BOOT_JS)></script>
            for sheet in STYLESHEETS {
                <link rel="stylesheet" href=(sheet)>
            }
            topcoat::runtime::script()
            for (stem, script) in SCRIPTS {
                <script type="module" data-js=(stem) src=(script)></script>
            }
            <script type="application/json" id="tuita-l10n">
                (Unescaped::new_unchecked(strings.to_owned()))
            </script>
        </head>
    })
}

#[layout]
async fn root_layout(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    let fragment = fragment::requested(cx);
    let web = context::<Web>(cx);
    let path = uri(cx).path().to_owned();
    let member = session::viewer(cx)
        .await?
        .as_ref()
        .filter(|_| shell::wants_shell(&path));
    let lang = context::<Locale>(cx).id().to_string();
    let head = if fragment {
        String::new()
    } else {
        tuita_server::routes::head::head_tags(&web.state, &path).await
    };
    let strings = client_strings_json(context::<Locale>(cx));
    let flash_text = cookies::get(cx, FLASH_COOKIE).and_then(|key| flash(cx, key));
    if cookies::get(cx, FLASH_COOKIE).is_some() {
        cookies::expire(cx, FLASH_COOKIE);
    }
    let fonts = font_faces(cx);
    let page = metadata::metadata(cx).await?;
    let content = view! {
        error_boundary(fallback: |error| error_page(cx, error), (slot))
    };
    Ok(view! {
        if fragment {
            (content)
        } else {
            <!DOCTYPE html>
            <html lang=(lang)>
                document_head(
                    title: page.title(),
                    head: &head,
                    fonts: &fonts,
                    strings: &strings
                )
                <body>
                    if let Some(viewer) = member {
                        app_shell(viewer: viewer, path: &path, (content))
                    } else {
                        (content)
                    }
                    if let Some(text) = flash_text {
                        <output hidden="" data-toast="" data-kind="info">(text)</output>
                    }
                    globals()
                </body>
            </html>
        }
    })
}

/// The tab title of `/`: the instance and its tagline for a guest, "Home" for a member.
async fn home_title(cx: &Cx) -> Result<Metadata> {
    if session::me(cx).await?.is_some() {
        let home = t!(nav_home).resolve(cx).into_owned();
        shell::titled(cx, &home).await
    } else {
        crate::auth::landing::title(cx).await
    }
}

#[page(with(Metadata::from_fn(async |cx| home_title(cx).await)))]
async fn home_page(cx: &Cx) -> Result<impl View> {
    let member = session::me(cx).await?.is_some();
    Ok(view! {
        if member {
            crate::feed::home::home()
        } else {
            crate::auth::landing::landing()
        }
    })
}
