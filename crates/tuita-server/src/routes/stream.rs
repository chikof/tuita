//! `GET /stream`: server-sent events for the signed-in user.

use std::convert::Infallible;
use std::time::Duration;

use axum::Extension;
use axum::Router;
use axum::extract::State;
use axum::http::header::{CACHE_CONTROL, HeaderName};
use axum::response::IntoResponse;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::get;
use futures_util::StreamExt;
use futures_util::stream::{self, Stream};
use tokio::sync::broadcast::error::RecvError;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::auth::bearer::TokenAuth;
use crate::auth::{Credential, CurrentUser};
use crate::db::{notifications, sessions, tokens};
use crate::dto::{StreamEvent, TokenScope};
use crate::error::AppError;
use crate::state::AppState;

/// How long a browser waits before it reconnects.
const RETRY: Duration = Duration::from_secs(5);
/// Gap between keep-alive comments, under the usual proxy idle timeout.
const KEEP_ALIVE: Duration = Duration::from_secs(25);

/// Whether the session or token an open stream started with still works.
async fn credential_is_live(db: &sqlx::PgPool, credential: Credential) -> Result<bool, AppError> {
    match credential {
        Credential::Session(id) => sessions::is_live(db, id).await,
        Credential::Token(id) => tokens::is_live(db, id).await,
    }
}

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/stream", get(stream))
}

fn frame(event: &StreamEvent) -> Option<Event> {
    match Event::default().event(event.name()).json_data(event) {
        Ok(frame) => Some(frame),
        Err(err) => {
            tracing::warn!(error = %err, "cannot encode a stream event");
            None
        }
    }
}

async fn stream(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    token: Option<Extension<TokenAuth>>,
) -> Result<impl IntoResponse, AppError> {
    let messages =
        token.is_none_or(|Extension(token)| token.scopes.contains(&TokenScope::Messages));
    let user = auth.user.id;
    // Subscribe first: an event published while the counts load is queued, not lost.
    let rx = state.hub().subscribe(user);
    let counts = notifications::badges(state.db(), user, auth.user.is_staff()).await?;
    let head: Vec<Result<Event, Infallible>> = std::iter::once(Event::default().retry(RETRY))
        .chain(frame(&StreamEvent::Badges(counts)))
        .map(Ok)
        .collect();
    let (stream_id, evicted) = state.hub().open_stream(user);
    let open = Open {
        state,
        user,
        stream_id,
        evicted,
        credential: auth.credential,
        messages,
        staff: auth.user.is_staff(),
    };
    let body = stream::iter(head).chain(live(open, rx));
    let headers = [
        (CACHE_CONTROL, "no-cache, no-transform"),
        (HeaderName::from_static("x-accel-buffering"), "no"),
    ];
    Ok((
        headers,
        Sse::new(body).keep_alive(KeepAlive::new().interval(KEEP_ALIVE).text("ping")),
    ))
}

/// One open stream. Dropping it unregisters the stream from the hub.
struct Open {
    state: AppState,
    user: Uuid,
    stream_id: u64,
    evicted: CancellationToken,
    credential: Credential,
    messages: bool,
    staff: bool,
}

impl Drop for Open {
    fn drop(&mut self) {
        self.state.hub().close_stream(self.user, self.stream_id);
    }
}

/// The live part of the stream. Ends when the server drains, when a newer stream pushes this
/// one out of the per-user cap, or when the session stops validating (logout, revoke, password
/// change, suspension), checked every [`AppState::stream_recheck`].
fn live(
    open: Open,
    rx: tokio::sync::broadcast::Receiver<crate::realtime::Event>,
) -> impl Stream<Item = Result<Event, Infallible>> {
    let mut recheck = tokio::time::interval(open.state.stream_recheck());
    recheck.reset();
    stream::unfold(
        (open, rx, recheck),
        move |(open, mut rx, mut recheck)| async move {
            loop {
                let event = tokio::select! {
                    () = open.state.shutdown().cancelled() => return None,
                    () = open.evicted.cancelled() => return None,
                    _ = recheck.tick() => {
                        match credential_is_live(open.state.db(), open.credential).await {
                            Ok(true) => continue,
                            Ok(false) => return None,
                            Err(err) => {
                                tracing::warn!(error = %err, "cannot re-check a stream session");
                                continue;
                            }
                        }
                    }
                    received = rx.recv() => received,
                };
                let next = match event {
                    // Direct messages need the `messages` scope; a `read` token never sees them.
                    Ok(event) if !open.messages && matches!(*event, StreamEvent::Message(_)) => {
                        None
                    }
                    // Timeline entries are for the Mastodon websocket; the web client never sees them.
                    Ok(event)
                        if matches!(*event, StreamEvent::Update(_) | StreamEvent::Delete(_)) =>
                    {
                        None
                    }
                    Ok(event) => frame(&event),
                    // A slow reader lost events: counts are the cheap way to catch up.
                    Err(RecvError::Lagged(_)) => {
                        match notifications::badges(open.state.db(), open.user, open.staff).await {
                            Ok(counts) => frame(&StreamEvent::Badges(counts)),
                            Err(err) => {
                                tracing::warn!(error = %err, "cannot refresh badges after lag");
                                None
                            }
                        }
                    }
                    Err(RecvError::Closed) => return None,
                };
                if let Some(next) = next {
                    return Some((Ok(next), (open, rx, recheck)));
                }
            }
        },
    )
}
