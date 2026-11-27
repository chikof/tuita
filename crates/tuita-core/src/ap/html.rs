//! HTML in and out. Inbound `content` is hostile: [`sanitize_html`] keeps `p br a span`, and
//! [`html_to_text`] turns what is left into the plain text tuita stores. Outbound,
//! [`rich_text_to_html`] writes the HTML Mastodon expects from a parsed post.

use std::collections::HashSet;

use ammonia::Builder;

use super::origin::url_authority;
use crate::text::{RichText, Segment};

/// Classes kept on `a` and `span`: the ones Mastodon, Misskey and GoToSocial use for mentions,
/// hashtags and shortened links.
const ALLOWED_CLASSES: &[&str] = &[
    "mention",
    "hashtag",
    "h-card",
    "u-url",
    "invisible",
    "ellipsis",
    "quote-inline",
];

/// Cuts `html` down to `p`, `br`, `a` (`href` with http or https, `rel`, `class`) and `span`
/// (`class`). Scripts and their bodies, styles, event handlers and `javascript:` links are gone.
///
/// ```
/// use tuita_core::ap::sanitize_html;
///
/// let clean = sanitize_html(r#"<p onclick="x()">hi<script>alert(1)</script></p>"#);
/// assert_eq!(clean, "<p>hi</p>");
/// ```
#[must_use]
pub fn sanitize_html(html: &str) -> String {
    let mut builder = Builder::empty();
    builder
        .tags(HashSet::from(["p", "br", "a", "span"]))
        .tag_attributes(std::collections::HashMap::from([(
            "a",
            HashSet::from(["href", "rel"]),
        )]))
        .allowed_classes(std::collections::HashMap::from([
            ("a", HashSet::from_iter(ALLOWED_CLASSES.iter().copied())),
            ("span", HashSet::from_iter(ALLOWED_CLASSES.iter().copied())),
        ]))
        .url_schemes(HashSet::from(["http", "https"]))
        .url_relative(ammonia::UrlRelative::Deny)
        .link_rel(None)
        .clean_content_tags(HashSet::from(["script", "style", "template"]))
        .strip_comments(true);
    builder.clean(html).to_string()
}

/// Plain text of untrusted HTML: sanitised first, then `<p>` becomes a blank line, `<br>` a line
/// break and a link its address. Mention links become `@user@domain` from the link's host, and
/// the invisible parts of Mastodon's shortened links disappear.
///
/// ```
/// use tuita_core::ap::html_to_text;
///
/// let html = r#"<p>hi <span class="h-card"><a href="https://b.example/@bob" class="u-url mention">@<span>bob</span></a></span></p><p>bye<br>now</p>"#;
/// assert_eq!(html_to_text(html), "hi @bob@b.example\n\nbye\nnow");
/// ```
#[must_use]
pub fn html_to_text(html: &str) -> String {
    let clean = sanitize_html(html);
    let mut walker = Walker::default();
    walker.run(&clean);
    strip_controls(&walker.finish())
}

/// `text` without control characters, but with its line feeds and tabs. Free text from another
/// server goes through this before it is stored: a NUL cannot be stored at all, and the rest
/// (escape codes, bells) only do harm in a terminal or a log.
///
/// ```
/// use tuita_core::ap::strip_controls;
///
/// assert_eq!(strip_controls("a\u{0}b\u{1b}[31m\tc\nd"), "ab[31m\tc\nd");
/// ```
#[must_use]
pub fn strip_controls(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
        .collect()
}

#[derive(Default)]
struct Anchor {
    href: String,
    classes: String,
    text: String,
    /// Mastodon hid part of the link text, so the text is not the address.
    shortened: bool,
    /// `rel="tag"`: Misskey marks hashtag links this way and adds no class.
    tag: bool,
}

#[derive(Default)]
struct Walker {
    out: String,
    anchor: Option<Anchor>,
    /// One entry per open `span`: whether it is invisible.
    spans: Vec<bool>,
    hidden: usize,
}

impl Walker {
    fn run(&mut self, html: &str) {
        let mut rest = html;
        while !rest.is_empty() {
            if let Some(tag_and_rest) = rest.strip_prefix('<') {
                let end = tag_end(tag_and_rest);
                self.tag(&tag_and_rest[..end]);
                rest = tag_and_rest.get(end + 1..).unwrap_or("");
            } else {
                let end = rest.find('<').unwrap_or(rest.len());
                self.text(&rest[..end]);
                rest = &rest[end..];
            }
        }
    }

    fn push(&mut self, text: &str) {
        match &mut self.anchor {
            Some(anchor) => anchor.text.push_str(text),
            None => self.out.push_str(text),
        }
    }

    fn text(&mut self, raw: &str) {
        if self.hidden == 0 {
            let decoded = decode_entities(raw);
            self.push(&decoded);
        }
    }

    fn tag(&mut self, inner: &str) {
        let closing = inner.starts_with('/');
        let body = inner.trim_start_matches('/').trim_end_matches('/');
        let name_end = body
            .find(|c: char| c.is_ascii_whitespace())
            .unwrap_or(body.len());
        let name = body[..name_end].to_ascii_lowercase();
        let attrs = &body[name_end..];
        match (name.as_str(), closing) {
            ("br", _) => self.push("\n"),
            ("p", false) => {
                let trimmed = self.out.trim_end_matches('\n').len();
                if trimmed > 0 {
                    self.out.truncate(trimmed);
                    self.out.push_str("\n\n");
                }
            }
            ("span", false) => {
                let invisible = attr(attrs, "class")
                    .is_some_and(|class| class.split_ascii_whitespace().any(|c| c == "invisible"));
                self.spans.push(invisible);
                if invisible {
                    self.hidden += 1;
                    if let Some(anchor) = &mut self.anchor {
                        anchor.shortened = true;
                    }
                }
            }
            ("span", true) => {
                if self.spans.pop() == Some(true) {
                    self.hidden = self.hidden.saturating_sub(1);
                }
            }
            ("a", false) => {
                if self.anchor.is_none() {
                    self.anchor = Some(Anchor {
                        href: attr(attrs, "href").unwrap_or_default(),
                        classes: attr(attrs, "class").unwrap_or_default(),
                        tag: attr(attrs, "rel")
                            .is_some_and(|rel| rel.split_ascii_whitespace().any(|r| r == "tag")),
                        text: String::new(),
                        shortened: false,
                    });
                }
            }
            ("a", true) => {
                if let Some(anchor) = self.anchor.take() {
                    let emitted = anchor_text(&anchor);
                    self.out.push_str(&emitted);
                }
            }
            _ => {}
        }
    }

    fn finish(mut self) -> String {
        if let Some(anchor) = self.anchor.take() {
            let emitted = anchor_text(&anchor);
            self.out.push_str(&emitted);
        }
        let mut text = String::with_capacity(self.out.len());
        let mut newlines = 0;
        for ch in self.out.chars() {
            if ch == '\n' {
                newlines += 1;
                if newlines <= 2 {
                    text.push(ch);
                }
            } else {
                newlines = 0;
                text.push(ch);
            }
        }
        text.trim().to_owned()
    }
}

fn anchor_text(anchor: &Anchor) -> String {
    let visible = anchor.text.trim();
    let classes: Vec<&str> = anchor.classes.split_ascii_whitespace().collect();
    if classes.contains(&"hashtag") || (anchor.tag && visible.starts_with('#')) {
        return if visible.starts_with('#') {
            visible.to_owned()
        } else {
            format!("#{visible}")
        };
    }
    if classes.contains(&"mention") {
        let user = visible.trim_start_matches('@');
        if user.is_empty() {
            return String::new();
        }
        if user.contains('@') {
            return format!("@{user}");
        }
        return match url_authority(&anchor.href) {
            Some(domain) => format!("@{user}@{domain}"),
            None => format!("@{user}"),
        };
    }
    if anchor.href.is_empty() {
        return visible.to_owned();
    }
    let bare = |text: &str| {
        text.trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/')
            .to_owned()
    };
    if anchor.shortened
        || visible.is_empty()
        || bare(visible) == bare(&anchor.href)
        || anchor.href.starts_with(visible.trim_end_matches('…'))
    {
        anchor.href.clone()
    } else {
        format!("{visible} ({})", anchor.href)
    }
}

/// Index of the `>` that ends a tag, skipping quoted attribute values. The whole input when the
/// tag never closes.
fn tag_end(input: &str) -> usize {
    let mut quote: Option<char> = None;
    for (at, ch) in input.char_indices() {
        match (quote, ch) {
            (None, '>') => return at,
            (None, '"' | '\'') => quote = Some(ch),
            (Some(open), _) if ch == open => quote = None,
            _ => {}
        }
    }
    input.len()
}

/// The value of attribute `name` in an attribute list such as ` href="x" class="y"`.
fn attr(attrs: &str, name: &str) -> Option<String> {
    let lower = attrs.to_ascii_lowercase();
    let mut from = 0;
    while let Some(found) = lower[from..].find(name) {
        let start = from + found;
        let before_ok = start == 0
            || lower[..start]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_whitespace());
        let after = attrs[start + name.len()..].trim_start();
        if before_ok && let Some(value) = after.strip_prefix('=') {
            let value = value.trim_start();
            return Some(match value.chars().next() {
                Some(q @ ('"' | '\'')) => {
                    let inner = &value[1..];
                    let end = inner.find(q).unwrap_or(inner.len());
                    decode_entities(&inner[..end])
                }
                _ => {
                    let end = value
                        .find(|c: char| c.is_ascii_whitespace())
                        .unwrap_or(value.len());
                    decode_entities(&value[..end])
                }
            });
        }
        from = start + name.len();
    }
    None
}

/// Decodes the entities HTML serializers emit: the named five, `&nbsp;` and numeric references.
fn decode_entities(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        let decoded = tail
            .find(';')
            .filter(|end| *end <= 10)
            .and_then(|end| entity(&tail[1..end]).map(|ch| (ch, end)));
        match decoded {
            Some((ch, end)) => {
                out.push(ch);
                rest = &tail[end + 1..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn entity(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some(' '),
        _ => {
            let number = name.strip_prefix('#')?;
            let code = match number.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => number.parse().ok()?,
            };
            char::from_u32(code).filter(|c| *c != '\0')
        }
    }
}

/// HTML-escapes `text` for element content and attribute values.
#[must_use]
pub fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

/// Where a mention points: the actor's profile URL for `handle` and `domain` (`None` means the
/// post author's own instance, which the caller resolves), or `None` to leave it as text.
pub type MentionUrl<'a> = &'a dyn Fn(&str, Option<&str>) -> Option<String>;

/// The page that lists a hashtag.
pub type TagUrl<'a> = &'a dyn Fn(&str) -> String;

/// Builds Mastodon-style HTML from a parsed post: paragraphs for blank lines, `<br>` for line
/// breaks, mention and hashtag links with `class="mention"` and `class="mention hashtag"`.
#[must_use]
pub fn rich_text_to_html(
    text: &RichText,
    mention_url: MentionUrl<'_>,
    tag_url: TagUrl<'_>,
) -> String {
    let mut out = String::from("<p>");
    let mut breaks = 0usize;
    let mut started = false;
    let flush = |out: &mut String, breaks: &mut usize, started: bool| {
        if started {
            match *breaks {
                0 => {}
                1 => out.push_str("<br>"),
                _ => out.push_str("</p><p>"),
            }
        }
        *breaks = 0;
    };
    for segment in text.segments() {
        match segment {
            Segment::Text(piece) => {
                for ch in piece.chars() {
                    if ch == '\n' {
                        breaks += 1;
                    } else if ch != '\r' {
                        flush(&mut out, &mut breaks, started);
                        started = true;
                        push_escaped(&mut out, ch);
                    }
                }
            }
            Segment::Mention { handle, domain } => {
                flush(&mut out, &mut breaks, started);
                started = true;
                match mention_url(handle, domain.as_deref()) {
                    Some(url) => out.push_str(&format!(
                        "<span class=\"h-card\"><a href=\"{}\" class=\"u-url mention\" rel=\"nofollow noopener noreferrer\">@<span>{}</span></a></span>",
                        escape_html(&url),
                        escape_html(handle)
                    )),
                    None => {
                        out.push('@');
                        out.push_str(&escape_html(handle));
                        if let Some(domain) = domain {
                            out.push('@');
                            out.push_str(&escape_html(domain));
                        }
                    }
                }
            }
            Segment::Hashtag { tag, text } => {
                flush(&mut out, &mut breaks, started);
                started = true;
                out.push_str(&format!(
                    "<a href=\"{}\" class=\"mention hashtag\" rel=\"tag nofollow noopener noreferrer\">#<span>{}</span></a>",
                    escape_html(&tag_url(tag)),
                    escape_html(text)
                ));
            }
            Segment::Link(url) => {
                flush(&mut out, &mut breaks, started);
                started = true;
                out.push_str(&format!(
                    "<a href=\"{}\" rel=\"nofollow noopener noreferrer\" target=\"_blank\">{}</a>",
                    escape_html(url),
                    escape_html(url)
                ));
            }
        }
    }
    out.push_str("</p>");
    out
}

fn push_escaped(out: &mut String, ch: char) {
    match ch {
        '&' => out.push_str("&amp;"),
        '<' => out.push_str("&lt;"),
        '>' => out.push_str("&gt;"),
        '"' => out.push_str("&quot;"),
        '\'' => out.push_str("&#39;"),
        other => out.push(other),
    }
}

/// Escaped plain text as HTML: blank lines split paragraphs and single line breaks become `<br>`.
/// For bios, where nothing is linked.
#[must_use]
pub fn plain_to_html(text: &str) -> String {
    let normalized = text.replace("\r\n", "\n");
    let paragraphs: Vec<String> = normalized
        .split("\n\n")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| format!("<p>{}</p>", escape_html(part).replace('\n', "<br>")))
        .collect();
    paragraphs.concat()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::text::parse_post;

    #[test]
    fn script_bodies_handlers_and_bad_schemes_are_removed() {
        let dirty = r#"<p onclick="evil()">a<script>alert(1)</script><style>p{}</style>
            <a href="javascript:alert(1)">x</a><a href="https://ok.example/">y</a><img src=x onerror=evil()><iframe src="https://e"></iframe></p>"#;
        let clean = sanitize_html(dirty);
        assert!(!clean.contains("script"), "{clean}");
        assert!(!clean.contains("alert"), "{clean}");
        assert!(!clean.contains("onclick"), "{clean}");
        assert!(!clean.contains("javascript:"), "{clean}");
        assert!(!clean.contains("<img"), "{clean}");
        assert!(!clean.contains("iframe"), "{clean}");
        assert!(clean.contains(r#"href="https://ok.example/""#), "{clean}");
    }

    #[test]
    fn text_of_a_mastodon_post() {
        let html = concat!(
            "<p>Hello <span class=\"h-card\"><a href=\"https://ada.example/@ada\" class=\"u-url mention\">@<span>ada</span></a></span> ",
            "<a href=\"https://mastodon.example/tags/rust\" class=\"mention hashtag\" rel=\"tag\">#<span>rust</span></a> &amp; &lt;3<br>",
            "<a href=\"https://example.com/a/very/long/path\" rel=\"nofollow noopener noreferrer\" target=\"_blank\">",
            "<span class=\"invisible\">https://</span><span class=\"ellipsis\">example.com/a/very/lo</span><span class=\"invisible\">ng/path</span></a></p>",
            "<p>second</p>"
        );
        assert_eq!(
            html_to_text(html),
            "Hello @ada@ada.example #rust & <3\nhttps://example.com/a/very/long/path\n\nsecond"
        );
    }

    #[test]
    fn mention_hosts_keep_their_port() {
        let html =
            r#"<a href="http://127.0.0.1:9000/@bob" class="u-url mention">@<span>bob</span></a>"#;
        assert_eq!(html_to_text(html), "@bob@127.0.0.1:9000");
    }

    #[test]
    fn anchors_with_their_own_text_keep_both() {
        assert_eq!(
            html_to_text(r#"<a href="https://x.example/p">read this</a>"#),
            "read this (https://x.example/p)"
        );
    }

    #[test]
    fn entities_and_broken_markup_do_not_panic() {
        assert_eq!(
            html_to_text("a &#128512; b &bogus; &#x41;"),
            "a \u{1f600} b &bogus; A"
        );
        assert_eq!(html_to_text("<p><a href=\"https://x"), "");
        assert_eq!(html_to_text("<<<>>>&"), "<<<>>>&");
        assert_eq!(html_to_text(""), "");
    }

    #[test]
    fn outbound_html_escapes_and_links() {
        let post = parse_post("hi <b> @ana@b.example #Rust\n\nnew para\nline https://x.example/a");
        let html = rich_text_to_html(
            &post,
            &|handle, domain| domain.map(|d| format!("https://{d}/@{handle}")),
            &|tag| format!("https://me.example/tags/{tag}"),
        );
        assert!(
            html.starts_with(
                "<p>hi &lt;b&gt; <span class=\"h-card\"><a href=\"https://b.example/@ana\""
            ),
            "{html}"
        );
        assert!(html.contains("class=\"mention hashtag\""), "{html}");
        assert!(
            html.contains("</p><p>new para<br>line <a href=\"https://x.example/a\""),
            "{html}"
        );
        assert!(html.ends_with("</a></p>"), "{html}");
    }

    #[test]
    fn a_mention_with_no_url_stays_text() {
        let post = parse_post("@ghost@nowhere.example");
        let html = rich_text_to_html(&post, &|_, _| None, &|tag| tag.to_owned());
        assert_eq!(html, "<p>@ghost@nowhere.example</p>");
    }

    #[test]
    fn plain_bios_become_paragraphs() {
        assert_eq!(
            plain_to_html("a <b>\nc\n\nd"),
            "<p>a &lt;b&gt;<br>c</p><p>d</p>"
        );
        assert_eq!(plain_to_html(""), "");
    }
}
