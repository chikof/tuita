//! `GET /u/{handle}/rss.xml`: a public RSS 2.0 feed of an account's top-level posts.

use std::fmt::Write as _;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use time::format_description::well_known::Rfc2822;

use crate::db::export::{FeedPost, feed_posts};
use crate::db::settings;
use crate::db::users::{UserRow, find_local};
use crate::error::AppError;
use crate::state::AppState;

/// Posts in a feed.
const FEED_POSTS: i64 = 30;
/// Characters of the text that make an item title when the post has no content warning.
const TITLE_CHARS: usize = 80;

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/u/{handle}/rss.xml", get(feed))
}

async fn feed(
    State(state): State<AppState>,
    Path(handle): Path<String>,
) -> Result<Response, AppError> {
    let user = match find_local(state.db(), &handle, false).await {
        Ok(user) => user,
        Err(AppError::Missing(_)) => {
            return Ok((
                StatusCode::NOT_FOUND,
                [(CONTENT_TYPE, "text/plain")],
                "Not found",
            )
                .into_response());
        }
        Err(err) => return Err(err),
    };
    let site = settings::load(state.db()).await?.name;
    let posts = feed_posts(state.db(), user.id, FEED_POSTS).await?;
    let body = document(&user, &site, &state.config().public_url, &posts);
    Ok((
        [
            (CONTENT_TYPE, "application/rss+xml; charset=utf-8"),
            (CACHE_CONTROL, "public, max-age=300"),
        ],
        body,
    )
        .into_response())
}

/// Escapes `text` for XML 1.0 text and attribute values, and drops the characters XML 1.0
/// cannot carry at all (most control characters, unpaired surrogates never occur in `str`).
fn escape_xml(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\'' => out.push_str("&apos;"),
            '"' => out.push_str("&quot;"),
            '\t'
            | '\n'
            | '\r'
            | '\u{20}'..='\u{d7ff}'
            | '\u{e000}'..='\u{fffd}'
            | '\u{10000}'..='\u{10ffff}' => out.push(c),
            _ => {}
        }
    }
    out
}

/// The RSS 2.0 document of `user`'s `posts`. `origin` is the public URL without a trailing slash.
fn document(user: &UserRow, site: &str, origin: &str, posts: &[FeedPost]) -> String {
    let handle = &user.handle;
    let description = if user.bio.is_empty() {
        format!("Posts by @{handle} on {site}")
    } else {
        user.bio.clone()
    };
    let mut out = String::new();
    // Writing to a String cannot fail.
    let _ = write!(
        out,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <rss version=\"2.0\"><channel><title>{} (@{}) \u{b7} {}</title>\n\
         <link>{origin}/@{}</link><description>{}</description>\n\
         <language>en</language>\n",
        escape_xml(&user.display_name),
        escape_xml(handle),
        escape_xml(site),
        escape_xml(handle),
        escape_xml(&description),
    );
    for post in posts {
        item(&mut out, user, origin, post);
    }
    out.push_str("</channel></rss>\n");
    out
}

fn item(out: &mut String, user: &UserRow, origin: &str, post: &FeedPost) {
    // Writing to a String cannot fail, so the `write!` results are ignored below.
    let link = format!("{origin}/@{}/{}", escape_xml(&user.handle), post.id);
    let title: String = if post.cw.is_empty() {
        post.text.chars().take(TITLE_CHARS).collect()
    } else {
        post.cw.clone()
    };
    let title = if title.is_empty() { "Image" } else { &title };

    let mut html = String::new();
    if !post.cw.is_empty() {
        let _ = write!(html, "<p><strong>CW: {}</strong></p>", escape_xml(&post.cw));
    }
    let text = escape_xml(&post.text.replace("\r\n", "\n")).replace('\n', "<br>");
    let _ = write!(html, "<p>{text}</p>");
    for (url, alt) in &post.media {
        let _ = write!(
            html,
            "<p><img src=\"{}\" alt=\"{}\"></p>",
            escape_xml(&format!("{origin}{url}")),
            escape_xml(alt)
        );
    }

    let _ = write!(
        out,
        "<item><title>{}</title><link>{link}</link>\n<guid isPermaLink=\"true\">{link}</guid>",
        escape_xml(title)
    );
    if let Ok(date) = post.created_at.format(&Rfc2822) {
        let _ = write!(out, "<pubDate>{date}</pubDate>");
    }
    let _ = write!(
        out,
        "\n<description>{}</description></item>\n",
        escape_xml(&html)
    );
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn escapes_markup_and_quotes() {
        assert_eq!(
            escape_xml(r#"<script>a&b</script> 'x' "y""#),
            "&lt;script&gt;a&amp;b&lt;/script&gt; &apos;x&apos; &quot;y&quot;"
        );
    }

    #[test]
    fn drops_characters_xml_cannot_carry() {
        assert_eq!(
            escape_xml("a\u{0}b\u{b}c\u{1f}d\u{fffe}e\u{ffff}f"),
            "abcdef"
        );
        assert_eq!(
            escape_xml("tab\there\nnew \u{1f600}"),
            "tab\there\nnew \u{1f600}"
        );
    }
}
