//! A page of posts as the infinite list `infinite.js` extends.

use topcoat::{
    Result,
    context::Cx,
    router::request::uri,
    view::{Child, View, component, view},
};
use tuita_server::dto::PostDto;

use crate::{api, l10n::t, post::post_card::post_card};

/// The URL of the next page: this page's path, the query pairs that must stay (`feed`, `tab`) and
/// `before=cursor`.
#[must_use]
pub(crate) fn more_href(cx: &Cx, keep: &[(&str, Option<&str>)], cursor: &str) -> String {
    let mut pairs = keep.to_vec();
    pairs.push(("before", Some(cursor)));
    format!("{}{}", uri(cx).path(), api::query(&pairs))
}

/// The entries of one page, then the sentinel that loads the next one when `more` is the next
/// page's URL. This is the whole answer to a `?before=` fragment request.
///
/// The sentinel wraps its link in a `data-more-host` item so `infinite.js` replaces the whole
/// item: the fetched items take its place instead of nesting inside it.
#[component]
pub(crate) async fn post_items(
    cx: &Cx,
    posts: &[PostDto],
    #[default] more: Option<&str>,
) -> Result<impl View> {
    Ok(view! {
        #[key(post.entry_id.as_bytes())]
        for post in posts {
            <div role="listitem">post_card(post: post)</div>
        }
        if let Some(href) = more {
            <div role="listitem" class="more" data-more-host="">
                <a data-more="" href=(href)>(t!(loading))</a>
            </div>
        }
    })
}

/// A named list of posts with its first page, the sentinel and, when there is nothing to show, the
/// `empty` content (usually an `empty_state`).
#[component]
pub(crate) async fn post_list(
    cx: &Cx,
    label: &str,
    posts: &[PostDto],
    #[default] more: Option<&str>,
    #[default] empty: Option<Child<'_>>,
) -> Result<impl View> {
    let none = posts.is_empty() && more.is_none();
    Ok(view! {
        <div class="list" role="list" aria-label=(label) data-c="post-list">
            post_items(posts: posts, more: more)
        </div>
        if none {
            if let Some(empty) = empty {
                <div data-empty="">(empty)</div>
            }
        }
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn a_page_is_a_named_list_ending_in_the_sentinel() {
        let html = render("en", |cx| {
            view! {
                post_list(label: "Timeline", posts: &[], more: Some("/?before=abc"))
            }
            .boxed()
        });
        assert!(
            html.contains(r#"role="list""#) && html.contains(r#"aria-label="Timeline""#),
            "{html}"
        );
        assert_eq!(html.matches(r#"role="listitem""#).count(), 1, "{html}");
        assert!(
            html.contains("data-more-host") && html.contains(r#"href="/?before=abc""#),
            "{html}"
        );
        assert!(!html.contains("data-empty"), "{html}");
    }

    #[test]
    fn the_last_page_has_no_sentinel_and_an_empty_one_shows_the_empty_content() {
        let html = render("en", |cx| {
            view! {
                post_list(
                    label: "Timeline",
                    posts: &[],
                    empty: Some(view! { <p>"nothing"</p> }.into())
                )
            }
            .boxed()
        });
        assert!(!html.contains("data-more"), "{html}");
        assert!(html.contains("data-empty"), "{html}");
    }
}
