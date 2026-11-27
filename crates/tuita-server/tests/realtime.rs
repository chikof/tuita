//! `GET /api/v1/stream`: the SSE frames, live delivery, headers and shutdown.
#![allow(missing_docs)]

mod common;

use std::net::SocketAddr;
use std::time::Duration;

use axum::body::{Body, BodyDataStream};
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use futures_util::StreamExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use tuita_server::db::notifications;
use tuita_server::dto::NotificationKindDto;
use uuid::Uuid;

const WAIT: Duration = Duration::from_secs(5);

struct Frames {
    body: BodyDataStream,
    buf: String,
}

impl Frames {
    /// The next `\n\n`-terminated frame, or `None` when the body ended first.
    async fn next(&mut self) -> Option<String> {
        loop {
            if let Some(end) = self.buf.find("\n\n") {
                let frame = self.buf[..end].to_owned();
                self.buf.drain(..end + 2);
                return Some(frame);
            }
            let chunk = tokio::time::timeout(WAIT, self.body.next())
                .await
                .expect("a frame within 5 s")?;
            self.buf
                .push_str(&String::from_utf8_lossy(&chunk.expect("body chunk")));
        }
    }

    /// The next frame that is not a keep-alive comment, as `(event, data)`.
    async fn event(&mut self) -> (String, Value) {
        let frame = self.next().await.expect("the stream is still open");
        let mut name = String::new();
        let mut data = Value::Null;
        for line in frame.lines() {
            if let Some(value) = line.strip_prefix("event: ") {
                name = value.to_owned();
            } else if let Some(value) = line.strip_prefix("data: ") {
                data = serde_json::from_str(value).expect("data is JSON");
            }
        }
        (name, data)
    }
}

async fn open(app: &common::App, token: Option<&str>) -> axum::response::Response {
    let mut request = Request::builder().uri("/api/v1/stream");
    if let Some(token) = token {
        request = request.header("cookie", format!("tuita_session={token}"));
    }
    let mut request = request.body(Body::empty()).expect("request");
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([198, 51, 100, 1], 40000))));
    app.router
        .clone()
        .oneshot(request)
        .await
        .expect("infallible")
}

async fn connect(app: &common::App, person: &common::Person) -> (axum::http::HeaderMap, Frames) {
    let token = person
        .browser
        .cookie("tuita_session")
        .expect("session cookie")
        .to_owned();
    let response = open(app, Some(&token)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let (parts, body) = response.into_parts();
    let frames = Frames {
        body: body.into_data_stream(),
        buf: String::new(),
    };
    (parts.headers, frames)
}

async fn like_notification(db: &PgPool, app: &common::App, user: Uuid, actor: Uuid) -> Uuid {
    let post = Uuid::now_v7();
    sqlx::query("insert into posts (id, author_id, text) values ($1, $2, 'liked post')")
        .bind(post)
        .bind(user)
        .execute(db)
        .await
        .expect("insert post");
    let mut conn = db.acquire().await.expect("connection");
    let pending = notifications::notify(
        &mut conn,
        user,
        actor,
        NotificationKindDto::Like,
        Some(post),
    )
    .await
    .expect("notify")
    .expect("stored");
    notifications::deliver(&app.state, [pending]).await;
    pending.id
}

#[sqlx::test(migrations = "./migrations")]
async fn sends_retry_then_badges_with_the_right_headers(db: PgPool) {
    let app = common::app(db.clone());
    let ana = app.person(&db, "ana").await;
    let (headers, mut frames) = connect(&app, &ana).await;
    assert_eq!(headers["content-type"], "text/event-stream");
    assert_eq!(headers["cache-control"], "no-cache, no-transform");
    assert_eq!(headers["x-accel-buffering"], "no");

    assert_eq!(frames.next().await.as_deref(), Some("retry: 5000"));
    let (name, data) = frames.event().await;
    assert_eq!(name, "badges");
    assert_eq!(
        data,
        json!({"type": "badges", "data": {"notifications": 0, "messages": 0, "reports": null}})
    );
    assert!(app.state.hub().is_listening(ana.id));
}

#[sqlx::test(migrations = "./migrations")]
async fn delivers_a_notification_then_fresh_badges(db: PgPool) {
    let app = common::app(db.clone());
    let ana = app.person(&db, "ana").await;
    let ben = common::insert_user(&db, "ben").await;
    let (_, mut frames) = connect(&app, &ana).await;
    frames.next().await;
    frames.event().await;

    let id = like_notification(&db, &app, ana.id, ben).await;
    let (name, data) = frames.event().await;
    assert_eq!(name, "notification");
    assert_eq!(data["type"], "notification");
    assert_eq!(data["data"]["id"], id.to_string());
    assert_eq!(data["data"]["kind"], "like");
    assert_eq!(data["data"]["actor"]["handle"], "ben");
    assert_eq!(data["data"]["post"]["text"], "liked post");
    let (name, data) = frames.event().await;
    assert_eq!(name, "badges");
    assert_eq!(data["data"]["notifications"], 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn marking_read_pushes_read_and_badges(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let ben = common::insert_user(&db, "ben").await;
    like_notification(&db, &app, ana.id, ben).await;
    let (_, mut frames) = connect(&app, &ana).await;
    frames.next().await;
    let (_, data) = frames.event().await;
    assert_eq!(data["data"]["notifications"], 1);

    ana.browser.post_empty("/notifications/read").await.ok();
    let (name, data) = frames.event().await;
    assert_eq!(
        (name.as_str(), &data),
        (
            "read",
            &json!({"type": "read", "data": {"kind": "notifications"}})
        )
    );
    let (name, data) = frames.event().await;
    assert_eq!(name, "badges");
    assert_eq!(data["data"]["notifications"], 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn other_users_events_stay_private(db: PgPool) {
    let app = common::app(db.clone());
    let ana = app.person(&db, "ana").await;
    let ben = app.person(&db, "ben").await;
    let (_, mut frames) = connect(&app, &ana).await;
    frames.next().await;
    frames.event().await;
    like_notification(&db, &app, ben.id, ana.id).await;
    like_notification(&db, &app, ana.id, ben.id).await;
    let (name, data) = frames.event().await;
    assert_eq!(
        (name.as_str(), &data["data"]["actor"]["handle"]),
        ("notification", &json!("ben"))
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn ends_when_the_server_drains(db: PgPool) {
    let app = common::app(db.clone());
    let ana = app.person(&db, "ana").await;
    let (_, mut frames) = connect(&app, &ana).await;
    frames.next().await;
    frames.event().await;
    app.state.shutdown().cancel();
    assert_eq!(frames.next().await, None);
}

#[sqlx::test(migrations = "./migrations")]
async fn needs_a_session(db: PgPool) {
    let app = common::app(db);
    let response = open(&app, None).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let (_, body) = response.into_parts();
    let bytes = axum::body::to_bytes(body, usize::MAX).await.expect("body");
    assert_eq!(
        serde_json::from_slice::<Value>(&bytes).expect("json"),
        json!({"error": "sign in required"})
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_stream_ends_when_its_session_is_revoked(db: PgPool) {
    let app = common::app(db.clone());
    app.state.set_stream_recheck(Duration::from_millis(100));
    let mut ana = app.person(&db, "ana").await;
    let (_, mut frames) = connect(&app, &ana).await;
    frames.next().await;
    frames.event().await;

    ana.browser.post_empty("/auth/logout").await.ok();
    assert!(
        frames.next().await.is_none(),
        "the stream must end after logout"
    );
    assert!(!app.state.hub().is_listening(ana.id) || app.state.hub().sweep() > 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_stream_ends_when_the_account_is_suspended(db: PgPool) {
    let app = common::app(db.clone());
    app.state.set_stream_recheck(Duration::from_millis(100));
    let ana = app.person(&db, "ana").await;
    let (_, mut frames) = connect(&app, &ana).await;
    frames.next().await;
    frames.event().await;
    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(ana.id)
        .execute(&db)
        .await
        .expect("suspend");
    assert!(frames.next().await.is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn a_user_keeps_at_most_twelve_streams_and_the_oldest_goes(db: PgPool) {
    let app = common::app(db.clone());
    let ana = app.person(&db, "ana").await;
    let mut streams = Vec::new();
    for _ in 0..13 {
        let (_, mut frames) = connect(&app, &ana).await;
        frames.next().await;
        frames.event().await;
        streams.push(frames);
    }
    let mut oldest = streams.remove(0);
    assert!(oldest.next().await.is_none(), "the oldest stream must end");
    let newest = streams.last_mut().expect("newest");
    assert!(
        tokio::time::timeout(Duration::from_millis(300), newest.next())
            .await
            .is_err(),
        "newer streams stay open"
    );
    drop(streams);
    drop(oldest);
}

/// Opens the stream as a script: `Authorization: Bearer`, no cookie.
async fn open_bearer(app: &common::App, secret: &str) -> Frames {
    let mut request = Request::builder()
        .uri("/api/v1/stream")
        .header("authorization", format!("Bearer {secret}"))
        .body(Body::empty())
        .expect("request");
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([198, 51, 100, 1], 40000))));
    let response = app
        .router
        .clone()
        .oneshot(request)
        .await
        .expect("infallible");
    assert_eq!(response.status(), StatusCode::OK);
    Frames {
        body: response.into_body().into_data_stream(),
        buf: String::new(),
    }
}

/// The names of the events that arrive until the first `notification`, inclusive.
async fn names_until_notification(frames: &mut Frames) -> Vec<String> {
    let mut names = Vec::new();
    loop {
        let (name, _) = frames.event().await;
        let done = name == "notification";
        names.push(name);
        if done {
            return names;
        }
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn a_read_token_never_receives_direct_messages_on_the_stream(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let mut secrets = Vec::new();
    for scopes in [json!(["read"]), json!(["read", "messages"])] {
        let made = ana
            .browser
            .post("/tokens", json!({ "name": "s", "scopes": scopes }))
            .await
            .ok();
        secrets.push(made["token"].as_str().expect("token").to_owned());
    }
    let mut read_only = open_bearer(&app, &secrets[0]).await;
    let mut with_messages = open_bearer(&app, &secrets[1]).await;
    for frames in [&mut read_only, &mut with_messages] {
        assert_eq!(frames.next().await.as_deref(), Some("retry: 5000"));
        assert_eq!(frames.event().await.0, "badges");
    }

    bob.browser
        .post("/conversations/ana", json!({ "text": "a private line" }))
        .await
        .ok();
    like_notification(&db, &app, ana.id, bob.id).await;

    let seen = names_until_notification(&mut read_only).await;
    assert!(!seen.iter().any(|name| name == "message"), "{seen:?}");
    let seen = names_until_notification(&mut with_messages).await;
    assert!(seen.iter().any(|name| name == "message"), "{seen:?}");
}
