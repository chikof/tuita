//! The tab bar at the bottom of the screen on phones.

use topcoat::{
    Result,
    context::Cx,
    view::{View, class, component, view},
};

use crate::{
    icons::Icon,
    l10n::t,
    session::Viewer,
    shell::{
        nav::{NavItem, bottom_items},
        parts::nav_badge,
    },
    ui::icon::icon,
};

/// Home, explore, a compose button, notifications and messages.
///
/// `shell.js` hides the bar while the page scrolls down (`data-island="bottom-bar"`, class
/// `hidden`). `demo` keeps it visible on wide screens.
#[component]
pub(crate) async fn bottom_bar(
    cx: &Cx,
    viewer: &Viewer,
    path: &str,
    #[default] demo: bool,
) -> Result<impl View> {
    let handle = viewer.me.user.handle.as_str();
    let items = bottom_items();
    let link = |item: NavItem| {
        let current = item.is_active(path, handle);
        (
            item.href(handle),
            item.label(cx),
            current,
            item.icon,
            item.badge.map(|kind| (kind, kind.count(&viewer.badges))),
        )
    };
    let [first, second, third, fourth] = items.map(link);
    Ok(view! {
        <nav
            class=(class!("bottom", "demo" if demo))
            aria-label=(t!(nav_main))
            data-island="bottom-bar"
            data-c="bottom-bar"
        >
            for (href, label, current, glyph, badge) in [first, second] {
                <a
                    href=(href)
                    class=(class!("current" if current))
                    aria-current=(current.then_some("page"))
                    aria-label=(label)
                >
                    icon(name: glyph, size: 24)
                    if let Some((kind, count)) = badge {
                        <span class="badge">nav_badge(kind: kind, count: count)</span>
                    }
                </a>
            }
            <button
                type="button"
                class="compose"
                aria-label=(t!(nav_compose))
                data-compose="new"
            >
                icon(name: Icon::Plus, size: 24)
            </button>
            for (href, label, current, glyph, badge) in [third, fourth] {
                <a
                    href=(href)
                    class=(class!("current" if current))
                    aria-current=(current.then_some("page"))
                    aria-label=(label)
                >
                    icon(name: glyph, size: 24)
                    if let Some((kind, count)) = badge {
                        <span class="badge">nav_badge(kind: kind, count: count)</span>
                    }
                </a>
            }
        </nav>
    })
}
