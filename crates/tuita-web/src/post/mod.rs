//! Posts: card, actions, thread, media, poll, lightbox, hover card, dialogs.

pub(crate) mod carousel;
pub(crate) mod content_warning;
pub(crate) mod custom_emoji;
pub(crate) mod dialogs;
pub(crate) mod domain_chip;
pub(crate) mod fragments;
pub(crate) mod globals;
pub(crate) mod heart;
pub(crate) mod lightbox;
pub(crate) mod link_card;
pub(crate) mod media_grid;
pub(crate) mod poll;
pub(crate) mod poll_math;
pub(crate) mod post_actions;
pub(crate) mod post_card;
pub(crate) mod post_thread;
pub(crate) mod quote_card;
pub(crate) mod rich_text;
pub(crate) mod sensitive_cover;
#[cfg(test)]
pub(crate) mod testing;
pub(crate) mod user_hover_card;
pub(crate) mod video_player;
pub(crate) mod visibility_badge;

use tuita_server::dto::{PostDto, UserRefDto};
use uuid::Uuid;

use crate::{fmt::acct_of, paths};

/// `/@ada/{post_id}`, the page of a post.
#[must_use]
pub(crate) fn post_path(author: &UserRefDto, id: Uuid) -> String {
    paths::post(
        &acct_of(&author.handle, author.domain.as_deref()),
        &id.to_string(),
    )
}

/// `/@ada`, the profile of an account.
#[must_use]
pub(crate) fn profile_path(user: &UserRefDto) -> String {
    paths::profile(&acct_of(&user.handle, user.domain.as_deref()))
}

/// The share text of a post: its content warning, else the first 120 characters.
#[must_use]
pub(crate) fn share_text(post: &PostDto) -> String {
    if post.cw.is_empty() {
        post.text.chars().take(120).collect()
    } else {
        post.cw.clone()
    }
}
