//! The emoji picker's grid: a fixed list, no library.

/// The Unicode emoji the picker offers, in grid order.
pub(crate) const EMOJI: [&str; 64] = [
    "😀", "😂", "🥹", "😍", "🤔", "😎", "🥳", "😭", "🙃", "😅", "😇", "🤩", "😴", "🤯", "😬", "🫠",
    "🙏", "👍", "👏", "🙌", "💪", "👀", "🫶", "🤝", "🔥", "✨", "💜", "💙", "💚", "❤️", "💔", "💯",
    "🎉", "🎨", "🎵", "📸", "🚀", "☕", "🍕", "🍻", "🌈", "🌙", "⭐", "☀️", "🌸", "🌊", "🐱", "🐶",
    "🌻", "🍩", "✅", "❌", "⚠️", "💡", "📌", "🔗", "🎮", "📚", "🎬", "🏆", "🌍", "🧠", "🤖", "👾",
];

#[cfg(test)]
mod test {
    use std::collections::BTreeSet;

    use super::EMOJI;

    #[test]
    fn it_has_64_distinct_entries() {
        assert_eq!(EMOJI.len(), 64);
        assert_eq!(EMOJI.iter().collect::<BTreeSet<_>>().len(), 64);
    }
}
