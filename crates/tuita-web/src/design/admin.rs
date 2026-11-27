//! Gallery sections: admin.
//!
//! The gallery plays an admin: the menus of the user rows take `viewer` explicitly, so it works
//! for a guest too. The rows, cards and summaries take the signals the console's lists share; no
//! list here re-renders, so one `revision` serves them all.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, signal},
    view::{Child, View, attributes, class, component, view},
};
use tuita_server::dto::{AdminUserDto, MeDto, Role, UserDto};

use super::{
    sample,
    section::{row, section},
};
use crate::{
    admin::{
        actor_key::actor_key,
        domain_block_row::domain_block_row,
        overview::{admin_bento, series_of},
        post_preview::post_preview,
        queue_summary::queue_summary,
        report_card::report_card,
        rules::{AdminTab, tabs_for},
        tabs::admin_tabs,
        user_row::admin_user_row,
    },
    settings::token_secret::token_secret,
    shell::announcement_banner::announcement_banner_view,
    ui::{
        bar_chart::bar_chart,
        bento::bento,
        button::{ButtonVariant, button},
        card::card,
        chart::{ChartPoint, day_label, weekday_labels},
        dither_field::{FieldKind, dither_field},
        heatmap::heatmap,
        number_format::NumberFormat,
        sparkline::sparkline,
        stat_card::stat_card,
    },
};

/// The id of the token dialog: `token-secret` is the section's.
const TOKEN_DIALOG: &str = "design-admin-token-secret";

/// The token the demo shows.
const TOKEN: &str = "tuita_exampleexampleexampleexampleexampleexample";

/// A bordered box that clips its rows (`.frame`); `pad` pads it.
#[component]
pub(crate) async fn frame(
    cx: &Cx,
    #[default] pad: bool,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <div class=(class!("frame", "pad" if pad)) data-c="design-admin-frame">
            (child)
        </div>
    })
}

/// The cells of a heatmap as counts.
fn counts(rows: &[Vec<i32>]) -> Vec<Vec<u64>> {
    rows.iter()
        .map(|cells| {
            cells
                .iter()
                .map(|n| u64::try_from(*n).unwrap_or(0))
                .collect()
        })
        .collect()
}

/// A user row's data: `user` patched by `over`.
fn user_entry(
    base: AdminUserDto,
    role: Role,
    open_reports: i32,
    patch: impl FnOnce(&mut UserDto),
) -> AdminUserDto {
    let mut entry = AdminUserDto {
        role,
        open_reports,
        ..base
    };
    patch(&mut entry.user);
    entry
}

/// `stat-card` to `admin-bento`.
#[component]
async fn charts(cx: &Cx) -> Result<impl View> {
    let stats = sample::stats();
    let quiet = sample::quiet_stats();
    let labels: Vec<String> = stats.days.iter().map(|d| day_label(cx, &d.day)).collect();
    let quiet_labels: Vec<String> = quiet.days.iter().map(|d| day_label(cx, &d.day)).collect();
    Ok(view! {
        section(
            id: "stat-card",
            title: "StatCard",
            note: "mono label, rolling pixel number, one line of context; a link when it has an href",
            bento(
                stat_card(
                    label: "People",
                    value: 12480,
                    format: NumberFormat::Compact,
                    sub: "3.1K active this week"
                )
                stat_card(
                    label: "Open reports",
                    value: 3,
                    href: "/design#stat-card",
                    sub: "Needs review",
                    format: NumberFormat::Compact
                )
                stat_card(label: "Suspended", value: 0, sub: "accounts")
                stat_card(
                    hero: true,
                    span: 2,
                    rows: 2,
                    label: "Hero",
                    value: 183204,
                    format: NumberFormat::Compact,
                    sub: "text sits on a solid panel",
                    field: Some(
                        view! {
                            dither_field(
                                field: FieldKind::Ripples,
                                seed: "7",
                                opacity: 0.4
                            )
                        }.into(

                        ),
                    )
                )
            )
        )
        section(
            id: "bar-chart",
            title: "BarChart",
            note: "14 days, bars alternate dither level 1 and 0.5 with solid caps; summary text and a data table back it",
            bento(
                card(
                    span: 2,
                    rows: 2,
                    bar_chart(
                        title: "Posts per day",
                        bars: series_of(&stats.days, &labels, |d| d.posts),
                        format: NumberFormat::Compact
                    )
                )
                card(
                    span: 2,
                    rows: 2,
                    bar_chart(
                        title: "Reports per day",
                        bars: series_of(&quiet.days, &quiet_labels, |d| d.posts)
                    )
                )
            )
        )
        section(
            id: "sparkline",
            title: "Sparkline",
            note: "a line over a dithered area with a solid end marker; the table lists every point",
            bento(
                card(
                    span: 2,
                    tight: true,
                    sparkline(
                        title: "Sign-ups per day",
                        points: series_of(&stats.days, &labels, |d| d.signups),
                        format: NumberFormat::Compact
                    )
                )
                card(
                    span: 2,
                    tight: true,
                    sparkline(
                        title: "Likes per day",
                        points: vec![ChartPoint { label: labels[0].as_str(), value: 4 }]
                    )
                )
            )
        )
        section(
            id: "heatmap",
            title: "Heatmap",
            note: "weekday by hour in five Bayer steps; the legend and the table carry the values",
            bento(
                card(
                    span: 4,
                    rows: 2,
                    heatmap(
                        title: "Activity by weekday and hour",
                        rows: counts(&stats.heatmap),
                        row_labels: weekday_labels(cx),
                        format: NumberFormat::Compact
                    )
                )
                card(
                    span: 4,
                    rows: 2,
                    heatmap(
                        title: "A quiet instance",
                        rows: counts(&quiet.heatmap),
                        row_labels: weekday_labels(cx)
                    )
                )
            )
        )
        section(
            id: "admin-bento",
            title: "AdminBento",
            note: "the admin overview: animated hero field, counters, charts and the heatmap",
            admin_bento(stats: &stats)
        )
    })
}

/// `announcement-banner` to `token-secret`.
#[component]
async fn moderation(cx: &Cx) -> Result<impl View> {
    let revision = signal(cx, || 0u32);
    let detail = signal(cx, String::new);
    let nonce = signal(cx, || 0u32);
    let secret = signal(cx, String::new);
    let viewer = MeDto {
        role: Role::Admin,
        ..sample::sample_me()
    };
    let banners = sample::announcements();
    let (open, account, resolved) = (
        sample::report_open(),
        sample::report_account(),
        sample::report_resolved(),
    );
    let warned = sample::warned();
    let admin_tabs_all = tabs_for(Role::Admin);
    let moderator_tabs = tabs_for(Role::Moderator);
    let ada = user_entry(sample::admin_user(&sample::ada()), Role::User, 2, |user| {
        user.verified = true
    });
    let bob = user_entry(
        sample::admin_user(&sample::bob()),
        Role::Moderator,
        0,
        |user| {
            user.suspended = true;
            user.org = true;
        },
    );
    Ok(view! {
        section(
            id: "announcement-banner",
            title: "AnnouncementBanner",
            note: "tones info, success and warning; dismissal is remembered in localStorage tuita:ann:id",
            frame(announcement_banner_view(items: &banners))
            row(
                button(
                    variant: ButtonVariant::Outline,
                    attrs: attributes! { data-design-act="reset-banners" },
                    "Show the banners again"
                )
            )
        )
        section(
            id: "admin-tabs",
            title: "AdminTabs",
            note: "links, not tabs: each tab is a page; the strip scrolls on a narrow screen; moderators get fewer",
            frame(admin_tabs(tabs: &admin_tabs_all, current: AdminTab::Reports))
            frame(admin_tabs(tabs: &moderator_tabs, current: AdminTab::Overview))
        )
        section(
            id: "report-card",
            title: "ReportCard",
            note: "open reports offer dismiss, delete post and suspend; resolved ones say who decided",
            frame(
                report_card(report: &open, revision: &revision)
                <hr>
                report_card(report: &account, revision: &revision)
                <hr>
                report_card(report: &resolved, revision: &revision)
            )
        )
        section(
            id: "admin-user-row",
            title: "AdminUserRow",
            note: "a person with roles, state and counts; the menu holds the actions the viewer may use",
            frame(
                admin_user_row(
                    entry: &ada,
                    scope: "design-admin",
                    revision: &revision,
                    detail: &detail,
                    nonce: &nonce,
                    viewer: Some(&viewer)
                )
                <hr>
                admin_user_row(
                    entry: &bob,
                    scope: "design-admin",
                    revision: &revision,
                    detail: &detail,
                    nonce: &nonce,
                    note: Some("The name contains profanity"),
                    viewer: Some(&viewer)
                )
            )
        )
        section(
            id: "post-preview",
            title: "PostPreview",
            note: "the reported post as staff see it",
            frame(pad: true, post_preview(post: &warned))
        )
        section(
            id: "token-secret",
            title: "TokenSecret",
            note: "a new API token appears once, in a copy box with a warning; closing the dialog forgets it",
            row(
                button(
                    variant: ButtonVariant::Outline,
                    attrs: attributes! {
                        command="show-modal"
                        commandfor=(TOKEN_DIALOG)
                        @click=$(|_e: Event| secret.set(TOKEN.to_owned()))
                    },
                    "Show a new token"
                )
            )
            token_secret(secret: &secret, id: TOKEN_DIALOG)
        )
    })
}

/// `queue-summary` to `domain-block-row`.
#[component]
async fn federation_admin(cx: &Cx) -> Result<impl View> {
    let revision = signal(cx, || 0u32);
    let (busy, empty) = (sample::queue_busy(), sample::queue_empty());
    let actor = sample::federation_actor();
    let (silence, suspend) = (
        sample::domain_block_silence(),
        sample::domain_block_suspend(),
    );
    Ok(view! {
        section(
            id: "queue-summary",
            title: "QueueSummary",
            note: "outgoing deliveries: counts, the oldest, Retry now, and the instances that keep failing with their last error",
            frame(pad: true, queue_summary(queue: &busy, revision: &revision))
            frame(pad: true, queue_summary(queue: &empty, revision: &revision))
        )
        section(
            id: "actor-key",
            title: "ActorKey",
            note: "the instance actor and its key; the SHA-256 fingerprint is mono, grouped by eight, copyable",
            frame(pad: true, actor_key(actor: &actor))
        )
        section(
            id: "domain-block-row",
            title: "DomainBlockRow",
            note: "a blocked instance: reason, known accounts, severity select (silence or suspend) and a confirmed remove",
            frame(
                pad: true,
                domain_block_row(block: &silence, revision: &revision)
                domain_block_row(block: &suspend, revision: &revision)
            )
        )
    })
}

#[component]
pub(crate) async fn admin(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        charts()
        moderation()
        federation_admin()
    })
}
