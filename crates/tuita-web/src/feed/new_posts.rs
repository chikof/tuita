//! `GET /_tuita/new-posts?feed=&since=`: the "show new posts" pill, or nothing (`204`).
//!
//! `newposts.js` polls it and puts the answer into the pill slot of the home page.

use topcoat::{
    Result,
    context::Cx,
    router::{StatusCode, query_params, route},
    view::{View, ViewExt, ViewHandle, component, view},
};
use tuita_server::dto::NewCountDto;

use super::Feed;
use crate::{api, icons::Icon, l10n::t, ui::icon::icon};

#[query_params(error = bad_request)]
struct NewQuery {
    feed: Option<String>,
    since: Option<String>,
}

/// The pill: "Show 3 new posts", or "Show 99+ new posts" past 99. `newposts.js` handles its click.
#[component]
pub(crate) async fn new_posts_pill(cx: &Cx, count: i64) -> Result<impl View> {
    let text = if count > 99 {
        t!(new_posts_many).resolve(cx).into_owned()
    } else {
        t!(new_posts, count: count).resolve(cx).into_owned()
    };
    Ok(view! {
        <div class="wrap" data-c="new-posts-pill">
            <button type="button" class="pill">
                icon(name: Icon::ChevronUp, size: 16)
                (text)
            </button>
        </div>
    })
}

#[route(GET "/_tuita/new-posts")]
async fn new_posts(cx: &Cx) -> Result<ViewHandle> {
    let query = query_params::<NewQuery>(cx)?;
    let feed = query
        .feed
        .as_deref()
        .and_then(Feed::parse)
        .unwrap_or(Feed::ForYou);
    let path = format!(
        "/timelines/{}/new{}",
        feed.name(),
        api::query(&[("since", query.since.as_deref())])
    );
    let count = api::get::<NewCountDto>(cx, &path).await?.count;
    if count <= 0 {
        return view! { (StatusCode::NO_CONTENT) }.first().await;
    }
    Box::pin(view! { new_posts_pill(count: count) }.first()).await
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    #[test]
    fn the_pill_counts_the_new_posts() {
        let one = render("en", |cx| view! { new_posts_pill(count: 1) }.boxed());
        assert!(one.contains("Show \u{2068}1\u{2069} new post<"), "{one}");
        let three = render("en", |cx| view! { new_posts_pill(count: 3) }.boxed());
        assert!(three.contains("new posts"), "{three}");
        assert!(three.contains(r#"class="pill""#), "{three}");
    }

    #[test]
    fn past_ninety_nine_the_pill_says_so() {
        let html = render("en", |cx| view! { new_posts_pill(count: 150) }.boxed());
        assert!(html.contains("Show 99+ new posts"), "{html}");
    }
}
