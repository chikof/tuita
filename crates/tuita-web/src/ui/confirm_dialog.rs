//! Confirm dialog for destructive actions.

use topcoat::{
    Result,
    context::Cx,
    runtime::{EventHandlerFn, Expr},
    view::{View, attributes, component, view},
};

use crate::{
    l10n::t,
    ui::{
        button::{ButtonVariant, button},
        dialog::dialog,
    },
};

/// A modal that asks `title` / `body` and runs `action` when the user clicks `confirm_label`.
///
/// There is no global store and no promise: render one `confirm_dialog` per question, next to the
/// control that asks it, and open it with `command="show-modal" commandfor=(id)`. `action` is the
/// confirm button's click handler, a runtime expression such as
/// `$(async |_e| { remove(post_id).await; })`. Both buttons close the dialog. `danger` paints the
/// confirm button red.
#[component]
pub(crate) async fn confirm_dialog<F: EventHandlerFn + Send + Sync>(
    cx: &Cx,
    id: &str,
    title: &str,
    body: &str,
    confirm_label: &str,
    #[default] danger: bool,
    action: Expr<F>,
) -> Result<impl View> {
    let variant = if danger {
        ButtonVariant::Danger
    } else {
        ButtonVariant::Solid
    };
    let cancel = t!(cancel).resolve(cx).into_owned();
    Ok(view! {
        dialog(
            id: id,
            title: title,
            actions: Some(
                view! {
                    button(
                        variant: ButtonVariant::Ghost,
                        attrs: attributes! { command="close" commandfor=(id) },
                        (&cancel)
                    )
                    button(
                        variant: variant,
                        attrs: attributes! { command="close" commandfor=(id) @click=(action) },
                        (confirm_label)
                    )
                }.into(

                ),
            ),
            <p>(body)</p>
        )
    })
}

#[cfg(test)]
mod test {
    use topcoat::{runtime::Event, view::ViewExt};

    use super::*;
    use crate::testing::render;

    #[component]
    async fn demo(cx: &Cx, danger: bool) -> Result<impl View> {
        Ok(view! {
            confirm_dialog(
                id: "sure",
                title: "Delete post?",
                body: "It cannot be undone.",
                confirm_label: "Delete",
                danger: danger,
                action: $(|_e: Event| raw!("history.back()"))
            )
        })
    }

    #[test]
    fn it_asks_in_a_dialog_with_cancel_and_confirm() {
        let html = render("en", |cx| view! { demo(danger: false) }.boxed());
        assert!(html.contains(r#"<dialog id="sure""#), "{html}");
        assert!(
            html.contains(r#"<h2 id="sure-title">Delete post?</h2>"#),
            "{html}"
        );
        assert!(html.contains("<p>It cannot be undone.</p>"), "{html}");
        assert!(
            html.contains(r#"<span class="label">Cancel</span>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<span class="label">Delete</span>"#),
            "{html}"
        );
        assert!(html.contains(r#"class="btn solid md wash""#), "{html}");
    }

    #[test]
    fn the_confirm_button_runs_the_action_and_both_close_the_dialog() {
        let html = render("en", |cx| view! { demo(danger: true) }.boxed());
        assert!(html.contains(r#"class="btn danger md wash""#), "{html}");
        assert_eq!(html.matches("data-topcoat-on:click").count(), 1, "{html}");
        // The close button of the header and the two footer buttons.
        assert_eq!(html.matches(r#"command="close""#).count(), 3, "{html}");
    }
}
