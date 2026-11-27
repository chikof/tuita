//! The menu of an account and the dialogs its entries open.
//!
//! The component is rendered inside a shard (a list of accounts, the detail of one): an entry
//! either runs `run_user_action` at once or opens a confirmation first, and then bumps
//! `revision` so the shard renders again from the API.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal, signal},
    view::{Attributes, View, attributes, component, view},
};
use tuita_server::dto::{AdminUserDto, MeDto, Role};

use crate::{
    admin::{
        common::tr,
        rules::{UserAction, user_actions},
        user_actions::{rename_user, run_user_action},
    },
    icons::Icon,
    session,
    ui::{
        button::{ButtonVariant, button},
        confirm_dialog::confirm_dialog,
        dialog::dialog,
        icon::icon,
        menu::{MenuItem, menu},
        switch::switch,
        text_field::text_field,
    },
};

/// An entry of the menu with what it asks before it runs.
struct Entry {
    /// The `action` argument of `run_user_action`, or `rename`.
    action: String,
    label: String,
    danger: bool,
    confirm: Option<Confirm>,
}

/// The dialog an entry opens before it acts.
struct Confirm {
    id: String,
    title: String,
    body: String,
    button: String,
    danger: bool,
}

fn role_label(cx: &Cx, role: Role) -> String {
    match role {
        Role::User => tr!(cx, admin_act_role_user),
        Role::Moderator => tr!(cx, admin_act_role_moderator),
        Role::Admin => tr!(cx, admin_act_role_admin),
    }
}

fn role_phrase(cx: &Cx, role: Role) -> String {
    match role {
        Role::User => tr!(cx, admin_role_phrase_user),
        Role::Moderator => tr!(cx, admin_role_phrase_moderator),
        Role::Admin => tr!(cx, admin_role_phrase_admin),
    }
}

/// The entries `actions` stand for. `scope` keeps the ids of two menus for the same account apart.
fn entries(
    cx: &Cx,
    target: &AdminUserDto,
    actions: &[UserAction],
    scope: &str,
    who: &str,
) -> Vec<Entry> {
    let id = target.user.id;
    let plain = |action: &str, label: String, danger: bool| Entry {
        action: action.to_owned(),
        label,
        danger,
        confirm: None,
    };
    let asking = |action: &str, label: String, danger: bool, confirm: Confirm| Entry {
        action: action.to_owned(),
        label,
        danger,
        confirm: Some(confirm),
    };
    let confirm =
        |action: &str, title: String, body: String, button: String, danger: bool| Confirm {
            id: format!("confirm-{scope}-{id}-{}", action.replace(':', "-")),
            title,
            body,
            button,
            danger,
        };
    let mut out = Vec::new();
    for action in actions {
        match action {
            UserAction::Verify => out.push(plain("verify", tr!(cx, admin_act_verify), false)),
            UserAction::Unverify => out.push(plain("unverify", tr!(cx, admin_act_unverify), false)),
            UserAction::Org => out.push(plain("org", tr!(cx, admin_act_org), false)),
            UserAction::Unorg => out.push(asking(
                "unorg",
                tr!(cx, admin_act_unorg),
                false,
                confirm(
                    "unorg",
                    tr!(cx, admin_unorg_title, handle: who),
                    tr!(cx, admin_unorg_body),
                    tr!(cx, admin_unorg_confirm),
                    true,
                ),
            )),
            UserAction::Role => {
                for role in [Role::User, Role::Moderator, Role::Admin] {
                    if role == target.role {
                        continue;
                    }
                    let name = format!("role:{}", role.as_str());
                    out.push(asking(
                        &name,
                        role_label(cx, role),
                        false,
                        confirm(
                            &name,
                            tr!(cx, admin_role_title, handle: who),
                            tr!(cx, admin_role_body, role: role_phrase(cx, role)),
                            tr!(cx, admin_role_confirm),
                            false,
                        ),
                    ));
                }
            }
            UserAction::Suspend => out.push(asking(
                "suspend",
                tr!(cx, admin_act_suspend),
                true,
                confirm(
                    "suspend",
                    tr!(cx, admin_suspend_title, handle: who),
                    tr!(cx, admin_suspend_body),
                    tr!(cx, admin_suspend_confirm),
                    true,
                ),
            )),
            UserAction::Unsuspend => {
                out.push(plain("unsuspend", tr!(cx, admin_act_unsuspend), false));
            }
            UserAction::Purge => out.push(asking(
                "purge",
                tr!(cx, admin_act_purge),
                true,
                confirm(
                    "purge",
                    tr!(cx, admin_purge_title, handle: who),
                    tr!(cx, admin_purge_body),
                    tr!(cx, admin_purge_confirm),
                    true,
                ),
            )),
            UserAction::Rename => out.push(plain("rename", tr!(cx, admin_act_rename), false)),
            UserAction::ResetName => out.push(asking(
                "resetName",
                tr!(cx, admin_act_reset_name),
                false,
                confirm(
                    "resetName",
                    tr!(cx, admin_reset_name_title, handle: who),
                    tr!(cx, admin_reset_name_body),
                    tr!(cx, admin_reset_name_confirm),
                    false,
                ),
            )),
            UserAction::ClearBio => {
                out.push(plain("clearBio", tr!(cx, admin_act_clear_bio), false))
            }
            UserAction::Delete => out.push(asking(
                "delete",
                tr!(cx, admin_act_delete),
                true,
                confirm(
                    "delete",
                    tr!(cx, admin_delete_title, handle: who),
                    tr!(cx, admin_delete_body),
                    tr!(cx, admin_delete_confirm),
                    true,
                ),
            )),
        }
    }
    out
}

/// The click handler of an entry that acts at once: runs the action, toasts the answer and
/// bumps `revision`. An account that was deleted closes the detail sheet it was opened from.
#[allow(
    clippy::needless_pass_by_value,
    reason = "attributes! captures the values by name for the browser expression"
)]
fn run_attrs(
    cx: &Cx,
    id: String,
    action: String,
    handle: String,
    revision: Signal<u32>,
) -> Attributes {
    let closes = action == "delete";
    attributes! {
        @click=$(async |_e: Event| {
            let done = run_user_action(id, action, handle).await;
            if done.is_ok() {
                let msg = done.unwrap();
                raw!(
                    "${closes} && document.getElementById('user-detail')?.close();",
                    {
                        let _ = closes;
                    },
                );
                raw!(
                    "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}));",
                    drop(msg),
                );
                revision.increment();
                false
            } else {
                let msg = done.unwrap_err();
                raw!(
                    "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}))",
                    msg.is_empty(),
                )
            }
        })
    }
}

/// The click handler of an entry that opens the dialog `dialog_id`.
fn open_attrs(cx: &Cx, dialog_id: String) -> Attributes {
    attributes! {
        @click=$(|_e: Event| raw!(
                "document.getElementById(${dialog_id}).showModal()",
                dialog_id.is_empty(),
            ))
    }
}

/// The "Manage" menu of `target` for the signed-in staff member; nothing when no action is open
/// to them. `scope` (`row`, `detail`, `scan`) keeps ids unique when an account shows twice.
/// `viewer` names the staff member when the caller already knows who looks (the design gallery);
/// without it the signed-in member of the request is used.
#[component]
pub(crate) async fn user_menu(
    cx: &Cx,
    target: &AdminUserDto,
    scope: &str,
    revision: &Signal<u32>,
    #[default] viewer: Option<&MeDto>,
) -> Result<impl View> {
    let (viewer_id, viewer_role) = match viewer {
        Some(me) => (me.user.id, me.role),
        None => {
            let member = session::member(cx).await?;
            (member.me.user.id, member.me.role)
        }
    };
    let actions = user_actions(viewer_id, viewer_role, target);
    let user = &target.user;
    let who = format!("@{}", user.handle);
    let id = user.id.to_string();
    let entries = entries(cx, target, &actions, scope, &who);
    let label = tr!(cx, admin_user_menu, handle: who.as_str());
    let renames = actions.contains(&UserAction::Rename);
    let rename_id = format!("rename-{scope}-{id}");
    let handle = user.handle.clone();
    let busy = signal(cx, || false);
    let next_handle = signal(cx, || handle.clone());
    let reset_name = signal(cx, || false);
    let error = signal(cx, String::new);
    let rename_title = tr!(cx, admin_rename_title, handle: who.as_str());
    let rename_note = tr!(cx, admin_rename_note);
    let rename_label = tr!(cx, admin_rename_handle);
    let rename_reset = tr!(cx, admin_rename_reset_name);
    let rename_save = tr!(cx, admin_rename_save);
    let cancel = tr!(cx, cancel);
    let form_id = format!("{rename_id}-form");
    let rename_user_id = id.clone();
    let rename_current = handle.clone();
    let rename_dialog = rename_id.clone();
    let has = !actions.is_empty();
    let menu_id = format!("um-{scope}-{id}");
    let trigger_label = label.clone();
    let close_target = rename_id.clone();
    let save_form = form_id.clone();
    let save_busy = busy.clone();
    Ok(view! {
        if has {
            menu(
                id: &menu_id,
                label: &label,
                plain: true,
                items: entries
                    .iter()
                    .map(
                        |entry| MenuItem {
                                id: &entry.action,
                                label: &entry.label,
                                danger: entry.danger,
                                attrs: match (&entry.confirm, entry.action.as_str()) {
                                    (Some(confirm), _) => open_attrs(cx, confirm.id.clone()),
                                    (None, "rename") => open_attrs(cx, rename_id.clone()),
                                    (None, action) => run_attrs(
                                        cx,
                                        id.clone(),
                                        action.to_owned(),
                                        handle.clone(),
                                        revision.clone(),
                                    ),
                                },
                                ..MenuItem::default()
                            },
                    )
                    .collect(),
                trigger: view! {
                    <span class="more">
                        icon(name: Icon::MoreVertical, size: 20)
                        <span class="sr-only">(&trigger_label)</span>
                    </span>
                }.into(

                )
            )
            #[key(entry.action.as_bytes())]
            for entry in &entries {
                if let Some(confirm) = &entry.confirm {
                    let user_id = id.clone();
                    let action = entry.action.clone();
                    let name = handle.clone();
                    let closes = entry.action == "delete";
                    confirm_dialog(
                        id: &confirm.id,
                        title: &confirm.title,
                        body: &confirm.body,
                        confirm_label: &confirm.button,
                        danger: confirm.danger,
                        action: $(async |_e: Event| {
                            let done = run_user_action(user_id, action, name).await;
                            if done.is_ok() {
                                let msg = done.unwrap();
                                raw!(
                                    "${closes} && document.getElementById('user-detail')?.close();",
                                    {
                                        let _ = closes;
                                    },
                                );
                                raw!(
                                    "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}));",
                                    drop(msg),
                                );
                                revision.increment();
                                false
                            } else {
                                let msg = done.unwrap_err();
                                raw!(
                                    "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}))",
                                    msg.is_empty(),
                                )
                            }
                        })
                    )
                }
            }
            if renames {
                dialog(
                    id: &rename_id,
                    title: &rename_title,
                    actions: Some(
                        view! {
                            button(
                                variant: ButtonVariant::Ghost,
                                attrs: attributes! { command="close" commandfor=(&close_target) },
                                (&cancel)
                            )
                            button(
                                submit: true,
                                attrs: attributes! { form=(&save_form) :disabled=$(save_busy.get()) },
                                (&rename_save)
                            )
                        }.into(

                        ),
                    ),
                    <form
                        id=(&form_id)
                        class="form"
                        data-c="admin-rename-form"
                        @submit=$(async |e: Event| {
                            e.prevent_default();
                            error.set("".to_owned());
                            busy.set(true);
                            let done = rename_user(
                                rename_user_id,
                                next_handle.get(),
                                rename_current,
                                reset_name.get(),
                            ).await;
                            busy.set(false);
                            if done.is_ok() {
                                let msg = done.unwrap();
                                raw!(
                                    "document.getElementById(${rename_dialog})?.close();",
                                    drop(rename_dialog),
                                );
                                raw!(
                                    "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}));",
                                    drop(msg),
                                );
                                revision.increment();
                                false
                            } else {
                                error.set(done.unwrap_err());
                                false
                            }
                        })
                    >
                        <p class="note">(&rename_note)</p>
                        text_field(
                            label: &rename_label,
                            attrs: attributes! {
                                name="handle"
                                maxlength="15"
                                autocapitalize="off"
                                spellcheck="false"
                                required=""
                                :value=$(next_handle.get())
                                @input=$(|e: Event| next_handle.set(e.target.value))
                            }
                        )
                        switch(label: &rename_reset, checked: &reset_name)
                        <p class="error" role="alert" :hidden=$(error.get().is_empty())>
                            $(error.get())
                        </p>
                    </form>
                )
            }
        }
    })
}
