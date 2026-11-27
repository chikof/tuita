//! Background workers. [`spawn_workers`] starts them; `serve_on` awaits them during the drain.

pub mod cleanup;
pub mod delivery;
pub mod scheduler;
pub mod unfurl;

use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use crate::state::AppState;

/// Starts every worker. Each one ends when `stop` is cancelled; await the returned set to wait
/// for them.
#[must_use]
pub fn spawn_workers(state: &AppState, stop: &CancellationToken) -> JoinSet<()> {
    let mut workers = JoinSet::new();
    workers.spawn(cleanup::run(state.clone(), stop.clone()));
    workers.spawn(scheduler::run(state.clone(), stop.clone()));
    workers.spawn(delivery::run(state.clone(), stop.clone()));
    if let Some(jobs) = state.take_unfurl_queue() {
        workers.spawn(unfurl::run(state.clone(), jobs, stop.clone()));
    }
    workers
}
