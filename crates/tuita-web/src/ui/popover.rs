//! A button that opens a floating panel on the native popover.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, class, component, view},
};

use crate::{icons::Icon, ui::icon::icon};

/// A trigger button and the panel it opens.
///
/// `label` names the panel (`role="dialog"`) and, with `glyph`, the trigger. `glyph` makes the
/// trigger a 32px icon button; without it `trigger` is the trigger's content. `id` must be unique
/// on the page and a valid CSS identifier part: the panel is `{id}-panel`, its anchor `--pop-{id}`.
///
/// Opening, light dismiss and Escape are native. The trigger has no `aria-expanded`; browsers
/// expose a popover invoker's expanded state themselves, and the open look is CSS
/// (`:popover-open`).
#[component]
pub(crate) async fn popover(
    cx: &Cx,
    id: &str,
    label: &str,
    #[default]
    #[into]
    glyph: Option<Icon>,
    #[default] trigger: Child<'_>,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let panel = format!("{id}-panel");
    let anchor = format!("--pop-{id}");
    Ok(view! {
        <button
            type="button"
            class=(class!("trigger", "icon" if glyph.is_some()))
            popovertarget=(&panel)
            aria-haspopup="dialog"
            style=(format!("anchor-name:{anchor}"))
            data-c="popover"
        >
            if let Some(name) = glyph {
                icon(name: name, size: 20)
                <span class="sr-only">(label)</span>
            } else {
                (trigger)
            }
        </button>
        <div
            id=(&panel)
            class="panel"
            popover="auto"
            role="dialog"
            aria-label=(label)
            style=(format!("position-anchor:{anchor}"))
            data-c="popover"
        >
            (child)
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn the_trigger_content_opens_a_named_dialog_panel() {
        let html = render("en", |cx| {
            view! {
                popover(
                    id: "emoji",
                    label: "Emoji",
                    trigger: view! { "Pick" }.into(),
                    <p>"grid"</p>
                )
            }
            .boxed()
        });
        assert!(html.contains(r#"popovertarget="emoji-panel""#), "{html}");
        assert!(html.contains(r#"aria-haspopup="dialog""#), "{html}");
        assert!(
            html.contains(r#"class="trigger""#) && html.contains(">Pick</button>"),
            "{html}"
        );
        assert!(html.contains(r#"id="emoji-panel""#), "{html}");
        assert!(
            html.contains(r#"role="dialog""#) && html.contains(r#"aria-label="Emoji""#),
            "{html}"
        );
        assert!(
            html.contains(r#"popover="auto""#) && html.contains("<p>grid</p>"),
            "{html}"
        );
    }

    #[test]
    fn an_icon_trigger_is_named_by_the_label() {
        let html = render("en", |cx| {
            view! { popover(id: "i", label: "Info", glyph: Icon::InfoBox, "x") }.boxed()
        });
        assert!(html.contains(r#"class="trigger icon""#), "{html}");
        assert!(
            html.contains(r#"<span class="sr-only">Info</span>"#),
            "{html}"
        );
        assert!(html.contains("<svg"), "{html}");
    }
}
