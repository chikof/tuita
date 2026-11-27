//! The create/edit dialog of a list.
//!
//! Fetched on demand from `/_tuita/dialog/list-editor` (new) or `/_tuita/dialog/list-editor/{id}`
//! (edit) and opened by `lists.js`, which also saves it: inserted markup is not hydrated, so the
//! private switch is plain markup the island toggles.

use topcoat::{
    Result,
    context::Cx,
    router::{path_param, route},
    view::{View, ViewExt, ViewHandle, attributes, component, view},
};
use tuita_server::dto::ListDto;

use crate::{
    api::{self, enc},
    l10n::t,
    ui::{
        button::{ButtonVariant, button},
        dialog::dialog,
        text_area::text_area,
        text_field::text_field,
    },
};

/// The id of the editor dialog; `lists.js` keeps at most one in the page.
pub(crate) const DIALOG_ID: &str = "list-editor";

/// The editor of `list`, or of a new list.
#[component]
pub(crate) async fn list_editor_dialog(
    cx: &Cx,
    #[default] list: Option<&ListDto>,
) -> Result<impl View> {
    let title = if list.is_some() {
        t!(lists_edit).resolve(cx).into_owned()
    } else {
        t!(lists_new).resolve(cx).into_owned()
    };
    let name = list.map_or("", |list| list.name.as_str());
    let description = list.map_or("", |list| list.description.as_str());
    let private = list.is_some_and(|list| list.private);
    let id = list.map(|list| list.id.to_string());
    let name_label = t!(lists_name).resolve(cx).into_owned();
    let description_label = t!(lists_description).resolve(cx).into_owned();
    Ok(view! {
        dialog(
            id: DIALOG_ID,
            title: &title,
            actions: Some(
                view! {
                    button(
                        variant: ButtonVariant::Ghost,
                        attrs: attributes! { command="close" commandfor=(DIALOG_ID) },
                        (t!(cancel))
                    )
                    button(
                        submit: true,
                        disabled: name.trim().is_empty(),
                        attrs: attributes! { form="list-editor-form" data-act="list-save" },
                        (t!(lists_save))
                    )
                }.into(

                ),
            ),
            <form
                id="list-editor-form"
                data-c="list-editor-form"
                novalidate=""
                data-list-form=""
                data-list-id=(id)
            >
                text_field(
                    label: &name_label,
                    value: name,
                    attrs: attributes! { name="name" maxlength="40" required="" }
                )
                text_area(
                    label: &description_label,
                    value: description,
                    rows: 2,
                    attrs: attributes! { name="description" maxlength="160" }
                )
                <button
                    type="button"
                    role="switch"
                    class="switch"
                    data-c="switch"
                    data-list-private=""
                    aria-checked=(if private { "true" } else { "false" })
                >
                    <span class="track" aria-hidden="true">
                        <i class="fill"></i>
                        <i class="thumb"></i>
                    </span>
                    <span class="text">(t!(lists_private_switch))</span>
                </button>
                <p class="failure" role="alert" hidden=""></p>
            </form>
        )
    })
}

path_param!(list_id);

#[route(GET "/_tuita/dialog/list-editor")]
async fn new_list(cx: &Cx) -> Result<ViewHandle> {
    Box::pin(view! { list_editor_dialog() }.first()).await
}

#[route(GET "/_tuita/dialog/list-editor/{list_id}")]
async fn edit_list(cx: &Cx) -> Result<ViewHandle> {
    let list = api::get::<ListDto>(cx, &format!("/lists/{}", enc(list_id(cx)))).await?;
    Box::pin(view! { list_editor_dialog(list: Some(&list)) }.first()).await
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::{lists::testing::sample_list, testing::render};

    #[test]
    fn a_new_list_starts_empty_with_a_disabled_save() {
        let html = render("en", |cx| view! { list_editor_dialog() }.boxed());
        assert!(
            html.contains(r#"<dialog"#) && html.contains(r#"id="list-editor""#),
            "{html}"
        );
        assert!(html.contains(">New list<"), "{html}");
        assert!(
            html.contains(r#"role="switch""#) && html.contains(r#"aria-checked="false""#),
            "{html}"
        );
        assert!(
            html.contains(r#"data-act="list-save""#) && html.contains("disabled"),
            "{html}"
        );
        assert!(
            html.contains(r#"maxlength="40""#) && html.contains(r#"maxlength="160""#),
            "{html}"
        );
        assert!(!html.contains("data-list-id"), "{html}");
    }

    #[test]
    fn editing_fills_the_fields_and_keeps_the_privacy() {
        let mut list = sample_list("Friends", true);
        list.description = "Close ones".to_owned();
        let html = render("en", |cx| {
            view! { list_editor_dialog(list: Some(&list)) }.boxed()
        });
        assert!(html.contains(">Edit list<"), "{html}");
        assert!(html.contains(r#"value="Friends""#), "{html}");
        assert!(html.contains("Close ones"), "{html}");
        assert!(html.contains(r#"aria-checked="true""#), "{html}");
        assert!(
            html.contains("data-list-id=\"00000000-0000-0000-0000-000000001157\""),
            "{html}"
        );
    }
}
