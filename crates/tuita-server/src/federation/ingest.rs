//! Storing what other servers send: notes, edits, deletions, profile updates.
//!
//! This is a library for the lookup code and the inbox pipeline. Every entry point treats its
//! input as hostile: ids are bound to the author's host, URLs must be http(s), lengths are
//! capped, and nothing a remote claims can name a local object as its own.
//!
//! A note maps to a `posts` row (visibility from `to`/`cc`, text from sanitised HTML), or, when
//! it is a direct note to exactly one local account, to a `messages` row.

use std::time::Duration;

use futures_util::future::join_all;
use time::OffsetDateTime;
use tokio::sync::Semaphore;
use tuita_core::ap::{self, Actor, Note, NoteKind, Tag};
use tuita_core::emoji::shortcodes;
use uuid::Uuid;

use super::media::download;
use super::{FedError, ids, lookup};
use crate::db::emoji;
use crate::db::messages::blocked_reason;
use crate::db::notifications::{self, publish_badges};
use crate::db::remote::{
    self, Audience, NewNote, NewPoll, NoteEdit, PostRef, delay_refresh, insert_media,
};
use crate::db::users::{UserRow, user_ref};
use crate::dto::{MessageEventDto, StreamEvent, Visibility};
use crate::error::AppError;
use crate::media::{Target, prepare_image};
use crate::state::AppState;

/// Longest post text kept, in characters. Remote posts may exceed the local 500; they are cut.
pub const MAX_TEXT_CHARS: usize = 5000;
/// Longest content warning kept, in characters.
const MAX_CW_CHARS: usize = 500;
/// Longest direct message kept, in characters: the local limit.
const MAX_MESSAGE_CHARS: usize = 2000;
/// Longest hashtag kept, in characters.
const MAX_TAG_CHARS: usize = 64;
/// Most hashtags kept.
const MAX_TAGS: usize = 20;
/// Most addressees and mentions looked at.
const MAX_ADDRESSEES: usize = 64;
/// Most poll answers kept.
const MAX_POLL_OPTIONS: usize = 10;
/// Longest poll answer kept, in characters.
const MAX_OPTION_CHARS: usize = 100;
/// Most `Emoji` tags of one object that are looked at.
const MAX_EMOJI_TAGS: usize = 16;
/// Largest emote image copied, in bytes.
const MAX_EMOTE_BYTES: usize = 256 * 1024;
/// Longest one emote download may take.
const EMOTE_TIMEOUT: Duration = Duration::from_secs(10);
/// Most remote emotes downloaded and decoded at the same time, across the whole process.
const MAX_CONCURRENT_EMOTES: usize = 4;
static EMOTE_COPIES: Semaphore = Semaphore::const_new(MAX_CONCURRENT_EMOTES);
/// Characters of a message that go into the live `message` event.
const PREVIEW_CHARS: usize = 280;

/// Why a note is being stored; it decides what is kept and who is told.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Context {
    /// Read from an account's outbox when the account was first looked up: public posts only,
    /// no notifications.
    Backfill,
    /// A user asked for this post by URL: always stored, whoever it is by and whatever it says.
    Lookup,
    /// Delivered to an inbox: stored only when [`should_store`] says so; notifies local users.
    Inbox,
}

/// What [`store_note`] did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stored {
    /// The note is a post. `created` is false when the post was already stored (its text may
    /// have been updated).
    Post {
        /// The stored post.
        id: Uuid,
        /// This call created it.
        created: bool,
    },
    /// The note is a direct message to a local account. `created` is false for a replay.
    Message {
        /// This call created it.
        created: bool,
    },
    /// Nothing was stored: not addressed to anyone here, deleted before, blocked, or empty.
    Dropped,
}

pub(super) fn internal(err: AppError) -> FedError {
    FedError::Internal(err.into())
}

/// `text` cut to at most `max` characters, at a character boundary.
pub(super) fn cap(text: &str, max: usize) -> String {
    match text.char_indices().nth(max) {
        Some((end, _)) => text.get(..end).unwrap_or(text).to_owned(),
        None => text.to_owned(),
    }
}

fn is_public(entry: &str) -> bool {
    entry == ap::PUBLIC || entry == "as:Public" || entry == "Public"
}

/// Everything a note is addressed to, bounded.
fn addressees(note: &Note) -> Vec<&str> {
    note.to
        .iter()
        .chain(note.cc.iter())
        .map(String::as_str)
        .take(MAX_ADDRESSEES * 2)
        .collect()
}

/// Who may see `note`: `Public` in `to`, unlisted for `Public` in `cc`, followers when the
/// author's followers collection is addressed, otherwise only the people named.
fn visibility_of(note: &Note, author: &str, followers_url: Option<&str>) -> Visibility {
    if note.to.iter().any(|entry| is_public(entry)) {
        return Visibility::Public;
    }
    if note.cc.iter().any(|entry| is_public(entry)) {
        return Visibility::Unlisted;
    }
    let default_followers = format!("{author}/followers");
    let to_followers = addressees(note)
        .into_iter()
        .any(|entry| Some(entry) == followers_url || entry == default_followers);
    if to_followers {
        Visibility::Followers
    } else {
        Visibility::Direct
    }
}

/// Lower-case handles of the local accounts `note` mentions with a `Mention` tag.
fn mentioned_handles(state: &AppState, note: &Note) -> Vec<String> {
    let mut handles: Vec<String> = Vec::new();
    for tag in note.tag.iter().take(MAX_ADDRESSEES) {
        if let Tag::Mention { href, .. } = tag
            && let Some(handle) = ids::local_handle_from_actor_id(state.config(), href)
        {
            let handle = handle.to_lowercase();
            if !handles.contains(&handle) {
                handles.push(handle);
            }
        }
    }
    handles
}

/// Lower-case handles of the local accounts `note` is addressed to.
fn addressed_handles(state: &AppState, note: &Note) -> Vec<String> {
    let mut handles: Vec<String> = Vec::new();
    for entry in addressees(note) {
        if let Some(handle) = ids::local_handle_from_actor_id(state.config(), entry) {
            let handle = handle.to_lowercase();
            if !handles.contains(&handle) {
                handles.push(handle);
            }
        }
    }
    handles
}

fn post_ref(state: &AppState, id: &str) -> Option<PostRef> {
    if !ap::is_http_url(id) {
        return None;
    }
    if let Some((handle, post)) = ids::local_post_from_note_id(state.config(), id) {
        return Some(PostRef::Local {
            handle: handle.to_owned(),
            id: post,
        });
    }
    if ap::url_authority(id).as_deref() == Some(state.config().domain()) {
        return None;
    }
    Some(PostRef::Remote(id.to_owned()))
}

fn clamp_to_now(at: &str, now: OffsetDateTime) -> Option<OffsetDateTime> {
    ap::parse_time(at).map(|at| at.min(now))
}

/// The hashtags of `note`: lower case, without `#`, letters, digits and underscores.
fn hashtags(note: &Note) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();
    for tag in note.tag.iter().take(MAX_ADDRESSEES) {
        let Tag::Hashtag { name, .. } = tag else {
            continue;
        };
        let name = name.trim().trim_start_matches('#').to_lowercase();
        let plain = !name.is_empty()
            && name.chars().count() <= MAX_TAG_CHARS
            && name.chars().all(|c| c.is_alphanumeric() || c == '_');
        if plain && !tags.contains(&name) && tags.len() < MAX_TAGS {
            tags.push(name);
        }
    }
    tags
}

fn count_of(votes: Option<u64>) -> i32 {
    votes.map_or(0, |n| i32::try_from(n).unwrap_or(i32::MAX))
}

/// The poll of a `Question`, as the other server counts it. `None` for anything else, or a poll
/// with fewer than two answers.
fn poll_of(note: &Note, now: OffsetDateTime) -> Option<NewPoll> {
    if note.kind != NoteKind::Question {
        return None;
    }
    let mut options = Vec::new();
    let mut counts = Vec::new();
    for option in note.poll_options().iter().take(MAX_POLL_OPTIONS) {
        let name = cap(
            ap::strip_controls(option.name.trim()).trim(),
            MAX_OPTION_CHARS,
        );
        if name.is_empty() {
            continue;
        }
        options.push(name);
        counts.push(count_of(
            option.replies.as_ref().and_then(|r| r.total_items),
        ));
    }
    if options.len() < 2 {
        return None;
    }
    let total = counts.iter().fold(0_i32, |sum, n| sum.saturating_add(*n));
    let closed = note.closed.as_deref().and_then(|at| clamp_to_now(at, now));
    let ends_at = note
        .end_time
        .as_deref()
        .and_then(ap::parse_time)
        .or(closed)
        .unwrap_or(now);
    Some(NewPoll {
        options,
        multiple: note.poll_is_multiple(),
        ends_at: closed.map_or(ends_at, |closed| closed.min(ends_at)),
        counts,
        voters: note.voters_count.map_or(total, |n| count_of(Some(n))),
    })
}

fn plain_text(html: Option<&str>, max: usize) -> String {
    html.map(|html| cap(ap::html_to_text(html).trim(), max))
        .unwrap_or_default()
}

/// Copies the emotes that `texts` of a remote author name and `tags` describe, so readers never
/// contact the author's server: each valid `Emoji` tag whose `:shortcode:` appears in a text is
/// fetched through [`crate::net::SafeClient`] (at most [`MAX_EMOJI_TAGS`] tags are looked at, 256
/// KiB each), re-encoded as an emote and stored for the author's `domain`. A shortcode already
/// copied for that domain is not fetched again. Any failure leaves the text as plain `:name:`.
pub(super) async fn copy_emotes(
    state: &AppState,
    owner: Uuid,
    domain: Option<&str>,
    tags: &[Tag],
    texts: &[&str],
) {
    let Some(domain) = domain else {
        return;
    };
    let mentioned: Vec<&str> = texts.iter().flat_map(|text| shortcodes(text)).collect();
    let mut wanted: Vec<(&str, &str)> = Vec::new();
    let emoji_tags = tags.iter().filter_map(|tag| match tag {
        Tag::Emoji { name, icon, .. } => Some((name.as_str(), icon.as_str())),
        _ => None,
    });
    for (name, icon) in emoji_tags.take(MAX_EMOJI_TAGS) {
        let named = mentioned.iter().any(|code| code.eq_ignore_ascii_case(name));
        let seen = wanted
            .iter()
            .any(|(seen, _)| seen.eq_ignore_ascii_case(name));
        if named && !seen {
            wanted.push((name, icon));
        }
    }
    if wanted.is_empty() {
        return;
    }
    let mut stale: Vec<(&str, &str)> = Vec::new();
    for (name, icon) in wanted {
        match emoji::remote_source(state.db(), domain, name).await {
            Ok(Some(source)) if source == icon => {}
            Ok(_) => stale.push((name, icon)),
            Err(err) => {
                tracing::warn!(error = %err, "cannot look up the copied emotes of a remote author");
                return;
            }
        }
    }
    let copies = stale
        .into_iter()
        .map(|(name, icon)| Box::pin(copy_emote(state, owner, domain, name, icon)));
    join_all(copies).await;
}

async fn copy_emote(state: &AppState, owner: Uuid, domain: &str, name: &str, url: &str) {
    let Ok(_permit) = EMOTE_COPIES.acquire().await else {
        return;
    };
    let Some(bytes) = Box::pin(download(
        state,
        url,
        "image/*",
        MAX_EMOTE_BYTES,
        EMOTE_TIMEOUT,
    ))
    .await
    else {
        return;
    };
    let mut media = match prepare_image(state, bytes, Target::Emote).await {
        Ok(media) => media,
        Err(err) => {
            tracing::debug!(url, error = %err, "remote emote is not usable");
            return;
        }
    };
    if let Err(err) = insert_media(state.db(), owner, &format!(":{name}:"), &media).await {
        tracing::warn!(error = %err, "cannot store a copied emote");
        return;
    }
    media.guard.keep();
    if let Err(err) = emoji::upsert_remote(state.db(), domain, name, media.id, url).await {
        tracing::warn!(error = %err, "cannot record a copied emote");
    }
}

/// `text` without the `@recipient` that starts it. Other servers need that mention to treat a
/// note as a direct message (tuita's own outbound notes add it), but a conversation already
/// says who it is with. A message that is only the mention keeps it.
fn without_leading_mention<'a>(text: &'a str, recipient: &UserRow, domain: &str) -> &'a str {
    let Some(rest) = text
        .strip_prefix('@')
        .and_then(|rest| rest.strip_prefix(recipient.handle.as_str()))
    else {
        return text;
    };
    let rest = rest
        .strip_prefix('@')
        .and_then(|rest| rest.strip_prefix(domain))
        .unwrap_or(rest);
    let stripped = rest.trim_start();
    if stripped.is_empty() || stripped.len() == rest.len() {
        text
    } else {
        stripped
    }
}

/// The author, the note id and the visibility checks every entry point runs first.
fn check_note(state: &AppState, author: &UserRow, note: &Note) -> Result<(), FedError> {
    let origin = author.ap_id.as_deref().ok_or(FedError::NotFound)?;
    let server = || ap::url_authority(origin).unwrap_or_default().into();
    let bound = ap::is_http_url(&note.id)
        && ap::same_authority(&note.id, origin)
        && note.attributed_to.as_deref() == Some(origin)
        && ap::url_authority(&note.id).as_deref() != Some(state.config().domain())
        && ids::local_post_from_note_id(state.config(), &note.id).is_none();
    if bound {
        Ok(())
    } else {
        Err(FedError::Unreadable(server()))
    }
}

/// Whether an `Inbox` note is worth storing: it is addressed to or mentions a local account, is
/// by an account a local user follows, or answers a local post. Anything else (a stranger's
/// public post that reached us through a relay or a shared inbox, or one that names somebody
/// else's followers collection) is dropped.
///
/// # Errors
///
/// [`FedError::Internal`] on database failures.
pub async fn should_store(
    state: &AppState,
    author: &UserRow,
    note: &Note,
) -> Result<bool, FedError> {
    let mut local_handles = mentioned_handles(state, note);
    local_handles.extend(addressed_handles(state, note));
    let reply_to = note
        .in_reply_to
        .as_deref()
        .and_then(|id| ids::local_post_from_note_id(state.config(), id));
    remote::addressed_to_us(
        state.db(),
        &Audience {
            local_handles: &local_handles,
            author: author.id,
            reply_to,
        },
    )
    .await
    .map_err(internal)
}

/// Stores `note`, written by remote account `author`, as a post or a direct message.
///
/// Storing the same `ap_id` again updates the post and never duplicates it. `Inbox` notes are
/// stored only when [`should_store`] allows. `Backfill` keeps public and unlisted posts only.
/// `Lookup` stores whatever it is given, and fetches the post it replies to once, when that is
/// not stored yet.
///
/// Notifications (mention, reply, quote) go to local accounts for `Inbox` notes only.
///
/// # Errors
///
/// [`FedError::Unreadable`] when the note's id is not on the author's host, it names another
/// author, or it claims a local id; [`FedError::Internal`] on database failures. Failed
/// attachments are skipped, never an error.
pub async fn store_note(
    state: &AppState,
    author: &UserRow,
    note: &Note,
    ctx: Context,
) -> Result<Stored, FedError> {
    Box::pin(store_inner(
        state,
        author,
        note,
        ctx,
        ctx == Context::Lookup,
    ))
    .await
}

/// [`store_note`] for the post a looked-up note replies to: no further parent is fetched.
pub(super) async fn store_parent(
    state: &AppState,
    author: &UserRow,
    note: &Note,
) -> Result<Stored, FedError> {
    Box::pin(store_inner(state, author, note, Context::Lookup, false)).await
}

async fn store_inner(
    state: &AppState,
    author: &UserRow,
    note: &Note,
    ctx: Context,
    fetch_parent: bool,
) -> Result<Stored, FedError> {
    check_note(state, author, note)?;
    let db = state.db();
    if author.deleted_at.is_some() || author.suspended_at.is_some() {
        return Ok(Stored::Dropped);
    }
    let blocked = match &author.domain {
        Some(domain) => remote::domain_suspended(db, domain)
            .await
            .map_err(internal)?,
        None => true,
    };
    if blocked {
        return Ok(Stored::Dropped);
    }
    let now = OffsetDateTime::now_utc();
    let origin = author.ap_id.as_deref().unwrap_or_default();
    let followers = remote::followers_url(db, author.id)
        .await
        .map_err(internal)?;
    let visibility = visibility_of(note, origin, followers.as_deref());

    if let Some(existing) = remote::existing_note(db, &note.id)
        .await
        .map_err(internal)?
    {
        if existing.deleted || existing.author != author.id {
            return Ok(Stored::Dropped);
        }
        return update_existing(state, author, note, existing.id, now).await;
    }
    if remote::tombstoned(db, &note.id).await.map_err(internal)? {
        return Ok(Stored::Dropped);
    }
    if ctx == Context::Inbox && !should_store(state, author, note).await? {
        return Ok(Stored::Dropped);
    }
    if ctx == Context::Backfill && !matches!(visibility, Visibility::Public | Visibility::Unlisted)
    {
        return Ok(Stored::Dropped);
    }

    let reply_to = note
        .in_reply_to
        .as_deref()
        .and_then(|id| post_ref(state, id));
    if visibility == Visibility::Direct {
        match route_direct(state, author, note, reply_to.as_ref()).await? {
            Route::Message(recipient) => {
                return store_message(state, author, note, &recipient, now).await;
            }
            Route::Drop => return Ok(Stored::Dropped),
            Route::Post => {}
        }
    }

    if fetch_parent
        && let Some(PostRef::Remote(parent)) = &reply_to
        && remote::find_post(db, &PostRef::Remote(parent.clone()))
            .await
            .map_err(internal)?
            .is_none()
    {
        lookup::fetch_parent(state, parent).await;
    }

    let text = plain_text(note.content.as_deref(), MAX_TEXT_CHARS);
    let poll = poll_of(note, now);
    let media = if note.attachment.is_empty() {
        Vec::new()
    } else {
        Box::pin(super::media::copy_attachments(
            state,
            author.id,
            note.attachment.as_slice(),
        ))
        .await
    };
    if text.is_empty() && media.is_empty() && poll.is_none() {
        return Ok(Stored::Dropped);
    }
    let cw = plain_text(note.summary.as_deref(), MAX_CW_CHARS);
    copy_note_emotes(state, author, note, &text, &cw, poll.as_ref()).await;
    let mut local_handles = mentioned_handles(state, note);
    if visibility == Visibility::Direct {
        for handle in addressed_handles(state, note) {
            if !local_handles.contains(&handle) {
                local_handles.push(handle);
            }
        }
    }
    let created_at = note
        .published
        .as_deref()
        .and_then(|at| clamp_to_now(at, now))
        .unwrap_or(now);
    let new = NewNote {
        ap_id: note.id.clone(),
        author: author.id,
        text,
        cw,
        sensitive: note.sensitive,
        visibility,
        created_at,
        edited_at: edited_at(note, created_at, now),
        url: note.url.clone().filter(|url| ap::is_http_url(url)),
        reply_to,
        quote_of: note.quoted().and_then(|id| post_ref(state, id)),
        tags: hashtags(note),
        local_handles,
        poll,
        media,
        notify: ctx == Context::Inbox,
    };
    match remote::insert_note(db, &new).await.map_err(internal)? {
        Some(created) => {
            notifications::deliver(state, created.pending).await;
            crate::db::live::posts_created(state, &[created.id]).await;
            Ok(Stored::Post {
                id: created.id,
                created: true,
            })
        }
        None => match remote::existing_note(db, &note.id)
            .await
            .map_err(internal)?
        {
            Some(existing) if !existing.deleted && existing.author == author.id => {
                update_existing(state, author, note, existing.id, now).await
            }
            _ => Ok(Stored::Dropped),
        },
    }
}

fn edited_at(
    note: &Note,
    created_at: OffsetDateTime,
    now: OffsetDateTime,
) -> Option<OffsetDateTime> {
    note.updated
        .as_deref()
        .and_then(|at| clamp_to_now(at, now))
        .filter(|at| *at > created_at)
}

async fn update_existing(
    state: &AppState,
    author: &UserRow,
    note: &Note,
    id: Uuid,
    now: OffsetDateTime,
) -> Result<Stored, FedError> {
    let origin = author.ap_id.as_deref().unwrap_or_default();
    let followers = remote::followers_url(state.db(), author.id)
        .await
        .map_err(internal)?;
    let visibility = visibility_of(note, origin, followers.as_deref());
    let mut local_handles = mentioned_handles(state, note);
    if visibility == Visibility::Direct {
        local_handles.extend(addressed_handles(state, note));
        local_handles.sort_unstable();
        local_handles.dedup();
    }
    let text = plain_text(note.content.as_deref(), MAX_TEXT_CHARS);
    let cw = plain_text(note.summary.as_deref(), MAX_CW_CHARS);
    let poll = poll_of(note, now);
    copy_note_emotes(state, author, note, &text, &cw, poll.as_ref()).await;
    let edit = NoteEdit {
        text,
        cw,
        sensitive: note.sensitive,
        edited_at: note.updated.as_deref().and_then(|at| clamp_to_now(at, now)),
        tags: hashtags(note),
        local_handles,
        poll,
    };
    let done = remote::edit_note(state.db(), id, author.id, &edit)
        .await
        .map_err(internal)?;
    Ok(if done {
        Stored::Post { id, created: false }
    } else {
        Stored::Dropped
    })
}

/// Notes the copy of the emotes a remote note mentions in its text, content warning and poll.
async fn copy_note_emotes(
    state: &AppState,
    author: &UserRow,
    note: &Note,
    text: &str,
    cw: &str,
    poll: Option<&NewPoll>,
) {
    let mut texts = vec![text, cw];
    if let Some(poll) = poll {
        texts.extend(poll.options.iter().map(String::as_str));
    }
    Box::pin(copy_emotes(
        state,
        author.id,
        author.domain.as_deref(),
        note.tag.as_slice(),
        &texts,
    ))
    .await;
}

/// Where a direct note goes.
enum Route {
    /// A message to this local account.
    Message(Box<UserRow>),
    /// A direct post.
    Post,
    /// It names one local account that cannot receive; nothing is stored.
    Drop,
}

/// A direct note is a message when it is addressed to exactly one local account and nobody else
/// and does not reply to a stored post; otherwise it is a direct post.
async fn route_direct(
    state: &AppState,
    author: &UserRow,
    note: &Note,
    reply_to: Option<&PostRef>,
) -> Result<Route, FedError> {
    let origin = author.ap_id.as_deref().unwrap_or_default();
    let mut local: Vec<&str> = Vec::new();
    for entry in addressees(note) {
        if entry == origin || entry.is_empty() {
            continue;
        }
        match ids::local_handle_from_actor_id(state.config(), entry) {
            Some(handle) => {
                if !local.iter().any(|seen| seen.eq_ignore_ascii_case(handle)) {
                    local.push(handle);
                }
            }
            None => return Ok(Route::Post),
        }
    }
    let [handle] = local.as_slice() else {
        return Ok(Route::Post);
    };
    if let Some(parent) = reply_to
        && remote::find_post(state.db(), parent)
            .await
            .map_err(internal)?
            .is_some()
    {
        return Ok(Route::Post);
    }
    let recipient = remote::local_recipient(state.db(), handle)
        .await
        .map_err(internal)?;
    Ok(recipient.map_or(Route::Drop, |row| Route::Message(Box::new(row))))
}

async fn store_message(
    state: &AppState,
    author: &UserRow,
    note: &Note,
    recipient: &UserRow,
    now: OffsetDateTime,
) -> Result<Stored, FedError> {
    let db = state.db();
    let text = plain_text(note.content.as_deref(), MAX_MESSAGE_CHARS);
    let text = without_leading_mention(&text, recipient, state.config().domain());
    if text.is_empty() {
        return Ok(Stored::Dropped);
    }
    let reason = blocked_reason(db, author.id, recipient)
        .await
        .map_err(internal)?;
    if reason.is_some() {
        return Ok(Stored::Dropped);
    }
    Box::pin(copy_emotes(
        state,
        author.id,
        author.domain.as_deref(),
        note.tag.as_slice(),
        &[text],
    ))
    .await;
    let created_at = note
        .published
        .as_deref()
        .and_then(|at| clamp_to_now(at, now))
        .unwrap_or(now);
    let stored = remote::insert_message(db, author.id, recipient.id, &note.id, text, created_at)
        .await
        .map_err(internal)?;
    let Some(message) = stored else {
        return Ok(Stored::Message { created: false });
    };
    state.hub().publish(
        recipient.id,
        StreamEvent::Message(Box::new(MessageEventDto {
            from: user_ref(author),
            text: cap(text, PREVIEW_CHARS),
            encrypted: false,
            created_at: message.created_at,
        })),
    );
    publish_badges(state, recipient.id).await;
    Ok(Stored::Message { created: true })
}

/// Applies an `Update` of a note or poll by `author`: text, content warning, `edited_at`, an
/// entry in the edit history and the poll's counts. Only the author's own post changes. A note
/// that is not stored yet goes through [`store_note`] as an `Inbox` note.
///
/// # Errors
///
/// As [`store_note`].
pub async fn apply_note_update(
    state: &AppState,
    author: &UserRow,
    note: &Note,
) -> Result<Stored, FedError> {
    Box::pin(store_note(state, author, note, Context::Inbox)).await
}

/// Deletes the post or direct message `ap_id` when `author` wrote it, as a local delete does.
/// Returns whether anything was deleted.
///
/// # Errors
///
/// [`FedError::Internal`] on database failures.
pub async fn delete_note_by_ap_id(
    state: &AppState,
    author: &UserRow,
    ap_id: &str,
) -> Result<bool, FedError> {
    let origin = author.ap_id.as_deref().unwrap_or_default();
    remote::delete_by_ap_id(state.db(), state.hub(), author.id, origin, ap_id)
        .await
        .map_err(internal)
}

/// Applies an `Update` of a person: the profile fields, keys and endpoints of `user` and its
/// avatar and banner, copied again. `actor.id` must be the account's id, and the key must be the
/// actor's own.
///
/// # Errors
///
/// [`FedError::Unreadable`] when the actor is not `user`, or breaks the rules a fetched actor
/// must keep; [`FedError::NotFound`] when this server deleted the account;
/// [`FedError::Internal`] on database failures.
pub async fn apply_actor_update(
    state: &AppState,
    user: &UserRow,
    actor: Actor,
) -> Result<UserRow, FedError> {
    let origin = user.ap_id.as_deref().ok_or(FedError::NotFound)?;
    let domain = user.domain.as_deref().ok_or(FedError::NotFound)?;
    if actor.id != origin {
        return Err(FedError::Unreadable(domain.into()));
    }
    let actor = lookup::check_actor(state, actor, origin, Some((&user.handle, domain)))?;
    match lookup::store_actor(state, &actor).await {
        Ok(Some(row)) => Ok(row),
        Ok(None) => Err(FedError::NotFound),
        Err(err) => {
            delay_refresh(state.db(), user.id).await.map_err(internal)?;
            Err(err)
        }
    }
}

/// Deletes remote account `user` here, as an account deletion does: posts hidden, follows and
/// messages gone, nothing delivered to it again. The row stays as a tombstone.
///
/// # Errors
///
/// [`FedError::Internal`] on database failures.
pub async fn forget_actor(state: &AppState, user: &UserRow) -> Result<(), FedError> {
    remote::forget_actor(state.db(), user)
        .await
        .map_err(internal)
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    fn note(value: serde_json::Value) -> Note {
        serde_json::from_value(value).expect("note")
    }

    #[test]
    fn audience_decides_visibility() {
        let author = "https://a.example/users/ann";
        let base = |to: serde_json::Value, cc: serde_json::Value| {
            note(json!({"id": "https://a.example/n/1", "type": "Note", "to": to, "cc": cc}))
        };
        let followers = Some("https://a.example/users/ann/followers");
        assert_eq!(
            visibility_of(&base(json!([ap::PUBLIC]), json!([])), author, followers),
            Visibility::Public
        );
        assert_eq!(
            visibility_of(&base(json!("as:Public"), json!([])), author, followers),
            Visibility::Public
        );
        assert_eq!(
            visibility_of(
                &base(
                    json!(["https://a.example/users/ann/followers"]),
                    json!([ap::PUBLIC])
                ),
                author,
                followers
            ),
            Visibility::Unlisted
        );
        assert_eq!(
            visibility_of(
                &base(json!(["https://a.example/users/ann/followers"]), json!([])),
                author,
                followers
            ),
            Visibility::Followers
        );
        assert_eq!(
            visibility_of(
                &base(json!(["https://b.example/users/bob"]), json!([])),
                author,
                followers
            ),
            Visibility::Direct
        );
        assert_eq!(
            visibility_of(&base(json!([]), json!([])), author, None),
            Visibility::Direct
        );
    }

    #[test]
    fn text_is_cut_at_a_character_boundary() {
        assert_eq!(cap("héllo wörld", 4), "héll");
        assert_eq!(cap("abc", 10), "abc");
        assert_eq!(cap("日本語のテキスト", 3), "日本語");
    }

    #[test]
    fn hashtags_are_plain_lowercase_and_deduplicated() {
        let note = note(json!({
            "id": "https://a.example/n/1", "type": "Note",
            "tag": [
                {"type": "Hashtag", "name": "#Rust"},
                {"type": "Hashtag", "name": "#rust"},
                {"type": "Hashtag", "name": "#bad tag"},
                {"type": "Hashtag", "name": "#日本"},
                {"type": "Emoji", "name": ":x:"},
                {"type": "Mention", "href": "https://b.example/u/b"}
            ]
        }));
        assert_eq!(hashtags(&note), ["rust", "日本"]);
    }

    #[test]
    fn a_poll_needs_two_answers_and_counts_votes() {
        let now = OffsetDateTime::now_utc();
        let one = note(json!({
            "id": "https://a.example/q/1", "type": "Question",
            "oneOf": [{"name": "only"}]
        }));
        assert!(poll_of(&one, now).is_none());
        let two = note(json!({
            "id": "https://a.example/q/2", "type": "Question",
            "anyOf": [
                {"name": "yes", "replies": {"totalItems": 3}},
                {"name": "no", "replies": {"totalItems": 1}}
            ],
            "endTime": "2999-01-01T00:00:00Z", "votersCount": 3
        }));
        let poll = poll_of(&two, now).expect("poll");
        assert_eq!(poll.options, ["yes", "no"]);
        assert_eq!(poll.counts, [3, 1]);
        assert_eq!(poll.voters, 3);
        assert!(poll.multiple);
        assert!(poll.ends_at > now);
    }
}
