//! The column right of the page on wide screens: search, trends, who to follow and the footer
//! links.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_server::dto::{TrendDto, UserDto};

use crate::{
    api,
    fmt::{acct_of, compact, handle_of},
    icons::Icon,
    l10n::t,
    paths,
    post::rich_text::user_name,
    profile::follow_button::follow_button,
    shell::parts::account_avatar,
    ui::icon::icon,
};

/// How many trends the rail lists.
const TRENDS: usize = 5;
/// How many accounts it suggests.
const PEOPLE: &str = "3";

/// A suggested account, with what the card shows copied out of the API answer.
struct Person {
    href: String,
    handle: String,
    user: UserDto,
}

impl Person {
    fn of(user: UserDto) -> Self {
        Self {
            href: paths::profile(&acct_of(&user.handle, user.domain.as_deref())),
            handle: handle_of(&user.handle, user.domain.as_deref()),
            user,
        }
    }
}

/// Trends and suggestions, each empty when its request fails: the rail is a courtesy.
async fn rail_data(cx: &Cx) -> (Vec<TrendDto>, Vec<UserDto>) {
    let suggestions = format!("/suggestions{}", api::query(&[("limit", Some(PEOPLE))]));
    let (trends, people) = tokio::join!(
        api::get::<Vec<TrendDto>>(cx, "/trends"),
        api::get::<Vec<UserDto>>(cx, &suggestions),
    );
    let mut trends = trends.unwrap_or_default();
    trends.truncate(TRENDS);
    (trends, people.unwrap_or_default())
}

/// The search form, the trends card, the suggestions card and the footer.
///
/// The search is a plain `GET /explore?q=`. The explore page carries its own search, trends and
/// suggestions, so on `/explore` the rail shows only the footer. The rail is hidden by CSS under
/// 1100px; the data is read for every page render regardless of the viewport.
#[component]
pub(crate) async fn right_rail(cx: &Cx, path: &str) -> Result<impl View> {
    let searching = path == "/explore" || path.starts_with("/explore/");
    let (trends, people) = if searching {
        (Vec::new(), Vec::new())
    } else {
        rail_data(cx).await
    };
    let trends: Vec<_> = trends
        .into_iter()
        .enumerate()
        .map(|(at, trend)| {
            let count = i32::try_from(trend.posts).unwrap_or(i32::MAX);
            let posts = t!(
                explore_trend_posts,
                count: count,
                compact: compact(cx, trend.posts),
            )
            .resolve(cx)
            .into_owned();
            (at + 1, paths::tag(&trend.tag), trend.tag, posts)
        })
        .collect();
    let people: Vec<Person> = people.into_iter().map(Person::of).collect();
    let search_label = t!(explore_search_label).resolve(cx).into_owned();
    let search_placeholder = t!(explore_search_placeholder).resolve(cx).into_owned();
    let submit_label = t!(rail_search_submit).resolve(cx).into_owned();
    Ok(view! {
        <div class="rail" data-c="right-rail">
            if !searching {
                <form class="search" role="search" action="/explore" method="get">
                    <div class="field">
                        icon(name: Icon::Search, size: 20)
                        <input
                            type="search"
                            name="q"
                            autocomplete="off"
                            enterkeyhint="search"
                            aria-label=(search_label)
                            placeholder=(search_placeholder)
                        >
                    </div>
                    <button type="submit" class="go" aria-label=(submit_label)>
                        icon(name: Icon::ArrowRight, size: 20)
                    </button>
                </form>
            }
            if !trends.is_empty() {
                <section class="card" aria-labelledby="rail-trends">
                    <h2 id="rail-trends">(t!(explore_trending))</h2>
                    <ul>
                        for (rank, href, tag, posts) in trends {
                            <li>
                                <a class="trend wash" href=(href)>
                                    <span class="rank" aria-hidden="true">(rank)</span>
                                    <span class="label">
                                        "#"
                                        (tag)
                                    </span>
                                    <span class="count">(posts)</span>
                                </a>
                            </li>
                        }
                    </ul>
                    <a class="more" href="/explore">(t!(rail_show_more))</a>
                </section>
            }
            if !people.is_empty() {
                <section class="card" aria-labelledby="rail-people">
                    <h2 id="rail-people">(t!(explore_suggestions))</h2>
                    <ul>
                        for person in people {
                            <li class="person">
                                <a
                                    class="avatar"
                                    href=(&person.href)
                                    tabindex="-1"
                                    aria-hidden="true"
                                >
                                    account_avatar(
                                        handle: &person.user.handle,
                                        src: person.user.avatar_url.as_deref(),
                                        shape: person.user.avatar_shape,
                                        ink: person.user.ink,
                                        size: 40
                                    )
                                </a>
                                <a class="who" href=(&person.href)>
                                    user_name(
                                        name: &person.user.display_name,
                                        verified: person.user.verified,
                                        bot: person.user.bot,
                                        emojis: &person.user.emojis
                                    )
                                    <span class="handle">(&person.handle)</span>
                                </a>
                                follow_button(user: &person.user)
                            </li>
                        }
                    </ul>
                    <a class="more" href="/explore">(t!(rail_show_more))</a>
                </section>
            }
            <nav class="foot" aria-label=(t!(rail_footer))>
                <a href="/terms">(t!(landing_terms))</a>
                <a href="/privacy">(t!(landing_privacy))</a>
                <a href="/developers">(t!(landing_developers))</a>
                <a href="/settings/about">(t!(settings_about))</a>
                <button type="button" command="show-modal" commandfor="shortcuts">
                    (t!(shortcuts_title))
                </button>
            </nav>
        </div>
    })
}
