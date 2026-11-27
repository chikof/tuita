//! `/settings/privacy`.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, signal},
    view::{View, component, view},
};
use tuita_server::dto::MeDto;

use crate::{
    l10n::t,
    settings::{
        actions::save_dm_policy,
        card::setting_card,
        muted_words::muted_words,
        page::settings_page,
        relation_list::{RelationKind, relation_list},
        switch::setting_switch,
    },
    ui::{segmented::segmented, tabs::Choice},
};

/// Who may message, discoverability, muted words and the blocked and muted accounts.
#[component]
pub(crate) async fn privacy(cx: &Cx, me: &MeDto) -> Result<impl View> {
    let title = t!(settings_privacy).resolve(cx).into_owned();
    let dm = signal(cx, || me.settings.dm_policy.as_str().to_owned());
    let saved = signal(cx, || me.settings.dm_policy.as_str().to_owned());
    let everyone = t!(settings_dm_everyone).resolve(cx).into_owned();
    let following = t!(settings_dm_following).resolve(cx).into_owned();
    let none = t!(settings_dm_none).resolve(cx).into_owned();
    let dm_label = t!(settings_dm_label).resolve(cx).into_owned();
    let discover = t!(settings_discoverable).resolve(cx).into_owned();
    let discover_hint = t!(settings_discoverable_hint).resolve(cx).into_owned();
    let client = t!(settings_show_client).resolve(cx).into_owned();
    let client_hint = t!(settings_show_client_hint).resolve(cx).into_owned();
    Ok(view! {
        settings_page(
            title: &title,
            setting_card(
                title: t!(settings_dm_title).resolve(cx).into_owned(),
                description: t!(settings_dm_desc).resolve(cx).into_owned(),
                <div
                    data-c="dm-policy"
                    @click=$(async |_e: Event| {
                        let next = dm.get();
                        if next != saved.get() {
                            let done = save_dm_policy(dm.get()).await;
                            if done.is_ok() {
                                saved.set(dm.get());
                                let msg = done.unwrap();
                                raw!(
                                    "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                    msg.is_empty(),
                                )
                            } else {
                                dm.set(saved.get());
                                let msg = done.unwrap_err();
                                raw!(
                                    "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                    msg.is_empty(),
                                )
                            };
                        }
                    })
                >
                    segmented(
                        items: vec![
                            Choice::new("everyone", &everyone),
                            Choice::new("following", &following),
                            Choice::new("none", &none),
                        ],
                        value: &dm,
                        label: &dm_label
                    )
                </div>
            )
            setting_card(
                title: t!(settings_discover_title).resolve(cx).into_owned(),
                setting_switch(
                    field: "discoverable",
                    label: &discover,
                    hint: discover_hint.as_str(),
                    on: me.settings.discoverable
                )
                setting_switch(
                    field: "showClient",
                    label: &client,
                    hint: client_hint.as_str(),
                    on: me.settings.show_client
                )
            )
            muted_words(words: me.settings.muted_words.clone())
            relation_list(kind: RelationKind::Blocks)
            relation_list(kind: RelationKind::Mutes)
        )
    })
}
