//! The bar above the page on phones.

use topcoat::{
    Result,
    context::Cx,
    view::{View, class, component, view},
};

use crate::{l10n::t, session::Viewer, shell::parts::account_avatar, ui::brand::brand};

/// The id of the drawer this bar opens.
pub(crate) const DRAWER: &str = "drawer";

/// The account button that opens the [drawer](super::drawer) and the brand link. `demo` keeps the
/// bar visible on wide screens (the design gallery shows it in a frame). `drawer` is the id of the
/// drawer it opens when that is not the default.
#[component]
pub(crate) async fn top_bar(
    cx: &Cx,
    viewer: &Viewer,
    #[default] demo: bool,
    #[default]
    #[into]
    drawer: Option<&str>,
) -> Result<impl View> {
    let drawer = drawer.unwrap_or(DRAWER);
    let me = &viewer.me.user;
    Ok(view! {
        <div class=(class!("top", "demo" if demo)) data-c="top-bar">
            <button
                type="button"
                class="avatar"
                aria-label=(t!(nav_menu))
                command="show-modal"
                commandfor=(drawer)
            >
                account_avatar(
                    handle: &me.handle,
                    src: me.avatar_url.as_deref(),
                    shape: me.avatar_shape,
                    ink: me.ink,
                    size: 32
                )
            </button>
            <a href="/" class="brand" aria-label=(t!(app_name))>brand(size: 26)</a>
        </div>
    })
}
