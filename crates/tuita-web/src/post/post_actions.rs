//! The action bar under a post.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, class, component, element_id, view},
};
use tuita_server::dto::{MediaKind, PostDto, ReplyPolicy};

use crate::{
    fmt::compact,
    icons::Icon,
    l10n::{strip_isolates, t},
    post::{heart::heart, post_path, share_text},
    ui::{
        icon::icon,
        menu::{MenuItem, menu},
    },
};

/// Why a viewer who may not reply cannot.
fn reply_note(cx: &Cx, post: &PostDto) -> String {
    if post.reply_policy == ReplyPolicy::Following {
        t!(post_reply_following_only).resolve(cx).into_owned()
    } else {
        t!(post_reply_mentioned_only).resolve(cx).into_owned()
    }
}

/// The count next to an icon: nothing on the big bar and for zero, else compact.
fn count_text(cx: &Cx, big: bool, count: i32) -> String {
    if big || count == 0 {
        String::new()
    } else {
        compact(cx, i64::from(count))
    }
}

/// The labels of the repost menu.
struct RepostLabels {
    repost: String,
    undo: String,
    quote: String,
}

/// The entries of the repost menu: toggle the repost or quote the post.
///
/// The repost entry carries both of its labels so `post.js` can swap them after a toggle.
fn repost_items<'a>(
    cx: &Cx,
    id: &str,
    reposted: bool,
    labels: &'a RepostLabels,
) -> Vec<MenuItem<'a>> {
    vec![
        MenuItem {
            id: "repost",
            label: if reposted {
                &labels.undo
            } else {
                &labels.repost
            },
            attrs: attributes! {
                data-act="repost"
                data-post-id=(id)
                data-on=(&labels.undo)
                data-off=(&labels.repost)
            },
            ..MenuItem::default()
        },
        MenuItem {
            id: "quote",
            label: &labels.quote,
            attrs: attributes! { data-compose="quote" data-post-id=(id) },
            ..MenuItem::default()
        },
    ]
}

/// The labels of the share menu.
struct ShareLabels {
    chat: String,
    copy: String,
    native: String,
    downloads: Vec<String>,
}

/// The entries of the share menu: chat, copy the link, the native share sheet (hidden until
/// `share.js` finds `navigator.share`) and one download per attachment.
fn share_items<'a>(
    cx: &Cx,
    post: &'a PostDto,
    labels: &'a ShareLabels,
    path: &str,
) -> Vec<MenuItem<'a>> {
    let id = post.id.to_string();
    let mut items = vec![
        MenuItem {
            id: "chat",
            label: &labels.chat,
            icon: Some(Icon::Message),
            attrs: attributes! { data-act="share-chat" data-post-id=(&id) },
            ..MenuItem::default()
        },
        MenuItem {
            id: "link",
            label: &labels.copy,
            icon: Some(Icon::Link),
            attrs: attributes! { data-act="copy-link" data-url=(path) },
            ..MenuItem::default()
        },
        MenuItem {
            id: "native",
            label: &labels.native,
            icon: Some(Icon::Share),
            attrs: attributes! {
                data-act="share"
                data-url=(path)
                data-text=(share_text(post))
                data-needs-share=""
                hidden=""
            },
            ..MenuItem::default()
        },
    ];
    for (index, (media, label)) in post.media.iter().zip(&labels.downloads).enumerate() {
        let kind = match media.kind {
            MediaKind::Image => "image",
            MediaKind::Video => "video",
        };
        items.push(MenuItem {
            id: "download",
            label,
            icon: Some(Icon::Download),
            separator: index == 0,
            attrs: attributes! {
                data-act="download"
                data-url=(&media.url)
                data-media-id=(media.id.to_string())
                data-kind=(kind)
            },
            ..MenuItem::default()
        });
    }
    items
}

/// Reply, repost, like, views, bookmark and share.
///
/// Like, repost and bookmark are handled by `post.js` on `[data-action]` and `[data-act]`: it flips
/// the state at once, calls the API, takes the answer's numbers and rolls back with a toast when
/// the API refuses. The root keeps the state in `data-*` so every bar of the same post (they carry
/// `data-post-id`) updates together. The reply button and the quote item carry `data-compose` for
/// the compose dialog.
#[component]
pub(crate) async fn post_actions(
    cx: &Cx,
    post: &PostDto,
    #[default] big: bool,
) -> Result<impl View> {
    let uid = element_id(cx);
    let id = post.id.to_string();
    let reposts = post.counts.reposts + post.counts.quotes;
    let views = post.counts.views;
    let views_i32 = i32::try_from(views).unwrap_or(i32::MAX);
    let can_reply = post.can_reply;
    let note = reply_note(cx, post);
    let path = post_path(&post.author, post.id);

    let reply_label = strip_isolates(&t!(post_reply_label, count: post.counts.replies).resolve(cx));
    let reposts_label = strip_isolates(&t!(post_reposts_label, count: reposts).resolve(cx));
    let like_label = strip_isolates(&t!(post_like_label, count: post.counts.likes).resolve(cx));
    let views_text = compact(cx, views);
    let views_label = format!(
        "{views_text} {}",
        strip_isolates(&t!(thread_views, count: views_i32).resolve(cx))
    );
    let reply_count = count_text(cx, big, post.counts.replies);
    let repost_count = count_text(cx, big, reposts);
    let like_count = count_text(cx, big, post.counts.likes);

    let repost_menu_label = t!(post_repost_menu).resolve(cx).into_owned();
    let share_label = t!(post_share).resolve(cx).into_owned();
    let share_sr = share_label.clone();
    let bookmark_label = t!(post_bookmark_label).resolve(cx).into_owned();
    let repost_labels = RepostLabels {
        repost: t!(post_repost).resolve(cx).into_owned(),
        undo: t!(post_undo_repost).resolve(cx).into_owned(),
        quote: t!(post_quote).resolve(cx).into_owned(),
    };
    let share_labels = ShareLabels {
        chat: t!(share_send_chat).resolve(cx).into_owned(),
        copy: t!(post_copy_link).resolve(cx).into_owned(),
        native: t!(post_share_menu).resolve(cx).into_owned(),
        downloads: (0..post.media.len())
            .map(|index| {
                if post.media.len() > 1 {
                    strip_isolates(&t!(share_download_n, number: index + 1).resolve(cx))
                } else {
                    t!(share_download).resolve(cx).into_owned()
                }
            })
            .collect(),
    };
    let repost_id = format!("repost-{uid}");
    let share_id = format!("share-{uid}");

    Ok(view! {
        <div
            class=(class!("actions", "big" if big))
            data-c="post-actions"
            data-post-id=(&id)
            data-liked=(post.viewer.liked.to_string())
            data-reposted=(post.viewer.reposted.to_string())
            data-bookmarked=(post.viewer.bookmarked.to_string())
            data-likes=(post.counts.likes.to_string())
            data-reposts=(reposts.to_string())
            data-big=(big.to_string())
        >
            <div class="main">
                <button
                    type="button"
                    class="act"
                    data-action="reply"
                    data-compose=(can_reply.then_some("reply"))
                    data-post-id=(&id)
                    aria-disabled=((!can_reply).then_some("true"))
                    title=((!can_reply).then_some(&note))
                    aria-label=(&reply_label)
                >
                    icon(name: Icon::Message, size: 20)
                    <span class="count">(reply_count)</span>
                </button>
                menu(
                    id: &repost_id,
                    label: &repost_menu_label,
                    items: repost_items(cx, &id, post.viewer.reposted, &repost_labels),
                    plain: true,
                    trigger: view! {
                        <span
                            class=(class!("act", "on" if post.viewer.reposted))
                            data-action="repost"
                        >
                            icon(name: Icon::Repeat, size: 20)
                            <span class="count">(repost_count)</span>
                            <span class="sr-only">(&reposts_label)</span>
                        </span>
                    }.into(

                    )
                )
                <button
                    type="button"
                    class=(class!("act", "like", "on" if post.viewer.liked))
                    data-action="like"
                    aria-pressed=(post.viewer.liked.to_string())
                    aria-label=(&like_label)
                >
                    heart(filled: post.viewer.liked)
                    <span class="count">(like_count)</span>
                </button>
                if !big && views > 0 {
                    <span class="act views" role="img" aria-label=(&views_label)>
                        icon(name: Icon::Analytics, size: 20)
                        <span class="count">(&views_text)</span>
                    </span>
                }
            </div>
            <div class="end">
                <button
                    type="button"
                    class=(class!("act", "on" if post.viewer.bookmarked))
                    data-action="bookmark"
                    aria-pressed=(post.viewer.bookmarked.to_string())
                    aria-label=(&bookmark_label)
                >
                    icon(name: Icon::Bookmark, size: 20)
                </button>
                menu(
                    id: &share_id,
                    label: &share_label,
                    items: share_items(cx, post, &share_labels, &path),
                    plain: true,
                    trigger: view! {
                        <span class="act" data-action="share">
                            icon(name: Icon::Share, size: 20)
                            <span class="sr-only">(&share_sr)</span>
                        </span>
                    }.into(

                    )
                )
            </div>
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::{post::testing::sample_post, testing::render};

    #[component]
    async fn demo(cx: &Cx, big: bool) -> Result<impl View> {
        let mut post = sample_post();
        post.counts.likes = 1;
        post.counts.replies = 3;
        post.counts.views = 1200;
        post.viewer.liked = true;
        Ok(view! { post_actions(post: &post, big: big) })
    }

    #[test]
    fn the_bar_names_every_action_and_its_count() {
        let html = render("en", |cx| view! { demo(big: false) }.boxed());
        assert!(html.contains(r#"class="actions""#), "{html}");
        assert!(html.contains(r#"aria-label="Reply (3)""#), "{html}");
        assert!(html.contains(r#"aria-label="Like (1)""#), "{html}");
        assert!(html.contains(r#"aria-label="Bookmark""#), "{html}");
        assert!(html.contains(r#"aria-pressed="true""#), "{html}");
        assert!(html.contains(r#"data-action="like""#), "{html}");
        assert!(html.contains(r#"class="act like on""#), "{html}");
        assert!(html.contains(r#"class="act views""#), "{html}");
        assert!(html.contains("1.2K Views"), "{html}");
        assert!(html.contains(r#"data-compose="reply""#), "{html}");
        assert!(html.contains(r#"data-compose="quote""#), "{html}");
        assert!(html.contains(r#"aria-label="Repost options""#), "{html}");
        assert!(
            html.contains(r#"aria-label="Share or copy link""#),
            "{html}"
        );
    }

    #[test]
    fn the_big_bar_shows_no_counts_and_no_views() {
        let html = render("en", |cx| view! { demo(big: true) }.boxed());
        assert!(html.contains(r#"class="actions big""#), "{html}");
        assert!(!html.contains("act views"), "{html}");
        assert!(html.contains(r#"<span class="count"></span>"#), "{html}");
    }

    #[test]
    fn a_post_the_viewer_may_not_reply_to_says_why() {
        #[component]
        async fn closed(cx: &Cx) -> Result<impl View> {
            let mut post = sample_post();
            post.can_reply = false;
            post.reply_policy = ReplyPolicy::Following;
            Ok(view! { post_actions(post: &post) })
        }
        let html = render("en", |cx| view! { closed() }.boxed());
        assert!(html.contains(r#"aria-disabled="true""#), "{html}");
        assert!(
            html.contains("Only people the author follows can reply."),
            "{html}"
        );
        assert!(!html.contains(r#"data-compose="reply""#), "{html}");
    }
}
