//! The main navigation: one table behind the rail, the bottom bar and the keyboard shortcuts.

use topcoat::context::Cx;
use tuita_server::dto::BadgesDto;

use crate::{icons::Icon, l10n::t, paths};

/// Which unread counter a nav entry shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BadgeKind {
    Notifications,
    Messages,
}

impl BadgeKind {
    /// The `data-badge` value `stream.js` updates.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Notifications => "notifications",
            Self::Messages => "messages",
        }
    }

    /// The unread count of this kind.
    pub(crate) const fn count(self, badges: &BadgesDto) -> i64 {
        match self {
            Self::Notifications => badges.notifications,
            Self::Messages => badges.messages,
        }
    }
}

/// One destination of the main navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NavId {
    Home,
    Explore,
    Notifications,
    Messages,
    Bookmarks,
    Lists,
    Scheduled,
    Profile,
    Settings,
    Admin,
}

/// A nav entry: where it goes, how it looks and who sees it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct NavItem {
    pub(crate) id: NavId,
    pub(crate) icon: Icon,
    pub(crate) badge: Option<BadgeKind>,
    /// Shown only to admins and moderators.
    pub(crate) staff: bool,
}

const fn item(id: NavId, icon: Icon) -> NavItem {
    NavItem {
        id,
        icon,
        badge: None,
        staff: false,
    }
}

/// Every entry, in rail order.
pub(crate) const NAV: [NavItem; 10] = [
    item(NavId::Home, Icon::Home),
    item(NavId::Explore, Icon::Search),
    NavItem {
        badge: Some(BadgeKind::Notifications),
        ..item(NavId::Notifications, Icon::Bell)
    },
    NavItem {
        badge: Some(BadgeKind::Messages),
        ..item(NavId::Messages, Icon::Mail)
    },
    item(NavId::Bookmarks, Icon::Bookmark),
    item(NavId::Lists, Icon::ListBox),
    item(NavId::Scheduled, Icon::Calendar),
    item(NavId::Profile, Icon::User),
    item(NavId::Settings, Icon::Gear),
    NavItem {
        staff: true,
        ..item(NavId::Admin, Icon::Shield)
    },
];

/// The entries the bottom bar shows: home and explore, the compose button, then notifications and
/// messages.
pub(crate) fn bottom_items() -> [NavItem; 4] {
    [NAV[0], NAV[1], NAV[2], NAV[3]]
}

impl NavItem {
    /// Where the entry leads for the account `handle`.
    pub(crate) fn href(self, handle: &str) -> String {
        match self.id {
            NavId::Home => "/".to_owned(),
            NavId::Explore => "/explore".to_owned(),
            NavId::Notifications => "/notifications".to_owned(),
            NavId::Messages => "/messages".to_owned(),
            NavId::Bookmarks => "/bookmarks".to_owned(),
            NavId::Lists => "/lists".to_owned(),
            NavId::Scheduled => "/scheduled".to_owned(),
            NavId::Profile => paths::profile(handle),
            NavId::Settings => "/settings".to_owned(),
            NavId::Admin => "/admin".to_owned(),
        }
    }

    /// The entry's text in the request's language.
    pub(crate) fn label(self, cx: &Cx) -> String {
        match self.id {
            NavId::Home => t!(nav_home).resolve(cx).into_owned(),
            NavId::Explore => t!(nav_explore).resolve(cx).into_owned(),
            NavId::Notifications => t!(nav_notifications).resolve(cx).into_owned(),
            NavId::Messages => t!(nav_messages).resolve(cx).into_owned(),
            NavId::Bookmarks => t!(nav_bookmarks).resolve(cx).into_owned(),
            NavId::Lists => t!(nav_lists).resolve(cx).into_owned(),
            NavId::Scheduled => t!(nav_scheduled).resolve(cx).into_owned(),
            NavId::Profile => t!(nav_profile).resolve(cx).into_owned(),
            NavId::Settings => t!(nav_settings).resolve(cx).into_owned(),
            NavId::Admin => t!(nav_admin).resolve(cx).into_owned(),
        }
    }

    /// Whether `path` belongs to the entry: home is exact, the profile ignores case, the rest
    /// match their subtree.
    pub(crate) fn is_active(self, path: &str, handle: &str) -> bool {
        let href = self.href(handle);
        match self.id {
            NavId::Home => path == "/",
            NavId::Profile => path.eq_ignore_ascii_case(&href),
            _ => {
                path == href
                    || path
                        .strip_prefix(&href)
                        .is_some_and(|rest| rest.starts_with('/'))
            }
        }
    }
}

/// Routes that need the 960px column and so get no right rail.
const WIDE: [&str; 2] = ["/admin", "/messages"];

/// Whether `path` is one of the wide routes.
pub(crate) fn is_wide(path: &str) -> bool {
    WIDE.iter().any(|root| {
        path == *root
            || path
                .strip_prefix(root)
                .is_some_and(|rest| rest.starts_with('/'))
    })
}

#[cfg(test)]
mod test {
    use super::*;

    fn find(id: NavId) -> NavItem {
        *NAV.iter().find(|entry| entry.id == id).expect("in table")
    }

    #[test]
    fn home_is_exact_and_the_rest_match_their_subtree() {
        assert!(find(NavId::Home).is_active("/", "ada"));
        assert!(!find(NavId::Home).is_active("/explore", "ada"));
        assert!(find(NavId::Lists).is_active("/lists", "ada"));
        assert!(find(NavId::Lists).is_active("/lists/1/members", "ada"));
        assert!(!find(NavId::Lists).is_active("/listsx", "ada"));
        assert!(find(NavId::Messages).is_active("/messages/bo", "ada"));
    }

    #[test]
    fn the_profile_entry_is_the_viewers_own_profile_only() {
        let profile = find(NavId::Profile);
        assert_eq!(profile.href("ada"), "/@ada");
        assert!(profile.is_active("/@Ada", "ada"));
        assert!(!profile.is_active("/@ada/followers", "ada"));
        assert!(!profile.is_active("/@bo", "ada"));
    }

    #[test]
    fn admin_and_messages_are_wide() {
        assert!(is_wide("/admin"));
        assert!(is_wide("/admin/reports"));
        assert!(is_wide("/messages/ada"));
        assert!(!is_wide("/adminx"));
        assert!(!is_wide("/"));
    }

    #[test]
    fn only_the_admin_entry_is_staff_only() {
        let staff: Vec<_> = NAV.iter().filter(|e| e.staff).map(|e| e.id).collect();
        assert_eq!(staff, [NavId::Admin]);
    }

    #[test]
    fn the_unread_counters_come_from_the_badges() {
        let badges = BadgesDto {
            notifications: 3,
            messages: 7,
            reports: None,
        };
        assert_eq!(BadgeKind::Notifications.count(&badges), 3);
        assert_eq!(BadgeKind::Messages.count(&badges), 7);
    }
}
