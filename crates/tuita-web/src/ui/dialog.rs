//! Dialog on the native `<dialog>` element.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, attributes, component, view},
};

use crate::{
    icons::Icon,
    l10n::t,
    ui::icon_button::{IconButtonVariant, icon_button},
};

/// Where a [`dialog`] sits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DialogVariant {
    /// Centred; a bottom sheet under 600px.
    #[default]
    Dialog,
    /// Docked to the right edge on wide screens; a bottom sheet under 600px.
    Sheet,
}

impl DialogVariant {
    const fn class(self) -> &'static str {
        match self {
            Self::Dialog => "dialog dialog",
            Self::Sheet => "dialog sheet",
        }
    }
}

/// A modal `<dialog id=(id)>` with a title, a close button, the content and optional `actions`
/// (the footer row).
///
/// It starts closed. Open it from any button with the invoker attributes
/// `command="show-modal" commandfor=(id)`; the close button closes it with `command="close"`.
/// `closedby="any"` gives Escape and a click on the backdrop the same effect. `id` must be unique
/// on the page; the title is `{id}-title`.
///
/// Dragging the handle down to dismiss on a phone is not part of this component: the `.handle`
/// element is rendered (`aria-hidden`) for an island to wire up, setting `--drag` on the
/// `<dialog>` while the pointer is down and calling `close()` past 80px.
#[component]
pub(crate) async fn dialog(
    cx: &Cx,
    id: &str,
    title: &str,
    #[default] variant: DialogVariant,
    #[default] actions: Option<Child<'_>>,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let close = t!(close).resolve(cx).into_owned();
    Ok(view! {
        <dialog
            id=(id)
            class=(variant.class())
            aria-labelledby=(format!("{id}-title"))
            closedby="any"
            data-c="dialog"
        >
            <div class="surface">
                <span class="handle" aria-hidden="true"></span>
                <header>
                    <h2 id=(format!("{id}-title"))>(title)</h2>
                    icon_button(
                        glyph: Icon::Close,
                        label: &close,
                        variant: IconButtonVariant::Ghost,
                        attrs: attributes! { command="close" commandfor=(id) }
                    )
                </header>
                <div class="body">(child)</div>
                if let Some(actions) = actions {
                    <footer>(actions)</footer>
                }
            </div>
        </dialog>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn the_dialog_is_named_by_its_title_and_closes_through_invoker_commands() {
        let html = render("en", |cx| {
            view! { dialog(id: "edit", title: "Edit post", <p>"body"</p>) }.boxed()
        });
        assert!(html.contains(r#"<dialog id="edit""#), "{html}");
        assert!(html.contains(r#"aria-labelledby="edit-title""#), "{html}");
        assert!(
            html.contains(r#"<h2 id="edit-title">Edit post</h2>"#),
            "{html}"
        );
        assert!(html.contains(r#"closedby="any""#), "{html}");
        assert!(
            html.contains(r#"command="close""#) && html.contains(r#"commandfor="edit""#),
            "{html}"
        );
        assert!(html.contains(r#"aria-label="Close""#), "{html}");
        assert!(
            html.contains(r#"<div class="body"><p>body</p></div>"#),
            "{html}"
        );
        assert!(!html.contains("<footer>"), "{html}");
    }

    #[test]
    fn actions_fill_the_footer_and_a_sheet_has_its_class() {
        let html = render("en", |cx| {
            view! {
                dialog(
                    id: "s",
                    title: "T",
                    variant: DialogVariant::Sheet,
                    actions: Some(view! { <b>"ok"</b> }.into()),
                    "x"
                )
            }
            .boxed()
        });
        assert!(html.contains(r#"class="dialog sheet""#), "{html}");
        assert!(html.contains("<footer><b>ok</b></footer>"), "{html}");
    }

    #[test]
    fn the_close_button_is_translated() {
        let html = render("pl", |cx| {
            view! { dialog(id: "d", title: "T", "x") }.boxed()
        });
        assert!(!html.contains(r#"aria-label="Close""#), "{html}");
    }
}
