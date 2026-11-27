//! A pixelarticons icon inline as `<svg>`.

use topcoat::{
    Result,
    context::Cx,
    view::{Unescaped, View, component, view},
};

use crate::icons::Icon;

/// The icon `name`, `size` px square (24 by default), drawn in the current text colour.
///
/// With a `label` it is an image with that accessible name; without one it is hidden from
/// assistive technology.
#[component]
pub(crate) async fn icon(
    cx: &Cx,
    name: Icon,
    #[default(24)] size: u32,
    #[default] label: Option<&str>,
) -> Result<impl View> {
    let markup = Unescaped::new_unchecked(name.inner());
    Ok(view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            width=(size)
            height=(size)
            fill="currentColor"
            shape-rendering="crispEdges"
            role=(label.map(|_| "img"))
            aria-label=(label)
            aria-hidden=(label.is_none().then_some("true"))
            focusable="false"
            data-c="icon"
        >
            (markup)
        </svg>
    })
}
