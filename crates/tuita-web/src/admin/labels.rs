//! Words for the enums and audit keys of the console.

use topcoat::context::Cx;
use tuita_server::dto::{ReportAction, ReportReason};

use crate::l10n::t;

pub(crate) fn reason_label(cx: &Cx, reason: ReportReason) -> String {
    match reason {
        ReportReason::Spam => t!(report_spam),
        ReportReason::Harassment => t!(report_harassment),
        ReportReason::Hate => t!(report_hate),
        ReportReason::Violence => t!(report_violence),
        ReportReason::Misinformation => t!(report_misinformation),
        ReportReason::Impersonation => t!(report_impersonation),
        ReportReason::Other => t!(report_other),
    }
    .resolve(cx)
    .into_owned()
}

/// What staff did, as a past-tense phrase.
pub(crate) fn resolution_label(cx: &Cx, action: ReportAction) -> String {
    match action {
        ReportAction::Dismiss => t!(admin_res_dismiss),
        ReportAction::DeletePost => t!(admin_res_delete_post),
        ReportAction::SuspendUser => t!(admin_res_suspend_user),
        ReportAction::DeleteAndSuspend => t!(admin_res_delete_and_suspend),
    }
    .resolve(cx)
    .into_owned()
}

/// The button for each action.
pub(crate) fn report_button(cx: &Cx, action: ReportAction) -> String {
    match action {
        ReportAction::Dismiss => t!(admin_report_dismiss),
        ReportAction::DeletePost => t!(admin_report_delete_post),
        ReportAction::SuspendUser => t!(admin_report_suspend),
        ReportAction::DeleteAndSuspend => t!(admin_report_delete_suspend),
    }
    .resolve(cx)
    .into_owned()
}

/// What an audit row did, in words; an action key this build does not know is shown as it is.
pub(crate) fn audit_label(cx: &Cx, action: &str) -> String {
    let text = match action {
        "admin.bootstrap" => t!(admin_act_log_admin_bootstrap),
        "announcement.create" => t!(admin_act_log_announcement_create),
        "announcement.end" => t!(admin_act_log_announcement_end),
        "domain.block" => t!(admin_act_log_domain_block),
        "domain.unblock" => t!(admin_act_log_domain_unblock),
        "emoji.create" => t!(admin_act_log_emoji_create),
        "emoji.delete" => t!(admin_act_log_emoji_delete),
        "emoji.update" => t!(admin_act_log_emoji_update),
        "federation.retry" => t!(admin_act_log_federation_retry),
        "instance.settings" => t!(admin_act_log_instance_settings),
        "invite.create" => t!(admin_act_log_invite_create),
        "invite.revoke" => t!(admin_act_log_invite_revoke),
        "post.delete" => t!(admin_act_log_post_delete),
        "report.dismiss" => t!(admin_act_log_report_dismiss),
        "user.delete" => t!(admin_act_log_user_delete),
        "user.org" => t!(admin_act_log_user_org),
        "user.unorg" => t!(admin_act_log_user_unorg),
        "user.purge_posts" => t!(admin_act_log_user_purge_posts),
        "user.rename" => t!(admin_act_log_user_rename),
        "user.role" => t!(admin_act_log_user_role),
        "user.suspend" => t!(admin_act_log_user_suspend),
        "user.unsuspend" => t!(admin_act_log_user_unsuspend),
        "user.unverify" => t!(admin_act_log_user_unverify),
        "user.verify" => t!(admin_act_log_user_verify),
        other => return other.to_owned(),
    };
    text.resolve(cx).into_owned()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::context;

    const EMITTED: [&str; 18] = [
        "admin.bootstrap",
        "announcement.create",
        "announcement.end",
        "instance.settings",
        "invite.create",
        "invite.revoke",
        "post.delete",
        "report.dismiss",
        "user.delete",
        "user.org",
        "user.unorg",
        "user.purge_posts",
        "user.rename",
        "user.role",
        "user.suspend",
        "user.unsuspend",
        "user.unverify",
        "user.verify",
    ];

    #[test]
    fn every_action_the_server_writes_has_words() {
        let cx = context("en");
        for action in EMITTED {
            assert_ne!(audit_label(&cx, action), action, "{action}");
        }
        assert_eq!(audit_label(&cx, "user.suspend"), "Account suspended");
        assert_eq!(audit_label(&cx, "admin.bootstrap"), "First admin set up");
    }

    #[test]
    fn an_unknown_action_keeps_its_key() {
        assert_eq!(
            audit_label(&context("en"), "federation.block"),
            "federation.block"
        );
    }

    #[test]
    fn reasons_and_resolutions_read_in_the_locale() {
        let cx = context("en");
        assert_eq!(reason_label(&cx, ReportReason::Spam), "Spam");
        assert_eq!(resolution_label(&cx, ReportAction::Dismiss), "Dismissed");
        assert_eq!(
            report_button(&cx, ReportAction::DeleteAndSuspend),
            "Delete and suspend"
        );
    }
}
