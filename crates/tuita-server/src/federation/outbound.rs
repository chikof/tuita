//! Local actions as queued ActivityPub deliveries.
//!
//! One function per event. Each takes the caller's connection (the transaction of the action,
//! so the rows commit or roll back with it, and no second pool connection is opened while that
//! transaction is open) and returns how many `ap_deliveries` rows it wrote. An action with no
//! remote recipient writes none: local accounts are never delivered to over HTTP.
//!
//! Recipients are the inboxes of the author's accepted remote followers (the shared inbox when
//! known, else the personal one) plus the inboxes of the remote accounts a post mentions,
//! answers or quotes. Every row names the acting local account as signer, so the delivery worker
//! signs with that account's key. Activity ids are stable, so an `Undo` names what it undoes.

use serde_json::{Value, json};
use sqlx::{FromRow, PgConnection};
use time::OffsetDateTime;
use tuita_core::ap::{self, Activity, ActivityKind, PUBLIC, format_time};
use uuid::Uuid;

use super::{ids, keys, render};
use crate::config::Config;
use crate::db::ap::{self as rows, LocalPost, Mentioned};
use crate::db::deliveries::{enqueue, enqueue_many};
use crate::db::{suspended_domain, users};
use crate::dto::Visibility;
use crate::error::AppError;
use crate::secrets;

/// Builds an activity document with `@context`.
fn document(
    kind: ActivityKind,
    id: String,
    actor: &str,
    object: Value,
    audience: (Vec<String>, Vec<String>),
    published: Option<OffsetDateTime>,
) -> Value {
    let mut activity =
        Activity::new(kind, Some(id), actor, object).addressed(audience.0, audience.1);
    activity.published = published.map(format_time);
    let mut doc = ap::document(&activity);
    // Mastodon always sends audiences as arrays, even for one account.
    for key in ["to", "cc"] {
        if let Some(Value::String(one)) = doc.get(key).cloned() {
            doc[key] = json!([one]);
        }
    }
    doc
}

/// The same object without its `@context`, for embedding in an `Undo` or an `Update`.
fn embedded(mut doc: Value) -> Value {
    if let Value::Object(map) = &mut doc {
        map.remove("@context");
    }
    doc
}

/// `a.b#likes/{post}-{micros}`: the `Like` of `post` made at `at`.
fn like_id(actor: &str, post: Uuid, at: OffsetDateTime) -> String {
    format!("{actor}#likes/{post}-{}", at.unix_timestamp_nanos() / 1000)
}

/// The `Announce` of the repost row `repost`.
fn announce_id(actor: &str, repost: Uuid) -> String {
    format!("{actor}#announces/{repost}")
}

/// SQL for the inbox column to use: the shared inbox first, or the personal one first.
const fn inbox_expr(personal: bool) -> &'static str {
    if personal {
        "coalesce(u.inbox_url, u.shared_inbox_url)"
    } else {
        "coalesce(u.shared_inbox_url, u.inbox_url)"
    }
}

/// Inboxes of the accepted remote followers of `author`.
async fn follower_inboxes(conn: &mut PgConnection, author: Uuid) -> Result<Vec<String>, AppError> {
    let inbox = inbox_expr(false);
    let sql = format!(
        "select distinct {inbox} as inbox from follows f join users u on u.id = f.follower_id
          where f.followee_id = $1 and f.state = 'accepted' and u.domain is not null
            and u.deleted_at is null and {inbox} is not null and not {}
          order by 1",
        suspended_domain("u")
    );
    Ok(sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .bind(author)
        .fetch_all(conn)
        .await?)
}

/// Inboxes of every known remote account.
async fn all_remote_inboxes(conn: &mut PgConnection) -> Result<Vec<String>, AppError> {
    let inbox = inbox_expr(false);
    let sql = format!(
        "select distinct {inbox} as inbox from users u
          where u.domain is not null and u.deleted_at is null and {inbox} is not null
            and not {}
          order by 1",
        suspended_domain("u")
    );
    Ok(sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .fetch_all(conn)
        .await?)
}

/// Inboxes of the remote accounts whose actor ids are `actors`.
async fn actor_inboxes(
    conn: &mut PgConnection,
    actors: &[String],
    personal: bool,
) -> Result<Vec<String>, AppError> {
    if actors.is_empty() {
        return Ok(Vec::new());
    }
    let inbox = inbox_expr(personal);
    let sql = format!(
        "select distinct {inbox} as inbox from users u
          where u.ap_id = any($1) and u.domain is not null and u.deleted_at is null
            and {inbox} is not null and not {}
          order by 1",
        suspended_domain("u")
    );
    Ok(sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .bind(actors)
        .fetch_all(conn)
        .await?)
}

/// The handle of local account `user`.
async fn local_handle(conn: &mut PgConnection, user: Uuid) -> Result<Option<String>, AppError> {
    Ok(
        sqlx::query_scalar("select handle from users where id = $1 and domain is null")
            .bind(user)
            .fetch_optional(conn)
            .await?,
    )
}

/// A post and its author as the activities of other accounts need to name them.
#[derive(FromRow)]
struct Target {
    id: Uuid,
    ap_id: Option<String>,
    visibility: String,
    handle: String,
    domain: Option<String>,
    actor_ap_id: Option<String>,
}

impl Target {
    async fn load(conn: &mut PgConnection, post: Uuid) -> Result<Option<Self>, AppError> {
        Ok(sqlx::query_as(
            "select p.id, p.ap_id, p.visibility, u.handle, u.domain, u.ap_id as actor_ap_id
               from posts p join users u on u.id = p.author_id where p.id = $1",
        )
        .bind(post)
        .fetch_optional(conn)
        .await?)
    }

    fn is_remote(&self) -> bool {
        self.domain.is_some()
    }

    fn note_id(&self, config: &Config) -> Option<String> {
        match self.domain {
            None => Some(ids::note_id(config, &self.handle, self.id)),
            Some(_) => self.ap_id.clone(),
        }
    }

    fn actor_id(&self, config: &Config) -> Option<String> {
        match self.domain {
            None => Some(ids::actor_id(config, &self.handle)),
            Some(_) => self.actor_ap_id.clone(),
        }
    }

    fn is_open(&self) -> bool {
        matches!(self.visibility.as_str(), "public" | "unlisted")
    }
}

/// The inboxes a post goes to: its author's remote followers (none for a direct post) and the
/// remote accounts it names.
async fn post_inboxes(
    conn: &mut PgConnection,
    config: &Config,
    post: &LocalPost,
) -> Result<Vec<String>, AppError> {
    let named: Vec<String> = render::recipients(config, post)
        .into_iter()
        .filter(|actor| ids::local_handle_from_actor_id(config, actor).is_none())
        .collect();
    let mut inboxes = if post.visibility == Visibility::Direct {
        Vec::new()
    } else {
        follower_inboxes(conn, post.author.id).await?
    };
    inboxes.extend(actor_inboxes(conn, &named, false).await?);
    Ok(inboxes)
}

async fn queue(
    conn: &mut PgConnection,
    signer: Uuid,
    inboxes: &[String],
    activity: &Value,
) -> Result<usize, AppError> {
    enqueue_many(
        conn,
        Some(signer),
        inboxes.iter().map(String::as_str),
        activity,
    )
    .await
}

/// Delivers the `Create` of a new local post. Call it once the post, its media, poll and tags
/// are written.
///
/// # Errors
///
/// Database failures.
pub async fn post_created(
    conn: &mut PgConnection,
    config: &Config,
    post: Uuid,
) -> Result<usize, AppError> {
    let Some(post) = Box::pin(rows::load_post_in(conn, config.domain(), post)).await? else {
        return Ok(0);
    };
    if post.deleted_at.is_some() {
        return Ok(0);
    }
    let inboxes = post_inboxes(conn, config, &post).await?;
    queue(
        conn,
        post.author.id,
        &inboxes,
        &render::create_value(config, &post),
    )
    .await
}

/// Delivers the `Update` of an edited local post. Call it after the edit is written.
///
/// # Errors
///
/// Database failures.
pub async fn post_updated(
    conn: &mut PgConnection,
    config: &Config,
    post: Uuid,
) -> Result<usize, AppError> {
    let Some(post) = Box::pin(rows::load_post_in(conn, config.domain(), post)).await? else {
        return Ok(0);
    };
    if post.deleted_at.is_some() {
        return Ok(0);
    }
    let inboxes = post_inboxes(conn, config, &post).await?;
    queue(
        conn,
        post.author.id,
        &inboxes,
        &render::update_value(config, &post),
    )
    .await
}

/// Delivers the `Delete` of a local post, and cancels the `Create` and `Update` of the same note
/// still waiting in the queue, so a retry cannot deliver them after the `Delete`. Call it before
/// the post's text is wiped, so the accounts it mentioned still hear of it; on a post that is
/// already soft-deleted it reaches the followers and the accounts of the reply and quote links
/// only. The caller guarantees the post is deleted once.
///
/// # Errors
///
/// Database failures.
pub async fn post_deleted(
    conn: &mut PgConnection,
    config: &Config,
    post: Uuid,
) -> Result<usize, AppError> {
    let Some(post) = Box::pin(rows::load_post_in(conn, config.domain(), post)).await? else {
        return Ok(0);
    };
    sqlx::query(
        "delete from ap_deliveries
          where activity->>'type' in ('Create', 'Update')
            and (activity->'object'->>'id' = $1 or activity->>'object' = $1)",
    )
    .bind(ids::note_id(config, &post.author.handle, post.id))
    .execute(&mut *conn)
    .await?;
    let inboxes = post_inboxes(conn, config, &post).await?;
    queue(
        conn,
        post.author.id,
        &inboxes,
        &render::delete_value(config, &post),
    )
    .await
}

/// Delivers the `Like` that `user` just gave `post`, when the post is a remote account's.
///
/// # Errors
///
/// Database failures.
pub async fn like(
    conn: &mut PgConnection,
    config: &Config,
    user: Uuid,
    post: Uuid,
) -> Result<usize, AppError> {
    like_activity(conn, config, user, post, false).await
}

/// Delivers the `Undo` of a `Like`. Call it before the `likes` row is deleted: it names the
/// `Like` by that row's time, and does nothing when the row is gone.
///
/// # Errors
///
/// Database failures.
pub async fn undo_like(
    conn: &mut PgConnection,
    config: &Config,
    user: Uuid,
    post: Uuid,
) -> Result<usize, AppError> {
    like_activity(conn, config, user, post, true).await
}

async fn like_activity(
    conn: &mut PgConnection,
    config: &Config,
    user: Uuid,
    post: Uuid,
    undo: bool,
) -> Result<usize, AppError> {
    let Some(target) = Target::load(conn, post).await? else {
        return Ok(0);
    };
    let Some(handle) = local_handle(conn, user).await? else {
        return Ok(0);
    };
    let at: Option<OffsetDateTime> =
        sqlx::query_scalar("select created_at from likes where user_id = $1 and post_id = $2")
            .bind(user)
            .bind(post)
            .fetch_optional(&mut *conn)
            .await?;
    let (Some(at), true, Some(note), Some(author)) = (
        at,
        target.is_remote(),
        target.note_id(config),
        target.actor_id(config),
    ) else {
        return Ok(0);
    };
    let actor = ids::actor_id(config, &handle);
    let id = like_id(&actor, post, at);
    let like = document(
        ActivityKind::Like,
        id.clone(),
        &actor,
        Value::String(note),
        (vec![author.clone()], Vec::new()),
        Some(at),
    );
    let activity = if undo {
        document(
            ActivityKind::Undo,
            format!("{id}/undo"),
            &actor,
            embedded(like),
            (vec![author.clone()], Vec::new()),
            Some(OffsetDateTime::now_utc()),
        )
    } else {
        like
    };
    let inboxes = actor_inboxes(conn, &[author], true).await?;
    queue(conn, user, &inboxes, &activity).await
}

/// Delivers the `Announce` of the repost row `repost` (`user` reposted `post`): to the remote
/// followers of the reposter and the remote author of the post. Only public and unlisted posts
/// are announced.
///
/// # Errors
///
/// Database failures.
pub async fn announce(
    conn: &mut PgConnection,
    config: &Config,
    user: Uuid,
    post: Uuid,
    repost: Uuid,
) -> Result<usize, AppError> {
    announce_activity(conn, config, user, post, repost, false).await
}

/// Delivers the `Undo` of an `Announce`. Call it before the `reposts` row is deleted; it does
/// nothing when `user` has no repost of `post`.
///
/// # Errors
///
/// Database failures.
pub async fn undo_announce(
    conn: &mut PgConnection,
    config: &Config,
    user: Uuid,
    post: Uuid,
) -> Result<usize, AppError> {
    let repost: Option<Uuid> =
        sqlx::query_scalar("select id from reposts where user_id = $1 and post_id = $2")
            .bind(user)
            .bind(post)
            .fetch_optional(&mut *conn)
            .await?;
    match repost {
        Some(repost) => announce_activity(conn, config, user, post, repost, true).await,
        None => Ok(0),
    }
}

async fn announce_activity(
    conn: &mut PgConnection,
    config: &Config,
    user: Uuid,
    post: Uuid,
    repost: Uuid,
    undo: bool,
) -> Result<usize, AppError> {
    let Some(target) = Target::load(conn, post).await? else {
        return Ok(0);
    };
    let Some(handle) = local_handle(conn, user).await? else {
        return Ok(0);
    };
    let at: Option<OffsetDateTime> =
        sqlx::query_scalar("select created_at from reposts where id = $1")
            .bind(repost)
            .fetch_optional(&mut *conn)
            .await?;
    let (Some(at), true, Some(note), Some(author)) = (
        at,
        target.is_open(),
        target.note_id(config),
        target.actor_id(config),
    ) else {
        return Ok(0);
    };
    let actor = ids::actor_id(config, &handle);
    let id = announce_id(&actor, repost);
    let audience = (
        vec![PUBLIC.to_owned()],
        vec![ids::followers_url(config, &handle), author.clone()],
    );
    let announce = document(
        ActivityKind::Announce,
        id.clone(),
        &actor,
        Value::String(note),
        audience.clone(),
        Some(at),
    );
    let activity = if undo {
        document(
            ActivityKind::Undo,
            format!("{id}/undo"),
            &actor,
            embedded(announce),
            audience,
            Some(OffsetDateTime::now_utc()),
        )
    } else {
        announce
    };
    let mut inboxes = follower_inboxes(conn, user).await?;
    if target.is_remote() {
        inboxes.extend(actor_inboxes(conn, &[author], false).await?);
    }
    queue(conn, user, &inboxes, &activity).await
}

/// A remote account as a `Follow` addresses it.
#[derive(FromRow)]
struct Followee {
    domain: Option<String>,
    ap_id: Option<String>,
    inbox_url: Option<String>,
    shared_inbox_url: Option<String>,
}

impl Followee {
    async fn load(conn: &mut PgConnection, id: Uuid) -> Result<Option<Self>, AppError> {
        Ok(sqlx::query_as(
            "select domain, ap_id, inbox_url, shared_inbox_url from users where id = $1",
        )
        .bind(id)
        .fetch_optional(conn)
        .await?)
    }

    /// The target's actor id and inbox, for a remote account that has both.
    fn address(&self) -> Option<(&str, &str)> {
        self.domain.as_ref()?;
        Some((
            self.ap_id.as_deref()?,
            self.inbox_url
                .as_deref()
                .or(self.shared_inbox_url.as_deref())?,
        ))
    }
}

/// Delivers a `Follow` of remote account `target` by `me`, and stores its id in
/// `follows.ap_id`. Does nothing for a local target.
///
/// # Errors
///
/// Database failures.
pub async fn follow(
    conn: &mut PgConnection,
    config: &Config,
    me: Uuid,
    target: Uuid,
) -> Result<usize, AppError> {
    let Some(followee) = Followee::load(conn, target).await? else {
        return Ok(0);
    };
    let Some((remote, inbox)) = followee.address() else {
        return Ok(0);
    };
    let Some(handle) = local_handle(conn, me).await? else {
        return Ok(0);
    };
    let actor = ids::actor_id(config, &handle);
    let id = ids::activity_id(config, Uuid::now_v7());
    sqlx::query("update follows set ap_id = $3 where follower_id = $1 and followee_id = $2")
        .bind(me)
        .bind(target)
        .bind(&id)
        .execute(&mut *conn)
        .await?;
    let activity = document(
        ActivityKind::Follow,
        id,
        &actor,
        Value::String(remote.to_owned()),
        (vec![remote.to_owned()], Vec::new()),
        Some(OffsetDateTime::now_utc()),
    );
    enqueue(&mut *conn, Some(me), inbox, &activity).await?;
    Ok(1)
}

/// Delivers the `Undo` of the `Follow` of remote account `target` by `me`, accepted or still
/// pending. `follow_id` is the stored `follows.ap_id`; a follow without one gets a fresh id.
/// Does nothing for a local target.
///
/// # Errors
///
/// Database failures.
pub async fn undo_follow(
    conn: &mut PgConnection,
    config: &Config,
    me: Uuid,
    target: Uuid,
    follow_id: Option<&str>,
) -> Result<usize, AppError> {
    let Some(followee) = Followee::load(conn, target).await? else {
        return Ok(0);
    };
    let Some((remote, inbox)) = followee.address() else {
        return Ok(0);
    };
    let Some(handle) = local_handle(conn, me).await? else {
        return Ok(0);
    };
    let actor = ids::actor_id(config, &handle);
    let id = follow_id.map_or_else(|| ids::activity_id(config, Uuid::now_v7()), str::to_owned);
    let follow = document(
        ActivityKind::Follow,
        id.clone(),
        &actor,
        Value::String(remote.to_owned()),
        (Vec::new(), Vec::new()),
        None,
    );
    let undo = document(
        ActivityKind::Undo,
        format!("{id}#undo"),
        &actor,
        embedded(follow),
        (vec![remote.to_owned()], Vec::new()),
        Some(OffsetDateTime::now_utc()),
    );
    enqueue(&mut *conn, Some(me), inbox, &undo).await?;
    Ok(1)
}

/// Delivers a `Reject` of the `Follow` that remote account `follower` sent to local account
/// `local`, so the other server drops its side of a follow that was removed here (a block).
/// `follow_id` is the stored `follows.ap_id`. Does nothing when `follower` is local.
///
/// # Errors
///
/// Database failures.
pub async fn reject_follow(
    conn: &mut PgConnection,
    config: &Config,
    local: Uuid,
    follower: Uuid,
    follow_id: Option<&str>,
) -> Result<usize, AppError> {
    let Some(remote_follower) = Followee::load(conn, follower).await? else {
        return Ok(0);
    };
    let Some((remote, inbox)) = remote_follower.address() else {
        return Ok(0);
    };
    let Some(handle) = local_handle(conn, local).await? else {
        return Ok(0);
    };
    let actor = ids::actor_id(config, &handle);
    let mut follow = json!({"type": "Follow", "actor": remote, "object": actor});
    if let (Some(id), Some(map)) = (follow_id, follow.as_object_mut()) {
        map.insert("id".to_owned(), Value::String(id.to_owned()));
    }
    let reject = document(
        ActivityKind::Reject,
        ids::activity_id(config, Uuid::now_v7()),
        &actor,
        follow,
        (vec![remote.to_owned()], Vec::new()),
        Some(OffsetDateTime::now_utc()),
    );
    enqueue(&mut *conn, Some(local), inbox, &reject).await?;
    Ok(1)
}

/// Delivers `voter`'s vote on the poll of remote post `post`: one `Create` of a `Note` named
/// after the chosen option for each of `choices` (zero-based option indexes), as Mastodon sends
/// votes. Does nothing for a local poll.
///
/// # Errors
///
/// Database failures.
pub async fn vote(
    conn: &mut PgConnection,
    config: &Config,
    voter: Uuid,
    post: Uuid,
    choices: &[i16],
) -> Result<usize, AppError> {
    let Some(target) = Target::load(conn, post).await? else {
        return Ok(0);
    };
    let (true, Some(question), Some(author)) = (
        target.is_remote(),
        target.note_id(config),
        target.actor_id(config),
    ) else {
        return Ok(0);
    };
    let Some(inbox) = actor_inboxes(conn, std::slice::from_ref(&author), true)
        .await?
        .pop()
    else {
        return Ok(0);
    };
    let Some(handle) = local_handle(conn, voter).await? else {
        return Ok(0);
    };
    let options: Option<Vec<String>> =
        sqlx::query_scalar("select options from polls where post_id = $1")
            .bind(post)
            .fetch_optional(&mut *conn)
            .await?;
    let options = options.unwrap_or_default();
    let actor = ids::actor_id(config, &handle);
    let now = OffsetDateTime::now_utc();
    let mut queued = 0;
    for choice in choices {
        let Some(name) = usize::try_from(*choice)
            .ok()
            .and_then(|index| options.get(index))
        else {
            continue;
        };
        let note_id = format!("{actor}#votes/{}", Uuid::now_v7());
        let note = json!({
            "id": note_id,
            "type": "Note",
            "name": name,
            "attributedTo": actor,
            "to": [author],
            "inReplyTo": question,
            "published": format_time(now),
        });
        let activity = document(
            ActivityKind::Create,
            format!("{note_id}/activity"),
            &actor,
            note,
            (vec![author.clone()], Vec::new()),
            Some(now),
        );
        enqueue(&mut *conn, Some(voter), &inbox, &activity).await?;
        queued += 1;
    }
    Ok(queued)
}

/// Delivers a direct message to remote account `recipient` as a `Create` of a `Note` addressed
/// to it alone, with a `Mention` tag, to its personal inbox. Stores the note id in
/// `messages.ap_id`. Does nothing for a local recipient.
///
/// # Errors
///
/// Database failures.
pub async fn message(
    conn: &mut PgConnection,
    config: &Config,
    sender: Uuid,
    recipient: Uuid,
    message: Uuid,
    text: &str,
    at: OffsetDateTime,
) -> Result<usize, AppError> {
    let Some(to) = users::by_id(&mut *conn, recipient).await? else {
        return Ok(0);
    };
    let Some(remote) = to.ap_id.clone().filter(|_| to.domain.is_some()) else {
        return Ok(0);
    };
    let Some(inbox) = to.inbox_url.clone().or(to.shared_inbox_url.clone()) else {
        return Ok(0);
    };
    let Some(handle) = local_handle(conn, sender).await? else {
        return Ok(0);
    };
    let actor = ids::actor_id(config, &handle);
    let note_id = ids::note_id(config, &handle, message);
    let mentioned = Mentioned {
        id: to.id,
        handle: to.handle.clone(),
        domain: to.domain.clone(),
        ap_id: Some(remote.clone()),
        url: to.url.clone(),
    };
    let name = format!(
        "@{}@{}",
        to.handle,
        to.domain.as_deref().unwrap_or_default()
    );
    let href = to
        .url
        .clone()
        .filter(|url| ap::is_http_url(url))
        .unwrap_or_else(|| remote.clone());
    let chip = format!(
        "<span class=\"h-card\"><a href=\"{}\" class=\"u-url mention\">@<span>{}</span></a></span>",
        ap::escape_html(&href),
        ap::escape_html(&to.handle)
    );
    let body = render::content_html(config, text, std::slice::from_ref(&mentioned));
    let content = match body.strip_prefix("<p>") {
        Some(rest) => format!("<p>{chip} {rest}"),
        None => format!("<p>{chip}</p>{body}"),
    };
    let note = json!({
        "id": note_id,
        "type": "Note",
        "attributedTo": actor,
        "to": [remote],
        "cc": [],
        "content": content,
        "published": format_time(at),
        "sensitive": false,
        "tag": [{"type": "Mention", "href": remote, "name": name}],
    });
    let activity = document(
        ActivityKind::Create,
        format!("{note_id}/activity"),
        &actor,
        note,
        (vec![remote], Vec::new()),
        Some(at),
    );
    sqlx::query("update messages set ap_id = $2 where id = $1")
        .bind(message)
        .bind(&note_id)
        .execute(&mut *conn)
        .await?;
    enqueue(&mut *conn, Some(sender), &inbox, &activity).await?;
    Ok(1)
}

/// Takes back a direct message that a local account deleted: cancels its `Create` still waiting
/// in the queue, and queues a `Delete` of the note for the remote recipient's inbox, since the
/// `Create` may already have gone out (a delivered row leaves the queue). Does nothing for a
/// message between two local accounts or one a remote account sent. Call it in the transaction
/// that deleted the message.
///
/// # Errors
///
/// Database failures.
pub async fn message_deleted(
    conn: &mut PgConnection,
    config: &Config,
    message: Uuid,
) -> Result<usize, AppError> {
    let row: Option<(Uuid, Uuid, Option<String>)> =
        sqlx::query_as("select sender_id, recipient_id, ap_id from messages where id = $1")
            .bind(message)
            .fetch_optional(&mut *conn)
            .await?;
    let Some((sender, recipient, Some(note_id))) = row else {
        return Ok(0);
    };
    let Some(handle) = local_handle(conn, sender).await? else {
        return Ok(0);
    };
    sqlx::query(
        "delete from ap_deliveries
          where activity->>'type' = 'Create' and activity->'object'->>'id' = $1",
    )
    .bind(&note_id)
    .execute(&mut *conn)
    .await?;
    let Some(to) = users::by_id(&mut *conn, recipient).await? else {
        return Ok(0);
    };
    let Some(remote) = to.ap_id.clone().filter(|_| to.domain.is_some()) else {
        return Ok(0);
    };
    let Some(inbox) = to.inbox_url.clone().or(to.shared_inbox_url.clone()) else {
        tracing::warn!(%message, "not telling a remote account to delete a message: it has no inbox");
        return Ok(0);
    };
    let actor = ids::actor_id(config, &handle);
    let now = OffsetDateTime::now_utc();
    let activity = document(
        ActivityKind::Delete,
        format!("{note_id}#delete"),
        &actor,
        json!({ "id": note_id, "type": "Tombstone" }),
        (vec![remote], Vec::new()),
        Some(now),
    );
    Ok(usize::from(
        enqueue(&mut *conn, Some(sender), &inbox, &activity).await?,
    ))
}

/// Delivers an `Update` of the actor of local account `user` to its remote followers. Call it
/// after the profile row changed. Does nothing for a suspended or deleted account.
///
/// An account that predates federation has no key pair yet: it gets one here (generated off the
/// async executor, no I/O), since the actor document carries the public key.
///
/// # Errors
///
/// Database and key failures.
pub async fn profile_updated(
    conn: &mut PgConnection,
    config: &Config,
    user: Uuid,
) -> Result<usize, AppError> {
    let Some(row) = users::by_id(&mut *conn, user).await? else {
        return Ok(0);
    };
    if row.domain.is_some() || row.suspended_at.is_some() || row.deleted_at.is_some() {
        return Ok(0);
    }
    let pem = public_pem(conn, config, user).await?;
    let files = rows::user_files(&mut *conn, &row).await?;
    let mentions = rows::mentions_in(&mut *conn, config.domain(), &row.bio).await?;
    let texts = [row.display_name.as_str(), row.bio.as_str()];
    let catalog = rows::local_emojis(&mut *conn, &texts).await?;
    let actor_doc = render::actor_value(
        config,
        &row,
        &pem,
        (files.0.as_ref(), files.1.as_ref()),
        &mentions,
        &rows::mentioned_emojis(&catalog, &texts),
    );
    let actor = ids::actor_id(config, &row.handle);
    let now = OffsetDateTime::now_utc();
    let activity = document(
        ActivityKind::Update,
        format!("{actor}#updates/{}", now.unix_timestamp_nanos() / 1000),
        &actor,
        embedded(actor_doc),
        (
            vec![PUBLIC.to_owned()],
            vec![ids::followers_url(config, &row.handle)],
        ),
        Some(now),
    );
    let inboxes = follower_inboxes(conn, user).await?;
    queue(conn, user, &inboxes, &activity).await
}

/// The public key of `user`, making the pair when the account has none.
async fn public_pem(
    conn: &mut PgConnection,
    config: &Config,
    user: Uuid,
) -> Result<String, AppError> {
    let read = "select public_key_pem from users where id = $1";
    if let Some(pem) = sqlx::query_scalar::<_, Option<String>>(read)
        .bind(user)
        .fetch_one(&mut *conn)
        .await?
    {
        return Ok(pem);
    }
    let pair = keys::new_pair().await?;
    sqlx::query(
        "update users set public_key_pem = $2, private_key_pem = $3
          where id = $1 and private_key_pem is null",
    )
    .bind(user)
    .bind(&pair.public_pem)
    .bind(secrets::store(
        config.secret_key.as_deref(),
        &secrets::user_context(user),
        &pair.private_pem,
    ))
    .execute(&mut *conn)
    .await?;
    sqlx::query_scalar::<_, Option<String>>(read)
        .bind(user)
        .fetch_one(conn)
        .await?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("key was not stored")))
}

/// Delivers the `Delete` of local account `user`'s actor to every known remote inbox (shared
/// inbox preferred, each once). Call it before the account is anonymised: the activity names
/// the actor by its current handle. The delivery worker signs it with `user`'s key under the
/// key id that actor id implies, so it verifies on servers that still hold the old key.
///
/// # Errors
///
/// Database failures.
pub async fn account_deleted(
    conn: &mut PgConnection,
    config: &Config,
    user: Uuid,
    handle: &str,
) -> Result<usize, AppError> {
    let actor = ids::actor_id(config, handle);
    let now = OffsetDateTime::now_utc();
    let activity = document(
        ActivityKind::Delete,
        format!("{actor}#delete-{}", now.unix_timestamp_nanos() / 1000),
        &actor,
        Value::String(actor.clone()),
        (vec![PUBLIC.to_owned()], Vec::new()),
        Some(now),
    );
    let inboxes = all_remote_inboxes(conn).await?;
    queue(conn, user, &inboxes, &activity).await
}
