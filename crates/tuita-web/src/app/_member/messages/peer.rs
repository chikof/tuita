//! `/messages/{peer}`: the conversation with one account (`ada` or `ada@remote.example`).

use topcoat::{
    Result,
    context::{Cx, memoize},
    metadata::Metadata,
    router::{module_param, page, query_params},
    view::{View, view},
};
use tuita_server::dto::ConversationViewDto;

use crate::{
    api::{self, enc},
    fragment,
    l10n::{strip_isolates, t},
    messages::{
        frame::{load_list, messages_frame},
        thread::{thread_fragment, thread_pane},
    },
    paths, session, shell,
};

module_param!(pub(crate) peer);

#[query_params(error = bad_request)]
struct ThreadQuery {
    before: Option<String>,
    after: Option<String>,
}

/// `GET /conversations/{peer}` with the page's `before` and `after`.
#[memoize(as_ref)]
async fn conversation(cx: &Cx) -> Result<ConversationViewDto> {
    let query = query_params::<ThreadQuery>(cx)?;
    let path = format!(
        "/conversations/{}{}",
        enc(peer(cx)),
        api::query(&[
            ("before", query.before.as_deref()),
            ("after", query.after.as_deref())
        ])
    );
    Ok(api::get(cx, &path).await?)
}

async fn title(cx: &Cx) -> Result<Metadata> {
    let name = conversation(cx).await?.user.display_name.as_str();
    let text = strip_isolates(&t!(chat_doc_title, name: name).resolve(cx));
    shell::titled(cx, &text).await
}

/// The thread. A fragment request answers with the items only: the newest page, the page after
/// `?after=<id>` or the page before `?before=<cursor>`.
#[page(with(Metadata::from_fn(async |cx| title(cx).await)))]
async fn conversation_page(cx: &Cx) -> Result<impl View> {
    let me = session::member(cx).await?.me.user.id.to_string();
    let view = conversation(cx).await?;
    let fragment = fragment::requested(cx);
    let query = query_params::<ThreadQuery>(cx)?;
    let older = view.next.as_deref().map(|cursor| {
        format!(
            "{}{}",
            paths::messages(peer(cx)),
            api::query(&[("before", Some(cursor))])
        )
    });
    let list = if fragment {
        None
    } else {
        Some(load_list(cx, None).await?)
    };
    let open = peer(cx).to_lowercase();
    let fragment_older = older.clone().filter(|_| query.before.is_some());
    Ok(view! {
        if fragment {
            thread_fragment(view: view, older: fragment_older.as_deref())
        } else if let Some(list) = &list {
            messages_frame(
                list: list,
                me: &me,
                open: open.as_str(),
                thread_pane(view: view, me: &me, older: older.as_deref())
            )
        }
    })
}
