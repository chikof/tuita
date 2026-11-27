//! Likes, reposts and poll votes by remote accounts on local posts. Only local posts count: an
//! object that is not a local note id is ignored.

use tracing::debug;
use tuita_core::ap::{self, Activity, Note};
use uuid::Uuid;

use crate::db::inbound;
use crate::db::notifications;
use crate::db::users::UserRow;
use crate::error::AppError;
use crate::federation::ids;
use crate::state::AppState;

/// The author handle and id of the local post a note id names.
fn local_post<'a>(state: &AppState, id: Option<&'a str>) -> Option<(&'a str, Uuid)> {
    id.and_then(|id| ids::local_post_from_note_id(state.config(), id))
}

/// `Like` of a local post.
pub(super) async fn like(
    state: &AppState,
    actor: &UserRow,
    activity: &Activity,
) -> Result<(), AppError> {
    let Some((handle, id)) = local_post(state, activity.object_id()) else {
        debug!("like of something that is not a local post");
        return Ok(());
    };
    let pending = inbound::like(state.db(), actor.id, handle, id).await?;
    notifications::deliver(state, pending).await;
    Ok(())
}

/// `Announce` of a local post.
pub(super) async fn announce(
    state: &AppState,
    actor: &UserRow,
    activity: &Activity,
) -> Result<(), AppError> {
    let Some((handle, id)) = local_post(state, activity.object_id()) else {
        debug!("announce of something that is not a local post");
        return Ok(());
    };
    let ap_id = super::clean_id(activity.id.as_deref()).filter(|id| {
        actor
            .ap_id
            .as_deref()
            .is_some_and(|own| ap::same_authority(id, own))
    });
    let pending = inbound::announce(state.db(), actor.id, handle, id, ap_id).await?;
    notifications::deliver(state, pending).await;
    Ok(())
}

/// `Undo` of a `Like` of a local post.
pub(super) async fn undo_like(
    state: &AppState,
    actor: &UserRow,
    inner: &Activity,
) -> Result<(), AppError> {
    if let Some((handle, id)) = local_post(state, inner.object_id()) {
        inbound::unlike(state.db(), actor.id, handle, id).await?;
    }
    Ok(())
}

/// `Undo` of an `Announce`: by the announce's id, or by the local post it named.
pub(super) async fn undo_announce(
    state: &AppState,
    actor: &UserRow,
    inner: &Activity,
) -> Result<(), AppError> {
    let post = local_post(state, inner.object_id());
    inbound::unannounce(
        state.db(),
        actor.id,
        super::clean_id(inner.id.as_deref()),
        post,
    )
    .await
}

/// `Undo` that names the announce by id only.
pub(super) async fn undo_announce_by_id(
    state: &AppState,
    actor: &UserRow,
    id: &str,
) -> Result<(), AppError> {
    inbound::unannounce(state.db(), actor.id, super::clean_id(Some(id)), None).await
}

/// A `Create` of a `Note` that is a vote: it has a `name` (the chosen option) and no content, and
/// answers a local post. Returns whether it was one; the vote is recorded when it counts, and
/// is never stored as a post.
pub(super) async fn poll_vote(
    state: &AppState,
    actor: &UserRow,
    note: &Note,
) -> Result<bool, AppError> {
    let Some(option) = note
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
    else {
        return Ok(false);
    };
    let has_content = note
        .content
        .as_deref()
        .is_some_and(|content| !content.trim().is_empty());
    let Some((handle, id)) = local_post(state, note.in_reply_to.as_deref()) else {
        return Ok(false);
    };
    if has_content {
        return Ok(false);
    }
    let counted = inbound::vote(state.db(), actor.id, handle, id, option).await?;
    debug!(counted, "poll vote");
    Ok(true)
}
