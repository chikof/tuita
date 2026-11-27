//! Timelines and bookmarks: the following feed, the ranked For you feed with its snapshots, the
//! "new posts" counter and the bookmark list.

use std::collections::{HashMap, HashSet};

use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use tuita_core::feed::{AFFINITY_WINDOW, CANDIDATE_LIMIT, Candidate, FeedCtx, SNAPSHOT_TTL, rank};
use uuid::Uuid;

use super::{
    discoverable_post, listable_post, posts, suspended_domain, users, visible_author, visible_post,
    word_visible,
};
use crate::cursor::{EdgeCursor, Limit, page};
use crate::dto::{Feed, Page, PostDto};
use crate::error::AppError;

/// Posts counted by [`new_count`] at most.
const NEW_COUNT_CAP: i64 = 200;

/// Accounts `$1` follows, as a subquery of ids.
const FOLLOWED: &str =
    "select followee_id from follows where follower_id = $1 and state = 'accepted'";

/// One row of the following feed: an original post, or a repost of one.
#[derive(FromRow)]
struct Entry {
    /// Post id for an original, repost row id for a repost: the keyset of the feed.
    key: Uuid,
    post_id: Uuid,
    reposter_id: Option<Uuid>,
}

/// One page of the following feed: own and followed top-level posts plus followed reposts,
/// newest first. `before` is the key of the last entry of the previous page.
///
/// # Errors
///
/// Database failures.
pub async fn following(
    db: &PgPool,
    viewer: Uuid,
    before: Option<Uuid>,
    limit: Limit,
) -> Result<Page<PostDto>, AppError> {
    let author_ok = visible_author("a", "$1");
    let reposter_ok = visible_author("ru", "$1");
    let words_ok = word_visible("p", "$1");
    let listable = listable_post("p");
    let visible = visible_post("p", "$1");
    let sql = format!(
        "select key, post_id, reposter_id from (
            select p.id as key, p.id as post_id, null::uuid as reposter_id
              from posts p join users a on a.id = p.author_id
             where p.deleted_at is null and p.reply_to_id is null
               and (p.author_id = $1 or p.author_id in ({FOLLOWED}))
               and {author_ok}
               and {words_ok} and {listable} and {visible}
            union all
            select r.id, r.post_id, r.user_id
              from reposts r
              join users ru on ru.id = r.user_id
              join posts p on p.id = r.post_id
              join users a on a.id = p.author_id
             where r.user_id in ({FOLLOWED}) and p.deleted_at is null
               and {reposter_ok} and {author_ok} and {words_ok} and {listable} and {visible}
         ) e
         where $2::uuid is null or e.key < $2
         order by e.key desc limit $3"
    );
    let rows: Vec<Entry> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(before)
        .bind(limit.fetch())
        .fetch_all(db)
        .await?;
    let entries = page(rows, limit, |entry| entry.key.to_string());

    let mut post_ids: Vec<Uuid> = entries.items.iter().map(|entry| entry.post_id).collect();
    post_ids.sort_unstable();
    post_ids.dedup();
    let loaded: HashMap<Uuid, PostDto> = posts::load(db, viewer, &post_ids, None)
        .await?
        .into_iter()
        .map(|post| (post.id, post))
        .collect();
    let reposter_ids: Vec<Uuid> = entries
        .items
        .iter()
        .filter_map(|entry| entry.reposter_id)
        .collect();
    let reposters = users::refs(db, &reposter_ids).await?;

    let items = entries
        .items
        .iter()
        .filter_map(|entry| {
            let mut post = loaded.get(&entry.post_id)?.clone();
            post.entry_id = entry.key;
            if let Some(reposter) = entry.reposter_id {
                post.reposted_by = Some(reposters.get(&reposter)?.clone());
            }
            Some(post)
        })
        .collect();
    Ok(Page {
        items,
        next: entries.next,
    })
}

/// One page of the For you feed. `offset` 0 ranks afresh and stores the ranking; a later offset
/// continues the stored ranking until it is older than [`SNAPSHOT_TTL`].
///
/// # Errors
///
/// Database failures.
pub async fn foryou(
    db: &PgPool,
    viewer: Uuid,
    offset: i64,
    limit: Limit,
) -> Result<Page<PostDto>, AppError> {
    let ids = snapshot(db, viewer, offset).await?;
    let start = usize::try_from(offset).unwrap_or(usize::MAX);
    let end = start.saturating_add(limit.size());
    let slice: Vec<Uuid> = ids.iter().skip(start).take(limit.size()).copied().collect();
    let next = (ids.len() > end).then(|| offset.saturating_add(i64::from(limit.get())).to_string());
    let items = posts::load(db, viewer, &slice, None).await?;
    Ok(Page { items, next })
}

async fn snapshot(db: &PgPool, viewer: Uuid, offset: i64) -> Result<Vec<Uuid>, AppError> {
    if offset > 0 {
        let stored: Option<(Vec<Uuid>, OffsetDateTime)> =
            sqlx::query_as("select ids, created_at from feed_snapshots where user_id = $1")
                .bind(viewer)
                .fetch_optional(db)
                .await?;
        if let Some((ids, at)) = stored
            && OffsetDateTime::now_utc() - at < SNAPSHOT_TTL
        {
            return Ok(ids);
        }
    }
    let ids = ranking(db, viewer).await?;
    sqlx::query(
        "insert into feed_snapshots (user_id, ids, created_at) values ($1, $2, now())
         on conflict (user_id) do update set ids = excluded.ids, created_at = excluded.created_at",
    )
    .bind(viewer)
    .bind(&ids)
    .execute(db)
    .await?;
    Ok(ids)
}

#[derive(FromRow)]
struct CandidateRow {
    id: Uuid,
    author_id: Uuid,
    created_at: OffsetDateTime,
    views: i64,
    likes: i32,
    reposts: i32,
    replies: i32,
    quotes: i32,
    followed_likes: i32,
    followed_reposts: i32,
    has_media: bool,
    is_quote: bool,
    engaged: bool,
}

async fn ranking(db: &PgPool, viewer: Uuid) -> Result<Vec<Uuid>, AppError> {
    let now = OffsetDateTime::now_utc();
    let words = word_visible("p", "$1");
    let suspended = suspended_domain("a");
    let discoverable = discoverable_post("p", "a");
    let sql = format!(
        "select p.id, p.author_id, p.created_at, coalesce(p.views, 0)::int8 as views,
                p.like_count as likes, p.repost_count as reposts,
                p.reply_count as replies, p.quote_count as quotes,
                (select count(*)::int4 from likes l join follows f
                   on f.followee_id = l.user_id and f.follower_id = $1 and f.state = 'accepted'
                  where l.post_id = p.id) as followed_likes,
                (select count(*)::int4 from reposts r join follows f
                   on f.followee_id = r.user_id and f.follower_id = $1 and f.state = 'accepted'
                  where r.post_id = p.id) as followed_reposts,
                exists (select 1 from post_media m where m.post_id = p.id) as has_media,
                p.quote_of_id is not null as is_quote,
                (exists (select 1 from likes l where l.post_id = p.id and l.user_id = $1)
                 or exists (select 1 from reposts r where r.post_id = p.id and r.user_id = $1))
                    as engaged
           from posts p join users a on a.id = p.author_id
          where p.deleted_at is null and p.reply_to_id is null
            and a.suspended_at is null and a.deleted_at is null and {words}
            and not {suspended} and {discoverable}
          order by p.id desc limit $2"
    );
    let rows: Vec<CandidateRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(i64::try_from(CANDIDATE_LIMIT).unwrap_or(i64::MAX))
        .fetch_all(db)
        .await?;
    let candidates: Vec<Candidate> = rows
        .into_iter()
        .map(|row| Candidate {
            id: row.id,
            author_id: row.author_id,
            created_at: row.created_at,
            views: row.views,
            likes: row.likes,
            reposts: row.reposts,
            replies: row.replies,
            quotes: row.quotes,
            followed_likes: row.followed_likes,
            followed_reposts: row.followed_reposts,
            has_media: row.has_media,
            is_quote: row.is_quote,
            engaged: row.engaged,
        })
        .collect();

    let followed: Vec<Uuid> = sqlx::query_scalar(FOLLOWED)
        .bind(viewer)
        .fetch_all(db)
        .await?;
    let hidden: Vec<Uuid> = sqlx::query_scalar(
        "select target_id from blocks where user_id = $1
         union select user_id from blocks where target_id = $1
         union select target_id from mutes where user_id = $1",
    )
    .bind(viewer)
    .fetch_all(db)
    .await?;
    let ctx = FeedCtx {
        me: viewer,
        now,
        followed: followed.into_iter().collect(),
        hidden: hidden.into_iter().collect::<HashSet<_>>(),
        affinity: affinities(db, viewer, now - AFFINITY_WINDOW).await?,
    };
    Ok(rank(&candidates, &ctx))
}

/// How often `viewer` liked, reposted or replied to each author since `since`.
async fn affinities(
    db: &PgPool,
    viewer: Uuid,
    since: OffsetDateTime,
) -> Result<HashMap<Uuid, u32>, AppError> {
    let rows: Vec<(Uuid, i64)> = sqlx::query_as(
        "select author_id, count(*) from (
            select p.author_id from likes l join posts p on p.id = l.post_id
             where l.user_id = $1 and l.created_at > $2
            union all
            select p.author_id from reposts r join posts p on p.id = r.post_id
             where r.user_id = $1 and r.created_at > $2
            union all
            select t.author_id from posts mine join posts t on t.id = mine.reply_to_id
             where mine.author_id = $1 and mine.created_at > $2 and mine.deleted_at is null
         ) x group by author_id",
    )
    .bind(viewer)
    .bind(since)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(author, count)| (author, u32::try_from(count).unwrap_or(u32::MAX)))
        .collect())
}

/// How many entries of `feed` are newer than `since` (a post or repost id), at most 200.
///
/// # Errors
///
/// Database failures.
pub async fn new_count(
    db: &PgPool,
    viewer: Uuid,
    feed: Feed,
    since: Uuid,
) -> Result<i64, AppError> {
    let words_ok = word_visible("p", "$1");
    let author_ok = visible_author("a", "$1");
    let listable = listable_post("p");
    let visible = visible_post("p", "$1");
    let discoverable = discoverable_post("p", "a");
    let sql = match feed {
        Feed::Following => {
            let reposter_ok = visible_author("ru", "$1");
            format!(
                "select count(*) from (
                    select 1 from posts p join users a on a.id = p.author_id
                     where p.deleted_at is null and p.reply_to_id is null and p.id > $2
                       and p.author_id <> $1 and p.author_id in ({FOLLOWED}) and {author_ok} and {words_ok}
                       and {listable} and {visible}
                    union all
                    select 1 from reposts r
                      join users ru on ru.id = r.user_id
                      join posts p on p.id = r.post_id
                      join users a on a.id = p.author_id
                     where r.id > $2 and r.user_id in ({FOLLOWED}) and p.deleted_at is null
                       and {reposter_ok} and {author_ok} and {words_ok} and {listable} and {visible}
                    limit {NEW_COUNT_CAP}
                 ) n"
            )
        }
        Feed::Foryou => format!(
            "select count(*) from (
                select 1 from posts p join users a on a.id = p.author_id
                 where p.deleted_at is null and p.reply_to_id is null and p.id > $2
                   and p.author_id <> $1 and {author_ok} and {words_ok} and {discoverable}
                 limit {NEW_COUNT_CAP}
             ) n"
        ),
    };
    Ok(sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(since)
        .fetch_one(db)
        .await?)
}

/// One page of `viewer`'s bookmarked posts, newest bookmark first.
///
/// # Errors
///
/// Database failures.
pub async fn bookmarks(
    db: &PgPool,
    viewer: Uuid,
    before: Option<EdgeCursor>,
    limit: Limit,
) -> Result<Page<PostDto>, AppError> {
    let rows: Vec<(Uuid, OffsetDateTime)> = sqlx::query_as(
        "select b.post_id, b.created_at
           from bookmarks b join posts p on p.id = b.post_id
          where b.user_id = $1 and p.deleted_at is null
            and ($2::timestamptz is null or (b.created_at, b.post_id) < ($2, $3))
          order by b.created_at desc, b.post_id desc limit $4",
    )
    .bind(viewer)
    .bind(before.map(|cursor| cursor.at))
    .bind(before.map(|cursor| cursor.id))
    .bind(limit.fetch())
    .fetch_all(db)
    .await?;
    let rows = page(rows, limit, |(id, at)| EdgeCursor::new(*at, *id).encode());
    let ids: Vec<Uuid> = rows.items.iter().map(|(id, _)| *id).collect();
    Ok(Page {
        items: posts::load(db, viewer, &ids, None).await?,
        next: rows.next,
    })
}
