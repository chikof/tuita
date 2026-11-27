//! The overview tab: the numbers and charts of the instance.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_server::dto::{AdminStatsDto, DayStatsDto};

use crate::{
    admin::common::tr,
    api, fmt,
    ui::{
        bar_chart::bar_chart,
        bento::bento,
        card::card,
        chart::{ChartPoint, day_label, weekday_labels},
        dither_field::{FieldKind, dither_field},
        heatmap::heatmap,
        number_format::NumberFormat,
        sparkline::sparkline,
        stat_card::stat_card,
    },
};

/// One point per day of `pick`, labelled with the day.
pub(crate) fn series_of<'a>(
    days: &[DayStatsDto],
    labels: &'a [String],
    pick: fn(&DayStatsDto) -> i32,
) -> Vec<ChartPoint<'a>> {
    days.iter()
        .zip(labels)
        .map(|(day, label)| ChartPoint {
            label,
            value: u64::try_from(pick(day)).unwrap_or(0),
        })
        .collect()
}

/// The grid of numbers and charts for `stats`.
#[component]
pub(crate) async fn admin_bento(cx: &Cx, stats: &AdminStatsDto) -> Result<impl View> {
    let days = &stats.days;
    let labels: Vec<String> = days.iter().map(|d| day_label(cx, &d.day)).collect();
    let count = |n: i32| u64::try_from(n).unwrap_or(0);
    let compact = |n: i32| fmt::compact(cx, i64::from(n));
    let today = days.last().map_or(0, |d| d.signups);
    let signups_sum: i32 = days.iter().map(|d| d.signups).sum();
    let people = tr!(cx, admin_stat_people);
    let people_sub = tr!(cx, admin_stat_people_sub, count: compact(stats.users.active7d));
    let posts_today = tr!(cx, admin_stat_posts_today);
    let posts_total = tr!(cx, admin_stat_posts_total, count: compact(stats.posts.total));
    let reports = tr!(cx, admin_stat_reports);
    let reports_sub = if stats.reports_open > 0 {
        tr!(cx, admin_stat_reports_open)
    } else {
        tr!(cx, admin_stat_reports_clear)
    };
    let signups_today = tr!(cx, admin_stat_signups_today);
    let signups_sub = tr!(cx, admin_stat_signups_sub, count: compact(signups_sum));
    let suspended = tr!(cx, admin_stat_suspended);
    let suspended_sub = tr!(cx, admin_stat_suspended_sub);
    let chart_posts = tr!(cx, admin_chart_posts);
    let chart_signups = tr!(cx, admin_chart_signups);
    let chart_reports = tr!(cx, admin_chart_reports);
    let chart_heat = tr!(cx, admin_chart_heat);
    let heat: Vec<Vec<u64>> = stats
        .heatmap
        .iter()
        .map(|row| row.iter().map(|n| count(*n)).collect())
        .collect();
    Ok(view! {
        bento(
            stat_card(
                hero: true,
                span: 2,
                rows: 2,
                entrance: 0u32,
                label: &people,
                value: count(stats.users.local),
                format: NumberFormat::Compact,
                sub: people_sub.as_str(),
                field: Some(
                    view! { dither_field(field: FieldKind::Waves, opacity: 0.4) }.into(),
                )
            )
            stat_card(
                entrance: 1u32,
                label: &posts_today,
                value: count(stats.posts.last24h),
                format: NumberFormat::Compact,
                sub: posts_total.as_str()
            )
            stat_card(
                entrance: 2u32,
                href: "/admin/reports",
                label: &reports,
                value: count(stats.reports_open),
                format: NumberFormat::Compact,
                sub: reports_sub.as_str()
            )
            stat_card(
                entrance: 3u32,
                label: &signups_today,
                value: count(today),
                format: NumberFormat::Compact,
                sub: signups_sub.as_str()
            )
            stat_card(
                entrance: 4u32,
                label: &suspended,
                value: count(stats.users.suspended),
                format: NumberFormat::Compact,
                sub: suspended_sub.as_str()
            )
            card(
                span: 2,
                rows: 2,
                entrance: 5u32,
                bar_chart(
                    title: &chart_posts,
                    bars: series_of(days, &labels, |d| d.posts),
                    format: NumberFormat::Compact
                )
            )
            card(
                span: 2,
                entrance: 6u32,
                tight: true,
                sparkline(
                    title: &chart_signups,
                    points: series_of(days, &labels, |d| d.signups),
                    format: NumberFormat::Compact
                )
            )
            card(
                span: 2,
                entrance: 7u32,
                tight: true,
                sparkline(
                    title: &chart_reports,
                    points: series_of(days, &labels, |d| d.reports),
                    format: NumberFormat::Compact
                )
            )
            card(
                span: 4,
                rows: 2,
                entrance: 8u32,
                heatmap(
                    title: &chart_heat,
                    rows: heat,
                    row_labels: weekday_labels(cx),
                    format: NumberFormat::Compact
                )
            )
        )
    })
}

/// `GET /admin/stats`.
#[component]
pub(crate) async fn overview(cx: &Cx) -> Result<impl View> {
    let stats = api::get::<AdminStatsDto>(cx, "/admin/stats").await?;
    Ok(view! {
        <div class="tab" data-c="admin-overview">admin_bento(stats: &stats)</div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;
    use tuita_server::dto::{StatsPostsDto, StatsUsersDto};

    use super::*;
    use crate::testing::render;

    fn stats() -> AdminStatsDto {
        AdminStatsDto {
            users: StatsUsersDto {
                local: 1200,
                demo: 0,
                suspended: 3,
                active7d: 40,
            },
            posts: StatsPostsDto {
                total: 5000,
                last24h: 12,
            },
            reports_open: 2,
            days: (1..=14)
                .map(|n| DayStatsDto {
                    day: format!("2026-11-{n:02}"),
                    signups: n,
                    posts: n * 2,
                    likes: 0,
                    reports: i32::from(n % 3 == 0),
                })
                .collect(),
            heatmap: vec![vec![1; 24]; 7],
        }
    }

    #[test]
    fn the_numbers_and_the_charts_of_the_instance_are_cards() {
        let stats = stats();
        let html = crate::l10n::strip_isolates(&render("en", |cx| {
            view! { admin_bento(stats: &stats) }.boxed()
        }));
        assert!(html.contains("People"), "{html}");
        assert!(html.contains("40 active this week"), "{html}");
        assert!(html.contains(r#"href="/admin/reports""#), "{html}");
        assert!(html.contains("Needs review"), "{html}");
        assert!(html.contains("Posts per day"), "{html}");
        assert!(html.contains("Sign-ups per day"), "{html}");
    }

    #[test]
    fn no_open_reports_reads_all_clear() {
        let mut stats = stats();
        stats.reports_open = 0;
        let html = render("en", |cx| view! { admin_bento(stats: &stats) }.boxed());
        assert!(html.contains("All clear"), "{html}");
    }
}
