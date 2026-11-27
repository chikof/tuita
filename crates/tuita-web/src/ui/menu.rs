//! A button that opens a menu of actions on the native popover.

use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, Child, View, class, component, view},
};

use crate::{icons::Icon, ui::icon::icon};

/// One entry of a [`menu`].
#[derive(Default)]
pub(crate) struct MenuItem<'a> {
    /// Stable name of the entry, rendered as `data-id` on its button.
    pub(crate) id: &'a str,
    pub(crate) label: &'a str,
    /// Small uppercase text on the right (a shortcut or a state).
    pub(crate) meta: Option<&'a str>,
    pub(crate) danger: bool,
    pub(crate) disabled: bool,
    /// A pixel icon drawn before the label.
    pub(crate) icon: Option<Icon>,
    /// A rule above this entry.
    pub(crate) separator: bool,
    /// What the entry does: attributes for its `<button>`, usually `@click=$(..)`. A click also
    /// closes the menu (`popovertargetaction="hide"`), so do not add `command`/`commandfor`.
    pub(crate) attrs: Attributes,
}

/// A trigger button (`trigger` is its content) and the `role="menu"` popover it opens.
///
/// `id` must be unique on the page and a valid CSS identifier part: the popover is `{id}-menu`,
/// its anchor `--menu-{id}`. `label` names the menu. `plain` drops the bordered button look so
/// the trigger content draws itself.
///
/// Opening, light dismiss and Escape are native. Moving focus to the first enabled item when the
/// menu opens and arrow, Home and End navigation between `[role="menuitem"]` are for the shell
/// island: it must handle every `[popover][role="menu"]`. The trigger has no `aria-expanded`;
/// browsers expose a popover invoker's expanded state themselves, and the open look is CSS
/// (`:popover-open`).
#[component]
pub(crate) async fn menu(
    cx: &Cx,
    id: &str,
    label: &str,
    items: Vec<MenuItem<'_>>,
    #[default] plain: bool,
    trigger: Child<'_>,
) -> Result<impl View> {
    let panel = format!("{id}-menu");
    let anchor = format!("--menu-{id}");
    Ok(view! {
        <button
            type="button"
            class=(class!("trigger", "plain" if plain))
            popovertarget=(&panel)
            aria-haspopup="menu"
            style=(format!("anchor-name:{anchor}"))
            data-c="menu"
        >
            (trigger)
        </button>
        <div
            id=(&panel)
            class="menu"
            popover="auto"
            role="menu"
            aria-label=(label)
            tabindex="-1"
            style=(format!("position-anchor:{anchor}"))
            data-c="menu"
        >
            for item in items {
                if item.separator {
                    <hr class="rule">
                }
                <button
                    type="button"
                    role="menuitem"
                    class=(class!("item", "danger" if item.danger))
                    disabled=(item.disabled)
                    popovertarget=(&panel)
                    popovertargetaction="hide"
                    data-id=(item.id)
                    (item.attrs)
                >
                    if let Some(name) = item.icon {
                        <span class="lead">
                            icon(name: name, size: 20)
                            <span>(item.label)</span>
                        </span>
                    } else {
                        <span>(item.label)</span>
                    }
                    if let Some(meta) = item.meta {
                        <span class="meta">(meta)</span>
                    }
                </button>
            }
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::{ViewExt, attributes};

    use super::*;
    use crate::testing::render;

    fn sample(plain: bool) -> String {
        render("en", |cx| {
            let items = vec![
                MenuItem {
                    id: "edit",
                    label: "Edit",
                    icon: Some(Icon::Pencil),
                    attrs: attributes! { data-action="edit" },
                    ..MenuItem::default()
                },
                MenuItem {
                    id: "delete",
                    label: "Delete",
                    danger: true,
                    separator: true,
                    meta: Some("DEL"),
                    ..MenuItem::default()
                },
                MenuItem {
                    id: "off",
                    label: "Off",
                    disabled: true,
                    ..MenuItem::default()
                },
            ];
            view! {
                menu(
                    id: "post",
                    label: "Post actions",
                    items: items,
                    plain: plain,
                    trigger: view! { "More" }.into()
                )
            }
            .boxed()
        })
    }

    #[test]
    fn the_trigger_opens_the_menu_popover() {
        let html = sample(false);
        assert!(html.contains(r#"popovertarget="post-menu""#), "{html}");
        assert!(html.contains(r#"aria-haspopup="menu""#), "{html}");
        assert!(html.contains(r#"class="trigger""#), "{html}");
        assert!(html.contains("anchor-name:--menu-post"), "{html}");
        assert!(html.contains("position-anchor:--menu-post"), "{html}");
        assert!(sample(true).contains(r#"class="trigger plain""#));
    }

    #[test]
    fn the_panel_is_a_named_menu_of_items() {
        let html = sample(false);
        assert!(html.contains(r#"id="post-menu""#), "{html}");
        assert!(
            html.contains(r#"role="menu""#) && html.contains(r#"aria-label="Post actions""#),
            "{html}"
        );
        assert!(html.contains(r#"popover="auto""#), "{html}");
        assert_eq!(html.matches(r#"role="menuitem""#).count(), 3, "{html}");
    }

    #[test]
    fn items_carry_their_look_state_and_attrs() {
        let html = sample(false);
        assert!(html.contains(r#"class="item danger""#), "{html}");
        assert!(
            html.contains(r#"data-id="edit""#) && html.contains(r#"data-action="edit""#),
            "{html}"
        );
        assert!(html.contains(r#"<span class="meta">DEL</span>"#), "{html}");
        assert!(html.contains(r#"<hr class="rule">"#), "{html}");
        assert!(html.contains(r#"<span class="lead">"#), "{html}");
        assert_eq!(html.matches("disabled").count(), 1, "{html}");
        assert_eq!(
            html.matches(r#"popovertargetaction="hide""#).count(),
            3,
            "{html}"
        );
    }
}
