//! `Flag`: a report from another instance, filed under the system account `@tuita`.

use tracing::debug;
use tuita_core::ap::{self, Activity};
use uuid::Uuid;

use crate::db::inbound;
use crate::db::users::UserRow;
use crate::error::AppError;
use crate::federation::{ids, ingest};
use crate::state::AppState;

/// Most objects of one `Flag` looked at.
const MAX_OBJECTS: usize = 10;
/// Most posts of one `Flag` that get a report each.
const MAX_POSTS: usize = 5;
/// Longest report note, in characters: the limit of a report by a local user.
const MAX_NOTE_CHARS: usize = 1000;
/// Longest reason kept, in characters. The rest of the note names the reporter.
const MAX_REASON_CHARS: usize = 800;

/// Files the report. The target is the local account the `Flag` names, or else the author of the
/// first local post it names; each reported post of that account gets its own report. Objects
/// that are not local are ignored, and a `Flag` without a local target is dropped.
pub(super) async fn flag(
    state: &AppState,
    actor: &UserRow,
    activity: &Activity,
) -> Result<(), AppError> {
    let config = state.config();
    let db = state.db();
    let mut account: Option<Uuid> = None;
    let mut posts: Vec<(Uuid, Uuid)> = Vec::new();
    for object in activity.object_ids().into_iter().take(MAX_OBJECTS) {
        if let Some(handle) = ids::local_handle_from_actor_id(config, object) {
            if account.is_none() {
                account = inbound::local_account(db, handle).await?;
            }
        } else if let Some((handle, id)) = ids::local_post_from_note_id(config, object)
            && let Some(author) = inbound::local_post_author(db, handle, id).await?
        {
            posts.push((id, author));
        }
    }
    let Some(target) = account.or_else(|| posts.first().map(|(_, author)| *author)) else {
        debug!("flag without a local target");
        return Ok(());
    };
    let reason = activity
        .content
        .as_deref()
        .map(|html| ingest::cap(ap::html_to_text(html).trim(), MAX_REASON_CHARS))
        .unwrap_or_default();
    let origin = actor.ap_id.as_deref().unwrap_or_default();
    let note = ingest::cap(&format!("Reported from {origin}: {reason}"), MAX_NOTE_CHARS);
    let ids: Vec<Option<Uuid>> = posts
        .iter()
        .filter(|(_, author)| *author == target)
        .map(|(id, _)| Some(*id))
        .take(MAX_POSTS)
        .collect();
    let reported = if ids.is_empty() { vec![None] } else { ids };
    for post in reported {
        inbound::file_report(db, target, post, &note).await?;
    }
    Ok(())
}
