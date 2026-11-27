//! `GET /streaming`: the Mastodon websocket.
//!
//! `GET /api/v1/streaming?stream=user&access_token=<token>` upgrades to a websocket that pushes
//! `update`, `delete` and `notification` events as JSON text frames
//! `{"stream":["user"],"event":"update","payload":"<json string>"}`: the payload of `update` is a
//! `Status` and of `notification` a `Notification`, each mapped for the connected user, and the
//! payload of `delete` is the id of the entry that went. Clients may also send
//! `{"type":"subscribe","stream":"user"}` and `{"type":"unsubscribe","stream":"user"}`. The
//! streams are `user` (updates, deletes and notifications) and `user:notification`
//! (notifications only); a client that asks for another stream gets an `error` event and stays
//! connected.
//!
//! Only a token authenticates a websocket. It comes from the `Authorization` header, the
//! `access_token` query parameter or the `Sec-WebSocket-Protocol` header (which the upgrade echoes
//! back, as browsers require): the bearer gate checks it, so a missing or bad one is `401` before
//! the upgrade, and a session cookie alone is refused so that another site cannot open a socket
//! with a visitor's cookie. The query string is never logged.
//!
//! The socket shares the per-user cap of [`crate::realtime::MAX_STREAMS_PER_USER`] with the SSE
//! stream (the newest connection pushes the oldest out) and ends when the server drains, when the
//! token is revoked or its owner is suspended (checked every [`AppState::stream_recheck`]), when
//! the client stops answering pings, or when it cannot take a frame in time. Binary frames are
//! ignored and a client message over [`MAX_CLIENT_MESSAGE`] bytes closes the connection.
//!
//! Which events a user hears of is decided when they happen, in [`crate::db::live`] and
//! [`crate::db::notifications`]. The hub carries ids; this file loads each post as the connected
//! user sees it, so blocks, mutes, muted words and visibility apply here.

use std::time::Duration;

use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing::get;
use axum::{Extension, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{Receiver, error::RecvError};
use tokio::time::{Instant, MissedTickBehavior, interval, timeout};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::ctx::Ctx;
use super::dto;
use crate::auth::bearer::TokenAuth;
use crate::db::{posts, tokens, users};
use crate::dto::{NotificationEventDto, PostEventDto, StreamEvent};
use crate::error::AppError;
use crate::realtime::Event;
use crate::state::AppState;

/// Gap between pings, under the usual proxy idle timeout.
const PING_EVERY: Duration = Duration::from_secs(25);
/// Missed ping intervals after which a silent client is dropped.
const IDLE_INTERVALS: u32 = 3;
/// How long a frame may take to leave before the client counts as stuck.
const SEND_TIMEOUT: Duration = Duration::from_secs(10);
/// Largest message a client may send; every legitimate one is a few dozen bytes.
const MAX_CLIENT_MESSAGE: usize = 4096;

/// Close code: the server is going away, or the connection was replaced.
const GOING_AWAY: u16 = 1001;
/// Close code: the token no longer works.
const POLICY: u16 = 1008;

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/streaming", get(connect))
}

#[derive(Deserialize)]
struct ConnectQuery {
    stream: Option<String>,
}

async fn connect(
    State(state): State<AppState>,
    token: Option<Extension<TokenAuth>>,
    Query(query): Query<ConnectQuery>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    let Some(Extension(token)) = token else {
        return Err(AppError::Unauthorized);
    };
    let user = token.user.id;
    // Subscribe first: an event published while the socket upgrades is queued, not lost.
    let rx = state.hub().subscribe(user);
    let (stream_id, evicted) = state.hub().open_stream(user);
    let open = Open {
        state,
        user,
        token: token.token_id,
        stream_id,
        evicted,
    };
    let ws = ws
        .max_message_size(MAX_CLIENT_MESSAGE)
        .max_frame_size(MAX_CLIENT_MESSAGE);
    let ws = match offered_protocol(&headers) {
        Some(protocol) => ws.protocols([protocol]),
        None => ws,
    };
    Ok(ws.on_upgrade(move |socket| serve(socket, open, rx, query.stream)))
}

/// The first `Sec-WebSocket-Protocol` value: where a browser puts the token.
fn offered_protocol(headers: &HeaderMap) -> Option<String> {
    let value = headers.get("sec-websocket-protocol")?.to_str().ok()?;
    let first = value.split(',').next()?.trim();
    (!first.is_empty()).then(|| first.to_owned())
}

/// One open socket. Dropping it unregisters the stream from the hub.
struct Open {
    state: AppState,
    user: Uuid,
    token: Uuid,
    stream_id: u64,
    evicted: CancellationToken,
}

impl Drop for Open {
    fn drop(&mut self) {
        self.state.hub().close_stream(self.user, self.stream_id);
    }
}

/// A stream a client can ask for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Channel {
    User,
    UserNotification,
}

impl Channel {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "user" => Some(Self::User),
            "user:notification" => Some(Self::UserNotification),
            _ => None,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::UserNotification => "user:notification",
        }
    }
}

/// The streams a socket listens to.
#[derive(Clone, Copy, Debug, Default)]
struct Subscriptions {
    user: bool,
    notifications: bool,
}

impl Subscriptions {
    fn set(&mut self, channel: Channel, on: bool) {
        match channel {
            Channel::User => self.user = on,
            Channel::UserNotification => self.notifications = on,
        }
    }
}

/// A message a client sends.
#[derive(Deserialize)]
struct ClientMessage {
    #[serde(rename = "type")]
    kind: String,
    stream: Option<String>,
}

/// A frame the server sends.
#[derive(Serialize)]
struct Frame<'a> {
    stream: [&'a str; 1],
    event: &'a str,
    payload: &'a str,
}

async fn serve(
    mut socket: WebSocket,
    open: Open,
    mut rx: Receiver<Event>,
    initial: Option<String>,
) {
    let mut subs = Subscriptions::default();
    if let Some(name) = initial
        && !subscribe(&mut socket, &mut subs, &name, true).await
    {
        return;
    }
    let ping_every = PING_EVERY.min(open.state.stream_recheck());
    let mut ping = interval(ping_every);
    ping.set_missed_tick_behavior(MissedTickBehavior::Delay);
    ping.reset();
    let mut recheck = interval(open.state.stream_recheck());
    recheck.reset();
    let mut last_heard = Instant::now();
    loop {
        let alive = tokio::select! {
            () = open.state.shutdown().cancelled() => Err(GOING_AWAY),
            () = open.evicted.cancelled() => Err(GOING_AWAY),
            _ = recheck.tick() => match tokens::is_live(open.state.db(), open.token).await {
                Ok(true) => Ok(()),
                Ok(false) => Err(POLICY),
                Err(err) => {
                    tracing::warn!(error = %err, "cannot re-check a streaming token");
                    Ok(())
                }
            },
            _ = ping.tick() => {
                if last_heard.elapsed() > ping_every * IDLE_INTERVALS {
                    Err(GOING_AWAY)
                } else if send(&mut socket, Message::Ping(Vec::new().into())).await {
                    Ok(())
                } else {
                    return;
                }
            }
            incoming = socket.recv() => match incoming {
                Some(Ok(message)) => {
                    last_heard = Instant::now();
                    if client_message(&mut socket, &mut subs, message).await {
                        Ok(())
                    } else {
                        return;
                    }
                }
                Some(Err(_)) | None => return,
            },
            received = rx.recv() => match received {
                Ok(event) => {
                    if forward(&mut socket, &open, subs, &event).await {
                        Ok(())
                    } else {
                        return;
                    }
                }
                // Mastodon's streaming server does not resync either: a client that lost events
                // refreshes its timeline.
                Err(RecvError::Lagged(_)) => Ok(()),
                Err(RecvError::Closed) => Err(GOING_AWAY),
            },
        };
        if let Err(code) = alive {
            close(&mut socket, code).await;
            return;
        }
    }
}

/// Handles one client frame. `false` when the connection is over.
async fn client_message(
    socket: &mut WebSocket,
    subs: &mut Subscriptions,
    message: Message,
) -> bool {
    let Message::Text(text) = message else {
        // Pings are answered by the socket itself; binary frames mean nothing here.
        return !matches!(message, Message::Close(_));
    };
    let Ok(request) = serde_json::from_str::<ClientMessage>(text.as_str()) else {
        return true;
    };
    let Some(name) = request.stream else {
        return true;
    };
    match request.kind.as_str() {
        "subscribe" => subscribe(socket, subs, &name, true).await,
        "unsubscribe" => subscribe(socket, subs, &name, false).await,
        _ => true,
    }
}

/// Turns stream `name` on or off, or answers an `error` event when it is not one we serve.
/// `false` when the answer could not be sent.
async fn subscribe(socket: &mut WebSocket, subs: &mut Subscriptions, name: &str, on: bool) -> bool {
    match Channel::parse(name) {
        Some(channel) => {
            subs.set(channel, on);
            true
        }
        None => {
            let stream = if name.len() <= 64 { name } else { "" };
            send_frame(socket, stream, "error", "Unknown stream type").await
        }
    }
}

/// Sends what `event` means for this socket, if anything. `false` when the socket is stuck.
async fn forward(
    socket: &mut WebSocket,
    open: &Open,
    subs: Subscriptions,
    event: &StreamEvent,
) -> bool {
    let state = &open.state;
    let frames = match event {
        StreamEvent::Update(update) if subs.user => status_frame(state, open.user, update)
            .await
            .map(|payload| payload.map(|payload| ("update", payload))),
        StreamEvent::Delete(entry) if subs.user => Ok(Some(("delete", entry.to_string()))),
        StreamEvent::Notification(note) if subs.user || subs.notifications => {
            notification_frame(state, open.user, note)
                .await
                .map(|payload| payload.map(|payload| ("notification", payload)))
        }
        _ => Ok(None),
    };
    let (name, payload) = match frames {
        Ok(Some(frame)) => frame,
        Ok(None) => return true,
        Err(err) => {
            tracing::warn!(error = %err, "cannot build a streaming event");
            return true;
        }
    };
    if subs.user && !send_frame(socket, Channel::User.name(), name, &payload).await {
        return false;
    }
    if name == "notification"
        && subs.notifications
        && !send_frame(socket, Channel::UserNotification.name(), name, &payload).await
    {
        return false;
    }
    true
}

/// The `Status` JSON of an `update` for `viewer`; `None` when they may not see the post (a block,
/// a mute or muted word, a deletion, followers-only without a follow).
async fn status_frame(
    state: &AppState,
    viewer: Uuid,
    update: &PostEventDto,
) -> Result<Option<String>, AppError> {
    let Some(mut post) = posts::load_one(state.db(), viewer, update.post_id, None).await? else {
        return Ok(None);
    };
    post.entry_id = update.entry_id;
    if let Some(booster) = update.reposted_by {
        let Some(booster) = users::refs(state.db(), &[booster]).await?.remove(&booster) else {
            return Ok(None);
        };
        post.reposted_by = Some(booster);
    }
    let ctx = Ctx::for_posts(state, std::slice::from_ref(&post)).await?;
    Ok(Some(to_json(&dto::status(&post, &ctx))?))
}

/// The `Notification` JSON for `viewer`; `None` when the notification is about a post they may no
/// longer see or has no Mastodon type.
async fn notification_frame(
    state: &AppState,
    viewer: Uuid,
    note: &NotificationEventDto,
) -> Result<Option<String>, AppError> {
    let post = match &note.post {
        Some(preview) => {
            match posts::load_one(state.db(), viewer, preview.id, Some(note.actor.id)).await? {
                Some(post) => Some(post),
                None => return Ok(None),
            }
        }
        None => None,
    };
    let ctx = Ctx::load(state, &post.iter().collect::<Vec<_>>(), &[note.actor.id]).await?;
    let mapped = dto::notification(
        note.id,
        note.kind,
        note.created_at,
        &note.actor,
        post.as_ref(),
        &ctx,
    );
    mapped.map(|mapped| to_json(&mapped)).transpose()
}

fn to_json<T: Serialize>(value: &T) -> Result<String, AppError> {
    serde_json::to_string(value).map_err(|err| AppError::Internal(err.into()))
}

/// Sends one event frame. `false` when the client did not take it in time.
async fn send_frame(socket: &mut WebSocket, stream: &str, event: &str, payload: &str) -> bool {
    let frame = Frame {
        stream: [stream],
        event,
        payload,
    };
    match serde_json::to_string(&frame) {
        Ok(text) => send(socket, Message::Text(text.into())).await,
        Err(err) => {
            tracing::warn!(error = %err, "cannot encode a streaming frame");
            true
        }
    }
}

/// Sends `message`; `false` when the connection is gone or too slow.
async fn send(socket: &mut WebSocket, message: Message) -> bool {
    matches!(
        timeout(SEND_TIMEOUT, socket.send(message)).await,
        Ok(Ok(()))
    )
}

async fn close(socket: &mut WebSocket, code: u16) {
    let frame = CloseFrame {
        code,
        reason: "".into(),
    };
    let _ = timeout(SEND_TIMEOUT, socket.send(Message::Close(Some(frame)))).await;
}
