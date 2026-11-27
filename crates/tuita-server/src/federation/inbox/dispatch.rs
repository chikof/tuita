//! What a verified activity does. The authorship rules are here: a remote account can only
//! create, change and delete objects that are its own, and an object that claims a local id or
//! an id on another instance is dropped.

use serde_json::{Map, Value};
use tracing::{debug, info};
use tuita_core::ap::{self, Activity, ActivityKind, Actor, Note};

use super::{clean_id, follow, interact, report, same_id, soft};
use crate::db::users::UserRow;
use crate::error::AppError;
use crate::federation::ids;
use crate::federation::ingest::{self, Context};
use crate::state::AppState;

/// Applies `activity` by verified account `actor`. Anything that cannot be applied is dropped
/// with a log line; only failures on our side are errors.
pub(super) async fn dispatch(
    state: &AppState,
    actor: &UserRow,
    activity: &Activity,
) -> Result<(), AppError> {
    match activity.kind {
        ActivityKind::Follow => follow::follow(state, actor, activity).await,
        ActivityKind::Accept => follow::accept(state, actor, activity).await,
        ActivityKind::Reject => follow::reject(state, actor, activity).await,
        ActivityKind::Block => follow::block(state, actor, activity).await,
        ActivityKind::Undo => undo(state, actor, activity).await,
        ActivityKind::Create => create(state, actor, activity).await,
        ActivityKind::Update => update(state, actor, activity).await,
        ActivityKind::Delete => delete(state, actor, activity).await,
        ActivityKind::Like => interact::like(state, actor, activity).await,
        ActivityKind::Announce => interact::announce(state, actor, activity).await,
        ActivityKind::Flag => report::flag(state, actor, activity).await,
        ActivityKind::Move => {
            info!(actor = %activity.actor, "move activity ignored");
            Ok(())
        }
        ActivityKind::Other => {
            debug!(actor = %activity.actor, "unsupported activity ignored");
            Ok(())
        }
    }
}

/// The embedded activity of an `Undo`, when it is one the actor did.
fn embedded_activity(actor: &UserRow, activity: &Activity) -> Option<Activity> {
    let map: &Map<String, Value> = activity.object_map()?;
    let inner: Activity = serde_json::from_value(Value::Object(map.clone())).ok()?;
    let own = actor.ap_id.as_deref()?;
    (inner.actor.is_empty() || same_id(&inner.actor, own)).then_some(inner)
}

/// `Undo` of a follow, like or announce. Blocks leave nothing to undo.
async fn undo(state: &AppState, actor: &UserRow, activity: &Activity) -> Result<(), AppError> {
    if activity.object_map().is_none() {
        let Some(id) = clean_id(activity.object_id()) else {
            return Ok(());
        };
        follow::undo_by_id(state, actor, id).await?;
        return interact::undo_announce_by_id(state, actor, id).await;
    }
    let Some(inner) = embedded_activity(actor, activity) else {
        debug!("undo of an activity by someone else");
        return Ok(());
    };
    match inner.kind {
        ActivityKind::Follow => follow::undo(state, actor, &inner).await,
        ActivityKind::Like => interact::undo_like(state, actor, &inner).await,
        ActivityKind::Announce => interact::undo_announce(state, actor, &inner).await,
        _ => Ok(()),
    }
}

/// Whether `kind` is a note type.
fn is_note_type(kind: &str) -> bool {
    matches!(kind, "Note" | "Question" | "Page" | "Article")
}

/// The embedded note of a `Create` or `Update`, when it is one the actor wrote: a note type,
/// `attributedTo` the actor, its id a URL on the actor's instance and not on this one.
fn authored_note(state: &AppState, actor: &UserRow, activity: &Activity) -> Option<Note> {
    let map = activity.object_map()?;
    if !activity.object_type().is_some_and(is_note_type) {
        return None;
    }
    let mut note: Note = serde_json::from_value(Value::Object(map.clone())).ok()?;
    // Some servers address the activity and leave the note bare.
    if note.to.is_empty() && note.cc.is_empty() {
        note.to = activity.to.clone();
        note.cc = activity.cc.clone();
    }
    let origin = actor.ap_id.as_deref()?;
    let ours = ap::url_authority(&note.id).as_deref() == Some(state.config().domain())
        || ids::local_post_from_note_id(state.config(), &note.id).is_some();
    let authored = ap::is_http_url(&note.id)
        && ap::same_authority(&note.id, origin)
        && note
            .attributed_to
            .as_deref()
            .is_some_and(|author| same_id(author, origin))
        && !ours;
    if !authored {
        debug!(note = %note.id, "note not written by its actor, or claims a local id");
    }
    authored.then_some(note)
}

/// `Create` of a note, or of a poll vote.
async fn create(state: &AppState, actor: &UserRow, activity: &Activity) -> Result<(), AppError> {
    let Some(note) = authored_note(state, actor, activity) else {
        return Ok(());
    };
    if interact::poll_vote(state, actor, &note).await? {
        return Ok(());
    }
    soft(
        "create",
        ingest::store_note(state, actor, &note, Context::Inbox).await,
    )?;
    Ok(())
}

/// `Update` of a note or a poll, or of the actor itself.
async fn update(state: &AppState, actor: &UserRow, activity: &Activity) -> Result<(), AppError> {
    let Some(kind) = activity.object_type() else {
        return Ok(());
    };
    if is_note_type(kind) {
        let Some(note) = authored_note(state, actor, activity) else {
            return Ok(());
        };
        soft(
            "update",
            ingest::apply_note_update(state, actor, &note).await,
        )?;
    } else if matches!(
        kind,
        "Person" | "Service" | "Organization" | "Application" | "Group"
    ) {
        let Some(map) = activity.object_map() else {
            return Ok(());
        };
        let Ok(profile) = serde_json::from_value::<Actor>(Value::Object(map.clone())) else {
            debug!("update of an actor that cannot be read");
            return Ok(());
        };
        if !actor
            .ap_id
            .as_deref()
            .is_some_and(|own| same_id(own, &profile.id))
        {
            debug!("update of someone else's profile");
            return Ok(());
        }
        soft(
            "profile update",
            ingest::apply_actor_update(state, actor, profile).await,
        )?;
    }
    Ok(())
}

/// `Delete` of a note (the object is its id, or a `Tombstone`, or the note) or of the actor
/// itself. Only the author deletes a note.
async fn delete(state: &AppState, actor: &UserRow, activity: &Activity) -> Result<(), AppError> {
    let Some(object) = clean_id(activity.object_id()) else {
        return Ok(());
    };
    let own = actor.ap_id.as_deref().unwrap_or_default();
    if same_id(object, own) {
        soft("delete account", ingest::forget_actor(state, actor).await)?;
        return Ok(());
    }
    let ours = ap::url_authority(object).as_deref() == Some(state.config().domain());
    if ours {
        debug!(object, "delete of a local id refused");
        return Ok(());
    }
    soft(
        "delete",
        ingest::delete_note_by_ap_id(state, actor, object).await,
    )?;
    Ok(())
}
