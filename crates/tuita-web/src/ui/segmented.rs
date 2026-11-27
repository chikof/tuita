//! A segmented control.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal},
    view::{View, component, element_id, view},
};

use crate::ui::tabs::Choice;

/// A pill-shaped `role="tablist"` named `label` whose selected segment is the `value` signal (an
/// item id). A click sets the signal in the browser. Arrow-key navigation is for the shell island
/// (`[role="tablist"][data-roving]`, see [`tabs`](super::tabs::tabs)).
#[component]
pub(crate) async fn segmented(
    cx: &Cx,
    items: Vec<Choice<'_>>,
    value: &Signal<String>,
    label: &str,
) -> Result<impl View> {
    let uid = element_id(cx);
    let pairs: Vec<(&str, &str)> = items.iter().map(|item| (item.id, item.label)).collect();
    Ok(view! {
        <div
            class="segmented"
            role="tablist"
            aria-label=(label)
            tabindex="-1"
            data-roving="tablist"
            data-c="segmented"
        >
            for (id, label) in pairs {
                <button
                    id=(format!("{uid}-seg-{id}"))
                    type="button"
                    role="tab"
                    class="segment"
                    :aria-selected=$(if value.get() == id { "true" } else { "false" })
                    :tabindex=$(if value.get() == id { "0" } else { "-1" })
                    @click=$(|_e: Event| value.set(id.to_owned()))
                >
                    <span class="pill" aria-hidden="true" :hidden=$(value.get() != id)></span>
                    <span class="text">(label)</span>
                </button>
            }
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::{runtime::signal, view::ViewExt};

    use super::*;
    use crate::testing::render;

    #[component]
    async fn demo(cx: &Cx) -> Result<impl View> {
        let value = signal(cx, || "week".to_owned());
        let items = vec![Choice::new("day", "Day"), Choice::new("week", "Week")];
        Ok(view! { segmented(items: items, value: &value, label: "Range") })
    }

    #[test]
    fn it_is_a_tablist_of_segments_with_one_selected() {
        let html = render("en", |cx| view! { demo() }.boxed());
        assert!(html.contains(r#"role="tablist""#), "{html}");
        assert!(html.contains(r#"aria-label="Range""#), "{html}");
        assert_eq!(html.matches(r#"role="tab""#).count(), 2, "{html}");
        assert_eq!(html.matches(r#"aria-selected="true""#).count(), 1, "{html}");
        assert!(html.contains(r#"<span class="text">Week</span>"#), "{html}");
        assert_eq!(html.matches("class=\"pill\"").count(), 2, "{html}");
    }
}
