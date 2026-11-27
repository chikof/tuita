//! The strip of tabs under the page bar.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::{
    admin::rules::AdminTab,
    l10n::t,
    ui::mask::{Axis, ramp_mask},
};

/// The words of a tab.
pub(crate) fn tab_label(cx: &Cx, tab: AdminTab) -> String {
    match tab {
        AdminTab::Overview => t!(admin_tab_overview),
        AdminTab::Reports => t!(admin_tab_reports),
        AdminTab::Users => t!(admin_tab_users),
        AdminTab::Instance => t!(admin_tab_instance),
        AdminTab::Federation => t!(admin_tab_federation),
        AdminTab::Announcements => t!(admin_tab_announcements),
        AdminTab::Invites => t!(admin_tab_invites),
        AdminTab::Emotes => t!(admin_emote_tab),
        AdminTab::Filter => t!(admin_tab_filter),
        AdminTab::Audit => t!(admin_tab_audit),
    }
    .resolve(cx)
    .into_owned()
}

/// The links to `tabs`, the `current` one marked `aria-current="page"` with a dithered underline.
#[component]
pub(crate) async fn admin_tabs(cx: &Cx, tabs: &[AdminTab], current: AdminTab) -> Result<impl View> {
    let underline = ramp_mask(0.3, 1.1, 4, Axis::Y);
    let style = format!("mask-image:{underline};-webkit-mask-image:{underline}");
    let items: Vec<_> = tabs
        .iter()
        .map(|tab| (tab_label(cx, *tab), tab.path(), *tab == current))
        .collect();
    Ok(view! {
        <nav aria-label=(t!(admin_sections)) data-c="admin-tabs">
            <div class="strip">
                for (label, href, is_current) in items {
                    <a href=(href) aria-current=(is_current.then_some("page"))>
                        (label)
                        if is_current {
                            <span class="indicator" aria-hidden="true">
                                <i style=(&style)></i>
                            </span>
                        }
                    </a>
                }
            </div>
        </nav>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn the_current_tab_is_marked_and_the_others_are_links() {
        let html = render("en", |cx| {
            view! {
                admin_tabs(
                    tabs: &[AdminTab::Overview, AdminTab::Reports, AdminTab::Users],
                    current: AdminTab::Reports
                )
            }
            .boxed()
        });
        assert!(
            html.contains(r#"<nav aria-label="Admin sections""#),
            "{html}"
        );
        assert!(html.contains(r#"href="/admin""#), "{html}");
        assert!(html.contains(r#"href="/admin/users""#), "{html}");
        assert_eq!(html.matches(r#"aria-current="page""#).count(), 1, "{html}");
        let current = html.split(r#"aria-current="page""#).nth(1).unwrap_or("");
        assert!(current.starts_with(">Reports"), "{html}");
        assert_eq!(html.matches("indicator").count(), 1, "{html}");
    }
}
