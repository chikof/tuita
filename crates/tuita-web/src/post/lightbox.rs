//! The media viewer.

use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};
use tuita_server::dto::{MediaDto, MediaKind, ThreadDto};

use crate::{
    compose::composer::composer,
    fmt::{full_time, handle_of, rel_time},
    icons::Icon,
    l10n::{strip_isolates, t},
    post::{
        content_warning::content_warning,
        post_actions::post_actions,
        post_card::{post_card, shape_of},
        post_path,
        post_thread::{post_stats, reply_note},
        profile_path,
        rich_text::{rich_text, user_name},
        video_player::{VideoFit, video_player},
    },
    ui::{
        avatar::avatar,
        icon_button::{IconButtonVariant, icon_button},
    },
};

/// The id of the dialog; the close button points at it.
pub(crate) const LIGHTBOX_ID: &str = "lightbox";

/// One slide: a picture, or a video in a box of its own shape.
#[component]
async fn slide(cx: &Cx, item: &MediaDto, index: usize, current: usize) -> Result<impl View> {
    let shown = index == current;
    let alt = if item.alt.is_empty() {
        t!(media_video).resolve(cx).into_owned()
    } else {
        item.alt.clone()
    };
    let ratio = match (item.width, item.height) {
        (Some(w), Some(h)) if w > 0 && h > 0 => f64::from(w) / f64::from(h),
        _ => 16.0 / 9.0,
    };
    Ok(view! {
        <div
            class="slide"
            data-index=(index.to_string())
            data-alt=(&item.alt)
            hidden=(!shown)
        >
            if item.kind == MediaKind::Video {
                <div class="video" style=(format!("--ar:{ratio}"))>
                    video_player(
                        src: &item.url,
                        poster: item.poster_url.as_deref(),
                        alt: &alt,
                        fit: VideoFit::Contain,
                        autoplay: shown
                    )
                </div>
            } else {
                <img src=(&item.url) alt=(&item.alt) draggable="false">
            }
        </div>
    })
}

/// The post beside the picture: the post itself, its stats and actions, and its replies.
#[component]
async fn lightbox_panel(cx: &Cx, thread: &ThreadDto) -> Result<impl View> {
    let post = &thread.post;
    let author = &post.author;
    let profile = profile_path(author);
    let label = strip_isolates(&t!(post_by, name: author.display_name.as_str()).resolve(cx));
    let created = post
        .created_at
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();
    let note = reply_note(cx, post);
    let _ = full_time(cx, &created);
    Ok(view! {
        <div data-c="lightbox-panel">
            <section class="post" aria-label=(&label)>
                <header>
                    <a class="avatar" href=(&profile) tabindex="-1" aria-hidden="true">
                        avatar(
                            seed: &author.handle,
                            src: author.avatar_url.as_deref(),
                            alt: "",
                            shape: shape_of(author),
                            ink: author.ink.as_str(),
                            size: 40
                        )
                    </a>
                    <a class="who" href=(&profile)>
                        user_name(
                            name: &author.display_name,
                            verified: author.verified,
                            bot: author.bot,
                            emojis: &author.emojis
                        )
                        <span class="handle">
                            (handle_of(&author.handle, author.domain.as_deref()))
                        </span>
                    </a>
                </header>
                if !post.cw.is_empty() {
                    content_warning(
                        cw: &post.cw,
                        emojis: &post.emojis,
                        panel_text(post: post)
                    )
                } else {
                    panel_text(post: post)
                }
                <a class="time" href=(post_path(author, post.id))>
                    rel_time(ts: &created)
                </a>
                post_stats(post: post)
                post_actions(post: post, big: true)
            </section>
            if post.can_reply {
                <div class="compose">composer(reply_to: Some(post))</div>
            } else if let Some(note) = &note {
                <p class="note">(note)</p>
            }
            if !thread.replies.is_empty() {
                <div role="list" aria-label=(t!(thread_replies))>
                    #[key(reply.entry_id.as_bytes())]
                    for reply in &thread.replies {
                        <div role="listitem" class="row">
                            post_card(post: reply, hide_reply_context: true)
                        </div>
                    }
                </div>
            }
        </div>
    })
}

#[component]
async fn panel_text(cx: &Cx, post: &tuita_server::dto::PostDto) -> Result<impl View> {
    Ok(view! {
        if !post.text.is_empty() {
            <div class="text">
                rich_text(
                    segments: &post.rich_text,
                    domain: post.author.domain.as_deref(),
                    emojis: &post.emojis
                )
            </div>
        }
    })
}

/// The viewer for the media of a post, opened at `index`.
///
/// One `<dialog>` holding every slide (only the current one shown), the counter, the arrows, the
/// caption and the post panel. `lightbox.js` (`[data-island="lightbox"]`) shows it with
/// `showModal()`, steps through the slides (arrows, keys, swipe), shows the caption and counter of
/// the current slide and plays or pauses its video, and hides the panel when the viewer asks
/// to. The panel only shows from 1100 px up (CSS); below that a link to the post replaces its
/// toggle.
#[component]
pub(crate) async fn lightbox(cx: &Cx, thread: &ThreadDto, index: usize) -> Result<impl View> {
    let post = &thread.post;
    let media = &post.media;
    let count = media.len();
    let current = index.min(count.saturating_sub(1));
    let many = count > 1;
    let close = t!(close).resolve(cx).into_owned();
    let prev = t!(lightbox_prev).resolve(cx).into_owned();
    let next = t!(lightbox_next).resolve(cx).into_owned();
    let hide_panel = t!(lightbox_hide_panel).resolve(cx).into_owned();
    let counter = if many {
        format!("{} / {count}", current + 1)
    } else {
        String::new()
    };
    let caption = media
        .get(current)
        .map(|item| item.alt.clone())
        .unwrap_or_default();
    let slides: Vec<_> = media.iter().enumerate().collect();
    Ok(view! {
        <dialog
            id=(LIGHTBOX_ID)
            class="lightbox"
            aria-label=(t!(lightbox_title))
            closedby="any"
            data-island="lightbox"
            data-index=(current.to_string())
            data-count=(count.to_string())
            data-show-label=(t!(lightbox_show_panel))
            data-hide-label=(&hide_panel)
            data-c="lightbox"
        >
            <div class="frame panel">
                <div class="main">
                    <div class="top">
                        icon_button(
                            glyph: Icon::Close,
                            label: &close,
                            variant: IconButtonVariant::Outline,
                            attrs: attributes! { command="close" commandfor=(LIGHTBOX_ID) }
                        )
                        <span class="count">(counter)</span>
                        <span class="toggle">
                            icon_button(
                                glyph: Icon::ChevronRight,
                                label: &hide_panel,
                                variant: IconButtonVariant::Outline,
                                attrs: attributes! { data-act="lightbox-panel" }
                            )
                        </span>
                        <a class="view" href=(post_path(&post.author, post.id))>
                            (t!(lightbox_view_post))
                        </a>
                    </div>
                    <div class="stage" role="presentation">
                        if many {
                            <span class="prev">
                                icon_button(
                                    glyph: Icon::ChevronLeft,
                                    label: &prev,
                                    variant: IconButtonVariant::Outline,
                                    attrs: attributes! { data-act="lightbox-prev" }
                                )
                            </span>
                        }
                        #[key(position)]
                        for (position, item) in slides {
                            slide(item: item, index: position, current: current)
                        }
                        if many {
                            <span class="next">
                                icon_button(
                                    glyph: Icon::ChevronRight,
                                    label: &next,
                                    variant: IconButtonVariant::Outline,
                                    attrs: attributes! { data-act="lightbox-next" }
                                )
                            </span>
                        }
                    </div>
                    <p class="caption" hidden=(caption.is_empty())>(&caption)</p>
                </div>
                <aside class="side" aria-label=(t!(lightbox_panel))>
                    lightbox_panel(thread: thread)
                </aside>
            </div>
        </dialog>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::{
        post::testing::{sample_image, sample_post},
        testing::render,
    };

    #[component]
    async fn demo(cx: &Cx, index: usize) -> Result<impl View> {
        let mut post = sample_post();
        post.can_reply = false;
        post.media = vec![
            sample_image(1, 640, 480, "first picture"),
            sample_image(2, 800, 600, ""),
        ];
        let thread = ThreadDto {
            post,
            ancestors: Vec::new(),
            replies: Vec::new(),
            hidden_replies: Vec::new(),
        };
        Ok(view! { lightbox(thread: &thread, index: index) })
    }

    #[test]
    fn the_viewer_is_a_named_dialog_with_every_slide() {
        let html = render("en", |cx| view! { demo(index: 0) }.boxed());
        assert!(html.contains(r#"<dialog id="lightbox""#), "{html}");
        assert!(html.contains(r#"aria-label="Media viewer""#), "{html}");
        assert_eq!(html.matches(r#"class="slide""#).count(), 2, "{html}");
        assert!(html.contains(r#"alt="first picture""#), "{html}");
        assert!(html.contains("1 / 2"), "{html}");
        assert!(html.contains(r#"aria-label="Next""#), "{html}");
        assert!(html.contains(r#"aria-label="Previous""#), "{html}");
        assert!(html.contains(r#"aria-label="Post and replies""#), "{html}");
        assert!(html.contains(">View post</a>"), "{html}");
    }

    #[test]
    fn only_the_opened_slide_is_shown() {
        let html = render("en", |cx| view! { demo(index: 1) }.boxed());
        assert!(html.contains("2 / 2"), "{html}");
        let first = html.find(r#"data-index="0""#).expect("slide 0");
        assert!(
            html[first..].starts_with(r#"data-index="0" data-alt="first picture" hidden"#),
            "{html}"
        );
    }
}
