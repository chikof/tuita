//! The social graph: follows, blocks, mutes, the lists built from them, suggestions and the
//! post tabs of a profile.

use std::collections::HashMap;

use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::emoji;
use super::notifications::{Pending, notify};
use super::posts;
use super::users::{COLS, UserRow, dtos, user_ref};
use super::visible_author;
use crate::config::Config;
use crate::cursor::{EdgeCursor, Limit, PageQuery, page};
use crate::dto::{
    KnownFollowersDto, MediaKind, NotificationKindDto, Page, PostDto, ProfileTab, UserDto,
};
use crate::error::AppError;
use crate::federation::outbound;

/// Whether either account blocks the other.
///
/// # Errors
///
/// Database failures.
pub async fn blocked_between(db: &PgPool, a: Uuid, b: Uuid) -> Result<bool, AppError> {
    Ok(sqlx::query_scalar(
        "select exists (select 1 from blocks
            where (user_id = $1 and target_id = $2) or (user_id = $2 and target_id = $1))",
    )
    .bind(a)
    .bind(b)
    .fetch_one(db)
    .await?)
}

/// Makes `me` follow `target`. Following a local account is `accepted` at once. Following a
/// remote account is `pending` until its `Accept` arrives; a pending follow is in neither
/// counter and does not notify. Counters and the notification change only when the follow is
/// new. The returned notification must be passed to [`super::notifications::deliver`] after
/// this returns.
///
/// # Errors
///
/// Database failures.
pub async fn follow(
    db: &PgPool,
    config: &Config,
    me: Uuid,
    target: Uuid,
) -> Result<Option<Pending>, AppError> {
    let mut tx = db.begin().await?;
    let inserted: Option<String> = sqlx::query_scalar(
        "insert into follows (follower_id, followee_id, state)
         select $1, u.id, case when u.domain is null then 'accepted' else 'pending' end
           from users u where u.id = $2
         on conflict do nothing returning state",
    )
    .bind(me)
    .bind(target)
    .fetch_optional(&mut *tx)
    .await?;
    let mut pending = None;
    match inserted.as_deref() {
        Some("accepted") => {
            bump_follow_counts(&mut tx, me, target, 1).await?;
            pending = notify(&mut tx, target, me, NotificationKindDto::Follow, None).await?;
        }
        Some(_) => {
            outbound::follow(&mut tx, config, me, target).await?;
        }
        None => {}
    }
    tx.commit().await?;
    Ok(pending)
}

/// Removes the follow of `target` by `me`, if any.
///
/// # Errors
///
/// Database failures.
pub async fn unfollow(
    db: &PgPool,
    config: &Config,
    me: Uuid,
    target: Uuid,
) -> Result<(), AppError> {
    let mut tx = db.begin().await?;
    drop_follow(&mut tx, config, me, target).await?;
    tx.commit().await?;
    Ok(())
}

/// Blocks `target` for `me` and removes the follows between them in both directions.
///
/// # Errors
///
/// Database failures.
pub async fn block(db: &PgPool, config: &Config, me: Uuid, target: Uuid) -> Result<(), AppError> {
    let mut tx = db.begin().await?;
    sqlx::query("insert into blocks (user_id, target_id) values ($1, $2) on conflict do nothing")
        .bind(me)
        .bind(target)
        .execute(&mut *tx)
        .await?;
    drop_follow(&mut tx, config, me, target).await?;
    drop_follow(&mut tx, config, target, me).await?;
    tx.commit().await?;
    Ok(())
}

/// Removes the block of `target` by `me`.
///
/// # Errors
///
/// Database failures.
pub async fn unblock(db: &PgPool, me: Uuid, target: Uuid) -> Result<(), AppError> {
    sqlx::query("delete from blocks where user_id = $1 and target_id = $2")
        .bind(me)
        .bind(target)
        .execute(db)
        .await?;
    Ok(())
}

/// Mutes `target` for `me`.
///
/// # Errors
///
/// Database failures.
pub async fn mute(db: &PgPool, me: Uuid, target: Uuid) -> Result<(), AppError> {
    sqlx::query("insert into mutes (user_id, target_id) values ($1, $2) on conflict do nothing")
        .bind(me)
        .bind(target)
        .execute(db)
        .await?;
    Ok(())
}

/// Removes the mute of `target` by `me`.
///
/// # Errors
///
/// Database failures.
pub async fn unmute(db: &PgPool, me: Uuid, target: Uuid) -> Result<(), AppError> {
    sqlx::query("delete from mutes where user_id = $1 and target_id = $2")
        .bind(me)
        .bind(target)
        .execute(db)
        .await?;
    Ok(())
}

async fn drop_follow(
    conn: &mut sqlx::PgConnection,
    config: &Config,
    follower: Uuid,
    followee: Uuid,
) -> Result<(), AppError> {
    // A pending follow (a request a remote account has not accepted) is in neither counter.
    let removed: Option<(String, Option<String>)> = sqlx::query_as(
        "delete from follows where follower_id = $1 and followee_id = $2 returning state, ap_id",
    )
    .bind(follower)
    .bind(followee)
    .fetch_optional(&mut *conn)
    .await?;
    let Some((state, ap_id)) = removed else {
        return Ok(());
    };
    if state == "accepted" {
        bump_follow_counts(conn, follower, followee, -1).await?;
    }
    // One of the two is local and the other remote at most; each call does nothing otherwise.
    outbound::undo_follow(conn, config, follower, followee, ap_id.as_deref()).await?;
    outbound::reject_follow(conn, config, followee, follower, ap_id.as_deref()).await?;
    Ok(())
}

pub(super) async fn bump_follow_counts(
    conn: &mut sqlx::PgConnection,
    follower: Uuid,
    followee: Uuid,
    delta: i32,
) -> Result<(), AppError> {
    sqlx::query("update users set followers_count = followers_count + $2 where id = $1")
        .bind(followee)
        .bind(delta)
        .execute(&mut *conn)
        .await?;
    sqlx::query("update users set following_count = following_count + $2 where id = $1")
        .bind(follower)
        .bind(delta)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// The lists that hang off a user and a join table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edges {
    /// Accounts following the subject.
    Followers,
    /// Accounts the subject follows.
    Following,
    /// Accounts the subject blocked.
    Blocks,
    /// Accounts the subject muted.
    Mutes,
}

impl Edges {
    /// Table, the column holding the subject, the column holding the listed account, and the
    /// extra row filter.
    const fn parts(self) -> (&'static str, &'static str, &'static str, &'static str) {
        match self {
            Self::Followers => (
                "follows",
                "followee_id",
                "follower_id",
                "e.state = 'accepted' and u.suspended_at is null",
            ),
            Self::Following => (
                "follows",
                "follower_id",
                "followee_id",
                "e.state = 'accepted' and u.suspended_at is null",
            ),
            Self::Blocks => ("blocks", "user_id", "target_id", "true"),
            Self::Mutes => ("mutes", "user_id", "target_id", "true"),
        }
    }
}

#[derive(FromRow)]
struct EdgeRow {
    #[sqlx(flatten)]
    user: UserRow,
    edge_at: OffsetDateTime,
}

/// One page of the accounts on the other side of `subject`'s `edges`, newest edge first, as
/// profile cards for `viewer`. Deleted accounts never show.
///
/// # Errors
///
/// Database failures.
pub async fn edge_users(
    db: &PgPool,
    viewer: Uuid,
    edges: Edges,
    subject: Uuid,
    before: Option<EdgeCursor>,
    limit: Limit,
) -> Result<Page<UserDto>, AppError> {
    let (table, subject_col, other_col, filter) = edges.parts();
    let sql = format!(
        "select {COLS}, e.created_at as edge_at
         from {table} e join users u on u.id = e.{other_col}
         where e.{subject_col} = $1 and u.deleted_at is null and {filter}
           and ($2::timestamptz is null or (e.created_at, e.{other_col}) < ($2, $3::uuid))
         order by e.created_at desc, e.{other_col} desc limit $4"
    );
    let rows: Vec<EdgeRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(subject)
        .bind(before.map(|cursor| cursor.at))
        .bind(before.map(|cursor| cursor.id))
        .bind(limit.fetch())
        .fetch_all(db)
        .await?;
    let Page { items, next } = page(rows, limit, |row| {
        EdgeCursor::new(row.edge_at, row.user.id).encode()
    });
    let users: Vec<UserRow> = items.into_iter().map(|row| row.user).collect();
    Ok(Page {
        items: dtos(db, viewer, &users).await?,
        next,
    })
}

/// Followers of `subject` that `viewer` follows: the newest `limit` and the total.
///
/// # Errors
///
/// Database failures.
pub async fn known_followers(
    db: &PgPool,
    viewer: Uuid,
    subject: Uuid,
    limit: i64,
) -> Result<KnownFollowersDto, AppError> {
    const FROM: &str = "from follows f join users u on u.id = f.follower_id
         where f.followee_id = $1 and f.state = 'accepted'
           and u.suspended_at is null and u.deleted_at is null
           and exists (select 1 from follows v where v.follower_id = $2
                       and v.followee_id = f.follower_id and v.state = 'accepted')";
    let sql =
        format!("select {COLS} {FROM} order by f.created_at desc, f.follower_id desc limit $3");
    let rows: Vec<UserRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(subject)
        .bind(viewer)
        .bind(limit)
        .fetch_all(db)
        .await?;
    let count: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!("select count(*) {FROM}")))
        .bind(subject)
        .bind(viewer)
        .fetch_one(db)
        .await?;
    let mut users: Vec<_> = rows.iter().map(user_ref).collect();
    emoji::hydrate(db, &mut users).await?;
    Ok(KnownFollowersDto { users, count })
}

/// Accounts worth following for `viewer`: local, discoverable, visible, not yet followed.
///
/// # Errors
///
/// Database failures.
pub async fn suggestions(db: &PgPool, viewer: Uuid, limit: i64) -> Result<Vec<UserDto>, AppError> {
    let visible = visible_author("u", "$1");
    let sql = format!(
        "select {COLS} from users u
         where u.id <> $1 and u.domain is null and coalesce(u.discoverable, true) and {visible}
           and not exists (select 1 from follows where follower_id = $1 and followee_id = u.id)
         order by u.verified desc, u.followers_count desc, u.id asc limit $2"
    );
    let rows: Vec<UserRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(limit)
        .fetch_all(db)
        .await?;
    dtos(db, viewer, &rows).await
}

#[derive(FromRow)]
struct Entry {
    key: Uuid,
    post_id: Uuid,
    reposted: bool,
    at: Option<OffsetDateTime>,
}

impl Entry {
    fn cursor(&self) -> String {
        match self.at {
            Some(at) => EdgeCursor::new(at, self.key).encode(),
            None => self.key.to_string(),
        }
    }
}

const POSTS_TAB: &str = "select key, post_id, reposted, null::timestamptz as at from (
        select id as key, id as post_id, false as reposted from posts
         where author_id = $1 and deleted_at is null and reply_to_id is null
           and visibility <> 'direct'
           and id is distinct from (select pinned_post_id from users where id = $1)
        union all
        select id, post_id, true from reposts where user_id = $1
    ) t where $2::uuid is null or key < $2 order by key desc limit $3";
const REPLIES_TAB: &str = "select id as key, id as post_id, false as reposted,
        null::timestamptz as at from posts
     where author_id = $1 and deleted_at is null and reply_to_id is not null
       and visibility <> 'direct'
       and ($2::uuid is null or id < $2) order by id desc limit $3";
const MEDIA_TAB: &str = "select id as key, id as post_id, false as reposted,
        null::timestamptz as at from posts p
     where author_id = $1 and deleted_at is null and visibility <> 'direct'
       and exists (select 1 from post_media pm join media m on m.id = pm.media_id
                    where pm.post_id = p.id and ($4::text is null or m.kind = $4))
       and ($2::uuid is null or id < $2) order by id desc limit $3";
const LIKES_TAB: &str = "select l.post_id as key, l.post_id, false as reposted,
        l.created_at as at
     from likes l join posts p on p.id = l.post_id
     where l.user_id = $1 and p.deleted_at is null and p.visibility <> 'direct'
       and ($4::timestamptz is null or (l.created_at, l.post_id) < ($4, $2::uuid))
     order by l.created_at desc, l.post_id desc limit $3";

/// One page of a profile tab as seen by `viewer`. A muted profile still shows its posts.
///
/// The posts tab lists the user's top-level posts and reposts newest first. The pinned post's
/// own entry is left out of the list and shown once, first, on the first page; a repost of it
/// keeps its own entry. A page can hold fewer than `limit` posts when some are hidden from the
/// viewer. `kind` limits the media tab to posts with an attachment of that kind and is ignored
/// by the other tabs.
///
/// # Errors
///
/// `400 Invalid cursor.` or a database failure.
pub async fn profile_posts(
    db: &PgPool,
    viewer: Uuid,
    profile: &UserRow,
    tab: ProfileTab,
    kind: Option<MediaKind>,
    query: &PageQuery,
    limit: Limit,
) -> Result<Page<PostDto>, AppError> {
    let sql = match tab {
        ProfileTab::Posts => POSTS_TAB,
        ProfileTab::Replies => REPLIES_TAB,
        ProfileTab::Media => MEDIA_TAB,
        ProfileTab::Likes => LIKES_TAB,
    };
    let (key, at) = if tab == ProfileTab::Likes {
        let cursor = query.edge_cursor()?;
        (cursor.map(|c| c.id), cursor.map(|c| c.at))
    } else {
        (query.id_cursor()?, None)
    };
    let mut rows = sqlx::query_as(sql)
        .bind(profile.id)
        .bind(key)
        .bind(limit.fetch());
    match tab {
        ProfileTab::Likes => rows = rows.bind(at),
        ProfileTab::Media => rows = rows.bind(kind.map(MediaKind::as_str)),
        ProfileTab::Posts | ProfileTab::Replies => {}
    }
    let rows: Vec<Entry> = rows.fetch_all(db).await?;
    let Page {
        items: entries,
        next,
    } = page(rows, limit, Entry::cursor);

    let ids: Vec<Uuid> = entries.iter().map(|entry| entry.post_id).collect();
    let loaded = posts::load(db, viewer, &ids, Some(profile.id)).await?;
    let by_id: HashMap<Uuid, PostDto> = loaded.into_iter().map(|post| (post.id, post)).collect();
    let mut reposter = user_ref(profile);
    emoji::hydrate(db, std::slice::from_mut(&mut reposter)).await?;
    let mut items: Vec<PostDto> = entries
        .iter()
        .filter_map(|entry| {
            let mut post = by_id.get(&entry.post_id)?.clone();
            if entry.reposted {
                post.reposted_by = Some(reposter.clone());
                post.entry_id = entry.key;
            }
            Some(post)
        })
        .collect();

    if tab == ProfileTab::Posts
        && query.before.is_none()
        && let Some(pinned) = profile.pinned_post_id
        && let Some(mut post) = posts::load_one(db, viewer, pinned, Some(profile.id)).await?
    {
        post.pinned = true;
        items.insert(0, post);
    }
    Ok(Page { items, next })
}
