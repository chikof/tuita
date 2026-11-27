//! The header of a page: title, back button and actions.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, attributes, component, view},
};

use crate::{icons::Icon, l10n::t, ui::icon_button::icon_button};

/// The heading level of a [`page_bar`]. The design gallery shows bars under its own `h1`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Heading {
    #[default]
    H1,
    H2,
}

/// A sticky bar with the page `title` (`sub` under it), an optional back button, an `actions` slot
/// on the right and a `below` slot (tabs) under the row.
///
/// The browser tab title is not set here: pages set it through their metadata
/// (`#[page(with(..))]`, see `shell::titled`). The back button is `[data-back]`, which `shell.js`
/// turns into history back or, for a page opened directly, a link home.
#[component]
pub(crate) async fn page_bar(
    cx: &Cx,
    title: &str,
    #[default]
    #[into]
    sub: Option<&str>,
    #[default] back: bool,
    #[default] heading: Heading,
    #[default] actions: Option<Child<'_>>,
    #[default] below: Option<Child<'_>>,
) -> Result<impl View> {
    let label = t!(back).resolve(cx).into_owned();
    Ok(view! {
        <header class="bar" data-c="page-bar">
            <div class="row">
                if back {
                    icon_button(
                        glyph: Icon::ArrowLeft,
                        label: &label,
                        attrs: attributes! { data-back="" }
                    )
                }
                <div class="titles">
                    if heading == Heading::H1 {
                        <h1>(title)</h1>
                    } else {
                        <h2>(title)</h2>
                    }
                    if let Some(sub) = sub {
                        <p class="sub">(sub)</p>
                    }
                </div>
                if let Some(actions) = actions {
                    <div class="actions">(actions)</div>
                }
            </div>
            if let Some(below) = below {
                (below)
            }
        </header>
    })
}
