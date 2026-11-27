//! `GET /api/v1/streaming`: the Mastodon websocket, over a real listener.
//!
//! Actions run through the in-process router of the same `AppState`, so they publish to the hub
//! the socket server listens on.
#![allow(missing_docs)]

mod common;

use std::net::SocketAddr;
use std::time::Duration;

use axum::body::{Body, BodyDataStream};
use axum::extract::ConnectInfo;
use axum::http::Request;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tokio_util::sync::CancellationToken;
use tower::ServiceExt;
use tuita_server::db::live;
use tuita_server::dto::StreamEvent;
use uuid::Uuid;

use common::{App, Person};

const WAIT: Duration = Duration::from_secs(5);
/// How long a test waits to be sure that nothing arrives.
const QUIET: Duration = Duration::from_millis(400);

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// A server started for one test. `stop` must be awaited before the test function returns:
/// `#[sqlx::test]` drops the database right after, and a server still draining (its workers
/// hold the pool open, see `serve_on`) makes that `DROP DATABASE` fail on an open session,
/// which is the root cause of the intermittent `PoolTimedOut` stall in this suite.
struct Server {
    stop: CancellationToken,
    handle: tokio::task::JoinHandle<anyhow::Result<()>>,
}

impl Server {
    /// Cancels the shutdown signal and waits for `serve_on` to finish draining: the listener
    /// closed, every worker joined, the pool connections it held released.
    async fn stop(self) {
        self.stop.cancel();
        self.handle
            .await
            .expect("server task did not panic")
            .expect("serve_on");
    }
}

/// Serves the app on a real listener. Returns its address and the `Server` guard: call
/// `server.stop().await` before the test ends.
async fn listen(app: &App) -> (SocketAddr, Server) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("address");
    let state = app.state.clone();
    let stop = CancellationToken::new();
    let signal = stop.clone();
    let handle = tokio::spawn(tuita_server::serve_on(
        listener,
        state,
        axum::Router::new(),
        async move { signal.cancelled().await },
    ));
    (addr, Server { stop, handle })
}

/// A `read` token of `person`: its secret and id.
async fn token(person: &mut Person) -> (String, String) {
    let created = person
        .browser
        .post("/tokens", json!({ "name": "stream", "scopes": ["read"] }))
        .await
        .ok();
    (
        created["token"].as_str().expect("secret").to_owned(),
        created["info"]["id"].as_str().expect("id").to_owned(),
    )
}

fn url(addr: SocketAddr, query: &str) -> String {
    format!("ws://{addr}/api/v1/streaming?{query}")
}

async fn open(addr: SocketAddr, secret: &str, stream: &str) -> Socket {
    let (socket, _) = connect_async(url(addr, &format!("stream={stream}&access_token={secret}")))
        .await
        .expect("websocket handshake");
    socket
}

/// The next JSON frame; pings and pongs are skipped. `None` when the socket ended or `within`
/// passed.
async fn frame_within(socket: &mut Socket, within: Duration) -> Option<Value> {
    let deadline = tokio::time::Instant::now() + within;
    loop {
        let next = tokio::time::timeout_at(deadline, socket.next())
            .await
            .ok()?;
        match next? {
            Ok(Message::Text(text)) => {
                return Some(serde_json::from_str(text.as_str()).expect("JSON"));
            }
            Ok(Message::Ping(_) | Message::Pong(_)) => {}
            Ok(Message::Close(_)) | Err(_) => return None,
            Ok(other) => panic!("unexpected frame {other:?}"),
        }
    }
}

async fn frame(socket: &mut Socket) -> Value {
    frame_within(socket, WAIT)
        .await
        .expect("a frame within 5 s")
}

async fn silent(socket: &mut Socket) {
    assert_eq!(frame_within(socket, QUIET).await, None, "no frame expected");
}

/// Whether the socket ends (close frame or EOF) within `within`.
async fn ends_within(socket: &mut Socket, within: Duration) -> bool {
    let deadline = tokio::time::Instant::now() + within;
    loop {
        match tokio::time::timeout_at(deadline, socket.next()).await {
            Err(_) => return false,
            Ok(None | Some(Err(_) | Ok(Message::Close(_)))) => return true,
            Ok(Some(Ok(_))) => {}
        }
    }
}

fn payload(frame: &Value) -> Value {
    serde_json::from_str(frame["payload"].as_str().expect("payload is a string"))
        .expect("payload is JSON")
}

async fn follow(follower: &mut Person, target: &Person) {
    follower
        .browser
        .post_empty(&format!("/users/{}/follow", target.handle))
        .await
        .ok();
}

fn id_of(post: &Value) -> String {
    post["id"].as_str().expect("post id").to_owned()
}

#[sqlx::test(migrations = "./migrations")]
async fn a_follower_hears_update_delete_and_notification(db: PgPool) {
    let app = common::app(db.clone());
    let (addr, server) = listen(&app).await;
    let mut ana = app.person(&db, "ana").await;
    let mut ben = app.person(&db, "ben").await;
    follow(&mut ana, &ben).await;
    let (secret, _) = token(&mut ana).await;
    let mut socket = open(addr, &secret, "user").await;

    let post = ben.browser.post_text("hello from ben").await;
    let update = frame(&mut socket).await;
    assert_eq!(update["stream"], json!(["user"]));
    assert_eq!(update["event"], "update");
    let status = payload(&update);
    assert_eq!(status["id"], post["id"]);
    assert_eq!(status["account"]["acct"], "ben");
    assert_eq!(status["content"], "<p>hello from ben</p>");
    assert_eq!(status["visibility"], "public");
    assert_eq!(status["favourited"], false);

    ben.browser
        .delete(&format!("/posts/{}", id_of(&post)))
        .await
        .ok();
    let delete = frame(&mut socket).await;
    assert_eq!(delete["event"], "delete");
    assert_eq!(delete["payload"], post["id"]);

    let own = ana.browser.post_text("my own post").await;
    let update = frame(&mut socket).await;
    assert_eq!(update["event"], "update");
    assert_eq!(payload(&update)["id"], own["id"]);

    ben.browser
        .post_empty(&format!("/posts/{}/like", id_of(&own)))
        .await
        .ok();
    let note = frame(&mut socket).await;
    assert_eq!(note["stream"], json!(["user"]));
    assert_eq!(note["event"], "notification");
    let note = payload(&note);
    assert_eq!(note["type"], "favourite");
    assert_eq!(note["account"]["acct"], "ben");
    assert_eq!(note["status"]["id"], own["id"]);
    assert_eq!(note["status"]["favourites_count"], 1);
    silent(&mut socket).await;
    server.stop().await;
}

#[sqlx::test(migrations = "./migrations")]
async fn a_boost_arrives_as_a_wrapper_and_leaves_with_the_unboost(db: PgPool) {
    let app = common::app(db.clone());
    let (addr, server) = listen(&app).await;
    let mut ana = app.person(&db, "ana").await;
    let mut ben = app.person(&db, "ben").await;
    let mut cy = app.person(&db, "cy").await;
    follow(&mut ana, &cy).await;
    let (secret, _) = token(&mut ana).await;
    let mut socket = open(addr, &secret, "user").await;

    let post = ben.browser.post_text("ben's words").await;
    silent(&mut socket).await;
    cy.browser
        .post_empty(&format!("/posts/{}/repost", id_of(&post)))
        .await
        .ok();
    let update = frame(&mut socket).await;
    assert_eq!(update["event"], "update");
    let wrapper = payload(&update);
    assert_ne!(wrapper["id"], post["id"]);
    assert_eq!(wrapper["account"]["acct"], "cy");
    assert_eq!(wrapper["reblog"]["id"], post["id"]);
    assert_eq!(wrapper["reblog"]["account"]["acct"], "ben");

    cy.browser
        .delete(&format!("/posts/{}/repost", id_of(&post)))
        .await
        .ok();
    let delete = frame(&mut socket).await;
    assert_eq!(delete["event"], "delete");
    assert_eq!(delete["payload"], wrapper["id"]);
    server.stop().await;
}

#[sqlx::test(migrations = "./migrations")]
async fn strangers_replies_muted_and_blocked_authors_stay_out(db: PgPool) {
    let app = common::app(db.clone());
    let (addr, server) = listen(&app).await;
    let mut ana = app.person(&db, "ana").await;
    let mut ben = app.person(&db, "ben").await;
    let mut cy = app.person(&db, "cy").await;
    let mut dee = app.person(&db, "dee").await;
    follow(&mut ana, &ben).await;
    follow(&mut ana, &cy).await;
    let (secret, _) = token(&mut ana).await;
    let mut socket = open(addr, &secret, "user").await;

    dee.browser.post_text("a stranger").await;
    let root = cy.browser.post_text("cy root").await;
    let first = frame(&mut socket).await;
    assert_eq!(
        payload(&first)["id"],
        root["id"],
        "only followed authors, in order"
    );
    cy.browser
        .post(
            "/posts",
            json!({ "text": "a reply", "replyTo": id_of(&root) }),
        )
        .await
        .ok();
    ana.browser.post_empty("/users/ben/mute").await.ok();
    ben.browser.post_text("muted ben").await;
    let marker = cy.browser.post_text("marker").await;
    let next = frame(&mut socket).await;
    assert_eq!(
        payload(&next)["id"],
        marker["id"],
        "replies and muted authors are skipped"
    );

    // The publisher found the follower; the load for the viewer is what keeps a blocked author out.
    let blocked = common::insert_user(&db, "bad").await;
    let post = Uuid::now_v7();
    sqlx::query("insert into posts (id, author_id, text) values ($1, $2, 'blocked words')")
        .bind(post)
        .bind(blocked)
        .execute(&db)
        .await
        .expect("insert post");
    sqlx::query("insert into blocks (user_id, target_id) values ($1, $2)")
        .bind(ana.id)
        .bind(blocked)
        .execute(&db)
        .await
        .expect("insert block");
    app.state.hub().publish(
        ana.id,
        StreamEvent::Update(Box::new(tuita_server::dto::PostEventDto {
            post_id: post,
            entry_id: post,
            reposted_by: None,
        })),
    );
    silent(&mut socket).await;
    server.stop().await;
}

#[sqlx::test(migrations = "./migrations")]
async fn direct_posts_reach_no_timeline(db: PgPool) {
    let app = common::app(db.clone());
    let (addr, server) = listen(&app).await;
    let mut ana = app.person(&db, "ana").await;
    let ben = app.person(&db, "ben").await;
    follow(&mut ana, &ben).await;
    let (secret, _) = token(&mut ana).await;
    let mut socket = open(addr, &secret, "user").await;

    let post = Uuid::now_v7();
    sqlx::query(
        "insert into posts (id, author_id, text, visibility) values ($1, $2, 'psst', 'direct')",
    )
    .bind(post)
    .bind(ben.id)
    .execute(&db)
    .await
    .expect("insert post");
    live::posts_created(&app.state, &[post]).await;
    silent(&mut socket).await;
    server.stop().await;
}

#[sqlx::test(migrations = "./migrations")]
async fn the_token_is_checked_before_the_upgrade(db: PgPool) {
    let app = common::app(db.clone());
    let (addr, server) = listen(&app).await;
    let mut ana = app.person(&db, "ana").await;
    let (secret, _) = token(&mut ana).await;

    let status = |result: Result<_, Error>| match result {
        Err(Error::Http(response)) => response.status().as_u16(),
        Err(other) => panic!("unexpected error {other:?}"),
        Ok(_) => 101,
    };
    let bad = connect_async(url(addr, "stream=user&access_token=tuita_nope")).await;
    assert_eq!(status(bad), 401, "a bad token");
    let none = connect_async(url(addr, "stream=user")).await;
    assert_eq!(status(none), 401, "no token");

    let cookie = ana
        .browser
        .cookie("tuita_session")
        .expect("session cookie")
        .to_owned();
    let mut request = url(addr, "stream=user")
        .into_client_request()
        .expect("request");
    request.headers_mut().insert(
        "cookie",
        format!("tuita_session={cookie}").parse().expect("header"),
    );
    assert_eq!(
        status(connect_async(request).await),
        401,
        "a session cookie alone"
    );

    let mut request = url(addr, "stream=user")
        .into_client_request()
        .expect("request");
    request.headers_mut().insert(
        "authorization",
        format!("Bearer {secret}").parse().expect("header"),
    );
    assert_eq!(
        status(connect_async(request).await),
        101,
        "the Authorization header"
    );

    let mut request = url(addr, "").into_client_request().expect("request");
    request
        .headers_mut()
        .insert("sec-websocket-protocol", secret.parse().expect("header"));
    let (mut socket, response) = connect_async(request).await.expect("subprotocol token");
    assert_eq!(
        response.headers()["sec-websocket-protocol"],
        secret.as_str()
    );
    socket
        .send(Message::text(r#"{"type":"subscribe","stream":"user"}"#))
        .await
        .expect("send");
    ana.browser.post_text("heard over the subprotocol").await;
    let update = frame(&mut socket).await;
    assert_eq!(update["event"], "update");
    server.stop().await;
}

#[sqlx::test(migrations = "./migrations")]
async fn clients_subscribe_unsubscribe_and_unknown_streams_get_an_error(db: PgPool) {
    let app = common::app(db.clone());
    let (addr, server) = listen(&app).await;
    let mut ana = app.person(&db, "ana").await;
    let (secret, _) = token(&mut ana).await;
    let mut socket = open(addr, &secret, "public").await;

    let error = frame(&mut socket).await;
    assert_eq!(error["event"], "error");
    assert_eq!(error["payload"], "Unknown stream type");
    ana.browser.post_text("nobody asked for this yet").await;
    silent(&mut socket).await;

    socket
        .send(Message::text(
            r#"{"type":"subscribe","stream":"hashtag","tag":"x"}"#,
        ))
        .await
        .expect("send");
    assert_eq!(frame(&mut socket).await["event"], "error");
    socket.send(Message::text("not json")).await.expect("send");
    socket
        .send(Message::binary(vec![1, 2, 3]))
        .await
        .expect("send");

    socket
        .send(Message::text(
            r#"{"type":"subscribe","stream":"user:notification"}"#,
        ))
        .await
        .expect("send");
    ana.browser.post_text("still no updates").await;
    silent(&mut socket).await;
    let mut ben = app.person(&db, "ben").await;
    let own = ana.browser.post_text("likeable").await;
    ben.browser
        .post_empty(&format!("/posts/{}/like", id_of(&own)))
        .await
        .ok();
    let note = frame(&mut socket).await;
    assert_eq!(note["stream"], json!(["user:notification"]));
    assert_eq!(note["event"], "notification");

    socket
        .send(Message::text(
            r#"{"type":"unsubscribe","stream":"user:notification"}"#,
        ))
        .await
        .expect("send");
    ben.browser
        .post_empty(&format!("/posts/{}/repost", id_of(&own)))
        .await
        .ok();
    silent(&mut socket).await;
    server.stop().await;
}

#[sqlx::test(migrations = "./migrations")]
async fn a_message_over_the_limit_closes_the_socket(db: PgPool) {
    let app = common::app(db.clone());
    let (addr, server) = listen(&app).await;
    let mut ana = app.person(&db, "ana").await;
    let (secret, _) = token(&mut ana).await;
    let mut socket = open(addr, &secret, "user").await;
    socket
        .send(Message::text("x".repeat(64 * 1024)))
        .await
        .expect("send");
    assert!(ends_within(&mut socket, WAIT).await);
    tokio::time::sleep(QUIET).await;
    assert!(!app.state.hub().is_listening(ana.id));
    server.stop().await;
}

#[sqlx::test(migrations = "./migrations")]
async fn revoking_the_token_closes_the_socket_within_the_recheck_interval(db: PgPool) {
    let app = common::app(db.clone());
    app.state.set_stream_recheck(Duration::from_millis(100));
    let (addr, server) = listen(&app).await;
    let mut ana = app.person(&db, "ana").await;
    let (secret, id) = token(&mut ana).await;
    let mut socket = open(addr, &secret, "user").await;
    silent(&mut socket).await;

    ana.browser.delete(&format!("/tokens/{id}")).await.ok();
    assert!(ends_within(&mut socket, Duration::from_secs(2)).await);
    assert!(!app.state.hub().is_listening(ana.id));
    server.stop().await;
}

#[sqlx::test(migrations = "./migrations")]
async fn a_silent_client_is_dropped_after_missed_pings(db: PgPool) {
    let app = common::app(db.clone());
    app.state.set_stream_recheck(Duration::from_millis(100));
    let (addr, server) = listen(&app).await;
    let mut ana = app.person(&db, "ana").await;
    let (secret, _) = token(&mut ana).await;
    // The client never reads, so it never answers a ping.
    let mut socket = open(addr, &secret, "user").await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(!app.state.hub().is_listening(ana.id));
    assert!(ends_within(&mut socket, WAIT).await);
    server.stop().await;
}

#[sqlx::test(migrations = "./migrations")]
async fn a_client_that_answers_pings_stays_connected(db: PgPool) {
    let app = common::app(db.clone());
    app.state.set_stream_recheck(Duration::from_millis(250));
    let (addr, server) = listen(&app).await;
    let mut ana = app.person(&db, "ana").await;
    let (secret, _) = token(&mut ana).await;
    let mut socket = open(addr, &secret, "user").await;
    // Reading answers the pings; for twice the idle limit (750 ms) there is nothing but pings.
    silent_for(&mut socket, Duration::from_millis(1600)).await;
    ana.browser.post_text("still here").await;
    assert_eq!(frame(&mut socket).await["event"], "update");
    server.stop().await;
}

async fn silent_for(socket: &mut Socket, within: Duration) {
    assert_eq!(frame_within(socket, within).await, None);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_thirteenth_connection_evicts_the_first(db: PgPool) {
    let app = common::app(db.clone());
    let (addr, server) = listen(&app).await;
    let mut ana = app.person(&db, "ana").await;
    let (secret, _) = token(&mut ana).await;
    let mut sockets = Vec::new();
    for _ in 0..12 {
        sockets.push(open(addr, &secret, "user").await);
    }
    let mut newest = open(addr, &secret, "user").await;
    assert!(
        ends_within(&mut sockets[0], WAIT).await,
        "the oldest is closed"
    );
    for socket in &mut sockets[1..] {
        silent(socket).await;
    }
    ana.browser.post_text("to the survivors").await;
    assert_eq!(frame(&mut newest).await["event"], "update");
    assert_eq!(frame(&mut sockets[11]).await["event"], "update");
    server.stop().await;
}

#[sqlx::test(migrations = "./migrations")]
async fn the_sse_stream_never_carries_the_mastodon_events(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let token = ana
        .browser
        .cookie("tuita_session")
        .expect("session cookie")
        .to_owned();
    let mut request = Request::builder()
        .uri("/api/v1/stream")
        .header("cookie", format!("tuita_session={token}"))
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
    let mut body: BodyDataStream = response.into_body().into_data_stream();
    let seen = drain(&mut body).await;
    assert!(seen.contains("event: badges"));

    let post = Uuid::now_v7();
    app.state.hub().publish(
        ana.id,
        StreamEvent::Update(Box::new(tuita_server::dto::PostEventDto {
            post_id: post,
            entry_id: post,
            reposted_by: None,
        })),
    );
    app.state.hub().publish(ana.id, StreamEvent::Delete(post));
    let mut ben = app.person(&db, "ben").await;
    let own = ana.browser.post_text("liked").await;
    ben.browser
        .post_empty(&format!("/posts/{}/like", id_of(&own)))
        .await
        .ok();
    let after = drain(&mut body).await;
    assert!(after.contains("event: notification"), "{after}");
    assert!(!after.contains("event: update"), "{after}");
    assert!(!after.contains("event: delete"), "{after}");
}

/// Everything the SSE body sends until it has been quiet for half a second.
async fn drain(body: &mut BodyDataStream) -> String {
    let mut seen = String::new();
    while let Ok(Some(chunk)) = tokio::time::timeout(Duration::from_millis(500), body.next()).await
    {
        seen.push_str(&String::from_utf8_lossy(&chunk.expect("chunk")));
    }
    seen
}

// ------------------------------------------------------------------ remote accounts and staff

/// A remote account stored without any network traffic, with the key that signs its deliveries.
struct Remote {
    id: String,
    uid: Uuid,
    key_id: String,
    key: tuita_core::ap::PrivateKey,
}

const REMOTE: &str = "remote.example";

impl Remote {
    async fn stored(db: &PgPool, name: &str) -> Self {
        let pair = tuita_core::ap::generate_keypair().expect("keypair");
        let id = format!("https://{REMOTE}/users/{name}");
        let profile = tuita_server::db::remote::RemoteProfile {
            handle: name.to_owned(),
            domain: REMOTE.to_owned(),
            display_name: name.to_owned(),
            bio: String::new(),
            website: String::new(),
            is_bot: false,
            is_org: false,
            locked: false,
            discoverable: true,
            inbox_url: format!("{id}/inbox"),
            shared_inbox_url: None,
            outbox_url: None,
            followers_url: Some(format!("{id}/followers")),
            featured_url: None,
            url: None,
            public_key_pem: pair.public_pem.clone(),
            created_at: None,
            ap_id: id.clone(),
        };
        let tuita_server::db::remote::Upsert::Stored(uid) =
            tuita_server::db::remote::upsert_actor(db, &profile)
                .await
                .expect("upsert")
        else {
            panic!("actor not stored");
        };
        Self {
            key_id: format!("{id}#main-key"),
            key: tuita_core::ap::PrivateKey::from_pem(&pair.private_pem).expect("private key"),
            id,
            uid,
        }
    }

    fn note_id(n: &str) -> String {
        format!("https://{REMOTE}/users/bob/statuses/{n}")
    }

    fn note(&self, n: &str, content: &str) -> Value {
        json!({
            "id": Self::note_id(n),
            "type": "Note",
            "attributedTo": self.id,
            "content": format!("<p>{content}</p>"),
            "published": tuita_core::ap::format_time(time::OffsetDateTime::now_utc()),
            "to": [tuita_core::ap::PUBLIC],
            "cc": [],
        })
    }

    /// Delivers `kind` of `object` to the shared inbox, signed, and expects `202`.
    async fn deliver(&self, app: &App, kind: &str, object: Value) {
        let activity = json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": format!("https://{REMOTE}/activities/{}", Uuid::now_v7()),
            "type": kind,
            "actor": self.id,
            "object": object,
        });
        let body = serde_json::to_vec(&activity).expect("json");
        let url = url::Url::parse(&format!("{}/inbox", common::ORIGIN)).expect("url");
        let headers: Vec<(String, String)> = tuita_core::ap::sign_post(
            &self.key,
            &self.key_id,
            &url,
            &body,
            time::OffsetDateTime::now_utc(),
        )
        .expect("signs")
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value))
        .collect();
        let view: Vec<(&str, &str)> = headers
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect();
        let reply = app
            .browser()
            .send_raw(axum::http::Method::POST, "/inbox", &view, Body::from(body))
            .await;
        assert_eq!(
            reply.status,
            axum::http::StatusCode::ACCEPTED,
            "{}",
            String::from_utf8_lossy(&reply.bytes)
        );
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn a_followed_remote_accounts_note_and_its_deletion_stream(db: PgPool) {
    let app = common::app(db.clone());
    let (addr, server) = listen(&app).await;
    let mut ana = app.person(&db, "ana").await;
    let bob = Remote::stored(&db, "bob").await;
    sqlx::query(
        "insert into follows (follower_id, followee_id, state, ap_id) values ($1, $2, 'accepted', 'https://x.example/f')",
    )
    .bind(ana.id)
    .bind(bob.uid)
    .execute(&db)
    .await
    .expect("follow");
    let (secret, _) = token(&mut ana).await;
    let mut socket = open(addr, &secret, "user").await;

    bob.deliver(&app, "Create", bob.note("1", "hello from afar"))
        .await;
    let update = frame(&mut socket).await;
    assert_eq!(update["event"], "update");
    let status = payload(&update);
    assert_eq!(status["account"]["acct"], format!("bob@{REMOTE}"));
    assert_eq!(status["content"], "<p>hello from afar</p>");
    let id = status["id"].as_str().expect("status id").to_owned();

    bob.deliver(
        &app,
        "Delete",
        json!({"id": Remote::note_id("1"), "type": "Tombstone"}),
    )
    .await;
    let delete = frame(&mut socket).await;
    assert_eq!(delete["event"], "delete");
    assert_eq!(delete["payload"], id.as_str());
    server.stop().await;
}

#[sqlx::test(migrations = "./migrations")]
async fn a_staff_deletion_streams_a_delete(db: PgPool) {
    let app = common::app(db.clone());
    let (addr, server) = listen(&app).await;
    let mut ana = app.person(&db, "ana").await;
    let mut ben = app.person(&db, "ben").await;
    let mut cy = app.person(&db, "cy").await;
    let mut moderator = app.person(&db, "moddy").await;
    sqlx::query("update users set role = 'moderator', password_hash = 'x' where id = $1")
        .bind(moderator.id)
        .execute(&db)
        .await
        .expect("role");
    follow(&mut ana, &ben).await;
    follow(&mut ana, &cy).await;
    let (secret, _) = token(&mut ana).await;
    let mut socket = open(addr, &secret, "user").await;

    let post = ben.browser.post_text("something awful").await;
    assert_eq!(frame(&mut socket).await["event"], "update");
    cy.browser
        .post(
            "/reports",
            json!({ "postId": id_of(&post), "reason": "harassment", "note": "" }),
        )
        .await
        .ok();
    let queue = moderator.browser.get("/admin/reports").await.ok();
    let report = queue["items"][0]["id"]
        .as_str()
        .expect("report id")
        .to_owned();
    moderator
        .browser
        .post(
            &format!("/admin/reports/{report}/resolve"),
            json!({ "action": "delete_post" }),
        )
        .await
        .ok();
    let delete = frame(&mut socket).await;
    assert_eq!(delete["event"], "delete");
    assert_eq!(delete["payload"], post["id"]);

    // An admin purge of everything an account posted announces each post too.
    let mut admin = app.person(&db, "boss").await;
    sqlx::query("update users set role = 'admin', password_hash = 'x' where id = $1")
        .bind(admin.id)
        .execute(&db)
        .await
        .expect("role");
    let one = cy.browser.post_text("one").await;
    let two = cy.browser.post_text("two").await;
    assert_eq!(frame(&mut socket).await["event"], "update");
    assert_eq!(frame(&mut socket).await["event"], "update");
    admin.browser.delete("/admin/users/cy/posts").await.ok();
    let mut gone = vec![
        frame(&mut socket).await["payload"].clone(),
        frame(&mut socket).await["payload"].clone(),
    ];
    gone.sort_by_key(ToString::to_string);
    let mut want = vec![one["id"].clone(), two["id"].clone()];
    want.sort_by_key(ToString::to_string);
    assert_eq!(gone, want);
    server.stop().await;
}
