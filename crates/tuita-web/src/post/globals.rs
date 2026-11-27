//! The hosts the post islands fill, rendered once per page for a signed-in visitor.

use topcoat::{
    Result,
    context::Cx,
    runtime::Event,
    view::{View, component, view},
};

use crate::{l10n::t, ui::confirm_dialog::confirm_dialog};

/// The host of the lightbox (`lightbox.js` puts the fetched dialog into it), the host of the post
/// dialogs (`post.js`) and the two questions a post can ask.
///
/// Confirming a question dispatches `tuita-delete` or `tuita-block` on `window`; `post.js` listens and does what it remembered. The block question
/// names the account, so `post.js` writes it into the dialog title before opening.
#[component]
pub(crate) async fn post_globals(cx: &Cx) -> Result<impl View> {
    let delete_title = t!(post_delete_title).resolve(cx).into_owned();
    let delete_body = t!(post_delete_body).resolve(cx).into_owned();
    let delete_label = t!(post_delete).resolve(cx).into_owned();
    let block_title = t!(block_confirm).resolve(cx).into_owned();
    let block_body = t!(block_body).resolve(cx).into_owned();
    Ok(view! {
        <div id="lightbox-host" data-c="post-globals"></div>
        <div id="post-dialogs"></div>
        confirm_dialog(
            id: "confirm-delete-post",
            title: &delete_title,
            body: &delete_body,
            confirm_label: &delete_label,
            danger: true,
            action: $(|_e: Event| raw!("dispatchEvent(new Event('tuita-delete'))"))
        )
        confirm_dialog(
            id: "confirm-block",
            title: &block_title,
            body: &block_body,
            confirm_label: &block_title,
            danger: true,
            action: $(|_e: Event| raw!("dispatchEvent(new Event('tuita-block'))"))
        )
    })
}
