//! Shared helpers for the API tests: an in-process app and a cookie-keeping fake browser.
//!
//! Each test file declares `mod common;` and uses what it needs.
#![allow(dead_code, unreachable_pub, missing_docs)]

use std::collections::BTreeMap;
use std::net::{IpAddr, SocketAddr};

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, Method, Request, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use tuita_server::media::MediaStore;
use tuita_server::{AppState, Config, router};

/// The origin the test instance believes it lives at.
pub const ORIGIN: &str = "https://social.example";
/// A password that passes the policy.
pub const PASSWORD: &str = "correct horse battery";

/// Stands in for the web client: answers `web` to everything the backend does not match.
pub fn stub_web() -> Router {
    Router::new().fallback(|| async { "web" })
}

/// A ready app over `db`.
pub struct App {
    pub state: AppState,
    pub router: Router,
}

pub fn app(db: PgPool) -> App {
    app_with(db, |_| None)
}

/// Like [`app`], with extra environment variables for the config.
pub fn app_with(db: PgPool, extra: impl Fn(&str) -> Option<String>) -> App {
    app_on(db, None, extra)
}

/// Like [`app_with`], over `media` instead of the store the config asks for.
pub fn app_on(
    db: PgPool,
    media: Option<MediaStore>,
    extra: impl Fn(&str) -> Option<String>,
) -> App {
    let config = Config::from_lookup(|var| match var {
        "DATABASE_URL" => Some("postgres://unused".to_owned()),
        "PUBLIC_URL" => Some(ORIGIN.to_owned()),
        // Most suites send plain DMs between accounts without keys; the sealed rules have their
        // own suite, which asks for the production default explicitly.
        "MESSAGES_E2EE_REQUIRED" => extra(var).or_else(|| Some("false".to_owned())),
        other => extra(other),
    })
    .expect("valid test config");
    let media = media.unwrap_or_else(|| MediaStore::from_config(&config).expect("media store"));
    let state = AppState::new(db, config, media);
    state.set_ready(true);
    let router = router(state.clone(), stub_web());
    App { state, router }
}

pub struct Reply {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: String,
}

impl Reply {
    pub fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or(Value::Null)
    }

    pub fn header(&self, name: &str) -> &str {
        self.headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
    }

    /// The `name=value` pairs of every `Set-Cookie` header; an ended cookie has an empty value.
    pub fn set_cookies(&self) -> Vec<(String, String)> {
        self.headers
            .get_all("set-cookie")
            .iter()
            .filter_map(|value| value.to_str().ok())
            .filter_map(|raw| {
                let first = raw.split(';').next()?;
                let (name, value) = first.split_once('=')?;
                Some((name.to_owned(), value.to_owned()))
            })
            .collect()
    }

    /// Asserts the status and returns the JSON body.
    #[track_caller]
    pub fn ok(&self) -> Value {
        assert_eq!(self.status, StatusCode::OK, "body: {}", self.body);
        self.json()
    }

    /// Asserts the status and the `error` message.
    #[track_caller]
    pub fn err(&self, status: StatusCode, message: &str) {
        assert_eq!(self.status, status, "body: {}", self.body);
        assert_eq!(self.json()["error"], message, "body: {}", self.body);
    }
}

/// A fake browser: keeps cookies, sends `Origin` like a real one and comes from one address.
pub struct Browser {
    router: Router,
    jar: BTreeMap<String, String>,
    pub ip: IpAddr,
    pub user_agent: String,
    pub origin: Option<String>,
    /// Extra request headers, such as `x-forwarded-for`.
    pub headers: Vec<(String, String)>,
}

impl App {
    pub fn browser(&self) -> Browser {
        self.browser_at([198, 51, 100, 1])
    }

    pub fn browser_at(&self, ip: [u8; 4]) -> Browser {
        Browser {
            router: self.router.clone(),
            jar: BTreeMap::new(),
            ip: IpAddr::from(ip),
            user_agent: "Mozilla/5.0 (X11; Linux x86_64) test".to_owned(),
            origin: Some(ORIGIN.to_owned()),
            headers: Vec::new(),
        }
    }
}

impl Browser {
    pub fn cookie(&self, name: &str) -> Option<&str> {
        self.jar
            .get(name)
            .map(String::as_str)
            .filter(|v| !v.is_empty())
    }

    pub fn set_cookie(&mut self, name: &str, value: &str) {
        self.jar.insert(name.to_owned(), value.to_owned());
    }

    pub async fn send(&mut self, method: Method, path: &str, body: Option<Value>) -> Reply {
        let mut builder = Request::builder()
            .method(method)
            .uri(format!("/api/v1{path}"))
            .header("user-agent", &self.user_agent);
        if let Some(origin) = &self.origin {
            builder = builder.header("origin", origin);
        }
        for (name, value) in &self.headers {
            builder = builder.header(name, value);
        }
        if !self.jar.is_empty() {
            let cookie: Vec<String> = self
                .jar
                .iter()
                .filter(|(_, value)| !value.is_empty())
                .map(|(name, value)| format!("{name}={value}"))
                .collect();
            if !cookie.is_empty() {
                builder = builder.header("cookie", cookie.join("; "));
            }
        }
        let request = if let Some(body) = body {
            builder
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
        } else {
            builder.body(Body::empty())
        };
        let mut request = request.expect("valid request");
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::new(self.ip, 40000)));
        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("router is infallible");
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.expect("body reads");
        let reply = Reply {
            status: parts.status,
            headers: parts.headers,
            body: String::from_utf8_lossy(&bytes).into_owned(),
        };
        for (name, value) in reply.set_cookies() {
            self.jar.insert(name, value);
        }
        reply
    }

    pub async fn get(&mut self, path: &str) -> Reply {
        self.send(Method::GET, path, None).await
    }

    pub async fn post(&mut self, path: &str, body: Value) -> Reply {
        self.send(Method::POST, path, Some(body)).await
    }

    /// `POST` without a body.
    pub async fn post_empty(&mut self, path: &str) -> Reply {
        self.send(Method::POST, path, None).await
    }

    pub async fn patch(&mut self, path: &str, body: Value) -> Reply {
        self.send(Method::PATCH, path, Some(body)).await
    }

    pub async fn delete(&mut self, path: &str) -> Reply {
        self.send(Method::DELETE, path, None).await
    }

    /// Signs up `handle` (display name `Name of {handle}`) with [`PASSWORD`] and returns the
    /// `MeDto`. `add` keeps the accounts already signed in on this browser.
    pub async fn signup_as(&mut self, handle: &str, add: bool) -> Value {
        let reply = self
            .post(
                "/auth/signup",
                json!({
                    "handle": handle,
                    "displayName": format!("Name of {handle}"),
                    "password": PASSWORD,
                    "acceptTerms": true,
                    "add": add,
                }),
            )
            .await;
        reply.ok()["user"].clone()
    }

    pub async fn signup(&mut self, handle: &str) -> Value {
        self.signup_as(handle, false).await
    }

    /// Posts `text` and returns the `PostDto`.
    pub async fn post_text(&mut self, text: &str) -> Value {
        self.post("/posts", json!({ "text": text })).await.ok()
    }
}

/// Inserts a local account straight into the database, without a password, and returns its id.
/// Counters start at 0; fixtures that insert posts or follows must keep them right themselves.
pub async fn insert_user(db: &PgPool, handle: &str) -> uuid::Uuid {
    let id = uuid::Uuid::now_v7();
    sqlx::query("insert into users (id, handle, display_name) values ($1, $2, $3)")
        .bind(id)
        .bind(handle)
        .bind(format!("Name of {handle}"))
        .execute(db)
        .await
        .expect("insert user");
    id
}

/// Sets the admin's blocked words (one per line or comma separated); the instance blocks
/// nothing until this is called.
pub async fn block_words(db: &PgPool, words: &str) {
    sqlx::query(
        "insert into instance_settings (key, value) values ('content_filter', $1)
         on conflict (key) do update set value = excluded.value",
    )
    .bind(json!({ "blocked": words, "allowed": "", "reserved": "" }))
    .execute(db)
    .await
    .expect("block words");
}

impl Browser {
    /// Signs this browser in as `user` by inserting a session row and setting the cookie,
    /// bypassing signup and login.
    pub async fn sign_in_as(&mut self, db: &PgPool, user: uuid::Uuid) {
        let token = tuita_server::auth::new_token();
        let meta = tuita_server::db::sessions::ClientMeta::default();
        tuita_server::db::sessions::insert(
            db,
            user,
            &tuita_server::auth::hash_token(&token),
            &meta,
        )
        .await
        .expect("insert session");
        self.set_cookie(tuita_server::auth::SESSION_COOKIE, &token);
    }
}

/// An account that exists in the database and a browser that is signed in as it.
pub struct Person {
    pub id: uuid::Uuid,
    pub handle: String,
    pub browser: Browser,
}

impl App {
    /// Inserts `handle` and returns it with a signed-in browser.
    pub async fn person(&self, db: &PgPool, handle: &str) -> Person {
        let id = insert_user(db, handle).await;
        let mut browser = self.browser();
        browser.sign_in_as(db, id).await;
        Person {
            id,
            handle: handle.to_owned(),
            browser,
        }
    }
}

/// A reply with the body kept as bytes, for files.
pub struct RawReply {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub bytes: Vec<u8>,
}

impl RawReply {
    pub fn header(&self, name: &str) -> &str {
        self.headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
    }

    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.bytes).unwrap_or(Value::Null)
    }

    /// Asserts the status and the `error` message.
    #[track_caller]
    pub fn err(&self, status: StatusCode, message: &str) {
        assert_eq!(
            self.status,
            status,
            "body: {}",
            String::from_utf8_lossy(&self.bytes)
        );
        assert_eq!(self.json()["error"], message);
    }

    /// Asserts a `200` and returns the JSON body.
    #[track_caller]
    pub fn ok(&self) -> Value {
        assert_eq!(
            self.status,
            StatusCode::OK,
            "body: {}",
            String::from_utf8_lossy(&self.bytes)
        );
        self.json()
    }
}

/// One part of a `multipart/form-data` body.
pub struct Part {
    pub name: &'static str,
    pub content_type: Option<&'static str>,
    pub data: Vec<u8>,
}

impl Part {
    pub fn text(name: &'static str, value: &str) -> Self {
        Self {
            name,
            content_type: None,
            data: value.as_bytes().to_vec(),
        }
    }

    pub fn file(name: &'static str, content_type: &'static str, data: Vec<u8>) -> Self {
        Self {
            name,
            content_type: Some(content_type),
            data,
        }
    }
}

/// Encodes `parts` and returns the `Content-Type` header value and the body.
pub fn multipart(parts: &[Part]) -> (String, Vec<u8>) {
    let boundary = "----tuita-test-boundary";
    let mut body = Vec::new();
    for part in parts {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{}\"", part.name).as_bytes(),
        );
        if let Some(content_type) = part.content_type {
            body.extend_from_slice(
                format!("; filename=\"upload\"\r\nContent-Type: {content_type}").as_bytes(),
            );
        }
        body.extend_from_slice(b"\r\n\r\n");
        body.extend_from_slice(&part.data);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), body)
}

impl Browser {
    /// Sends a request to `uri` as given (not prefixed with `/api/v1`) with this browser's
    /// cookies and `Origin`, and returns the body as bytes.
    pub async fn send_raw(
        &mut self,
        method: Method,
        uri: &str,
        headers: &[(&str, &str)],
        body: Body,
    ) -> RawReply {
        let mut builder = Request::builder()
            .method(method)
            .uri(uri)
            .header("user-agent", &self.user_agent);
        if let Some(origin) = &self.origin {
            builder = builder.header("origin", origin);
        }
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        let cookie: Vec<String> = self
            .jar
            .iter()
            .filter(|(_, value)| !value.is_empty())
            .map(|(name, value)| format!("{name}={value}"))
            .collect();
        if !cookie.is_empty() {
            builder = builder.header("cookie", cookie.join("; "));
        }
        let mut request = builder.body(body).expect("valid request");
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::new(self.ip, 40000)));
        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("router is infallible");
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.expect("body reads");
        RawReply {
            status: parts.status,
            headers: parts.headers,
            bytes: bytes.to_vec(),
        }
    }

    /// `POST /api/v1/media` with `parts`.
    pub async fn upload(&mut self, parts: &[Part]) -> RawReply {
        let (content_type, body) = multipart(parts);
        self.send_raw(
            Method::POST,
            "/api/v1/media",
            &[("content-type", &content_type)],
            Body::from(body),
        )
        .await
    }
}
