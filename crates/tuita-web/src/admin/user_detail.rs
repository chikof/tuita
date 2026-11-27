//! The sheet with everything about one account.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Signal, shard},
    view::{View, component, view},
};
use tuita_server::dto::{AdminUserDetailDto, ReportStatus};
use uuid::Uuid;

use crate::{
    admin::{
        common::{stamp, tr},
        labels::{audit_label, reason_label, resolution_label},
        post_preview::post_preview,
        user_menu::user_menu,
    },
    api,
    fmt::{compact, full_time, handle_of, rel_label},
    post::rich_text::user_name,
    shell::parts::account_avatar,
    ui::{
        dialog::{DialogVariant, dialog},
        tag::tag,
    },
};

/// One cell of the numbers grid.
struct Stat {
    name: String,
    count: String,
}

/// The sheet `#user-detail`, empty until a row sets `detail`. The content is a shard: it reads
/// `detail` (the account), `nonce` (bumped on every open) and `revision` again.
#[component]
pub(crate) async fn user_detail_sheet(
    cx: &Cx,
    detail: &Signal<String>,
    nonce: &Signal<u32>,
    revision: &Signal<u32>,
) -> Result<impl View> {
    let title = tr!(cx, admin_detail_title);
    Ok(view! {
        dialog(
            id: "user-detail",
            title: &title,
            variant: DialogVariant::Sheet,
            user_detail(
                detail: detail.clone(),
                nonce: nonce.clone(),
                revision: revision.clone()
            )
        )
    })
}

/// `GET /admin/users/{ref}`: the account `detail` names, rendered again when `nonce` or
/// `revision` change.
#[shard]
async fn user_detail(
    cx: &Cx,
    detail: Signal<String>,
    nonce: Signal<u32>,
    revision: Signal<u32>,
) -> Result<impl View> {
    let _ = nonce.get();
    let _ = revision.get();
    let loaded = match detail.get().parse::<Uuid>() {
        Ok(id) => {
            let path = format!("/admin/users/{}", api::enc(&format!("id:{id}")));
            Some(api::get::<AdminUserDetailDto>(cx, &path).await?)
        }
        Err(_) => None,
    };
    Ok(view! {
        if let Some(d) = &loaded {
            user_detail_body(d: d, revision: &revision)
        }
    })
}

/// Everything about the account of `d`.
#[component]
async fn user_detail_body(
    cx: &Cx,
    d: &AdminUserDetailDto,
    revision: &Signal<u32>,
) -> Result<impl View> {
    let user = &d.user.user;
    let handle = handle_of(&user.handle, user.domain.as_deref());
    let status = if user.suspended {
        tr!(cx, admin_tag_suspended)
    } else {
        tr!(cx, admin_tag_active)
    };
    let role = (d.user.role != tuita_server::dto::Role::User).then(|| d.user.role.as_str());
    let org = user.org.then(|| tr!(cx, admin_tag_org));
    let invite = d
        .invited_with
        .as_ref()
        .map(|code| tr!(cx, admin_detail_invite, code: code.as_str()));
    let number = |n: i32| compact(cx, i64::from(n));
    let stats = vec![
        Stat {
            name: tr!(cx, admin_detail_posts),
            count: number(user.counts.posts),
        },
        Stat {
            name: tr!(cx, admin_detail_followers),
            count: number(user.counts.followers),
        },
        Stat {
            name: tr!(cx, admin_detail_following),
            count: number(d.following),
        },
        Stat {
            name: tr!(cx, admin_detail_open_reports),
            count: number(d.user.open_reports),
        },
        Stat {
            name: tr!(cx, admin_detail_sessions),
            count: number(d.sessions),
        },
        Stat {
            name: tr!(cx, admin_detail_passkeys),
            count: number(d.passkeys),
        },
        Stat {
            name: tr!(cx, admin_detail_tokens),
            count: number(d.tokens),
        },
    ];
    let mut when = vec![tr!(cx, admin_detail_joined, time: full_time(cx, &stamp(user.created_at)))];
    if let Some(at) = d.user.last_active_at {
        when.push(tr!(cx, admin_user_active, time: rel_label(cx, &stamp(at))));
    }
    let when = when.join(" · ");
    let unknown = tr!(cx, admin_unknown);
    let system = tr!(cx, admin_audit_system);
    let open = tr!(cx, admin_report_open);
    let resolved = tr!(cx, admin_report_resolved);
    let reports: Vec<(String, String, String)> = d
        .reports
        .iter()
        .map(|report| {
            let state = match (report.status, report.resolution) {
                (ReportStatus::Open, _) => open.clone(),
                (_, Some(done)) => resolution_label(cx, done),
                (_, None) => resolved.clone(),
            };
            let by = tr!(
                cx,
                admin_report_by,
                handle: report
                    .reporter
                    .as_ref()
                    .map_or_else(|| unknown.clone(), |who| format!("@{who}"))
            );
            let mut line = format!("{by} · {}", rel_label(cx, &stamp(report.created_at)));
            if !report.note.is_empty() {
                line.push_str(&format!(" · “{}”", report.note));
            }
            (reason_label(cx, report.reason), state, line)
        })
        .collect();
    let history: Vec<(String, String, String, String)> = d
        .history
        .iter()
        .map(|row| {
            let who = row
                .actor
                .as_ref()
                .map_or_else(|| system.clone(), |actor| format!("@{actor}"));
            let mut line = format!("{who} · {}", rel_label(cx, &stamp(row.created_at)));
            if !row.detail.is_empty() {
                line.push_str(&format!(" · {}", row.detail));
            }
            (
                row.action.clone(),
                audit_label(cx, &row.action),
                row.target.clone(),
                line,
            )
        })
        .collect();
    let heading_posts = tr!(cx, admin_detail_recent_posts);
    let heading_reports = tr!(cx, admin_detail_reports);
    let heading_history = tr!(cx, admin_detail_history);
    let none_posts = tr!(cx, admin_detail_no_posts);
    let none_reports = tr!(cx, admin_detail_no_reports);
    let none_history = tr!(cx, admin_detail_nothing);
    Ok(view! {
        <div class="detail" data-c="admin-user-detail">
            <header class="head">
                <span aria-hidden="true">
                    account_avatar(
                        handle: &user.handle,
                        src: user.avatar_url.as_deref(),
                        shape: user.avatar_shape,
                        ink: user.ink,
                        size: 64
                    )
                </span>
                <div class="who">
                    user_name(
                        name: &user.display_name,
                        verified: user.verified,
                        bot: user.bot
                    )
                    <span class="dim">(&handle)</span>
                    <span class="tags">
                        tag(pill: true, compact: true, (&status))
                        if let Some(role) = role {
                            tag(pill: true, compact: true, (role))
                        }
                        if let Some(text) = &org {
                            tag(pill: true, compact: true, (text))
                        }
                        if let Some(text) = &invite {
                            tag(pill: true, compact: true, (text))
                        }
                    </span>
                </div>
                user_menu(target: &d.user, scope: "detail", revision: revision)
            </header>
            if !user.bio.is_empty() {
                <p class="bio">(&user.bio)</p>
            }
            <dl class="stats">
                for stat in stats {
                    <div>
                        <dt>(&stat.name)</dt>
                        <dd>(&stat.count)</dd>
                    </div>
                }
            </dl>
            <p class="dim">(&when)</p>
            <section>
                <h3>(&heading_posts)</h3>
                if d.posts.is_empty() {
                    <p class="dim">(&none_posts)</p>
                }
                #[key(post.id.as_bytes())]
                for post in &d.posts {
                    post_preview(post: post)
                }
            </section>
            <section>
                <h3>(&heading_reports)</h3>
                if reports.is_empty() {
                    <p class="dim">(&none_reports)</p>
                }
                for (reason, state, line) in reports {
                    <p class="item">
                        <span>
                            (&reason)
                            " "
                            tag((&state))
                        </span>
                        <span class="dim">(&line)</span>
                    </p>
                }
            </section>
            <section>
                <h3>(&heading_history)</h3>
                if history.is_empty() {
                    <p class="dim">(&none_history)</p>
                }
                for (action, label, target, line) in history {
                    <p class="item">
                        <span title=(&action)>
                            (&label)
                            " "
                            (&target)
                        </span>
                        <span class="dim">(&line)</span>
                    </p>
                }
            </section>
        </div>
    })
}
