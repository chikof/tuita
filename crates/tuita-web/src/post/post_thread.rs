//! A post with its ancestors and replies.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_server::dto::{PostDto, ReplyPolicy, ThreadDto};

use crate::{
    compose::composer::composer,
    fmt::compact,
    l10n::{strip_isolates, t},
    post::post_card::{ThreadLine, post_card},
    shell::page_bar::page_bar,
};

/// The post kinds the stats of a post list: reposts, quotes and likes.
#[derive(Clone, Copy)]
pub(crate) enum ListKind {
    Reposts,
    Quotes,
    Likes,
}

impl ListKind {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Reposts => "reposts",
            Self::Quotes => "quotes",
            Self::Likes => "likes",
        }
    }
}

/// Why the viewer cannot reply, when they cannot.
pub(crate) fn reply_note(cx: &Cx, post: &PostDto) -> Option<String> {
    if post.can_reply {
        return None;
    }
    match post.reply_policy {
        ReplyPolicy::Following => Some(t!(post_reply_following_only).resolve(cx).into_owned()),
        ReplyPolicy::Mentioned => Some(t!(post_reply_mentioned_only).resolve(cx).into_owned()),
        ReplyPolicy::Everyone => None,
    }
}

/// One entry of the stats row: a label with its number.
struct Stat {
    kind: ListKind,
    number: String,
    label: String,
}

/// The views and the reposts, quotes and likes of a post, as a row under its text. The counts
/// that are not zero are buttons that open the list of who did it (`post.js`,
/// `data-act="post-list"`).
#[component]
pub(crate) async fn post_stats(cx: &Cx, post: &PostDto) -> Result<impl View> {
    let counts = post.counts;
    let views = counts.views;
    let views_i32 = i32::try_from(views).unwrap_or(i32::MAX);
    let views_number = compact(cx, views);
    let views_label = strip_isolates(&t!(thread_views, count: views_i32).resolve(cx));
    let stats: Vec<Stat> = [
        (
            ListKind::Reposts,
            counts.reposts,
            strip_isolates(&t!(thread_reposts, count: counts.reposts).resolve(cx)),
        ),
        (
            ListKind::Quotes,
            counts.quotes,
            strip_isolates(&t!(thread_quotes, count: counts.quotes).resolve(cx)),
        ),
        (
            ListKind::Likes,
            counts.likes,
            strip_isolates(&t!(thread_likes, count: counts.likes).resolve(cx)),
        ),
    ]
    .into_iter()
    .filter(|(_, n, _)| *n > 0)
    .map(|(kind, n, label)| Stat {
        kind,
        number: compact(cx, i64::from(n)),
        label,
    })
    .collect();
    let id = post.id.to_string();
    Ok(view! {
        if views > 0 || !stats.is_empty() {
            <div
                class="stats"
                role="group"
                aria-label=(t!(thread_stats))
                data-c="post-stats"
            >
                if views > 0 {
                    <span class="stat">
                        <b>(&views_number)</b>
                        " "
                        (&views_label)
                    </span>
                }
                for stat in stats {
                    <button
                        type="button"
                        class="stat link"
                        data-act="post-list"
                        data-kind=(stat.kind.name())
                        data-post-id=(&id)
                    >
                        <b>(stat.number)</b>
                        " "
                        (stat.label)
                    </button>
                }
            </div>
        }
    })
}

/// The thread of a post: the posts above it, the post itself, its replies and the replies its
/// author hid.
///
/// `bar` draws the page header; the lightbox panel and the gallery have their own.
#[component]
pub(crate) async fn post_thread(
    cx: &Cx,
    thread: &ThreadDto,
    #[default(true)] bar: bool,
) -> Result<impl View> {
    let post = &thread.post;
    let note = reply_note(cx, post);
    let title = t!(thread_title).resolve(cx).into_owned();
    let hidden = strip_isolates(
        &t!(
            thread_hidden,
            count: i32::try_from(thread.hidden_replies.len()).unwrap_or(i32::MAX),
        )
        .resolve(cx),
    );
    Ok(view! {
        <div class="thread" data-c="post-thread">
            if bar {
                page_bar(title: &title, back: true)
            }
            <div class="ancestors" role="list">
                #[key(ancestor.entry_id.as_bytes())]
                for ancestor in &thread.ancestors {
                    <div role="listitem" class="row">
                        post_card(
                            post: ancestor,
                            thread: Some(ThreadLine::Down),
                            hide_reply_context: true
                        )
                    </div>
                }
            </div>
            <div
                class="main row"
                data-scroll-to=((!thread.ancestors.is_empty()).then_some(""))
            >
                post_card(
                    post: post,
                    main: true,
                    footer: Some(view! { post_stats(post: post) }.into())
                )
                if post.can_reply {
                    <div class="compose">composer(reply_to: Some(post))</div>
                } else if let Some(note) = &note {
                    <p class="note">(note)</p>
                }
            </div>
            <div
                role="list"
                aria-label=(t!(thread_replies))
                data-replies=(post.id.to_string())
                hidden=(thread.replies.is_empty())
            >
                #[key(reply.entry_id.as_bytes())]
                for reply in &thread.replies {
                    <div role="listitem" class="row">
                        post_card(post: reply, hide_reply_context: true)
                    </div>
                }
            </div>
            if !thread.hidden_replies.is_empty() {
                <details class="hidden">
                    <summary>(&hidden)</summary>
                    <p class="note">(t!(thread_hidden_note))</p>
                    <div role="list">
                        #[key(reply.entry_id.as_bytes())]
                        for reply in &thread.hidden_replies {
                            <div role="listitem" class="row">
                                post_card(post: reply, hide_reply_context: true)
                            </div>
                        }
                    </div>
                </details>
            }
        </div>
    })
}
