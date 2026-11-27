//! The reads and writes behind Mastodon notifications, idempotency keys, pins and conversations.

use std::collections::{HashMap, HashSet};

use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::mastodon_feeds::Range;
use super::users::refs;
use super::{posts, visible_author};
use crate::config::Config;
use crate::cursor::Limit;
use crate::dto::{ConversationDto, NotificationKindDto, PostDto, UserRefDto};
use crate::error::AppError;
use crate::federation::outbound;

/// A stored notification row.
#[derive(Clone, Debug, FromRow)]
pub struct NotificationRow {
    /// The notification id.
    pub id: Uuid,
    /// Who caused it.
    pub actor_id: Uuid,
    /// The tuita kind, as stored.
    pub kind: String,
    /// The post it is about.
    pub post_id: Option<Uuid>,
    /// When it happened.
    pub created_at: OffsetDateTime,
}

/// Which notifications a list asks for. Kinds are tuita's stored kinds.
#[derive(Clone, Debug, Default)]
pub struct NotificationFilter {
    /// Only these kinds; `None` for any.
    pub include: Option<Vec<String>>,
    /// Never these kinds.
    pub exclude: Vec<String>,
    /// Only notifications caused by this account.
    pub account: Option<Uuid>,
}

/// A notification with what the mapper needs.
#[derive(Clone, Debug)]
pub struct Resolved {
    /// The notification id.
    pub id: Uuid,
    /// What happened.
    pub kind: NotificationKindDto,
    /// When.
    pub created_at: OffsetDateTime,
    /// Who did it.
    pub actor: UserRefDto,
    /// The post it is about.
    pub post: Option<PostDto>,
}

/// `viewer`'s notifications, `fetch` rows at most, never affiliations.
///
/// # Errors
///
/// Database failures.
pub async fn notification_rows(
    db: &PgPool,
    viewer: Uuid,
    filter: &NotificationFilter,
    range: Range<Uuid>,
    fetch: i64,
) -> Result<Vec<NotificationRow>, AppError> {
    let dir = if range.ascending { "asc" } else { "desc" };
    let sql = format!(
        "select id, actor_id, kind, post_id, created_at from notifications
          where user_id = $1 and kind <> 'affiliation'
            and ($2::text[] is null or kind = any($2))
            and not (kind = any($3))
            and ($4::uuid is null or actor_id = $4)
            and ($5::uuid is null or id < $5) and ($6::uuid is null or id > $6)
          order by id {dir} limit $7"
    );
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(filter.include.as_deref())
        .bind(&filter.exclude)
        .bind(filter.account)
        .bind(range.upper)
        .bind(range.lower)
        .bind(fetch)
        .fetch_all(db)
        .await?)
}

/// One notification of `viewer`.
///
/// # Errors
///
/// Database failures.
pub async fn notification_row(
    db: &PgPool,
    viewer: Uuid,
    id: Uuid,
) -> Result<Option<NotificationRow>, AppError> {
    Ok(sqlx::query_as(
        "select id, actor_id, kind, post_id, created_at from notifications
          where id = $1 and user_id = $2 and kind <> 'affiliation'",
    )
    .bind(id)
    .bind(viewer)
    .fetch_optional(db)
    .await?)
}

/// Resolves `rows` for `viewer` in a few batched queries: rows whose actor the viewer may not
/// see, and rows about a post the viewer may not see, are left out.
///
/// # Errors
///
/// Database failures.
pub async fn resolve(
    db: &PgPool,
    viewer: Uuid,
    rows: &[NotificationRow],
) -> Result<Vec<Resolved>, AppError> {
    let actor_ids: Vec<Uuid> = rows.iter().map(|row| row.actor_id).collect();
    let visible_sql = format!(
        "select u.id from users u where u.id = any($2) and {}",
        visible_author("u", "$1")
    );
    let visible: HashSet<Uuid> = sqlx::query_scalar(sqlx::AssertSqlSafe(visible_sql))
        .bind(viewer)
        .bind(&actor_ids)
        .fetch_all(db)
        .await?
        .into_iter()
        .collect();
    let post_ids: Vec<Uuid> = rows.iter().filter_map(|row| row.post_id).collect();
    let loaded: HashMap<Uuid, PostDto> = posts::load(db, viewer, &post_ids, None)
        .await?
        .into_iter()
        .map(|post| (post.id, post))
        .collect();
    let actors = refs(db, &actor_ids).await?;
    Ok(rows
        .iter()
        .filter(|row| visible.contains(&row.actor_id))
        .filter_map(|row| {
            let kind = NotificationKindDto::parse(&row.kind)?;
            let post = match row.post_id {
                Some(id) => Some(loaded.get(&id)?.clone()),
                None => None,
            };
            Some(Resolved {
                id: row.id,
                kind,
                created_at: row.created_at,
                actor: actors.get(&row.actor_id)?.clone(),
                post,
            })
        })
        .collect())
}

/// Deletes notification `id` of `viewer`. `false` when there is none.
///
/// # Errors
///
/// Database failures.
pub async fn dismiss(db: &PgPool, viewer: Uuid, id: Uuid) -> Result<bool, AppError> {
    let done = sqlx::query("delete from notifications where id = $1 and user_id = $2")
        .bind(id)
        .bind(viewer)
        .execute(db)
        .await?;
    Ok(done.rows_affected() > 0)
}

/// Deletes every notification of `viewer`.
///
/// # Errors
///
/// Database failures.
pub async fn clear(db: &PgPool, viewer: Uuid) -> Result<u64, AppError> {
    let done = sqlx::query("delete from notifications where user_id = $1")
        .bind(viewer)
        .execute(db)
        .await?;
    Ok(done.rows_affected())
}

/// The post `user` has pinned.
///
/// # Errors
///
/// Database failures.
pub async fn pinned_post(db: &PgPool, user: Uuid) -> Result<Option<Uuid>, AppError> {
    Ok(
        sqlx::query_scalar("select pinned_post_id from users where id = $1")
            .bind(user)
            .fetch_optional(db)
            .await?
            .flatten(),
    )
}

/// What a replayed `Idempotency-Key` finds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Claim {
    /// The key is new and now belongs to this request.
    Fresh,
    /// A request with this key made post `0`.
    Post(Uuid),
    /// A request with this key scheduled post `0`.
    Scheduled(Uuid),
    /// A request with this key is still running.
    Running,
}

/// Claims `key` for `user`. Rows older than an hour, and unfinished ones older than a minute (a
/// crashed request), are removed first.
///
/// # Errors
///
/// Database failures.
pub async fn claim_key(db: &PgPool, user: Uuid, key: &str) -> Result<Claim, AppError> {
    sqlx::query(
        "delete from status_idempotency
          where created_at < now() - interval '1 hour'
             or (post_id is null and scheduled_id is null and created_at < now() - interval '1 minute')",
    )
    .execute(db)
    .await?;
    let claimed = sqlx::query(
        "insert into status_idempotency (user_id, key) values ($1, $2) on conflict do nothing",
    )
    .bind(user)
    .bind(key)
    .execute(db)
    .await?
    .rows_affected();
    if claimed > 0 {
        return Ok(Claim::Fresh);
    }
    let found: Option<(Option<Uuid>, Option<Uuid>)> = sqlx::query_as(
        "select post_id, scheduled_id from status_idempotency where user_id = $1 and key = $2",
    )
    .bind(user)
    .bind(key)
    .fetch_optional(db)
    .await?;
    Ok(match found {
        Some((Some(post), _)) => Claim::Post(post),
        Some((None, Some(scheduled))) => Claim::Scheduled(scheduled),
        _ => Claim::Running,
    })
}

/// Records what the request that claimed `key` made.
///
/// # Errors
///
/// Database failures.
pub async fn finish_key(
    db: &PgPool,
    user: Uuid,
    key: &str,
    post: Option<Uuid>,
    scheduled: Option<Uuid>,
) -> Result<(), AppError> {
    sqlx::query(
        "update status_idempotency set post_id = $3, scheduled_id = $4
          where user_id = $1 and key = $2",
    )
    .bind(user)
    .bind(key)
    .bind(post)
    .bind(scheduled)
    .execute(db)
    .await?;
    Ok(())
}

/// Frees `key` after its request failed, so a retry can run.
///
/// # Errors
///
/// Database failures.
pub async fn release_key(db: &PgPool, user: Uuid, key: &str) -> Result<(), AppError> {
    sqlx::query("delete from status_idempotency where user_id = $1 and key = $2")
        .bind(user)
        .bind(key)
        .execute(db)
        .await?;
    Ok(())
}

/// The conversation of `viewer` with `other`, as the list shows it; `None` when they have no
/// messages (or `other` is hidden from the viewer).
///
/// # Errors
///
/// Database failures.
pub async fn conversation_with(
    db: &PgPool,
    viewer: Uuid,
    other: Uuid,
) -> Result<Option<ConversationDto>, AppError> {
    let sql = format!(
        "select exists (select 1 from users u where u.id = $2 and not {})",
        super::blocked_author("u", "$1")
    );
    let shown: bool = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(other)
        .fetch_one(db)
        .await?;
    if !shown {
        return Ok(None);
    }
    let page = super::messages::thread(
        db,
        viewer,
        other,
        None,
        None,
        Limit::new(Some(1)).map_err(|_| AppError::NotFound)?,
    )
    .await?;
    let Some(last) = page.items.into_iter().next() else {
        return Ok(None);
    };
    let unread: i64 = sqlx::query_scalar(
        "select count(*) from messages
          where recipient_id = $1 and sender_id = $2 and read_at is null and deleted_at is null",
    )
    .bind(viewer)
    .bind(other)
    .fetch_one(db)
    .await?;
    let user = refs(db, &[other])
        .await?
        .remove(&other)
        .ok_or(AppError::NotFound)?;
    let peer_key = super::message_keys::get_many(db, &[other])
        .await?
        .remove(&other);
    Ok(Some(ConversationDto {
        user,
        unread,
        last,
        peer_key,
    }))
}

/// Deletes the messages `viewer` sent to `other` and scrubs their text and ciphertext, as
/// [`super::messages::delete`] does for one. Returns how many. Messages `other` sent stay:
/// tuita has no per-account hiding.
///
/// # Errors
///
/// Database failures.
pub async fn delete_sent(
    db: &PgPool,
    config: &Config,
    viewer: Uuid,
    other: Uuid,
) -> Result<u64, AppError> {
    let mut tx = db.begin().await?;
    let ids: Vec<Uuid> = sqlx::query_scalar(
        "update messages set deleted_at = now(), text = '', ciphertext = null
          where sender_id = $1 and recipient_id = $2 and deleted_at is null
          returning id",
    )
    .bind(viewer)
    .bind(other)
    .fetch_all(&mut *tx)
    .await?;
    for id in &ids {
        outbound::message_deleted(&mut tx, config, *id).await?;
    }
    tx.commit().await?;
    Ok(u64::try_from(ids.len()).unwrap_or(u64::MAX))
}

/// The id of `user`'s repost row of `post`: the id of the boost as a Mastodon status.
///
/// # Errors
///
/// Database failures.
pub async fn repost_entry(db: &PgPool, user: Uuid, post: Uuid) -> Result<Option<Uuid>, AppError> {
    Ok(
        sqlx::query_scalar("select id from reposts where user_id = $1 and post_id = $2")
            .bind(user)
            .bind(post)
            .fetch_optional(db)
            .await?,
    )
}
