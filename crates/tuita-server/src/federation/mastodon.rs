//! The Mastodon status JSON of a local post, which link-preview crawlers (Discord, and other
//! clients that find an `application/activity+json` alternate) read from
//! `GET /api/v1/statuses/{id}`. Read-only and anonymous: it shows what a visitor sees.

use serde_json::{Value, json};
use time::OffsetDateTime;
use tuita_core::ap::format_time;
use uuid::Uuid;

use super::ids;
use super::render::{content_html, media_url};
use crate::config::Config;
use crate::db::ap::{Attachment, EmojiRef, LocalAuthor, LocalPost, Mentioned, PollData};
use crate::dto::Visibility;
use crate::error::AppError;
use crate::state::AppState;

/// The status of local post `post`; `None` unless the post is live and public or unlisted and
/// its author is neither suspended nor deleted.
///
/// # Errors
///
/// Database failures.
pub async fn status(state: &AppState, post: Uuid) -> Result<Option<Value>, AppError> {
    let Some(post) = super::render::load(state, post)
        .await?
        .filter(LocalPost::is_open)
    else {
        return Ok(None);
    };
    let author_bio =
        crate::db::ap::mentions_in(state.db(), state.config().domain(), &post.author.bio).await?;
    let texts = [post.author.display_name.as_str(), post.author.bio.as_str()];
    let catalog = crate::db::ap::local_emojis(state.db(), &texts).await?;
    let author_emojis = crate::db::ap::mentioned_emojis(&catalog, &texts);
    Ok(Some(status_value(
        state.config(),
        &post,
        &author_bio,
        &author_emojis,
    )))
}

/// [`status`] over a loaded post; `bio_mentions` are the accounts the author's bio names and
/// `author_emojis` the local emotes the author's display name and bio use.
#[must_use]
pub fn status_value(
    config: &Config,
    post: &LocalPost,
    bio_mentions: &[Mentioned],
    author_emojis: &[EmojiRef],
) -> Value {
    let handle = &post.author.handle;
    let status_emojis = emojis_json(config, &post.emojis);
    let visibility = match post.visibility {
        Visibility::Unlisted => "unlisted",
        _ => "public",
    };
    json!({
        "id": post.id,
        "uri": ids::note_id(config, handle, post.id),
        "url": ids::note_url(config, handle, post.id),
        "created_at": format_time(post.created_at),
        "edited_at": post.edited_at.map(format_time),
        "in_reply_to_id": post.reply_to.as_ref().map(|parent| parent.id),
        "in_reply_to_account_id": post.reply_to.as_ref().map(|parent| parent.author_id),
        "sensitive": post.sensitive || !post.cw.is_empty(),
        "spoiler_text": post.cw,
        "visibility": visibility,
        "language": null,
        "replies_count": post.reply_count,
        "reblogs_count": post.repost_count,
        "favourites_count": post.like_count,
        "content": content_html(config, &post.text, &post.mentions),
        "reblog": null,
        "application": null,
        "account": account(config, &post.author, bio_mentions, &emojis_json(config, author_emojis)),
        "media_attachments": post.media.iter().map(|media| attachment(config, media)).collect::<Vec<_>>(),
        "mentions": post.mentions.iter().map(|mentioned| mention(config, mentioned)).collect::<Vec<_>>(),
        "tags": tags(config, &post.text),
        "emojis": status_emojis,
        "card": null,
        "poll": post.poll.as_ref().map(|poll| poll_json(post.id, poll, &status_emojis)),
    })
}

fn account(
    config: &Config,
    author: &LocalAuthor,
    bio_mentions: &[Mentioned],
    emojis: &[Value],
) -> Value {
    let site_icon = format!("{}/icon-512.png", config.public_url);
    let avatar = author
        .avatar
        .as_ref()
        .map_or(site_icon, |file| media_url(config, &file.path));
    let header = author
        .banner
        .as_ref()
        .map_or_else(String::new, |file| media_url(config, &file.path));
    let note = if author.bio.trim().is_empty() {
        String::new()
    } else {
        content_html(config, &author.bio, bio_mentions)
    };
    json!({
        "id": author.id,
        "username": author.handle,
        "acct": author.handle,
        "display_name": author.display_name,
        "locked": false,
        "bot": author.bot,
        "discoverable": author.discoverable,
        "group": false,
        "created_at": format_time(author.created_at),
        "note": note,
        "url": ids::profile_url(config, &author.handle),
        "uri": ids::actor_id(config, &author.handle),
        "avatar": avatar,
        "avatar_static": avatar,
        "header": header,
        "header_static": header,
        "followers_count": author.followers,
        "following_count": author.following,
        "statuses_count": author.posts,
        "last_status_at": null,
        "emojis": emojis,
        "fields": [],
    })
}

fn attachment(config: &Config, media: &Attachment) -> Value {
    let url = media_url(config, &media.file.path);
    let preview = media
        .poster
        .as_deref()
        .map_or_else(|| url.clone(), |poster| media_url(config, poster));
    let meta = match (media.width, media.height) {
        (Some(width), Some(height)) if width > 0 && height > 0 => {
            let size = json!({
                "width": width,
                "height": height,
                "size": format!("{width}x{height}"),
                "aspect": f64::from(width) / f64::from(height),
            });
            json!({"original": size, "small": size})
        }
        _ => json!({}),
    };
    json!({
        "id": media.id,
        "type": if media.kind == "video" { "video" } else { "image" },
        "url": url,
        "preview_url": preview,
        "remote_url": null,
        "text_url": null,
        "description": (!media.alt.is_empty()).then_some(&media.alt),
        "blurhash": null,
        "meta": meta,
    })
}

fn mention(config: &Config, mentioned: &Mentioned) -> Value {
    let (acct, url) = match &mentioned.domain {
        None => (
            mentioned.handle.clone(),
            ids::profile_url(config, &mentioned.handle),
        ),
        Some(domain) => (
            format!("{}@{domain}", mentioned.handle),
            mentioned
                .url
                .clone()
                .or_else(|| mentioned.ap_id.clone())
                .unwrap_or_default(),
        ),
    };
    json!({
        "id": mentioned.id,
        "username": mentioned.handle,
        "url": url,
        "acct": acct,
    })
}

fn tags(config: &Config, text: &str) -> Vec<Value> {
    tuita_core::text::parse_post(text)
        .hashtags()
        .into_iter()
        .map(|tag| json!({"name": tag, "url": format!("{}/tags/{tag}", config.public_url)}))
        .collect()
}

/// The `CustomEmoji` entities of local emotes, with absolute image URLs.
fn emojis_json(config: &Config, emojis: &[EmojiRef]) -> Vec<Value> {
    emojis
        .iter()
        .map(|emoji| {
            let url = media_url(config, &emoji.file.path);
            json!({
                "shortcode": emoji.shortcode,
                "url": url,
                "static_url": url,
                "visible_in_picker": true,
            })
        })
        .collect()
}

fn poll_json(post: Uuid, poll: &PollData, emojis: &[Value]) -> Value {
    let votes: i64 = poll
        .options
        .iter()
        .map(|(_, votes)| i64::from(*votes))
        .sum();
    json!({
        "id": post,
        "expires_at": format_time(poll.ends_at),
        "expired": poll.ends_at <= OffsetDateTime::now_utc(),
        "multiple": poll.multiple,
        "votes_count": votes,
        "voters_count": poll.voters,
        "voted": false,
        "own_votes": [],
        "options": poll
            .options
            .iter()
            .map(|(title, votes)| json!({"title": title, "votes_count": votes}))
            .collect::<Vec<_>>(),
        "emojis": emojis,
    })
}
