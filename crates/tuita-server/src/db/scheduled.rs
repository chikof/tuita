//! Scheduled posts: storing them, listing them, and publishing the due ones as their authors.

use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{Connection, FromRow, PgConnection, PgPool};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::compose::{self, Effects};
use super::media;
use super::post_actions::interaction_target;
use super::settings;
use super::users::{self, UserRow};
use crate::dto::{
    CreatePostRequest, PollInput, ReplyPolicy, ScheduleRequest, ScheduledPostDto, ScheduledStatus,
};
use crate::error::AppError;
use crate::filter::ContentFilter;
use crate::state::AppState;

/// Pending scheduled posts one account may hold.
const MAX_PENDING: i64 = 100;
/// Earliest publish time, counted from now.
const MIN_AHEAD: Duration = Duration::minutes(1);
/// Latest publish time, counted from now.
const MAX_AHEAD: Duration = Duration::days(90);
/// Due posts one worker pass publishes.
const BATCH: i64 = 20;
/// Published entries stay in the list this long.
const KEEP_PUBLISHED: Duration = Duration::days(7);
/// Longest stored failure message.
const MAX_ERROR_CHARS: usize = 200;

const NOT_FOUND: &str = "Scheduled post not found.";
/// The device families a post can record; the stored one is mapped back to a static name.
const CLIENTS: [&str; 9] = [
    "api", "iphone", "ipad", "android", "chromeos", "windows", "mac", "linux", "web",
];

/// The post as it will be created, stored as JSON in `scheduled_posts.payload`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Payload {
    text: String,
    cw: String,
    sensitive: bool,
    reply_policy: Option<ReplyPolicy>,
    media_ids: Vec<Uuid>,
    poll: Option<PollInput>,
    quote_of: Option<Uuid>,
    client: Option<String>,
}

#[derive(FromRow)]
struct Row {
    id: Uuid,
    author_id: Uuid,
    payload: Json<Payload>,
    publish_at: OffsetDateTime,
    status: String,
    post_id: Option<Uuid>,
    error: Option<String>,
    created_at: OffsetDateTime,
}

const COLS: &str = "id, author_id, payload, publish_at, status, post_id, error, created_at";

fn check_time(at: OffsetDateTime, message: &'static str) -> Result<(), AppError> {
    let now = OffsetDateTime::now_utc();
    if at < now + MIN_AHEAD || at > now + MAX_AHEAD {
        return Err(AppError::bad_request(message));
    }
    Ok(())
}

async fn dto(db: &PgPool, rows: Vec<Row>) -> Result<Vec<ScheduledPostDto>, AppError> {
    let ids: Vec<Uuid> = rows
        .iter()
        .flat_map(|row| row.payload.0.media_ids.iter().copied())
        .collect();
    let by_id = media::by_ids(db, &ids).await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let payload = row.payload.0;
            ScheduledPostDto {
                id: row.id,
                publish_at: row.publish_at,
                status: ScheduledStatus::parse(&row.status).unwrap_or(ScheduledStatus::Pending),
                post_id: row.post_id,
                error: row.error,
                created_at: row.created_at,
                text: payload.text,
                cw: payload.cw,
                sensitive: payload.sensitive,
                reply_policy: payload.reply_policy.unwrap_or(ReplyPolicy::Everyone),
                media: payload
                    .media_ids
                    .iter()
                    .filter_map(|id| by_id.get(id).cloned())
                    .collect(),
                poll: payload.poll,
                quote_of: payload.quote_of,
            }
        })
        .collect())
}

async fn one(db: &PgPool, id: Uuid) -> Result<ScheduledPostDto, AppError> {
    let sql = format!("select {COLS} from scheduled_posts where id = $1");
    let row: Row = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .fetch_one(db)
        .await?;
    dto(db, vec![row])
        .await?
        .pop()
        .ok_or(AppError::Missing(NOT_FOUND))
}

/// The account's scheduled posts: pending ones by time, then published ones of the last week
/// and failed ones, at most 100.
///
/// # Errors
///
/// Database failures.
pub async fn list(db: &PgPool, user: Uuid) -> Result<Vec<ScheduledPostDto>, AppError> {
    let sql = format!(
        "select {COLS} from scheduled_posts
          where author_id = $1 and (status <> 'published' or publish_at > $2)
          order by (status = 'pending') desc, publish_at asc limit 100"
    );
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(user)
        .bind(OffsetDateTime::now_utc() - KEEP_PUBLISHED)
        .fetch_all(db)
        .await?;
    dto(db, rows).await
}

/// Schedules a post. Checks it now, as [`compose::create`] would, and reserves its uploads.
///
/// # Errors
///
/// `400` for a time outside 1 minute to 90 days ahead, a reply, 100 pending posts already, or
/// anything [`compose::create`] refuses; `404 Post not found.` for a missing quote target.
pub async fn create(
    state: &AppState,
    author: &UserRow,
    client: Option<&'static str>,
    req: ScheduleRequest,
) -> Result<ScheduledPostDto, AppError> {
    let db = state.db();
    let now = OffsetDateTime::now_utc();
    if req.publish_at < now + MIN_AHEAD {
        return Err(AppError::bad_request(
            "Pick a time at least a minute from now.",
        ));
    }
    if req.publish_at > now + MAX_AHEAD {
        return Err(AppError::bad_request(
            "You can schedule up to 90 days ahead.",
        ));
    }
    let post = req.post;
    if post.reply_to.is_some() {
        return Err(AppError::bad_request("Replies can’t be scheduled."));
    }
    compose::validate(&post)?;
    compose::check_filter(
        &settings::content_filter(db).await?,
        std::slice::from_ref(&post),
    )?;
    let mut tx = db.begin().await?;
    if let Some(quote) = post.quote_of {
        interaction_target(&mut tx, author.id, quote).await?;
    }
    let pending: i64 = sqlx::query_scalar(
        "select count(*) from scheduled_posts where author_id = $1 and status = 'pending'",
    )
    .bind(author.id)
    .fetch_one(&mut *tx)
    .await?;
    if pending >= MAX_PENDING {
        return Err(AppError::bad_request(
            "You can have up to 100 scheduled posts.",
        ));
    }
    let claimed = media::claim(&mut tx, author.id, &post.media_ids).await?;
    compose::check_extras(post.poll.is_some(), &claimed)?;
    let reserved: Vec<String> = post.media_ids.iter().map(Uuid::to_string).collect();
    let taken: bool = sqlx::query_scalar(
        "select exists (select 1 from scheduled_posts
          where status = 'pending' and payload->'mediaIds' ?| $1)",
    )
    .bind(&reserved)
    .fetch_one(&mut *tx)
    .await?;
    if taken {
        return Err(AppError::bad_request(
            "That upload is already part of another scheduled post.",
        ));
    }
    let payload = Payload {
        text: post.text.trim().to_owned(),
        cw: post.cw.trim().to_owned(),
        sensitive: post.sensitive,
        reply_policy: post.reply_policy,
        media_ids: post.media_ids,
        poll: post.poll,
        quote_of: post.quote_of,
        client: client.map(str::to_owned),
    };
    let id = Uuid::now_v7();
    sqlx::query(
        "insert into scheduled_posts (id, author_id, payload, publish_at) values ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(author.id)
    .bind(Json(&payload))
    .bind(req.publish_at)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    one(db, id).await
}

/// Moves a pending post to another time.
///
/// # Errors
///
/// `404 Scheduled post not found.`; `400` for a time outside 1 minute to 90 days ahead.
pub async fn reschedule(
    db: &PgPool,
    user: Uuid,
    id: Uuid,
    at: OffsetDateTime,
) -> Result<ScheduledPostDto, AppError> {
    check_time(at, "Pick a time between a minute and 90 days from now.")?;
    let moved = sqlx::query(
        "update scheduled_posts set publish_at = $3
          where id = $1 and author_id = $2 and status = 'pending'",
    )
    .bind(id)
    .bind(user)
    .bind(at)
    .execute(db)
    .await?
    .rows_affected();
    if moved == 0 {
        return Err(AppError::Missing(NOT_FOUND));
    }
    one(db, id).await
}

/// Removes a pending or failed entry.
///
/// # Errors
///
/// `404 Scheduled post not found.`
pub async fn delete(db: &PgPool, user: Uuid, id: Uuid) -> Result<(), AppError> {
    let gone = sqlx::query(
        "delete from scheduled_posts where id = $1 and author_id = $2 and status <> 'published'",
    )
    .bind(id)
    .bind(user)
    .execute(db)
    .await?
    .rows_affected();
    if gone == 0 {
        return Err(AppError::Missing(NOT_FOUND));
    }
    Ok(())
}

/// What happened to one entry.
enum Outcome {
    Published(Uuid, Effects),
    Failed(String),
}

fn client_name(name: Option<&str>) -> Option<&'static str> {
    CLIENTS.iter().copied().find(|known| Some(*known) == name)
}

/// Publishes one pending entry as its author inside `tx` and records the result on the row.
async fn publish_row(
    state: &AppState,
    tx: &mut PgConnection,
    row: Row,
    filter: &ContentFilter,
) -> Result<Outcome, AppError> {
    let author = match users::by_id(&mut *tx, row.author_id).await? {
        Some(author) if author.suspended_at.is_none() && author.deleted_at.is_none() => author,
        _ => return fail(tx, row.id, "Account unavailable".to_owned()).await,
    };
    let payload = row.payload.0;
    // A quoted post deleted meanwhile is dropped instead of failing the whole post.
    let quote_of = match payload.quote_of {
        Some(quoted) => {
            sqlx::query_scalar::<_, Uuid>(
                "select id from posts where id = $1 and deleted_at is null",
            )
            .bind(quoted)
            .fetch_optional(&mut *tx)
            .await?
        }
        None => None,
    };
    let req = CreatePostRequest {
        text: payload.text,
        cw: payload.cw,
        sensitive: payload.sensitive,
        reply_to: None,
        quote_of,
        reply_policy: payload.reply_policy,
        media_ids: payload.media_ids,
        poll: payload.poll,
    };
    let client = client_name(payload.client.as_deref());
    let mut savepoint = tx.begin().await?;
    match compose::create_in_tx(state, &mut savepoint, &author, client, vec![req], filter).await {
        Ok((mut ids, effects)) => {
            savepoint.commit().await?;
            let post = ids.pop().ok_or(AppError::Missing(NOT_FOUND))?;
            sqlx::query(
                "update scheduled_posts set status = 'published', post_id = $2, error = null
                  where id = $1",
            )
            .bind(row.id)
            .bind(post)
            .execute(&mut *tx)
            .await?;
            Ok(Outcome::Published(post, effects))
        }
        Err(err) => {
            savepoint.rollback().await?;
            // Only a refusal of the post itself is final; a database or other internal error
            // leaves the row pending so the next pass retries it.
            if matches!(
                err,
                AppError::BadRequest(_)
                    | AppError::Forbidden(_)
                    | AppError::Missing(_)
                    | AppError::Conflict(_)
            ) {
                fail(tx, row.id, err.to_string()).await
            } else {
                Err(err)
            }
        }
    }
}

async fn fail(tx: &mut PgConnection, id: Uuid, message: String) -> Result<Outcome, AppError> {
    let message: String = message.chars().take(MAX_ERROR_CHARS).collect();
    sqlx::query("update scheduled_posts set status = 'failed', error = $2 where id = $1")
        .bind(id)
        .bind(&message)
        .execute(tx)
        .await?;
    Ok(Outcome::Failed(message))
}

/// Publishes a pending entry now.
///
/// # Errors
///
/// `404 Scheduled post not found.`; `400` with the reason when publishing fails (the entry is
/// then marked failed).
pub async fn publish_now(state: &AppState, user: &UserRow, id: Uuid) -> Result<Uuid, AppError> {
    let filter = settings::content_filter(state.db()).await?;
    let mut tx = state.db().begin().await?;
    let sql = format!(
        "select {COLS} from scheduled_posts
          where id = $1 and author_id = $2 and status = 'pending' for update"
    );
    let row: Row = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .bind(user.id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::Missing(NOT_FOUND))?;
    let outcome = publish_row(state, &mut tx, row, &filter).await?;
    tx.commit().await?;
    match outcome {
        Outcome::Published(post, effects) => {
            effects.deliver(state).await;
            Ok(post)
        }
        Outcome::Failed(message) => Err(AppError::bad_request(message)),
    }
}

/// Publishes the due entries (at most 20), skipping rows another worker holds. Returns how many
/// were published.
///
/// # Errors
///
/// Database failures.
pub async fn publish_due(state: &AppState) -> Result<usize, AppError> {
    let filter = settings::content_filter(state.db()).await?;
    let mut tx = state.db().begin().await?;
    let sql = format!(
        "select {COLS} from scheduled_posts
          where status = 'pending' and publish_at <= now()
          order by publish_at limit {BATCH} for update skip locked"
    );
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .fetch_all(&mut *tx)
        .await?;
    let mut published = 0;
    let mut effects = Effects::default();
    for row in rows {
        let id = row.id;
        match publish_row(state, &mut tx, row, &filter).await {
            Ok(Outcome::Published(_, post_effects)) => {
                effects.absorb(post_effects);
                published += 1;
            }
            Ok(Outcome::Failed(_)) => {}
            Err(err) => tracing::error!(%id, error = %err, "scheduled post will be retried"),
        }
    }
    tx.commit().await?;
    effects.deliver(state).await;
    Ok(published)
}
