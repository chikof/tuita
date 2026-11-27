//! Shared helpers for the page tests: the whole app in-process and a cookie-keeping fake browser.
//!
//! The assets come from `TUITA_TEST_ASSETS`, else `<workspace>/target/debug/assets`; run
//! `topcoat asset bundle --package tuita` first.
#![allow(dead_code, unreachable_pub, missing_docs)]

use std::{collections::BTreeMap, net::SocketAddr, path::PathBuf};

use axum::{
    Router,
    body::{Body, to_bytes},
    extract::ConnectInfo,
    http::{HeaderMap, Method, Request, StatusCode},
};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use tuita_server::{AppState, Config, media::MediaStore};
use tuita_web::Assets;

/// The origin the test instance believes it lives at.
pub const ORIGIN: &str = "https://social.example";
/// A password that passes the policy.
pub const PASSWORD: &str = "correct horse battery";

fn assets_dir() -> PathBuf {
    std::env::var_os("TUITA_TEST_ASSETS").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/assets"),
        PathBuf::from,
    )
}

/// The full application over `db`: the backend, the web client and the server's middleware.
pub fn app(db: PgPool) -> Router {
    let config = Config::from_lookup(|var| match var {
        "DATABASE_URL" => Some("postgres://unused".to_owned()),
        "PUBLIC_URL" => Some(ORIGIN.to_owned()),
        _ => None,
    })
    .expect("valid test config");
    let media = MediaStore::from_config(&config).expect("media store");
    let state = AppState::new(db, config, media);
    state.set_ready(true);
    let dir = assets_dir();
    let assets = Assets::from_dir(&dir).unwrap_or_else(|error| {
        panic!(
            "no asset bundle in {}: {error}; run `topcoat asset bundle --package tuita`",
            dir.display()
        )
    });
    tuita_server::router(state.clone(), tuita_web::router(state, assets))
}

pub struct Reply {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: String,
}

impl Reply {
    pub fn header(&self, name: &str) -> &str {
        self.headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
    }

    /// Every value of `name`, joined with `, `.
    pub fn header_all(&self, name: &str) -> String {
        self.headers
            .get_all(name)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// The `name=value` pairs of every `Set-Cookie` header; an ended cookie has an empty value.
    pub fn set_cookies(&self) -> Vec<(String, String)> {
        self.headers
            .get_all("set-cookie")
            .iter()
            .filter_map(|value| value.to_str().ok())
            .filter_map(|raw| {
                let (name, value) = raw.split(';').next()?.split_once('=')?;
                Some((name.to_owned(), value.to_owned()))
            })
            .collect()
    }
}

/// A fake browser: keeps cookies and comes from one address.
pub struct Browser {
    router: Router,
    jar: BTreeMap<String, String>,
    pub headers: Vec<(String, String)>,
}

impl Browser {
    pub fn new(router: &Router) -> Self {
        Self {
            router: router.clone(),
            jar: BTreeMap::new(),
            headers: Vec::new(),
        }
    }

    pub fn set_cookie(&mut self, name: &str, value: &str) {
        self.jar.insert(name.to_owned(), value.to_owned());
    }

    pub async fn request(&mut self, method: Method, path: &str, body: Option<Value>) -> Reply {
        let mut builder = Request::builder()
            .method(method)
            .uri(path)
            .header("user-agent", "Mozilla/5.0 (X11; Linux x86_64) test")
            .header("origin", ORIGIN);
        for (name, value) in &self.headers {
            builder = builder.header(name, value);
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
        let mut request = match body {
            Some(body) => builder
                .header("content-type", "application/json")
                .body(Body::from(body.to_string())),
            None => builder.body(Body::empty()),
        }
        .expect("valid request");
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([198, 51, 100, 1], 40000))));
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
        self.request(Method::GET, path, None).await
    }

    /// Signs up `handle` through the API and keeps the session cookie.
    pub async fn signup(&mut self, handle: &str) {
        let reply = self
            .request(
                Method::POST,
                "/api/v1/auth/signup",
                Some(json!({
                    "handle": handle,
                    "displayName": format!("Name of {handle}"),
                    "password": PASSWORD,
                    "acceptTerms": true,
                })),
            )
            .await;
        assert_eq!(reply.status, StatusCode::OK, "signup: {}", reply.body);
    }
}
