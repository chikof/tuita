//! What the ActivityPub JSON of a local account or post is built from: the rows
//! [`crate::federation::render`] and [`crate::federation::mastodon`] read, and the lists the
//! public collections page through. Everything here is public data of local accounts; remote
//! posts are never loaded (the author join keeps `users.domain is null`).

use std::collections::{HashMap, HashSet};

use sqlx::{FromRow, PgConnection, PgExecutor, PgPool};
use time::OffsetDateTime;
use tuita_core::emoji::shortcodes;
use tuita_core::text::parse_post;
use uuid::Uuid;

use super::users::{COLS, UserRow};
use crate::dto::Visibility;
use crate::error::AppError;

/// Most mentions of one post that are looked up.
const MAX_MENTIONS: usize = 50;

/// A stored media file: its name under `/media/` and its type.
#[derive(Clone, Debug)]
pub struct FileRef {
    /// File name, `{id}.{ext}`.
    pub path: String,
    /// Media type, such as `image/png`.
    pub mime: String,
}

/// A picture or video attached to a post.
#[derive(Clone, Debug)]
pub struct Attachment {
    /// Media id.
    pub id: Uuid,
    /// The file.
    pub file: FileRef,
    /// `image` or `video`.
    pub kind: String,
    /// Alt text; may be empty.
    pub alt: String,
    /// Width in pixels.
    pub width: Option<i32>,
    /// Height in pixels.
    pub height: Option<i32>,
    /// File name of a video's poster image.
    pub poster: Option<String>,
}

/// A poll as stored.
#[derive(Clone, Debug)]
pub struct PollData {
    /// Option texts with their vote counts.
    pub options: Vec<(String, i32)>,
    /// Whether several options may be picked.
    pub multiple: bool,
    /// When voting ends.
    pub ends_at: OffsetDateTime,
    /// Accounts that voted.
    pub voters: i32,
}

/// The post a reply answers or a quote shows: enough to name it and its author.
#[derive(Clone, Debug)]
pub struct PostRef {
    /// Post id.
    pub id: Uuid,
    /// The author's account id.
    pub author_id: Uuid,
    /// The author's handle.
    pub handle: String,
    /// The author's home domain; `None` for a local author.
    pub domain: Option<String>,
    /// The post's `ap_id`; `None` for a local post.
    pub ap_id: Option<String>,
    /// The author's `ap_id`; `None` for a local author.
    pub actor_ap_id: Option<String>,
}

/// An account a post mentions, resolved from its text.
#[derive(Clone, Debug)]
pub struct Mentioned {
    /// Account id.
    pub id: Uuid,
    /// The handle as stored.
    pub handle: String,
    /// The home domain; `None` for a local account.
    pub domain: Option<String>,
    /// A remote account's actor id.
    pub ap_id: Option<String>,
    /// A remote account's profile page.
    pub url: Option<String>,
}

/// The author of a local post.
#[allow(
    clippy::struct_excessive_bools,
    reason = "one field per account flag a public document shows"
)]
#[derive(Clone, Debug)]
pub struct LocalAuthor {
    /// Account id.
    pub id: Uuid,
    /// Handle.
    pub handle: String,
    /// Display name.
    pub display_name: String,
    /// Biography.
    pub bio: String,
    /// A bot.
    pub bot: bool,
    /// An organization.
    pub org: bool,
    /// Listed in directories.
    pub discoverable: bool,
    /// Sign-up time.
    pub created_at: OffsetDateTime,
    /// Followers counter.
    pub followers: i32,
    /// Following counter.
    pub following: i32,
    /// Posts counter.
    pub posts: i32,
    /// Suspended by staff.
    pub suspended: bool,
    /// Deleted by its owner.
    pub deleted: bool,
    /// Avatar file.
    pub avatar: Option<FileRef>,
    /// Banner file.
    pub banner: Option<FileRef>,
}

/// A post of a local account with everything its `Note` needs.
#[derive(Clone, Debug)]
pub struct LocalPost {
    /// Post id.
    pub id: Uuid,
    /// The author.
    pub author: LocalAuthor,
    /// Plain text (empty once deleted).
    pub text: String,
    /// Content warning.
    pub cw: String,
    /// Media is sensitive.
    pub sensitive: bool,
    /// Creation time.
    pub created_at: OffsetDateTime,
    /// Last edit.
    pub edited_at: Option<OffsetDateTime>,
    /// Deletion time.
    pub deleted_at: Option<OffsetDateTime>,
    /// Who may see it.
    pub visibility: Visibility,
    /// Posting client.
    pub client: Option<String>,
    /// View counter.
    pub views: i64,
    /// Likes counter.
    pub like_count: i32,
    /// Reposts counter.
    pub repost_count: i32,
    /// Replies counter.
    pub reply_count: i32,
    /// Quotes counter.
    pub quote_count: i32,
    /// The post it answers.
    pub reply_to: Option<PostRef>,
    /// The post it quotes.
    pub quote_of: Option<PostRef>,
    /// Attached media, in order.
    pub media: Vec<Attachment>,
    /// The poll, when it has one.
    pub poll: Option<PollData>,
    /// Accounts its text mentions.
    pub mentions: Vec<Mentioned>,
    /// The local emotes its text, content warning and poll options mention.
    pub emojis: Vec<EmojiRef>,
}

impl LocalPost {
    /// Whether the author's pages are visible to the public: not suspended, not deleted.
    #[must_use]
    pub fn author_is_live(&self) -> bool {
        !self.author.suspended && !self.author.deleted
    }

    /// Whether anyone with the link may read the post: live, public or unlisted.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.deleted_at.is_none()
            && self.author_is_live()
            && matches!(self.visibility, Visibility::Public | Visibility::Unlisted)
    }
}

#[allow(
    clippy::struct_excessive_bools,
    reason = "one column per flag the query computes"
)]
#[derive(FromRow)]
struct Row {
    id: Uuid,
    text: String,
    cw: String,
    sensitive: bool,
    created_at: OffsetDateTime,
    edited_at: Option<OffsetDateTime>,
    deleted_at: Option<OffsetDateTime>,
    visibility: String,
    client: Option<String>,
    views: i64,
    like_count: i32,
    repost_count: i32,
    reply_count: i32,
    quote_count: i32,
    a_id: Uuid,
    a_handle: String,
    a_display_name: String,
    a_bio: String,
    a_bot: bool,
    a_org: bool,
    a_discoverable: bool,
    a_created_at: OffsetDateTime,
    a_followers: i32,
    a_following: i32,
    a_posts: i32,
    a_suspended: bool,
    a_deleted: bool,
    a_avatar: Option<String>,
    a_avatar_mime: Option<String>,
    a_banner: Option<String>,
    a_banner_mime: Option<String>,
    rp_id: Option<Uuid>,
    rp_author_id: Option<Uuid>,
    ra_handle: Option<String>,
    ra_domain: Option<String>,
    rp_ap_id: Option<String>,
    ra_ap_id: Option<String>,
    qp_id: Option<Uuid>,
    qp_author_id: Option<Uuid>,
    qa_handle: Option<String>,
    qa_domain: Option<String>,
    qp_ap_id: Option<String>,
    qa_ap_id: Option<String>,
}

fn file(path: Option<String>, mime: Option<String>) -> Option<FileRef> {
    Some(FileRef {
        path: path?,
        mime: mime?,
    })
}

impl Row {
    fn reply_to(&self) -> Option<PostRef> {
        Some(PostRef {
            id: self.rp_id?,
            author_id: self.rp_author_id?,
            handle: self.ra_handle.clone()?,
            domain: self.ra_domain.clone(),
            ap_id: self.rp_ap_id.clone(),
            actor_ap_id: self.ra_ap_id.clone(),
        })
    }

    fn quote_of(&self) -> Option<PostRef> {
        Some(PostRef {
            id: self.qp_id?,
            author_id: self.qp_author_id?,
            handle: self.qa_handle.clone()?,
            domain: self.qa_domain.clone(),
            ap_id: self.qp_ap_id.clone(),
            actor_ap_id: self.qa_ap_id.clone(),
        })
    }

    fn author(&self) -> LocalAuthor {
        LocalAuthor {
            id: self.a_id,
            handle: self.a_handle.clone(),
            display_name: self.a_display_name.clone(),
            bio: self.a_bio.clone(),
            bot: self.a_bot,
            org: self.a_org,
            discoverable: self.a_discoverable,
            created_at: self.a_created_at,
            followers: self.a_followers,
            following: self.a_following,
            posts: self.a_posts,
            suspended: self.a_suspended,
            deleted: self.a_deleted,
            avatar: file(self.a_avatar.clone(), self.a_avatar_mime.clone()),
            banner: file(self.a_banner.clone(), self.a_banner_mime.clone()),
        }
    }
}

const POST_SQL: &str = "select p.id, p.text, p.cw, p.sensitive, p.created_at, p.edited_at,
        p.deleted_at, p.visibility, p.client, coalesce(p.views, 0) as views,
        p.like_count, p.repost_count, p.reply_count, p.quote_count,
        a.id as a_id, a.handle as a_handle, a.display_name as a_display_name, a.bio as a_bio,
        coalesce(a.is_bot, false) as a_bot, coalesce(a.is_org, false) as a_org,
        coalesce(a.discoverable, true) as a_discoverable, a.created_at as a_created_at,
        a.followers_count as a_followers, a.following_count as a_following,
        a.posts_count as a_posts, a.suspended_at is not null as a_suspended,
        a.deleted_at is not null as a_deleted,
        av.path as a_avatar, av.mime as a_avatar_mime, bn.path as a_banner, bn.mime as a_banner_mime,
        rp.id as rp_id, rp.author_id as rp_author_id, ra.handle as ra_handle,
        ra.domain as ra_domain, rp.ap_id as rp_ap_id, ra.ap_id as ra_ap_id,
        qp.id as qp_id, qp.author_id as qp_author_id, qa.handle as qa_handle,
        qa.domain as qa_domain, qp.ap_id as qp_ap_id, qa.ap_id as qa_ap_id
    from posts p join users a on a.id = p.author_id
    left join media av on av.id = a.avatar_id
    left join media bn on bn.id = a.banner_id
    left join posts rp on rp.id = p.reply_to_id left join users ra on ra.id = rp.author_id
    left join posts qp on qp.id = p.quote_of_id left join users qa on qa.id = qp.author_id
    where p.id = any($1) and a.domain is null";

/// The key a mention is looked up by: lower-case handle and the domain, empty for this
/// instance (a missing domain, or the instance's own).
fn mention_key(handle: &str, domain: Option<&str>, local_domain: &str) -> (String, String) {
    let domain = domain
        .filter(|domain| !domain.eq_ignore_ascii_case(local_domain))
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    (handle.to_ascii_lowercase(), domain)
}

/// The accounts mentioned in each of `texts`, in the order the text names them; mentions of
/// unknown, suspended and deleted accounts are left out.
///
/// # Errors
///
/// Database failures.
pub async fn resolve_mentions<'e>(
    db: impl PgExecutor<'e>,
    local_domain: &str,
    texts: &[&str],
) -> Result<Vec<Vec<Mentioned>>, AppError> {
    let wanted: Vec<Vec<(String, String)>> = texts
        .iter()
        .map(|text| {
            parse_post(text)
                .mentions()
                .into_iter()
                .take(MAX_MENTIONS)
                .map(|(handle, domain)| mention_key(handle, domain, local_domain))
                .collect()
        })
        .collect();
    let mut handles = Vec::new();
    let mut domains = Vec::new();
    let mut seen = HashSet::new();
    for key in wanted.iter().flatten() {
        if seen.insert(key) {
            handles.push(key.0.clone());
            domains.push(key.1.clone());
        }
    }
    if handles.is_empty() {
        return Ok(vec![Vec::new(); texts.len()]);
    }
    #[derive(FromRow)]
    struct Found {
        id: Uuid,
        handle: String,
        domain: Option<String>,
        ap_id: Option<String>,
        url: Option<String>,
    }
    let found: Vec<Found> = sqlx::query_as(
        "select u.id, u.handle, u.domain, u.ap_id, u.url
           from users u join unnest($1::text[], $2::text[]) as m(h, d)
             on lower(u.handle) = m.h and coalesce(u.domain, '') = m.d
          where u.suspended_at is null and u.deleted_at is null",
    )
    .bind(&handles)
    .bind(&domains)
    .fetch_all(db)
    .await?;
    let by_key: HashMap<(String, String), Mentioned> = found
        .into_iter()
        .map(|row| {
            let key = (
                row.handle.to_ascii_lowercase(),
                row.domain.clone().unwrap_or_default(),
            );
            let mentioned = Mentioned {
                id: row.id,
                handle: row.handle,
                domain: row.domain,
                ap_id: row.ap_id,
                url: row.url,
            };
            (key, mentioned)
        })
        .collect();
    Ok(wanted
        .iter()
        .map(|keys| {
            keys.iter()
                .filter_map(|key| by_key.get(key))
                .cloned()
                .collect()
        })
        .collect())
}

/// The accounts mentioned in `text`; see `resolve_mentions`. `local_domain` is this
/// instance's federation domain.
///
/// # Errors
///
/// Database failures.
pub async fn mentions_in<'e>(
    db: impl PgExecutor<'e>,
    local_domain: &str,
    text: &str,
) -> Result<Vec<Mentioned>, AppError> {
    Ok(resolve_mentions(db, local_domain, &[text])
        .await?
        .pop()
        .unwrap_or_default())
}

/// A local emote and its image.
#[derive(Clone, Debug)]
pub struct EmojiRef {
    /// The shortcode as the emote is spelled, without colons.
    pub shortcode: String,
    /// The image.
    pub file: FileRef,
}

/// The local emotes the shortcodes in `texts` name, one query. Order is not meaningful; pick
/// per text with [`mentioned_emojis`].
///
/// # Errors
///
/// Database failures.
pub async fn local_emojis<'e>(
    db: impl PgExecutor<'e>,
    texts: &[&str],
) -> Result<Vec<EmojiRef>, AppError> {
    let mut codes: Vec<String> = texts
        .iter()
        .flat_map(|text| shortcodes(text))
        .map(str::to_ascii_lowercase)
        .collect();
    codes.sort_unstable();
    codes.dedup();
    if codes.is_empty() {
        return Ok(Vec::new());
    }
    #[derive(FromRow)]
    struct Found {
        shortcode: String,
        path: String,
        mime: String,
    }
    let found: Vec<Found> = sqlx::query_as(
        "select e.shortcode, m.path, m.mime
           from custom_emojis e join media m on m.id = e.media_id
          where e.domain is null and lower(e.shortcode) = any($1)",
    )
    .bind(&codes)
    .fetch_all(db)
    .await?;
    Ok(found
        .into_iter()
        .map(|row| EmojiRef {
            shortcode: row.shortcode,
            file: FileRef {
                path: row.path,
                mime: row.mime,
            },
        })
        .collect())
}

/// The emotes of `catalog` that `texts` mention, each once, in order of first appearance.
#[must_use]
pub fn mentioned_emojis(catalog: &[EmojiRef], texts: &[&str]) -> Vec<EmojiRef> {
    let mut out: Vec<EmojiRef> = Vec::new();
    for code in texts.iter().flat_map(|text| shortcodes(text)) {
        let found = catalog
            .iter()
            .find(|emoji| emoji.shortcode.eq_ignore_ascii_case(code));
        if let Some(emoji) = found
            && !out.iter().any(|seen| seen.shortcode == emoji.shortcode)
        {
            out.push(emoji.clone());
        }
    }
    out
}

/// Loads the posts of local accounts among `ids`, deleted ones included, in the order given.
/// Unknown ids and posts of remote accounts are left out. `local_domain` is this instance's
/// federation domain.
///
/// # Errors
///
/// Database failures.
pub async fn load_posts(
    db: &PgPool,
    local_domain: &str,
    ids: &[Uuid],
) -> Result<Vec<LocalPost>, AppError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut conn = db.acquire().await?;
    load_posts_in(&mut conn, local_domain, ids).await
}

/// [`load_posts`] on the caller's connection, so a transaction sees its own writes.
///
/// # Errors
///
/// Database failures.
pub async fn load_posts_in(
    db: &mut PgConnection,
    local_domain: &str,
    ids: &[Uuid],
) -> Result<Vec<LocalPost>, AppError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows: Vec<Row> = sqlx::query_as(POST_SQL)
        .bind(ids)
        .fetch_all(&mut *db)
        .await?;

    #[derive(FromRow)]
    struct MediaRow {
        post_id: Uuid,
        id: Uuid,
        path: String,
        mime: String,
        kind: String,
        alt: String,
        width: Option<i32>,
        height: Option<i32>,
        poster: Option<String>,
    }
    let media: Vec<MediaRow> = sqlx::query_as(
        "select pm.post_id, m.id, m.path, m.mime, m.kind, m.alt, m.width, m.height,
                pp.path as poster
           from post_media pm join media m on m.id = pm.media_id
           left join media pp on pp.id = m.poster_id
          where pm.post_id = any($1) order by pm.post_id, pm.position",
    )
    .bind(ids)
    .fetch_all(&mut *db)
    .await?;
    let mut media_by_post: HashMap<Uuid, Vec<Attachment>> = HashMap::new();
    for row in media {
        media_by_post
            .entry(row.post_id)
            .or_default()
            .push(Attachment {
                id: row.id,
                file: FileRef {
                    path: row.path,
                    mime: row.mime,
                },
                kind: row.kind,
                alt: row.alt,
                width: row.width,
                height: row.height,
                poster: row.poster,
            });
    }

    #[derive(FromRow)]
    struct PollRow {
        post_id: Uuid,
        options: Vec<String>,
        multiple: bool,
        ends_at: OffsetDateTime,
        counts: Vec<i32>,
        voters: i32,
    }
    let polls: Vec<PollRow> = sqlx::query_as(
        "select post_id, options, multiple, ends_at, counts, voters
           from polls where post_id = any($1)",
    )
    .bind(ids)
    .fetch_all(&mut *db)
    .await?;
    let mut poll_by_post: HashMap<Uuid, PollData> = polls
        .into_iter()
        .map(|row| {
            let options = row
                .options
                .into_iter()
                .zip(row.counts.into_iter().chain(std::iter::repeat(0)))
                .collect();
            (
                row.post_id,
                PollData {
                    options,
                    multiple: row.multiple,
                    ends_at: row.ends_at,
                    voters: row.voters,
                },
            )
        })
        .collect();

    let mut emoji_texts: Vec<&str> = rows
        .iter()
        .flat_map(|row| [row.text.as_str(), row.cw.as_str()])
        .collect();
    emoji_texts.extend(
        poll_by_post
            .values()
            .flat_map(|poll| poll.options.iter().map(|(text, _)| text.as_str())),
    );
    let catalog = local_emojis(&mut *db, &emoji_texts).await?;
    let texts: Vec<&str> = rows.iter().map(|row| row.text.as_str()).collect();
    let mentions = resolve_mentions(&mut *db, local_domain, &texts).await?;
    let mut by_id: HashMap<Uuid, LocalPost> = rows
        .iter()
        .zip(mentions)
        .map(|(row, mentions)| {
            let poll = poll_by_post.remove(&row.id);
            let options = poll
                .iter()
                .flat_map(|poll| poll.options.iter().map(|(text, _)| text.as_str()));
            let own: Vec<&str> = [row.text.as_str(), row.cw.as_str()]
                .into_iter()
                .chain(options)
                .collect();
            let emojis = mentioned_emojis(&catalog, &own);
            (
                row.id,
                LocalPost {
                    id: row.id,
                    author: row.author(),
                    text: row.text.clone(),
                    cw: row.cw.clone(),
                    sensitive: row.sensitive,
                    created_at: row.created_at,
                    edited_at: row.edited_at,
                    deleted_at: row.deleted_at,
                    visibility: Visibility::parse(&row.visibility).unwrap_or(Visibility::Public),
                    client: row.client.clone(),
                    views: row.views,
                    like_count: row.like_count,
                    repost_count: row.repost_count,
                    reply_count: row.reply_count,
                    quote_count: row.quote_count,
                    reply_to: row.reply_to(),
                    quote_of: row.quote_of(),
                    media: media_by_post.remove(&row.id).unwrap_or_default(),
                    poll,
                    emojis,
                    mentions,
                },
            )
        })
        .collect();
    Ok(ids.iter().filter_map(|id| by_id.remove(id)).collect())
}

/// One post of a local account; see [`load_posts`].
///
/// # Errors
///
/// Database failures.
pub async fn load_post(
    db: &PgPool,
    local_domain: &str,
    id: Uuid,
) -> Result<Option<LocalPost>, AppError> {
    Ok(load_posts(db, local_domain, &[id]).await?.pop())
}

/// [`load_post`] on the caller's connection.
///
/// # Errors
///
/// Database failures.
pub async fn load_post_in(
    db: &mut PgConnection,
    local_domain: &str,
    id: Uuid,
) -> Result<Option<LocalPost>, AppError> {
    Ok(load_posts_in(db, local_domain, &[id]).await?.pop())
}

/// The local account `handle` names, whatever its state; `None` for no such local account. The
/// `tuita` system account is not an account here: it is never served over ActivityPub.
///
/// # Errors
///
/// Database failures.
pub async fn local_user(db: &PgPool, handle: &str) -> Result<Option<UserRow>, AppError> {
    let sql = format!(
        "select {COLS} from users u where lower(u.handle) = lower($1) and u.domain is null
           and lower(u.handle) <> 'tuita'"
    );
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(handle)
        .fetch_optional(db)
        .await?)
}

/// The avatar and banner files of `user`.
///
/// # Errors
///
/// Database failures.
pub async fn user_files<'e>(
    db: impl PgExecutor<'e>,
    user: &UserRow,
) -> Result<(Option<FileRef>, Option<FileRef>), AppError> {
    let wanted: Vec<Uuid> = [user.avatar_id, user.banner_id]
        .into_iter()
        .flatten()
        .collect();
    if wanted.is_empty() {
        return Ok((None, None));
    }
    let rows: Vec<(Uuid, String, String)> =
        sqlx::query_as("select id, path, mime from media where id = any($1)")
            .bind(&wanted)
            .fetch_all(db)
            .await?;
    let find = |id: Option<Uuid>| {
        let id = id?;
        rows.iter()
            .find(|(row, _, _)| *row == id)
            .map(|(_, path, mime)| FileRef {
                path: path.clone(),
                mime: mime.clone(),
            })
    };
    Ok((find(user.avatar_id), find(user.banner_id)))
}

/// How many public, live posts `author` has: the outbox's `totalItems`.
///
/// # Errors
///
/// Database failures.
pub async fn outbox_total(db: &PgPool, author: Uuid) -> Result<i64, AppError> {
    Ok(sqlx::query_scalar(
        "select count(*) from posts
          where author_id = $1 and deleted_at is null and visibility = 'public'",
    )
    .bind(author)
    .fetch_one(db)
    .await?)
}

/// Up to `limit` ids of `author`'s public, live posts, newest first, older than `before`.
///
/// # Errors
///
/// Database failures.
pub async fn outbox_ids(
    db: &PgPool,
    author: Uuid,
    before: Option<Uuid>,
    limit: i64,
) -> Result<Vec<Uuid>, AppError> {
    Ok(sqlx::query_scalar(
        "select id from posts
          where author_id = $1 and deleted_at is null and visibility = 'public'
            and ($2::uuid is null or id < $2)
          order by id desc limit $3",
    )
    .bind(author)
    .bind(before)
    .bind(limit)
    .fetch_all(db)
    .await?)
}

/// The id of `author`'s pinned post when anyone with the link may read it.
///
/// # Errors
///
/// Database failures.
pub async fn pinned_open(db: &PgPool, author: &UserRow) -> Result<Option<Uuid>, AppError> {
    let Some(pinned) = author.pinned_post_id else {
        return Ok(None);
    };
    Ok(sqlx::query_scalar(
        "select id from posts where id = $1 and author_id = $2 and deleted_at is null
            and visibility in ('public', 'unlisted')",
    )
    .bind(pinned)
    .bind(author.id)
    .fetch_optional(db)
    .await?)
}

/// What NodeInfo reports: active local accounts and their live posts. The `tuita` system
/// account and remote accounts do not count.
///
/// # Errors
///
/// Database failures.
pub async fn usage(db: &PgPool) -> Result<(i64, i64), AppError> {
    let users = sqlx::query_scalar(
        "select count(*) from users
          where domain is null and handle <> 'tuita' and suspended_at is null and deleted_at is null",
    )
    .fetch_one(db)
    .await?;
    let posts = sqlx::query_scalar(
        "select count(*) from posts p join users u on u.id = p.author_id
          where u.domain is null and u.handle <> 'tuita' and p.deleted_at is null",
    )
    .fetch_one(db)
    .await?;
    Ok((users, posts))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn the_local_domain_and_no_domain_share_a_mention_key() {
        let key = mention_key("Ana", None, "a.example");
        assert_eq!(key, ("ana".to_owned(), String::new()));
        assert_eq!(mention_key("ana", Some("A.example"), "a.example"), key);
        assert_eq!(
            mention_key("ana", Some("b.example"), "a.example"),
            ("ana".to_owned(), "b.example".to_owned())
        );
    }
}
