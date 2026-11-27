//! Tabs that are links: the selected tab comes from the URL (a variant of `Tabs` for
//! pages whose tabs are `?tab=` or path links, so a full page load shows the tab).

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::ui::mask::{Axis, ramp_mask};

/// One link tab: a stable `id`, its visible `label` and where it leads.
#[derive(Clone, Debug)]
pub(crate) struct LinkTab<'a> {
    pub(crate) id: &'a str,
    pub(crate) label: &'a str,
    pub(crate) href: String,
}

/// A `role="tablist"` named `label` of `<a role="tab">` links; the one whose id equals `selected`
/// is `aria-selected` and carries the dithered underline. Arrow keys are `shell.js` (every
/// `[data-roving="tablist"]`): they focus and click the neighbour, which follows its link.
#[component]
pub(crate) async fn link_tabs(
    cx: &Cx,
    items: Vec<LinkTab<'_>>,
    selected: &str,
    label: &str,
) -> Result<impl View> {
    let underline = ramp_mask(0.3, 1.1, 4, Axis::Y);
    Ok(view! {
        <div class="tabs" data-c="link-tabs">
            <div
                class="list"
                role="tablist"
                aria-label=(label)
                data-roving="tablist"
                style=(format!("--underline:{underline}"))
            >
                for item in items {
                    <a
                        role="tab"
                        class="tab"
                        href=(item.href)
                        aria-selected=(if item.id == selected {
                            "true" // lint-strings: allow
                        } else {
                            "false" // lint-strings: allow
                        })
                        tabindex=(if item.id == selected { "0" } else { "-1" })
                    >
                        <span class="text">
                            (item.label)
                            if item.id == selected {
                                <span class="indicator"><i></i></span>
                            }
                        </span>
                    </a>
                }
            </div>
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn the_selected_tab_is_a_link_marked_selected() {
        let html = render("en", |cx| {
            let items = vec![
                LinkTab {
                    id: "posts",
                    label: "Posts",
                    href: "/@ada".to_owned(),
                },
                LinkTab {
                    id: "media",
                    label: "Media",
                    href: "/@ada?tab=media".to_owned(),
                },
            ];
            view! { link_tabs(items: items, selected: "media", label: "Profile") }.boxed()
        });
        assert!(html.contains(r#"role="tablist""#), "{html}");
        assert_eq!(html.matches(r#"role="tab""#).count(), 2, "{html}");
        assert_eq!(html.matches(r#"aria-selected="true""#).count(), 1, "{html}");
        assert!(html.contains(r#"href="/@ada?tab=media""#), "{html}");
        assert_eq!(html.matches("indicator").count(), 1, "{html}");
    }
}
