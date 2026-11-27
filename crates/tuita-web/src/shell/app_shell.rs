//! The frame around every page of a signed-in member.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, class, component, view},
};

use crate::{
    l10n::t,
    session::Viewer,
    shell::{
        announcement_banner::announcement_banner, bottom_bar::bottom_bar, drawer::drawer,
        nav::is_wide, rail_nav::rail_nav, right_rail::right_rail, top_bar::top_bar,
    },
};

/// The page in `main#main`, with the rail on wide screens, the top bar and bottom bar on phones,
/// the drawer, the announcements and the right rail.
///
/// The admin console and the chat split pane (`nav::is_wide`) need a wider column and have no
/// right rail. The compose, lightbox and share dialogs are not here: they render from
/// `globals::globals` for every page.
#[component]
pub(crate) async fn app_shell(
    cx: &Cx,
    viewer: &Viewer,
    path: &str,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let wide = is_wide(path);
    let rail = !wide;
    Ok(view! {
        <div class=(class!("shell", "wide" if wide, "rail" if rail)) data-c="app-shell">
            <aside class="side">rail_nav(viewer: viewer, path: path)</aside>
            <div class="col">
                announcement_banner()
                top_bar(viewer: viewer)
                <main id="main">(child)</main>
            </div>
            if rail {
                <aside class="right" aria-label=(t!(rail_label))>
                    right_rail(path: path)
                </aside>
            }
        </div>
        bottom_bar(viewer: viewer, path: path)
        drawer(viewer: viewer)
    })
}
