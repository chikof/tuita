//! `/messages` and `/messages/{peer}`.

mod peer;

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::{page, query_params},
    view::{View, view},
};

use crate::{
    fragment,
    l10n::t,
    messages::frame::{conversation_rows, load_list, messages_frame, start_pane},
    session, shell,
};

#[query_params(error = bad_request)]
struct ListQuery {
    before: Option<String>,
    open: Option<String>,
}

async fn title(cx: &Cx) -> Result<Metadata> {
    let title = t!(messages_title).resolve(cx).into_owned();
    shell::titled(cx, &title).await
}

/// The conversation list. A fragment request (`?before=`) answers with the rows and the next
/// link only.
#[page(with(Metadata::from_fn(async |cx| title(cx).await)))]
async fn messages_page(cx: &Cx) -> Result<impl View> {
    let me = session::member(cx).await?.me.user.id.to_string();
    let query = query_params::<ListQuery>(cx)?;
    let list = load_list(cx, query.before.as_deref()).await?;
    let fragment = fragment::requested(cx);
    Ok(view! {
        if fragment {
            conversation_rows(list: &list, open: query.open.as_deref())
        } else {
            messages_frame(list: &list, me: &me, start_pane())
        }
    })
}
