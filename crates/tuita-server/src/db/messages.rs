//! Database access for messages: the conversation list, one conversation, sending, read
//! receipts and deleting.

use std::collections::HashMap;

use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::blocked_author;
use super::emoji;
use super::graph::blocked_between;
use super::message_keys;
use super::users::{UserRow, refs};
use crate::config::Config;
use crate::cursor::{Limit, page};
use crate::dto::{ConversationDto, EmojiDto, MessageDto, MessageMode, Page};
use crate::error::AppError;
use crate::federation::outbound;

/// The columns [`MessageRow`] reads, for a `messages` alias `m`.
const COLS: &str = "m.id, m.sender_id, m.text, m.ciphertext, m.created_at, m.read_at,
    (select su.domain from users su where su.id = m.sender_id) as sender_domain";

#[derive(FromRow)]
struct MessageRow {
    id: Uuid,
    sender_id: Uuid,
    text: String,
    ciphertext: Option<String>,
    created_at: OffsetDateTime,
    read_at: Option<OffsetDateTime>,
    sender_domain: Option<String>,
}

impl MessageRow {
    fn dto(self, viewer: Uuid, emojis: Vec<EmojiDto>) -> MessageDto {
        MessageDto {
            id: self.id,
            text: self.text,
            ciphertext: self.ciphertext,
            created_at: self.created_at,
            mine: self.sender_id == viewer,
            read_at: self.read_at,
            emojis,
        }
    }
}

/// The emotes each message mentions, keyed by message id: one query for all `rows`. A message
/// uses its sender's emotes.
async fn emojis_of(
    db: &PgPool,
    rows: &[&MessageRow],
) -> Result<HashMap<Uuid, Vec<EmojiDto>>, AppError> {
    let mut wanted = emoji::Wanted::default();
    for row in rows {
        wanted.add(row.sender_domain.as_deref(), &row.text);
    }
    let catalog = emoji::Catalog::load(db, wanted).await?;
    Ok(rows
        .iter()
        .map(|row| {
            let mentioned = catalog.mentioned(row.sender_domain.as_deref(), [row.text.as_str()]);
            (row.id, mentioned)
        })
        .collect())
}

/// Why `me` cannot message `other` now, or `None` when they can.
///
/// A recipient who only takes messages from people they follow still receives replies in a
/// conversation they started. A suspended or deleted recipient receives nothing.
///
/// # Errors
///
/// Database failures.
pub async fn blocked_reason(
    db: &PgPool,
    me: Uuid,
    other: &UserRow,
) -> Result<Option<String>, AppError> {
    if other.id == me {
        return Ok(Some("You can\u{2019}t message yourself.".to_owned()));
    }
    if other.suspended_at.is_some() || other.deleted_at.is_some() {
        return Ok(Some("You can\u{2019}t message this account.".to_owned()));
    }
    if blocked_between(db, me, other.id).await? {
        return Ok(Some("You can\u{2019}t message this account.".to_owned()));
    }
    match other.dm_policy.as_str() {
        "none" => Ok(Some(format!(
            "@{} isn\u{2019}t accepting messages.",
            other.handle
        ))),
        "following" => {
            let allowed: bool = sqlx::query_scalar(
                "select exists (select 1 from follows
                    where follower_id = $1 and followee_id = $2 and state = 'accepted')
                 or exists (select 1 from messages
                    where sender_id = $1 and recipient_id = $2 and deleted_at is null)",
            )
            .bind(other.id)
            .bind(me)
            .fetch_one(db)
            .await?;
            Ok((!allowed).then(|| {
                format!(
                    "@{} only accepts messages from people they follow.",
                    other.handle
                )
            }))
        }
        _ => Ok(None),
    }
}

/// How a message from one account to another travels, and the recipient's key it is sealed to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route {
    /// The conversation's mode.
    pub mode: MessageMode,
    /// The recipient's current public key; `None` for a remote account and for a local one
    /// without a key.
    pub peer_key: Option<String>,
}

/// The mode of the conversation of `me` with `other`: the one place that decides it.
///
/// - A remote account: `plain`, nothing can seal to it.
/// - Both local accounts have a key: `sealed`.
/// - Otherwise `unavailable`, except that with `MESSAGES_E2EE_REQUIRED` off a recipient
///   without a key gets `plain`. A recipient who has a key never gets `plain`, in either setting.
///
/// # Errors
///
/// Database failures.
pub async fn route(
    db: &PgPool,
    config: &Config,
    me: Uuid,
    other: &UserRow,
) -> Result<Route, AppError> {
    if other.domain.is_some() {
        return Ok(Route {
            mode: MessageMode::Plain,
            peer_key: None,
        });
    }
    let mut keys = message_keys::get_many(db, &[me, other.id]).await?;
    let mine = keys.remove(&me).is_some();
    let peer_key = keys.remove(&other.id);
    let mode = match (mine, peer_key.is_some()) {
        (true, true) => MessageMode::Sealed,
        (_, false) if !config.messages_e2ee_required => MessageMode::Plain,
        _ => MessageMode::Unavailable,
    };
    Ok(Route { mode, peer_key })
}

/// One page of `viewer`'s conversations, newest first. `before` is the id of the newest message
/// of the last conversation of the previous page.
///
/// Conversations with suspended or deleted accounts and with accounts blocked either way are
/// left out; muted accounts stay.
///
/// # Errors
///
/// Database failures.
pub async fn conversations(
    db: &PgPool,
    viewer: Uuid,
    before: Option<Uuid>,
    limit: Limit,
) -> Result<Page<ConversationDto>, AppError> {
    let gone = blocked_author("u", "$1");
    let sql = format!(
        "select c.other, c.last_id from (
            select distinct on (other) other, id as last_id from (
                select id, case when sender_id = $1 then recipient_id else sender_id end as other
                  from messages
                 where (sender_id = $1 or recipient_id = $1) and deleted_at is null
            ) m order by other, id desc
         ) c join users u on u.id = c.other
         where not {gone} and ($2::uuid is null or c.last_id < $2)
         order by c.last_id desc limit $3"
    );
    let rows: Vec<(Uuid, Uuid)> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(before)
        .bind(limit.fetch())
        .fetch_all(db)
        .await?;
    let Page { items: rows, next } = page(rows, limit, |(_, last)| last.to_string());

    let others: Vec<Uuid> = rows.iter().map(|(other, _)| *other).collect();
    let last_ids: Vec<Uuid> = rows.iter().map(|(_, last)| *last).collect();
    let sql = format!("select {COLS} from messages m where m.id = any($1)");
    let mut last: HashMap<Uuid, MessageRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(&last_ids)
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|row: MessageRow| (row.id, row))
        .collect();
    let shown: Vec<&MessageRow> = last.values().collect();
    let mut emojis = emojis_of(db, &shown).await?;
    let unread: HashMap<Uuid, i64> = sqlx::query_as(
        "select sender_id, count(*) from messages
          where recipient_id = $1 and sender_id = any($2) and read_at is null and deleted_at is null
          group by sender_id",
    )
    .bind(viewer)
    .bind(&others)
    .fetch_all(db)
    .await?
    .into_iter()
    .collect();
    let users = refs(db, &others).await?;
    let mut peer_keys = message_keys::get_many(db, &others).await?;

    let items = rows
        .into_iter()
        .filter_map(|(other, last_id)| {
            let row = last.remove(&last_id)?;
            let mentioned = emojis.remove(&last_id).unwrap_or_default();
            Some(ConversationDto {
                user: users.get(&other)?.clone(),
                unread: unread.get(&other).copied().unwrap_or(0),
                last: row.dto(viewer, mentioned),
                peer_key: peer_keys.remove(&other),
            })
        })
        .collect();
    Ok(Page { items, next })
}

/// One page of the conversation between `viewer` and `other`, oldest first, newest page by
/// default. `before` asks for messages older than that id, `after` for newer ones only.
/// `next` is the cursor of the older page.
///
/// # Errors
///
/// Database failures.
pub async fn thread(
    db: &PgPool,
    viewer: Uuid,
    other: Uuid,
    before: Option<Uuid>,
    after: Option<Uuid>,
    limit: Limit,
) -> Result<Page<MessageDto>, AppError> {
    let sql = format!(
        "select {COLS} from messages m
          where least(sender_id, recipient_id) = least($1::uuid, $2::uuid)
            and greatest(sender_id, recipient_id) = greatest($1::uuid, $2::uuid)
            and deleted_at is null
            and ($3::uuid is null or id < $3) and ($4::uuid is null or id > $4)
          order by id desc limit $5"
    );
    let rows: Vec<MessageRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(other)
        .bind(before)
        .bind(after)
        .bind(limit.fetch())
        .fetch_all(db)
        .await?;
    let Page { mut items, next } = page(rows, limit, |row| row.id.to_string());
    items.reverse();
    let shown: Vec<&MessageRow> = items.iter().collect();
    let mut emojis = emojis_of(db, &shown).await?;
    Ok(Page {
        items: items
            .into_iter()
            .map(|row| {
                let mentioned = emojis.remove(&row.id).unwrap_or_default();
                row.dto(viewer, mentioned)
            })
            .collect(),
        next,
    })
}

/// What a message holds: text, or a sealed message with the keys it names.
#[derive(Clone, Copy)]
pub enum Content<'a> {
    /// Plaintext, for a `plain` conversation.
    Plain(&'a str),
    /// A sealed message from the sender's key `sender_key` to the recipient's key
    /// `recipient_key`.
    Sealed {
        /// The whole sealed message.
        ciphertext: &'a str,
        /// The sender key part of `ciphertext`.
        sender_key: &'a str,
        /// The recipient key part of `ciphertext`.
        recipient_key: &'a str,
    },
}

/// Stores a message from `sender` to `recipient`.
///
/// The insert itself checks what the caller decided from [`route`], so a key that changed since
/// then cannot slip through: a plaintext message is refused once the recipient has a key, a
/// sealed one unless both key parts are the accounts' current keys.
///
/// # Errors
///
/// `409` when a key changed, database failures.
pub async fn send(
    db: &PgPool,
    config: &Config,
    sender: Uuid,
    recipient: Uuid,
    content: Content<'_>,
) -> Result<MessageDto, AppError> {
    let id = Uuid::now_v7();
    let mut tx = db.begin().await?;
    let created_at: Option<OffsetDateTime> = match content {
        Content::Plain(text) => {
            sqlx::query_scalar(
                "insert into messages (id, sender_id, recipient_id, text)
                 select $1::uuid, $2::uuid, $3::uuid, $4::text
                  where not exists (select 1 from message_keys where user_id = $3)
                 returning created_at",
            )
            .bind(id)
            .bind(sender)
            .bind(recipient)
            .bind(text)
            .fetch_optional(&mut *tx)
            .await?
        }
        Content::Sealed {
            ciphertext,
            sender_key,
            recipient_key,
        } => {
            sqlx::query_scalar(
                "insert into messages (id, sender_id, recipient_id, text, ciphertext)
                 select $1::uuid, $2::uuid, $3::uuid, '', $4::text
                  where exists (select 1 from message_keys where user_id = $2 and public_key = $5)
                    and exists (select 1 from message_keys where user_id = $3 and public_key = $6)
                 returning created_at",
            )
            .bind(id)
            .bind(sender)
            .bind(recipient)
            .bind(ciphertext)
            .bind(sender_key)
            .bind(recipient_key)
            .fetch_optional(&mut *tx)
            .await?
        }
    };
    let created_at = created_at
        .ok_or_else(|| AppError::conflict("Encryption keys changed. Reload and try again."))?;
    if let Content::Plain(text) = content {
        outbound::message(&mut tx, config, sender, recipient, id, text, created_at).await?;
    }
    tx.commit().await?;
    let (text, ciphertext, emojis) = match content {
        Content::Plain(text) => (
            text.to_owned(),
            None,
            emoji::resolve(db, None, &[text]).await?,
        ),
        Content::Sealed { ciphertext, .. } => {
            (String::new(), Some(ciphertext.to_owned()), Vec::new())
        }
    };
    Ok(MessageDto {
        id,
        text,
        ciphertext,
        created_at,
        mine: true,
        read_at: None,
        emojis,
    })
}

/// Marks the messages `other` sent to `reader` as read. Returns how many changed.
///
/// # Errors
///
/// Database failures.
pub async fn mark_read(db: &PgPool, reader: Uuid, other: Uuid) -> Result<u64, AppError> {
    let done = sqlx::query(
        "update messages set read_at = now()
          where sender_id = $2 and recipient_id = $1 and read_at is null and deleted_at is null",
    )
    .bind(reader)
    .bind(other)
    .execute(db)
    .await?;
    Ok(done.rows_affected())
}

/// What deleting a message changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Deleted {
    /// Who received it.
    pub recipient: Uuid,
    /// The recipient had not read it, so their unread badge changed.
    pub was_unread: bool,
}

/// Deletes message `id` when `sender` wrote it, and scrubs its text and ciphertext: a deleted
/// message keeps no content. A copy still waiting to go to a remote inbox is cancelled and the
/// remote server is told to delete it. `None` when there is no such message.
///
/// # Errors
///
/// Database failures.
pub async fn delete(
    db: &PgPool,
    config: &Config,
    sender: Uuid,
    id: Uuid,
) -> Result<Option<Deleted>, AppError> {
    let mut tx = db.begin().await?;
    let row: Option<(Uuid, bool)> = sqlx::query_as(
        "update messages set deleted_at = now(), text = '', ciphertext = null
          where id = $1 and sender_id = $2 and deleted_at is null
          returning recipient_id, read_at is null",
    )
    .bind(id)
    .bind(sender)
    .fetch_optional(&mut *tx)
    .await?;
    if row.is_some() {
        outbound::message_deleted(&mut tx, config, id).await?;
    }
    tx.commit().await?;
    Ok(row.map(|(recipient, was_unread)| Deleted {
        recipient,
        was_unread,
    }))
}
