//! The member home page: the post box, the two timelines as tabs, the
//! "show new posts" pill and pull to refresh.
//!
//! The tab is the URL: `/?feed=following`. A visit with `?feed=` also stores the choice in the
//! session cookie `tuita_feed`, so `/` alone opens the feed the member used last (this replaces
//! a `sessionStorage` entry). Without either, the feed is "For you".
//!
//! `newposts.js` (`[data-island="feed"]`) polls for new posts, shows the pill, prepends a post the
//! member just wrote and swaps the list when asked to refresh; `pull.js` turns a touch pull into
//! that refresh. The markup of a first page is one `[data-feed-body]` element; the island swaps it
//! as a whole, which is what a `x-tuita-fragment` request without `before` answers.

use topcoat::{
    Result,
    context::Cx,
    router::{query_params, request::uri},
    view::{View, component, view},
};
use tuita_server::dto::{Page, PostDto};
use uuid::Uuid;

use super::{
    Feed,
    list::{more_href, post_items, post_list},
};
use crate::{
    api,
    compose::composer::composer,
    cookies, fragment,
    l10n::t,
    shell::page_bar::page_bar,
    ui::{
        empty_state::empty_state,
        link_tabs::{LinkTab, link_tabs},
    },
};

/// The session cookie that remembers the chosen feed.
pub(crate) const FEED_COOKIE: &str = "tuita_feed";

/// The mark the pill counts from: the moment this list was built, as the smallest UUID v7 of that
/// millisecond. Entry ids are v7, so "newer than the mark" is "made after the page was shown". The
/// first entry of the list is no mark: "For you" is ranked, so its first post is not its newest.
fn since_now() -> String {
    let millis =
        u128::try_from((time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000).max(0))
            .unwrap_or_default();
    Uuid::from_u128((millis << 80) | (0x7 << 76) | (0b10 << 62)).to_string()
}

#[query_params(error = bad_request)]
struct HomeQuery {
    feed: Option<String>,
    before: Option<String>,
}

/// The feed of this request: `?feed=`, else the cookie, else "For you".
fn chosen(cx: &Cx, query: &HomeQuery) -> Feed {
    query
        .feed
        .as_deref()
        .and_then(Feed::parse)
        .or_else(|| cookies::get(cx, FEED_COOKIE).and_then(Feed::parse))
        .unwrap_or(Feed::ForYou)
}

/// `GET /timelines/{feed}`: the For you feed pages by `cursor`, Following by `before`.
async fn load(cx: &Cx, feed: Feed, cursor: Option<&str>) -> Result<Page<PostDto>> {
    let path = match feed {
        Feed::ForYou => format!("/timelines/foryou{}", api::query(&[("cursor", cursor)])),
        Feed::Following => format!("/timelines/following{}", api::query(&[("before", cursor)])),
    };
    Ok(api::get::<Page<PostDto>>(cx, &path).await?)
}

/// The list of one feed with its `since` mark (see [`since_now`]) and, for an empty
/// feed, the empty state.
#[component]
async fn feed_body(
    cx: &Cx,
    feed: Feed,
    page: &Page<PostDto>,
    more: Option<&str>,
) -> Result<impl View> {
    let label = t!(home_feed).resolve(cx).into_owned();
    let since = since_now();
    let following = feed == Feed::Following;
    let title = if following {
        t!(home_following_empty_title).resolve(cx).into_owned()
    } else {
        t!(home_empty_title).resolve(cx).into_owned()
    };
    let body = if following {
        t!(home_following_empty_body).resolve(cx).into_owned()
    } else {
        t!(home_empty_body).resolve(cx).into_owned()
    };
    Ok(view! {
        <div data-feed-body="" data-since=(&since)>
            post_list(
                label: &label,
                posts: &page.items,
                more: more,
                empty: Some(
                    view! {
                        if following {
                            empty_state(
                                context: "following",
                                title: &title,
                                body: body.as_str(),
                                action: Some(
                                    view! {
                                        <a
                                            class="btn solid md wash"
                                            data-c="button"
                                            href="/explore"
                                        >
                                            <span class="label">(t!(home_find_people))</span>
                                        </a>
                                    }.into(

                                    ),
                                )
                            )
                        } else {
                            empty_state(
                                context: "foryou",
                                title: &title,
                                body: body.as_str()
                            )
                        }
                    }.into(

                    ),
                )
            )
        </div>
    })
}

/// What a member sees at `/`.
#[component]
pub(crate) async fn home(cx: &Cx) -> Result<impl View> {
    let query = query_params::<HomeQuery>(cx)?;
    let fragment = fragment::requested(cx);
    let feed = chosen(cx, query);
    if !fragment && query.feed.as_deref().and_then(Feed::parse).is_some() {
        cookies::set(cx, FEED_COOKIE, feed.name(), None);
    }
    let page = load(cx, feed, query.before.as_deref()).await?;
    let more = page
        .next
        .as_deref()
        .map(|cursor| more_href(cx, &[("feed", Some(feed.name()))], cursor));
    let paging = query.before.is_some();
    let title = t!(nav_home).resolve(cx).into_owned();
    let tabs_label = t!(home_feed).resolve(cx).into_owned();
    let for_you = t!(home_for_you).resolve(cx).into_owned();
    let following = t!(home_following).resolve(cx).into_owned();
    let path = uri(cx).path().to_owned();
    Ok(view! {
        if fragment && paging {
            post_items(posts: &page.items, more: more.as_deref())
        } else if fragment {
            feed_body(feed: feed, page: &page, more: more.as_deref())
        } else {
            <div
                data-c="home"
                data-island="feed"
                data-feed=(feed.name())
                data-stream="badges"
            >
                page_bar(
                    title: &title,
                    below: Some(
                        view! {
                            link_tabs(
                                items: vec![
                                    LinkTab {
                                        id: Feed::ForYou.name(),
                                        label: &for_you,
                                        href: format!("{path}?feed=foryou"),
                                    },
                                    LinkTab {
                                        id: Feed::Following.name(),
                                        label: &following,
                                        href: format!("{path}?feed=following"),
                                    },
                                ],
                                selected: feed.name(),
                                label: &tabs_label
                            )
                        }.into(

                        ),
                    )
                )
                <div class="compose">composer()</div>
                <div class="pill-slot" data-slot="pill"></div>
                feed_body(feed: feed, page: &page, more: more.as_deref())
            </div>
        }
    })
}

#[cfg(test)]
mod since_test {
    use super::*;

    #[test]
    fn the_mark_is_a_version_7_uuid_at_the_current_time() {
        let mark = Uuid::parse_str(&since_now()).expect("a uuid");
        assert_eq!(mark.get_version_num(), 7);
        let millis = mark.get_timestamp().expect("a v7 timestamp").to_unix();
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        assert!((now - i64::try_from(millis.0).unwrap_or_default()).abs() <= 1);
    }

    #[test]
    fn a_post_made_after_the_mark_is_newer_and_one_made_before_is_not() {
        let mark = Uuid::parse_str(&since_now()).expect("a uuid");
        std::thread::sleep(std::time::Duration::from_millis(5));
        assert!(Uuid::now_v7() > mark);
        assert!(Uuid::from_u128(mark.as_u128() - (1 << 80)) < mark);
    }
}
