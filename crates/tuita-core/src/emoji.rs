//! Custom emote shortcodes: the naming rule and finding `:shortcode:` tokens in text.
//!
//! A shortcode is 2 to 32 ASCII letters, digits or underscores. In text a token is
//! `:shortcode:` where the opening colon starts the text or follows a character that is neither
//! a word character nor `/`, and the closing colon ends the text or is followed by a non-word
//! character (Mastodon's rule). `12:30:45`, `http://x/:foo:` and `:ok:ok:` hold no token.
//!
//! ```
//! use tuita_core::emoji::{is_valid_shortcode, shortcodes};
//!
//! assert!(is_valid_shortcode("blobcat"));
//! assert!(!is_valid_shortcode("a"));
//! assert_eq!(shortcodes("hi :Blob_Cat: and :party:!"), ["Blob_Cat", "party"]);
//! assert!(shortcodes("it is 12:30:45 now").is_empty());
//! ```

/// Fewest characters in a shortcode.
pub const MIN_LEN: usize = 2;
/// Most characters in a shortcode.
pub const MAX_LEN: usize = 32;

fn is_shortcode_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Whether `shortcode` (without colons) is a legal emote name.
#[must_use]
pub fn is_valid_shortcode(shortcode: &str) -> bool {
    (MIN_LEN..=MAX_LEN).contains(&shortcode.len()) && shortcode.chars().all(is_shortcode_char)
}

/// The shortcodes `text` mentions, without colons, in order of first appearance. Case is kept;
/// two spellings that differ only in case count once, because emotes are unique
/// case-insensitively.
#[must_use]
pub fn shortcodes(text: &str) -> Vec<&str> {
    let mut found: Vec<&str> = Vec::new();
    let mut pos = 0;
    while let Some(offset) = text[pos..].find(':') {
        let start = pos + offset;
        pos = start + 1;
        if text[..start]
            .chars()
            .next_back()
            .is_some_and(|before| is_word(before) || before == '/')
        {
            continue;
        }
        let rest = &text[pos..];
        let len = rest
            .find(|c: char| !is_shortcode_char(c))
            .unwrap_or(rest.len());
        let Some(after) = rest[len..].strip_prefix(':') else {
            continue;
        };
        if !(MIN_LEN..=MAX_LEN).contains(&len) || after.chars().next().is_some_and(is_word) {
            continue;
        }
        let name = &rest[..len];
        if !found.iter().any(|seen| seen.eq_ignore_ascii_case(name)) {
            found.push(name);
        }
        // Resume after the closing colon; the colon that follows may open the next token.
        pos += len + 1;
    }
    found
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn shortcode_rule() {
        assert!(is_valid_shortcode("ab"));
        assert!(is_valid_shortcode("A_1"));
        assert!(is_valid_shortcode(&"a".repeat(32)));
        assert!(!is_valid_shortcode("a"));
        assert!(!is_valid_shortcode(&"a".repeat(33)));
        assert!(!is_valid_shortcode("a-b"));
        assert!(!is_valid_shortcode("é_é"));
        assert!(!is_valid_shortcode(""));
    }

    #[test]
    fn finds_tokens_in_order() {
        assert_eq!(shortcodes(":one: two :three:"), ["one", "three"]);
        assert_eq!(shortcodes("(:wave:)"), ["wave"]);
        assert_eq!(shortcodes(":wave:"), ["wave"]);
    }

    #[test]
    fn clock_times_are_not_tokens() {
        assert!(shortcodes("12:30:45").is_empty());
        assert!(shortcodes("at 12:30:45 sharp").is_empty());
    }

    #[test]
    fn too_short_or_too_long_is_no_token() {
        assert!(shortcodes(":a:").is_empty());
        assert!(shortcodes(&format!(":{}:", "a".repeat(33))).is_empty());
        assert_eq!(shortcodes(&format!(":{}:", "a".repeat(32))).len(), 1);
    }

    #[test]
    fn glued_closing_colon_is_no_token() {
        assert!(shortcodes(":ok:ok:").is_empty());
        assert!(shortcodes(":ok:é").is_empty());
    }

    #[test]
    fn urls_are_not_tokens() {
        assert!(shortcodes("http://x/:foo:").is_empty());
        assert!(shortcodes("see https://a.example/:foo: now").is_empty());
    }

    #[test]
    fn adjacent_tokens_both_match() {
        assert_eq!(shortcodes(":a_b::c_d:"), ["a_b", "c_d"]);
    }

    #[test]
    fn distinct_ignoring_case_keeps_the_first_spelling() {
        assert_eq!(shortcodes(":Wave: :wave: :WAVE:"), ["Wave"]);
    }

    #[test]
    fn non_ascii_around_tokens() {
        assert_eq!(shortcodes("日本 :wave: 語"), ["wave"]);
        assert!(shortcodes("é:wave:").is_empty());
    }
}
