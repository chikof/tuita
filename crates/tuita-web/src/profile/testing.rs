//! `UserDto` fixtures for the render tests of this area.

use time::macros::datetime;
use tuita_server::dto::{AvatarShape, Ink, Relation, UserCounts, UserDto};
use uuid::Uuid;

/// A local account with nothing set.
pub(crate) fn user(handle: &str) -> UserDto {
    UserDto {
        id: Uuid::from_u128(0xb0b0_0000 + handle.bytes().map(u128::from).sum::<u128>()),
        handle: handle.to_owned(),
        domain: None,
        display_name: handle.to_uppercase(),
        bio: String::new(),
        location: String::new(),
        website: String::new(),
        avatar_url: None,
        banner_url: None,
        avatar_shape: AvatarShape::Circle,
        ink: Ink::Bone,
        verified: false,
        staff: None,
        org: false,
        bot: false,
        bot_owner: None,
        affiliation: None,
        pinned_post_id: None,
        created_at: datetime!(2026-11-09 10:00 UTC),
        counts: UserCounts {
            posts: 3,
            followers: 1200,
            following: 7,
        },
        relation: Relation {
            following: false,
            follows_you: false,
            blocked: false,
            blocks_you: false,
            muted: false,
            follow_pending: false,
        },
        suspended: false,
        url: None,
        locked: false,
        emojis: Vec::new(),
    }
}

/// A remote account.
pub(crate) fn remote(handle: &str, domain: &str) -> UserDto {
    let mut user = user(handle);
    user.domain = Some(domain.to_owned());
    user.url = Some(format!("https://{domain}/@{handle}"));
    user
}
