//! The reads behind the Mastodon timelines, bookmarks, favourites, `favourited_by` and the status
//! context.
//!
//! Mastodon pages by `max_id`, `min_id` and `since_id`, tuita's own lists only by `before`, so the
//! timeline queries here take a [`Range`] (an exclusive upper and lower bound and a direction).
//! They use the same visibility fragments as [`super::timeline`], [`super::lists`] and
//! [`super::search`], so every block, mute, word filter and silenced-domain rule applies alike.
//! Each query returns raw rows, newest first or oldest first as asked; the caller trims, builds
//! the `Link` header from the row cursors, and then turns the rows into posts with [`posts_of`].

use std::collections::HashMap;

use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::users::{COLS, UserRow, refs};
use super::{
    blocked_author, discoverable_post, listable_post, posts, visible_author, visible_post,
    word_visible,
};
use crate::cursor::EdgeCursor;
use crate::dto::PostDto;
use crate::error::AppError;

/// Accounts `$1` follows, as a subquery of ids.
const FOLLOWED: &str =
    "select followee_id from follows where follower_id = $1 and state = 'accepted'";
/// Deepest reply chain the context reads.
const CONTEXT_DEPTH: i32 = 20;
/// Most replies the context reads.
const CONTEXT_LIMIT: i64 = 200;
/// Most ancestors the context reads.
const ANCESTOR_DEPTH: i32 = 25;

/// The id bounds and direction of one page. `lower` is exclusive and so is `upper`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Range<C> {
    /// Rows older than this (`max_id`).
    pub upper: Option<C>,
    /// Rows newer than this (`min_id` or `since_id`).
    pub lower: Option<C>,
    /// Read the rows nearest `lower` first (`min_id`).
    pub ascending: bool,
}

impl<C> Range<C> {
    const fn dir(&self) -> &'static str {
        if self.ascending { "asc" } else { "desc" }
    }
}

/// One entry of a timeline: an original post, or somebody's repost of one.
#[derive(Clone, Copy, Debug, FromRow)]
pub struct Entry {
    /// The post id for an original, the repost row id for a repost: the keyset.
    pub key: Uuid,
    /// The post.
    pub post_id: Uuid,
    /// Who reposted it.
    pub reposter_id: Option<Uuid>,
}

/// The entries of `viewer`'s home timeline: own and followed top-level posts and followed
/// reposts. `fetch` rows at most.
///
/// # Errors
///
/// Database failures.
pub async fn home(
    db: &PgPool,
    viewer: Uuid,
    range: Range<Uuid>,
    fetch: i64,
) -> Result<Vec<Entry>, AppError> {
    let author_ok = visible_author("a", "$1");
    let reposter_ok = visible_author("ru", "$1");
    let words_ok = word_visible("p", "$1");
    let listable = listable_post("p");
    let visible = visible_post("p", "$1");
    let dir = range.dir();
    let sql = format!(
        "select key, post_id, reposter_id from (
            select p.id as key, p.id as post_id, null::uuid as reposter_id
              from posts p join users a on a.id = p.author_id
             where p.deleted_at is null and p.reply_to_id is null
               and (p.author_id = $1 or p.author_id in ({FOLLOWED}))
               and {author_ok} and {words_ok} and {listable} and {visible}
            union all
            select r.id, r.post_id, r.user_id
              from reposts r
              join users ru on ru.id = r.user_id
              join posts p on p.id = r.post_id
              join users a on a.id = p.author_id
             where r.user_id in ({FOLLOWED}) and p.deleted_at is null
               and {reposter_ok} and {author_ok} and {words_ok} and {listable} and {visible}
         ) e
         where ($2::uuid is null or e.key < $2) and ($3::uuid is null or e.key > $3)
         order by e.key {dir} limit $4"
    );
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(range.upper)
        .bind(range.lower)
        .bind(fetch)
        .fetch_all(db)
        .await?)
}

/// The entries of list `list`: its members' top-level posts, their replies to other members and
/// their reposts. The caller checked that the viewer may open the list.
///
/// # Errors
///
/// Database failures.
pub async fn list(
    db: &PgPool,
    viewer: Uuid,
    list: Uuid,
    range: Range<Uuid>,
    fetch: i64,
) -> Result<Vec<Entry>, AppError> {
    const MEMBERS: &str = "select user_id from list_members where list_id = $2";
    let author_ok = visible_author("a", "$1");
    let reposter_ok = visible_author("ru", "$1");
    let words_ok = word_visible("p", "$1");
    let listable = listable_post("p");
    let visible = visible_post("p", "$1");
    let dir = range.dir();
    let sql = format!(
        "select key, post_id, reposter_id from (
            select p.id as key, p.id as post_id, null::uuid as reposter_id
              from posts p join users a on a.id = p.author_id
             where p.deleted_at is null and p.author_id in ({MEMBERS})
               and (p.reply_to_id is null
                    or exists (select 1 from posts rp
                                where rp.id = p.reply_to_id and rp.author_id in ({MEMBERS})))
               and {author_ok} and {words_ok} and {listable} and {visible}
            union all
            select r.id, r.post_id, r.user_id
              from reposts r
              join users ru on ru.id = r.user_id
              join posts p on p.id = r.post_id
              join users a on a.id = p.author_id
             where r.user_id in ({MEMBERS}) and p.deleted_at is null
               and {reposter_ok} and {author_ok} and {words_ok} and {listable} and {visible}
         ) e
         where ($3::uuid is null or e.key < $3) and ($4::uuid is null or e.key > $4)
         order by e.key {dir} limit $5"
    );
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(list)
        .bind(range.upper)
        .bind(range.lower)
        .bind(fetch)
        .fetch_all(db)
        .await?)
}

/// What a public or tag timeline lists.
#[derive(Clone, Debug, Default)]
pub struct PublicScope {
    /// Only accounts of this instance.
    pub local: bool,
    /// Only accounts of other instances.
    pub remote: bool,
    /// Only posts with media.
    pub only_media: bool,
    /// Only posts with this lower-case hashtag.
    pub tag: Option<String>,
}

/// The post ids of the public timeline, or of a tag when `scope.tag` is set: top-level public
/// posts of accounts on no silenced instance, as [`super::timeline::foryou`] and
/// [`super::search::tag_posts`] list them.
///
/// # Errors
///
/// Database failures.
pub async fn public(
    db: &PgPool,
    viewer: Uuid,
    scope: &PublicScope,
    range: Range<Uuid>,
    fetch: i64,
) -> Result<Vec<Uuid>, AppError> {
    let author_ok = visible_author("a", "$1");
    let words_ok = word_visible("p", "$1");
    let discoverable = discoverable_post("p", "a");
    let dir = range.dir();
    let flags = "and (not $4 or a.domain is null) and (not $5 or a.domain is not null)
         and (not $6 or exists (select 1 from post_media pm where pm.post_id = p.id))";
    let bounds = "($2::uuid is null or p.id < $2) and ($3::uuid is null or p.id > $3)";
    let sql = if scope.tag.is_some() {
        format!(
            "select p.id from post_tags pt join posts p on p.id = pt.post_id
               join users a on a.id = p.author_id
             where pt.tag = $7 and p.deleted_at is null and {author_ok} and {words_ok}
               and {discoverable} and {bounds} {flags}
             order by p.id {dir} limit $8"
        )
    } else {
        format!(
            "select p.id from posts p join users a on a.id = p.author_id
             where p.deleted_at is null and p.reply_to_id is null and {author_ok} and {words_ok}
               and {discoverable} and {bounds} {flags}
             order by p.id {dir} limit $7"
        )
    };
    let query = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(range.upper)
        .bind(range.lower)
        .bind(scope.local)
        .bind(scope.remote)
        .bind(scope.only_media);
    let query = match &scope.tag {
        Some(tag) => query.bind(tag.as_str()).bind(fetch),
        None => query.bind(fetch),
    };
    Ok(query.fetch_all(db).await?)
}

/// Loads the posts of `entries` as `viewer` sees them, in entry order, with the repost data set.
/// Entries whose post or reposter the viewer cannot see are dropped.
///
/// # Errors
///
/// Database failures.
pub async fn posts_of(
    db: &PgPool,
    viewer: Uuid,
    entries: &[Entry],
) -> Result<Vec<PostDto>, AppError> {
    let mut post_ids: Vec<Uuid> = entries.iter().map(|entry| entry.post_id).collect();
    post_ids.sort_unstable();
    post_ids.dedup();
    let loaded: HashMap<Uuid, PostDto> = posts::load(db, viewer, &post_ids, None)
        .await?
        .into_iter()
        .map(|post| (post.id, post))
        .collect();
    let reposter_ids: Vec<Uuid> = entries
        .iter()
        .filter_map(|entry| entry.reposter_id)
        .collect();
    let reposters = refs(db, &reposter_ids).await?;
    Ok(entries
        .iter()
        .filter_map(|entry| {
            let mut post = loaded.get(&entry.post_id)?.clone();
            post.entry_id = entry.key;
            if let Some(reposter) = entry.reposter_id {
                post.reposted_by = Some(reposters.get(&reposter)?.clone());
            }
            Some(post)
        })
        .collect())
}

/// Which of the viewer's records a record list reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Record {
    /// `bookmarks`.
    Bookmarks,
    /// `likes`.
    Likes,
}

/// The posts `viewer` bookmarked or liked, as `(post id, record time)`, with the cursor of each
/// row being `EdgeCursor::new(time, post id)`.
///
/// # Errors
///
/// Database failures.
pub async fn records(
    db: &PgPool,
    viewer: Uuid,
    record: Record,
    range: Range<EdgeCursor>,
    fetch: i64,
) -> Result<Vec<(Uuid, OffsetDateTime)>, AppError> {
    let table = match record {
        Record::Bookmarks => "bookmarks",
        Record::Likes => "likes",
    };
    let dir = range.dir();
    let sql = format!(
        "select e.post_id, e.created_at
           from {table} e join posts p on p.id = e.post_id
          where e.user_id = $1 and p.deleted_at is null
            and ($2::timestamptz is null or (e.created_at, e.post_id) < ($2, $3))
            and ($4::timestamptz is null or (e.created_at, e.post_id) > ($4, $5))
          order by e.created_at {dir}, e.post_id {dir} limit $6"
    );
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(range.upper.map(|cursor| cursor.at))
        .bind(range.upper.map(|cursor| cursor.id))
        .bind(range.lower.map(|cursor| cursor.at))
        .bind(range.lower.map(|cursor| cursor.id))
        .bind(fetch)
        .fetch_all(db)
        .await?)
}

/// The accounts that liked or reposted `post`, with the cursor of each row. The caller checked
/// that the viewer may see the post.
///
/// # Errors
///
/// Database failures.
pub async fn reactors(
    db: &PgPool,
    viewer: Uuid,
    post: Uuid,
    record: ReactorTable,
    range: Range<EdgeCursor>,
    fetch: i64,
) -> Result<Vec<(UserRow, EdgeCursor)>, AppError> {
    #[derive(FromRow)]
    struct Row {
        edge_at: OffsetDateTime,
        #[sqlx(flatten)]
        user: UserRow,
    }
    let table = match record {
        ReactorTable::Likes => "likes",
        ReactorTable::Reposts => "reposts",
    };
    let dir = range.dir();
    let sql = format!(
        "select {COLS}, e.created_at as edge_at
           from {table} e join users u on u.id = e.user_id
          where e.post_id = $2 and {}
            and ($3::timestamptz is null or (e.created_at, e.user_id) < ($3, $4))
            and ($5::timestamptz is null or (e.created_at, e.user_id) > ($5, $6))
          order by e.created_at {dir}, e.user_id {dir} limit $7",
        visible_author("u", "$1")
    );
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(post)
        .bind(range.upper.map(|cursor| cursor.at))
        .bind(range.upper.map(|cursor| cursor.id))
        .bind(range.lower.map(|cursor| cursor.at))
        .bind(range.lower.map(|cursor| cursor.id))
        .bind(fetch)
        .fetch_all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let cursor = EdgeCursor::new(row.edge_at, row.user.id);
            (row.user, cursor)
        })
        .collect())
}

/// The table [`reactors`] reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReactorTable {
    /// `likes`.
    Likes,
    /// `reposts`.
    Reposts,
}

/// The ids above and below one post, as `viewer` may see them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ContextIds {
    /// Ancestors, oldest first.
    pub ancestors: Vec<Uuid>,
    /// Replies as `(id, depth)`; at most 200.
    pub descendants: Vec<(Uuid, i32)>,
}

/// The ancestor and reply ids of `post`, written by `author`. Replies the viewer may not open
/// (non-visible posts, blocked authors) are not read, and neither is anything below them; the
/// replies a post's author hid are read only for the viewer who hid them or wrote them.
///
/// # Errors
///
/// Database failures.
pub async fn context(
    db: &PgPool,
    viewer: Uuid,
    post: Uuid,
    author: Uuid,
) -> Result<ContextIds, AppError> {
    let ancestors: Vec<Uuid> = sqlx::query_scalar(
        "with recursive up(id, reply_to_id, depth) as (
            select id, reply_to_id, 0 from posts where id = $1
            union all
            select p.id, p.reply_to_id, up.depth + 1 from posts p
              join up on p.id = up.reply_to_id where up.depth < $2)
         select id from up where depth > 0 order by depth desc",
    )
    .bind(post)
    .bind(ANCESTOR_DEPTH)
    .fetch_all(db)
    .await?;
    let visible = visible_post("c", "$1");
    let blocked = blocked_author("a", "$1");
    let sql = format!(
        "with recursive down(id, author_id, depth) as (
            select c.id, c.author_id, 1 from posts c join users a on a.id = c.author_id
             where c.reply_to_id = $2 and c.deleted_at is null and {visible} and not {blocked}
               and (not coalesce(c.hidden_reply, false) or c.author_id = $1 or $3 = $1)
            union all
            select c.id, c.author_id, d.depth + 1
              from posts c join down d on c.reply_to_id = d.id
              join users a on a.id = c.author_id
             where c.deleted_at is null and {visible} and not {blocked} and d.depth < $4
               and (not coalesce(c.hidden_reply, false) or c.author_id = $1 or d.author_id = $1))
         select id, depth from down limit $5"
    );
    let descendants: Vec<(Uuid, i32)> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(post)
        .bind(author)
        .bind(CONTEXT_DEPTH)
        .bind(CONTEXT_LIMIT)
        .fetch_all(db)
        .await?;
    Ok(ContextIds {
        ancestors,
        descendants,
    })
}
