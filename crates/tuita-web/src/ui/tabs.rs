//! Tabs with a dithered underline.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal},
    view::{Child, View, component, element_id, view},
};

use crate::ui::mask::{Axis, ramp_mask};

/// One tab (or segment): a stable `id` and its visible `label`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Choice<'a> {
    pub(crate) id: &'a str,
    pub(crate) label: &'a str,
}

impl<'a> Choice<'a> {
    #[must_use]
    pub(crate) const fn new(id: &'a str, label: &'a str) -> Self {
        Self { id, label }
    }
}

/// A `role="tablist"` named `label` whose selected tab is the `value` signal (an item id). A click
/// sets the signal in the browser; read it in a `$(..)` or a shard to show the matching content.
///
/// With `panel` the content is a `role="tabpanel"` named like the list; without it
/// the caller owns the content. (Runtime expressions cannot join strings, so the panel is named by `label`, not by the selected tab.) Never read the signal outside a `$(..)`: that makes the whole page rerun on every click. Arrow keys, Home and End moving the selection are for the shell
/// island: it must handle `[role="tablist"][data-roving]` by focusing and clicking the
/// neighbouring tab.
#[component]
pub(crate) async fn tabs(
    cx: &Cx,
    items: Vec<Choice<'_>>,
    value: &Signal<String>,
    label: &str,
    #[default] panel: bool,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let uid = element_id(cx);
    let pairs: Vec<(&str, &str)> = items.iter().map(|item| (item.id, item.label)).collect();
    let underline = ramp_mask(0.3, 1.1, 4, Axis::Y);
    let panel_id = format!("{uid}-panel");
    Ok(view! {
        <div class="tabs" data-c="tabs">
            <div
                class="list"
                role="tablist"
                aria-label=(label)
                tabindex="-1"
                data-roving="tablist"
                style=(format!("--underline:{underline}"))
            >
                for (id, label) in pairs {
                    <button
                        id=(format!("{uid}-tab-{id}"))
                        type="button"
                        role="tab"
                        class="tab"
                        :aria-selected=$(if value.get() == id {
                            "true" // lint-strings: allow
                        } else {
                            "false" // lint-strings: allow
                        })
                        :tabindex=$(if value.get() == id { "0" } else { "-1" })
                        aria-controls=(panel.then(|| panel_id.clone()))
                        @click=$(|_e: Event| value.set(id.to_owned()))
                    >
                        (label)
                        <span class="indicator" :hidden=$(value.get() != id)>
                            <i></i>
                        </span>
                    </button>
                }
            </div>
            if panel {
                <div id=(&panel_id) role="tabpanel" aria-label=(label) tabindex="0">
                    (child)
                </div>
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
    async fn demo(cx: &Cx, panel: bool) -> Result<impl View> {
        let value = signal(cx, || "posts".to_owned());
        let items = vec![Choice::new("posts", "Posts"), Choice::new("media", "Media")];
        Ok(view! {
            tabs(
                items: items,
                value: &value,
                label: "Profile",
                panel: panel,
                <p>"body"</p>
            )
        })
    }

    #[test]
    fn it_is_a_tablist_with_the_signals_tab_selected() {
        let html = render("en", |cx| view! { demo(panel: false) }.boxed());
        assert!(html.contains(r#"role="tablist""#), "{html}");
        assert!(html.contains(r#"aria-label="Profile""#), "{html}");
        assert_eq!(html.matches(r#"role="tab""#).count(), 2, "{html}");
        assert_eq!(html.matches(r#"aria-selected="true""#).count(), 1, "{html}");
        assert_eq!(
            html.matches(r#"aria-selected="false""#).count(),
            1,
            "{html}"
        );
        assert_eq!(html.matches(r#"tabindex="0""#).count(), 1, "{html}");
        assert!(!html.contains("tabpanel"), "{html}");
        assert!(html.contains("data-topcoat-on:click"), "{html}");
    }

    #[test]
    fn a_panel_is_named_like_the_list() {
        let html = render("en", |cx| view! { demo(panel: true) }.boxed());
        assert!(html.contains(r#"role="tabpanel""#), "{html}");
        assert!(html.contains("<p>body</p>"), "{html}");
        assert!(
            html.contains(r#"aria-label="Profile" tabindex="0""#),
            "{html}"
        );
        assert!(html.contains("aria-controls"), "{html}");
    }
}
