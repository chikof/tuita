//! A post as a report or an account page shows it.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_server::dto::PostDto;

use crate::{
    admin::common::{stamp, tr},
    fmt::{acct_of, handle_of, rel_time},
    paths,
    post::rich_text::{around, emote_text, rich_text, user_name},
    shell::parts::account_avatar,
};

/// The author, the time, the warning, the text and a note on media and polls.
#[component]
pub(crate) async fn post_preview(cx: &Cx, post: &PostDto) -> Result<impl View> {
    let author = &post.author;
    let (before, after) = around(&tr!(cx, cw_label, reason: "\u{0}"));
    let handle = handle_of(&author.handle, author.domain.as_deref());
    let href = paths::post(
        &acct_of(&author.handle, author.domain.as_deref()),
        &post.id.to_string(),
    );
    let at = stamp(post.created_at);
    let media = (!post.media.is_empty()).then(
        || tr!(cx, admin_report_media, count: i32::try_from(post.media.len()).unwrap_or(i32::MAX)),
    );
    let poll = post.poll.is_some().then(|| tr!(cx, admin_report_poll));
    Ok(view! {
        <div class="preview" data-c="admin-post-preview">
            <p class="head">
                <span aria-hidden="true">
                    account_avatar(
                        handle: &author.handle,
                        src: author.avatar_url.as_deref(),
                        shape: author.avatar_shape,
                        ink: author.ink,
                        size: 24
                    )
                </span>
                user_name(
                    name: &author.display_name,
                    verified: author.verified,
                    emojis: &author.emojis
                )
                <a class="dim" href=(href)>
                    (handle)
                    " · "
                    rel_time(ts: &at)
                </a>
            </p>
            if !post.cw.is_empty() {
                <p class="dim" translate="no">
                    (before)
                    emote_text(text: &post.cw, emojis: &post.emojis)
                    (after)
                </p>
            }
            <p class="text">
                rich_text(
                    segments: &post.rich_text,
                    domain: author.domain.as_deref(),
                    emojis: &post.emojis
                )
            </p>
            if let Some(media) = media {
                <p class="dim">(media)</p>
            }
            if let Some(poll) = poll {
                <p class="dim">(poll)</p>
            }
        </div>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::{
        post::testing::{sample_image, sample_poll, sample_post},
        testing::render,
    };

    #[test]
    fn it_shows_the_author_the_text_and_what_is_attached() {
        let mut post = sample_post();
        post.media = vec![sample_image(1, 10, 10, "a")];
        post.poll = Some(sample_poll());
        post.cw = "spoilers".to_owned();
        let html = crate::l10n::strip_isolates(&render("en", |cx| {
            view! { post_preview(post: &post) }.boxed()
        }));
        assert!(html.contains(r#"data-c="admin-post-preview""#), "{html}");
        assert!(html.contains("hello world"), "{html}");
        assert!(html.contains(r#"href="/@ada/"#), "{html}");
        assert!(html.contains("@ada"), "{html}");
        assert!(html.contains("1 attachment"), "{html}");
        assert!(html.contains("spoilers"), "{html}");
        assert!(html.contains("Content warning: "), "{html}");
    }

    #[test]
    fn a_plain_post_has_no_notes() {
        let post = sample_post();
        let html = render("en", |cx| view! { post_preview(post: &post) }.boxed());
        assert!(!html.contains("attachment"), "{html}");
        assert!(!html.contains("translate=\"no\" class"), "{html}");
    }
}
