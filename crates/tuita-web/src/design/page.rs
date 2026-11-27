//! The gallery page: toolbar, component index and every section.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use super::{
    actions::actions, admin::admin, dither::dither, domain::domain, encryption::encryption,
    federation::federation, feedback::feedback, forms::forms, identity::identity, media::media,
    oauth::oauth, overlays::overlays, toolbar::toolbar,
};

/// The anchors of the component index, in the gallery order. Each is a
/// `section#<id>` of one of the groups.
pub(crate) const INDEX: [&str; 108] = [
    "button",
    "icon-button",
    "card",
    "bento",
    "tag",
    "badge",
    "divider",
    "text-field",
    "text-area",
    "select",
    "switch",
    "checkbox",
    "radio",
    "slider",
    "segmented",
    "tabs",
    "menu",
    "popover",
    "dialog",
    "sheet",
    "toast",
    "tooltip",
    "avatar",
    "avatar-stack",
    "skeleton",
    "loader",
    "progress-bar",
    "char-counter",
    "dither-field",
    "dither-fill",
    "dither-text",
    "dither-number",
    "empty-state",
    "not-found",
    "brand",
    "icon",
    "utilities",
    "user-name",
    "rich-text",
    "heart",
    "content-warning",
    "quote-card",
    "post-actions",
    "post-card",
    "post-thread",
    "user-row",
    "follow-button",
    "profile-header",
    "notification-row",
    "composer",
    "infinite-list",
    "new-posts-pill",
    "page-bar",
    "rail-nav",
    "top-bar",
    "bottom-bar",
    "drawer",
    "account-menu",
    "shortcuts-dialog",
    "confirm-dialog",
    "language-picker",
    "landing",
    "app-shell",
    "dropzone",
    "cropper",
    "media-grid",
    "lightbox",
    "video-player",
    "sensitive-cover",
    "link-card",
    "poll",
    "poll-editor",
    "media-tray",
    "emoji-picker",
    "schedule-picker",
    "edit-dialogs",
    "conversation-row",
    "chat-thread",
    "e2ee-prompt",
    "e2ee-notice",
    "e2ee-thread",
    "e2ee-row",
    "e2ee-settings",
    "list-row",
    "scheduled-card",
    "stat-card",
    "bar-chart",
    "sparkline",
    "heatmap",
    "admin-bento",
    "announcement-banner",
    "admin-tabs",
    "report-card",
    "admin-user-row",
    "post-preview",
    "token-secret",
    "queue-summary",
    "actor-key",
    "domain-block-row",
    "domain-chip",
    "visibility-badge",
    "remote-post",
    "remote-user",
    "remote-profile",
    "remote-notifications",
    "remote-lookup",
    "user-gate",
    "consent-card",
];

#[component]
pub(crate) async fn gallery(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        <div class="page" data-c="design-page">
            <header class="top">toolbar()</header>
            <main>
                <h1>"Design system"</h1>
                <nav aria-label="Components" class="index">
                    for id in INDEX {
                        <a href=(format!("#{id}"))>(id)</a>
                    }
                </nav>
                actions()
                forms()
                overlays()
                feedback()
                identity()
                dither()
                domain()
                media()
                encryption()
                admin()
                federation()
                oauth()
            </main>
        </div>
    })
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::INDEX;
    use crate::design::sample;

    #[test]
    fn every_anchor_of_the_index_is_listed_once() {
        let unique: HashSet<_> = INDEX.iter().collect();
        assert_eq!(unique.len(), INDEX.len(), "a duplicate anchor");
        assert!(INDEX.len() >= 36);
    }

    #[test]
    fn fixtures_have_stable_ids() {
        assert_eq!(sample::plain().id, sample::plain().id);
        assert_ne!(sample::plain().id, sample::liked().id);
        assert_eq!(sample::stats().days.len(), 14);
        assert_eq!(sample::stats().heatmap.len(), 7);
    }
}
