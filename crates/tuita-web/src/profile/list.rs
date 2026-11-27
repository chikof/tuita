//! Lists that load more as you scroll.
//!
//! A list page ends with a `more` item: `<a data-more href="?before=…">`. `infinite.js` fetches
//! that URL with `x-tuita-fragment: 1` and swaps the item for the next entries and the next
//! `more` item; without scripts it is a working link. With the header a page renders only its
//! entries and the next `more` item (see [`fragment::requested`](crate::fragment::requested)).

use topcoat::{
    Result,
    context::Cx,
    router::request::uri,
    view::{Child, View, component, view},
};
use tuita_server::dto::UserDto;

use crate::{l10n::t, profile::user_row::user_row};

/// The URL of the page after `cursor`: this page's path and query with `before` set.
pub(crate) fn next_href(cx: &Cx, cursor: &str) -> String {
    let uri = uri(cx);
    let mut query = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in url::form_urlencoded::parse(uri.query().unwrap_or("").as_bytes()) {
        if key != "before" {
            query.append_pair(&key, &value);
        }
    }
    query.append_pair("before", cursor);
    format!("{}?{}", uri.path(), query.finish())
}

/// `role="list"` named `label`; its entries are `role="listitem"` rows with a rule between them.
#[component]
pub(crate) async fn infinite_list(
    cx: &Cx,
    label: &str,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <div class="list" role="list" aria-label=(label) data-c="infinite-list">
            (child)
        </div>
    })
}

/// The item that loads the next page.
#[component]
pub(crate) async fn more_item(cx: &Cx, href: &str) -> Result<impl View> {
    Ok(view! {
        <div role="listitem" class="more" data-more-host="">
            <a data-more="" href=(href)>(t!(loading))</a>
        </div>
    })
}

/// People rows as list entries, then the `more` item when there is a next page.
#[component]
pub(crate) async fn user_items(
    cx: &Cx,
    users: &[UserDto],
    more: Option<&str>,
) -> Result<impl View> {
    Ok(view! {
        #[key(user.id.as_bytes())]
        for user in users {
            <div role="listitem">user_row(user: user)</div>
        }
        if let Some(href) = more {
            more_item(href: href)
        }
    })
}
