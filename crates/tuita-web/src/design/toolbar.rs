//! The appearance toolbar.
//!
//! Every segment is a `button[data-pref][data-value]`, so `prefs.js` stores the choice in
//! `localStorage['tuita:prefs']`, applies it and marks the stored one `aria-selected`.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::ui::brand::brand;

/// One preference: a mono label over a tablist of segments `(value, text)`.
#[component]
async fn control(
    cx: &Cx,
    label: &str,
    pref: &str,
    items: Vec<(String, String)>,
) -> Result<impl View> {
    Ok(view! {
        <div class="control" data-c="design-control">
            <span class="mono-tag">(label)</span>
            <div
                class="segmented"
                role="tablist"
                aria-label=(label)
                tabindex="-1"
                data-roving="tablist"
                data-c="segmented"
            >
                for (value, text) in items {
                    <button
                        type="button"
                        role="tab"
                        class="segment"
                        aria-selected="false"
                        data-pref=(pref)
                        data-value=(&value)
                    >
                        <span class="pill" aria-hidden="true"></span>
                        <span class="text">(text)</span>
                    </button>
                }
            </div>
        </div>
    })
}

fn items<const N: usize>(pairs: [(&str, &str); N]) -> Vec<(String, String)> {
    pairs
        .into_iter()
        .map(|(value, text)| (value.to_owned(), text.to_owned()))
        .collect()
}

#[component]
pub(crate) async fn toolbar(cx: &Cx) -> Result<impl View> {
    let inks: Vec<(String, String)> = ["bone", "phosphor", "amber", "cyan", "rose"]
        .into_iter()
        .map(|id| (id.to_owned(), id.to_owned()))
        .collect();
    let sizes: Vec<(String, String)> = ["0.9", "1", "1.1", "1.25"]
        .into_iter()
        .map(|id| {
            let percent = (id.parse::<f64>().unwrap_or(1.0) * 100.0).round();
            (id.to_owned(), format!("{percent}%"))
        })
        .collect();
    Ok(view! {
        <div
            class="toolbar"
            role="toolbar"
            aria-label="Appearance"
            data-c="design-toolbar"
        >
            brand(size: 28)
            control(
                label: "Theme",
                pref: "mode",
                items: items(
                    [("system", "System"), ("light", "Light"), ("dark", "Dark")],
                )
            )
            control(label: "Ink", pref: "ink", items: inks)
            control(
                label: "Contrast",
                pref: "contrast",
                items: items([("normal", "Normal"), ("more", "More")])
            )
            control(label: "Text size", pref: "textScale", items: sizes)
            control(
                label: "Motion",
                pref: "motion",
                items: items([("system", "System"), ("reduce", "Reduce")])
            )
        </div>
    })
}
