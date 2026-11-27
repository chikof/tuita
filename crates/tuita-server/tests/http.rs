//! HTTP tests: in-process via `oneshot`, plus one real socket with graceful shutdown.
//! Each `#[sqlx::test]` gets a fresh database; `DATABASE_URL` must point at a running Postgres.

use std::time::Duration;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{HeaderMap, Request, StatusCode};
use sqlx::PgPool;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tower::ServiceExt;

use tuita_server::{AppState, CONTENT_SECURITY_POLICY, Config, router, serve_on};

const TIMEOUT: Duration = Duration::from_secs(5);

fn stub_web() -> Router {
    Router::new().fallback(|| async { "web" })
}

fn state(db: PgPool) -> AppState {
    let config = Config::from_lookup(|var| match var {
        "DATABASE_URL" => Some("postgres://unused".to_owned()),
        "PUBLIC_URL" => Some("https://social.example".to_owned()),
        _ => None,
    })
    .expect("valid test config");
    let media = tuita_server::media::MediaStore::from_config(&config).expect("media store");
    AppState::new(db, config, media)
}

fn ready_app(db: PgPool) -> Router {
    let state = state(db);
    state.set_ready(true);
    router(state, stub_web())
}

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: String,
}

impl Reply {
    fn header(&self, name: &str) -> &str {
        self.headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.body).unwrap_or(serde_json::Value::Null)
    }
}

async fn send(app: Router, request: Request<Body>) -> Reply {
    let response = app.oneshot(request).await.expect("router is infallible");
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.expect("body reads");
    Reply {
        status: parts.status,
        headers: parts.headers,
        body: String::from_utf8_lossy(&bytes).into_owned(),
    }
}

async fn get(app: Router, uri: &str) -> Reply {
    let request = Request::get(uri)
        .body(Body::empty())
        .expect("valid request");
    send(app, request).await
}

#[sqlx::test(migrations = "./migrations")]
async fn healthz_is_ok_even_when_not_ready(db: PgPool) {
    let reply = get(router(state(db), stub_web()), "/healthz").await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.json()["status"], "ok");
}

#[sqlx::test(migrations = "./migrations")]
async fn readyz_follows_state_and_database(db: PgPool) {
    let state = state(db.clone());
    let app = router(state.clone(), stub_web());

    let reply = get(app.clone(), "/readyz").await;
    assert_eq!(reply.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(reply.json()["status"], "not ready");

    state.set_ready(true);
    let reply = get(app.clone(), "/readyz").await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.json()["status"], "ready");

    db.close().await;
    let reply = get(app, "/readyz").await;
    assert_eq!(reply.status, StatusCode::SERVICE_UNAVAILABLE);
}

#[sqlx::test(migrations = "./migrations")]
async fn backend_paths_are_json_404(db: PgPool) {
    for uri in ["/api/v1/nope", "/users/ada"] {
        let reply = get(ready_app(db.clone()), uri).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{uri}");
        assert_eq!(reply.json()["error"], "not found", "{uri}");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn backend_paths_stay_json_404_for_any_method(db: PgPool) {
    let request = Request::post("/users/ada/nope")
        .body(Body::empty())
        .expect("valid request");
    let reply = send(ready_app(db), request).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.json()["error"], "not found");
}

#[sqlx::test(migrations = "./migrations")]
async fn every_response_has_security_headers(db: PgPool) {
    for uri in ["/healthz", "/@ada", "/api/nope"] {
        let reply = get(ready_app(db.clone()), uri).await;
        assert_eq!(reply.header("x-content-type-options"), "nosniff", "{uri}");
        assert_eq!(reply.header("x-frame-options"), "DENY", "{uri}");
        assert_eq!(
            reply.header("referrer-policy"),
            "strict-origin-when-cross-origin",
            "{uri}"
        );
        assert_eq!(
            reply.header("permissions-policy"),
            "camera=(), microphone=(), geolocation=()",
            "{uri}"
        );
        assert_eq!(
            reply.header("content-security-policy"),
            CONTENT_SECURITY_POLICY,
            "{uri}"
        );
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn request_id_is_set_and_propagated(db: PgPool) {
    let reply = get(ready_app(db.clone()), "/healthz").await;
    assert!(
        !reply.header("x-request-id").is_empty(),
        "generated id missing"
    );

    let request = Request::get("/healthz")
        .header("x-request-id", "abc-123")
        .body(Body::empty())
        .expect("valid request");
    let reply = send(ready_app(db), request).await;
    assert_eq!(reply.header("x-request-id"), "abc-123");
}

#[sqlx::test(migrations = "./migrations")]
async fn unmatched_paths_reach_the_web_service(db: PgPool) {
    for (method, uri) in [
        ("GET", "/nope"),
        ("POST", "/login"),
        ("GET", "/oauth/authorize"),
    ] {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .expect("valid request");
        let reply = send(ready_app(db.clone()), request).await;
        assert_eq!(reply.body, "web", "{method} {uri}");
    }
    let request = Request::post("/api/nope")
        .body(Body::empty())
        .expect("valid request");
    let reply = send(ready_app(db), request).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.json()["error"], "not found");
}

#[sqlx::test(migrations = "./migrations")]
async fn serves_over_tcp_and_shuts_down_cleanly(db: PgPool) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("local addr");
    let (stop, stopped) = oneshot::channel::<()>();
    let state = state(db);
    let server = tokio::spawn(serve_on(listener, state, stub_web(), async {
        // A dropped sender also counts as a shutdown request.
        let _ = stopped.await;
    }));

    let mut stream = TcpStream::connect(addr).await.expect("connect");
    stream
        .write_all(b"GET /readyz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .expect("send request");
    let mut raw = Vec::new();
    tokio::time::timeout(TIMEOUT, stream.read_to_end(&mut raw))
        .await
        .expect("response in time")
        .expect("read response");
    let text = String::from_utf8_lossy(&raw);
    assert!(
        text.starts_with("HTTP/1.1 200"),
        "unexpected response: {text}"
    );
    assert!(
        text.contains(r#"{"status":"ready"}"#),
        "unexpected body: {text}"
    );

    stop.send(()).expect("server still running");
    let result = tokio::time::timeout(TIMEOUT, server)
        .await
        .expect("server stops after shutdown")
        .expect("server task does not panic");
    assert!(result.is_ok(), "serve returned {result:?}");
    assert!(
        TcpStream::connect(addr).await.is_err(),
        "listener still accepting"
    );
}

async fn og(state: &AppState, path: &str) -> String {
    tuita_server::routes::head::head_tags(state, path).await
}

#[sqlx::test(migrations = "./migrations")]
async fn profile_and_post_pages_get_real_og_tags(db: PgPool) {
    let user = uuid::Uuid::now_v7();
    sqlx::query("insert into users (id, handle, display_name, bio) values ($1, 'Ada_B', 'Ada <Moreno>', 'Makes \"pixels\" behave')")
        .bind(user)
        .execute(&db)
        .await
        .expect("user");
    let post = uuid::Uuid::now_v7();
    let text = "x".repeat(300);
    sqlx::query("insert into posts (id, author_id, text) values ($1, $2, $3)")
        .bind(post)
        .bind(user)
        .bind(&text)
        .execute(&db)
        .await
        .expect("post");
    let cw_post = uuid::Uuid::now_v7();
    sqlx::query(
        "insert into posts (id, author_id, text, cw) values ($1, $2, 'secret', 'spoilers')",
    )
    .bind(cw_post)
    .bind(user)
    .execute(&db)
    .await
    .expect("cw post");

    let state = state(db.clone());
    let body = og(&state, "/@ada_b").await;
    assert!(
        body.contains(r#"<meta property="og:type" content="profile">"#),
        "{body}"
    );
    assert!(
        body.contains(r#"<meta property="og:title" content="Ada &lt;Moreno&gt; (@Ada_B)">"#),
        "{body}"
    );
    assert!(
        body.contains(
            r#"<meta property="og:description" content="Makes &quot;pixels&quot; behave">"#
        ),
        "{body}"
    );
    assert_eq!(body.matches("og:title").count(), 1);

    let body = og(&state, &format!("/@ada_b/{post}")).await;
    assert!(
        body.contains(r#"<meta property="og:type" content="article">"#),
        "{body}"
    );
    assert!(
        body.contains(r#"content="Ada &lt;Moreno&gt; on tuita""#),
        "{body}"
    );
    assert!(
        body.contains(&format!(r#"content="{}""#, "x".repeat(200))),
        "{body}"
    );
    assert!(!body.contains(&"x".repeat(201)));

    let body = og(&state, &format!("/@ada_b/{cw_post}")).await;
    assert!(
        body.contains(r#"content="spoilers""#) && !body.contains("secret"),
        "{body}"
    );

    // Someone else's handle with this post id, and a bad id, fall back to the generic tags.
    for uri in [
        format!("/@nobody/{post}"),
        "/@ada_b/not-a-uuid".to_owned(),
        "/@ada_b/followers".to_owned(),
    ] {
        let body = og(&state, &uri).await;
        assert!(
            body.contains(r#"<meta property="og:type" content="website">"#),
            "{uri}: {body}"
        );
    }

    // Deleted posts and suspended accounts never leak.
    sqlx::query("update posts set deleted_at = now(), text = '' where id = $1")
        .bind(post)
        .execute(&db)
        .await
        .expect("delete");
    let body = og(&state, &format!("/@ada_b/{post}")).await;
    assert!(body.contains(r#"content="website""#), "{body}");
    sqlx::query("update users set suspended_at = now()")
        .execute(&db)
        .await
        .expect("suspend");
    let body = og(&state, "/@ada_b").await;
    assert!(
        body.contains(r#"<meta property="og:type" content="website">"#),
        "{body}"
    );
    assert!(!body.contains("Moreno"), "{body}");
}

#[sqlx::test(migrations = "./migrations")]
async fn generic_tags_use_the_instance_settings(db: PgPool) {
    sqlx::query("insert into instance_settings (key, value) values ('name', '\"Quiet corner\"'), ('description', '\"A calm place\"')")
        .execute(&db)
        .await
        .expect("settings");
    let body = og(&state(db), "/explore").await;
    assert!(
        body.contains(r#"<meta property="og:site_name" content="Quiet corner">"#),
        "{body}"
    );
    assert!(
        body.contains(r#"<meta property="og:description" content="A calm place">"#),
        "{body}"
    );
}
