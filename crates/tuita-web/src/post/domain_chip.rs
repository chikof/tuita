//! The home domain of a remote account.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::{icons::Icon, l10n::t, ui::icon::icon};

/// The home domain of a remote account, in mono, so no one mistakes it for a local one.
#[component]
pub(crate) async fn domain_chip(cx: &Cx, domain: &str) -> Result<impl View> {
    let title = t!(remote_domain_title, domain: domain)
        .resolve(cx)
        .into_owned();
    Ok(view! {
        <span class="chip" data-domain=(domain) title=(&title) data-c="domain-chip">
            icon(name: Icon::Globe, size: 12)
            <span class="domain" translate="no">(domain)</span>
        </span>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn the_chip_names_the_domain_and_explains_it() {
        let html = render("en", |cx| {
            view! { domain_chip(domain: "x.example") }.boxed()
        });
        assert!(html.contains(r#"data-domain="x.example""#), "{html}");
        assert!(
            html.contains(r#"<span class="domain" translate="no">x.example</span>"#),
            "{html}"
        );
        assert!(html.contains("title=\""), "{html}");
    }
}
