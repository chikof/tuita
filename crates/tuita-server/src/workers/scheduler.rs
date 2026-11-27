//! The scheduler worker: publishes due scheduled posts every few seconds.

use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::db::scheduled;
use crate::state::AppState;

/// Gap between two passes. A post publishes at most this long after its time.
const INTERVAL: Duration = Duration::from_secs(5);

/// Publishes due posts every 5 seconds until `stop` is cancelled. A failed pass is logged;
/// the next one still happens. Several replicas can run it: rows are taken with
/// `FOR UPDATE SKIP LOCKED`.
pub async fn run(state: AppState, stop: CancellationToken) {
    let mut ticker = tokio::time::interval(INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            () = stop.cancelled() => return,
            _ = ticker.tick() => {}
        }
        match scheduled::publish_due(&state).await {
            Ok(0) => {}
            Ok(published) => tracing::info!(published, "published scheduled posts"),
            Err(err) => tracing::error!(error = %err, "scheduler pass failed"),
        }
    }
}
