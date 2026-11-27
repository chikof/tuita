//! Reading posts: the one loader every list, thread and action response goes through.
//!
//! [`load`] turns post ids into [`PostDto`]s for a viewer. It drops what the viewer must not
//! see: deleted posts, authors that are suspended, blocked either way or muted, and posts that
//! match the viewer's muted words. The rest of the post area (writes, lists) builds on it.

use std::collections::HashMap;

use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use tuita_core::text::{Segment, parse_post};
use uuid::Uuid;

use super::users::media_url;
use super::{
    blocked_author, emoji, media, muted_author, polls, visible_author, visible_post, word_muted,
};
use crate::dto::{
    AvatarShape, Ink, LinkCardDto, MediaDto, PollDto, PostCounts, PostDto, PostViewer, QuoteDto,
    ReplyPolicy, ReplyToDto, SegmentDto, UserRefDto, Visibility,
};
use crate::error::AppError;

/// Splits post text into the segments clients render.
#[must_use]
pub fn segments(text: &str) -> Vec<SegmentDto> {
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

#[allow(
    clippy::struct_excessive_bools,
    reason = "one column per flag the query computes"
)]
#[derive(FromRow)]
struct PostRow {
    id: Uuid,
    author_id: Uuid,
    text: String,
    cw: String,
    sensitive: bool,
    created_at: OffsetDateTime,
    edited_at: Option<OffsetDateTime>,
    reply_policy: String,
    visibility: String,
    url: Option<String>,
    client: Option<String>,
    reply_to_id: Option<Uuid>,
    quote_of_id: Option<Uuid>,
    hidden_reply: bool,
    views: i64,
    like_count: i32,
    repost_count: i32,
    reply_count: i32,
    quote_count: i32,
    a_handle: String,
    a_domain: Option<String>,
    a_display_name: String,
    a_avatar_id: Option<Uuid>,
    a_avatar_shape: String,
    a_ink: String,
    a_verified: bool,
    a_bot: bool,
    blocked: bool,
    muted: bool,
    liked: bool,
    reposted: bool,
    bookmarked: bool,
    author_follows_me: bool,
    mentions_me: bool,
    reply_to_handle: Option<String>,
    word_muted: bool,
    card_title: Option<String>,
    card_description: Option<String>,
    card_site: Option<String>,
    card_large: Option<bool>,
    card_link: Option<String>,
    card_image: Option<String>,
}

impl PostRow {
    /// The finished link card, when the unfurl worker stored one for this post's link.
    fn card(&self) -> Option<LinkCardDto> {
        let url = self.card_link.clone()?;
        Some(LinkCardDto {
            url,
            title: self.card_title.clone().unwrap_or_default(),
            description: self.card_description.clone().unwrap_or_default(),
            site: self.card_site.clone().unwrap_or_default(),
            image: self.card_image.as_deref().map(media::url),
            large: self.card_large.unwrap_or(false),
        })
    }

    fn author(&self) -> UserRefDto {
        author_ref(
            self.author_id,
            &self.a_handle,
            self.a_domain.as_deref(),
            &self.a_display_name,
            self.a_avatar_id,
            &self.a_avatar_shape,
            &self.a_ink,
            self.a_verified,
            self.a_bot,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn author_ref(
    id: Uuid,
    handle: &str,
    domain: Option<&str>,
    display_name: &str,
    avatar_id: Option<Uuid>,
    avatar_shape: &str,
    ink: &str,
    verified: bool,
    bot: bool,
) -> UserRefDto {
    UserRefDto {
        id,
        handle: handle.to_owned(),
        domain: domain.map(str::to_owned),
        display_name: display_name.to_owned(),
        avatar_url: avatar_id.map(media_url),
        avatar_shape: AvatarShape::parse(avatar_shape).unwrap_or(AvatarShape::Square),
        ink: Ink::parse(ink).unwrap_or(Ink::Bone),
        verified,
        bot,
        emojis: Vec::new(),
    }
}

/// The post columns [`PostRow`] reads, for a posts alias `p` joined to its author `a`.
fn select_sql() -> String {
    let blocked = blocked_author("a", "$1");
    let muted = muted_author("a", "$1");
    let word_muted = word_muted("p", "$1");
    let parent_visible = visible_post("rp", "$1");
    let parent_blocked = blocked_author("ru", "$1");
    format!(
        "select p.id, p.author_id, p.text, p.cw, p.sensitive, p.created_at, p.edited_at,
            p.reply_policy, p.client, p.reply_to_id, p.quote_of_id, p.visibility, p.url,
            coalesce(p.hidden_reply, false) as hidden_reply, coalesce(p.views, 0) as views,
            p.like_count, p.repost_count, p.reply_count, p.quote_count,
            a.handle as a_handle, a.domain as a_domain, a.display_name as a_display_name,
            a.avatar_id as a_avatar_id, a.avatar_shape as a_avatar_shape, a.ink as a_ink,
            a.verified as a_verified, coalesce(a.is_bot, false) as a_bot,
            {blocked} as blocked, {muted} as muted,
            exists (select 1 from likes where post_id = p.id and user_id = $1) as liked,
            exists (select 1 from reposts where post_id = p.id and user_id = $1) as reposted,
            exists (select 1 from bookmarks where post_id = p.id and user_id = $1) as bookmarked,
            exists (select 1 from follows where follower_id = p.author_id and followee_id = $1
                    and state = 'accepted') as author_follows_me,
            exists (select 1 from mentions where post_id = p.id and user_id = $1) as mentions_me,
            (select ru.handle from posts rp join users ru on ru.id = rp.author_id
              where rp.id = p.reply_to_id and rp.deleted_at is null and {parent_visible}
                and not {parent_blocked}
            ) as reply_to_handle,
            {word_muted} as word_muted,
            lc.title as card_title, lc.description as card_description, lc.site as card_site,
            lc.large as card_large, lc.url as card_link, ci.path as card_image
         from posts p join users a on a.id = p.author_id
         left join link_cards lc on lc.url = p.card_url and lc.ok
         left join media ci on ci.id = lc.image_id"
    )
}

/// Loads `ids` as seen by `viewer`, in the order given. Posts the viewer cannot see are left
/// out, so the result may be shorter than `ids`.
///
/// `allow` names an author whose posts stay visible although muted: the profile and thread of
/// someone you muted still show their posts. Blocks, suspensions and deletions always hide.
///
/// # Errors
///
/// Database failures.
pub async fn load(
    db: &PgPool,
    viewer: Uuid,
    ids: &[Uuid],
    allow: Option<Uuid>,
) -> Result<Vec<PostDto>, AppError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let visible = visible_post("p", "$1");
    let sql = format!(
        "{} where p.id = any($2) and p.deleted_at is null and {visible}",
        select_sql()
    );
    let rows: Vec<PostRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(ids)
        .fetch_all(db)
        .await?;
    let rows: Vec<PostRow> = rows
        .into_iter()
        .filter(|row| {
            !row.blocked
                && (!row.muted || Some(row.author_id) == allow)
                && (!row.word_muted || row.author_id == viewer || Some(row.author_id) == allow)
        })
        .collect();

    let post_ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
    let mut media = media::for_posts(db, &post_ids).await?;
    let mut polls = polls::for_posts(db, viewer, &post_ids).await?;

    let quoted: Vec<Uuid> = rows.iter().filter_map(|row| row.quote_of_id).collect();
    let quotes = load_quotes(db, viewer, &quoted).await?;

    let mut by_id: HashMap<Uuid, PostRow> = rows.into_iter().map(|row| (row.id, row)).collect();
    let mut out = Vec::with_capacity(by_id.len());
    for id in ids {
        let Some(row) = by_id.remove(id) else {
            continue;
        };
        let quote = row.quote_of_id.map(|quote_id| {
            quotes
                .get(&quote_id)
                .cloned()
                .unwrap_or_else(|| unavailable_quote(quote_id))
        });
        let row_media = media.remove(id).unwrap_or_default();
        let row_poll = polls.remove(id);
        out.push(post_dto(row, quote, viewer, row_media, row_poll));
    }
    emoji::hydrate(db, &mut out).await?;
    Ok(out)
}

/// Loads one post as seen by `viewer`; `None` when it does not exist or is hidden.
///
/// # Errors
///
/// Database failures.
pub async fn load_one(
    db: &PgPool,
    viewer: Uuid,
    id: Uuid,
    allow: Option<Uuid>,
) -> Result<Option<PostDto>, AppError> {
    Ok(load(db, viewer, &[id], allow).await?.pop())
}

fn post_dto(
    row: PostRow,
    quote: Option<QuoteDto>,
    viewer: Uuid,
    media: Vec<MediaDto>,
    poll: Option<PollDto>,
) -> PostDto {
    let card = row.card();
    let reply_policy = ReplyPolicy::parse(&row.reply_policy).unwrap_or(ReplyPolicy::Everyone);
    let can_reply = row.author_id == viewer
        || match reply_policy {
            ReplyPolicy::Everyone => true,
            ReplyPolicy::Following => row.author_follows_me,
            ReplyPolicy::Mentioned => row.mentions_me,
        };
    let id = row.id;
    PostDto {
        id: row.id,
        author: row.author(),
        rich_text: segments(&row.text),
        text: row.text,
        cw: row.cw,
        sensitive: row.sensitive,
        created_at: row.created_at,
        edited_at: row.edited_at,
        reply_policy,
        visibility: Visibility::parse(&row.visibility).unwrap_or(Visibility::Public),
        url: row.url,
        client: row.client,
        counts: PostCounts {
            likes: row.like_count,
            reposts: row.repost_count,
            replies: row.reply_count,
            quotes: row.quote_count,
            views: row.views,
        },
        viewer: PostViewer {
            liked: row.liked,
            reposted: row.reposted,
            bookmarked: row.bookmarked,
        },
        can_reply,
        reply_to: row
            .reply_to_id
            .zip(row.reply_to_handle)
            .map(|(id, handle)| ReplyToDto {
                id,
                handle: Some(handle),
            }),
        quote,
        media,
        card,
        poll,
        hidden_reply: row.hidden_reply,
        pinned: false,
        entry_id: id,
        reposted_by: None,
        emojis: Vec::new(),
    }
}

#[allow(
    clippy::struct_excessive_bools,
    reason = "one column per flag the query computes"
)]
#[derive(FromRow)]
struct QuoteRow {
    id: Uuid,
    author_id: Uuid,
    text: String,
    cw: String,
    sensitive: bool,
    created_at: OffsetDateTime,
    a_handle: String,
    a_domain: Option<String>,
    a_display_name: String,
    a_avatar_id: Option<Uuid>,
    a_avatar_shape: String,
    a_ink: String,
    a_verified: bool,
    a_bot: bool,
    hidden: bool,
}

async fn load_quotes(
    db: &PgPool,
    viewer: Uuid,
    ids: &[Uuid],
) -> Result<HashMap<Uuid, QuoteDto>, AppError> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let visible = visible_author("a", "$1");
    let post_visible = visible_post("p", "$1");
    let sql = format!(
        "select p.id, p.author_id, p.text, p.cw, p.sensitive, p.created_at,
            a.handle as a_handle, a.domain as a_domain, a.display_name as a_display_name,
            a.avatar_id as a_avatar_id, a.avatar_shape as a_avatar_shape, a.ink as a_ink,
            a.verified as a_verified, coalesce(a.is_bot, false) as a_bot,
            not {visible} as hidden
         from posts p join users a on a.id = p.author_id
         where p.id = any($2) and p.deleted_at is null and {post_visible}"
    );
    let rows: Vec<QuoteRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(ids)
        .fetch_all(db)
        .await?;
    Ok(rows
        .into_iter()
        .filter(|row| !row.hidden)
        .map(|row| {
            let author = author_ref(
                row.author_id,
                &row.a_handle,
                row.a_domain.as_deref(),
                &row.a_display_name,
                row.a_avatar_id,
                &row.a_avatar_shape,
                &row.a_ink,
                row.a_verified,
                row.a_bot,
            );
            (
                row.id,
                QuoteDto {
                    id: row.id,
                    available: true,
                    author: Some(author),
                    rich_text: segments(&row.text),
                    text: row.text,
                    cw: row.cw,
                    sensitive: row.sensitive,
                    created_at: row.created_at,
                    emojis: Vec::new(),
                },
            )
        })
        .collect())
}

fn unavailable_quote(id: Uuid) -> QuoteDto {
    QuoteDto {
        id,
        available: false,
        author: None,
        text: String::new(),
        rich_text: Vec::new(),
        cw: String::new(),
        sensitive: false,
        created_at: OffsetDateTime::UNIX_EPOCH,
        emojis: Vec::new(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn segments_cover_every_kind() {
        let got = segments("hi @ana #Rust https://a.example/x");
        assert!(matches!(&got[1], SegmentDto::Mention { handle, domain: None } if handle == "ana"));
        assert!(
            matches!(&got[3], SegmentDto::Hashtag { tag, text } if tag == "rust" && text == "Rust")
        );
        assert!(matches!(&got[5], SegmentDto::Link { url } if url == "https://a.example/x"));
    }
}
