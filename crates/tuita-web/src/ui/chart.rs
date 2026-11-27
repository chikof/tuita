//! Pure geometry and bucketing behind the bar chart, sparkline and heatmap.

use time::{Date, Month, OffsetDateTime};
use topcoat::{context::Cx, l10n::locale};

use crate::fmt::{format_pattern, short_date_in};

/// One labelled value of a bar chart or sparkline, oldest first.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ChartPoint<'a> {
    pub(crate) label: &'a str,
    pub(crate) value: u64,
}

/// Top of a value axis: the largest value rounded up to a multiple of 4, never below 4.
#[must_use]
pub(crate) fn nice_max(values: &[f64]) -> f64 {
    let max = values.iter().copied().fold(0.0, f64::max);
    if max <= 4.0 {
        4.0
    } else {
        (max / 4.0).ceil() * 4.0
    }
}

/// `value` as a percentage of `top`, clamped to 0..100.
#[must_use]
pub(crate) fn share(value: f64, top: f64) -> f64 {
    if top <= 0.0 || value <= 0.0 || top.is_nan() || value.is_nan() {
        return 0.0;
    }
    (value / top * 100.0).min(100.0)
}

/// Dither levels of the five heatmap steps, from empty to solid.
pub(crate) const HEAT_LEVELS: [f32; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

/// 0 for an empty cell, else 1..=4 in equal steps of `max`.
#[must_use]
pub(crate) fn heat_bucket(value: f64, max: f64) -> usize {
    if value <= 0.0 || max <= 0.0 || value.is_nan() || max.is_nan() {
        return 0;
    }
    ((value / max * 4.0).ceil() as usize).clamp(1, 4)
}

/// The largest cell of a grid, 0 when it is empty.
#[must_use]
pub(crate) fn grid_max(rows: &[Vec<f64>]) -> f64 {
    rows.iter().flatten().copied().fold(0.0, f64::max)
}

/// Sum of every cell.
#[must_use]
pub(crate) fn grid_total(rows: &[Vec<f64>]) -> f64 {
    rows.iter().flatten().sum()
}

/// A point of a sparkline in a 100 by 100 box (y grows downward).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Point {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

/// A line over `values` in a 100 by 100 box. A single value draws a flat line. `top` fixes the
/// scale; it defaults to the largest value, at least 1.
#[must_use]
pub(crate) fn spark_points(values: &[f64], top: Option<f64>) -> Vec<Point> {
    let top = top.unwrap_or_else(|| values.iter().copied().fold(1.0, f64::max));
    let last = values.len().saturating_sub(1).max(1) as f64;
    values
        .iter()
        .enumerate()
        .map(|(i, value)| Point {
            x: if values.len() == 1 {
                0.0
            } else {
                i as f64 / last * 100.0
            },
            y: 100.0 - share(*value, top),
        })
        .collect()
}

/// `round(n * 100) / 100`.
fn round(n: f64) -> f64 {
    (n * 100.0).round() / 100.0
}

/// `points` attribute of the SVG polyline.
#[must_use]
pub(crate) fn polyline_of(points: &[Point]) -> String {
    let parts: Vec<String> = points
        .iter()
        .map(|p| format!("{},{}", round(p.x), round(p.y)))
        .collect();
    parts.join(" ")
}

/// `clip-path` that keeps the area under the line.
#[must_use]
pub(crate) fn area_clip(points: &[Point]) -> String {
    let Some(first) = points.first() else {
        return "polygon(0% 100%, 100% 100%, 100% 100%)".to_owned();
    };
    let edge = (points.len() == 1).then_some(Point {
        x: 100.0,
        y: first.y,
    });
    let line: Vec<String> = points
        .iter()
        .copied()
        .chain(edge)
        .map(|p| format!("{}% {}%", round(p.x), round(p.y)))
        .collect();
    format!("polygon(0% 100%, {}, 100% 100%)", line.join(", "))
}

/// `2026-11-03` as `Nov 3`, read as a UTC day so the label never shifts with the time zone. A
/// string that is not a day comes back unchanged.
#[must_use]
pub(crate) fn day_label(cx: &Cx, day: &str) -> String {
    let parsed = day.split('-').map(str::parse::<u32>).collect::<Vec<_>>();
    let at = match parsed.as_slice() {
        [Ok(year), Ok(month), Ok(date)] => i32::try_from(*year)
            .ok()
            .zip(Month::try_from(*month as u8).ok())
            .zip(u8::try_from(*date).ok())
            .and_then(|((year, month), date)| Date::from_calendar_date(year, month, date).ok()),
        _ => None,
    };
    at.map_or_else(
        || day.to_owned(),
        |date| {
            let noon = date.midnight().assume_utc() + time::Duration::hours(12);
            short_date_in(locale(cx), noon, false)
        },
    )
}

/// Short weekday names, Monday first. 2024-01-01 was a Monday.
#[must_use]
pub(crate) fn weekday_labels(cx: &Cx) -> Vec<String> {
    (1..=7)
        .map(|day| {
            let at: Option<OffsetDateTime> = Date::from_calendar_date(2024, Month::January, day)
                .ok()
                .map(|d| d.midnight().assume_utc());
            at.and_then(|at| format_pattern(locale(cx), "EEE", at))
                .unwrap_or_default()
        })
        .collect()
}

/// `7` as `07:00`: the hour of the UTC day a heatmap column stands for.
#[must_use]
pub(crate) fn hour_label(hour: u32) -> String {
    format!("{hour:02}:00")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::context;

    #[test]
    fn nice_max_rounds_up_to_a_multiple_of_four_at_least_four() {
        assert!((nice_max(&[]) - 4.0).abs() < f64::EPSILON);
        assert!((nice_max(&[0.0, 0.0]) - 4.0).abs() < f64::EPSILON);
        assert!((nice_max(&[3.0]) - 4.0).abs() < f64::EPSILON);
        assert!((nice_max(&[5.0, 2.0]) - 8.0).abs() < f64::EPSILON);
        assert!((nice_max(&[8.0]) - 8.0).abs() < f64::EPSILON);
        assert!((nice_max(&[41.0, 7.0]) - 44.0).abs() < f64::EPSILON);
    }

    #[test]
    fn share_is_a_clamped_percentage() {
        assert!((share(5.0, 20.0) - 25.0).abs() < f64::EPSILON);
        assert!((share(30.0, 20.0) - 100.0).abs() < f64::EPSILON);
        assert!(share(-1.0, 20.0).abs() < f64::EPSILON);
        assert!(share(3.0, 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn heat_bucket_keeps_zero_empty_and_steps_the_rest_through_four_levels() {
        let cases = [
            (0.0, 0),
            (1.0, 1),
            (10.0, 1),
            (11.0, 2),
            (20.0, 2),
            (30.0, 3),
            (31.0, 4),
            (40.0, 4),
            (99.0, 4),
        ];
        for (value, bucket) in cases {
            assert_eq!(heat_bucket(value, 40.0), bucket, "{value}");
        }
        assert_eq!(heat_bucket(0.0, 0.0), 0);
        assert_eq!(heat_bucket(5.0, 0.0), 0);
    }

    #[test]
    fn grid_totals_find_the_maximum_and_the_sum() {
        let grid = vec![vec![1.0, 2.0, 3.0], vec![0.0, 9.0, 4.0]];
        assert!((grid_max(&grid) - 9.0).abs() < f64::EPSILON);
        assert!((grid_total(&grid) - 19.0).abs() < f64::EPSILON);
        assert!(grid_max(&[]).abs() < f64::EPSILON);
        assert!(grid_total(&[vec![]]).abs() < f64::EPSILON);
    }

    #[test]
    fn spark_points_spread_values_with_larger_ones_higher_up() {
        let points = spark_points(&[0.0, 5.0, 10.0], None);
        let xy: Vec<(f64, f64)> = points.iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(xy, [(0.0, 100.0), (50.0, 50.0), (100.0, 0.0)]);
        assert_eq!(polyline_of(&points), "0,100 50,50 100,0");
    }

    #[test]
    fn spark_points_draw_flat_at_the_bottom_when_everything_is_zero() {
        let ys: Vec<f64> = spark_points(&[0.0, 0.0, 0.0], None)
            .iter()
            .map(|p| p.y)
            .collect();
        assert_eq!(ys, [100.0, 100.0, 100.0]);
    }

    #[test]
    fn spark_points_honour_a_fixed_top() {
        assert_eq!(
            spark_points(&[5.0], Some(10.0)),
            [Point { x: 0.0, y: 50.0 }]
        );
    }

    #[test]
    fn the_area_clip_keeps_the_area_under_the_line() {
        assert_eq!(
            area_clip(&spark_points(&[0.0, 10.0], None)),
            "polygon(0% 100%, 0% 100%, 100% 0%, 100% 100%)"
        );
        assert_eq!(
            area_clip(&spark_points(&[5.0], Some(10.0))),
            "polygon(0% 100%, 0% 50%, 100% 50%, 100% 100%)"
        );
        assert_eq!(area_clip(&[]), "polygon(0% 100%, 100% 100%, 100% 100%)");
    }

    #[test]
    fn a_utc_day_is_named_without_shifting() {
        let cx = context("en");
        assert_eq!(day_label(&cx, "2026-11-03"), "Nov 3");
        assert_eq!(day_label(&cx, "2026-01-01"), "Jan 1");
        assert_eq!(day_label(&cx, "not a day"), "not a day");
        assert_eq!(day_label(&context("de"), "2026-11-03"), "3. Nov.");
    }

    #[test]
    fn weekdays_start_on_monday() {
        assert_eq!(
            weekday_labels(&context("en")),
            ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
        );
    }

    #[test]
    fn hours_are_padded() {
        assert_eq!(hour_label(7), "07:00");
        assert_eq!(hour_label(23), "23:00");
    }
}
