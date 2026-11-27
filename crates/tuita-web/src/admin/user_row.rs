//! One account of a list: a button that opens its details and its menu.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal},
    view::{View, component, view},
};
use tuita_server::dto::{AdminUserDto, MeDto};

use crate::{
    admin::{
        common::{stamp, tr},
        user_menu::user_menu,
    },
    fmt::{compact, handle_of, rel_label},
    post::rich_text::user_name,
    shell::parts::account_avatar,
    ui::tag::tag,
};

/// The row of `entry`. A click on the name sets `detail` to the account's id and bumps `nonce`
/// (so the detail sheet reads it again even for the same account) and opens the sheet
/// `#user-detail`. `note` is a line under the facts, e.g. why the word filter flagged it.
/// `viewer` is passed on to the menu (see [`user_menu`]).
#[component]
pub(crate) async fn admin_user_row(
    cx: &Cx,
    entry: &AdminUserDto,
    scope: &str,
    revision: &Signal<u32>,
    detail: &Signal<String>,
    nonce: &Signal<u32>,
    #[default] note: Option<&str>,
    #[default] viewer: Option<&MeDto>,
) -> Result<impl View> {
    let user = &entry.user;
    let handle = handle_of(&user.handle, user.domain.as_deref());
    let id = user.id.to_string();
    let label = tr!(cx, admin_user_details, handle: handle.as_str());
    let mut facts = vec![
        tr!(
            cx,
            admin_user_posts,
            count: user.counts.posts,
            compact: compact(cx, i64::from(user.counts.posts))
        ),
        tr!(
            cx,
            admin_user_followers,
            count: user.counts.followers,
            compact: compact(cx, i64::from(user.counts.followers))
        ),
        tr!(cx, admin_user_joined, time: rel_label(cx, &stamp(user.created_at))),
    ];
    if let Some(at) = entry.last_active_at {
        facts.push(tr!(cx, admin_user_active, time: rel_label(cx, &stamp(at))));
    }
    let facts = facts.join(" · ");
    let role = (entry.role != tuita_server::dto::Role::User).then(|| entry.role.as_str());
    let org = user.org.then(|| tr!(cx, admin_tag_org));
    let suspended = user.suspended.then(|| tr!(cx, admin_tag_suspended));
    let deleted = entry.deleted.then(|| tr!(cx, admin_tag_deleted));
    let reports = (entry.open_reports > 0).then(|| {
        tr!(
            cx,
            admin_user_open_reports,
            count: entry.open_reports,
            compact: compact(cx, i64::from(entry.open_reports))
        )
    });
    Ok(view! {
        <div class="row" data-c="admin-user-row">
            <button
                type="button"
                class="main wash"
                aria-label=(&label)
                @click=$(|_e: Event| {
                    detail.set(id.to_owned());
                    nonce.increment();
                    raw!("document.getElementById('user-detail').showModal()")
                })
            >
                <span aria-hidden="true">
                    account_avatar(
                        handle: &user.handle,
                        src: user.avatar_url.as_deref(),
                        shape: user.avatar_shape,
                        ink: user.ink,
                        size: 40
                    )
                </span>
                <span class="text">
                    <span class="line">
                        user_name(
                            name: &user.display_name,
                            verified: user.verified,
                            bot: user.bot
                        )
                        <span class="handle">(&handle)</span>
                    </span>
                    <span class="tags">
                        if let Some(role) = role {
                            tag(pill: true, compact: true, (role))
                        }
                        if let Some(text) = &org {
                            tag(pill: true, compact: true, (text))
                        }
                        if let Some(text) = &suspended {
                            tag(pill: true, compact: true, (text))
                        }
                        if let Some(text) = &deleted {
                            tag(pill: true, compact: true, (text))
                        }
                        if let Some(text) = &reports {
                            tag(pill: true, compact: true, (text))
                        }
                    </span>
                    <span class="facts">(&facts)</span>
                    if let Some(note) = note {
                        <span class="facts note">(note)</span>
                    }
                </span>
            </button>
            user_menu(target: entry, scope: scope, revision: revision, viewer: viewer)
        </div>
    })
}
