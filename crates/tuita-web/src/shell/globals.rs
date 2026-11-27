use topcoat::{
    Result,
    context::Cx,
    router::request::uri,
    view::{View, component, view},
};

use crate::{
    compose::compose_dialog::compose_dialog, messages::globals::messages_globals,
    post::globals::post_globals, session, shell::shortcuts_dialog::shortcuts_dialog,
    ui::toaster::toaster,
};

/// Rendered once at the end of `<body>` by the root layout.
///
/// For a signed-in visitor (the gallery at `/design` fakes a session and is excluded) it renders
/// the marker `stream.js` starts the live stream from, and the shortcuts dialog the `?` key
/// opens, and the compose dialog.
#[component]
pub(crate) async fn globals(cx: &Cx) -> Result<impl View> {
    let path = uri(cx).path().to_owned();
    let member = if path == "/design" || path.starts_with("/design/") {
        None
    } else {
        session::viewer(cx).await?.as_ref()
    };
    let handle = member.map(|viewer| viewer.me.user.handle.clone());
    Ok(view! {
        toaster()
        if let Some(handle) = handle {
            <div data-island="stream" data-handle=(handle) hidden=""></div>
            shortcuts_dialog()
            compose_dialog()
            post_globals()
            messages_globals()
        }
    })
}
