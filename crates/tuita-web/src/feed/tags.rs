//! `/tags/{tag}`: the posts that use a hashtag.

use topcoat::{
    Result,
    context::Cx,
    router::query_params,
    view::{View, component, view},
};
use tuita_server::dto::{Page, PostDto};

use super::list::{more_href, post_items, post_list};
use crate::{
    api::{self, enc},
    fragment,
    l10n::t,
    shell::page_bar::page_bar,
    ui::empty_state::empty_state,
};

#[query_params(error = bad_request)]
struct TagQuery {
    before: Option<String>,
}

/// The tag as shown: lowercase, with the `#`.
#[must_use]
pub(crate) fn label(tag: &str) -> String {
    format!("#{}", tag.to_lowercase())
}

/// The page of `tag`: a bar with the tag and one infinite list.
#[component]
pub(crate) async fn tag_posts(cx: &Cx, tag: &str) -> Result<impl View> {
    let query = query_params::<TagQuery>(cx)?;
    let name = tag.to_lowercase();
    let label = label(tag);
    let path = format!(
        "/tags/{}/posts{}",
        enc(&name),
        api::query(&[("before", query.before.as_deref())])
    );
    let page = api::get::<Page<PostDto>>(cx, &path).await?;
    let more = page
        .next
        .as_deref()
        .map(|cursor| more_href(cx, &[], cursor));
    let list_label = t!(tag_posts_label, tag: label.as_str())
        .resolve(cx)
        .into_owned();
    let empty_title = t!(tag_empty_title, tag: label.as_str())
        .resolve(cx)
        .into_owned();
    let empty_body = t!(tag_empty_body).resolve(cx).into_owned();
    let context = format!("tag:{name}");
    Ok(view! {
        if fragment::requested(cx) {
            post_items(posts: &page.items, more: more.as_deref())
        } else {
            page_bar(back: true, title: &label)
            post_list(
                label: &list_label,
                posts: &page.items,
                more: more.as_deref(),
                empty: Some(
                    view! {
                        empty_state(
                            context: &context,
                            title: &empty_title,
                            body: empty_body.as_str()
                        )
                    }.into(

                    ),
                )
            )
        }
    })
}
