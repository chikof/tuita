//! Who may do what in the admin console.

use time::OffsetDateTime;
use tuita_server::dto::{
    AdminAnnouncementDto, AdminReportDto, AdminUserDto, InviteDto, ReportAction, ReportStatus, Role,
};
use uuid::Uuid;

/// A tab of the console, in display order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AdminTab {
    Overview,
    Reports,
    Users,
    Instance,
    Federation,
    Announcements,
    Invites,
    Emotes,
    Filter,
    Audit,
}

impl AdminTab {
    /// Every tab, in display order.
    pub(crate) const ALL: [Self; 10] = [
        Self::Overview,
        Self::Reports,
        Self::Users,
        Self::Instance,
        Self::Federation,
        Self::Announcements,
        Self::Invites,
        Self::Emotes,
        Self::Filter,
        Self::Audit,
    ];

    /// Tabs whose routes only an admin may call. Moderators keep the rest.
    const ADMIN_ONLY: [Self; 5] = [
        Self::Instance,
        Self::Federation,
        Self::Announcements,
        Self::Invites,
        Self::Audit,
    ];

    /// The URL segment of the tab.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Reports => "reports",
            Self::Users => "users",
            Self::Instance => "instance",
            Self::Federation => "federation",
            Self::Announcements => "announcements",
            Self::Invites => "invites",
            Self::Emotes => "emotes",
            Self::Filter => "filter",
            Self::Audit => "audit",
        }
    }

    /// The tab a URL segment names.
    pub(crate) fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|tab| tab.name() == name)
    }

    /// Where the tab lives: the overview is `/admin`.
    pub(crate) fn path(self) -> String {
        match self {
            Self::Overview => "/admin".to_owned(),
            other => format!("/admin/{}", other.name()),
        }
    }
}

impl std::fmt::Display for AdminTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl std::str::FromStr for AdminTab {
    type Err = UnknownTab;

    fn from_str(name: &str) -> Result<Self, UnknownTab> {
        Self::parse(name).ok_or(UnknownTab)
    }
}

/// The URL segment names no tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct UnknownTab;

/// The tabs a viewer with `role` may open, in display order. A plain user gets none.
pub(crate) fn tabs_for(role: Role) -> Vec<AdminTab> {
    match role {
        Role::Admin => AdminTab::ALL.to_vec(),
        Role::Moderator => AdminTab::ALL
            .into_iter()
            .filter(|tab| !AdminTab::ADMIN_ONLY.contains(tab))
            .collect(),
        Role::User => Vec::new(),
    }
}

/// One entry of the menu of an account.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UserAction {
    Verify,
    Unverify,
    Org,
    Unorg,
    Role,
    Suspend,
    Unsuspend,
    Purge,
    Rename,
    ResetName,
    ClearBio,
    Delete,
}

/// The actions the menu for `target` offers the viewer. It mirrors the server's gates so people
/// never see a button that can only answer 403: roles, deletes and purges are admin only, nobody
/// suspends or deletes their own account here, only an admin touches an admin, and remote
/// accounts keep their handle, name and role on their own instance.
pub(crate) fn user_actions(
    viewer_id: Uuid,
    viewer_role: Role,
    target: &AdminUserDto,
) -> Vec<UserAction> {
    if target.deleted || viewer_role == Role::User {
        return Vec::new();
    }
    let admin = viewer_role == Role::Admin;
    let local = target.user.domain.is_none();
    let this_is_me = viewer_id == target.user.id;
    let target_admin = target.role == Role::Admin;
    let mut actions = vec![if target.user.verified {
        UserAction::Unverify
    } else {
        UserAction::Verify
    }];
    if local {
        actions.push(if target.user.org {
            UserAction::Unorg
        } else {
            UserAction::Org
        });
    }
    if admin && local && !this_is_me {
        actions.push(UserAction::Role);
    }
    if !this_is_me && (admin || !target_admin) {
        actions.push(if target.user.suspended {
            UserAction::Unsuspend
        } else {
            UserAction::Suspend
        });
    }
    if admin && !this_is_me {
        actions.push(UserAction::Purge);
    }
    if local {
        actions.extend([
            UserAction::Rename,
            UserAction::ResetName,
            UserAction::ClearBio,
        ]);
    }
    if admin && local && !this_is_me && !target_admin {
        actions.push(UserAction::Delete);
    }
    actions
}

/// Buttons an open report offers: dismiss, delete the post, and suspend (with the post when there
/// is one).
pub(crate) fn report_actions(report: &AdminReportDto) -> Vec<ReportAction> {
    if report.status != ReportStatus::Open {
        return Vec::new();
    }
    let mut actions = vec![ReportAction::Dismiss];
    if report.post.is_some() {
        actions.push(ReportAction::DeletePost);
    }
    if !report.target_suspended {
        actions.push(if report.post.is_some() {
            ReportAction::DeleteAndSuspend
        } else {
            ReportAction::SuspendUser
        });
    }
    actions
}

/// Whether an invite code can still be used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InviteState {
    Active,
    Revoked,
    Used,
    Expired,
}

pub(crate) fn invite_state(invite: &InviteDto, now: OffsetDateTime) -> InviteState {
    if invite.revoked {
        InviteState::Revoked
    } else if invite.uses >= invite.max_uses {
        InviteState::Used
    } else if invite.expires_at.is_some_and(|at| at <= now) {
        InviteState::Expired
    } else {
        InviteState::Active
    }
}

/// Whether an announcement still shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnnouncementState {
    Live,
    Ended,
    Expired,
}

pub(crate) fn announcement_state(
    item: &AdminAnnouncementDto,
    now: OffsetDateTime,
) -> AnnouncementState {
    if !item.active {
        AnnouncementState::Ended
    } else if item.expires_at.is_some_and(|at| at <= now) {
        AnnouncementState::Expired
    } else {
        AnnouncementState::Live
    }
}

/// One entry per line, trimmed, blank lines dropped. The inverse of `rules.join("\n")`.
pub(crate) fn lines_of(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod test {
    use time::macros::datetime;
    use tuita_server::dto::{AvatarShape, Ink, Relation, UserCounts, UserDto};

    use super::*;

    fn user(domain: Option<&str>, verified: bool, org: bool, suspended: bool) -> UserDto {
        UserDto {
            id: Uuid::from_u128(1),
            handle: "target".to_owned(),
            domain: domain.map(str::to_owned),
            display_name: "Target".to_owned(),
            bio: String::new(),
            location: String::new(),
            website: String::new(),
            avatar_url: None,
            banner_url: None,
            avatar_shape: AvatarShape::Circle,
            ink: Ink::Bone,
            verified,
            staff: None,
            org,
            bot: false,
            bot_owner: None,
            affiliation: None,
            pinned_post_id: None,
            created_at: datetime!(2026-01-01 0:00 UTC),
            counts: UserCounts {
                posts: 0,
                followers: 0,
                following: 0,
            },
            relation: Relation::default(),
            suspended,
            url: None,
            locked: false,
            emojis: Vec::new(),
        }
    }

    fn target(user: UserDto, role: Role, deleted: bool) -> AdminUserDto {
        AdminUserDto {
            user,
            role,
            deleted,
            open_reports: 0,
            last_active_at: None,
        }
    }

    fn plain() -> AdminUserDto {
        target(user(None, false, false, false), Role::User, false)
    }

    const VIEWER: Uuid = Uuid::from_u128(2);

    #[test]
    fn admins_get_every_tab_and_moderators_the_moderation_tabs() {
        let names = |tabs: Vec<AdminTab>| tabs.into_iter().map(AdminTab::name).collect::<Vec<_>>();
        assert_eq!(
            names(tabs_for(Role::Admin)),
            [
                "overview",
                "reports",
                "users",
                "instance",
                "federation",
                "announcements",
                "invites",
                "emotes",
                "filter",
                "audit"
            ]
        );
        assert_eq!(
            names(tabs_for(Role::Moderator)),
            ["overview", "reports", "users", "emotes", "filter"]
        );
        assert!(tabs_for(Role::User).is_empty());
    }

    #[test]
    fn tab_names_and_paths() {
        assert_eq!(AdminTab::parse("audit"), Some(AdminTab::Audit));
        assert_eq!(AdminTab::parse("federation"), Some(AdminTab::Federation));
        assert_eq!(AdminTab::parse("peers"), None);
        assert_eq!(AdminTab::Overview.path(), "/admin");
        assert_eq!(AdminTab::Federation.path(), "/admin/federation");
    }

    #[test]
    fn a_moderator_verifies_suspends_renames_and_marks_organizations_but_deletes_nothing() {
        use UserAction as A;
        assert_eq!(
            user_actions(VIEWER, Role::Moderator, &plain()),
            [
                A::Verify,
                A::Org,
                A::Suspend,
                A::Rename,
                A::ResetName,
                A::ClearBio
            ]
        );
    }

    #[test]
    fn an_admin_gets_everything_for_an_ordinary_local_account() {
        use UserAction as A;
        let everything = target(user(None, true, true, true), Role::User, false);
        assert_eq!(
            user_actions(VIEWER, Role::Admin, &everything),
            [
                A::Unverify,
                A::Unorg,
                A::Role,
                A::Unsuspend,
                A::Purge,
                A::Rename,
                A::ResetName,
                A::ClearBio,
                A::Delete
            ]
        );
    }

    #[test]
    fn admins_are_safe_from_moderators_and_from_deletion() {
        let admin = target(user(None, false, false, false), Role::Admin, false);
        assert!(!user_actions(VIEWER, Role::Moderator, &admin).contains(&UserAction::Suspend));
        assert!(user_actions(VIEWER, Role::Admin, &admin).contains(&UserAction::Suspend));
        assert!(!user_actions(VIEWER, Role::Admin, &admin).contains(&UserAction::Delete));
    }

    #[test]
    fn nobody_suspends_re_roles_or_deletes_themselves() {
        let me = target(user(None, false, false, false), Role::Admin, false);
        let actions = user_actions(me.user.id, Role::Admin, &me);
        for blocked in [
            UserAction::Suspend,
            UserAction::Unsuspend,
            UserAction::Role,
            UserAction::Delete,
            UserAction::Purge,
        ] {
            assert!(!actions.contains(&blocked), "{blocked:?}");
        }
    }

    #[test]
    fn remote_accounts_keep_their_handle_name_and_role() {
        let remote = target(
            user(Some("remote.example"), false, false, false),
            Role::User,
            false,
        );
        assert_eq!(
            user_actions(VIEWER, Role::Admin, &remote),
            [UserAction::Verify, UserAction::Suspend, UserAction::Purge]
        );
    }

    #[test]
    fn a_deleted_account_and_a_plain_viewer_get_nothing() {
        let gone = target(user(None, false, false, false), Role::User, true);
        assert!(user_actions(VIEWER, Role::Admin, &gone).is_empty());
        assert!(user_actions(VIEWER, Role::User, &plain()).is_empty());
    }

    fn report(post: bool, status: ReportStatus, suspended: bool) -> AdminReportDto {
        let mut report = crate::admin::testing::sample_report();
        report.status = status;
        report.target_suspended = suspended;
        if !post {
            report.post = None;
        }
        report
    }

    #[test]
    fn a_report_suspends_together_with_the_post_when_there_is_one() {
        assert_eq!(
            report_actions(&report(true, ReportStatus::Open, false)),
            [
                ReportAction::Dismiss,
                ReportAction::DeletePost,
                ReportAction::DeleteAndSuspend
            ]
        );
    }

    #[test]
    fn an_account_report_suspends_the_account() {
        assert_eq!(
            report_actions(&report(false, ReportStatus::Open, false)),
            [ReportAction::Dismiss, ReportAction::SuspendUser]
        );
    }

    #[test]
    fn the_suspend_button_goes_once_suspended_and_everything_once_resolved() {
        assert_eq!(
            report_actions(&report(true, ReportStatus::Open, true)),
            [ReportAction::Dismiss, ReportAction::DeletePost]
        );
        assert!(report_actions(&report(true, ReportStatus::Resolved, false)).is_empty());
    }

    #[test]
    fn invites_are_live_until_revoked_used_up_or_expired() {
        let now = datetime!(2026-11-09 12:00 UTC);
        let base = InviteDto {
            code: "AAAAA-BBBBB".to_owned(),
            note: String::new(),
            max_uses: 3,
            uses: 0,
            expires_at: None,
            revoked: false,
            created_at: now,
            used_by: Vec::new(),
        };
        assert_eq!(invite_state(&base, now), InviteState::Active);
        let revoked = InviteDto {
            revoked: true,
            ..base.clone()
        };
        assert_eq!(invite_state(&revoked, now), InviteState::Revoked);
        let used = InviteDto {
            uses: 3,
            ..base.clone()
        };
        assert_eq!(invite_state(&used, now), InviteState::Used);
        let expired = InviteDto {
            expires_at: Some(datetime!(2026-11-09 11:59:59 UTC)),
            ..base.clone()
        };
        assert_eq!(invite_state(&expired, now), InviteState::Expired);
        let later = InviteDto {
            expires_at: Some(datetime!(2026-11-10 0:00 UTC)),
            ..base
        };
        assert_eq!(invite_state(&later, now), InviteState::Active);
    }

    #[test]
    fn announcements_are_live_until_ended_or_expired() {
        let now = datetime!(2026-11-09 12:00 UTC);
        let item = |active, expires_at| AdminAnnouncementDto {
            id: Uuid::from_u128(3),
            text: "Hi".to_owned(),
            tone: tuita_server::dto::AnnouncementTone::Info,
            link: String::new(),
            active,
            created_at: now,
            expires_at,
        };
        assert_eq!(
            announcement_state(&item(true, None), now),
            AnnouncementState::Live
        );
        assert_eq!(
            announcement_state(&item(false, None), now),
            AnnouncementState::Ended
        );
        assert_eq!(
            announcement_state(&item(true, Some(datetime!(2026-11-09 11:00 UTC))), now),
            AnnouncementState::Expired
        );
    }

    #[test]
    fn lines_are_trimmed_and_blanks_dropped() {
        assert_eq!(
            lines_of(" Be kind.\n\n  No spam. \n"),
            ["Be kind.", "No spam."]
        );
        assert!(lines_of("").is_empty());
    }
}
