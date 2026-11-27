//! `/@{handle}/{post_id}`: one post and its thread.
//!
//! The API returns every reply at once (`GET /posts/{id}` has no cursor), so the page lists them
//! all.

use topcoat::{
    Result,
    context::{Cx, memoize},
    metadata::Metadata,
    router::{module_param, page},
    view::{View, view},
};
use tuita_server::dto::ThreadDto;

use super::handle;
use crate::{
    api::{self, enc},
    l10n::{strip_isolates, t},
    post::post_thread::post_thread,
    shell,
};

module_param!(pub(crate) post_id);

/// `GET /posts/{id}`, once per request: the title and the page both read it.
#[memoize(as_ref)]
async fn thread(cx: &Cx) -> Result<ThreadDto> {
    Ok(api::get::<ThreadDto>(cx, &format!("/posts/{}", enc(post_id(cx)))).await?)
}

/// The tab title: `Name: "first 40 characters" · tuita`.
async fn title(cx: &Cx) -> Result<Metadata> {
    let Ok(thread) = thread(cx).await else {
        return Ok(Metadata::new().title(t!(app_name).resolve(cx).into_owned()));
    };
    let name = &thread.post.author.display_name;
    let excerpt: String = thread.post.text.chars().take(40).collect();
    let text = strip_isolates(
        &t!(thread_doc_title, name: name.as_str(), text: excerpt.as_str()).resolve(cx),
    );
    shell::titled(cx, &text).await
}

#[page(with(Metadata::from_fn(async |cx| title(cx).await)))]
async fn post_page(cx: &Cx) -> Result<impl View> {
    handle(cx)?;
    let thread = thread(cx).await?;
    Ok(view! { post_thread(thread: thread) })
}
