//! Post text: splits it into plain text, mentions, hashtags and links, and measures it.
//!
//! The rules, written as a scanner:
//!
//! - A link starts with `http://` or `https://` and runs to the next space, `<`, `>` or `"`. Its
//!   last character is never `. , : ; ' ! ? )` or `]`, so "see https://a.example/x." links
//!   `https://a.example/x`.
//! - A mention is `@handle` or `@handle@domain`, with a handle of 1 to 15 ASCII letters, digits
//!   or underscores. It must follow the start of the text or a character that is not a word
//!   character or `@`, and must not run on into more word characters (`@a_very_long_handle_x`
//!   is plain text).
//! - A hashtag is `#` and 1 to 50 letters, digits or underscores, after the start of the text or
//!   a character that is not a letter, digit, `_` or `&`. Letters and digits are
//!   `char::is_alphanumeric`, which also takes the marks that belong to a word (a Devanagari
//!   vowel sign), where a plain `\p{L}\p{N}` stops before them.
//!
//! The length of a post counts Unicode scalar values, and a link counts [`LINK_LEN`] whatever its
//! size (Mastodon's rule). One emoji can be several scalar
//! values.
//!
//! ```
//! use tuita_core::text::{Segment, parse_post};
//!
//! let post = parse_post("hi @ana@example.org #Rust https://tuita.example/a.");
//! assert_eq!(post.mentions(), [("ana", Some("example.org"))]);
//! assert_eq!(post.hashtags(), ["rust"]);
//! assert_eq!(post.links(), ["https://tuita.example/a"]);
//! assert_eq!(post.len(), "hi @ana@example.org #Rust ".chars().count() + 23 + 1);
//! assert!(matches!(post.segments()[0], Segment::Text(_)));
//! ```

/// How many characters a link counts for in a post's length.
pub const LINK_LEN: usize = 23;
/// Most characters in a post, as counted by [`RichText::len`] on the trimmed text.
pub const MAX_POST_LEN: usize = 500;

/// Most characters in a hashtag, without the `#`.
const MAX_TAG_LEN: usize = 50;
/// Most characters in a mention's handle.
const MAX_MENTION_HANDLE_LEN: usize = 15;
/// Most digits in a mention domain's port.
const MAX_PORT_DIGITS: usize = 5;

/// One piece of a post.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    /// Plain text.
    Text(Box<str>),
    /// `@handle` or `@handle@domain`, without the leading `@`s. The handle keeps its case and the
    /// domain is lower case. A missing domain means the author's own instance.
    Mention {
        /// The handle as typed.
        handle: Box<str>,
        /// The domain, with an optional `:port`, in lower case.
        domain: Option<Box<str>>,
    },
    /// `#tag`.
    Hashtag {
        /// The tag in lower case, as stored and searched.
        tag: Box<str>,
        /// The tag as typed, without the `#`, for display.
        text: Box<str>,
    },
    /// An `http://` or `https://` URL as typed.
    Link(Box<str>),
}

// 40: the largest variants hold two Box<str> (32 B) and no niche is left for the tag.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<Segment>() == 40);

impl Segment {
    fn char_len(&self) -> usize {
        match self {
            Self::Text(text) => text.chars().count(),
            Self::Mention { handle, domain } => {
                1 + handle.chars().count() + domain.as_ref().map_or(0, |d| 1 + d.chars().count())
            }
            Self::Hashtag { text, .. } => 1 + text.chars().count(),
            Self::Link(_) => LINK_LEN,
        }
    }
}

/// A parsed post: its segments in order. Joining them back, with the link and mention text as
/// typed, gives the original text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RichText {
    segments: Vec<Segment>,
}

impl RichText {
    /// The segments in order of appearance.
    #[must_use]
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// The length in Unicode scalar values, where every link counts [`LINK_LEN`].
    #[must_use]
    pub fn len(&self) -> usize {
        self.segments.iter().map(Segment::char_len).sum()
    }

    /// Whether the text is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// The mentioned `(handle, domain)` pairs in order, each once (ignoring case).
    #[must_use]
    pub fn mentions(&self) -> Vec<(&str, Option<&str>)> {
        let mut out: Vec<(&str, Option<&str>)> = Vec::new();
        for segment in &self.segments {
            if let Segment::Mention { handle, domain } = segment {
                let domain = domain.as_deref();
                if !out
                    .iter()
                    .any(|(h, d)| h.eq_ignore_ascii_case(handle) && *d == domain)
                {
                    out.push((handle, domain));
                }
            }
        }
        out
    }

    /// The lower-case hashtags in order, each once.
    #[must_use]
    pub fn hashtags(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        for segment in &self.segments {
            if let Segment::Hashtag { tag, .. } = segment
                && !out.contains(&&**tag)
            {
                out.push(tag);
            }
        }
        out
    }

    /// The URLs in order, as typed.
    #[must_use]
    pub fn links(&self) -> Vec<&str> {
        self.segments
            .iter()
            .filter_map(|segment| match segment {
                Segment::Link(url) => Some(&**url),
                _ => None,
            })
            .collect()
    }
}

/// Splits post text into segments. Never fails: whatever is not a link, mention or hashtag is
/// text.
#[must_use]
pub fn parse_post(text: &str) -> RichText {
    let mut segments = Vec::new();
    let mut last = 0;
    let mut at = 0;
    while at < text.len() {
        let Some(token) = token_at(text, at) else {
            at += text[at..].chars().next().map_or(1, char::len_utf8);
            continue;
        };
        if last < token.start {
            segments.push(Segment::Text(text[last..token.start].into()));
        }
        segments.push(token.segment);
        last = token.end;
        at = token.end;
    }
    if last < text.len() {
        segments.push(Segment::Text(text[last..].into()));
    }
    RichText { segments }
}

/// A token found at some position: the byte range after any prefix character, and its segment.
struct Token {
    start: usize,
    end: usize,
    segment: Segment,
}

/// Tries link, then mention, then hashtag at byte offset `at`, as the alternatives of `TOKEN_RE`.
fn token_at(text: &str, at: usize) -> Option<Token> {
    link_at(text, at)
        .or_else(|| mention_at(text, at))
        .or_else(|| hashtag_at(text, at))
}

/// JavaScript's `\s`.
pub(crate) const fn is_js_space(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
    )
}

/// JavaScript's `\w`: ASCII letters, digits and underscore.
const fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn link_at(text: &str, at: usize) -> Option<Token> {
    let rest = &text[at..];
    let scheme = ["https://", "http://"]
        .into_iter()
        .find(|s| rest.starts_with(s))?;
    let body_start = at + scheme.len();
    // The longest run of at least two characters whose last character may end a URL.
    let mut end = None;
    for (n, (offset, c)) in text[body_start..].char_indices().enumerate() {
        if is_js_space(c) || matches!(c, '<' | '>' | '"') {
            break;
        }
        if n >= 1 && !matches!(c, '.' | ',' | ':' | ';' | '\'' | '!' | '?' | ')' | ']') {
            end = Some(body_start + offset + c.len_utf8());
        }
    }
    let end = end?;
    Some(Token {
        start: at,
        end,
        segment: Segment::Link(text[at..end].into()),
    })
}

/// Finds where a token introduced by `sigil` may start at `at`: at the start of the text, or
/// after one character that `is_prefix` accepts. Returns the offsets of the sigil, in the order
/// the regex tries them.
fn sigil_positions(
    text: &str,
    at: usize,
    sigil: char,
    is_prefix: fn(char) -> bool,
) -> impl Iterator<Item = usize> {
    let at_start = (at == 0 && text.starts_with(sigil)).then_some(0);
    let after_prefix = text[at..].chars().next().and_then(|c| {
        let next = at + c.len_utf8();
        (is_prefix(c) && text[next..].starts_with(sigil)).then_some(next)
    });
    at_start.into_iter().chain(after_prefix)
}

fn mention_at(text: &str, at: usize) -> Option<Token> {
    let can_precede = |c: char| !(c.is_ascii() && is_word_byte(c as u8)) && c != '@';
    sigil_positions(text, at, '@', can_precede).find_map(|sigil| mention_body(text, sigil))
}

/// Matches `handle` and an optional `@domain` after the `@` at `sigil`.
fn mention_body(text: &str, sigil: usize) -> Option<Token> {
    let bytes = text.as_bytes();
    let handle_start = sigil + 1;
    let handle_len = bytes[handle_start..]
        .iter()
        .take_while(|b| is_word_byte(**b))
        .count();
    // A longer handle cannot match by giving up characters: the next one would be a word character.
    if handle_len == 0 || handle_len > MAX_MENTION_HANDLE_LEN {
        return None;
    }
    let handle_end = handle_start + handle_len;
    let handle = &text[handle_start..handle_end];
    let (end, domain) = if bytes.get(handle_end) == Some(&b'@') {
        let domain_start = handle_end + 1;
        let end = domain_end(bytes, domain_start)?;
        (
            end,
            Some(
                text[domain_start..end]
                    .to_ascii_lowercase()
                    .into_boxed_str(),
            ),
        )
    } else {
        (handle_end, None)
    };
    Some(Token {
        start: sigil,
        end,
        segment: Segment::Mention {
            handle: handle.into(),
            domain,
        },
    })
}

/// The end of a domain: dot-separated labels of letters, digits and `-`, then an optional
/// `:port`, cut back until the next character is neither a word character nor `@`.
fn domain_end(bytes: &[u8], start: usize) -> Option<usize> {
    let is_label = |b: u8| b.is_ascii_alphanumeric() || b == b'-';
    let run = |from: usize| bytes[from..].iter().take_while(|b| is_label(**b)).count();
    let mut end = start + run(start);
    if end == start {
        return None;
    }
    while bytes.get(end) == Some(&b'.') && bytes.get(end + 1).is_some_and(|b| is_label(*b)) {
        end += 1 + run(end + 1);
    }
    if bytes.get(end) == Some(&b':') {
        let digits = bytes[end + 1..]
            .iter()
            .take(MAX_PORT_DIGITS)
            .take_while(|b| b.is_ascii_digit())
            .count();
        if digits > 0 {
            end += 1 + digits;
        }
    }
    (start + 1..=end).rev().find(|&cut| {
        let ends_cleanly = !matches!(bytes[cut - 1], b'.' | b':');
        let runs_on = bytes
            .get(cut)
            .is_some_and(|b| is_word_byte(*b) || *b == b'@');
        ends_cleanly && !runs_on
    })
}

fn hashtag_at(text: &str, at: usize) -> Option<Token> {
    let can_precede = |c: char| !(c.is_alphanumeric() || c == '_' || c == '&');
    sigil_positions(text, at, '#', can_precede).find_map(|sigil| {
        let tag_start = sigil + 1;
        let is_tag_char = |c: &char| c.is_alphanumeric() || *c == '_';
        let tag_len: usize = text[tag_start..]
            .chars()
            .take(MAX_TAG_LEN)
            .take_while(is_tag_char)
            .map(char::len_utf8)
            .sum();
        if tag_len == 0 {
            return None;
        }
        let end = tag_start + tag_len;
        let shown = &text[tag_start..end];
        let segment = Segment::Hashtag {
            tag: shown.to_lowercase().into(),
            text: shown.into(),
        };
        Some(Token {
            start: sigil,
            end,
            segment,
        })
    })
}

#[cfg(test)]
mod reference;

#[cfg(test)]
mod test {
    use super::*;

    /// A compact form of the segments: `T` text, `M` mention, `H` hashtag, `U` link.
    fn show(text: &str) -> Vec<String> {
        parse_post(text)
            .segments()
            .iter()
            .map(|segment| match segment {
                Segment::Text(t) => format!("T:{t}"),
                Segment::Mention {
                    handle,
                    domain: None,
                } => format!("M:{handle}"),
                Segment::Mention {
                    handle,
                    domain: Some(d),
                } => format!("M:{handle}@{d}"),
                Segment::Hashtag { text, .. } => format!("H:{text}"),
                Segment::Link(url) => format!("U:{url}"),
            })
            .collect()
    }

    #[test]
    fn segments_reference_values() {
        let wrong: Vec<_> = reference::CASES
            .iter()
            .filter(|(text, expected)| show(text).join("\u{1f}") != *expected)
            .collect();
        assert_eq!(wrong, Vec::<&(&str, &str)>::new());
    }

    #[test]
    fn splits_mixed_post() {
        assert_eq!(
            show("hi @ana, see https://a.example/x. #Rust!"),
            [
                "T:hi ",
                "M:ana",
                "T:, see ",
                "U:https://a.example/x",
                "T:. ",
                "H:Rust",
                "T:!"
            ]
        );
    }

    #[test]
    fn link_drops_trailing_punctuation() {
        for (text, url) in [
            ("https://a.example.", "https://a.example"),
            ("https://a.example/x?y=1!", "https://a.example/x?y=1"),
            ("(https://a.example/x)", "https://a.example/x"),
            ("https://a.example/[x]", "https://a.example/[x"),
            ("https://a.example/x.,;:", "https://a.example/x"),
            ("<https://a.example>", "https://a.example"),
        ] {
            assert_eq!(parse_post(text).links(), [url], "{text}");
        }
        assert!(parse_post("https://.").links().is_empty());
        assert!(parse_post("HTTPS://a.example").links().is_empty());
    }

    #[test]
    fn mention_forms() {
        assert_eq!(parse_post("@ana").mentions(), [("ana", None)]);
        assert_eq!(
            parse_post("hi @Ana@Example.ORG:8080!").mentions(),
            [("Ana", Some("example.org:8080"))]
        );
        assert_eq!(parse_post("(@ana)").mentions(), [("ana", None)]);
        assert!(parse_post("mail me@example.org").mentions().is_empty());
        assert!(parse_post("@a_very_long_handle_x").mentions().is_empty());
        assert!(parse_post("@@ana").mentions().is_empty());
        assert!(parse_post("@ana@").mentions().is_empty());
    }

    #[test]
    fn mention_domain_backs_off_before_a_word_character() {
        assert_eq!(
            parse_post("@a@b.com:123456").mentions(),
            [("a", Some("b.com"))]
        );
        assert_eq!(parse_post("@a@exam-_").mentions(), [("a", Some("exam"))]);
        assert!(parse_post("@a@exam_ple").mentions().is_empty());
    }

    #[test]
    fn mentions_are_unique_ignoring_case() {
        let post = parse_post("@Ana @ana @ana@x.org @ANA@x.org @bo");
        assert_eq!(
            post.mentions(),
            [("Ana", None), ("ana", Some("x.org")), ("bo", None)]
        );
    }

    #[test]
    fn hashtags_are_lowercase_but_keep_display_text() {
        let post = parse_post("#Rust #ÉCOLE #rust #日本語 a#no &#38; #_x");
        assert_eq!(post.hashtags(), ["rust", "école", "日本語", "_x"]);
        assert_eq!(show("#Rust")[0], "H:Rust");
    }

    #[test]
    fn hashtags_keep_alphabetic_marks_of_a_word() {
        // A plain \p{L} ends the tag at a vowel sign (a mark that is part of the word) and reads "ह".
        assert_eq!(parse_post("#हि").hashtags(), ["हि"]);
    }

    #[test]
    fn hashtag_stops_at_fifty_characters() {
        let post = parse_post(&format!("#{}", "a".repeat(60)));
        assert_eq!(post.hashtags(), ["a".repeat(50)]);
        assert_eq!(post.segments().len(), 2);
    }

    #[test]
    fn length_counts_scalar_values_and_links_as_23() {
        assert_eq!(parse_post("").len(), 0);
        assert!(parse_post("").is_empty());
        assert_eq!(parse_post("héllo").len(), 5);
        assert_eq!(parse_post("👍").len(), 1);
        assert_eq!(parse_post("👨‍👩‍👧").len(), 5);
        assert_eq!(
            parse_post("https://a.example/a/very/long/path/that/goes/on/and/on").len(),
            23
        );
        assert_eq!(parse_post("a https://b.example. c").len(), 2 + 23 + 3);
        assert_eq!(parse_post("@ana@x.org #Tag").len(), "@ana@x.org #Tag".len());
    }

    #[test]
    fn emoji_next_to_tokens() {
        assert_eq!(show("😀#tag"), ["T:😀", "H:tag"]);
        assert_eq!(show("😀@ana"), ["T:😀", "M:ana"]);
        assert_eq!(
            show("🎉 https://a.example/🎉"),
            ["T:🎉 ", "U:https://a.example/🎉"]
        );
    }

    #[test]
    fn plain_text_stays_one_segment() {
        assert_eq!(
            show("nothing special: 100% #  @ http"),
            ["T:nothing special: 100% #  @ http"]
        );
    }
}
