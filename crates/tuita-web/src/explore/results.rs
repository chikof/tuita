//! Search results: people, posts or media for a query.
//!
//! The API does the search (including fetching a remote account or post a query names), so the
//! page waits for it and the "looking up…" line is the pending navigation.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_server::dto::{SearchDto, SearchType};

use crate::{
    api::{self},
    explore::lookup::is_lookup,
    feed::list::post_items,
    l10n::{strip_isolates, t},
    profile::list::{infinite_list, next_href, user_items},
    ui::empty_state::empty_state,
};

/// The tab a `?type=` value names; anything else is posts.
pub(crate) fn type_of(text: Option<&str>) -> SearchType {
    text.and_then(SearchType::parse)
        .unwrap_or(SearchType::Posts)
}

/// `GET /search`, the page after `before` when given.
async fn search(cx: &Cx, q: &str, kind: SearchType, before: Option<&str>) -> Result<SearchDto> {
    let query = api::query(&[
        ("q", Some(q)),
        ("type", Some(kind.as_str())),
        ("before", before),
    ]);
    Ok(api::get::<SearchDto>(cx, &format!("/search{query}")).await?)
}

/// A page of results for `infinite.js`: the rows or cards and the link to the next page.
#[component]
pub(crate) async fn results_fragment(
    cx: &Cx,
    q: &str,
    kind: SearchType,
    before: Option<&str>,
) -> Result<impl View> {
    let found = search(cx, q, kind, before).await?;
    let more = found.next.as_deref().map(|cursor| next_href(cx, cursor));
    Ok(view! {
        if kind == SearchType::People {
            user_items(users: &found.users, more: more.as_deref())
        } else {
            post_items(posts: &found.posts, more: more.as_deref())
        }
    })
}

/// The first page of results: for a posts search the few accounts that match, above the posts.
#[component]
pub(crate) async fn explore_results(
    cx: &Cx,
    q: &str,
    kind: SearchType,
    before: Option<&str>,
) -> Result<impl View> {
    let found = search(cx, q, kind, before).await?;
    let more = found.next.as_deref().map(|cursor| next_href(cx, cursor));
    let lookup = is_lookup(q);
    let people = kind == SearchType::People;
    let matches = kind == SearchType::Posts && before.is_none() && !found.users.is_empty();
    let empty = if people {
        found.users.is_empty()
    } else {
        found.posts.is_empty()
    };
    let label = if people {
        t!(explore_tab_people).resolve(cx).into_owned()
    } else if kind == SearchType::Media {
        t!(explore_tab_media).resolve(cx).into_owned()
    } else {
        t!(explore_tab_posts).resolve(cx).into_owned()
    };
    let (context, title, body) = match kind {
        SearchType::People => (
            "search-people",
            strip_isolates(&t!(explore_no_people_title, q: q).resolve(cx)),
            if lookup {
                t!(explore_lookup_empty_body).resolve(cx).into_owned()
            } else {
                t!(explore_no_people_body).resolve(cx).into_owned()
            },
        ),
        SearchType::Media => (
            "search-media",
            strip_isolates(&t!(explore_no_media_title, q: q).resolve(cx)),
            t!(explore_no_media_body).resolve(cx).into_owned(),
        ),
        SearchType::Posts => (
            "search-posts",
            strip_isolates(&t!(explore_no_posts_title, q: q).resolve(cx)),
            if lookup {
                t!(explore_lookup_empty_body).resolve(cx).into_owned()
            } else {
                t!(explore_no_posts_body).resolve(cx).into_owned()
            },
        ),
    };
    Ok(view! {
        if matches {
            <section class="matches" data-c="explore-results">
                <h2>(t!(explore_people))</h2>
                <ul>
                    for user in &found.users {
                        <li>crate::profile::user_row::user_row(user: user)</li>
                    }
                </ul>
                <h2>(t!(explore_posts))</h2>
            </section>
        }
        if people {
            infinite_list(
                label: &label,
                user_items(users: &found.users, more: more.as_deref())
            )
        } else {
            infinite_list(
                label: &label,
                post_items(posts: &found.posts, more: more.as_deref())
            )
        }
        if empty {
            empty_state(context: context, title: &title, body: body.as_str())
        }
    })
}
