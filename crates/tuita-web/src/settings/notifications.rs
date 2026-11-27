//! `/settings/notifications`.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_server::dto::MeDto;

use crate::{
    l10n::t,
    settings::{
        card::setting_card, desktop_notifications::desktop_notifications, page::settings_page,
        switch::setting_switch,
    },
};

/// One switch per notification kind, then the desktop permission.
#[component]
pub(crate) async fn notifications(cx: &Cx, me: &MeDto) -> Result<impl View> {
    let prefs = me.settings.notif_prefs;
    let title = t!(settings_notifications).resolve(cx).into_owned();
    let kinds = [
        (
            "like",
            t!(settings_notif_like).resolve(cx).into_owned(),
            t!(settings_notif_like_hint).resolve(cx).into_owned(),
            prefs.like,
        ),
        (
            "repost",
            t!(settings_notif_repost).resolve(cx).into_owned(),
            t!(settings_notif_repost_hint).resolve(cx).into_owned(),
            prefs.repost,
        ),
        (
            "reply",
            t!(settings_notif_reply).resolve(cx).into_owned(),
            t!(settings_notif_reply_hint).resolve(cx).into_owned(),
            prefs.reply,
        ),
        (
            "mention",
            t!(settings_notif_mention).resolve(cx).into_owned(),
            t!(settings_notif_mention_hint).resolve(cx).into_owned(),
            prefs.mention,
        ),
        (
            "quote",
            t!(settings_notif_quote).resolve(cx).into_owned(),
            t!(settings_notif_quote_hint).resolve(cx).into_owned(),
            prefs.quote,
        ),
        (
            "follow",
            t!(settings_notif_follow).resolve(cx).into_owned(),
            t!(settings_notif_follow_hint).resolve(cx).into_owned(),
            prefs.follow,
        ),
    ];
    Ok(view! {
        settings_page(
            title: &title,
            setting_card(
                title: t!(settings_notif_title).resolve(cx).into_owned(),
                description: t!(settings_notif_desc).resolve(cx).into_owned(),
                #[key(field)]
                for (field, label, hint, on) in kinds {
                    setting_switch(
                        field: field,
                        label: &label,
                        hint: hint.as_str(),
                        on: on
                    )
                }
            )
            desktop_notifications()
        )
    })
}
