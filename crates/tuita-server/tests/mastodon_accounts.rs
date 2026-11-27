//! The Mastodon account, instance, search and list routes, driven with OAuth tokens.
#![allow(missing_docs)]

mod common;

use std::collections::HashMap;
use std::io::Cursor;
use std::path::PathBuf;

use axum::body::Body;
use axum::http::{Method, StatusCode};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use common::{App, Browser, ORIGIN, Part, Person, RawReply, multipart};
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use sqlx::PgPool;
use tuita_core::ap;
use tuita_server::db::remote::{self, RemoteProfile, Upsert};
use url::{Url, form_urlencoded};
use uuid::Uuid;

const CLIENT_ORIGIN: &str = "https://client.example";
const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
const REDIRECT: &str = "https://client.example/callback";

// ------------------------------------------------------------------ helpers

/// A Mastodon client: another origin, no cookies, one bearer token.
struct Client {
    browser: Browser,
    token: Option<String>,
}

impl Client {
    async fn call(&mut self, method: Method, uri: &str, body: Option<(&str, Vec<u8>)>) -> RawReply {
        let auth = self.token.as_ref().map(|token| format!("Bearer {token}"));
        let mut headers: Vec<(&str, &str)> = Vec::new();
        if let Some(auth) = &auth {
            headers.push(("authorization", auth));
        }
        let (content_type, body) = match body {
            Some((content_type, bytes)) => (Some(content_type), Body::from(bytes)),
            None => (None, Body::empty()),
        };
        if let Some(content_type) = &content_type {
            headers.push(("content-type", content_type));
        }
        self.browser.send_raw(method, uri, &headers, body).await
    }

    async fn get(&mut self, uri: &str) -> RawReply {
        self.call(Method::GET, uri, None).await
    }

    async fn post(&mut self, uri: &str) -> RawReply {
        self.call(Method::POST, uri, None).await
    }

    async fn form(&mut self, method: Method, uri: &str, pairs: &[(&str, &str)]) -> RawReply {
        let body = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(pairs)
            .finish();
        self.call(
            method,
            uri,
            Some(("application/x-www-form-urlencoded", body.into_bytes())),
        )
        .await
    }

    async fn json(&mut self, method: Method, uri: &str, body: &Value) -> RawReply {
        self.call(
            method,
            uri,
            Some(("application/json", body.to_string().into_bytes())),
        )
        .await
    }
}

fn client(app: &App) -> Browser {
    let mut browser = app.browser_at([203, 0, 113, 7]);
    browser.origin = Some(CLIENT_ORIGIN.to_owned());
    browser
}

async fn send_form(browser: &mut Browser, uri: &str, pairs: &[(&str, &str)]) -> RawReply {
    let body = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs)
        .finish();
    browser
        .send_raw(
            Method::POST,
            uri,
            &[("content-type", "application/x-www-form-urlencoded")],
            Body::from(body),
        )
        .await
}

/// Registers an app, runs the PKCE flow as `user` and returns a client holding the token.
async fn sign_in(app: &App, user: &mut Person, scopes: &str) -> Client {
    let mut browser = client(app);
    let registered = browser
        .send_raw(
            Method::POST,
            "/api/v1/apps",
            &[("content-type", "application/json")],
            Body::from(
                json!({
                    "client_name": "Test App",
                    "redirect_uris": REDIRECT,
                    "scopes": scopes,
                })
                .to_string(),
            ),
        )
        .await
        .ok();
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(VERIFIER.as_bytes()));
    let approved = user
        .browser
        .send_raw(
            Method::POST,
            "/oauth/authorize",
            &[("content-type", "application/json")],
            Body::from(
                json!({
                    "clientId": registered["client_id"],
                    "redirectUri": REDIRECT,
                    "scope": scopes,
                    "responseType": "code",
                    "state": "s",
                    "codeChallenge": challenge,
                    "codeChallengeMethod": "S256",
                    "approve": true,
                })
                .to_string(),
            ),
        )
        .await
        .ok();
    let redirect = approved["redirect"].as_str().expect("redirect");
    let code: HashMap<String, String> = Url::parse(redirect)
        .expect("url")
        .query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    let token = send_form(
        &mut browser,
        "/oauth/token",
        &[
            ("grant_type", "authorization_code"),
            ("client_id", registered["client_id"].as_str().expect("id")),
            ("code", &code["code"]),
            ("redirect_uri", REDIRECT),
            ("code_verifier", VERIFIER),
        ],
    )
    .await
    .ok();
    Client {
        browser,
        token: Some(token["access_token"].as_str().expect("token").to_owned()),
    }
}

fn ids(list: &Value) -> Vec<String> {
    list.as_array()
        .expect("array")
        .iter()
        .map(|item| item["id"].as_str().expect("id").to_owned())
        .collect()
}

fn png() -> Vec<u8> {
    let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
        64,
        32,
        image::Rgb([200, 30, 30]),
    ));
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .expect("png");
    bytes
}

async fn remote_account(db: &PgPool, name: &str) -> Uuid {
    let ap_id = format!("https://remote.example/users/{name}");
    let profile = RemoteProfile {
        handle: name.to_owned(),
        domain: "remote.example".to_owned(),
        display_name: name.to_owned(),
        bio: String::new(),
        website: String::new(),
        is_bot: false,
        is_org: false,
        locked: false,
        discoverable: true,
        inbox_url: format!("{ap_id}/inbox"),
        shared_inbox_url: None,
        outbox_url: None,
        followers_url: None,
        featured_url: None,
        url: Some(format!("https://remote.example/@{name}")),
        public_key_pem: ap::generate_keypair().expect("keypair").public_pem,
        created_at: None,
        ap_id,
    };
    let Upsert::Stored(id) = remote::upsert_actor(db, &profile).await.expect("upsert") else {
        panic!("not stored");
    };
    id
}

// ------------------------------------------------------------------ accounts

#[sqlx::test(migrations = "./migrations")]
async fn credentials_and_accounts_have_the_shape_clients_parse(db: PgPool) {
    let app = common::app(db.clone());
    let mut alice = app.person(&db, "alice").await;
    let mut bob = app.person(&db, "bob").await;
    bob.browser
        .post("/posts", json!({"text": "hello"}))
        .await
        .ok();
    let mut me = sign_in(&app, &mut alice, "read write").await;

    let account = me.get("/api/v1/accounts/verify_credentials").await.ok();
    assert_eq!(account["id"], alice.id.to_string());
    assert_eq!(account["username"], "alice");
    assert_eq!(account["acct"], "alice");
    assert_eq!(account["display_name"], "Name of alice");
    assert_eq!(account["url"], format!("{ORIGIN}/@alice"));
    for key in ["emojis", "fields"] {
        assert!(account[key].is_array(), "{key}");
    }
    assert_eq!(account["source"]["note"], "");
    assert_eq!(account["source"]["follow_requests_count"], 0);
    assert!(account["source"]["fields"].is_array());
    assert_eq!(account["following_count"], 0);

    let other = me.get(&format!("/api/v1/accounts/{}", bob.id)).await.ok();
    assert_eq!(other["acct"], "bob");
    assert_eq!(other["statuses_count"], 1);
    assert!(other["created_at"].as_str().expect("time").ends_with('Z'));
    assert!(other.get("source").is_none());

    let looked = me.get("/api/v1/accounts/lookup?acct=bob").await.ok();
    assert_eq!(looked["id"], bob.id.to_string());
    let own_domain = me
        .get("/api/v1/accounts/lookup?acct=%40bob%40social.example")
        .await
        .ok();
    assert_eq!(own_domain["id"], bob.id.to_string());
    me.get("/api/v1/accounts/lookup?acct=nobody")
        .await
        .err(StatusCode::NOT_FOUND, "Record not found");
    me.get("/api/v1/accounts/lookup").await.err(
        StatusCode::BAD_REQUEST,
        "param is missing or the value is empty: acct",
    );
    me.get(&format!("/api/v1/accounts/{}", Uuid::now_v7()))
        .await
        .err(StatusCode::NOT_FOUND, "Record not found");
    me.get("/api/v1/accounts/not-an-id")
        .await
        .err(StatusCode::NOT_FOUND, "Record not found");

    // Without a sign-in the account routes refuse.
    let mut guest = Client {
        browser: client(&app),
        token: None,
    };
    guest
        .get(&format!("/api/v1/accounts/{}", bob.id))
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");
}

#[sqlx::test(migrations = "./migrations")]
async fn follow_block_and_mute_set_the_relationship_flags(db: PgPool) {
    let app = common::app(db.clone());
    let mut alice = app.person(&db, "alice").await;
    let bob = app.person(&db, "bob").await;
    let carol = app.person(&db, "carol").await;
    let mut me = sign_in(&app, &mut alice, "read write").await;

    let followed = me
        .post(&format!("/api/v1/accounts/{}/follow", bob.id))
        .await
        .ok();
    assert_eq!(followed["id"], bob.id.to_string());
    assert_eq!(followed["following"], true);
    assert_eq!(followed["requested"], false);
    assert_eq!(followed["showing_reblogs"], true);
    assert_eq!(followed["languages"], Value::Null);
    // Following twice changes nothing.
    me.post(&format!("/api/v1/accounts/{}/follow", bob.id))
        .await
        .ok();

    let bob_now = me.get(&format!("/api/v1/accounts/{}", bob.id)).await.ok();
    assert_eq!(bob_now["followers_count"], 1);
    let alice_now = me.get("/api/v1/accounts/verify_credentials").await.ok();
    assert_eq!(alice_now["following_count"], 1);

    // Bob follows back through tuita's own API, so `followed_by` shows.
    let mut bob_browser = bob.browser;
    bob_browser.post_empty("/users/alice/follow").await.ok();
    let query = format!(
        "/api/v1/accounts/relationships?id[]={}&id[]={}&id[]={}",
        bob.id,
        carol.id,
        Uuid::now_v7()
    );
    let rels = me.get(&query).await.ok();
    let rels = rels.as_array().expect("array");
    assert_eq!(rels.len(), 2, "an unknown id is left out");
    assert_eq!(rels[0]["id"], bob.id.to_string());
    assert_eq!(rels[0]["following"], true);
    assert_eq!(rels[0]["followed_by"], true);
    assert_eq!(rels[1]["id"], carol.id.to_string());
    assert_eq!(rels[1]["following"], false);

    let unfollowed = me
        .post(&format!("/api/v1/accounts/{}/unfollow", bob.id))
        .await
        .ok();
    assert_eq!(unfollowed["following"], false);
    assert_eq!(unfollowed["followed_by"], true);
    let bob_after = me.get(&format!("/api/v1/accounts/{}", bob.id)).await.ok();
    assert_eq!(bob_after["followers_count"], 0);

    let muted = me
        .post(&format!("/api/v1/accounts/{}/mute", carol.id))
        .await
        .ok();
    assert_eq!(muted["muting"], true);
    assert_eq!(muted["blocking"], false);
    let unmuted = me
        .post(&format!("/api/v1/accounts/{}/unmute", carol.id))
        .await
        .ok();
    assert_eq!(unmuted["muting"], false);

    let blocked = me
        .post(&format!("/api/v1/accounts/{}/block", bob.id))
        .await
        .ok();
    assert_eq!(blocked["blocking"], true);
    assert_eq!(
        blocked["followed_by"], false,
        "a block drops the follows both ways"
    );
    me.post(&format!("/api/v1/accounts/{}/follow", bob.id))
        .await
        .err(
            StatusCode::FORBIDDEN,
            "You can\u{2019}t follow this account.",
        );
    let unblocked = me
        .post(&format!("/api/v1/accounts/{}/unblock", bob.id))
        .await
        .ok();
    assert_eq!(unblocked["blocking"], false);

    for action in ["follow", "block", "mute"] {
        let reply = me
            .post(&format!("/api/v1/accounts/{}/{action}", alice.id))
            .await;
        assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY, "{action}");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn followers_and_following_page_by_account_id(db: PgPool) {
    let app = common::app(db.clone());
    let mut alice = app.person(&db, "alice").await;
    let bob = app.person(&db, "bob").await;
    let carol = app.person(&db, "carol").await;
    let dave = app.person(&db, "dave").await;
    let mut me = sign_in(&app, &mut alice, "read write").await;
    for mut fan in [bob, carol, dave] {
        fan.browser.post_empty("/users/alice/follow").await.ok();
    }

    let first = me
        .get(&format!("/api/v1/accounts/{}/followers?limit=2", alice.id))
        .await;
    let page_one = first.ok();
    assert_eq!(page_one.as_array().expect("array").len(), 2);
    let link = first.header("link").to_owned();
    assert!(link.contains("rel=\"next\""), "{link}");
    assert!(link.contains("rel=\"prev\""), "{link}");
    assert!(link.starts_with(&format!(
        "<{ORIGIN}/api/v1/accounts/{}/followers?limit=2&max_id=",
        alice.id
    )));
    assert_eq!(first.header("access-control-allow-origin"), "*");
    // Newest follower first: dave, carol.
    assert_eq!(page_one[0]["acct"], "dave");
    assert_eq!(page_one[1]["acct"], "carol");

    let last = page_one[1]["id"].as_str().expect("id");
    let second = me
        .get(&format!(
            "/api/v1/accounts/{}/followers?limit=2&max_id={last}",
            alice.id
        ))
        .await;
    let page_two = second.ok();
    assert_eq!(ids(&page_two).len(), 1);
    assert_eq!(page_two[0]["acct"], "bob");
    assert!(!second.header("link").contains("rel=\"next\""));

    // `min_id` reads forward from a known account.
    let newer = me
        .get(&format!(
            "/api/v1/accounts/{}/followers?min_id={}",
            alice.id,
            page_two[0]["id"].as_str().expect("id")
        ))
        .await
        .ok();
    assert_eq!(newer[0]["acct"], "dave");
    assert_eq!(newer[1]["acct"], "carol");

    // Alice follows nobody; dave follows alice only.
    let following = me
        .get(&format!("/api/v1/accounts/{}/following", alice.id))
        .await
        .ok();
    assert_eq!(following, json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn account_statuses_honour_the_filters(db: PgPool) {
    let app = common::app(db.clone());
    let mut alice = app.person(&db, "alice").await;
    let mut bob = app.person(&db, "bob").await;
    let first = bob.browser.post_text("first post").await["id"]
        .as_str()
        .expect("id")
        .to_owned();
    let reply = bob
        .browser
        .post("/posts", json!({"text": "a reply", "replyTo": first}))
        .await
        .ok()["id"]
        .as_str()
        .expect("id")
        .to_owned();
    let carol_post = {
        let mut carol = app.person(&db, "carol").await;
        carol.browser.post_text("from carol").await["id"]
            .as_str()
            .expect("id")
            .to_owned()
    };
    bob.browser
        .post_empty(&format!("/posts/{carol_post}/repost"))
        .await
        .ok();
    bob.browser
        .post_empty(&format!("/posts/{first}/pin"))
        .await
        .ok();
    let mut me = sign_in(&app, &mut alice, "read").await;
    let base = format!("/api/v1/accounts/{}/statuses", bob.id);

    let all = me.get(&base).await.ok();
    let all = all.as_array().expect("array");
    assert_eq!(all.len(), 3);
    assert_eq!(all[0]["reblog"]["id"], carol_post, "the boost comes first");
    assert_eq!(all[0]["account"]["acct"], "bob");
    assert_eq!(all[0]["reblog"]["account"]["acct"], "carol");
    assert_eq!(all[1]["id"], reply);
    assert_eq!(all[1]["in_reply_to_id"], first);
    assert_eq!(all[2]["id"], first);
    assert_eq!(all[2]["pinned"], true);

    let no_replies = me.get(&format!("{base}?exclude_replies=true")).await.ok();
    assert_eq!(ids(&no_replies).len(), 2);
    assert!(!ids(&no_replies).contains(&reply));
    let no_boosts = me.get(&format!("{base}?exclude_reblogs=true")).await.ok();
    assert_eq!(ids(&no_boosts), vec![reply.clone(), first.clone()]);
    let media = me.get(&format!("{base}?only_media=true")).await.ok();
    assert_eq!(media, json!([]));
    let pinned = me.get(&format!("{base}?pinned=true")).await.ok();
    assert_eq!(ids(&pinned), vec![first.clone()]);
    assert_eq!(pinned[0]["pinned"], true);

    let page = me.get(&format!("{base}?limit=1")).await;
    assert_eq!(ids(&page.ok()).len(), 1);
    assert!(page.header("link").contains("rel=\"next\""));
    let older = me.get(&format!("{base}?max_id={reply}&limit=5")).await.ok();
    assert_eq!(ids(&older), vec![first.clone()]);

    let refused = me.post(&format!("/api/v1/accounts/{}/block", bob.id)).await;
    assert_eq!(
        refused.status,
        StatusCode::FORBIDDEN,
        "a read token cannot block"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_remote_account_is_looked_up_followed_and_found_by_search(db: PgPool) {
    let app = common::app(db.clone());
    let mut alice = app.person(&db, "alice").await;
    let remote = remote_account(&db, "rem").await;
    let mut me = sign_in(&app, &mut alice, "read write").await;

    let found = me
        .get("/api/v1/accounts/lookup?acct=rem%40remote.example")
        .await
        .ok();
    assert_eq!(found["id"], remote.to_string());
    assert_eq!(found["acct"], "rem@remote.example");
    assert_eq!(found["username"], "rem");
    assert_eq!(found["url"], "https://remote.example/@rem");
    assert_eq!(found["uri"], "https://remote.example/users/rem");

    let requested = me
        .post(&format!("/api/v1/accounts/{remote}/follow"))
        .await
        .ok();
    assert_eq!(requested["following"], false);
    assert_eq!(
        requested["requested"], true,
        "a remote follow waits for its Accept"
    );
    let queued: i64 = sqlx::query_scalar("select count(*) from ap_deliveries")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(queued, 1, "the Follow is queued for delivery");
    let undone = me
        .post(&format!("/api/v1/accounts/{remote}/unfollow"))
        .await
        .ok();
    assert_eq!(undone["requested"], false);

    let results = me
        .get("/api/v2/search?q=%40rem%40remote.example&resolve=true&type=accounts")
        .await
        .ok();
    assert_eq!(ids(&results["accounts"]), vec![remote.to_string()]);
    assert_eq!(results["statuses"], json!([]));
    assert_eq!(results["hashtags"], json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn update_credentials_changes_the_profile_with_tuitas_rules(db: PgPool) {
    let dir = std::env::temp_dir().join(format!("tuita-mastodon-{}", Uuid::now_v7()));
    let data = dir.display().to_string();
    let app = common::app_with(db.clone(), |var| (var == "DATA_DIR").then(|| data.clone()));
    let mut alice = app.person(&db, "alice").await;
    let mut me = sign_in(&app, &mut alice, "read write").await;

    let updated = me
        .json(
            Method::PATCH,
            "/api/v1/accounts/update_credentials",
            &json!({
                "display_name": "Alice A.",
                "note": "line one\nline two",
                "bot": true,
                "discoverable": false,
                "fields_attributes": [
                    {"name": "Website", "value": "example.org"},
                    {"name": "Location", "value": "Lisbon"},
                ],
            }),
        )
        .await
        .ok();
    assert_eq!(updated["display_name"], "Alice A.");
    assert_eq!(updated["bot"], true);
    assert_eq!(updated["discoverable"], false);
    assert_eq!(updated["source"]["note"], "line one\nline two");
    assert_eq!(updated["source"]["fields"][0]["name"], "Website");
    assert_eq!(
        updated["source"]["fields"][0]["value"],
        "https://example.org"
    );
    assert_eq!(updated["fields"][1]["value"], "Lisbon");
    assert!(updated["note"].as_str().expect("note").contains("<br>"));

    // A form body works the same; fields left out are cleared.
    let form = me
        .form(
            Method::PATCH,
            "/api/v1/accounts/update_credentials",
            &[
                ("display_name", "Alice B."),
                ("fields_attributes[0][name]", "Location"),
                ("fields_attributes[0][value]", "Porto"),
            ],
        )
        .await
        .ok();
    assert_eq!(form["display_name"], "Alice B.");
    assert_eq!(form["fields"].as_array().expect("fields").len(), 1);
    assert_eq!(form["fields"][0]["name"], "Location");

    // Avatar and header arrive as multipart files.
    let (content_type, body) = multipart(&[
        Part::text("display_name", "Alice C."),
        Part::file("avatar", "image/png", png()),
        Part::file("header", "image/png", png()),
    ]);
    let with_images = me
        .call(
            Method::PATCH,
            "/api/v1/accounts/update_credentials",
            Some((&content_type, body)),
        )
        .await
        .ok();
    assert_eq!(with_images["display_name"], "Alice C.");
    for key in ["avatar", "avatar_static", "header", "header_static"] {
        let url = with_images[key].as_str().expect("url");
        assert!(url.starts_with(&format!("{ORIGIN}/media/")), "{key}: {url}");
    }
    let media: PathBuf = dir.join("media");
    assert_eq!(std::fs::read_dir(media).expect("media dir").count(), 2);
    let public = app
        .browser()
        .send_raw(
            Method::GET,
            with_images["avatar"]
                .as_str()
                .and_then(|url| url.strip_prefix(ORIGIN))
                .expect("path"),
            &[],
            Body::empty(),
        )
        .await;
    assert_eq!(public.status, StatusCode::OK);
    let _ = std::fs::remove_dir_all(&dir);

    for (key, value) in [
        ("display_name", String::new()),
        ("note", "x".repeat(161)),
        ("display_name", "y".repeat(51)),
    ] {
        let reply = me
            .form(
                Method::PATCH,
                "/api/v1/accounts/update_credentials",
                &[(key, &value)],
            )
            .await;
        assert_eq!(
            reply.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{key}: {}",
            String::from_utf8_lossy(&reply.bytes)
        );
    }
    let not_an_image = {
        let (content_type, body) =
            multipart(&[Part::file("avatar", "image/png", b"nope".to_vec())]);
        me.call(
            Method::PATCH,
            "/api/v1/accounts/update_credentials",
            Some((&content_type, body)),
        )
        .await
    };
    assert_eq!(not_an_image.status, StatusCode::UNPROCESSABLE_ENTITY);
}

// ------------------------------------------------------------------ scopes

#[sqlx::test(migrations = "./migrations")]
async fn a_read_token_cannot_change_anything_and_a_write_token_can(db: PgPool) {
    let app = common::app(db.clone());
    let mut alice = app.person(&db, "alice").await;
    let bob = app.person(&db, "bob").await;
    let mut reader = sign_in(&app, &mut alice, "read").await;
    let mut writer = sign_in(&app, &mut alice, "read write").await;

    reader.get("/api/v1/accounts/verify_credentials").await.ok();
    reader
        .get(&format!("/api/v1/accounts/{}", bob.id))
        .await
        .ok();
    reader.get("/api/v1/lists").await.ok();
    let list = writer
        .form(Method::POST, "/api/v1/lists", &[("title", "L")])
        .await
        .ok();
    let list_id = list["id"].as_str().expect("id");

    type Mutation<'a> = (Method, String, Vec<(&'a str, &'a str)>);
    let mutations: Vec<Mutation<'_>> = vec![
        (
            Method::POST,
            format!("/api/v1/accounts/{}/follow", bob.id),
            vec![],
        ),
        (
            Method::POST,
            format!("/api/v1/accounts/{}/unfollow", bob.id),
            vec![],
        ),
        (
            Method::POST,
            format!("/api/v1/accounts/{}/block", bob.id),
            vec![],
        ),
        (
            Method::POST,
            format!("/api/v1/accounts/{}/unblock", bob.id),
            vec![],
        ),
        (
            Method::POST,
            format!("/api/v1/accounts/{}/mute", bob.id),
            vec![],
        ),
        (
            Method::POST,
            format!("/api/v1/accounts/{}/unmute", bob.id),
            vec![],
        ),
        (
            Method::PATCH,
            "/api/v1/accounts/update_credentials".to_owned(),
            vec![("display_name", "x")],
        ),
        (
            Method::POST,
            "/api/v1/lists".to_owned(),
            vec![("title", "x")],
        ),
        (
            Method::PUT,
            format!("/api/v1/lists/{list_id}"),
            vec![("title", "x")],
        ),
        (Method::DELETE, format!("/api/v1/lists/{list_id}"), vec![]),
        (
            Method::POST,
            format!("/api/v1/lists/{list_id}/accounts"),
            vec![("account_ids[]", "x")],
        ),
        (
            Method::DELETE,
            format!("/api/v1/lists/{list_id}/accounts"),
            vec![("account_ids[]", "x")],
        ),
    ];
    for (method, uri, pairs) in &mutations {
        let reply = reader.form(method.clone(), uri, pairs).await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{method} {uri}");
    }
    let before: i64 = sqlx::query_scalar("select count(*) from follows")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(before, 0);
    writer
        .post(&format!("/api/v1/accounts/{}/follow", bob.id))
        .await
        .ok();
    writer
        .form(
            Method::PATCH,
            "/api/v1/accounts/update_credentials",
            &[("display_name", "Written")],
        )
        .await
        .ok();
}

// ------------------------------------------------------------------ search

#[sqlx::test(migrations = "./migrations")]
async fn search_finds_accounts_statuses_and_hashtags(db: PgPool) {
    let app = common::app(db.clone());
    let mut alice = app.person(&db, "alice").await;
    let mut bob = app.person(&db, "bobcat").await;
    let post = bob
        .browser
        .post_text("learning about rustaceans #rustlang")
        .await["id"]
        .as_str()
        .expect("id")
        .to_owned();
    bob.browser.post_text("another #rustlang day").await;
    let mut me = sign_in(&app, &mut alice, "read").await;

    let all = me.get("/api/v2/search?q=rustlang").await.ok();
    for key in ["accounts", "statuses", "hashtags"] {
        assert!(all[key].is_array(), "{key}");
    }
    assert_eq!(all["statuses"].as_array().expect("array").len(), 2);
    assert_eq!(all["hashtags"][0]["name"], "rustlang");
    assert_eq!(all["hashtags"][0]["url"], format!("{ORIGIN}/tags/rustlang"));
    let history = all["hashtags"][0]["history"].as_array().expect("history");
    assert_eq!(history.len(), 7);
    assert_eq!(history[0]["uses"], "2", "both posts are from today");
    assert_eq!(history[0]["accounts"], "1");
    assert_eq!(history[6]["uses"], "0");

    let accounts = me.get("/api/v2/search?q=bobc&type=accounts").await.ok();
    assert_eq!(accounts["accounts"][0]["acct"], "bobcat");
    assert_eq!(accounts["statuses"], json!([]));
    assert_eq!(accounts["hashtags"], json!([]));

    let statuses = me
        .get("/api/v2/search?q=rustaceans&type=statuses")
        .await
        .ok();
    assert_eq!(ids(&statuses["statuses"]), vec![post.clone()]);
    assert_eq!(statuses["statuses"][0]["account"]["acct"], "bobcat");
    assert_eq!(statuses["statuses"][0]["tags"][0]["name"], "rustlang");

    let paged = me
        .get("/api/v2/search?q=rustlang&type=statuses&limit=1&offset=1")
        .await
        .ok();
    assert_eq!(paged["statuses"].as_array().expect("array").len(), 1);
    let tags = me.get("/api/v2/search?q=%23rustl&type=hashtags").await.ok();
    assert_eq!(tags["hashtags"][0]["name"], "rustlang");

    // Not following: `following=true` leaves bobcat out; after a follow it is back.
    let none = me
        .get("/api/v2/search?q=bobcat&type=accounts&following=true")
        .await
        .ok();
    assert_eq!(none["accounts"], json!([]));

    assert_eq!(
        me.get("/api/v2/search?q=%20%20").await.ok(),
        json!({"accounts": [], "statuses": [], "hashtags": []})
    );
    assert_eq!(
        me.get("/api/v2/search").await.status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        me.get("/api/v2/search?q=x&type=bogus").await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );

    // A blocked author drops out of the results.
    let mut writer = sign_in(&app, &mut alice, "read write").await;
    writer
        .post(&format!("/api/v1/accounts/{}/block", bob.id))
        .await
        .ok();
    let after = writer.get("/api/v2/search?q=rustlang").await.ok();
    assert_eq!(after["statuses"], json!([]));
    assert_eq!(after["accounts"], json!([]));
}

// ------------------------------------------------------------------ lists

#[sqlx::test(migrations = "./migrations")]
async fn lists_can_be_made_filled_changed_and_deleted(db: PgPool) {
    let app = common::app(db.clone());
    let mut alice = app.person(&db, "alice").await;
    let bob = app.person(&db, "bob").await;
    let carol = app.person(&db, "carol").await;
    let mut me = sign_in(&app, &mut alice, "read write").await;
    let mut other = sign_in(&app, &mut app.person(&db, "dave").await, "read write").await;

    let list = me
        .form(Method::POST, "/api/v1/lists", &[("title", "  Friends ")])
        .await
        .ok();
    assert_eq!(list["title"], "Friends");
    assert_eq!(list["replies_policy"], "list");
    assert_eq!(list["exclusive"], false);
    let id = list["id"].as_str().expect("id").to_owned();
    // JSON works too.
    let second = me
        .json(
            Method::POST,
            "/api/v1/lists",
            &json!({"title": "Second", "replies_policy": "none"}),
        )
        .await
        .ok();
    assert_eq!(second["title"], "Second");

    assert_eq!(
        me.get("/api/v1/lists")
            .await
            .ok()
            .as_array()
            .expect("array")
            .len(),
        2
    );
    assert_eq!(
        me.get(&format!("/api/v1/lists/{id}")).await.ok()["title"],
        "Friends"
    );

    let renamed = me
        .form(
            Method::PUT,
            &format!("/api/v1/lists/{id}"),
            &[("title", "Pals")],
        )
        .await
        .ok();
    assert_eq!(renamed["title"], "Pals");

    let added = me
        .form(
            Method::POST,
            &format!("/api/v1/lists/{id}/accounts"),
            &[
                ("account_ids[]", &bob.id.to_string()),
                ("account_ids[]", &carol.id.to_string()),
            ],
        )
        .await
        .ok();
    assert_eq!(added, json!({}));
    let members = me.get(&format!("/api/v1/lists/{id}/accounts")).await.ok();
    assert_eq!(
        ids(&members),
        vec![carol.id.to_string(), bob.id.to_string()]
    );
    let all = me
        .get(&format!("/api/v1/lists/{id}/accounts?limit=0"))
        .await;
    assert_eq!(ids(&all.ok()).len(), 2);
    assert_eq!(all.header("link"), "");
    let one = me
        .get(&format!("/api/v1/lists/{id}/accounts?limit=1"))
        .await;
    assert_eq!(ids(&one.ok()).len(), 1);
    assert!(one.header("link").contains("rel=\"next\""));
    let with_bob = me
        .get(&format!("/api/v1/accounts/{}/lists", bob.id))
        .await
        .ok();
    assert_eq!(ids(&with_bob), vec![id.clone()]);

    let removed = me
        .json(
            Method::DELETE,
            &format!("/api/v1/lists/{id}/accounts"),
            &json!({"account_ids": [bob.id.to_string()]}),
        )
        .await
        .ok();
    assert_eq!(removed, json!({}));
    assert_eq!(
        ids(&me.get(&format!("/api/v1/lists/{id}/accounts")).await.ok()),
        vec![carol.id.to_string()]
    );
    assert_eq!(
        me.get(&format!("/api/v1/accounts/{}/lists", bob.id))
            .await
            .ok(),
        json!([])
    );

    // Somebody else's list is not there, whatever the verb.
    for (method, suffix) in [
        (Method::GET, ""),
        (Method::PUT, ""),
        (Method::DELETE, ""),
        (Method::GET, "/accounts"),
        (Method::POST, "/accounts"),
    ] {
        let reply = other
            .form(
                method.clone(),
                &format!("/api/v1/lists/{id}{suffix}"),
                &[
                    ("title", "mine now"),
                    ("account_ids[]", &bob.id.to_string()),
                ],
            )
            .await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{method} {suffix}");
    }

    // The session of the same user keeps tuita's own shape on the shared paths.
    let tuita_lists = alice.browser.get("/lists").await.ok();
    assert!(tuita_lists[0]["name"].is_string(), "{tuita_lists}");
    assert!(tuita_lists[0].get("title").is_none());

    // Invalid input.
    let too_long = "x".repeat(41);
    for pairs in [
        vec![("title", "")],
        vec![("title", too_long.as_str())],
        vec![("replies_policy", "sometimes"), ("title", "ok")],
    ] {
        let reply = me.form(Method::POST, "/api/v1/lists", &pairs).await;
        assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY, "{pairs:?}");
    }
    me.form(
        Method::POST,
        &format!("/api/v1/lists/{id}/accounts"),
        &[("account_ids[]", &Uuid::now_v7().to_string())],
    )
    .await
    .err(StatusCode::NOT_FOUND, "Record not found");

    let deleted = me
        .call(Method::DELETE, &format!("/api/v1/lists/{id}"), None)
        .await
        .ok();
    assert_eq!(deleted, json!({}));
    me.get(&format!("/api/v1/lists/{id}"))
        .await
        .err(StatusCode::NOT_FOUND, "Record not found");
}

// ------------------------------------------------------------------ instance

#[sqlx::test(migrations = "./migrations")]
async fn instance_v1_serves_both_shapes_to_anyone_with_cors(db: PgPool) {
    let app = common::app(db.clone());
    let mut guest = Client {
        browser: client(&app),
        token: None,
    };
    let reply = guest.get("/api/v1/instance").await;
    assert_eq!(reply.header("access-control-allow-origin"), "*");
    let body = reply.ok();
    // Tuita's fields, as the web client reads them.
    assert_eq!(body["domain"], "social.example");
    assert_eq!(body["registrationsOpen"], true);
    assert_eq!(body["detectedLanguage"], "en");
    assert_eq!(body["stats"]["users"], 0);
    assert!(body["rules"].is_array());
    // Mastodon's v1 fields.
    assert_eq!(body["uri"], "social.example");
    assert_eq!(body["title"], body["name"]);
    assert!(
        body["version"]
            .as_str()
            .expect("version")
            .starts_with("4.3.0 (compatible; tuita ")
    );
    assert_eq!(body["tuitaVersion"], env!("CARGO_PKG_VERSION"));
    assert_eq!(body["urls"]["streaming_api"], "wss://social.example");
    assert_eq!(body["stats"]["user_count"], 0);
    assert_eq!(body["stats"]["status_count"], 0);
    assert_eq!(body["stats"]["domain_count"], 0);
    assert_eq!(body["registrations"], true);
    assert_eq!(body["approval_required"], false);
    assert_eq!(body["configuration"]["statuses"]["max_characters"], 500);
    assert_eq!(
        body["configuration"]["statuses"]["max_media_attachments"],
        4
    );
    assert!(body["languages"].is_array());
    assert_eq!(body["contact_account"], Value::Null);

    let preflight = guest
        .browser
        .send_raw(
            Method::OPTIONS,
            "/api/v2/instance",
            &[
                ("access-control-request-method", "GET"),
                ("access-control-request-headers", "authorization"),
            ],
            Body::empty(),
        )
        .await;
    assert_eq!(preflight.header("access-control-allow-origin"), "*");

    // A token that an app holds gets Mastodon's rule objects.
    let mut alice = app.person(&db, "alice").await;
    let mut me = sign_in(&app, &mut alice, "read").await;
    let with_token = me.get("/api/v1/instance").await.ok();
    assert_eq!(with_token["stats"]["users"], 1);
    assert_eq!(with_token["stats"]["user_count"], 1);
    sqlx::query("insert into instance_settings (key, value) values ('rules', $1)")
        .bind(json!(["Be kind.", "No spam."]))
        .execute(&db)
        .await
        .expect("rules");
    let rules = me.get("/api/v1/instance").await.ok();
    assert_eq!(
        rules["rules"][1],
        json!({"id": "2", "text": "No spam.", "hint": ""})
    );
    let tuita_rules = guest.get("/api/v1/instance").await.ok();
    assert_eq!(tuita_rules["rules"], json!(["Be kind.", "No spam."]));
}

#[sqlx::test(migrations = "./migrations")]
async fn instance_v2_reports_the_real_limits(db: PgPool) {
    let app = common::app(db.clone());
    let mut guest = Client {
        browser: client(&app),
        token: None,
    };
    let reply = guest.get("/api/v2/instance").await;
    assert_eq!(reply.header("access-control-allow-origin"), "*");
    let body = reply.ok();
    assert_eq!(body["domain"], "social.example");
    assert!(
        body["version"]
            .as_str()
            .expect("version")
            .starts_with("4.3.0 (")
    );
    assert_eq!(
        body["configuration"]["urls"]["streaming"],
        "wss://social.example"
    );
    assert_eq!(body["configuration"]["statuses"]["max_characters"], 500);
    assert_eq!(
        body["configuration"]["statuses"]["max_media_attachments"],
        4
    );
    assert_eq!(
        body["configuration"]["statuses"]["characters_reserved_per_url"],
        23
    );
    let media = &body["configuration"]["media_attachments"];
    assert_eq!(media["image_size_limit"], 10 * 1024 * 1024);
    assert_eq!(media["video_size_limit"], 50 * 1024 * 1024);
    assert_eq!(media["description_limit"], 1000);
    assert!(
        media["supported_mime_types"]
            .as_array()
            .expect("types")
            .contains(&json!("image/webp"))
    );
    assert_eq!(body["configuration"]["polls"]["max_options"], 4);
    assert_eq!(
        body["configuration"]["polls"]["max_characters_per_option"],
        25
    );
    assert_eq!(body["configuration"]["polls"]["max_expiration"], 168 * 3600);
    assert_eq!(body["registrations"]["enabled"], true);
    assert_eq!(body["usage"]["users"]["active_month"], 0);
    assert_eq!(body["thumbnail"]["url"], format!("{ORIGIN}/icon-512.png"));
    assert!(body["rules"].is_array());
    assert_eq!(body["contact"]["account"], Value::Null);
    assert!(body["contact"]["email"].is_string());
    assert!(body["languages"].is_array());

    assert_eq!(guest.get("/api/v1/custom_emojis").await.ok(), json!([]));
}

/// The unauthenticated instance routes must not hold a pool connection while they take another:
/// with a pool of one, the request that holds it and asks for a second waits for itself. Each
/// route gets its own app, so its counts are not cached yet and the request really queries.
#[sqlx::test(migrations = "./migrations")]
async fn concurrent_instance_requests_do_not_starve_a_small_pool(db: PgPool) {
    for path in ["/api/v1/instance", "/api/v2/instance"] {
        let small = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(2))
            .connect_with((*db.connect_options()).clone())
            .await
            .expect("small pool");
        let app = common::app(small);
        let started = std::time::Instant::now();
        let mut tasks = Vec::new();
        for _ in 0..8 {
            let mut browser = client(&app);
            tasks.push(tokio::spawn(async move {
                browser
                    .send_raw(Method::GET, path, &[], Body::empty())
                    .await
                    .status
            }));
        }
        for task in tasks {
            assert_eq!(task.await.expect("task"), StatusCode::OK, "{path}");
        }
        assert!(
            started.elapsed() < std::time::Duration::from_secs(5),
            "{path}"
        );
    }
}

/// A multipart `update_credentials` is counted as an upload before its body is read, so a caller
/// over the limit is refused with `429` even when the body is broken.
#[sqlx::test(migrations = "./migrations")]
async fn update_credentials_checks_the_upload_limit_before_reading_the_body(db: PgPool) {
    let app = common::app(db.clone());
    let mut alice = app.person(&db, "alice").await;
    let mut me = sign_in(&app, &mut alice, "read write").await;
    let (content_type, _) = multipart(&[Part::text("display_name", "x")]);
    for _ in 0..60 {
        app.state.limits().check_upload(alice.id).expect("budget");
    }
    assert!(app.state.limits().check_upload(alice.id).is_err(), "spent");
    let reply = me
        .call(
            Method::PATCH,
            "/api/v1/accounts/update_credentials",
            Some((&content_type, b"not a multipart body".to_vec())),
        )
        .await;
    assert_eq!(
        reply.status,
        StatusCode::TOO_MANY_REQUESTS,
        "{}",
        String::from_utf8_lossy(&reply.bytes)
    );
    let json = me
        .form(
            Method::PATCH,
            "/api/v1/accounts/update_credentials",
            &[("display_name", "still fine")],
        )
        .await;
    assert_eq!(json.status, StatusCode::OK, "a form body is not an upload");
}
