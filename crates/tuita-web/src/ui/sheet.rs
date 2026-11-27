//! A dialog docked to the side.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, component, view},
};

use crate::ui::dialog::{DialogVariant, dialog};

/// A [`dialog`] in its `Sheet` variant: docked to the right edge on wide screens and a bottom
/// sheet on phones. Open it with `command="show-modal" commandfor=(id)`.
#[component]
pub(crate) async fn sheet(
    cx: &Cx,
    id: &str,
    title: &str,
    #[default] actions: Option<Child<'_>>,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        dialog(
            id: id,
            title: title,
            variant: DialogVariant::Sheet,
            actions: actions,
            (child)
        )
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn it_is_a_sheet_variant_dialog() {
        let html = render("en", |cx| {
            view! { sheet(id: "filters", title: "Filters", <p>"x"</p>) }.boxed()
        });
        assert!(html.contains(r#"<dialog id="filters""#), "{html}");
        assert!(html.contains(r#"class="dialog sheet""#), "{html}");
    }
}
