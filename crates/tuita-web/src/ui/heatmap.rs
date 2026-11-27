//! A weekday by hour heatmap of dithered cells.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::{
    l10n::t,
    ui::{
        chart::{HEAT_LEVELS, grid_max, grid_total, heat_bucket, hour_label},
        chart_frame::chart_frame,
        number_format::NumberFormat,
    },
};

/// The `mask-image` of a heat level: the page-wide dither mask of that share, none for the empty
/// and the solid levels.
fn mask_style(level: usize) -> Option<String> {
    let share = HEAT_LEVELS[level];
    (share > 0.0 && share < 1.0)
        .then(|| format!("mask-image:var(--l{level});-webkit-mask-image:var(--l{level})"))
}

/// A grid of seven rows (`rows[weekday][hour]`, 24 counts each, labelled by `row_labels`) in five
/// dither levels of the busiest cell. The grid is `aria-hidden`; its frame carries the summary
/// sentence and the table.
#[component]
pub(crate) async fn heatmap(
    cx: &Cx,
    title: &str,
    rows: Vec<Vec<u64>>,
    row_labels: Vec<String>,
    #[default] format: NumberFormat,
) -> Result<impl View> {
    let grid: Vec<Vec<f64>> = rows
        .iter()
        .map(|row| row.iter().map(|v| *v as f64).collect())
        .collect();
    let max = grid_max(&grid);
    let total: u64 = rows.iter().flatten().sum();
    let mut busiest: Option<(usize, usize, u64)> = None;
    for (row, cells) in rows.iter().enumerate() {
        for (hour, value) in cells.iter().enumerate() {
            if *value > busiest.map_or(0, |(_, _, best)| best) {
                busiest = Some((row, hour, *value));
            }
        }
    }
    debug_assert!((grid_total(&grid) - total as f64).abs() < 0.5);
    let summary = match busiest {
        Some((row, hour, value)) => t!(
            chart_heat_summary,
            title: title,
            total: format.apply(cx, total),
            peak: format.apply(cx, value),
            label: row_labels.get(row).map_or("", String::as_str),
            hour: hour_label(hour as u32),
        )
        .resolve(cx)
        .into_owned(),
        None => t!(chart_empty, title: title).resolve(cx).into_owned(),
    };
    let meta = format.apply(cx, total);
    let table_rows = rows.clone();
    let table_labels = row_labels.clone();
    Ok(view! {
        chart_frame(
            title: title,
            meta: meta.as_str(),
            summary: &summary,
            table: view! {
                <thead>
                    <tr>
                        <th scope="col">(t!(chart_col_hour))</th>
                        for label in &table_labels {
                            <th scope="col">(label)</th>
                        }
                    </tr>
                </thead>
                <tbody>
                    for hour in 0..24usize {
                        <tr>
                            <td>(hour_label(hour as u32))</td>
                            for cells in &table_rows {
                                <td>(cells.get(hour).copied().unwrap_or(0))</td>
                            }
                        </tr>
                    }
                </tbody>
            }.into(

            ),
            <div class="grid" aria-hidden="true" data-c="heatmap">
                <span></span>
                for hour in 0..24usize {
                    <span class="hour" style=(format!("grid-column:{}", hour + 2))>
                        (if hour % 6 == 0 { hour.to_string() } else { String::new() })
                    </span>
                }
                for (row, cells) in rows.iter().enumerate() {
                    <span class="day">
                        (row_labels.get(row).map_or("", String::as_str))
                    </span>
                    for value in cells {
                        <i
                            class=(format!(
                                "cell level-{}",
                                heat_bucket(*value as f64, max),
                            ))
                            style=(mask_style(heat_bucket(*value as f64, max)))
                        ></i>
                    }
                }
            </div>
            <div class="legend" aria-hidden="true" data-c="heatmap-legend">
                <span>(t!(chart_heat_less))</span>
                for level in 0..HEAT_LEVELS.len() {
                    <i
                        class=(format!("swatch level-{level}"))
                        style=(mask_style(level))
                    ></i>
                }
                <span>(t!(chart_heat_more))</span>
            </div>
        )
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::{l10n::strip_isolates, testing::render};

    fn week() -> (Vec<Vec<u64>>, Vec<String>) {
        let mut rows = vec![vec![0; 24]; 7];
        rows[1][9] = 40;
        rows[1][10] = 10;
        rows[4][23] = 25;
        let labels = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
            .map(str::to_owned)
            .to_vec();
        (rows, labels)
    }

    #[test]
    fn it_summarises_the_busiest_cell() {
        let html = render("en", |cx| {
            let (rows, row_labels) = week();
            view! { heatmap(title: "Activity", rows: rows, row_labels: row_labels) }.boxed()
        });
        let text = strip_isolates(&html);
        assert!(
            text.contains("Activity: 75 in total, busiest on Tue at 09:00 (40)"),
            "{text}"
        );
        assert!(text.contains(r#"<span class="meta">75</span>"#), "{text}");
        assert!(
            text.contains(r#"<th scope="col">Hour (UTC)</th><th scope="col">Mon</th>"#),
            "{text}"
        );
        assert!(text.contains("<td>09:00</td>"), "{text}");
    }

    #[test]
    fn cells_step_through_levels_of_the_busiest_one() {
        let html = render("en", |cx| {
            let (rows, row_labels) = week();
            view! { heatmap(title: "Activity", rows: rows, row_labels: row_labels) }.boxed()
        });
        assert_eq!(html.matches("cell level-").count(), 7 * 24, "{html}");
        assert_eq!(html.matches("cell level-4").count(), 1, "{html}");
        assert_eq!(html.matches("cell level-3").count(), 1, "{html}");
        assert_eq!(html.matches("cell level-1").count(), 1, "{html}");
        assert_eq!(html.matches("cell level-0").count(), 7 * 24 - 3, "{html}");
        assert_eq!(html.matches("swatch level-").count(), 5, "{html}");
        assert!(html.contains("mask-image:var(--l3)"), "{html}");
        assert_eq!(html.matches(r#"class="hour""#).count(), 24, "{html}");
    }

    #[test]
    fn an_empty_grid_says_nothing_to_show() {
        let html = render("en", |cx| {
            let rows = vec![vec![0; 24]; 7];
            view! {
                heatmap(
                    title: "Activity",
                    rows: rows,
                    row_labels: vec![String::new(); 7]
                )
            }
            .boxed()
        });
        assert!(
            strip_isolates(&html).contains("Activity: nothing to show yet."),
            "{html}"
        );
    }
}
