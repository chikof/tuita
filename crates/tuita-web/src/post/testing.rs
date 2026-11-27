//! `PostDto` fixtures for the render tests of this area.

use time::macros::datetime;
use tuita_server::dto::{
    AvatarShape, EmojiDto, Ink, LinkCardDto, MediaDto, MediaKind, PollDto, PollOptionDto,
    PostCounts, PostDto, PostViewer, QuoteDto, ReplyPolicy, UserRefDto, Visibility,
};
use uuid::Uuid;

/// An account.
pub(crate) fn sample_user(handle: &str) -> UserRefDto {
    UserRefDto {
        id: Uuid::from_u128(0xa11c_e000 + handle.bytes().map(u128::from).sum::<u128>()),
        handle: handle.to_owned(),
        domain: None,
        display_name: handle.to_uppercase(),
        avatar_url: None,
        avatar_shape: AvatarShape::Circle,
        ink: Ink::Bone,
        verified: false,
        bot: false,
        emojis: Vec::new(),
    }
}

/// A plain public post by `ada` with nothing attached.
pub(crate) fn sample_post() -> PostDto {
    PostDto {
        id: Uuid::from_u128(0x90_57),
        author: sample_user("ada"),
        text: "hello world".to_owned(),
        rich_text: vec![tuita_server::dto::SegmentDto::Text {
            text: "hello world".to_owned(),
        }],
        cw: String::new(),
        sensitive: false,
        created_at: datetime!(2026-11-09 10:00 UTC),
        edited_at: None,
        reply_policy: ReplyPolicy::Everyone,
        visibility: Visibility::Public,
        url: None,
        client: None,
        counts: PostCounts {
            likes: 0,
            reposts: 0,
            replies: 0,
            quotes: 0,
            views: 0,
        },
        viewer: PostViewer::default(),
        can_reply: true,
        reply_to: None,
        quote: None,
        media: Vec::new(),
        card: None,
        poll: None,
        hidden_reply: false,
        pinned: false,
        entry_id: Uuid::from_u128(0x90_57),
        reposted_by: None,
        emojis: Vec::new(),
    }
}

/// A picture of `width` by `height` px named `alt`.
pub(crate) fn sample_image(n: u128, width: i32, height: i32, alt: &str) -> MediaDto {
    MediaDto {
        id: Uuid::from_u128(0x3ed1_a000 + n),
        kind: MediaKind::Image,
        url: format!("/media/{n}.png"),
        poster_url: None,
        alt: alt.to_owned(),
        width: Some(width),
        height: Some(height),
    }
}

/// An open single-choice poll with three answers and seven votes.
pub(crate) fn sample_poll() -> PollDto {
    PollDto {
        options: [3, 1, 3]
            .into_iter()
            .zip(["red", "green", "blue"])
            .map(|(votes, text)| PollOptionDto {
                text: text.to_owned(),
                votes,
            })
            .collect(),
        total: 7,
        voters: 7,
        multiple: false,
        ends_at: datetime!(2099-01-01 00:00 UTC),
        closed: false,
        voted: Vec::new(),
    }
}

/// A link preview.
pub(crate) fn sample_card() -> LinkCardDto {
    LinkCardDto {
        url: "https://www.example.com/a".to_owned(),
        title: "A title".to_owned(),
        description: "About it".to_owned(),
        site: "Example".to_owned(),
        image: None,
        large: false,
    }
}

/// An embedded quote by `bob`.
pub(crate) fn sample_quote() -> QuoteDto {
    QuoteDto {
        id: Uuid::from_u128(0x90_58),
        available: true,
        author: Some(sample_user("bob")),
        text: "quoted words".to_owned(),
        rich_text: Vec::new(),
        cw: String::new(),
        sensitive: false,
        created_at: datetime!(2026-11-09 09:00 UTC),
        emojis: Vec::new(),
    }
}

/// A custom emote.
pub(crate) fn sample_emoji(shortcode: &str) -> EmojiDto {
    EmojiDto {
        shortcode: shortcode.to_owned(),
        url: format!("/media/{}.png", shortcode.to_lowercase()),
        category: None,
    }
}
