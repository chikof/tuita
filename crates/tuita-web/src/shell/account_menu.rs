//! The account switcher in the rail.

use topcoat::{
    Result,
    context::Cx,
    runtime::Event,
    view::{View, component, view},
};

use crate::{
    auth::flows::{sign_out, switch_account},
    fmt::handle_of,
    icons::Icon,
    l10n::t,
    post::rich_text::emote_text,
    session::Viewer,
    shell::parts::account_avatar,
    ui::{icon::icon, popover::popover},
};

/// The id the popover is built from: its panel is `account-panel`.
const POPOVER: &str = "account";

/// A button showing the active account that opens a panel to switch accounts, add one or sign out.
///
/// Every sign-out button carries `data-forget-keys` with the ids of the accounts it signs out
/// (islands holding per-account device data clear it when it is clicked, before the request).
/// `shell.js` focuses the first row when the panel opens and moves between rows with the arrow
/// keys (`data-roving`). `id` replaces the popover id when a page shows several menus.
#[component]
pub(crate) async fn account_menu(
    cx: &Cx,
    viewer: &Viewer,
    #[default]
    #[into]
    id: Option<&str>,
) -> Result<impl View> {
    let popover_id = id.unwrap_or(POPOVER);
    let me = &viewer.me.user;
    let accounts = &viewer.accounts;
    let label = t!(account_menu).resolve(cx).into_owned();
    let panel = format!("{popover_id}-panel");
    let own_id = me.id.to_string();
    let every_id = accounts
        .accounts
        .iter()
        .map(|account| account.user.id.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    let can_add = accounts.accounts.len() < usize::from(accounts.max);
    let several = accounts.accounts.len() > 1;
    let rows: Vec<_> = accounts
        .accounts
        .iter()
        .map(|account| (account.user.id.to_string(), account))
        .collect();
    let logout = t!(account_logout, handle: format!("@{}", me.handle));
    Ok(view! {
        <div class="menu" data-c="account-menu">
            popover(
                id: popover_id,
                label: &label,
                trigger: view! {
                    <span class="account">
                        account_avatar(
                            handle: &me.handle,
                            src: me.avatar_url.as_deref(),
                            shape: me.avatar_shape,
                            ink: me.ink,
                            size: 40
                        )
                        <span class="who">
                            <span class="display">
                                emote_text(text: &me.display_name, emojis: &me.emojis)
                            </span>
                            <span class="handle">
                                (handle_of(&me.handle, me.domain.as_deref()))
                            </span>
                        </span>
                        icon(name: Icon::MoreHorizontal, size: 20)
                    </span>
                }.into(

                ),
                <div class="list" data-roving="">
                    for (id, account) in rows {
                        if account.active {
                            <button
                                type="button"
                                class="row"
                                aria-current="true"
                                popovertarget=(&panel)
                                popovertargetaction="hide"
                            >
                                account_avatar(
                                    handle: &account.user.handle,
                                    src: account.user.avatar_url.as_deref(),
                                    shape: account.user.avatar_shape,
                                    ink: account.user.ink,
                                    size: 40
                                )
                                <span class="who">
                                    <span class="display">
                                        emote_text(
                                            text: &account.user.display_name,
                                            emojis: &account.user.emojis
                                        )
                                    </span>
                                    <span class="handle">
                                        (handle_of(
                                            &account.user.handle,
                                            account.user.domain.as_deref(),
                                        ))
                                    </span>
                                </span>
                                icon(name: Icon::Check, size: 20)
                            </button>
                        } else {
                            <button
                                type="button"
                                class="row"
                                @click=$(async |_e: Event| {
                                    let done = switch_account(id).await;
                                    if done.is_ok() {
                                        raw!("location.assign('/')")
                                    } else {
                                        let msg = done.unwrap_err();
                                        raw!(
                                            "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                            msg.is_empty(),
                                        )
                                    }
                                })
                            >
                                account_avatar(
                                    handle: &account.user.handle,
                                    src: account.user.avatar_url.as_deref(),
                                    shape: account.user.avatar_shape,
                                    ink: account.user.ink,
                                    size: 40
                                )
                                <span class="who">
                                    <span class="display">
                                        emote_text(
                                            text: &account.user.display_name,
                                            emojis: &account.user.emojis
                                        )
                                    </span>
                                    <span class="handle">
                                        (handle_of(
                                            &account.user.handle,
                                            account.user.domain.as_deref(),
                                        ))
                                    </span>
                                </span>
                            </button>
                        }
                    }
                    <hr>
                    if can_add {
                        <a class="row plain" href="/login?add=1">
                            (t!(account_add_existing))
                        </a>
                    }
                    <button
                        type="button"
                        class="row plain danger"
                        data-forget-keys=(&own_id)
                        @click=$(async |_e: Event| {
                            let done = sign_out(false).await;
                            if done.is_ok() {
                                raw!("location.assign('/')")
                            } else {
                                let msg = done.unwrap_err();
                                raw!(
                                    "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                    msg.is_empty(),
                                )
                            }
                        })
                    >
                        (logout)
                    </button>
                    if several {
                        <button
                            type="button"
                            class="row plain danger"
                            data-forget-keys=(&every_id)
                            @click=$(async |_e: Event| {
                                let done = sign_out(true).await;
                                if done.is_ok() {
                                    raw!("location.assign('/')")
                                } else {
                                    let msg = done.unwrap_err();
                                    raw!(
                                        "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                        msg.is_empty(),
                                    )
                                }
                            })
                        >
                            (t!(account_logout_all))
                        </button>
                    }
                </div>
            )
        </div>
    })
}
