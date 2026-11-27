//! The frame around a chart: title, total, a table view and the chart.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, attributes, component, element_id, view},
};

use crate::{
    icons::Icon,
    l10n::t,
    ui::{
        button::ButtonSize,
        dialog::dialog,
        icon_button::{IconButtonVariant, icon_button},
    },
};

/// A `<figure>` with `title` and an optional `meta` figure (the total) in its caption, a button
/// that opens the data as a table in a dialog, and the chart (`child`) as a `role="img"` named
/// `summary`: the chart in one sentence for people who cannot see it. `table` is the `<thead>` and
/// `<tbody>` of the table behind the chart.
#[component]
pub(crate) async fn chart_frame(
    cx: &Cx,
    title: &str,
    #[default]
    #[into]
    meta: Option<&str>,
    summary: &str,
    table: Child<'_>,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let dialog_id = element_id(cx);
    let show = t!(chart_show_table, title: title).resolve(cx).into_owned();
    Ok(view! {
        <figure class="frame" data-c="chart-frame">
            <figcaption>
                <span class="title">(title)</span>
                if let Some(meta) = meta {
                    <span class="meta">(meta)</span>
                }
                icon_button(
                    glyph: Icon::ChartBarBig,
                    label: &show,
                    size: ButtonSize::Sm,
                    variant: IconButtonVariant::Ghost,
                    attrs: attributes! { command="show-modal" commandfor=(&dialog_id) }
                )
            </figcaption>
            <div class="plot" role="img" aria-label=(summary)>(child)</div>
        </figure>
        dialog(
            id: &dialog_id,
            title: title,
            <div class="scroll" data-c="chart-table">
                <table>
                    <caption class="sr-only">(summary)</caption>
                    (table)
                </table>
            </div>
        )
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn it_names_the_chart_by_its_summary_and_backs_it_with_a_table_dialog() {
        let html = render("en", |cx| {
            view! {
                chart_frame(
                    title: "Posts per day",
                    meta: "42",
                    summary: "Posts per day: 42 in total.",
                    table: view! { <thead><tr><th scope="col">"Day"</th></tr></thead> }.into(

                    ),
                    <p>"plot"</p>
                )
            }
            .boxed()
        });
        assert!(html.contains("<figure class=\"frame\""), "{html}");
        assert!(
            html.contains(r#"<span class="title">Posts per day</span>"#),
            "{html}"
        );
        assert!(html.contains(r#"<span class="meta">42</span>"#), "{html}");
        assert!(
            html.contains(r#"role="img" aria-label="Posts per day: 42 in total.""#),
            "{html}"
        );
        assert!(
            crate::l10n::strip_isolates(&html)
                .contains(r#"aria-label="Show Posts per day as a table""#),
            "{html}"
        );
        assert!(html.contains("<dialog"), "{html}");
        assert!(
            html.contains(r#"<caption class="sr-only">Posts per day: 42 in total.</caption>"#),
            "{html}"
        );
        assert!(html.contains(r#"<th scope="col">Day</th>"#), "{html}");
        assert!(html.contains("<p>plot</p>"), "{html}");
    }

    #[test]
    fn the_opener_and_the_dialog_share_an_id() {
        let html = render("en", |cx| {
            view! {
                chart_frame(
                    title: "T",
                    summary: "S",
                    table: view! { <tbody></tbody> }.into(),
                    "x"
                )
            }
            .boxed()
        });
        let target = crate::testing::attr_values(&html, "commandfor");
        let opener = target.first().expect("commandfor");
        assert!(
            html.contains(&format!(r#"<dialog id="{opener}""#)),
            "{html}"
        );
        assert!(!html.contains(r#"class="meta""#), "{html}");
    }
}
