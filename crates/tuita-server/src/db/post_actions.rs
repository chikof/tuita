//! Writing posts: creating, deleting, reactions (like, repost, bookmark), pins, and the lists
//! of who reacted or quoted.
//!
//! Every counter changes in the same transaction as the row it counts. Notifications are stored
//! inside the transaction and delivered to live streams after the commit.

use sqlx::{FromRow, PgConnection, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::notifications::{self, notify};
use super::users::{COLS, UserRow, dtos};
use super::{audit, blocked_author, live, posts, visible_author, visible_post, word_visible};
use crate::config::Config;
use crate::cursor::{EdgeCursor, Limit, page};
use crate::dto::{NotificationKindDto, Page, PostDto, ReplyPolicy, ThreadDto, UserDto};
use crate::error::AppError;
use crate::federation::outbound;
use crate::state::AppState;
use crate::workers::cleanup;

const NOT_FOUND: &str = "Post not found.";
const BLOCKED: &str = "You can’t interact with this post.";
/// Direct replies a thread shows, hidden ones included.
const REPLY_LIMIT: i64 = 200;

/// What a viewer can do with a post and have it undone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reaction {
    /// Like.
    Like,
    /// Repost.
    Repost,
    /// Bookmark.
    Bookmark,
}

impl Reaction {
    const fn table(self) -> &'static str {
        match self {
            Self::Like => "likes",
            Self::Repost => "reposts",
            Self::Bookmark => "bookmarks",
        }
    }

    const fn counter(self) -> Option<&'static str> {
        match self {
            Self::Like => Some("like_count"),
            Self::Repost => Some("repost_count"),
            Self::Bookmark => None,
        }
    }

    const fn kind(self) -> Option<NotificationKindDto> {
        match self {
            Self::Like => Some(NotificationKindDto::Like),
            Self::Repost => Some(NotificationKindDto::Repost),
            Self::Bookmark => None,
        }
    }
}

/// Who may see the list of one kind of reaction: likes or reposts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReactorList {
    /// People who liked.
    Likes,
    /// People who reposted.
    Reposts,
}

/// A live post a new post replies to or quotes.
#[derive(FromRow)]
pub(super) struct Target {
    /// The post.
    pub(super) id: Uuid,
    /// Its author.
    pub(super) author_id: Uuid,
    /// The root of its thread; `None` when it is a root itself.
    pub(super) root_id: Option<Uuid>,
    reply_policy: String,
}
pub(super) async fn live_target<'e>(
    db: impl sqlx::PgExecutor<'e>,
    id: Uuid,
) -> Result<Target, AppError> {
    sqlx::query_as(
        "select id, author_id, root_id, reply_policy from posts
         where id = $1 and deleted_at is null",
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::Missing(NOT_FOUND))
}

/// [`live_target`] for a post `viewer` may open per [`visible_post`]: a followers-only or direct
/// post of someone else answers like a missing one.
pub(super) async fn visible_target<'e>(
    db: impl sqlx::PgExecutor<'e>,
    viewer: Uuid,
    id: Uuid,
) -> Result<Target, AppError> {
    let sql = format!(
        "select p.id, p.author_id, p.root_id, p.reply_policy from posts p
         where p.id = $1 and p.deleted_at is null and {}",
        visible_post("p", "$2")
    );
    sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .bind(viewer)
        .fetch_optional(db)
        .await?
        .ok_or(AppError::Missing(NOT_FOUND))
}

/// Re-reads a post inside a transaction and locks it (`for no key update`: a share lock would
/// deadlock two transactions that both go on to bump its counters), so a concurrent delete waits
/// for this transaction (and then removes what it added) or has already won (404 here).
pub(super) async fn lock_live(conn: &mut PgConnection, id: Uuid) -> Result<(), AppError> {
    sqlx::query_scalar::<_, Uuid>(
        "select id from posts where id = $1 and deleted_at is null for no key update",
    )
    .bind(id)
    .fetch_optional(conn)
    .await?
    .map(|_| ())
    .ok_or(AppError::Missing(NOT_FOUND))
}

pub(super) async fn blocked_between(
    db: &mut PgConnection,
    a: Uuid,
    b: Uuid,
) -> Result<bool, AppError> {
    Ok(sqlx::query_scalar(
        "select exists (select 1 from blocks
            where (user_id = $1 and target_id = $2) or (user_id = $2 and target_id = $1))",
    )
    .bind(a)
    .bind(b)
    .fetch_one(db)
    .await?)
}

pub(super) async fn may_reply(
    db: &mut PgConnection,
    me: Uuid,
    target: &Target,
) -> Result<bool, AppError> {
    if target.author_id == me {
        return Ok(true);
    }
    let allowed = match ReplyPolicy::parse(&target.reply_policy).unwrap_or(ReplyPolicy::Everyone) {
        ReplyPolicy::Everyone => true,
        ReplyPolicy::Following => {
            sqlx::query_scalar(
                "select exists (select 1 from follows where follower_id = $1
                    and followee_id = $2 and state = 'accepted')",
            )
            .bind(target.author_id)
            .bind(me)
            .fetch_one(db)
            .await?
        }
        ReplyPolicy::Mentioned => {
            sqlx::query_scalar(
                "select exists (select 1 from mentions where post_id = $1 and user_id = $2)",
            )
            .bind(target.id)
            .bind(me)
            .fetch_one(db)
            .await?
        }
    };
    Ok(allowed)
}

/// A reply or quote target the author may interact with.
pub(super) async fn interaction_target(
    db: &mut PgConnection,
    me: Uuid,
    id: Uuid,
) -> Result<Target, AppError> {
    let target = visible_target(&mut *db, me, id).await?;
    if blocked_between(db, me, target.author_id).await? {
        return Err(AppError::forbidden(BLOCKED));
    }
    Ok(target)
}

/// Adds `by` to a counter column of one post. `column` is always one of our own constants.
pub(super) async fn bump(
    conn: &mut PgConnection,
    post: Uuid,
    column: &'static str,
    by: i32,
) -> Result<(), AppError> {
    let sql = format!("update posts set {column} = greatest({column} + $2, 0) where id = $1");
    sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(post)
        .bind(by)
        .execute(conn)
        .await?;
    Ok(())
}

/// Soft-deletes a post and everything attached to it (edits, polls, votes, views included), then
/// tells the live timelines that held it. Once the delete is committed its uploads leave the
/// media store; one that cannot be removed stays for the hourly cleanup.
///
/// A staff member deleting someone else's post leaves a `post.delete` audit row. A post the viewer
/// cannot open answers like a missing one, except for staff.
///
/// # Errors
///
/// `404 Post not found.`; `403 You can only delete your own posts.` unless the viewer is the
/// author or staff; database failures.
pub async fn delete(state: &AppState, viewer: &UserRow, id: Uuid) -> Result<(), AppError> {
    let target = if viewer.is_staff() {
        live_target(state.db(), id).await?
    } else {
        visible_target(state.db(), viewer.id, id).await?
    };
    if target.author_id != viewer.id && !viewer.is_staff() {
        return Err(AppError::forbidden("You can only delete your own posts."));
    }
    let mut tx = state.db().begin().await?;
    let removal = live::removal(&mut tx, state.hub(), id).await?;
    if !soft_delete_federated(&mut tx, state.config(), id).await? {
        return Err(AppError::Missing(NOT_FOUND));
    }
    if target.author_id != viewer.id {
        let handle: String = sqlx::query_scalar("select handle from users where id = $1")
            .bind(target.author_id)
            .fetch_one(&mut *tx)
            .await?;
        audit::record(
            &mut tx,
            audit::Entry::new(Some(viewer.id), "post.delete")
                .target(format!("@{handle}"))
                .text(format!("post {id}"))
                .about(target.author_id),
        )
        .await?;
    }
    tx.commit().await?;
    live::publish_removal(state.hub(), removal);
    if let Err(err) = cleanup::remove_orphaned_media(state, Some(id)).await {
        tracing::warn!(error = %err, post = %id, "cannot remove the uploads of a deleted post; cleanup retries");
    }
    Ok(())
}

/// [`soft_delete`] for a post of a local account: the `Delete` is queued for its audience in the
/// same transaction, before the text is wiped so the accounts the post mentions hear of it too.
/// A post of a remote account is deleted without sending anything.
///
/// # Errors
///
/// Database failures.
pub(super) async fn soft_delete_federated(
    conn: &mut PgConnection,
    config: &Config,
    id: Uuid,
) -> Result<bool, AppError> {
    let live: Option<Uuid> = sqlx::query_scalar(
        "select id from posts where id = $1 and deleted_at is null for no key update",
    )
    .bind(id)
    .fetch_optional(&mut *conn)
    .await?;
    if live.is_none() {
        return Ok(false);
    }
    outbound::post_deleted(conn, config, id).await?;
    soft_delete(conn, id).await
}

/// Soft-deletes the live post `id` and removes what hangs on it, inside the caller's
/// transaction. `false` when the post is already gone. Sends nothing to other servers; see
/// [`soft_delete_federated`].
///
/// # Errors
///
/// Database failures.
pub(super) async fn soft_delete(conn: &mut PgConnection, id: Uuid) -> Result<bool, AppError> {
    let gone: Option<(Uuid, Option<Uuid>, Option<Uuid>)> = sqlx::query_as(
        "update posts set deleted_at = now(), text = '', cw = ''
         where id = $1 and deleted_at is null returning author_id, reply_to_id, quote_of_id",
    )
    .bind(id)
    .fetch_optional(&mut *conn)
    .await?;
    let Some((author, reply_to, quote_of)) = gone else {
        return Ok(false);
    };
    for table in [
        "likes",
        "reposts",
        "bookmarks",
        "notifications",
        "post_tags",
        "mentions",
        "post_media",
        "post_edits",
        "post_views",
        "polls",
    ] {
        let sql = format!("delete from {table} where post_id = $1");
        sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(id)
            .execute(&mut *conn)
            .await?;
    }
    // The uploads keep pointing at the deleted post: cleanup removes them at once, not after the
    // 24 hours an unattached upload waits.
    sqlx::query(
        "update scheduled_posts set payload = (payload - 'poll') || jsonb_build_object('text', ''::text, 'cw', ''::text)
          where post_id = $1",
    )
    .bind(id)
    .execute(&mut *conn)
    .await?;
    sqlx::query("update users set pinned_post_id = null where pinned_post_id = $1")
        .bind(id)
        .execute(&mut *conn)
        .await?;
    sqlx::query("update users set posts_count = greatest(posts_count - 1, 0) where id = $1")
        .bind(author)
        .execute(&mut *conn)
        .await?;
    if let Some(parent) = reply_to {
        bump(&mut *conn, parent, "reply_count", -1).await?;
    }
    if let Some(quoted) = quote_of {
        bump(&mut *conn, quoted, "quote_count", -1).await?;
    }
    Ok(true)
}

/// Adds or removes a reaction and returns the fresh post. Repeating either is a no-op.
///
/// # Errors
///
/// `404 Post not found.`; `403` when blocked either way (like and repost, adding only);
/// database failures.
pub async fn react(
    state: &AppState,
    viewer: Uuid,
    id: Uuid,
    reaction: Reaction,
    add: bool,
) -> Result<PostDto, AppError> {
    let db = state.db();
    let target = if add {
        visible_target(db, viewer, id).await?
    } else {
        live_target(db, id).await?
    };
    let mut pending = None;
    let mut boosted = None;
    let mut unboosted = live::Removal::default();
    let mut tx = db.begin().await?;
    lock_live(&mut tx, id).await?;
    if add {
        if reaction != Reaction::Bookmark
            && blocked_between(&mut tx, viewer, target.author_id).await?
        {
            return Err(AppError::forbidden(BLOCKED));
        }
        let repost_id = Uuid::now_v7();
        let inserted = if reaction == Reaction::Repost {
            sqlx::query(
                "insert into reposts (id, user_id, post_id) values ($1, $2, $3)
                 on conflict do nothing",
            )
            .bind(repost_id)
            .bind(viewer)
            .bind(id)
            .execute(&mut *tx)
            .await?
            .rows_affected()
        } else {
            let sql = format!(
                "insert into {} (user_id, post_id) values ($1, $2) on conflict do nothing",
                reaction.table()
            );
            sqlx::query(sqlx::AssertSqlSafe(sql))
                .bind(viewer)
                .bind(id)
                .execute(&mut *tx)
                .await?
                .rows_affected()
        };
        if inserted > 0 {
            if reaction == Reaction::Repost {
                boosted = Some(repost_id);
            }
            match reaction {
                Reaction::Like => {
                    outbound::like(&mut tx, state.config(), viewer, id).await?;
                }
                Reaction::Repost => {
                    outbound::announce(&mut tx, state.config(), viewer, id, repost_id).await?;
                }
                Reaction::Bookmark => {}
            }
            if let Some(counter) = reaction.counter() {
                bump(&mut tx, id, counter, 1).await?;
            }
            if let Some(kind) = reaction.kind() {
                pending = notify(&mut tx, target.author_id, viewer, kind, Some(id)).await?;
            }
        }
    } else {
        if reaction == Reaction::Repost {
            unboosted = live::unboosted(&mut tx, state.hub(), id, viewer).await?;
        }
        match reaction {
            Reaction::Like => {
                outbound::undo_like(&mut tx, state.config(), viewer, id).await?;
            }
            Reaction::Repost => {
                outbound::undo_announce(&mut tx, state.config(), viewer, id).await?;
            }
            Reaction::Bookmark => {}
        }
        let sql = format!(
            "delete from {} where user_id = $1 and post_id = $2",
            reaction.table()
        );
        let removed = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(viewer)
            .bind(id)
            .execute(&mut *tx)
            .await?
            .rows_affected();
        if let Some(counter) = reaction.counter().filter(|_| removed > 0) {
            bump(&mut tx, id, counter, -1).await?;
        }
    }
    tx.commit().await?;
    notifications::deliver(state, pending).await;
    if let Some(repost) = boosted {
        live::reposted(state, repost, id, viewer).await;
    }
    live::publish_removal(state.hub(), unboosted);
    // The author is allowed: a muted author's post the viewer opened (profile, thread) must
    // come back, and `allow` also lifts the viewer's muted words.
    posts::load_one(db, viewer, id, Some(target.author_id))
        .await?
        .ok_or(AppError::Missing(NOT_FOUND))
}

/// Pins one of the viewer's own posts to their profile.
///
/// # Errors
///
/// `404 Post not found.`; `403 You can only pin your own posts.`; database failures.
pub async fn pin(db: &PgPool, viewer: Uuid, id: Uuid) -> Result<(), AppError> {
    let target = visible_target(db, viewer, id).await?;
    if target.author_id != viewer {
        return Err(AppError::forbidden("You can only pin your own posts."));
    }
    sqlx::query("update users set pinned_post_id = $1 where id = $2")
        .bind(id)
        .bind(viewer)
        .execute(db)
        .await?;
    Ok(())
}

/// Clears the viewer's pinned post, whichever it is.
///
/// # Errors
///
/// Database failures.
pub async fn unpin(db: &PgPool, viewer: Uuid) -> Result<(), AppError> {
    sqlx::query("update users set pinned_post_id = null where id = $1")
        .bind(viewer)
        .execute(db)
        .await?;
    Ok(())
}

/// A post with its ancestors and replies, as `viewer` sees them.
///
/// # Errors
///
/// `404 Post not found.`; `404 This post is unavailable.` when its author is suspended,
/// deleted or blocked either way; database failures.
pub async fn thread(db: &PgPool, viewer: Uuid, id: Uuid) -> Result<ThreadDto, AppError> {
    let sql = format!(
        "select p.author_id, {} from posts p join users a on a.id = p.author_id
         where p.id = $2 and p.deleted_at is null and {}",
        blocked_author("a", "$1"),
        visible_post("p", "$1")
    );
    let (author, blocked): (Uuid, bool) = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(AppError::Missing(NOT_FOUND))?;
    if blocked {
        return Err(AppError::Missing("This post is unavailable."));
    }
    let post = posts::load_one(db, viewer, id, Some(author))
        .await?
        .ok_or(AppError::Missing(NOT_FOUND))?;

    let ancestor_ids: Vec<Uuid> = sqlx::query_scalar(
        "with recursive up(id, reply_to_id, depth) as (
            select id, reply_to_id, 0 from posts where id = $1
            union all
            select p.id, p.reply_to_id, up.depth + 1 from posts p
              join up on p.id = up.reply_to_id where up.depth < 25)
         select id from up where depth > 0 order by depth desc",
    )
    .bind(id)
    .fetch_all(db)
    .await?;
    let replies_visible = word_visible("p", "$4");
    let sql = format!(
        "select p.id from posts p where p.reply_to_id = $1 and p.deleted_at is null
           and {replies_visible}
         order by (p.author_id = $2) desc, p.like_count desc, p.id asc limit $3"
    );
    let reply_ids: Vec<Uuid> = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .bind(author)
        .bind(REPLY_LIMIT)
        .bind(viewer)
        .fetch_all(db)
        .await?;
    let ancestors = posts::load(db, viewer, &ancestor_ids, None).await?;
    let (hidden_replies, replies) = posts::load(db, viewer, &reply_ids, None)
        .await?
        .into_iter()
        .partition(|reply| reply.hidden_reply);
    Ok(ThreadDto {
        post,
        ancestors,
        replies,
        hidden_replies,
    })
}

#[derive(FromRow)]
struct Reactor {
    edge_at: OffsetDateTime,
    #[sqlx(flatten)]
    user: UserRow,
}

/// One page of the people who liked or reposted a post, newest first.
///
/// # Errors
///
/// `404 Post not found.`; database failures.
pub async fn reactors(
    db: &PgPool,
    viewer: Uuid,
    id: Uuid,
    list: ReactorList,
    before: Option<EdgeCursor>,
    limit: Limit,
) -> Result<Page<UserDto>, AppError> {
    visible_target(db, viewer, id).await?;
    let table = match list {
        ReactorList::Likes => "likes",
        ReactorList::Reposts => "reposts",
    };
    let sql = format!(
        "select {COLS}, e.created_at as edge_at
         from {table} e join users u on u.id = e.user_id
         where e.post_id = $2 and {}
           and ($3::timestamptz is null or (e.created_at, e.user_id) < ($3, $4))
         order by e.created_at desc, e.user_id desc limit $5",
        visible_author("u", "$1")
    );
    let rows: Vec<Reactor> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(id)
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

/// One page of the posts that quote a post, newest first.
///
/// # Errors
///
/// `404 Post not found.`; database failures.
pub async fn quotes(
    db: &PgPool,
    viewer: Uuid,
    id: Uuid,
    before: Option<Uuid>,
    limit: Limit,
) -> Result<Page<PostDto>, AppError> {
    visible_target(db, viewer, id).await?;
    let ids: Vec<Uuid> = sqlx::query_scalar(
        "select id from posts where quote_of_id = $1 and deleted_at is null
           and ($2::uuid is null or id < $2)
         order by id desc limit $3",
    )
    .bind(id)
    .bind(before)
    .bind(limit.fetch())
    .fetch_all(db)
    .await?;
    let Page { items, next } = page(ids, limit, Uuid::to_string);
    Ok(Page {
        items: posts::load(db, viewer, &items, None).await?,
        next,
    })
}
