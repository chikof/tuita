//! The little wrappers the media and encryption sections share (the `.frame`, `.stack`, `.drop`,
//! `.clip` and `.foot` classes of the media and encryption sections).

use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, Child, View, class, component, view},
};

/// A bordered box that clips its content; `stack` lays the children out in a padded column.
/// `attrs` go on the root, for a demo's hooks.
#[component]
pub(crate) async fn frame(
    cx: &Cx,
    #[default] stack: bool,
    #[default] attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <div class=(class!("frame", "stack" if stack)) data-c="design-frame" (attrs)>
            (child)
        </div>
    })
}

/// A footer strip under a thread, divided from it by a line.
#[component]
pub(crate) async fn foot(
    cx: &Cx,
    #[default] attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! { <div class="foot" data-c="design-foot" (attrs)>(child)</div> })
}

/// A 16:9 (or square) box a video fills.
#[component]
pub(crate) async fn clip(
    cx: &Cx,
    #[default] square: bool,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <div class=(class!("clip", "square" if square)) data-c="design-clip">
            (child)
        </div>
    })
}

/// The content of the dropzone demo; `data-design-dropped` lists the names dropped on it.
#[component]
pub(crate) async fn drop_hint(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        <div class="drop" data-c="design-drop">
            <p>"Drop images here"</p>
            <p class="dim" data-design-dropped="">"nothing dropped yet"</p>
        </div>
    })
}
