//! Hourly housekeeping: expired sessions, stale feed snapshots, orphaned uploads, old inbox
//! activity ids, remembered deleted note ids, deleted direct messages, finished scheduled posts
//! and idle in-memory channels and limiter keys.

use std::time::Duration;

use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::db::{deletion, oauth, passkeys};
use crate::error::AppError;
use crate::state::AppState;

/// Gap between two runs.
const INTERVAL: Duration = Duration::from_secs(3600);

/// What one cleanup run removed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CleanupStats {
    /// Sessions past their expiry.
    pub sessions: u64,
    /// Feed snapshots older than the snapshot lifetime.
    pub snapshots: u64,
    /// Uploads that never joined a post or profile, or whose post is deleted, deleted from the
    /// database.
    pub media_rows: u64,
    /// Objects of those uploads removed from the media store (an S3 bucket reports success for a
    /// key it never had).
    pub media_files: u64,
    /// Post view rows older than two days.
    pub post_views: u64,
    /// Idle hub channels dropped.
    pub hub_channels: usize,
    /// Passkey challenges that expired unused.
    pub challenges: u64,
    /// Deleted direct messages older than a week.
    pub messages: u64,
    /// Queued deliveries of those messages.
    pub deliveries: u64,
    /// Published or failed scheduled posts older than a week.
    pub scheduled_posts: u64,
    /// Accounts whose federation key was wiped, once nothing is left to sign with it.
    pub keys: u64,
}

/// Runs the cleanup every hour until `stop` is cancelled. A failed run is logged; the next
/// one still happens.
pub async fn run(state: AppState, stop: CancellationToken) {
    let mut ticker = tokio::time::interval(INTERVAL);
    loop {
        tokio::select! {
            () = stop.cancelled() => return,
            _ = ticker.tick() => {}
        }
        match run_once(&state).await {
            Ok(stats) => tracing::info!(?stats, "cleanup finished"),
            Err(err) => tracing::error!(error = %err, "cleanup failed"),
        }
    }
}

/// Rows of `media m` that nothing needs: not an avatar, banner, link card image or custom emote,
/// not on a post, not listed by a pending scheduled post, not the poster of a video that is still
/// wanted, and either unattached for 24 hours (an upload waits for its post that long), attached
/// to a deleted post, or the poster of a video that is gone. `$1` is a post: when set, only the
/// media of that post.
const ORPHANS: &str = "select m.id, m.path from media m
  where not exists (select 1 from users u where u.avatar_id = m.id or u.banner_id = m.id)
    and not exists (select 1 from post_media pm where pm.media_id = m.id)
    and not exists (select 1 from link_cards lc where lc.image_id = m.id)
    and not exists (select 1 from custom_emojis ce where ce.media_id = m.id)
    and not exists (select 1 from scheduled_posts s
                     where s.status = 'pending' and s.payload->'mediaIds' @> to_jsonb(m.id::text))
    and not exists (select 1 from media o left join posts op on op.id = o.post_id
                     where o.poster_id = m.id and (o.post_id is null or op.deleted_at is null))
    and ((m.post_id is null and m.created_at < now() - interval '24 hours')
         or exists (select 1 from posts dp where dp.id = m.post_id and dp.deleted_at is not null)
         or exists (select 1 from media o where o.poster_id = m.id))
    and ($1::uuid is null or m.post_id = $1
         or exists (select 1 from media o where o.poster_id = m.id and o.post_id = $1))
  for update of m";

/// Removes the orphaned uploads (the rule is [`run_once`]'s), or only those of deleted post `post`. Each
/// object goes before its row, so an object that cannot be removed keeps its row and the next
/// run tries again. Returns the rows and the objects removed.
///
/// # Errors
///
/// Database failures. Objects that cannot be removed are logged, not returned.
pub async fn remove_orphaned_media(
    state: &AppState,
    post: Option<Uuid>,
) -> Result<(u64, u64), AppError> {
    let mut tx = state.db().begin().await?;
    let orphans: Vec<(Uuid, String)> = sqlx::query_as(ORPHANS)
        .bind(post)
        .fetch_all(&mut *tx)
        .await?;
    let mut removed = Vec::with_capacity(orphans.len());
    let mut files = 0;
    for (id, path) in orphans {
        match state.media().delete(&path).await {
            Ok(existed) => {
                files += u64::from(existed);
                removed.push(id);
            }
            Err(err) => tracing::warn!(error = %err, path, "cannot remove an orphaned upload"),
        }
    }
    sqlx::query("delete from media where id = any($1)")
        .bind(&removed)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok((removed.len() as u64, files))
}

/// Runs one cleanup pass.
///
/// Media is orphaned when no post holds it, nobody uses it as avatar, banner, video poster,
/// link card image or custom emote, no pending scheduled post lists it, and it is older than 24
/// hours (an upload waits for its post that long), or when its post is deleted. The object is
/// removed from the store before the row. `post_views` rows older than two days go too,
/// and so do passkey challenges that expired unused. Activity ids the inbox remembered for replay
/// protection (`ap_seen`) go after seven days, and deleted remote note ids (`ap_tombstones`)
/// after thirty. Deleted direct messages go after seven days with their queued deliveries, and so
/// do finished scheduled posts (published or failed). OAuth authorization codes go a day after
/// they expire: a used code has to stay recognisable that long, so that a replay of it still
/// revokes the token it made. The federation key of a deleted local account goes once no
/// delivery is left that it has to sign.
///
/// # Errors
///
/// Database failures. Files that cannot be removed are logged, not returned.
pub async fn run_once(state: &AppState) -> Result<CleanupStats, AppError> {
    let db = state.db();
    let sessions = sqlx::query("delete from sessions where expires_at < now()")
        .execute(db)
        .await?
        .rows_affected();
    let snapshots =
        sqlx::query("delete from feed_snapshots where created_at < now() - interval '20 minutes'")
            .execute(db)
            .await?
            .rows_affected();
    let post_views = sqlx::query("delete from post_views where day < current_date - 2")
        .execute(db)
        .await?
        .rows_affected();
    let (media_rows, media_files) = remove_orphaned_media(state, None).await?;

    let mut tx = db.begin().await?;
    let gone: Vec<Option<String>> = sqlx::query_scalar(
        "delete from messages where deleted_at < now() - interval '7 days' returning ap_id",
    )
    .fetch_all(&mut *tx)
    .await?;
    let messages = gone.len() as u64;
    let notes: Vec<String> = gone.into_iter().flatten().collect();
    let deliveries = deletion::drop_message_deliveries(&mut tx, &notes).await?;
    tx.commit().await?;
    let scheduled_posts = sqlx::query(
        "delete from scheduled_posts
          where status in ('published', 'failed') and publish_at < now() - interval '7 days'",
    )
    .execute(db)
    .await?
    .rows_affected();
    let keys = deletion::wipe_idle_keys(db).await?;

    sqlx::query("delete from ap_seen where received_at < now() - interval '7 days'")
        .execute(db)
        .await?;
    sqlx::query("delete from ap_tombstones where deleted_at < now() - interval '30 days'")
        .execute(db)
        .await?;
    oauth::expire_codes(db).await?;
    let challenges = passkeys::expire_challenges(db).await?;
    let hub_channels = state.hub().sweep();
    state.limits().sweep();
    Ok(CleanupStats {
        sessions,
        snapshots,
        media_rows,
        media_files,
        post_views,
        hub_channels,
        challenges,
        messages,
        deliveries,
        scheduled_posts,
        keys,
    })
}
