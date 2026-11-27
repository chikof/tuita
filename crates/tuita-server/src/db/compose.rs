//! Writing new content: creating posts and threads, editing within the 30-minute window, and
//! hiding replies.
//!
//! Counters change in the transaction of the row they count. Notifications are stored inside
//! the transaction; [`Effects`] carry them, and the unfurl jobs, to be delivered after the
//! commit.

use sqlx::{FromRow, PgConnection};
use time::{Duration, OffsetDateTime};
use tuita_core::text::{MAX_POST_LEN, RichText, parse_post};
use uuid::Uuid;

use super::live;
use super::media;
use super::notifications::{self, Pending, notify};
use super::polls::{self, ValidPoll};
use super::post_actions::{Target, bump, interaction_target, lock_live, may_reply, visible_target};
use super::settings;
use super::users::UserRow;
use crate::dto::{CreatePostRequest, EditPostRequest, MediaKind, NotificationKindDto, ReplyPolicy};
use crate::error::AppError;
use crate::federation::outbound;
use crate::filter::{ContentFilter, REASON};
use crate::state::AppState;
use crate::workers::unfurl::Job;

/// How long after posting the text may be edited.
pub const EDIT_WINDOW: Duration = Duration::minutes(30);
/// Most posts in one thread.
pub const MAX_THREAD: usize = 25;
/// Longest link that is previewed.
const MAX_CARD_URL: usize = 2048;

const NOT_FOUND: &str = "Post not found.";

/// What has to happen once the transaction that created posts has committed.
#[derive(Debug, Default)]
pub struct Effects {
    pending: Vec<Pending>,
    unfurl: Vec<Job>,
    created: Vec<Uuid>,
}

impl Effects {
    /// Sends the stored notifications and the new posts to live streams and queues the link
    /// previews.
    pub async fn deliver(self, state: &AppState) {
        notifications::deliver(state, self.pending).await;
        live::posts_created(state, &self.created).await;
        for job in self.unfurl {
            state.queue_unfurl(job);
        }
    }

    /// Adds what `other` collected.
    pub fn absorb(&mut self, other: Self) {
        self.pending.extend(other.pending);
        self.unfurl.extend(other.unfurl);
        self.created.extend(other.created);
    }
}

/// A post request after the checks that need no database.
struct Draft<'a> {
    text: &'a str,
    cw: &'a str,
    poll: Option<ValidPoll>,
}

/// Runs the checks of [`create`] that need no database: lengths, poll, empty post.
///
/// # Errors
///
/// `400` with the message of the failed rule.
pub fn validate(req: &CreatePostRequest) -> Result<(), AppError> {
    check(req).map(|_| ())
}

/// Rejects posts the instance's word filter blocks. Does nothing until the admin adds words.
///
/// # Errors
///
/// `400 Your post contains blocked words, which isn’t allowed on this instance.`
pub fn check_filter(filter: &ContentFilter, reqs: &[CreatePostRequest]) -> Result<(), AppError> {
    for req in reqs {
        let options = req
            .poll
            .as_ref()
            .map(|poll| poll.options.join(" "))
            .unwrap_or_default();
        let text = format!("{}\n{}\n{options}", req.text, req.cw);
        if filter.blocks_text(&text) {
            return Err(AppError::bad_request(blocked_post()));
        }
    }
    Ok(())
}

fn blocked_post() -> String {
    format!("Your post contains {REASON}, which isn\u{2019}t allowed on this instance.")
}

fn check(req: &CreatePostRequest) -> Result<Draft<'_>, AppError> {
    let text = req.text.trim();
    let cw = req.cw.trim();
    check_text(text, cw)?;
    let poll = req.poll.as_ref().map(polls::validate).transpose()?;
    if text.is_empty() {
        if poll.is_some() {
            return Err(AppError::bad_request("Ask a question for your poll."));
        }
        if req.media_ids.is_empty() {
            return Err(AppError::bad_request("Write something first."));
        }
    }
    Ok(Draft { text, cw, poll })
}

/// Rejects a poll together with media.
///
/// # Errors
///
/// `400` with the message that names what was combined.
pub fn check_extras(has_poll: bool, claimed: &[(Uuid, MediaKind)]) -> Result<(), AppError> {
    if has_poll && !claimed.is_empty() {
        return Err(AppError::bad_request(
            if claimed.iter().any(|(_, kind)| *kind == MediaKind::Video) {
                "A post can have a video, images or a poll, but only one of them."
            } else {
                "A post can have images or a poll, not both."
            },
        ));
    }
    Ok(())
}

fn check_text(text: &str, cw: &str) -> Result<(), AppError> {
    if parse_post(text).len() > MAX_POST_LEN {
        return Err(AppError::bad_request(format!(
            "Post must be at most {MAX_POST_LEN} characters."
        )));
    }
    if cw.chars().count() > 100 {
        return Err(AppError::bad_request(
            "Content warning must be at most 100 characters.",
        ));
    }
    Ok(())
}

/// The post a new one answers: a stored target, or the previous post of a thread.
struct Parent {
    id: Uuid,
    author_id: Uuid,
    root_id: Uuid,
}

impl From<&Target> for Parent {
    fn from(target: &Target) -> Self {
        Self {
            id: target.id,
            author_id: target.author_id,
            root_id: target.root_id.unwrap_or(target.id),
        }
    }
}

/// Creates one post and returns its id. The caller loads it for the response.
///
/// # Errors
///
/// `400` for invalid text, content warning, poll, media or an empty post; `404 Post not found.`
/// for a missing reply or quote target; `403` when blocked or the reply policy forbids it;
/// database failures.
pub async fn create(
    state: &AppState,
    author: &UserRow,
    client: Option<&'static str>,
    req: CreatePostRequest,
) -> Result<Uuid, AppError> {
    let mut ids = create_chain(state, author, client, vec![req]).await?;
    ids.pop().ok_or(AppError::Missing(NOT_FOUND))
}

/// Creates a thread: the first post may reply to or quote one, each later post replies to the
/// one before. All or nothing.
///
/// # Errors
///
/// `400 A thread needs 2 to 25 posts.`, `400 Only the first post of a thread can reply to or
/// quote a post.`, and everything [`create`] returns.
pub async fn create_thread(
    state: &AppState,
    author: &UserRow,
    client: Option<&'static str>,
    reqs: Vec<CreatePostRequest>,
) -> Result<Vec<Uuid>, AppError> {
    if !(2..=MAX_THREAD).contains(&reqs.len()) {
        return Err(AppError::bad_request("A thread needs 2 to 25 posts."));
    }
    if reqs
        .iter()
        .skip(1)
        .any(|req| req.reply_to.is_some() || req.quote_of.is_some())
    {
        return Err(AppError::bad_request(
            "Only the first post of a thread can reply to or quote a post.",
        ));
    }
    create_chain(state, author, client, reqs).await
}

async fn create_chain(
    state: &AppState,
    author: &UserRow,
    client: Option<&'static str>,
    reqs: Vec<CreatePostRequest>,
) -> Result<Vec<Uuid>, AppError> {
    let filter = settings::content_filter(state.db()).await?;
    let mut tx = state.db().begin().await?;
    let (ids, effects) = create_in_tx(state, &mut tx, author, client, reqs, &filter).await?;
    tx.commit().await?;
    effects.deliver(state).await;
    Ok(ids)
}

/// The body of [`create`] and [`create_thread`] inside the caller's transaction. The caller
/// commits, then delivers the [`Effects`]. The scheduler uses it to publish in the same
/// transaction that marks the schedule row done.
///
/// # Errors
///
/// As [`create`]. After an error the transaction is unusable.
pub async fn create_in_tx(
    state: &AppState,
    tx: &mut PgConnection,
    author: &UserRow,
    client: Option<&'static str>,
    reqs: Vec<CreatePostRequest>,
    filter: &ContentFilter,
) -> Result<(Vec<Uuid>, Effects), AppError> {
    let drafts = reqs.iter().map(check).collect::<Result<Vec<_>, _>>()?;
    check_filter(filter, &reqs)?;
    let Some(first) = reqs.first() else {
        return Err(AppError::bad_request("Write something first."));
    };
    let reply_to = match first.reply_to {
        Some(id) => Some(interaction_target(tx, author.id, id).await?),
        None => None,
    };
    let quote_of = match first.quote_of {
        Some(id) => Some(interaction_target(tx, author.id, id).await?),
        None => None,
    };
    if let Some(parent) = &reply_to
        && !may_reply(tx, author.id, parent).await?
    {
        return Err(AppError::forbidden(
            "The author limited who can reply to this post.",
        ));
    }
    for target in reply_to.iter().chain(quote_of.iter()) {
        lock_live(tx, target.id).await?;
    }

    let client = client.filter(|_| author.show_client.unwrap_or(true));
    let mut parent: Option<Parent> = reply_to.as_ref().map(Parent::from);
    let mut ids = Vec::with_capacity(reqs.len());
    let mut effects = Effects::default();
    for (index, (req, draft)) in reqs.iter().zip(&drafts).enumerate() {
        let quote = if index == 0 { quote_of.as_ref() } else { None };
        let (id, post_effects) = insert_post(
            state,
            tx,
            author,
            client,
            req,
            draft,
            parent.as_ref(),
            quote,
        )
        .await?;
        effects.absorb(post_effects);
        parent = Some(Parent {
            id,
            author_id: author.id,
            root_id: parent.as_ref().map_or(id, |parent| parent.root_id),
        });
        ids.push(id);
    }
    Ok((ids, effects))
}

#[allow(
    clippy::too_many_arguments,
    reason = "one call site; a struct would only rename them"
)]
async fn insert_post(
    state: &AppState,
    tx: &mut PgConnection,
    author: &UserRow,
    client: Option<&'static str>,
    req: &CreatePostRequest,
    draft: &Draft<'_>,
    parent: Option<&Parent>,
    quote: Option<&Target>,
) -> Result<(Uuid, Effects), AppError> {
    let claimed = media::claim(tx, author.id, &req.media_ids).await?;
    check_extras(draft.poll.is_some(), &claimed)?;
    let rich = parse_post(draft.text);
    let card_url = rich
        .links()
        .first()
        .filter(|url| url.len() <= MAX_CARD_URL)
        .map(|url| (*url).to_owned());
    let id = Uuid::now_v7();
    let policy = req.reply_policy.unwrap_or(ReplyPolicy::Everyone);
    sqlx::query(
        "insert into posts (id, author_id, text, cw, sensitive, reply_to_id, root_id,
            quote_of_id, reply_policy, client, card_url)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(id)
    .bind(author.id)
    .bind(draft.text)
    .bind(draft.cw)
    .bind(req.sensitive)
    .bind(parent.map(|parent| parent.id))
    .bind(parent.map(|parent| parent.root_id))
    .bind(quote.map(|quoted| quoted.id))
    .bind(policy.as_str())
    .bind(client)
    .bind(card_url.as_deref())
    .execute(&mut *tx)
    .await?;
    let mentioned = store_tags_and_mentions(state, tx, id, author.id, &rich).await?;
    let ids: Vec<Uuid> = claimed.iter().map(|(id, _)| *id).collect();
    media::attach(tx, id, &ids).await?;
    if let Some(poll) = &draft.poll {
        polls::insert(tx, id, poll).await?;
    }

    sqlx::query("update users set posts_count = posts_count + 1 where id = $1")
        .bind(author.id)
        .execute(&mut *tx)
        .await?;
    let mut effects = Effects {
        created: vec![id],
        ..Effects::default()
    };
    let mut notified = Vec::new();
    if let Some(parent) = parent {
        bump(tx, parent.id, "reply_count", 1).await?;
        effects.pending.extend(
            notify(
                tx,
                parent.author_id,
                author.id,
                NotificationKindDto::Reply,
                Some(id),
            )
            .await?,
        );
        notified.push(parent.author_id);
    }
    if let Some(quoted) = quote {
        bump(tx, quoted.id, "quote_count", 1).await?;
        effects.pending.extend(
            notify(
                tx,
                quoted.author_id,
                author.id,
                NotificationKindDto::Quote,
                Some(id),
            )
            .await?,
        );
        notified.push(quoted.author_id);
    }
    for user in mentioned.iter().filter(|user| !notified.contains(user)) {
        effects
            .pending
            .extend(notify(tx, *user, author.id, NotificationKindDto::Mention, Some(id)).await?);
    }
    if let Some(url) = card_url {
        effects.unfurl.push(Job {
            post_id: id,
            author_id: author.id,
            url,
        });
    }
    Box::pin(outbound::post_created(tx, state.config(), id)).await?;
    Ok((id, effects))
}

/// Replaces the hashtag and mention rows of `post` with those of `rich`. Returns the local
/// accounts mentioned, other than the author.
async fn store_tags_and_mentions(
    state: &AppState,
    tx: &mut PgConnection,
    post: Uuid,
    author: Uuid,
    rich: &RichText,
) -> Result<Vec<Uuid>, AppError> {
    sqlx::query("delete from post_tags where post_id = $1")
        .bind(post)
        .execute(&mut *tx)
        .await?;
    sqlx::query("delete from mentions where post_id = $1")
        .bind(post)
        .execute(&mut *tx)
        .await?;
    for tag in rich.hashtags() {
        sqlx::query("insert into post_tags (post_id, tag) values ($1, $2) on conflict do nothing")
            .bind(post)
            .bind(tag)
            .execute(&mut *tx)
            .await?;
    }
    let mut mentioned = Vec::new();
    for (handle, domain) in rich.mentions() {
        if domain.is_some_and(|domain| !domain.eq_ignore_ascii_case(state.config().domain())) {
            continue;
        }
        let user: Option<Uuid> = sqlx::query_scalar(
            "select id from users where lower(handle) = lower($1) and domain is null
             and suspended_at is null and deleted_at is null",
        )
        .bind(handle)
        .fetch_optional(&mut *tx)
        .await?;
        if let Some(user) = user.filter(|user| *user != author && !mentioned.contains(user)) {
            sqlx::query("insert into mentions (post_id, user_id) values ($1, $2)")
                .bind(post)
                .bind(user)
                .execute(&mut *tx)
                .await?;
            mentioned.push(user);
        }
    }
    Ok(mentioned)
}

#[derive(FromRow)]
struct EditTarget {
    author_id: Uuid,
    text: String,
    cw: String,
    created_at: OffsetDateTime,
    card_url: Option<String>,
    has_extras: bool,
}

/// Edits the text and content warning of one of the viewer's own posts within 30 minutes of
/// posting, keeping the earlier version in the history. Nothing is stored when nothing changed.
///
/// # Errors
///
/// `404 Post not found.`; `403 You can only edit your own posts.`; `403 Posts can only be edited
/// for 30 minutes after posting.`; `400` for invalid text or a post left empty; database
/// failures.
pub async fn edit(
    state: &AppState,
    viewer: &UserRow,
    id: Uuid,
    req: EditPostRequest,
) -> Result<(), AppError> {
    let filter = settings::content_filter(state.db()).await?;
    let mut tx = state.db().begin().await?;
    let target: EditTarget = sqlx::query_as(
        "select author_id, text, cw, created_at, card_url,
                (exists (select 1 from post_media where post_id = posts.id)
                 or exists (select 1 from polls where post_id = posts.id)) as has_extras
           from posts where id = $1 and deleted_at is null for update",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::Missing(NOT_FOUND))?;
    if target.author_id != viewer.id {
        return Err(AppError::forbidden("You can only edit your own posts."));
    }
    if OffsetDateTime::now_utc() - target.created_at > EDIT_WINDOW {
        return Err(AppError::forbidden(
            "Posts can only be edited for 30 minutes after posting.",
        ));
    }
    let text = req.text.as_deref().map_or(target.text.as_str(), str::trim);
    let cw = req.cw.as_deref().map_or(target.cw.as_str(), str::trim);
    check_text(text, cw)?;
    if text.is_empty() && !target.has_extras {
        return Err(AppError::bad_request("A post can’t be empty."));
    }
    if text == target.text && cw == target.cw {
        return Ok(());
    }
    if filter.blocks_text(&format!("{text}\n{cw}")) {
        return Err(AppError::bad_request(blocked_post()));
    }
    sqlx::query("insert into post_edits (id, post_id, text, cw) values ($1, $2, $3, $4)")
        .bind(Uuid::now_v7())
        .bind(id)
        .bind(&target.text)
        .bind(&target.cw)
        .execute(&mut *tx)
        .await?;
    let rich = parse_post(text);
    let card_url = rich
        .links()
        .first()
        .filter(|url| url.len() <= MAX_CARD_URL)
        .map(|url| (*url).to_owned());
    sqlx::query(
        "update posts set text = $2, cw = $3, edited_at = now(), card_url = $4 where id = $1",
    )
    .bind(id)
    .bind(text)
    .bind(cw)
    .bind(card_url.as_deref())
    .execute(&mut *tx)
    .await?;
    store_tags_and_mentions(state, &mut tx, id, viewer.id, &rich).await?;
    outbound::post_updated(&mut tx, state.config(), id).await?;
    tx.commit().await?;
    if let Some(url) = card_url.filter(|url| target.card_url.as_ref() != Some(url)) {
        state.queue_unfurl(Job {
            post_id: id,
            author_id: viewer.id,
            url,
        });
    }
    Ok(())
}

/// Hides a reply to the viewer's post from its thread, or shows it again.
///
/// # Errors
///
/// `404 Post not found.`; `403 You can only hide replies to your own posts.`; `400 You can’t hide
/// your own reply.`; database failures.
pub async fn set_hidden(
    db: &sqlx::PgPool,
    viewer: Uuid,
    id: Uuid,
    hidden: bool,
) -> Result<(), AppError> {
    let target = visible_target(db, viewer, id).await?;
    let parent_author: Option<Uuid> = sqlx::query_scalar(
        "select parent.author_id from posts p join posts parent on parent.id = p.reply_to_id
          where p.id = $1",
    )
    .bind(id)
    .fetch_optional(db)
    .await?;
    if parent_author != Some(viewer) {
        return Err(AppError::forbidden(
            "You can only hide replies to your own posts.",
        ));
    }
    if target.author_id == viewer {
        return Err(AppError::bad_request("You can’t hide your own reply."));
    }
    sqlx::query("update posts set hidden_reply = $2 where id = $1")
        .bind(id)
        .bind(hidden)
        .execute(db)
        .await?;
    Ok(())
}

#[derive(FromRow)]
struct EditRow {
    text: String,
    cw: String,
    edited_at: OffsetDateTime,
}

/// The current text of a post and its earlier versions, newest first. A version is timed from
/// when the edit that replaced it came in, so each version runs from the previous edit to the next.
///
/// # Errors
///
/// `404 Post not found.`; database failures.
pub async fn edits(db: &sqlx::PgPool, id: Uuid) -> Result<crate::dto::PostEditsDto, AppError> {
    use crate::dto::{EditVersionDto, PostEditsDto};
    let (text, cw, edited_at, created_at): (
        String,
        String,
        Option<OffsetDateTime>,
        OffsetDateTime,
    ) = sqlx::query_as(
        "select text, cw, edited_at, created_at from posts where id = $1 and deleted_at is null",
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::Missing(NOT_FOUND))?;
    let rows: Vec<EditRow> = sqlx::query_as(
        "select text, cw, edited_at from post_edits where post_id = $1 order by id desc",
    )
    .bind(id)
    .fetch_all(db)
    .await?;
    let edits = rows
        .iter()
        .enumerate()
        .map(|(index, row)| EditVersionDto {
            text: row.text.clone(),
            cw: row.cw.clone(),
            at: rows
                .get(index + 1)
                .map_or(created_at, |older| older.edited_at),
        })
        .collect();
    Ok(PostEditsDto {
        current: EditVersionDto {
            text,
            cw,
            at: edited_at.unwrap_or(created_at),
        },
        edits,
    })
}
