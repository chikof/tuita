//! Follows: a remote account follows a local one, answers a follow a local account sent, takes
//! it back, or blocks.

use serde_json::{Value, json};
use tracing::debug;
use tuita_core::ap::{Activity, ActivityKind, id_of};
use uuid::Uuid;

use super::clean_id;
use crate::db::inbound::{self, FollowReplies};
use crate::db::notifications;
use crate::db::remote::local_recipient;
use crate::db::users::UserRow;
use crate::error::AppError;
use crate::federation::ids;
use crate::state::AppState;

/// The live local account an activity's `object` names as an actor.
async fn local_target(state: &AppState, object: Option<&str>) -> Result<Option<UserRow>, AppError> {
    let Some(handle) = object.and_then(|id| ids::local_handle_from_actor_id(state.config(), id))
    else {
        return Ok(None);
    };
    local_recipient(state.db(), handle).await
}

/// `Accept` or `Reject` of `follow`, by `local`, as Mastodon sends them: the original `Follow`
/// is embedded.
fn reply(
    state: &AppState,
    kind: ActivityKind,
    local: &UserRow,
    remote_actor: &str,
    follow_id: Option<&str>,
) -> Value {
    let config = state.config();
    let local_actor = ids::actor_id(config, &local.handle);
    let mut follow = json!({
        "type": "Follow",
        "actor": remote_actor,
        "object": local_actor,
    });
    if let (Some(id), Some(map)) = (follow_id, follow.as_object_mut()) {
        map.insert("id".to_owned(), Value::String(id.to_owned()));
    }
    Activity::new(
        kind,
        Some(ids::activity_id(config, Uuid::now_v7())),
        &local_actor,
        follow,
    )
    .to_document()
}

/// `Follow` of a local account. A blocked follower is rejected, anyone else accepted at once.
pub(super) async fn follow(
    state: &AppState,
    actor: &UserRow,
    activity: &Activity,
) -> Result<(), AppError> {
    let Some(local) = local_target(state, activity.object_id()).await? else {
        debug!("follow of something that is not a local account");
        return Ok(());
    };
    let (Some(remote_actor), Some(inbox)) = (
        actor.ap_id.as_deref(),
        actor
            .inbox_url
            .as_deref()
            .or(actor.shared_inbox_url.as_deref()),
    ) else {
        return Ok(());
    };
    let follow_id = clean_id(activity.id.as_deref());
    let accept = reply(state, ActivityKind::Accept, &local, remote_actor, follow_id);
    let reject = reply(state, ActivityKind::Reject, &local, remote_actor, follow_id);
    let done = inbound::receive_follow(
        state.db(),
        local.id,
        actor.id,
        follow_id,
        &FollowReplies {
            inbox,
            accept: &accept,
            reject: &reject,
        },
    )
    .await?;
    notifications::deliver(state, done.pending).await;
    Ok(())
}

/// What an `Accept` or `Reject` says about the follow: its id, and the local account that sent
/// it when the `Follow` is embedded.
struct Answered<'a> {
    id: Option<&'a str>,
    follower: Option<&'a str>,
}

fn answered<'a>(state: &AppState, activity: &'a Activity) -> Answered<'a> {
    let follower = activity
        .object_map()
        .and_then(|map| map.get("actor"))
        .and_then(id_of)
        .and_then(|actor| ids::local_handle_from_actor_id(state.config(), actor));
    Answered {
        id: clean_id(activity.object_id()),
        follower,
    }
}

/// `Accept` of a follow a local account sent: the pending follow becomes accepted.
pub(super) async fn accept(
    state: &AppState,
    actor: &UserRow,
    activity: &Activity,
) -> Result<(), AppError> {
    let found = answered(state, activity);
    let count = inbound::accept_pending(state.db(), actor.id, found.id, found.follower).await?;
    if count == 0 {
        debug!("accept without a pending follow");
    }
    Ok(())
}

/// `Reject` of a follow a local account sent, or a remote account dropping a follower.
pub(super) async fn reject(
    state: &AppState,
    actor: &UserRow,
    activity: &Activity,
) -> Result<(), AppError> {
    let found = answered(state, activity);
    let count = inbound::reject_follow(state.db(), actor.id, found.id, found.follower).await?;
    if count == 0 {
        debug!("reject without a follow");
    }
    Ok(())
}

/// `Undo` of a `Follow` by `actor`. `inner` is the embedded `Follow`; without one, `object_id`
/// is the id of the follow to undo.
pub(super) async fn undo(
    state: &AppState,
    actor: &UserRow,
    inner: &Activity,
) -> Result<(), AppError> {
    let target = local_target(state, inner.object_id())
        .await?
        .map(|row| row.id);
    inbound::undo_follow(state.db(), actor.id, clean_id(inner.id.as_deref()), target).await?;
    Ok(())
}

/// `Undo` that names the follow by id only.
pub(super) async fn undo_by_id(
    state: &AppState,
    actor: &UserRow,
    id: &str,
) -> Result<(), AppError> {
    inbound::undo_follow(state.db(), actor.id, clean_id(Some(id)), None).await?;
    Ok(())
}

/// `Block` of a local account by a remote one: the follows between them go.
pub(super) async fn block(
    state: &AppState,
    actor: &UserRow,
    activity: &Activity,
) -> Result<(), AppError> {
    let Some(local) = local_target(state, activity.object_id()).await? else {
        return Ok(());
    };
    inbound::sever_follows(state.db(), actor.id, local.id).await
}
