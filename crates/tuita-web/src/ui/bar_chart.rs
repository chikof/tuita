//! A bar chart of dithered columns.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::{
    l10n::t,
    ui::{
        chart::{ChartPoint, nice_max, share},
        chart_frame::chart_frame,
        number_format::NumberFormat,
    },
};

/// `i` gets a label under it: every third bar, ending with the last one, each three bars wide.
fn labelled(i: usize, len: usize) -> bool {
    i >= 2 && (len - 1 - i).is_multiple_of(3)
}

/// A chart of one bar per entry (oldest first) on a 0 to a round maximum axis, every other bar
/// dithered. `format` is how a value reads on the axis and in the summary. The chart is
/// `aria-hidden`; its frame carries the summary sentence and the table of values.
#[component]
pub(crate) async fn bar_chart(
    cx: &Cx,
    title: &str,
    bars: Vec<ChartPoint<'_>>,
    #[default] format: NumberFormat,
) -> Result<impl View> {
    let values: Vec<f64> = bars.iter().map(|b| b.value as f64).collect();
    let top = nice_max(&values);
    let total: u64 = bars.iter().map(|b| b.value).sum();
    let peak = bars
        .iter()
        .fold(None::<&ChartPoint<'_>>, |best, bar| match best {
            Some(best) if bar.value <= best.value => Some(best),
            _ => Some(bar),
        });
    let summary = match peak {
        Some(peak) if total > 0 => t!(
            chart_bar_summary,
            title: title,
            total: format.apply(cx, total),
            peak: format.apply(cx, peak.value),
            label: peak.label,
        )
        .resolve(cx)
        .into_owned(),
        _ => t!(chart_empty, title: title).resolve(cx).into_owned(),
    };
    let meta = format.apply(cx, total);
    let top_text = format.apply(cx, top as u64);
    let half_text = format.apply(cx, (top / 2.0) as u64);
    let len = bars.len();
    let table_bars = bars.clone();
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
                    for bar in &table_bars {
                        <tr>
                            <td>(bar.label)</td>
                            <td>(bar.value)</td>
                        </tr>
                    }
                </tbody>
            }.into(

            ),
            <div class="chart" style=(format!("--n:{len}")) data-c="bar-chart">
                <div class="axis" aria-hidden="true">
                    <span>(&top_text)</span>
                    <span>(&half_text)</span>
                    <span>"0"</span>
                </div>
                <div class="bars" aria-hidden="true">
                    for (i, bar) in bars.iter().enumerate() {
                        <div class="slot">
                            <div
                                class="bar"
                                style=(format!("height:{}%", share(bar.value as f64, top)))
                            >
                                <i
                                    class="fill"
                                    style=((i % 2 == 0).then_some(
                                        "mask-image:var(--l2);-webkit-mask-image:var(--l2)",
                                    ))
                                ></i>
                                <i class="cap"></i>
                            </div>
                        </div>
                    }
                </div>
                <div class="ticks" aria-hidden="true">
                    for (i, bar) in bars.iter().enumerate() {
                        if labelled(i, len) {
                            <span style=(format!("grid-column:{} / span 3", i - 1))>
                                (bar.label)
                            </span>
                        }
                    }
                </div>
            </div>
        )
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::{l10n::strip_isolates, testing::render};

    fn week() -> Vec<ChartPoint<'static>> {
        [
            ("Nov 1", 2),
            ("Nov 2", 9),
            ("Nov 3", 5),
            ("Nov 4", 0),
            ("Nov 5", 3),
            ("Nov 6", 1),
        ]
        .into_iter()
        .map(|(label, value)| ChartPoint { label, value })
        .collect()
    }

    #[test]
    fn it_summarises_the_total_and_the_peak_and_lists_every_value_in_the_table() {
        let html = render("en", |cx| {
            view! { bar_chart(title: "Posts", bars: week()) }.boxed()
        });
        let text = strip_isolates(&html);
        assert!(
            text.contains("Posts: 20 in total, most on Nov 2 (9)"),
            "{text}"
        );
        assert!(text.contains(r#"<span class="meta">20</span>"#), "{text}");
        assert!(
            text.contains(r#"<tr><td>Nov 2</td><td>9</td></tr>"#),
            "{text}"
        );
        assert!(
            text.contains(r#"<th scope="col">Day</th><th scope="col">Posts</th>"#),
            "{text}"
        );
    }

    #[test]
    fn the_axis_tops_out_at_a_round_maximum_and_bars_are_a_share_of_it() {
        let html = render("en", |cx| {
            view! { bar_chart(title: "Posts", bars: week()) }.boxed()
        });
        assert!(
            html.contains("<span>12</span><span>6</span><span>0</span>"),
            "{html}"
        );
        assert!(html.contains("height:75%"), "{html}");
        assert!(html.contains("height:0%"), "{html}");
        assert_eq!(html.matches(r#"class="slot""#).count(), 6, "{html}");
        assert!(html.contains("--n:6"), "{html}");
    }

    #[test]
    fn every_third_bar_gets_a_label_ending_with_the_last() {
        let html = render("en", |cx| {
            view! { bar_chart(title: "Posts", bars: week()) }.boxed()
        });
        assert!(
            html.contains(r#"<span style="grid-column:1 / span 3">Nov 3</span>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<span style="grid-column:4 / span 3">Nov 6</span>"#),
            "{html}"
        );
        assert!(!labelled(0, 6) && !labelled(1, 6) && labelled(2, 6) && labelled(5, 6));
    }

    #[test]
    fn no_data_says_so() {
        let html = render("en", |cx| {
            view! { bar_chart(title: "Posts", bars: Vec::new()) }.boxed()
        });
        assert!(
            strip_isolates(&html).contains("Posts: nothing to show yet."),
            "{html}"
        );
    }
}
