//! Post text: custom emotes, rich text segments and display names.

use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use tuita_core::text::{Segment, parse_post};
use tuita_server::dto::{EmojiDto, SegmentDto};

use crate::{
    icons::Icon,
    l10n::{strip_isolates, t},
    paths,
    post::custom_emoji::{EmojiPart, split_emoji},
    ui::icon::icon,
};

/// The text before and after the slot a localized template puts a substituted value in, so the
/// value can be drawn as markup (with emotes) in place: `around(&template_with_slot)`.
///
/// The template is rendered with U+0000 as the value.
#[must_use]
pub(crate) fn around(template: &str) -> (String, String) {
    let text = strip_isolates(template);
    let (before, after) = text.split_once('\0').unwrap_or((&text, ""));
    (before.to_owned(), after.to_owned())
}

/// A custom emote picture.
#[component]
pub(crate) async fn emote(cx: &Cx, shortcode: &str, url: &str) -> Result<impl View> {
    let name = format!(":{shortcode}:");
    Ok(view! {
        <img
            class="emote"
            src=(url)
            alt=(&name)
            title=(&name)
            loading="lazy"
            decoding="async"
            draggable="false"
        >
    })
}

/// `text` with the `:shortcode:` tokens of `emojis` drawn as pictures, in place, without a
/// wrapper element.
#[component]
pub(crate) async fn emote_text(
    cx: &Cx,
    text: &str,
    #[default] emojis: &[EmojiDto],
) -> Result<impl View> {
    let parts = split_emoji(text, emojis);
    Ok(view! {
        for part in parts {
            if let EmojiPart::Emoji { shortcode, url } = part {
                emote(shortcode: shortcode, url: url)
            } else if let EmojiPart::Text(text) = part {
                (text)
            }
        }
    })
}

/// The shortened form of a link shown in a post: no scheme, no `www.`, no trailing slash, at most
/// 32 characters.
#[must_use]
pub(crate) fn short_url(url: &str) -> String {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let rest = rest.strip_prefix("www.").unwrap_or(rest);
    let shown = rest.strip_suffix('/').unwrap_or(rest);
    if shown.chars().count() > 32 {
        let head: String = shown.chars().take(31).collect();
        format!("{head}…")
    } else {
        shown.to_owned()
    }
}

/// Splits plain text (a bio, an old version of a post) into the segments the API sends for posts.
#[must_use]
pub(crate) fn segments_of(text: &str) -> Vec<SegmentDto> {
    parse_post(text)
        .segments()
        .iter()
        .map(|segment| match segment {
            Segment::Text(text) => SegmentDto::Text {
                text: text.to_string(),
            },
            Segment::Mention { handle, domain } => SegmentDto::Mention {
                handle: handle.to_string(),
                domain: domain.as_deref().map(str::to_owned),
            },
            Segment::Hashtag { tag, text } => SegmentDto::Hashtag {
                tag: tag.to_string(),
                text: text.to_string(),
            },
            Segment::Link(url) => SegmentDto::Link {
                url: url.to_string(),
            },
        })
        .collect()
}

/// What `rich_text` draws for one segment, with the link target worked out.
enum Piece<'a> {
    Text(&'a str),
    Link {
        href: String,
        label: String,
        external: bool,
    },
}

fn piece<'a>(segment: &'a SegmentDto, domain: Option<&str>) -> Piece<'a> {
    match segment {
        SegmentDto::Text { text } => Piece::Text(text),
        SegmentDto::Mention {
            handle,
            domain: mentioned,
        } => {
            let home = mentioned.as_deref().or(domain).filter(|d| !d.is_empty());
            let acct = home.map_or_else(|| handle.clone(), |home| format!("{handle}@{home}"));
            Piece::Link {
                href: paths::profile(&acct),
                label: mentioned
                    .as_deref()
                    .filter(|d| !d.is_empty())
                    .map_or_else(|| format!("@{handle}"), |d| format!("@{handle}@{d}")),
                external: false,
            }
        }
        SegmentDto::Hashtag { tag, text } => Piece::Link {
            href: paths::tag(tag),
            label: format!("#{text}"),
            external: false,
        },
        SegmentDto::Link { url } => Piece::Link {
            href: url.clone(),
            label: short_url(url),
            external: true,
        },
    }
}

/// A post's text: plain runs (with emotes), mentions, hashtags and links. `domain` is the home
/// instance of the author, which a mention without a domain belongs to.
#[component]
pub(crate) async fn rich_text(
    cx: &Cx,
    segments: &[SegmentDto],
    #[default] domain: Option<&str>,
    #[default] emojis: &[EmojiDto],
) -> Result<impl View> {
    let pieces: Vec<Piece<'_>> = segments
        .iter()
        .map(|segment| piece(segment, domain))
        .collect();
    Ok(view! {
        <span class="rich" translate="no" data-c="rich-text">
            for piece in pieces {
                if let Piece::Text(text) = piece {
                    emote_text(text: text, emojis: emojis)
                } else if let Piece::Link { href, label, external } = piece {
                    if external {
                        <a
                            href=(href)
                            target="_blank"
                            rel="noopener noreferrer nofollow"
                        >
                            (label)
                        </a>
                    } else {
                        <a href=(href)>(label)</a>
                    }
                }
            }
        </span>
    })
}

/// A display name with its verified mark and bot label.
#[component]
pub(crate) async fn user_name(
    cx: &Cx,
    name: &str,
    #[default] verified: bool,
    #[default] bot: bool,
    #[default] emojis: &[EmojiDto],
) -> Result<impl View> {
    let verified_label = t!(verified).resolve(cx).into_owned();
    let bot_title = t!(bot_account).resolve(cx).into_owned();
    Ok(view! {
        <span class="name" data-c="user-name">
            <span class="text" translate="no">
                emote_text(text: name, emojis: emojis)
            </span>
            if verified {
                <span
                    class="mark"
                    role="img"
                    aria-label=(&verified_label)
                    title=(&verified_label)
                >
                    icon(name: Icon::Check, size: 12)
                </span>
            }
            if bot {
                <span class="bot" title=(&bot_title)>(t!(bot_label))</span>
            }
        </span>
    })
}

#[cfg(test)]
mod test {
    use topcoat::view::ViewExt;

    use super::*;
    use crate::testing::render;

    fn blob() -> Vec<EmojiDto> {
        vec![EmojiDto {
            shortcode: "blob".to_owned(),
            url: "/media/blob.png".to_owned(),
            category: None,
        }]
    }

    #[test]
    fn around_splits_a_template_at_the_slot() {
        assert_eq!(
            around("Content warning: \u{2068}\0\u{2069}"),
            ("Content warning: ".to_owned(), String::new())
        );
        assert_eq!(around("<\0>!"), ("<".to_owned(), ">!".to_owned()));
    }

    #[test]
    fn urls_are_shortened_for_display() {
        assert_eq!(short_url("https://www.example.com/"), "example.com");
        assert_eq!(short_url("http://a.example/x?y=1"), "a.example/x?y=1");
        let long = format!("https://example.com/{}", "a".repeat(40));
        let short = short_url(&long);
        assert_eq!(short.chars().count(), 32);
        assert!(short.ends_with('…'));
    }

    #[test]
    fn emotes_are_pictures_named_by_their_shortcode() {
        let emojis = blob();
        let html = render("en", |cx| {
            view! { emote_text(text: "hi :blob: there", emojis: &emojis) }.boxed()
        });
        assert!(html.contains(r#"class="emote""#), "{html}");
        assert!(html.contains(r#"src="/media/blob.png""#), "{html}");
        assert!(html.contains(r#"alt=":blob:""#), "{html}");
        assert!(
            html.starts_with("hi <img") && html.ends_with("> there"),
            "{html}"
        );
    }

    #[test]
    fn rich_text_links_mentions_tags_and_urls() {
        let segments = vec![
            SegmentDto::Text {
                text: "hi ".to_owned(),
            },
            SegmentDto::Mention {
                handle: "ada".to_owned(),
                domain: None,
            },
            SegmentDto::Mention {
                handle: "bob".to_owned(),
                domain: Some("x.example".to_owned()),
            },
            SegmentDto::Hashtag {
                tag: "rust".to_owned(),
                text: "Rust".to_owned(),
            },
            SegmentDto::Link {
                url: "https://www.example.com/a".to_owned(),
            },
        ];
        let html = render("en", |cx| {
            view! { rich_text(segments: &segments, domain: Some("home.example")) }.boxed()
        });
        assert!(html.contains(r#"translate="no""#), "{html}");
        assert!(
            html.contains(r#"<a href="/@ada@home.example">@ada</a>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<a href="/@bob@x.example">@bob@x.example</a>"#),
            "{html}"
        );
        assert!(html.contains(r#"<a href="/tags/rust">#Rust</a>"#), "{html}");
        assert!(
            html.contains(r#"href="https://www.example.com/a""#)
                && html.contains(r#"rel="noopener noreferrer nofollow""#)
                && html.contains(">example.com/a</a>"),
            "{html}"
        );
    }

    #[test]
    fn a_name_carries_its_mark_and_label() {
        let html = render("en", |cx| {
            view! { user_name(name: "Ada", verified: true, bot: true) }.boxed()
        });
        assert!(html.contains(r#"class="name""#), "{html}");
        assert!(
            html.contains(r#"<span class="text" translate="no">Ada</span>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"role="img""#) && html.contains(r#"aria-label="Verified account""#),
            "{html}"
        );
        assert!(html.contains(r#"class="bot""#), "{html}");
        let plain = render("en", |cx| view! { user_name(name: "Ada") }.boxed());
        assert!(!plain.contains("mark") && !plain.contains("bot"), "{plain}");
    }
}
