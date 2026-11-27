//! The blocked and muted account lists.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal, procedure, shard, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{Page, UserDto};

use crate::{
    api, fmt,
    l10n::t,
    paths,
    post::rich_text::user_name,
    settings::{
        actions::{Done, failed},
        card::setting_card,
    },
    shell::parts::account_avatar,
    ui::{
        button::{ButtonSize, ButtonVariant, button},
        empty_state::empty_state,
    },
};

/// Which list: the accounts the viewer blocked, or muted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RelationKind {
    Blocks,
    Mutes,
}

impl RelationKind {
    /// The list's API path under `/me`.
    const fn path(self) -> &'static str {
        match self {
            Self::Blocks => "/me/blocks",
            Self::Mutes => "/me/mutes",
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Blocks => "blocks",
            Self::Mutes => "mutes",
        }
    }
}

/// Ends the block or mute of `acct`; answers the toast.
#[procedure]
pub(crate) async fn undo_relation(cx: &Cx, kind: String, acct: String) -> Done {
    let (suffix, unblock) = match kind.as_str() {
        "blocks" => ("block", true),
        "mutes" => ("mute", false),
        _ => return Ok(Err("Unknown list".to_owned())),
    };
    let path = format!("/users/{}/{suffix}", api::enc(&acct));
    let handle = format!("@{acct}");
    Ok(failed(api::delete::<UserDto>(cx, &path).await).map(|_| {
        if unblock {
            t!(unblocked_toast, handle: handle.as_str())
        } else {
            t!(unmuted_toast, handle: handle.as_str())
        }
        .resolve(cx)
        .into_owned()
    }))
}

/// The rows of one list: `pages` pages from the API, with a button for the next one. Reruns when
/// `rev` (a change in the list) or `pages` (the viewer asked for more) moves.
#[shard]
async fn relation_rows(
    cx: &Cx,
    kind: String,
    rev: Signal<u32>,
    pages: Signal<u32>,
) -> Result<impl View> {
    let _ = rev.get();
    let kind = if kind == "mutes" {
        RelationKind::Mutes
    } else {
        RelationKind::Blocks
    };
    let wanted = pages.get().clamp(1, 50);
    let mut users: Vec<UserDto> = Vec::new();
    let mut cursor: Option<String> = None;
    for _ in 0..wanted {
        let page = api::get::<Page<UserDto>>(
            cx,
            &format!(
                "{}{}",
                kind.path(),
                api::query(&[("before", cursor.as_deref())])
            ),
        )
        .await?;
        users.extend(page.items);
        cursor = page.next;
        if cursor.is_none() {
            break;
        }
    }
    let more = cursor.is_some();
    let blocks = kind == RelationKind::Blocks;
    let rows: Vec<_> = users
        .into_iter()
        .map(|user| {
            let acct = fmt::acct_of(&user.handle, user.domain.as_deref());
            let handle = fmt::handle_of(&user.handle, user.domain.as_deref());
            let label = if blocks {
                t!(settings_unblock_user, handle: handle.as_str())
            } else {
                t!(settings_unmute_user, handle: handle.as_str())
            }
            .resolve(cx)
            .into_owned();
            (user, acct, handle, label)
        })
        .collect();
    let empty_title = if blocks {
        t!(settings_blocked_empty)
    } else {
        t!(settings_muted_empty)
    }
    .resolve(cx)
    .into_owned();
    let list = kind.name();
    let action = if blocks {
        t!(settings_unblock)
    } else {
        t!(settings_unmute)
    }
    .resolve(cx)
    .into_owned();
    Ok(view! {
        if rows.is_empty() {
            empty_state(context: list, title: &empty_title)
        } else {
            <ul class="people">
                #[key(acct.as_str())]
                for (user, acct, handle, label) in rows {
                    <li class="row">
                        <span aria-hidden="true">
                            account_avatar(
                                handle: &user.handle,
                                src: user.avatar_url.as_deref(),
                                shape: user.avatar_shape,
                                ink: user.ink,
                                size: 40
                            )
                        </span>
                        <a class="who" href=(paths::profile(&acct))>
                            user_name(
                                name: &user.display_name,
                                verified: user.verified,
                                bot: user.bot,
                                emojis: &user.emojis
                            )
                            <span class="handle">(&handle)</span>
                        </a>
                        button(
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            attrs: attributes! {
                                aria-label=(&label)
                                @click=$(async |_e: Event| {
                                    let done = undo_relation(list.to_owned(), acct).await;
                                    if done.is_ok() {
                                        let msg = done.unwrap();
                                        rev.increment();
                                        raw!(
                                            "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                            msg.is_empty(),
                                        )
                                    } else {
                                        let msg = done.unwrap_err();
                                        raw!(
                                            "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:${msg}}))",
                                            msg.is_empty(),
                                        )
                                    }
                                })
                            },
                            (&action)
                        )
                    </li>
                }
            </ul>
            if more {
                <div class="more">
                    button(
                        variant: ButtonVariant::Outline,
                        attrs: attributes! { @click=$(|_e: Event| pages.increment()) },
                        (t!(rail_show_more))
                    )
                </div>
            }
        }
    })
}

/// A card with the viewer's blocked or muted accounts and a button to end each.
#[component]
pub(crate) async fn relation_list(cx: &Cx, kind: RelationKind) -> Result<impl View> {
    let rev = signal(cx, || 0u32);
    let pages = signal(cx, || 1u32);
    let blocks = kind == RelationKind::Blocks;
    let title = if blocks {
        t!(settings_blocked_title)
    } else {
        t!(settings_muted_title)
    }
    .resolve(cx)
    .into_owned();
    let description = if blocks {
        t!(settings_blocked_desc)
    } else {
        t!(settings_muted_desc)
    }
    .resolve(cx)
    .into_owned();
    Ok(view! {
        setting_card(
            title: &title,
            description: &description,
            <div data-c="relation-list">
                relation_rows(kind: kind.name().to_owned(), rev: rev, pages: pages)
            </div>
        )
    })
}
