//! The SQL behind the inbox: replay rows, and what remote accounts do to local ones (follows,
//! likes, reposts, poll votes, blocks, reports).
//!
//! The functions here take values the inbox already verified (the signature, the actor binding,
//! the target being a local object). Every state change runs in one transaction, counters
//! change with the rows they count, and notifications go through [`notify`] so preferences,
//! mutes and blocks apply as they do for local actions. Callers pass the returned [`Pending`]
//! values to [`super::notifications::deliver`] after the call. No function reaches the network.

use serde_json::Value;
use sqlx::{PgConnection, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::deliveries::enqueue;
use super::graph::bump_follow_counts;
use super::notifications::{Pending, notify};
use super::post_actions::bump;
use crate::dto::{NotificationKindDto, ReportReason};
use crate::error::AppError;

// ------------------------------------------------------------------ replays

/// Records `activity_id` as handled. `true` when it was new, `false` for a replay.
///
/// # Errors
///
/// Database failures.
pub async fn mark_seen(db: &PgPool, activity_id: &str) -> Result<bool, AppError> {
    let added = sqlx::query(
        "insert into ap_seen (activity_id) values ($1) on conflict (activity_id) do nothing",
    )
    .bind(activity_id)
    .execute(db)
    .await?
    .rows_affected();
    Ok(added > 0)
}

/// Forgets `activity_id`, so a retry of an activity that failed on our side is processed.
///
/// # Errors
///
/// Database failures.
pub async fn unmark_seen(db: &PgPool, activity_id: &str) -> Result<(), AppError> {
    sqlx::query("delete from ap_seen where activity_id = $1")
        .bind(activity_id)
        .execute(db)
        .await?;
    Ok(())
}

// ------------------------------------------------------------------ follows

/// What [`receive_follow`] answered.
#[derive(Debug)]
pub struct Followed {
    /// The remote account follows now (an `Accept` was queued); `false` when it was refused
    /// because the local account blocked it (a `Reject` was queued).
    pub accepted: bool,
    /// The notification to deliver to live streams.
    pub pending: Option<Pending>,
}

/// The replies [`receive_follow`] queues to the follower's inbox.
#[derive(Debug)]
pub struct FollowReplies<'a> {
    /// The follower's inbox.
    pub inbox: &'a str,
    /// The `Accept` of the follow.
    pub accept: &'a Value,
    /// The `Reject` of the follow.
    pub reject: &'a Value,
}

/// Handles a `Follow` of local account `target` by remote account `follower`.
///
/// A follower that `target` blocked is refused: any follow row between them goes, and the
/// `Reject` is queued. Otherwise the follow becomes `accepted` and the `Accept` is queued, signed
/// by `target`. A new follow (or a pending one that is accepted now) bumps both counters and
/// notifies `target`; a follow that already stands only gets the `Accept` again. `follow_id` is
/// the id of the `Follow` activity, stored in `follows.ap_id`.
///
/// # Errors
///
/// Database failures.
pub async fn receive_follow(
    db: &PgPool,
    target: Uuid,
    follower: Uuid,
    follow_id: Option<&str>,
    replies: &FollowReplies<'_>,
) -> Result<Followed, AppError> {
    let mut tx = db.begin().await?;
    let blocked: bool = sqlx::query_scalar(
        "select exists (select 1 from blocks where user_id = $1 and target_id = $2)",
    )
    .bind(target)
    .bind(follower)
    .fetch_one(&mut *tx)
    .await?;
    if blocked {
        remove_follow(&mut tx, follower, target).await?;
        enqueue(&mut *tx, Some(target), replies.inbox, replies.reject).await?;
        tx.commit().await?;
        return Ok(Followed {
            accepted: false,
            pending: None,
        });
    }
    let existing: Option<String> = sqlx::query_scalar(
        "select state from follows where follower_id = $1 and followee_id = $2 for update",
    )
    .bind(follower)
    .bind(target)
    .fetch_optional(&mut *tx)
    .await?;
    let counted = match existing.as_deref() {
        Some("accepted") => {
            sqlx::query(
                "update follows set ap_id = coalesce($3, ap_id)
                  where follower_id = $1 and followee_id = $2",
            )
            .bind(follower)
            .bind(target)
            .bind(follow_id)
            .execute(&mut *tx)
            .await?;
            false
        }
        Some(_) => {
            sqlx::query(
                "update follows set state = 'accepted', ap_id = coalesce($3, ap_id)
                  where follower_id = $1 and followee_id = $2",
            )
            .bind(follower)
            .bind(target)
            .bind(follow_id)
            .execute(&mut *tx)
            .await?;
            true
        }
        None => {
            sqlx::query(
                "insert into follows (follower_id, followee_id, state, ap_id)
                 values ($1, $2, 'accepted', $3) on conflict do nothing",
            )
            .bind(follower)
            .bind(target)
            .bind(follow_id)
            .execute(&mut *tx)
            .await?
            .rows_affected()
                > 0
        }
    };
    let mut pending = None;
    if counted {
        bump_follow_counts(&mut tx, follower, target, 1).await?;
        pending = notify(&mut tx, target, follower, NotificationKindDto::Follow, None).await?;
    }
    enqueue(&mut *tx, Some(target), replies.inbox, replies.accept).await?;
    tx.commit().await?;
    Ok(Followed {
        accepted: true,
        pending,
    })
}

/// Deletes the follow of `followee` by `follower`, whatever its state, and takes it out of the
/// counters when it was accepted. `true` when a row went.
async fn remove_follow(
    conn: &mut PgConnection,
    follower: Uuid,
    followee: Uuid,
) -> Result<bool, AppError> {
    let removed: Option<String> = sqlx::query_scalar(
        "delete from follows where follower_id = $1 and followee_id = $2 returning state",
    )
    .bind(follower)
    .bind(followee)
    .fetch_optional(&mut *conn)
    .await?;
    match removed.as_deref() {
        Some("accepted") => {
            bump_follow_counts(conn, follower, followee, -1).await?;
            Ok(true)
        }
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

/// A remote `Accept` of a follow a local account sent: the pending follow of `followee` becomes
/// accepted and both counters go up. The follow is the one whose `ap_id` is `follow_id`, or the
/// one by the local account `follower_handle` (the actor of the embedded `Follow`). Returns how
/// many follows it accepted; zero when no pending follow matches, which also covers replays.
///
/// # Errors
///
/// Database failures.
pub async fn accept_pending(
    db: &PgPool,
    followee: Uuid,
    follow_id: Option<&str>,
    follower_handle: Option<&str>,
) -> Result<usize, AppError> {
    let mut tx = db.begin().await?;
    let followers: Vec<Uuid> = sqlx::query_scalar(
        "update follows f set state = 'accepted'
           from users l
          where l.id = f.follower_id and l.domain is null and l.deleted_at is null
            and f.followee_id = $1 and f.state = 'pending'
            and (f.ap_id = $2 or lower(l.handle) = lower($3))
          returning f.follower_id",
    )
    .bind(followee)
    .bind(follow_id)
    .bind(follower_handle)
    .fetch_all(&mut *tx)
    .await?;
    for follower in &followers {
        bump_follow_counts(&mut tx, *follower, followee, 1).await?;
    }
    tx.commit().await?;
    Ok(followers.len())
}

/// A remote `Reject` of a follow a local account sent, or the remote account dropping a
/// follower: the follow of `followee` goes, with the counters when it was accepted. Matched as
/// [`accept_pending`] does. Returns how many follows it removed.
///
/// # Errors
///
/// Database failures.
pub async fn reject_follow(
    db: &PgPool,
    followee: Uuid,
    follow_id: Option<&str>,
    follower_handle: Option<&str>,
) -> Result<usize, AppError> {
    let mut tx = db.begin().await?;
    let removed: Vec<(Uuid, String)> = sqlx::query_as(
        "delete from follows f
          using users l
          where l.id = f.follower_id and l.domain is null
            and f.followee_id = $1 and (f.ap_id = $2 or lower(l.handle) = lower($3))
          returning f.follower_id, f.state",
    )
    .bind(followee)
    .bind(follow_id)
    .bind(follower_handle)
    .fetch_all(&mut *tx)
    .await?;
    for (follower, state) in &removed {
        if state == "accepted" {
            bump_follow_counts(&mut tx, *follower, followee, -1).await?;
        }
    }
    tx.commit().await?;
    Ok(removed.len())
}

/// `Undo` of a `Follow` by remote account `follower`: removes its follow of a local account,
/// found by the `Follow`'s id or by the local account it named (`target`). Returns whether a
/// follow was removed.
///
/// # Errors
///
/// Database failures.
pub async fn undo_follow(
    db: &PgPool,
    follower: Uuid,
    follow_id: Option<&str>,
    target: Option<Uuid>,
) -> Result<bool, AppError> {
    let mut tx = db.begin().await?;
    let removed: Vec<(Uuid, String)> = sqlx::query_as(
        "delete from follows f
          using users t
          where t.id = f.followee_id and t.domain is null and f.follower_id = $1
            and (f.ap_id = $2 or f.followee_id = $3)
          returning f.followee_id, f.state",
    )
    .bind(follower)
    .bind(follow_id)
    .bind(target)
    .fetch_all(&mut *tx)
    .await?;
    for (followee, state) in &removed {
        if state == "accepted" {
            bump_follow_counts(&mut tx, follower, *followee, -1).await?;
        }
    }
    tx.commit().await?;
    Ok(!removed.is_empty())
}

/// A remote account blocks a local one: the follows between them go, both ways, with their
/// counters. No block row is stored: it is the remote's own decision, and nothing here would
/// consult it.
///
/// # Errors
///
/// Database failures.
pub async fn sever_follows(db: &PgPool, remote: Uuid, local: Uuid) -> Result<(), AppError> {
    let mut tx = db.begin().await?;
    remove_follow(&mut tx, remote, local).await?;
    remove_follow(&mut tx, local, remote).await?;
    tx.commit().await?;
    Ok(())
}

// ------------------------------------------------------------------ likes and reposts

/// The live post `id` of local account `handle`, locked for the transaction, and its author.
async fn lock_local_post(
    conn: &mut PgConnection,
    handle: &str,
    id: Uuid,
) -> Result<Option<Uuid>, AppError> {
    Ok(sqlx::query_scalar(
        "select p.author_id from posts p join users u on u.id = p.author_id
          where p.id = $1 and lower(u.handle) = lower($2) and u.domain is null
            and p.deleted_at is null for no key update of p",
    )
    .bind(id)
    .bind(handle)
    .fetch_optional(conn)
    .await?)
}

/// Whether local `author` blocked remote `actor`.
async fn author_blocks(
    conn: &mut PgConnection,
    author: Uuid,
    actor: Uuid,
) -> Result<bool, AppError> {
    Ok(sqlx::query_scalar(
        "select exists (select 1 from blocks where user_id = $1 and target_id = $2)",
    )
    .bind(author)
    .bind(actor)
    .fetch_one(conn)
    .await?)
}

/// A `Like` by remote `actor` of the local post `id` by `handle`: the like row, the counter and
/// the notification. Nothing happens for a missing or deleted post, an author that blocked the
/// actor, or a like that stands already.
///
/// # Errors
///
/// Database failures.
pub async fn like(
    db: &PgPool,
    actor: Uuid,
    handle: &str,
    id: Uuid,
) -> Result<Option<Pending>, AppError> {
    let mut tx = db.begin().await?;
    let Some(author) = lock_local_post(&mut tx, handle, id).await? else {
        return Ok(None);
    };
    if author_blocks(&mut tx, author, actor).await? {
        return Ok(None);
    }
    let added =
        sqlx::query("insert into likes (user_id, post_id) values ($1, $2) on conflict do nothing")
            .bind(actor)
            .bind(id)
            .execute(&mut *tx)
            .await?
            .rows_affected();
    let mut pending = None;
    if added > 0 {
        bump(&mut tx, id, "like_count", 1).await?;
        pending = notify(&mut tx, author, actor, NotificationKindDto::Like, Some(id)).await?;
    }
    tx.commit().await?;
    Ok(pending)
}

/// `Undo` of a `Like`: removes the like of `actor` on the local post `id` by `handle`.
///
/// # Errors
///
/// Database failures.
pub async fn unlike(db: &PgPool, actor: Uuid, handle: &str, id: Uuid) -> Result<(), AppError> {
    let mut tx = db.begin().await?;
    if lock_local_post(&mut tx, handle, id).await?.is_none() {
        return Ok(());
    }
    let removed = sqlx::query("delete from likes where user_id = $1 and post_id = $2")
        .bind(actor)
        .bind(id)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if removed > 0 {
        bump(&mut tx, id, "like_count", -1).await?;
    }
    tx.commit().await?;
    Ok(())
}

/// An `Announce` by remote `actor` of the local post `id` by `handle`: the repost row (with the
/// activity id `ap_id`), the counter and the notification. Idempotent like [`like`].
///
/// # Errors
///
/// Database failures.
pub async fn announce(
    db: &PgPool,
    actor: Uuid,
    handle: &str,
    id: Uuid,
    ap_id: Option<&str>,
) -> Result<Option<Pending>, AppError> {
    let mut tx = db.begin().await?;
    let Some(author) = lock_local_post(&mut tx, handle, id).await? else {
        return Ok(None);
    };
    if author_blocks(&mut tx, author, actor).await? {
        return Ok(None);
    }
    let added = sqlx::query(
        "insert into reposts (id, user_id, post_id, ap_id) values ($1, $2, $3, $4)
         on conflict do nothing",
    )
    .bind(Uuid::now_v7())
    .bind(actor)
    .bind(id)
    .bind(ap_id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    let mut pending = None;
    if added > 0 {
        bump(&mut tx, id, "repost_count", 1).await?;
        pending = notify(
            &mut tx,
            author,
            actor,
            NotificationKindDto::Repost,
            Some(id),
        )
        .await?;
    }
    tx.commit().await?;
    Ok(pending)
}

/// `Undo` of an `Announce` by `actor`: removes the repost with activity id `ap_id`, or the one of
/// the local post `post` (`handle`, id).
///
/// # Errors
///
/// Database failures.
pub async fn unannounce(
    db: &PgPool,
    actor: Uuid,
    ap_id: Option<&str>,
    post: Option<(&str, Uuid)>,
) -> Result<(), AppError> {
    let mut tx = db.begin().await?;
    let (handle, id) = match post {
        Some((handle, id)) => (Some(handle), Some(id)),
        None => (None, None),
    };
    let removed: Vec<Uuid> = sqlx::query_scalar(
        "delete from reposts r
          using posts p join users u on u.id = p.author_id
          where p.id = r.post_id and u.domain is null and r.user_id = $1
            and (r.ap_id = $2 or (r.post_id = $3 and lower(u.handle) = lower($4)))
          returning r.post_id",
    )
    .bind(actor)
    .bind(ap_id)
    .bind(id)
    .bind(handle)
    .fetch_all(&mut *tx)
    .await?;
    for post in removed {
        bump(&mut tx, post, "repost_count", -1).await?;
    }
    tx.commit().await?;
    Ok(())
}

// ------------------------------------------------------------------ poll votes

/// A vote by remote `voter` for the option titled `option` on the poll of the local post `id` by
/// `handle`. One call records one choice, as Mastodon sends one `Create` per choice of a
/// multiple-choice poll. `true` when the vote counted; `false` when there is no such poll or
/// option, the poll ended, the author blocked the voter, the voter already voted (a single-choice
/// poll) or already picked this option.
///
/// # Errors
///
/// Database failures.
pub async fn vote(
    db: &PgPool,
    voter: Uuid,
    handle: &str,
    id: Uuid,
    option: &str,
) -> Result<bool, AppError> {
    let mut tx = db.begin().await?;
    let poll: Option<(Vec<String>, bool, OffsetDateTime, Uuid)> = sqlx::query_as(
        "select pl.options, pl.multiple, pl.ends_at, p.author_id
           from polls pl join posts p on p.id = pl.post_id join users u on u.id = p.author_id
          where pl.post_id = $1 and lower(u.handle) = lower($2) and u.domain is null
            and p.deleted_at is null for update of pl",
    )
    .bind(id)
    .bind(handle)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((options, multiple, ends_at, author)) = poll else {
        return Ok(false);
    };
    let Some(choice) = options
        .iter()
        .position(|title| title == option)
        .and_then(|index| i16::try_from(index).ok())
    else {
        return Ok(false);
    };
    if OffsetDateTime::now_utc() >= ends_at || author_blocks(&mut tx, author, voter).await? {
        return Ok(false);
    }
    let earlier: Vec<i16> =
        sqlx::query_scalar("select choice from poll_votes where post_id = $1 and user_id = $2")
            .bind(id)
            .bind(voter)
            .fetch_all(&mut *tx)
            .await?;
    if (!multiple && !earlier.is_empty()) || earlier.contains(&choice) {
        return Ok(false);
    }
    sqlx::query("insert into poll_votes (post_id, user_id, choice) values ($1, $2, $3)")
        .bind(id)
        .bind(voter)
        .bind(choice)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "update polls set voters = voters + $3,
            counts = (select array_agg(case when i = $2::int4 then c + 1 else c end order by i)
                        from unnest(counts) with ordinality as t(c, i))
          where post_id = $1",
    )
    .bind(id)
    .bind(i32::from(choice) + 1)
    .bind(i32::from(earlier.is_empty()))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(true)
}

// ------------------------------------------------------------------ reports

/// The local account `handle` as a report target: its id when it can receive reports.
///
/// # Errors
///
/// Database failures.
pub async fn local_account(db: &PgPool, handle: &str) -> Result<Option<Uuid>, AppError> {
    Ok(sqlx::query_scalar(
        "select id from users where lower(handle) = lower($1) and domain is null
            and lower(handle) <> 'tuita' and deleted_at is null and suspended_at is null",
    )
    .bind(handle)
    .fetch_optional(db)
    .await?)
}

/// The author of the live post `id` of local account `handle`.
///
/// # Errors
///
/// Database failures.
pub async fn local_post_author(
    db: &PgPool,
    handle: &str,
    id: Uuid,
) -> Result<Option<Uuid>, AppError> {
    Ok(sqlx::query_scalar(
        "select p.author_id from posts p join users u on u.id = p.author_id
          where p.id = $1 and lower(u.handle) = lower($2) and u.domain is null
            and p.deleted_at is null",
    )
    .bind(id)
    .bind(handle)
    .fetch_optional(db)
    .await?)
}

/// Files a report received from another instance, with the system account `@tuita` as the
/// reporter. `note` says who reported and why. Returns `false` when an open report on the same
/// target and post exists already.
///
/// # Errors
///
/// Database failures.
pub async fn file_report(
    db: &PgPool,
    target: Uuid,
    post: Option<Uuid>,
    note: &str,
) -> Result<bool, AppError> {
    let reporter: Option<Uuid> =
        sqlx::query_scalar("select id from users where handle = 'tuita' and domain is null")
            .fetch_optional(db)
            .await?;
    let Some(reporter) = reporter else {
        return Ok(false);
    };
    let duplicate =
        super::reports::create(db, reporter, target, post, ReportReason::Other, note).await?;
    Ok(!duplicate)
}
