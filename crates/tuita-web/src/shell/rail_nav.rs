//! The navigation rail on wide screens.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, class, component, view},
};

use crate::{
    icons::Icon,
    l10n::t,
    session::{Viewer, is_staff},
    shell::{account_menu::account_menu, nav::NAV, parts::nav_badge},
    ui::{
        brand::brand,
        button::{ButtonSize, button},
        icon::icon,
    },
};

/// The brand, the main links with their unread counters, the compose button and the account menu.
///
/// The compose button is `[data-compose="new"]`: `composer.js` opens the compose dialog on it.
#[component]
pub(crate) async fn rail_nav(cx: &Cx, viewer: &Viewer, path: &str) -> Result<impl View> {
    let handle = viewer.me.user.handle.as_str();
    let staff = is_staff(viewer);
    let items: Vec<_> = NAV
        .iter()
        .filter(|item| !item.staff || staff)
        .map(|item| {
            (
                item.href(handle),
                item.label(cx),
                item.is_active(path, handle),
                item.icon,
                item.badge.map(|kind| (kind, kind.count(&viewer.badges))),
            )
        })
        .collect();
    Ok(view! {
        <div class="rail" data-c="rail-nav">
            <a class="brand" href="/" aria-label=(t!(app_name))>brand(size: 32)</a>
            <nav aria-label=(t!(nav_main))>
                for (href, label, current, glyph, badge) in items {
                    <a
                        href=(href)
                        class=(class!("current" if current))
                        aria-current=(current.then_some("page"))
                    >
                        icon(name: glyph, size: 24)
                        <span>(label)</span>
                        if let Some((kind, count)) = badge {
                            nav_badge(kind: kind, count: count)
                        }
                    </a>
                }
            </nav>
            button(
                size: ButtonSize::Lg,
                attrs: attributes! { data-compose="new" },
                icon(name: Icon::Plus, size: 20)
                (t!(nav_compose))
            )
            <div class="chip">account_menu(viewer: viewer)</div>
        </div>
    })
}
