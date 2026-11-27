//! Notifications: creating them inside a transaction, delivering them live afterwards, and the
//! unread badge counts.
//!
//! Handlers call [`notify`] with their transaction, keep the returned [`Pending`] values, and
//! call [`deliver`] after the commit. A rolled back transaction therefore never reaches a stream.

use sqlx::{PgConnection, PgPool, Row};
use time::OffsetDateTime;
use uuid::Uuid;

use super::users::{notif_prefs_allow, refs};
use crate::dto::{
    BadgesDto, NotificationEventDto, NotificationKindDto, PostPreviewDto, StreamEvent,
};
use crate::error::AppError;
use crate::state::AppState;

/// Characters of a post that go into a live notification.
const PREVIEW_CHARS: usize = 280;

/// A notification that was stored and still has to be sent to the recipient's open streams.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pending {
    /// The stored notification.
    pub id: Uuid,
    /// Who it is for.
    pub user_id: Uuid,
    /// Who caused it.
    pub actor_id: Uuid,
    /// What happened.
    pub kind: NotificationKindDto,
    /// The post it is about.
    pub post_id: Option<Uuid>,
    /// When it was stored.
    pub created_at: OffsetDateTime,
}

/// Stores a notification for `user` about `actor`'s action, unless the rules say to skip it.
///
/// Skipped: notifying yourself; a recipient that is missing, remote, suspended or deleted; a
/// kind the recipient switched off; an actor the recipient blocked or muted; a second like,
/// repost or follow by the same actor on the same target.
///
/// # Errors
///
/// Database failures.
pub async fn notify(
    conn: &mut PgConnection,
    user: Uuid,
    actor: Uuid,
    kind: NotificationKindDto,
    post: Option<Uuid>,
) -> Result<Option<Pending>, AppError> {
    if user == actor {
        return Ok(None);
    }
    let recipient = sqlx::query(
        "select notif_prefs from users
         where id = $1 and domain is null and suspended_at is null and deleted_at is null",
    )
    .bind(user)
    .fetch_optional(&mut *conn)
    .await?;
    let Some(recipient) = recipient else {
        return Ok(None);
    };
    let prefs: Option<serde_json::Value> = recipient.try_get("notif_prefs")?;
    if !notif_prefs_allow(prefs.as_ref(), kind) {
        return Ok(None);
    }
    let silenced: bool = sqlx::query_scalar(
        "select exists (select 1 from blocks where user_id = $1 and target_id = $2)
             or exists (select 1 from mutes where user_id = $1 and target_id = $2)",
    )
    .bind(user)
    .bind(actor)
    .fetch_one(&mut *conn)
    .await?;
    if silenced {
        return Ok(None);
    }
    if matches!(
        kind,
        NotificationKindDto::Like | NotificationKindDto::Repost | NotificationKindDto::Follow
    ) {
        let seen: bool = sqlx::query_scalar(
            "select exists (select 1 from notifications
                where user_id = $1 and actor_id = $2 and kind = $3 and post_id is not distinct from $4)",
        )
        .bind(user)
        .bind(actor)
        .bind(kind.as_str())
        .bind(post)
        .fetch_one(&mut *conn)
        .await?;
        if seen {
            return Ok(None);
        }
    }
    let id = Uuid::now_v7();
    let created_at: OffsetDateTime = sqlx::query_scalar(
        "insert into notifications (id, user_id, actor_id, kind, post_id)
         values ($1, $2, $3, $4, $5) returning created_at",
    )
    .bind(id)
    .bind(user)
    .bind(actor)
    .bind(kind.as_str())
    .bind(post)
    .fetch_one(&mut *conn)
    .await?;
    Ok(Some(Pending {
        id,
        user_id: user,
        actor_id: actor,
        kind,
        post_id: post,
        created_at,
    }))
}

/// Sends committed notifications to their recipients' open streams, each followed by fresh
/// badge counts. Failures are logged, never returned: the request already succeeded.
pub async fn deliver(state: &AppState, pending: impl IntoIterator<Item = Pending>) {
    for item in pending {
        if !state.hub().is_listening(item.user_id) {
            continue;
        }
        match event_for(state.db(), &item).await {
            Ok(Some(event)) => {
                state
                    .hub()
                    .publish(item.user_id, StreamEvent::Notification(Box::new(event)));
                publish_badges(state, item.user_id).await;
            }
            Ok(None) => {}
            Err(err) => tracing::warn!(error = %err, "cannot build a live notification"),
        }
    }
}

async fn event_for(db: &PgPool, item: &Pending) -> Result<Option<NotificationEventDto>, AppError> {
    let actors = refs(db, &[item.actor_id]).await?;
    let Some(actor) = actors.get(&item.actor_id).cloned() else {
        return Ok(None);
    };
    // Through the loader, like every list: a reply from a followers-only author the recipient
    // does not follow, or a muted word, must not reach the toast.
    let post = match item.post_id {
        None => None,
        Some(post_id) => posts::load_one(db, item.user_id, post_id, None)
            .await?
            .map(|post| PostPreviewDto {
                id: post.id,
                text: post.text.chars().take(PREVIEW_CHARS).collect(),
                cw: post.cw,
            }),
    };
    Ok(Some(NotificationEventDto {
        id: item.id,
        kind: item.kind,
        actor,
        post,
        created_at: item.created_at,
    }))
}

/// The unread counters of `user`: notifications, messages (not deleted, sender not suspended or
/// blocked by `user`) and, for staff, open reports.
///
/// # Errors
///
/// Database failures.
pub async fn badges(db: &PgPool, user: Uuid, staff: bool) -> Result<BadgesDto, AppError> {
    let notifications: i64 = sqlx::query_scalar(
        "select count(*) from notifications n join users a on a.id = n.actor_id
         where n.user_id = $1 and n.read_at is null and a.suspended_at is null",
    )
    .bind(user)
    .fetch_one(db)
    .await?;
    let messages: i64 = sqlx::query_scalar(
        "select count(*) from messages m join users s on s.id = m.sender_id
         where m.recipient_id = $1 and m.read_at is null and m.deleted_at is null
           and s.suspended_at is null and s.deleted_at is null
           and not exists (select 1 from blocks b where b.user_id = $1 and b.target_id = m.sender_id)",
    )
    .bind(user)
    .fetch_one(db)
    .await?;
    let reports = if staff {
        let open: i64 = sqlx::query_scalar("select count(*) from reports where status = 'open'")
            .fetch_one(db)
            .await?;
        Some(open)
    } else {
        None
    };
    Ok(BadgesDto {
        notifications,
        messages,
        reports,
    })
}

/// Pushes `user`'s current badge counts to their open streams; does nothing when none is open.
pub async fn publish_badges(state: &AppState, user: Uuid) {
    if !state.hub().is_listening(user) {
        return;
    }
    let staff: Result<bool, sqlx::Error> =
        sqlx::query_scalar("select role <> 'user' from users where id = $1")
            .bind(user)
            .fetch_one(state.db())
            .await;
    let counts = match staff {
        Ok(staff) => badges(state.db(), user, staff).await,
        Err(err) => Err(err.into()),
    };
    match counts {
        Ok(counts) => state.hub().publish(user, StreamEvent::Badges(counts)),
        Err(err) => tracing::warn!(error = %err, "cannot compute live badges"),
    }
}

// ------------------------------------------------------------------ listing

use std::collections::{HashMap, HashSet};

use tuita_core::notify::{NotificationKind, NotificationRow, group};

use super::{posts, visible_author};
use crate::cursor::Limit;
use crate::dto::{NotificationFilter, NotificationGroupDto, Page, PostDto};

/// A stored notification as the list reads it.
struct RawRow {
    id: Uuid,
    actor_id: Uuid,
    kind: NotificationKindDto,
    post_id: Option<Uuid>,
    read: bool,
    created_at: OffsetDateTime,
}

const fn core_kind(kind: NotificationKindDto) -> NotificationKind {
    match kind {
        NotificationKindDto::Like => NotificationKind::Like,
        NotificationKindDto::Repost => NotificationKind::Repost,
        NotificationKindDto::Reply => NotificationKind::Reply,
        NotificationKindDto::Mention => NotificationKind::Mention,
        NotificationKindDto::Quote => NotificationKind::Quote,
        NotificationKindDto::Follow => NotificationKind::Follow,
        NotificationKindDto::FollowRequest => NotificationKind::FollowRequest,
        NotificationKindDto::Poll => NotificationKind::Poll,
        NotificationKindDto::Affiliation => NotificationKind::Affiliation,
    }
}

const fn dto_kind(kind: NotificationKind) -> NotificationKindDto {
    match kind {
        NotificationKind::Like => NotificationKindDto::Like,
        NotificationKind::Repost => NotificationKindDto::Repost,
        NotificationKind::Reply => NotificationKindDto::Reply,
        NotificationKind::Mention => NotificationKindDto::Mention,
        NotificationKind::Quote => NotificationKindDto::Quote,
        NotificationKind::Follow => NotificationKindDto::Follow,
        NotificationKind::FollowRequest => NotificationKindDto::FollowRequest,
        NotificationKind::Poll => NotificationKindDto::Poll,
        NotificationKind::Affiliation => NotificationKindDto::Affiliation,
    }
}

/// One page of `viewer`'s notifications, grouped.
///
/// Reads raw rows newest first (twice the page size for `all`, so grouping still fills a page),
/// drops the ones whose actor or post the viewer cannot see, and groups the rest. `before` and
/// `next` are ids of raw rows.
///
/// # Errors
///
/// Database failures.
pub async fn list(
    db: &PgPool,
    viewer: Uuid,
    filter: NotificationFilter,
    before: Option<Uuid>,
    limit: Limit,
) -> Result<Page<NotificationGroupDto>, AppError> {
    let take = match filter {
        NotificationFilter::All => limit.size() * 2,
        NotificationFilter::Mentions => limit.size(),
    };
    let rows = sqlx::query(
        "select id, actor_id, kind, post_id, read_at is not null as read, created_at
           from notifications
          where user_id = $1 and ($2::uuid is null or id < $2)
            and ($3 or kind in ('reply', 'mention', 'quote'))
          order by id desc limit $4",
    )
    .bind(viewer)
    .bind(before)
    .bind(filter == NotificationFilter::All)
    .bind(i64::try_from(take).unwrap_or(i64::MAX) + 1)
    .fetch_all(db)
    .await?;
    let mut raw = Vec::with_capacity(rows.len());
    for row in &rows {
        let kind: String = row.try_get("kind")?;
        let Some(kind) = NotificationKindDto::parse(&kind) else {
            continue;
        };
        raw.push(RawRow {
            id: row.try_get("id")?,
            actor_id: row.try_get("actor_id")?,
            kind,
            post_id: row.try_get("post_id")?,
            read: row.try_get("read")?,
            created_at: row.try_get("created_at")?,
        });
    }
    let next = if raw.len() > take {
        raw.truncate(take);
        raw.last().map(|row| row.id.to_string())
    } else {
        None
    };

    let actor_ids: Vec<Uuid> = raw.iter().map(|row| row.actor_id).collect();
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
    let post_ids: Vec<Uuid> = raw.iter().filter_map(|row| row.post_id).collect();
    let loaded: HashMap<Uuid, PostDto> = posts::load(db, viewer, &post_ids, None)
        .await?
        .into_iter()
        .map(|post| (post.id, post))
        .collect();

    let kept: Vec<NotificationRow> = raw
        .iter()
        .filter(|row| visible.contains(&row.actor_id))
        .filter(|row| {
            row.kind == NotificationKindDto::Follow
                || row.post_id.is_none_or(|id| loaded.contains_key(&id))
        })
        .map(|row| NotificationRow {
            id: row.id,
            kind: core_kind(row.kind),
            actor_id: row.actor_id,
            post_id: row.post_id,
            created_at: row.created_at,
            read: row.read,
        })
        .collect();
    let groups = group(&kept);
    let shown: Vec<Uuid> = groups
        .iter()
        .flat_map(|group| group.actors.iter().copied())
        .collect();
    let actors = refs(db, &shown).await?;
    let items = groups
        .into_iter()
        .map(|group| NotificationGroupDto {
            id: group.id,
            kind: dto_kind(group.kind),
            read: group.read,
            created_at: group.created_at,
            actors: group
                .actors
                .iter()
                .filter_map(|id| actors.get(id).cloned())
                .collect(),
            actor_count: group.actor_count,
            post: group.post_id.and_then(|id| loaded.get(&id).cloned()),
        })
        .collect();
    Ok(Page { items, next })
}

/// Marks every unread notification of `user` as read. Returns how many changed.
///
/// # Errors
///
/// Database failures.
pub async fn mark_read(db: &PgPool, user: Uuid) -> Result<u64, AppError> {
    let done = sqlx::query(
        "update notifications set read_at = now() where user_id = $1 and read_at is null",
    )
    .bind(user)
    .execute(db)
    .await?;
    Ok(done.rows_affected())
}
