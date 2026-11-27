//! Finding custom emote tokens in text.
//!
//! The rule is the one of `tuita_core::emoji`: the opening colon starts the text or follows a
//! character that is neither a word character nor `/`; the closing colon ends the text or is
//! followed by a non-word character; a shortcode is 2 to 32 ASCII letters, digits or
//! underscores. Only shortcodes the post lists in `emojis` (compared case-insensitively) become
//! emotes; every other token stays text.

use tuita_core::emoji::{MAX_LEN, MIN_LEN};
use tuita_server::dto::EmojiDto;

/// One piece of text: literal characters or a resolved custom emote.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EmojiPart<'a> {
    Text(&'a str),
    Emoji {
        /// The shortcode as typed, without colons.
        shortcode: &'a str,
        url: &'a str,
    },
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn is_shortcode_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Splits `text` into literal runs and the emotes of `emojis`.
#[must_use]
pub(crate) fn split_emoji<'a>(text: &'a str, emojis: &'a [EmojiDto]) -> Vec<EmojiPart<'a>> {
    if text.is_empty() {
        return Vec::new();
    }
    if emojis.is_empty() || !text.contains(':') {
        return vec![EmojiPart::Text(text)];
    }
    let mut parts = Vec::new();
    let mut last = 0;
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
        pos += len + 1;
        let Some(emoji) = emojis
            .iter()
            .rev()
            .find(|emoji| emoji.shortcode.eq_ignore_ascii_case(name))
        else {
            continue;
        };
        if start > last {
            parts.push(EmojiPart::Text(&text[last..start]));
        }
        parts.push(EmojiPart::Emoji {
            shortcode: name,
            url: &emoji.url,
        });
        last = pos;
    }
    if parts.is_empty() {
        return vec![EmojiPart::Text(text)];
    }
    if last < text.len() {
        parts.push(EmojiPart::Text(&text[last..]));
    }
    parts
}

#[cfg(test)]
mod test {
    use super::*;

    fn emoji(shortcode: &str) -> EmojiDto {
        EmojiDto {
            shortcode: shortcode.to_owned(),
            url: format!("/media/{}.png", shortcode.to_lowercase()),
            category: None,
        }
    }

    fn list() -> Vec<EmojiDto> {
        vec![emoji("blob"), emoji("a1"), emoji("Party_Cat")]
    }

    fn em<'a>(shortcode: &'a str, url: &'a str) -> EmojiPart<'a> {
        EmojiPart::Emoji { shortcode, url }
    }

    #[test]
    fn empty_text_has_no_parts_and_no_emotes_leave_one_text_part() {
        let emojis = list();
        assert!(split_emoji("", &emojis).is_empty());
        assert_eq!(
            split_emoji("hello :blob:", &[]),
            [EmojiPart::Text("hello :blob:")]
        );
        assert_eq!(
            split_emoji("no colons", &emojis),
            [EmojiPart::Text("no colons")]
        );
    }

    #[test]
    fn a_known_token_is_replaced_and_the_text_around_it_stays() {
        let emojis = list();
        assert_eq!(
            split_emoji("hi :blob: there", &emojis),
            [
                EmojiPart::Text("hi "),
                em("blob", "/media/blob.png"),
                EmojiPart::Text(" there")
            ]
        );
        assert_eq!(
            split_emoji(":blob: x :a1:", &emojis),
            [
                em("blob", "/media/blob.png"),
                EmojiPart::Text(" x "),
                em("a1", "/media/a1.png")
            ]
        );
    }

    #[test]
    fn adjacent_and_repeated_tokens_all_match() {
        let emojis = list();
        assert_eq!(
            split_emoji(":blob::a1::blob:", &emojis),
            [
                em("blob", "/media/blob.png"),
                em("a1", "/media/a1.png"),
                em("blob", "/media/blob.png")
            ]
        );
    }

    #[test]
    fn matching_ignores_case_and_keeps_the_typed_shortcode() {
        let emojis = list();
        assert_eq!(
            split_emoji(":BLOB: :party_cat:", &emojis),
            [
                em("BLOB", "/media/blob.png"),
                EmojiPart::Text(" "),
                em("party_cat", "/media/party_cat.png")
            ]
        );
    }

    #[test]
    fn unknown_shortcodes_stay_text() {
        let emojis = list();
        assert_eq!(
            split_emoji("a :nope: b", &emojis),
            [EmojiPart::Text("a :nope: b")]
        );
        assert_eq!(
            split_emoji(":nope::blob:", &emojis),
            [EmojiPart::Text(":nope:"), em("blob", "/media/blob.png")]
        );
    }

    #[test]
    fn times_and_glued_tokens_do_not_match() {
        let thirty = [emoji("30")];
        assert_eq!(
            split_emoji("12:30:45", &thirty),
            [EmojiPart::Text("12:30:45")]
        );
        assert_eq!(
            split_emoji("at 12:30:45 or :30:", &thirty),
            [
                EmojiPart::Text("at 12:30:45 or "),
                em("30", "/media/30.png")
            ]
        );
        let emojis = list();
        for glued in ["a:blob:", ":blob:a", "https://x.example/:blob:"] {
            assert_eq!(split_emoji(glued, &emojis), [EmojiPart::Text(glued)]);
        }
        assert_eq!(
            split_emoji("see (:blob:)", &emojis),
            [
                EmojiPart::Text("see ("),
                em("blob", "/media/blob.png"),
                EmojiPart::Text(")")
            ]
        );
    }

    #[test]
    fn the_length_limits_are_two_and_thirty_two() {
        assert_eq!(split_emoji(":a:", &[emoji("a")]), [EmojiPart::Text(":a:")]);
        let long = "x".repeat(33);
        let text = format!(":{long}:");
        assert_eq!(
            split_emoji(&text, &[emoji(&long)]),
            [EmojiPart::Text(text.as_str())]
        );
        let max = "x".repeat(32);
        let text = format!(":{max}:");
        let emojis = [emoji(&max)];
        let url = format!("/media/{max}.png");
        assert_eq!(split_emoji(&text, &emojis), [em(&max, &url)]);
    }

    #[test]
    fn it_finds_the_tokens_of_the_vectors_shared_with_the_server() {
        #[derive(serde::Deserialize)]
        struct Case {
            input: String,
            shortcodes: Vec<String>,
        }
        let cases: Vec<Case> = serde_json::from_str(include_str!(
            "../../../tuita-core/tests/fixtures/emoji_tokens.json"
        ))
        .expect("vectors parse");
        for case in cases {
            let words: Vec<EmojiDto> = case
                .input
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .filter(|word| word.len() >= 2)
                .map(emoji)
                .collect();
            let mut found: Vec<String> = Vec::new();
            for part in split_emoji(&case.input, &words) {
                if let EmojiPart::Emoji { shortcode, .. } = part
                    && !found
                        .iter()
                        .any(|seen| seen.eq_ignore_ascii_case(shortcode))
                {
                    found.push(shortcode.to_owned());
                }
            }
            assert_eq!(found, case.shortcodes, "input {:?}", case.input);
        }
    }

    #[test]
    fn hostile_input_stays_fast() {
        let input = format!(
            "{}{}{}",
            ":".repeat(50_000),
            "a".repeat(50_000),
            ":ab".repeat(20_000)
        );
        let emojis = list();
        let start = std::time::Instant::now();
        let _ = split_emoji(&input, &emojis);
        assert!(start.elapsed().as_millis() < 500);
    }
}
