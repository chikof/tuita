//! A poll under a post.

use time::OffsetDateTime;
use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, class, component, view},
};
use tuita_server::dto::PostDto;

use crate::{
    icons::Icon,
    l10n::{strip_isolates, t},
    post::{
        poll_math::{Left, leaders, shares, shows_results, time_left},
        rich_text::emote_text,
    },
    session,
    ui::{
        button::{ButtonSize, button},
        dither_fill::dither_fill,
        icon::icon,
    },
};

/// The footer line: votes, time left or `final`, and whether several answers are allowed.
fn footer(cx: &Cx, post: &PostDto) -> String {
    let Some(poll) = &post.poll else {
        return String::new();
    };
    let votes = strip_isolates(
        &t!(
            poll_votes,
            count: poll.voters,
            compact: crate::fmt::compact(cx, i64::from(poll.voters)),
        )
        .resolve(cx),
    );
    let state = if poll.closed {
        t!(poll_final).resolve(cx).into_owned()
    } else {
        match time_left(poll.ends_at, OffsetDateTime::now_utc()) {
            Left::Minutes(n) => strip_isolates(&t!(poll_left_minutes, n: n).resolve(cx)),
            Left::Hours(n) => strip_isolates(&t!(poll_left_hours, n: n).resolve(cx)),
            Left::Days(n) => strip_isolates(&t!(poll_left_days, n: n).resolve(cx)),
        }
    };
    let multiple = if poll.multiple {
        format!(" · {}", t!(poll_multiple).resolve(cx))
    } else {
        String::new()
    };
    format!("{votes} · {state}{multiple}")
}

/// The poll of `post`: answer buttons, or the tally once the viewer voted, the poll ended or the
/// viewer wrote it.
///
/// Voting is `post.js`: a click on `[data-act="vote"]` posts to `/api/v1/posts/<id>/vote` (a
/// multiple-choice poll collects ticks first and `[data-act="vote-submit"]` sends them) and swaps
/// the whole `.poll` element for `/_tuita/poll/<id>`.
#[component]
pub(crate) async fn poll(cx: &Cx, post: &PostDto) -> Result<impl View> {
    let mine = session::viewer(cx)
        .await?
        .as_ref()
        .is_some_and(|viewer| viewer.me.user.id == post.author.id);
    Ok(view! { poll_view(post: post, mine: mine) })
}

/// [`poll`] for a known author.
#[component]
pub(crate) async fn poll_view(cx: &Cx, post: &PostDto, mine: bool) -> Result<impl View> {
    let foot = footer(cx, post);
    let pick = t!(poll_vote).resolve(cx).into_owned();
    let your_vote = t!(poll_your_vote).resolve(cx).into_owned();
    let data = post.poll.as_ref();
    let results = data.is_some_and(|data| shows_results(data, mine));
    let multiple = data.is_some_and(|data| data.multiple);
    let percent = data.map(shares).unwrap_or_default();
    let top = data
        .filter(|data| data.closed)
        .map(leaders)
        .unwrap_or_default();
    let rows: Vec<_> = data
        .into_iter()
        .flat_map(|data| data.options.iter().map(move |option| (data, option)))
        .enumerate()
        .map(|(index, (data, option))| {
            let voted = data
                .voted
                .iter()
                .any(|picked| usize::from(*picked) == index);
            (
                index,
                &option.text,
                voted,
                top.contains(&index),
                percent[index],
            )
        })
        .collect();
    Ok(view! {
        <div
            class="poll"
            data-poll=(post.id.to_string())
            data-multiple=(multiple.then_some(""))
            data-c="poll"
        >
            <ul>
                for (index, text, voted, winner, pct) in rows {
                    <li>
                        if results {
                            <div
                                class=(class!("row", "voted" if voted, "winner" if winner))
                            >
                                <span class="label" translate="no">
                                    emote_text(text: text, emojis: &post.emojis)
                                    if voted {
                                        icon(name: Icon::Check, size: 16)
                                        <span class="sr-only">(&your_vote)</span>
                                    }
                                </span>
                                <span class="pct">
                                    (pct)
                                    "%"
                                </span>
                                <span class="bar" aria-hidden="true">
                                    <span class="fill" style=(format!("width:{pct}%"))>
                                        dither_fill(level: if voted { 1.0 } else { 0.5 })
                                    </span>
                                </span>
                            </div>
                        } else {
                            <button
                                type="button"
                                class="choice"
                                data-act="vote"
                                data-index=(index.to_string())
                                aria-pressed=(multiple.then_some("false"))
                            >
                                <span translate="no">
                                    emote_text(text: text, emojis: &post.emojis)
                                </span>
                                <span class="tick" hidden="" aria-hidden="true">
                                    icon(name: Icon::Check, size: 16)
                                </span>
                            </button>
                        }
                    </li>
                }
            </ul>
            if !results && multiple {
                button(
                    size: ButtonSize::Sm,
                    disabled: true,
                    attrs: attributes! { data-act="vote-submit" },
                    (&pick)
                )
            }
            <p class="foot">(&foot)</p>
        </div>
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::post::testing::{sample_poll, sample_post};

    #[test]
    fn the_footer_counts_votes_and_names_the_state() {
        let mut post = sample_post();
        let mut data = sample_poll();
        data.closed = true;
        post.poll = Some(data);
        let cx = crate::testing::context("en");
        assert_eq!(footer(&cx, &post), "7 votes · Final results");
    }
}
