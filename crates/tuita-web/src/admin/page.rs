//! One tab of the console, or the denied state.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::StatusCode,
    view::{View, component, view},
};

use crate::{
    admin::{
        common::tr,
        rules::{AdminTab, tabs_for},
        tabs::{admin_tabs, tab_label},
    },
    fragment,
    l10n::t,
    session,
    shell::{page_bar::page_bar, parts::nav_button, titled},
    ui::empty_state::empty_state,
};

/// The tab title: `Admin · <tab>` when the viewer may open the tab, else `Admin`.
pub(crate) async fn title(cx: &Cx, tab: AdminTab) -> Result<Metadata> {
    let admin = tr!(cx, admin_title);
    let allowed = match session::viewer(cx).await? {
        Some(viewer) => tabs_for(viewer.me.role).contains(&tab),
        None => false,
    };
    if allowed {
        titled(cx, &format!("{admin} · {}", tab_label(cx, tab))).await
    } else {
        titled(cx, &admin).await
    }
}

/// `tab` for staff that may open it; the denied state (status 403) for everyone else.
#[component]
pub(crate) async fn admin_page(cx: &Cx, tab: AdminTab) -> Result<impl View> {
    let viewer = session::member(cx).await?;
    let tabs = tabs_for(viewer.me.role);
    let allowed = tabs.contains(&tab);
    let instance = session::instance(cx).await?;
    let heading = tr!(cx, admin_title);
    let denied_title = tr!(cx, admin_denied_title);
    let denied_body = tr!(cx, admin_denied_body);
    let fragment = fragment::requested(cx) && tab == AdminTab::Audit;
    Ok(view! {
        if allowed {
            if !fragment {
                page_bar(
                    title: &heading,
                    sub: instance.domain.as_str(),
                    below: Some(view! { admin_tabs(tabs: &tabs, current: tab) }.into())
                )
            }
            if tab == AdminTab::Overview {
                crate::admin::overview::overview()
            } else if tab == AdminTab::Reports {
                crate::admin::reports::reports()
            } else if tab == AdminTab::Users {
                crate::admin::users::users()
            } else if tab == AdminTab::Instance {
                crate::admin::instance::instance()
            } else if tab == AdminTab::Federation {
                crate::admin::federation_tab::federation_tab()
            } else if tab == AdminTab::Announcements {
                crate::admin::announcements::announcements()
            } else if tab == AdminTab::Invites {
                crate::admin::invites::invites()
            } else if tab == AdminTab::Emotes {
                crate::admin::emotes::emotes()
            } else if tab == AdminTab::Filter {
                crate::admin::filter::filter()
            } else {
                crate::admin::audit::audit()
            }
        } else {
            (StatusCode::FORBIDDEN)
            page_bar(title: &heading)
            <div class="denied" data-c="admin-denied">
                empty_state(
                    context: "admin-denied",
                    title: &denied_title,
                    body: denied_body.as_str(),
                    action: Some(
                        view! { nav_button(href: "/", (t!(admin_denied_home))) }.into(),
                    )
                )
            </div>
        }
    })
}
