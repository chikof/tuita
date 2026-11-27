//! `tuita-web`: the server-rendered web client of tuita, built on Topcoat.
//!
//! [`router`] returns the service that answers everything the backend does not: pages, fragment
//! routes, the Topcoat runtime and the fixed-URL files. Pages read and change data by calling the
//! JSON API in-process (see `api`), so authentication, CSRF, rate limits and every business rule
//! stay in `tuita-server`.

// The async view of the root layout nests deeper than the default query depth.
#![recursion_limit = "256"]

mod api;
mod app;
mod assets;
mod cookies;
mod fmt;
mod fragment;
mod icons;
mod l10n;
mod locale;
mod next;
mod paths;
mod session;
mod static_files;

mod admin;
mod auth;
mod compose;
mod design;
mod docs;
mod explore;
mod feed;
mod lists;
mod messages;
mod notifications;
mod oauth;
mod post;
mod profile;
mod settings;
mod shell;
#[cfg(test)]
pub(crate) mod testing;
mod ui;

use std::path::Path;

use axum::Router;
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    metadata::Metadata,
    router::{OriginPolicy, RouterBuilderDiscoverExt, tower::TowerService},
    runtime::RuntimeLayer,
};
use tuita_server::AppState;

use crate::l10n::t;

/// The hashed assets (stylesheets, scripts, fonts) the pages link to, as `topcoat asset bundle`
/// wrote them.
pub struct Assets(AssetBundle);

impl Assets {
    /// The bundle next to the running binary: the `assets` directory beside the executable.
    ///
    /// # Errors
    ///
    /// Fails when the directory or its manifest is missing or unreadable.
    pub fn next_to_binary() -> std::io::Result<Self> {
        AssetBundle::load().map(Self)
    }

    /// The bundle in `dir`, a directory written by `topcoat asset bundle`.
    ///
    /// # Errors
    ///
    /// Fails when `dir` or its manifest is missing or unreadable.
    pub fn from_dir(dir: &Path) -> std::io::Result<Self> {
        AssetBundle::load_dir(dir).map(Self)
    }
}

/// The web client as a service for `tuita_server::router(state, web)`.
///
/// It handles every request the backend does not match; `tuita-server` puts its gate (backend
/// 404s, ActivityPub negotiation) and its security headers around it.
pub fn router(state: AppState, assets: Assets) -> Router {
    let backend = tuita_server::backend(&state);
    let origin = origin_of(&state.config().public_url);
    let built = app::routes()
        .discover()
        .assets(assets.0)
        .with(api::Web { state, backend })
        .with(Metadata::new().title(t!(app_name)))
        .with(OriginPolicy::trust_origins([origin]))
        .layer(RuntimeLayer::new())
        .build();
    Router::new().fallback_service(TowerService::new(built))
}

/// `scheme://host[:port]` of the public URL, which is what a browser sends as `Origin`.
fn origin_of(public_url: &str) -> String {
    let after_scheme = public_url.find("://").map_or(0, |at| at + 3);
    let end = public_url[after_scheme..]
        .find('/')
        .map_or(public_url.len(), |at| after_scheme + at);
    public_url[..end].to_owned()
}

#[cfg(test)]
mod test {
    use super::origin_of;

    #[test]
    fn origin_drops_the_path() {
        assert_eq!(
            origin_of("https://social.example"),
            "https://social.example"
        );
        assert_eq!(
            origin_of("https://social.example/"),
            "https://social.example"
        );
        assert_eq!(
            origin_of("http://localhost:8080/x/y"),
            "http://localhost:8080"
        );
    }
}
