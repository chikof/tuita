//! The slide-in menu on phones.

use topcoat::{
    Result,
    context::Cx,
    runtime::Event,
    view::{View, attributes, component, view},
};

use crate::{
    auth::flows::{sign_out, switch_account},
    fmt::handle_of,
    icons::Icon,
    l10n::t,
    paths,
    post::rich_text::emote_text,
    session::{Viewer, is_staff},
    shell::{
        parts::{account_avatar, nav_button},
        top_bar::DRAWER,
    },
    ui::{
        button::{ButtonVariant, button},
        icon::icon,
        icon_button::icon_button,
    },
};

/// A modal `<dialog id="drawer">` with the account, the secondary links and the account actions.
///
/// The top bar opens it (`command="show-modal"`); the close button and a click outside close it
/// (`command="close"`, `closedby="any"`). A page load replaces it, so it needs no close-on-
/// navigation handling. `id` replaces the default dialog id (the design gallery has a section
/// named `drawer`). The sign-out buttons carry `data-forget-keys`, as in the account menu.
#[component]
pub(crate) async fn drawer(
    cx: &Cx,
    viewer: &Viewer,
    #[default]
    #[into]
    id: Option<&str>,
) -> Result<impl View> {
    let dialog_id = id.unwrap_or(DRAWER);
    let me = &viewer.me.user;
    let profile = paths::profile(&me.handle);
    let staff = is_staff(viewer);
    let others: Vec<_> = viewer
        .accounts
        .accounts
        .iter()
        .filter(|account| !account.active)
        .map(|account| (account.user.id.to_string(), account))
        .collect();
    let every_id = viewer
        .accounts
        .accounts
        .iter()
        .map(|account| account.user.id.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    let can_add = viewer.accounts.accounts.len() < usize::from(viewer.accounts.max);
    let several = viewer.accounts.accounts.len() > 1;
    let close = t!(close).resolve(cx).into_owned();
    let logout = t!(account_logout, handle: format!("@{}", me.handle))
        .resolve(cx)
        .into_owned();
    let own_id = me.id.to_string();
    Ok(view! {
        <dialog
            id=(dialog_id)
            class="drawer"
            aria-label=(t!(nav_menu))
            closedby="any"
            data-c="drawer"
        >
            <div class="panel">
                <div class="head">
                    <a href=(&profile) class="me">
                        account_avatar(
                            handle: &me.handle,
                            src: me.avatar_url.as_deref(),
                            shape: me.avatar_shape,
                            ink: me.ink,
                            size: 48
                        )
                        <span class="who">
                            <span class="display">
                                emote_text(text: &me.display_name, emojis: &me.emojis)
                            </span>
                            <span class="handle">
                                (handle_of(&me.handle, me.domain.as_deref()))
                            </span>
                        </span>
                    </a>
                    icon_button(
                        glyph: Icon::Close,
                        label: &close,
                        attrs: attributes! { command="close" commandfor=(dialog_id) }
                    )
                </div>
                <nav aria-label=(t!(nav_menu))>
                    <a href=(&profile)>
                        icon(name: Icon::User, size: 24)
                        (t!(nav_profile))
                    </a>
                    <a href="/messages">
                        icon(name: Icon::Mail, size: 24)
                        (t!(nav_messages))
                    </a>
                    <a href="/bookmarks">
                        icon(name: Icon::Bookmark, size: 24)
                        (t!(nav_bookmarks))
                    </a>
                    <a href="/lists">
                        icon(name: Icon::ListBox, size: 24)
                        (t!(nav_lists))
                    </a>
                    <a href="/scheduled">
                        icon(name: Icon::Calendar, size: 24)
                        (t!(nav_scheduled))
                    </a>
                    <a href="/settings">
                        icon(name: Icon::Gear, size: 24)
                        (t!(nav_settings))
                    </a>
                    if staff {
                        <a href="/admin">
                            icon(name: Icon::Shield, size: 24)
                            (t!(nav_admin))
                        </a>
                    }
                </nav>
                if !others.is_empty() {
                    <h2>(t!(account_switch))</h2>
                    <ul>
                        for (id, account) in others {
                            <li>
                                <button
                                    type="button"
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
                                        size: 32
                                    )
                                    (handle_of(
                                        &account.user.handle,
                                        account.user.domain.as_deref(),
                                    ))
                                </button>
                            </li>
                        }
                    </ul>
                }
                <div class="foot">
                    if can_add {
                        nav_button(
                            href: "/login?add=1",
                            variant: ButtonVariant::Outline,
                            (t!(account_add_existing))
                        )
                    }
                    button(
                        variant: ButtonVariant::Danger,
                        attrs: attributes! {
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
                        },
                        (&logout)
                    )
                    if several {
                        button(
                            variant: ButtonVariant::Danger,
                            attrs: attributes! {
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
                            },
                            (t!(account_logout_all))
                        )
                    }
                </div>
            </div>
        </dialog>
    })
}
