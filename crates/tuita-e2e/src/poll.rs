//! Retry loop behind every action and expectation.

use std::{
    fmt::Display,
    future::Future,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow};

/// Gap between two attempts.
pub const INTERVAL: Duration = Duration::from_millis(50);

/// One attempt's verdict.
#[derive(Debug)]
pub enum Poll<T> {
    /// Done with this value.
    Ready(T),
    /// Not yet; the reason is shown if the deadline passes.
    Retry(String),
}

/// Runs `attempt` every 50 ms until it returns [`Poll::Ready`]. An `Err` counts as "not yet" too,
/// because a page that navigates under a running query answers with transient protocol errors.
/// After `timeout` the last reason or error is returned together with `what`.
pub async fn poll_for<T, F, Fut>(what: impl Display, timeout: Duration, mut attempt: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<Poll<T>>>,
{
    let deadline = Instant::now() + timeout;
    loop {
        let reason = match attempt().await {
            Ok(Poll::Ready(value)) => return Ok(value),
            Ok(Poll::Retry(reason)) => reason,
            Err(error) => format!("{error:#}"),
        };
        if Instant::now() >= deadline {
            return Err(anyhow!(
                "{what}: timed out after {} ms: {reason}",
                timeout.as_millis()
            ));
        }
        tokio::time::sleep(INTERVAL).await;
    }
}

/// [`poll_for`] with the default `E2E_TIMEOUT_MS`.
pub async fn poll<T, F, Fut>(what: impl Display, attempt: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<Poll<T>>>,
{
    poll_for(what, crate::env().timeout, attempt).await
}
