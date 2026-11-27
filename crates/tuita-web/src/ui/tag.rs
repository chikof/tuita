//! A small uppercase label.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, class, component, view},
};

/// A mono, uppercase `<span>`. `pill` draws a border around it; `compact` tightens the padding
/// and letter spacing of a pill, for tags inside list rows.
#[component]
pub(crate) async fn tag(
    cx: &Cx,
    #[default] pill: bool,
    #[default] compact: bool,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <span class=(class!("tag", "pill" if pill, "compact" if compact)) data-c="tag">
            (child)
        </span>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn flags_become_classes() {
        let plain = render("en", |cx| view! { tag("new") }.boxed());
        assert!(plain.contains(r#"<span class="tag""#), "{plain}");
        let pill = render("en", |cx| {
            view! { tag(pill: true, compact: true, "new") }.boxed()
        });
        assert!(pill.contains(r#"class="tag pill compact""#), "{pill}");
    }
}
