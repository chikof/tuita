//! Live timeline events: who hears of a new post, a boost or a removal while connected.
//!
//! The Mastodon websocket shows the same home feed as [`super::timeline::following`]: the
//! viewer's own and followed accounts' top-level posts, and boosts by followed accounts. A
//! publisher here finds the online users that feed would list the entry for and sends each an
//! id-only [`StreamEvent`]; the websocket loads the post as that user sees it, which is where
//! blocks, mutes, muted words and visibility apply. Direct posts reach no timeline and no one
//! here.
//!
//! Publishers run after the transaction committed and never fail the request: an error is
//! logged. With nobody online they run no query.

use sqlx::PgConnection;
use uuid::Uuid;

use super::{blocked_author, listable_post, muted_author};
use crate::dto::{PostEventDto, StreamEvent};
use crate::error::AppError;
use crate::realtime::Hub;
use crate::state::AppState;

/// `$2` holds the online users `l.u`; true when `l.u` follows account `{account}`.
fn follows(account: &str) -> String {
    format!(
        "exists (select 1 from follows f where f.follower_id = l.u and f.followee_id = {account} \
         and f.state = 'accepted')"
    )
}

/// Sends an `update` for each new post in `posts` to the author and to the followers who are
/// online. Replies and direct posts are skipped, as in the home feed.
pub async fn posts_created(state: &AppState, posts: &[Uuid]) {
    let online = state.hub().listening();
    if online.is_empty() {
        return;
    }
    let sql = format!(
        "select l.u from posts p, unnest($2::uuid[]) as l(u)
         where p.id = $1 and p.deleted_at is null and p.reply_to_id is null and {}
           and (l.u = p.author_id or {})",
        listable_post("p"),
        follows("p.author_id"),
    );
    for post in posts {
        let audience: Result<Vec<Uuid>, _> = sqlx::query_scalar(sqlx::AssertSqlSafe(sql.clone()))
            .bind(post)
            .bind(&online)
            .fetch_all(state.db())
            .await;
        match audience {
            Ok(users) => publish(state.hub(), &users, *post, *post, None),
            Err(err) => tracing::warn!(error = %err, "cannot find who hears of a new post"),
        }
    }
}

/// Sends an `update` for the boost `repost` of `post` by `reposter` to the online followers of
/// the booster, unless they blocked or muted the booster.
pub async fn reposted(state: &AppState, repost: Uuid, post: Uuid, reposter: Uuid) {
    let online = state.hub().listening();
    if online.is_empty() {
        return;
    }
    let sql = format!(
        "select l.u from posts p, users ru, unnest($3::uuid[]) as l(u)
         where p.id = $1 and p.deleted_at is null and {} and ru.id = $2 and {}
           and not {} and not {}",
        listable_post("p"),
        follows("ru.id"),
        blocked_author("ru", "l.u"),
        muted_author("ru", "l.u"),
    );
    let audience: Result<Vec<Uuid>, _> = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .bind(post)
        .bind(reposter)
        .bind(&online)
        .fetch_all(state.db())
        .await;
    match audience {
        Ok(users) => publish(state.hub(), &users, post, repost, Some(reposter)),
        Err(err) => tracing::warn!(error = %err, "cannot find who hears of a boost"),
    }
}

fn publish(hub: &Hub, users: &[Uuid], post_id: Uuid, entry_id: Uuid, reposted_by: Option<Uuid>) {
    let event = PostEventDto {
        post_id,
        entry_id,
        reposted_by,
    };
    for user in users {
        hub.publish(*user, StreamEvent::Update(Box::new(event)));
    }
}

/// The online users that have to drop timeline entries because a post is being deleted.
#[derive(Debug, Default)]
pub struct Removal {
    /// `(user, entry)`: the entry is the post id, or the id of a boost of it.
    entries: Vec<(Uuid, Uuid)>,
}

impl Removal {
    /// Adds the entries of `other`.
    pub fn absorb(&mut self, other: Self) {
        self.entries.extend(other.entries);
    }
}

/// Finds who has the post `post`, and the boosts of it, in a live timeline. Call it inside the
/// deleting transaction before the post is deleted (the boosts go with it) and hand the result
/// to [`publish_removal`] after the commit.
///
/// # Errors
///
/// Database failures.
pub async fn removal(conn: &mut PgConnection, hub: &Hub, post: Uuid) -> Result<Removal, AppError> {
    let online = hub.listening();
    if online.is_empty() {
        return Ok(Removal::default());
    }
    let sql = format!(
        "select l.u, p.id from posts p, unnest($2::uuid[]) as l(u)
         where p.id = $1 and {} and (l.u = p.author_id or {})
         union all
         select l.u, r.id from reposts r, unnest($2::uuid[]) as l(u)
         where r.post_id = $1 and {}",
        listable_post("p"),
        follows("p.author_id"),
        follows("r.user_id"),
    );
    let entries: Vec<(Uuid, Uuid)> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(post)
        .bind(&online)
        .fetch_all(conn)
        .await?;
    Ok(Removal { entries })
}

/// Sends the `delete` events of a committed [`Removal`].
pub fn publish_removal(hub: &Hub, removal: Removal) {
    for (user, entry) in removal.entries {
        hub.publish(user, StreamEvent::Delete(entry));
    }
}

/// Finds who has the boost of `post` by `reposter` in a live timeline. Call it inside the
/// transaction that removes the boost, before it does, and hand the result to
/// [`publish_removal`] after the commit.
///
/// # Errors
///
/// Database failures.
pub async fn unboosted(
    conn: &mut PgConnection,
    hub: &Hub,
    post: Uuid,
    reposter: Uuid,
) -> Result<Removal, AppError> {
    let online = hub.listening();
    if online.is_empty() {
        return Ok(Removal::default());
    }
    let sql = format!(
        "select l.u, r.id from reposts r, unnest($3::uuid[]) as l(u)
         where r.post_id = $1 and r.user_id = $2 and {}",
        follows("r.user_id"),
    );
    let entries: Vec<(Uuid, Uuid)> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(post)
        .bind(reposter)
        .bind(&online)
        .fetch_all(conn)
        .await?;
    Ok(Removal { entries })
}
