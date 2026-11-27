//! `/explore`: search and discovery.
//!
//! Without `?q=` it is the home (`?tab=foryou|trending|people`); with it, the results of a search
//! (`&type=posts|people|media`). `?before=` with `x-tuita-fragment: 1` answers one more page.

use topcoat::{
    Result,
    context::Cx,
    metadata::Metadata,
    router::{page, query_params},
    view::{View, ViewExt, view},
};
use tuita_server::dto::SearchType;

use crate::{
    api::enc,
    explore::{
        home::{HomeTab, explore_home},
        results::{explore_results, results_fragment, type_of},
        search::explore_search,
    },
    fragment,
    l10n::t,
    session, shell,
    shell::page_bar::page_bar,
    ui::link_tabs::{LinkTab, link_tabs},
};

#[query_params(error = bad_request)]
struct ExploreQuery {
    q: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    tab: Option<String>,
    before: Option<String>,
}

/// The labels of the tabs of both modes.
struct Labels {
    posts: String,
    people: String,
    media: String,
    foryou: String,
    trending: String,
}

/// The tabs under the search field: result types for a query, else the home tabs.
fn tab_items<'a>(q: &str, labels: &'a Labels) -> Vec<LinkTab<'a>> {
    if q.is_empty() {
        vec![
            LinkTab {
                id: "foryou",
                label: &labels.foryou,
                href: "/explore".to_owned(),
            },
            LinkTab {
                id: "trending",
                label: &labels.trending,
                href: "/explore?tab=trending".to_owned(),
            },
            LinkTab {
                id: "people",
                label: &labels.people,
                href: "/explore?tab=people".to_owned(),
            },
        ]
    } else {
        let base = format!("/explore?q={}", enc(q));
        vec![
            LinkTab {
                id: "posts",
                label: &labels.posts,
                href: base.clone(),
            },
            LinkTab {
                id: "people",
                label: &labels.people,
                href: format!("{base}&type=people"),
            },
            LinkTab {
                id: "media",
                label: &labels.media,
                href: format!("{base}&type=media"),
            },
        ]
    }
}

#[page(
    with(Metadata::from_fn(
        async |cx| {
            let title = t!(nav_explore).resolve(cx).into_owned();
            shell::titled(cx, &title).await
        },
    ))
)]
async fn explore_page(cx: &Cx) -> Result<impl View> {
    let viewer = session::member(cx).await?;
    let query = query_params::<ExploreQuery>(cx)?;
    let q = query.q.as_deref().unwrap_or("").trim();
    let kind = type_of(query.kind.as_deref());
    if fragment::requested(cx) && !q.is_empty() {
        return Ok(view! {
            results_fragment(q: q, kind: kind, before: query.before.as_deref())
        }
        .boxed());
    }
    let home = HomeTab::parse(query.tab.as_deref());
    let user_id = viewer.me.user.id.to_string();
    let title = t!(nav_explore).resolve(cx).into_owned();
    let labels = Labels {
        posts: t!(explore_tab_posts).resolve(cx).into_owned(),
        people: t!(explore_tab_people).resolve(cx).into_owned(),
        media: t!(explore_tab_media).resolve(cx).into_owned(),
        foryou: t!(home_for_you).resolve(cx).into_owned(),
        trending: t!(explore_trending).resolve(cx).into_owned(),
    };
    let tabs_label = if q.is_empty() {
        t!(explore_sections).resolve(cx).into_owned()
    } else {
        t!(explore_results).resolve(cx).into_owned()
    };
    let selected = if q.is_empty() {
        home.name()
    } else {
        match kind {
            SearchType::Posts => "posts",
            SearchType::People => "people",
            SearchType::Media => "media",
        }
    };
    Ok(view! {
        page_bar(
            title: &title,
            below: Some(
                view! {
                    explore_search(q: q, user_id: &user_id)
                    link_tabs(
                        items: tab_items(q, &labels),
                        selected: selected,
                        label: &tabs_label
                    )
                }.into(

                ),
            )
        )
        if q.is_empty() {
            explore_home(tab: home)
        } else {
            explore_results(q: q, kind: kind, before: query.before.as_deref())
        }
    }
    .boxed())
}
