//! A Mastodon client for the tests: it registers an app, runs the authorization-code flow with
//! PKCE through `POST /oauth/authorize` with a session cookie, and then calls the API with the
//! bearer token. Included with `#[path]` by the Mastodon test files.
#![allow(dead_code, unreachable_pub, missing_docs)]

use std::collections::HashMap;
use std::path::PathBuf;

use axum::body::Body;
use axum::http::Method;
use serde_json::{Value, json};
use sqlx::PgPool;
use url::{Url, form_urlencoded};
use uuid::Uuid;

use crate::common::{self, App, Browser, Part, Person, RawReply};

pub const CLIENT_ORIGIN: &str = "https://client.example";
/// RFC 7636 appendix B.
pub const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
pub const CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
pub const REDIRECT: &str = "https://client.example/callback";

/// An app over its own `DATA_DIR`, removed on drop.
pub struct Env {
    pub app: App,
    pub data: PathBuf,
}

impl Env {
    pub fn new(db: PgPool) -> Self {
        Self::with(db, |_| None)
    }

    pub fn with(db: PgPool, extra: impl Fn(&str) -> Option<String>) -> Self {
        let data = std::env::temp_dir().join(format!("tuita-mastodon-{}", Uuid::now_v7()));
        let dir = data.display().to_string();
        let app = common::app_with(db, |var| {
            if var == "DATA_DIR" {
                Some(dir.clone())
            } else {
                extra(var)
            }
        });
        Self { app, data }
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.data);
    }
}

/// A bearer-token client on another origin, with no cookies.
pub struct Api {
    browser: Browser,
    pub token: String,
}

impl Api {
    pub async fn send(
        &mut self,
        method: Method,
        path: &str,
        extra: &[(&str, &str)],
        body: Option<(&str, Vec<u8>)>,
    ) -> RawReply {
        let auth = format!("Bearer {}", self.token);
        let mut headers: Vec<(&str, &str)> = vec![("authorization", &auth)];
        headers.extend_from_slice(extra);
        let content_type;
        let body = match body {
            Some((kind, bytes)) => {
                content_type = kind.to_owned();
                headers.push(("content-type", &content_type));
                Body::from(bytes)
            }
            None => Body::empty(),
        };
        self.browser.send_raw(method, path, &headers, body).await
    }

    pub async fn get(&mut self, path: &str) -> RawReply {
        self.send(Method::GET, path, &[], None).await
    }

    pub async fn delete(&mut self, path: &str) -> RawReply {
        self.send(Method::DELETE, path, &[], None).await
    }

    /// `POST` with no body.
    pub async fn post(&mut self, path: &str) -> RawReply {
        self.send(Method::POST, path, &[], None).await
    }

    pub async fn post_json(&mut self, path: &str, body: &Value) -> RawReply {
        self.send_json(Method::POST, path, &[], body).await
    }

    pub async fn put_json(&mut self, path: &str, body: &Value) -> RawReply {
        self.send_json(Method::PUT, path, &[], body).await
    }

    pub async fn send_json(
        &mut self,
        method: Method,
        path: &str,
        extra: &[(&str, &str)],
        body: &Value,
    ) -> RawReply {
        self.send(
            method,
            path,
            extra,
            Some(("application/json", body.to_string().into_bytes())),
        )
        .await
    }

    pub async fn post_form(&mut self, path: &str, pairs: &[(&str, &str)]) -> RawReply {
        let body = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(pairs)
            .finish();
        self.send(
            Method::POST,
            path,
            &[],
            Some(("application/x-www-form-urlencoded", body.into_bytes())),
        )
        .await
    }

    /// `POST /api/v2/media` with a file and an optional description.
    pub async fn upload(
        &mut self,
        content_type: &'static str,
        bytes: Vec<u8>,
        alt: &str,
    ) -> RawReply {
        let mut parts = vec![Part::file("file", content_type, bytes)];
        if !alt.is_empty() {
            parts.push(Part::text("description", alt));
        }
        let (kind, body) = common::multipart(&parts);
        self.send(Method::POST, "/api/v2/media", &[], Some((&kind, body)))
            .await
    }
}

fn query_of(url: &str) -> HashMap<String, String> {
    Url::parse(url)
        .expect("url")
        .query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect()
}

/// A bare client for registration and the token endpoint.
pub fn client(app: &App, ip: [u8; 4]) -> Browser {
    let mut browser = app.browser_at(ip);
    browser.origin = Some(CLIENT_ORIGIN.to_owned());
    browser
}

/// Runs the whole OAuth flow for `user`: registers an app, approves it with the user's session
/// cookie (PKCE S256), exchanges the code, and returns the client that holds the token.
pub async fn sign_in(app: &App, user: &mut Person, scopes: &str, ip: [u8; 4]) -> Api {
    let mut browser = client(app, ip);
    let body = json!({
        "client_name": "Test App",
        "redirect_uris": REDIRECT,
        "scopes": scopes,
        "website": "https://client.example",
    });
    let registered = browser
        .send_raw(
            Method::POST,
            "/api/v1/apps",
            &[("content-type", "application/json")],
            Body::from(body.to_string()),
        )
        .await
        .ok();
    let approve = json!({
        "clientId": registered["client_id"],
        "redirectUri": REDIRECT,
        "scope": scopes,
        "responseType": "code",
        "state": "st-1",
        "codeChallenge": CHALLENGE,
        "codeChallengeMethod": "S256",
        "approve": true,
    });
    let answer = user
        .browser
        .send_raw(
            Method::POST,
            "/oauth/authorize",
            &[("content-type", "application/json")],
            Body::from(approve.to_string()),
        )
        .await
        .ok();
    let redirect = answer["redirect"].as_str().expect("redirect").to_owned();
    let query = query_of(&redirect);
    assert_eq!(query["state"], "st-1");
    let pairs = [
        ("grant_type", "authorization_code"),
        ("client_id", registered["client_id"].as_str().expect("id")),
        ("code", query["code"].as_str()),
        ("redirect_uri", REDIRECT),
        ("code_verifier", VERIFIER),
    ];
    let form = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs)
        .finish();
    let token = browser
        .send_raw(
            Method::POST,
            "/oauth/token",
            &[("content-type", "application/x-www-form-urlencoded")],
            Body::from(form),
        )
        .await
        .ok();
    assert_eq!(token["token_type"], "Bearer");
    Api {
        browser,
        token: token["access_token"].as_str().expect("token").to_owned(),
    }
}
