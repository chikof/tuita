//! Rendering helpers for unit tests: turn a component or a `view!` into an HTML string without a
//! router, a backend or a database.
//!
//! ```ignore
//! let html = render("en", |cx| view! { button(variant: ButtonVariant::Solid, "Save") }.boxed());
//! assert!(html.contains(r#"class="btn solid md wash""#));
//! ```
//!
//! The context holds the locale and, when a bundle exists, the asset catalog (`TUITA_TEST_ASSETS`,
//! else `<workspace>/target/debug/assets`, written by `topcoat asset bundle --package tuita`).
//! Components that render no `asset!` need no bundle. Anything that needs the request, a session
//! or the backend belongs in `tests/` instead.

use std::path::PathBuf;

use topcoat::{
    asset::{AssetBundle, AssetConfig},
    context::Cx,
    view::{BoxView, ViewExt},
};

use crate::locale::locale_of;

/// The bundle directory the integration tests use.
fn assets_dir() -> PathBuf {
    std::env::var_os("TUITA_TEST_ASSETS").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/assets"),
        PathBuf::from,
    )
}

/// A context for rendering in `locale` (a code such as `"en"`; English for an unknown one).
#[must_use]
pub(crate) fn context(locale: &str) -> Cx {
    let cx = Cx::new().with(locale_of(locale));
    match AssetBundle::load_dir(assets_dir()) {
        Ok(bundle) => cx.with(AssetConfig::serve(bundle)),
        Err(_) => cx,
    }
}

/// Renders the view `build` makes from a context in `locale` and returns its HTML.
///
/// `build` receives the context and returns a boxed `view!` (`.boxed()`), which borrows it.
///
/// # Panics
///
/// Panics when the view fails to render or is live (a test fails loudly instead).
#[must_use]
pub(crate) fn render(locale: &str, build: impl for<'a> FnOnce(&'a Cx) -> BoxView<'a>) -> String {
    let cx = context(locale);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("tokio runtime");
    runtime.block_on(async {
        let handle = build(&cx).single().await.expect("view renders");
        handle.render(&cx)
    })
}

/// The values of attribute `name` on every tag of `html` that has it, in document order.
///
/// A deliberately small scanner for assertions (`class`, `aria-label`, `role`, ...), not a parser.
#[must_use]
pub(crate) fn attr_values(html: &str, name: &str) -> Vec<String> {
    let needle = format!(" {name}=\"");
    let mut found = Vec::new();
    let mut rest = html;
    while let Some(at) = rest.find(&needle) {
        let after = &rest[at + needle.len()..];
        let end = after.find('"').unwrap_or(after.len());
        found.push(after[..end].to_owned());
        rest = &after[end..];
    }
    found
}
