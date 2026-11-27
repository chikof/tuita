//! Explore without a query: trends and who to follow.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_server::dto::{TrendDto, UserDto};

use crate::{
    api::{self, ApiError, enc},
    fmt::compact,
    l10n::{strip_isolates, t},
    profile::user_row::user_row,
};

/// The three tabs of the explore home.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum HomeTab {
    /// A preview of trends and people.
    #[default]
    Foryou,
    /// Every trend.
    Trending,
    /// Every suggestion.
    People,
}

impl HomeTab {
    /// The tab a `?tab=` value names; anything else is the first.
    pub(crate) fn parse(text: Option<&str>) -> Self {
        match text {
            Some("trending") => Self::Trending,
            Some("people") => Self::People,
            _ => Self::Foryou,
        }
    }

    /// The tab's id, as in `?tab=`.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Foryou => "foryou",
            Self::Trending => "trending",
            Self::People => "people",
        }
    }
}

/// How many entries the `foryou` preview shows of each list.
const PREVIEW: usize = 5;
/// How many accounts the people tab asks for.
const SUGGESTIONS: &str = "20";

/// One trend as the page draws it.
struct Trend {
    rank: usize,
    href: String,
    tag: String,
    posts: String,
}

/// The sections of `tab`; a failed list shows its error where the list would be.
#[component]
pub(crate) async fn explore_home(cx: &Cx, tab: HomeTab) -> Result<impl View> {
    let preview = tab == HomeTab::Foryou;
    let trends: Option<Result<Vec<TrendDto>, ApiError>> = if tab == HomeTab::People {
        None
    } else {
        Some(api::get::<Vec<TrendDto>>(cx, "/trends").await)
    };
    let people: Option<Result<Vec<UserDto>, ApiError>> = if tab == HomeTab::Trending {
        None
    } else {
        let limit = if preview { "5" } else { SUGGESTIONS };
        let query = api::query(&[("limit", Some(limit))]);
        Some(api::get::<Vec<UserDto>>(cx, &format!("/suggestions{query}")).await)
    };
    let more_trends = trends
        .as_ref()
        .is_some_and(|list| preview && list.as_ref().is_ok_and(|list| list.len() > PREVIEW));
    let trend_rows = |list: Vec<TrendDto>| -> Vec<Trend> {
        list.into_iter()
            .take(if preview { PREVIEW } else { usize::MAX })
            .enumerate()
            .map(|(at, trend)| {
                let count = i32::try_from(trend.posts).unwrap_or(i32::MAX);
                Trend {
                    rank: at + 1,
                    href: format!("/tags/{}", enc(&trend.tag)),
                    posts: strip_isolates(
                        &t!(
                            explore_trend_posts,
                            count: count,
                            compact: compact(cx, trend.posts),
                        )
                        .resolve(cx),
                    ),
                    tag: trend.tag,
                }
            })
            .collect()
    };
    let (trend_list, trend_error) = match trends {
        Some(Ok(list)) => (Some(trend_rows(list)), None),
        Some(Err(error)) => (None, Some(error.message)),
        None => (None, None),
    };
    let (people_list, people_error) = match people {
        Some(Ok(list)) => (Some(list), None),
        Some(Err(error)) => (None, Some(error.message)),
        None => (None, None),
    };
    let shows_trends = trend_list.is_some() || trend_error.is_some();
    let shows_people = people_list.is_some() || people_error.is_some();

    Ok(view! {
        if shows_trends {
            <section aria-labelledby="explore-trends" data-c="explore-home">
                <div class="head">
                    <h2 id="explore-trends">(t!(explore_trending))</h2>
                </div>
                if let Some(list) = trend_list {
                    if list.is_empty() {
                        <p class="note">(t!(explore_trends_empty))</p>
                    } else {
                        <ul>
                            for trend in list {
                                <li>
                                    <a class="trend wash" href=(trend.href)>
                                        <span class="rank" aria-hidden="true">(trend.rank)</span>
                                        <span class="label">
                                            "#"
                                            (trend.tag)
                                        </span>
                                        <span class="count">(trend.posts)</span>
                                    </a>
                                </li>
                            }
                        </ul>
                        if more_trends {
                            <a class="more wash" href="/explore?tab=trending">
                                <span>(t!(explore_see_more))</span>
                            </a>
                        }
                    }
                }
                if let Some(message) = trend_error {
                    <p class="note failed" role="alert">(message)</p>
                }
            </section>
        }
        if shows_people {
            <section aria-labelledby="explore-people" data-c="explore-home">
                <div class="head">
                    <h2 id="explore-people">(t!(explore_suggestions))</h2>
                </div>
                if let Some(list) = &people_list {
                    if list.is_empty() {
                        <p class="note">(t!(explore_suggestions_empty))</p>
                    } else {
                        <ul>
                            for user in list {
                                <li>user_row(user: user)</li>
                            }
                        </ul>
                        if preview {
                            <a class="more wash" href="/explore?tab=people">
                                <span>(t!(explore_see_more))</span>
                            </a>
                        }
                    }
                }
                if let Some(message) = people_error {
                    <p class="note failed" role="alert">(message)</p>
                }
            </section>
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn an_unknown_tab_is_the_first() {
        assert_eq!(HomeTab::parse(None), HomeTab::Foryou);
        assert_eq!(HomeTab::parse(Some("people")), HomeTab::People);
        assert_eq!(HomeTab::parse(Some("trending")), HomeTab::Trending);
        assert_eq!(HomeTab::parse(Some("x")), HomeTab::Foryou);
        assert_eq!(HomeTab::People.name(), "people");
    }
}
