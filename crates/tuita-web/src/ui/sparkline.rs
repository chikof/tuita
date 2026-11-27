//! A line over dithered area.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::{
    l10n::t,
    ui::{
        chart::{ChartPoint, area_clip, polyline_of, spark_points},
        chart_frame::chart_frame,
        number_format::NumberFormat,
    },
};

/// A line chart of one point per entry (oldest first) with the area under it dithered and a dot
/// at the end. The chart is `aria-hidden`; its frame carries the summary sentence and the table.
#[component]
pub(crate) async fn sparkline(
    cx: &Cx,
    title: &str,
    points: Vec<ChartPoint<'_>>,
    #[default] format: NumberFormat,
) -> Result<impl View> {
    let values: Vec<f64> = points.iter().map(|p| p.value as f64).collect();
    let geometry = spark_points(&values, None);
    let total: u64 = points.iter().map(|p| p.value).sum();
    let summary = match points.last() {
        Some(last) if total > 0 => t!(
            chart_spark_summary,
            title: title,
            total: format.apply(cx, total),
            last: format.apply(cx, last.value),
            label: last.label,
        )
        .resolve(cx)
        .into_owned(),
        _ => t!(chart_empty, title: title).resolve(cx).into_owned(),
    };
    let meta = format.apply(cx, total);
    let end = geometry.last().copied();
    let table_points = points.clone();
    Ok(view! {
        chart_frame(
            title: title,
            meta: meta.as_str(),
            summary: &summary,
            table: view! {
                <thead>
                    <tr>
                        <th scope="col">(t!(chart_col_day))</th>
                        <th scope="col">(title)</th>
                    </tr>
                </thead>
                <tbody>
                    for point in &table_points {
                        <tr>
                            <td>(point.label)</td>
                            <td>(point.value)</td>
                        </tr>
                    }
                </tbody>
            }.into(

            ),
            <div class="spark" aria-hidden="true" data-c="sparkline">
                <i
                    class="area"
                    style=(format!(
                        "clip-path:{};mask-image:var(--l2);-webkit-mask-image:var(--l2)",
                        area_clip(&geometry),
                    ))
                ></i>
                <svg viewBox="0 0 100 100" preserveAspectRatio="none">
                    <polyline points=(polyline_of(&geometry))></polyline>
                </svg>
                if let Some(end) = end {
                    <i class="end" style=(format!("left:{}%;top:{}%", end.x, end.y))></i>
                }
            </div>
        )
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::{l10n::strip_isolates, testing::render};

    fn days() -> Vec<ChartPoint<'static>> {
        [("Nov 1", 0), ("Nov 2", 5), ("Nov 3", 10)]
            .into_iter()
            .map(|(label, value)| ChartPoint { label, value })
            .collect()
    }

    #[test]
    fn it_draws_the_line_the_area_and_the_end_dot_and_summarises_the_last_point() {
        let html = render("en", |cx| {
            view! { sparkline(title: "Sign-ups", points: days()) }.boxed()
        });
        assert!(html.contains(r#"points="0,100 50,50 100,0""#), "{html}");
        assert!(
            html.contains("clip-path:polygon(0% 100%, 0% 100%, 50% 50%, 100% 0%, 100% 100%)"),
            "{html}"
        );
        assert!(
            html.contains(r#"<i class="end" style="left:100%;top:0%">"#),
            "{html}"
        );
        assert!(
            strip_isolates(&html).contains("Sign-ups: 15 in total, 10 on Nov 3."),
            "{html}"
        );
        assert!(html.contains("<tr><td>Nov 2</td><td>5</td></tr>"), "{html}");
    }

    #[test]
    fn no_points_say_nothing_to_show_and_draw_no_end_dot() {
        let html = render("en", |cx| {
            view! { sparkline(title: "Sign-ups", points: Vec::new()) }.boxed()
        });
        assert!(
            strip_isolates(&html).contains("Sign-ups: nothing to show yet."),
            "{html}"
        );
        assert!(!html.contains(r#"class="end""#), "{html}");
    }
}
