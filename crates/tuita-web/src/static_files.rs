//! Files with fixed URLs: the favicon, the app icons, the web manifest and `robots.txt`.
//!
//! They are not content-hashed assets, because browsers and crawlers ask for these exact paths.
//! They are cached for an hour and keep out of the `no-store` rule of the pages.

use topcoat::{
    Result,
    context::Cx,
    router::{Body, HeaderValue, header, response::Response, route},
};

/// `Cache-Control` of every fixed-URL file.
pub(crate) const CACHE_CONTROL: &str = "public, max-age=3600";

fn file(content_type: &'static str, bytes: &'static [u8]) -> Response {
    let mut response = Response::new(Body::from(bytes));
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(CACHE_CONTROL),
    );
    response
}

#[route(GET "/favicon.svg")]
async fn favicon(cx: &Cx) -> Result<Response> {
    let _ = cx;
    Ok(file(
        "image/svg+xml",
        include_bytes!("../assets/static/favicon.svg"),
    ))
}

#[route(GET "/icon-192.png")]
async fn icon_192(cx: &Cx) -> Result<Response> {
    let _ = cx;
    Ok(file(
        "image/png",
        include_bytes!("../assets/static/icon-192.png"),
    ))
}

#[route(GET "/icon-512.png")]
async fn icon_512(cx: &Cx) -> Result<Response> {
    let _ = cx;
    Ok(file(
        "image/png",
        include_bytes!("../assets/static/icon-512.png"),
    ))
}

#[route(GET "/icon-maskable-192.png")]
async fn icon_maskable_192(cx: &Cx) -> Result<Response> {
    let _ = cx;
    Ok(file(
        "image/png",
        include_bytes!("../assets/static/icon-maskable-192.png"),
    ))
}

#[route(GET "/icon-maskable-512.png")]
async fn icon_maskable_512(cx: &Cx) -> Result<Response> {
    let _ = cx;
    Ok(file(
        "image/png",
        include_bytes!("../assets/static/icon-maskable-512.png"),
    ))
}

#[route(GET "/manifest.webmanifest")]
async fn manifest(cx: &Cx) -> Result<Response> {
    let _ = cx;
    Ok(file(
        "application/manifest+json",
        include_bytes!("../assets/static/manifest.webmanifest"),
    ))
}

#[route(GET "/robots.txt")]
async fn robots(cx: &Cx) -> Result<Response> {
    let _ = cx;
    Ok(file(
        "text/plain; charset=utf-8",
        include_bytes!("../assets/static/robots.txt"),
    ))
}

/// The URLs above, so the edge layer leaves them their own cache headers.
pub(crate) const PATHS: [&str; 7] = [
    "/favicon.svg",
    "/icon-192.png",
    "/icon-512.png",
    "/icon-maskable-192.png",
    "/icon-maskable-512.png",
    "/manifest.webmanifest",
    "/robots.txt",
];
