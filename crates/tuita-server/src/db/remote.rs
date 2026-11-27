//! Accounts and posts of other servers: the SQL behind remote lookups and inbound objects.
//!
//! The functions here take values that [`crate::federation`] already validated (host bindings,
//! URL schemes, lengths). They never reach the network, so callers fetch first and write after.

use sqlx::{PgConnection, PgPool, Row as _};
use time::OffsetDateTime;
use uuid::Uuid;

use super::live;
use super::notifications::{Pending, notify};
use super::post_actions::{self, bump, soft_delete};
use super::users::UserRow;
use super::{deletion, media};
use crate::dto::{NotificationKindDto, Visibility};
use crate::error::AppError;
use crate::media::NewMedia;
use crate::realtime::Hub;

/// A remote actor after validation, ready to store.
#[allow(
    clippy::struct_excessive_bools,
    reason = "one column per flag the actor sets"
)]
#[derive(Clone, Debug)]
pub struct RemoteProfile {
    /// The actor's id, its identity.
    pub ap_id: String,
    /// `preferredUsername`.
    pub handle: String,
    /// The instance the actor lives on: the authority of its id.
    pub domain: String,
    /// Display name.
    pub display_name: String,
    /// Biography as plain text.
    pub bio: String,
    /// The `Website` profile field when it is an http(s) address, else empty.
    pub website: String,
    /// A `Service` actor.
    pub is_bot: bool,
    /// An `Organization` actor.
    pub is_org: bool,
    /// `manuallyApprovesFollowers`.
    pub locked: bool,
    /// Listed in search.
    pub discoverable: bool,
    /// The personal inbox.
    pub inbox_url: String,
    /// The shared inbox.
    pub shared_inbox_url: Option<String>,
    /// The outbox.
    pub outbox_url: Option<String>,
    /// The followers collection.
    pub followers_url: Option<String>,
    /// The collection of pinned posts.
    pub featured_url: Option<String>,
    /// The profile page.
    pub url: Option<String>,
    /// The key that signs the actor's requests, PEM.
    pub public_key_pem: String,
    /// When the account was created, already clamped to now.
    pub created_at: Option<OffsetDateTime>,
}

/// What [`upsert_actor`] did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Upsert {
    /// The account row is stored or updated.
    Stored(Uuid),
    /// This server deleted the account (the remote deleted it); it stays gone.
    Gone,
    /// Another account holds the handle on that domain under a different id and cannot be
    /// replaced right now.
    Conflict,
}

/// What to do with an avatar or banner column.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MediaSlot {
    /// Leave the stored file.
    Keep,
    /// The actor has none.
    Clear,
    /// Use this upload, copied from `src`.
    Set {
        /// The copied upload.
        id: Uuid,
        /// The URL it was copied from.
        src: String,
    },
}

fn constraint_is(err: &sqlx::Error, name: &str) -> bool {
    err.as_database_error()
        .is_some_and(|db| db.constraint() == Some(name))
}

/// Stores `profile` as a remote account, or updates the one it names. The row is found by actor
/// id. A different actor id on a handle that is already stored (same instance, same name) is
/// [`Upsert::Conflict`], never a takeover: it would inherit the followers and posts of the
/// account, and an instance has no proof to give that it owns the new id. A moved account is a
/// new account. Local moderation state (suspension, verified badge) is never touched. A
/// deleted account is not brought back.
///
/// # Errors
///
/// Database failures.
pub async fn upsert_actor(db: &PgPool, profile: &RemoteProfile) -> Result<Upsert, AppError> {
    for _ in 0..3 {
        let known: Option<bool> =
            sqlx::query_scalar("select deleted_at is not null from users where ap_id = $1")
                .bind(&profile.ap_id)
                .fetch_optional(db)
                .await?;
        let stored = match known {
            Some(true) => return Ok(Upsert::Gone),
            Some(false) => update_by_ap_id(db, profile).await,
            None => insert_or_take_over(db, profile).await,
        };
        match stored {
            Ok(Some(id)) => return Ok(Upsert::Stored(id)),
            Ok(None) => {
                tracing::warn!(
                    actor = profile.ap_id,
                    handle = profile.handle,
                    domain = profile.domain,
                    "refused an actor that claims a handle another actor id holds"
                );
                return Ok(Upsert::Conflict);
            }
            // Another request inserted the same actor between our read and write: read again.
            Err(err) if constraint_is(&err, "users_ap_id_key") => {}
            Err(err) if constraint_is(&err, "users_handle") => return Ok(Upsert::Conflict),
            Err(err) => return Err(err.into()),
        }
    }
    Ok(Upsert::Conflict)
}

/// The profile columns, `$5` to `$18`, shared by the update and the takeover.
const PROFILE_SET: &str = "display_name = $5, bio = $6, website = $7, \
    is_org = $8, is_bot = $9, discoverable = $10, manually_approves = $11, inbox_url = $12, \
    shared_inbox_url = $13, outbox_url = $14, followers_url = $15, featured_url = $16, \
    url = $17, public_key_pem = $18, fetched_at = now()";

async fn update_by_ap_id(db: &PgPool, p: &RemoteProfile) -> Result<Option<Uuid>, sqlx::Error> {
    let sql = format!(
        "update users set handle = $2, {PROFILE_SET}
          where ap_id = $1 and domain is not null and deleted_at is null returning id"
    );
    bind_profile(sqlx::query_scalar(sqlx::AssertSqlSafe(sql)), p)
        .fetch_optional(db)
        .await
}

async fn insert_or_take_over(db: &PgPool, p: &RemoteProfile) -> Result<Option<Uuid>, sqlx::Error> {
    let sql = format!(
        "insert into users (ap_id, handle, domain, created_at, display_name, bio, website,
                is_org, is_bot, discoverable, manually_approves, inbox_url, shared_inbox_url,
                outbox_url, followers_url, featured_url, url, public_key_pem, fetched_at, id)
         values ($1, $2, $3, coalesce($4, now()), $5, $6, $7, $8, $9, $10, $11, $12, $13,
                $14, $15, $16, $17, $18, now(), $19)
         on conflict (lower(handle), (coalesce(domain, ''))) do update set
                ap_id = $1, {PROFILE_SET}
          where users.domain is not null and users.deleted_at is null and users.ap_id = $1
         returning id"
    );
    bind_profile(sqlx::query_scalar(sqlx::AssertSqlSafe(sql)), p)
        .bind(Uuid::now_v7())
        .fetch_optional(db)
        .await
}

type ProfileQuery<'q> = sqlx::query::QueryScalar<
    'q,
    sqlx::Postgres,
    Uuid,
    <sqlx::Postgres as sqlx::Database>::Arguments,
>;

fn bind_profile<'q>(query: ProfileQuery<'q>, p: &'q RemoteProfile) -> ProfileQuery<'q> {
    query
        .bind(&p.ap_id)
        .bind(&p.handle)
        .bind(&p.domain)
        .bind(p.created_at)
        .bind(&p.display_name)
        .bind(&p.bio)
        .bind(&p.website)
        .bind(p.is_org)
        .bind(p.is_bot)
        .bind(p.discoverable)
        .bind(p.locked)
        .bind(&p.inbox_url)
        .bind(&p.shared_inbox_url)
        .bind(&p.outbox_url)
        .bind(&p.followers_url)
        .bind(&p.featured_url)
        .bind(&p.url)
        .bind(&p.public_key_pem)
}

/// The URLs the avatar and banner of remote account `user` were last copied from, each only
/// while that upload is still set.
///
/// # Errors
///
/// Database failures.
pub async fn image_sources(
    db: &PgPool,
    user: Uuid,
) -> Result<(Option<String>, Option<String>), AppError> {
    Ok(sqlx::query_as(
        "select case when avatar_id is not null then avatar_src end,
                case when banner_id is not null then banner_src end
           from users where id = $1",
    )
    .bind(user)
    .fetch_one(db)
    .await?)
}

/// Points the avatar and banner of remote account `user` at copied uploads.
///
/// # Errors
///
/// Database failures.
pub async fn set_images(
    db: &PgPool,
    user: Uuid,
    avatar: MediaSlot,
    banner: MediaSlot,
) -> Result<(), AppError> {
    let column = |slot: MediaSlot| match slot {
        MediaSlot::Keep => (false, None, None),
        MediaSlot::Clear => (true, None, None),
        MediaSlot::Set { id, src } => (true, Some(id), Some(src)),
    };
    let (set_avatar, avatar, avatar_src) = column(avatar);
    let (set_banner, banner, banner_src) = column(banner);
    sqlx::query(
        "update users set avatar_id = case when $2 then $3 else avatar_id end,
                          avatar_src = case when $2 then $4 else avatar_src end,
                          banner_id = case when $5 then $6 else banner_id end,
                          banner_src = case when $5 then $7 else banner_src end
          where id = $1 and domain is not null",
    )
    .bind(user)
    .bind(set_avatar)
    .bind(avatar)
    .bind(avatar_src)
    .bind(set_banner)
    .bind(banner)
    .bind(banner_src)
    .execute(db)
    .await?;
    Ok(())
}

/// Inserts the `media` row of a copied file owned by remote account `owner`.
///
/// # Errors
///
/// Database failures.
pub(crate) async fn insert_media(
    db: &PgPool,
    owner: Uuid,
    alt: &str,
    media: &NewMedia,
) -> Result<(), AppError> {
    let mut conn = db.acquire().await?;
    crate::media::insert(&mut conn, owner, alt, None, media).await
}

/// The remote account `handle`@`domain`, in any state.
///
/// # Errors
///
/// Database failures.
pub async fn find(db: &PgPool, handle: &str, domain: &str) -> Result<Option<UserRow>, AppError> {
    let sql = format!(
        "select {} from users u where lower(u.handle) = lower($1) and u.domain = $2",
        super::users::COLS
    );
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(handle)
        .bind(domain)
        .fetch_optional(db)
        .await?)
}

/// The remote account with actor id `ap_id`, in any state.
///
/// # Errors
///
/// Database failures.
pub async fn find_by_ap_id(db: &PgPool, ap_id: &str) -> Result<Option<UserRow>, AppError> {
    let sql = format!(
        "select {} from users u where u.ap_id = $1 and u.domain is not null",
        super::users::COLS
    );
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(ap_id)
        .fetch_optional(db)
        .await?)
}

/// After a failed refresh, makes the profile due again in an hour instead of on every view.
///
/// # Errors
///
/// Database failures.
pub async fn delay_refresh(db: &PgPool, user: Uuid) -> Result<(), AppError> {
    sqlx::query("update users set fetched_at = now() - interval '23 hours' where id = $1")
        .bind(user)
        .execute(db)
        .await?;
    Ok(())
}

/// Whether `domain` (or a parent of it) has a `suspend` block. A trailing dot and capitals in
/// `domain` do not matter.
///
/// # Errors
///
/// Database failures.
pub async fn domain_suspended(db: &PgPool, domain: &str) -> Result<bool, AppError> {
    let sql = format!(
        "select exists (select 1 from domain_blocks b where b.severity = 'suspend' and {})",
        super::domain_in_block("$1::text", "b.domain")
    );
    Ok(sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .bind(domain.trim_end_matches('.').to_ascii_lowercase())
        .fetch_one(db)
        .await?)
}

/// Removes a remote account the way an account deletion does (posts hidden, follows and
/// messages gone, handle freed) and drops the delivery data, so nothing is sent to it again. The
/// row and its `ap_id` stay: a later lookup finds the tombstone instead of fetching again.
///
/// # Errors
///
/// Database failures.
pub async fn forget_actor(db: &PgPool, user: &UserRow) -> Result<(), AppError> {
    if user.domain.is_none() {
        return Ok(());
    }
    deletion::delete_account(db, user).await?;
    sqlx::query(
        "update users set inbox_url = null, shared_inbox_url = null, outbox_url = null,
                followers_url = null, featured_url = null, url = null, public_key_pem = null
          where id = $1 and domain is not null",
    )
    .bind(user.id)
    .execute(db)
    .await?;
    Ok(())
}

// ------------------------------------------------------------------ addressing

/// What [`addressed_to_us`] asks about an incoming note.
#[derive(Debug)]
pub struct Audience<'a> {
    /// Lower-case handles of local accounts the note is addressed to or mentions.
    pub local_handles: &'a [String],
    /// The note's author.
    pub author: Uuid,
    /// The local post it replies to: the author handle and the post id.
    pub reply_to: Option<(&'a str, Uuid)>,
}

/// Whether a note is worth storing: it is addressed to or mentions a local account, is by an
/// account a local user follows, or answers a local post. A followers collection in `to` or `cc`
/// proves nothing: any author can name another account's collection.
///
/// Follows count in any state: a `pending` one means the local user asked and the other server
/// may already be sending.
///
/// # Errors
///
/// Database failures.
pub async fn addressed_to_us(db: &PgPool, audience: &Audience<'_>) -> Result<bool, AppError> {
    let (reply_handle, reply_id) = match audience.reply_to {
        Some((handle, id)) => (Some(handle), Some(id)),
        None => (None, None),
    };
    Ok(sqlx::query_scalar(
        "select exists (select 1 from users where domain is null and deleted_at is null
                           and suspended_at is null and lower(handle) = any($1))
             or exists (select 1 from follows f join users l on l.id = f.follower_id
                         where f.followee_id = $2 and l.domain is null and l.deleted_at is null)
             or ($3::uuid is not null and exists (select 1 from posts p join users u on u.id = p.author_id
                         where p.id = $3 and lower(u.handle) = lower($4) and u.domain is null
                           and p.deleted_at is null))",
    )
    .bind(audience.local_handles)
    .bind(audience.author)
    .bind(reply_id)
    .bind(reply_handle)
    .fetch_one(db)
    .await?)
}

// ------------------------------------------------------------------ notes

/// A post a note answers or quotes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PostRef {
    /// A post of this instance: the author handle and the post id.
    Local {
        /// The author's handle.
        handle: String,
        /// The post.
        id: Uuid,
    },
    /// A remote post by its `ap_id`.
    Remote(String),
}

/// One poll of a remote `Question`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewPoll {
    /// The answers.
    pub options: Vec<String>,
    /// A voter may pick several.
    pub multiple: bool,
    /// When it ends.
    pub ends_at: OffsetDateTime,
    /// Votes per answer, as the other server counts them.
    pub counts: Vec<i32>,
    /// Accounts that voted.
    pub voters: i32,
}

/// A remote note, mapped to what tuita stores.
#[derive(Clone, Debug)]
pub struct NewNote {
    /// The note's id.
    pub ap_id: String,
    /// The stored remote account that wrote it.
    pub author: Uuid,
    /// Plain text.
    pub text: String,
    /// Content warning.
    pub cw: String,
    /// Media is sensitive.
    pub sensitive: bool,
    /// Who may see it.
    pub visibility: Visibility,
    /// When it was written, not in the future.
    pub created_at: OffsetDateTime,
    /// When it was last edited.
    pub edited_at: Option<OffsetDateTime>,
    /// The page of the post.
    pub url: Option<String>,
    /// The post it answers.
    pub reply_to: Option<PostRef>,
    /// The post it quotes.
    pub quote_of: Option<PostRef>,
    /// Lower-case hashtags without `#`.
    pub tags: Vec<String>,
    /// Lower-case handles of local accounts that see the note because it mentions them or, for a
    /// direct note, addresses them.
    pub local_handles: Vec<String>,
    /// The poll of a `Question`.
    pub poll: Option<NewPoll>,
    /// Copied uploads, in order.
    pub media: Vec<Uuid>,
    /// Create notifications for mentions, replies and quotes.
    pub notify: bool,
}

/// A newly stored post.
#[derive(Debug)]
pub struct Created {
    /// The post.
    pub id: Uuid,
    /// Stored notifications to deliver to live streams after this returns.
    pub pending: Vec<Pending>,
}

/// A post as `ap_id` names it.
#[derive(Clone, Debug)]
pub struct Existing {
    /// The post.
    pub id: Uuid,
    /// Its author.
    pub author: Uuid,
    /// The post was deleted.
    pub deleted: bool,
    /// It has attached media.
    pub has_media: bool,
}

/// The stored post with `ap_id`, deleted or not.
///
/// # Errors
///
/// Database failures.
pub async fn existing_note(db: &PgPool, ap_id: &str) -> Result<Option<Existing>, AppError> {
    let row = sqlx::query(
        "select id, author_id, deleted_at is not null as deleted,
                exists (select 1 from post_media where post_id = posts.id) as has_media
           from posts where ap_id = $1",
    )
    .bind(ap_id)
    .fetch_optional(db)
    .await?;
    row.map(|row| -> Result<Existing, sqlx::Error> {
        Ok(Existing {
            id: row.try_get("id")?,
            author: row.try_get("author_id")?,
            deleted: row.try_get("deleted")?,
            has_media: row.try_get("has_media")?,
        })
    })
    .transpose()
    .map_err(Into::into)
}

/// The id of the stored, live post `r` names, for a lookup that does not lock it.
///
/// # Errors
///
/// Database failures.
pub async fn find_post(db: &PgPool, r: &PostRef) -> Result<Option<Uuid>, AppError> {
    Ok(match r {
        PostRef::Local { handle, id } => {
            sqlx::query_scalar(
                "select p.id from posts p join users u on u.id = p.author_id
                  where p.id = $1 and lower(u.handle) = lower($2) and u.domain is null
                    and p.deleted_at is null",
            )
            .bind(id)
            .bind(handle)
            .fetch_optional(db)
            .await?
        }
        PostRef::Remote(ap_id) => {
            sqlx::query_scalar("select id from posts where ap_id = $1 and deleted_at is null")
                .bind(ap_id)
                .fetch_optional(db)
                .await?
        }
    })
}

/// A post a new note links to, locked for the transaction.
struct Linked {
    id: Uuid,
    author: Uuid,
    root: Option<Uuid>,
}

async fn lock_linked(conn: &mut PgConnection, r: &PostRef) -> Result<Option<Linked>, AppError> {
    let row = match r {
        PostRef::Local { handle, id } => sqlx::query(
            "select p.id, p.author_id, p.root_id from posts p join users u on u.id = p.author_id
                  where p.id = $1 and lower(u.handle) = lower($2) and u.domain is null
                    and p.deleted_at is null for no key update of p",
        )
        .bind(id)
        .bind(handle)
        .fetch_optional(conn)
        .await?,
        PostRef::Remote(ap_id) => {
            sqlx::query(
                "select id, author_id, root_id from posts
                  where ap_id = $1 and deleted_at is null for no key update",
            )
            .bind(ap_id)
            .fetch_optional(conn)
            .await?
        }
    };
    row.map(|row| -> Result<Linked, sqlx::Error> {
        Ok(Linked {
            id: row.try_get("id")?,
            author: row.try_get("author_id")?,
            root: row.try_get("root_id")?,
        })
    })
    .transpose()
    .map_err(Into::into)
}

/// Whether remote `replier` may answer post `parent`: the parent's author did not block it and
/// the post's reply policy lets it in. The local compose path asks the same
/// ([`post_actions::may_reply`]).
async fn reply_allowed(
    conn: &mut PgConnection,
    parent: Uuid,
    replier: Uuid,
) -> Result<bool, AppError> {
    let target = post_actions::live_target(&mut *conn, parent).await?;
    if post_actions::blocked_between(conn, replier, target.author_id).await? {
        return Ok(false);
    }
    post_actions::may_reply(conn, replier, &target).await
}

/// A UUIDv7 stamped with `at`, so posts sort by when they were written and not by when this
/// server heard of them.
#[must_use]
pub fn id_at(at: OffsetDateTime) -> Uuid {
    let seconds = u64::try_from(at.unix_timestamp()).unwrap_or(0);
    Uuid::new_v7(uuid::Timestamp::from_unix(
        uuid::NoContext,
        seconds,
        at.nanosecond(),
    ))
}

/// Stores a new post for `note`: row, tags, mentions, poll, media, counters and notifications,
/// in one transaction. `None` when a post with that `ap_id` already exists, the id was deleted,
/// or when the note answers a post whose author blocked the replier or whose reply policy keeps
/// it out (nothing is stored and no counter moves).
///
/// # Errors
///
/// Database failures.
pub async fn insert_note(db: &PgPool, note: &NewNote) -> Result<Option<Created>, AppError> {
    let mut tx = db.begin().await?;
    lock_note_id(&mut tx, &note.ap_id).await?;
    if tombstoned(&mut *tx, &note.ap_id).await? {
        return Ok(None);
    }
    let reply = match &note.reply_to {
        Some(r) => lock_linked(&mut tx, r).await?,
        None => None,
    };
    if let Some(parent) = &reply
        && !reply_allowed(&mut tx, parent.id, note.author).await?
    {
        return Ok(None);
    }
    let quote = match &note.quote_of {
        Some(r) => lock_linked(&mut tx, r).await?,
        None => None,
    };
    let id = id_at(note.created_at);
    let inserted: Option<Uuid> = sqlx::query_scalar(
        "insert into posts (id, author_id, text, cw, sensitive, reply_to_id, root_id, quote_of_id,
                            visibility, ap_id, url, created_at, edited_at)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
         on conflict (ap_id) do nothing returning id",
    )
    .bind(id)
    .bind(note.author)
    .bind(&note.text)
    .bind(&note.cw)
    .bind(note.sensitive)
    .bind(reply.as_ref().map(|parent| parent.id))
    .bind(
        reply
            .as_ref()
            .map(|parent| parent.root.unwrap_or(parent.id)),
    )
    .bind(quote.as_ref().map(|quoted| quoted.id))
    .bind(note.visibility.as_str())
    .bind(&note.ap_id)
    .bind(&note.url)
    .bind(note.created_at)
    .bind(note.edited_at)
    .fetch_optional(&mut *tx)
    .await?;
    if inserted.is_none() {
        return Ok(None);
    }
    for tag in &note.tags {
        sqlx::query("insert into post_tags (post_id, tag) values ($1, $2) on conflict do nothing")
            .bind(id)
            .bind(tag)
            .execute(&mut *tx)
            .await?;
    }
    let mentioned = store_mentions(&mut tx, id, note.author, &note.local_handles).await?;
    if let Some(poll) = &note.poll {
        insert_poll(&mut tx, id, poll).await?;
    }
    media::attach(&mut tx, id, &note.media).await?;
    sqlx::query("update users set posts_count = posts_count + 1 where id = $1")
        .bind(note.author)
        .execute(&mut *tx)
        .await?;
    let mut pending = Vec::new();
    let mut notified = Vec::new();
    if let Some(parent) = &reply {
        bump(&mut tx, parent.id, "reply_count", 1).await?;
        if note.notify {
            pending.extend(
                notify(
                    &mut tx,
                    parent.author,
                    note.author,
                    NotificationKindDto::Reply,
                    Some(id),
                )
                .await?,
            );
        }
        notified.push(parent.author);
    }
    if let Some(quoted) = &quote {
        bump(&mut tx, quoted.id, "quote_count", 1).await?;
        if note.notify {
            pending.extend(
                notify(
                    &mut tx,
                    quoted.author,
                    note.author,
                    NotificationKindDto::Quote,
                    Some(id),
                )
                .await?,
            );
        }
        notified.push(quoted.author);
    }
    if note.notify {
        for user in mentioned.iter().filter(|user| !notified.contains(user)) {
            pending.extend(
                notify(
                    &mut tx,
                    *user,
                    note.author,
                    NotificationKindDto::Mention,
                    Some(id),
                )
                .await?,
            );
        }
    }
    tx.commit().await?;
    Ok(Some(Created { id, pending }))
}

/// Inserts the mention rows of the local accounts in `handles` (lower case), other than the
/// author. Returns their ids.
async fn store_mentions(
    conn: &mut PgConnection,
    post: Uuid,
    author: Uuid,
    handles: &[String],
) -> Result<Vec<Uuid>, AppError> {
    if handles.is_empty() {
        return Ok(Vec::new());
    }
    Ok(sqlx::query_scalar(
        "insert into mentions (post_id, user_id)
         select $1, id from users
          where domain is null and deleted_at is null and suspended_at is null
            and lower(handle) = any($2) and id <> $3
         on conflict do nothing returning user_id",
    )
    .bind(post)
    .bind(handles)
    .bind(author)
    .fetch_all(conn)
    .await?)
}

async fn insert_poll(conn: &mut PgConnection, post: Uuid, poll: &NewPoll) -> Result<(), AppError> {
    sqlx::query(
        "insert into polls (post_id, options, multiple, ends_at, counts, voters)
         values ($1, $2, $3, $4, $5, $6)",
    )
    .bind(post)
    .bind(&poll.options)
    .bind(poll.multiple)
    .bind(poll.ends_at)
    .bind(&poll.counts)
    .bind(poll.voters)
    .execute(conn)
    .await?;
    Ok(())
}

/// What an edit of a remote note changes.
#[derive(Clone, Debug)]
pub struct NoteEdit {
    /// The new plain text.
    pub text: String,
    /// The new content warning.
    pub cw: String,
    /// Media is sensitive.
    pub sensitive: bool,
    /// When the edit happened, not in the future.
    pub edited_at: Option<OffsetDateTime>,
    /// Lower-case hashtags without `#`.
    pub tags: Vec<String>,
    /// As [`NewNote::local_handles`].
    pub local_handles: Vec<String>,
    /// The poll as the other server counts it now.
    pub poll: Option<NewPoll>,
}

/// Applies `edit` to post `id`, written by `author`. A changed text or content warning keeps the
/// old version in the history and sets `edited_at`. Returns `false` when the post is not that
/// author's live post.
///
/// # Errors
///
/// Database failures.
pub async fn edit_note(
    db: &PgPool,
    id: Uuid,
    author: Uuid,
    edit: &NoteEdit,
) -> Result<bool, AppError> {
    let mut tx = db.begin().await?;
    let current: Option<(String, String)> = sqlx::query_as(
        "select text, cw from posts
          where id = $1 and author_id = $2 and deleted_at is null for update",
    )
    .bind(id)
    .bind(author)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((text, cw)) = current else {
        return Ok(false);
    };
    if text != edit.text || cw != edit.cw {
        sqlx::query("insert into post_edits (id, post_id, text, cw) values ($1, $2, $3, $4)")
            .bind(Uuid::now_v7())
            .bind(id)
            .bind(&text)
            .bind(&cw)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "update posts set text = $2, cw = $3, edited_at = coalesce($4, now()) where id = $1",
        )
        .bind(id)
        .bind(&edit.text)
        .bind(&edit.cw)
        .bind(edit.edited_at)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("update posts set sensitive = $2 where id = $1")
        .bind(id)
        .bind(edit.sensitive)
        .execute(&mut *tx)
        .await?;
    sqlx::query("delete from post_tags where post_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for tag in &edit.tags {
        sqlx::query("insert into post_tags (post_id, tag) values ($1, $2) on conflict do nothing")
            .bind(id)
            .bind(tag)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("delete from mentions where post_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    store_mentions(&mut tx, id, author, &edit.local_handles).await?;
    if let Some(poll) = &edit.poll {
        update_poll(&mut tx, id, poll).await?;
    }
    tx.commit().await?;
    Ok(true)
}

async fn update_poll(conn: &mut PgConnection, post: Uuid, poll: &NewPoll) -> Result<(), AppError> {
    let options: Option<Vec<String>> =
        sqlx::query_scalar("select options from polls where post_id = $1 for update")
            .bind(post)
            .fetch_optional(&mut *conn)
            .await?;
    match options {
        None => insert_poll(conn, post, poll).await,
        Some(old) => {
            if old != poll.options {
                // New answers: the votes of local accounts pointed at the old ones.
                sqlx::query("delete from poll_votes where post_id = $1")
                    .bind(post)
                    .execute(&mut *conn)
                    .await?;
            }
            sqlx::query(
                "update polls set options = $2, multiple = $3, ends_at = $4, counts = $5, voters = $6
                  where post_id = $1",
            )
            .bind(post)
            .bind(&poll.options)
            .bind(poll.multiple)
            .bind(poll.ends_at)
            .bind(&poll.counts)
            .bind(poll.voters)
            .execute(conn)
            .await?;
            Ok(())
        }
    }
}

/// Takes the lock that serialises a delete and a store of one remote note id, so a `Create`
/// that races its `Delete` either sees the tombstone or is deleted by it.
async fn lock_note_id(conn: &mut PgConnection, ap_id: &str) -> Result<(), AppError> {
    sqlx::query("select pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(ap_id)
        .execute(conn)
        .await?;
    Ok(())
}

/// Whether the remote note id `ap_id` was deleted.
///
/// # Errors
///
/// Database failures.
pub async fn tombstoned<'e>(db: impl sqlx::PgExecutor<'e>, ap_id: &str) -> Result<bool, AppError> {
    Ok(
        sqlx::query_scalar("select exists (select 1 from ap_tombstones where ap_id = $1)")
            .bind(ap_id)
            .fetch_one(db)
            .await?,
    )
}

/// Soft-deletes the post or direct message with `ap_id` when `author` wrote it, and remembers
/// the id in `ap_tombstones`, so a `Create` that arrives later (or a backfill) does not store
/// it. An id nobody stored yet is remembered too, but only when it is on `origin`, the author's
/// own instance; one that another account wrote is left alone. `false` when nothing was
/// deleted.
///
/// # Errors
///
/// Database failures.
pub async fn delete_by_ap_id(
    db: &PgPool,
    hub: &Hub,
    author: Uuid,
    origin: &str,
    ap_id: &str,
) -> Result<bool, AppError> {
    let mut tx = db.begin().await?;
    lock_note_id(&mut tx, ap_id).await?;
    let post: Option<(Uuid, Uuid, bool)> = sqlx::query_as(
        "select id, author_id, deleted_at is not null from posts where ap_id = $1
           for no key update",
    )
    .bind(ap_id)
    .fetch_optional(&mut *tx)
    .await?;
    let message_sender: Option<Uuid> =
        sqlx::query_scalar("select sender_id from messages where ap_id = $1")
            .bind(ap_id)
            .fetch_optional(&mut *tx)
            .await?;
    let owner = post.map(|(_, owner, _)| owner).or(message_sender);
    if owner.is_some_and(|owner| owner != author) {
        return Ok(false);
    }
    let mut deleted = false;
    let mut removal = live::Removal::default();
    if let Some((id, _, false)) = post {
        removal = live::removal(&mut tx, hub, id).await?;
        deleted = soft_delete(&mut tx, id).await?;
    }
    let messages = sqlx::query(
        "update messages set deleted_at = now(), text = '', ciphertext = null
          where ap_id = $1 and sender_id = $2 and deleted_at is null",
    )
    .bind(ap_id)
    .bind(author)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if owner.is_some() || tuita_core::ap::same_authority(ap_id, origin) {
        sqlx::query("insert into ap_tombstones (ap_id) values ($1) on conflict do nothing")
            .bind(ap_id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    live::publish_removal(hub, removal);
    Ok(deleted || messages > 0)
}

/// A stored direct message.
#[derive(Clone, Copy, Debug)]
pub struct StoredMessage {
    /// The message.
    pub id: Uuid,
    /// When it was written.
    pub created_at: OffsetDateTime,
}

/// Stores a message from remote account `sender` to local account `recipient`. `None` when a
/// message with that `ap_id` exists or the id was deleted before (see [`delete_by_ap_id`]).
///
/// # Errors
///
/// Database failures.
pub async fn insert_message(
    db: &PgPool,
    sender: Uuid,
    recipient: Uuid,
    ap_id: &str,
    text: &str,
    created_at: OffsetDateTime,
) -> Result<Option<StoredMessage>, AppError> {
    let id = id_at(created_at);
    let row: Option<OffsetDateTime> = sqlx::query_scalar(
        "insert into messages (id, sender_id, recipient_id, text, ap_id, created_at)
         select $1::uuid, $2::uuid, $3::uuid, $4::text, $5::text, $6::timestamptz
          where not exists (select 1 from ap_tombstones where ap_id = $5)
         on conflict (ap_id) do nothing returning created_at",
    )
    .bind(id)
    .bind(sender)
    .bind(recipient)
    .bind(text)
    .bind(ap_id)
    .bind(created_at)
    .fetch_optional(db)
    .await?;
    Ok(row.map(|created_at| StoredMessage { id, created_at }))
}

/// The local account with handle `handle`, if it can receive.
///
/// # Errors
///
/// Database failures.
pub async fn local_recipient(db: &PgPool, handle: &str) -> Result<Option<UserRow>, AppError> {
    let sql = format!(
        "select {} from users u where lower(u.handle) = lower($1) and u.domain is null
            and u.deleted_at is null and u.suspended_at is null",
        super::users::COLS
    );
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(handle)
        .fetch_optional(db)
        .await?)
}

/// The followers collection of remote account `user`, when it published one.
///
/// # Errors
///
/// Database failures.
pub async fn followers_url(db: &PgPool, user: Uuid) -> Result<Option<String>, AppError> {
    Ok(
        sqlx::query_scalar("select followers_url from users where id = $1")
            .bind(user)
            .fetch_optional(db)
            .await?
            .flatten(),
    )
}

/// The public key of remote account `user`, PEM, for checking its signatures.
///
/// # Errors
///
/// Database failures.
pub async fn public_key_pem(db: &PgPool, user: Uuid) -> Result<Option<String>, AppError> {
    Ok(
        sqlx::query_scalar("select public_key_pem from users where id = $1 and domain is not null")
            .bind(user)
            .fetch_optional(db)
            .await?
            .flatten(),
    )
}

/// A stored, live post that `url` names: its id, or its page when the author's instance (`domain`,
/// the authority of `url`) is the one that serves the page. A post cannot claim another
/// instance's page address.
///
/// # Errors
///
/// Database failures.
pub async fn find_known_post(
    db: &PgPool,
    url: &str,
    domain: &str,
) -> Result<Option<Uuid>, AppError> {
    Ok(sqlx::query_scalar(
        "select p.id from posts p join users a on a.id = p.author_id
          where (p.ap_id = $1 or (p.url = $1 and a.domain = $2)) and p.deleted_at is null
          order by (p.ap_id = $1) desc limit 1",
    )
    .bind(url)
    .bind(domain)
    .fetch_optional(db)
    .await?)
}
