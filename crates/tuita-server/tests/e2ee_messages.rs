#![allow(missing_docs)]

mod common;
#[path = "common/oauth_client.rs"]
mod oauth_client;

use axum::http::{Method, StatusCode};
use common::{App, Browser, Person};
use oauth_client::{Env, sign_in};
use serde_json::{Value, json};
use sqlx::PgPool;
use tuita_server::db::{messages, remote};
use tuita_server::dto::StreamEvent;
use uuid::Uuid;

/// A message public key that passes the shape check; `fill` tells two keys apart.
fn key(fill: char) -> String {
    format!("BE{}A", fill.to_string().repeat(84))
}

const BACKUP: &str = "v1.600000.c2FsdA.aXY.c2VhbGVk";

/// A sealed message from `sender` to `recipient` that passes the shape check.
fn sealed(sender: &str, recipient: &str) -> String {
    format!("v1.{sender}.{recipient}.c2FsdA.aXY.Y2lwaGVy")
}

/// Turns encryption on for `person` with the key made from `fill`.
async fn turn_on(person: &mut Person, fill: char) -> String {
    let public_key = key(fill);
    person
        .browser
        .send(
            Method::PUT,
            "/me/message-key",
            Some(json!({ "publicKey": public_key, "backup": BACKUP })),
        )
        .await
        .ok();
    public_key
}

async fn put_key(browser: &mut Browser, body: Value) -> common::Reply {
    browser
        .send(Method::PUT, "/me/message-key", Some(body))
        .await
}

/// Makes a personal token for `browser` and returns a client that sends only that token.
async fn script(app: &App, browser: &mut Browser) -> Browser {
    let created = browser
        .post(
            "/tokens",
            json!({ "name": "script", "scopes": ["read", "write", "messages"] }),
        )
        .await
        .ok();
    let secret = created["token"].as_str().expect("secret");
    let mut client = app.browser();
    client.origin = None;
    client
        .headers
        .push(("authorization".to_owned(), format!("Bearer {secret}")));
    client
}

// ------------------------------------------------------------------ keys

#[sqlx::test(migrations = "./migrations")]
async fn key_is_null_until_it_is_set_and_then_served_back(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    assert_eq!(
        ana.browser.get("/me/message-key").await.ok(),
        json!({ "key": null })
    );

    let made = turn_on(&mut ana, 'Q').await;
    let got = ana.browser.get("/me/message-key").await.ok();
    assert_eq!(got["key"]["publicKey"], made);
    assert_eq!(got["key"]["backup"], BACKUP);
    assert!(got["key"]["createdAt"].is_string());
}

#[sqlx::test(migrations = "./migrations")]
async fn key_is_kept_unless_the_second_put_says_replace(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let first = turn_on(&mut ana, 'Q').await;

    put_key(
        &mut ana.browser,
        json!({ "publicKey": key('R'), "backup": BACKUP }),
    )
    .await
    .err(StatusCode::CONFLICT, "You already have a message key.");
    put_key(
        &mut ana.browser,
        json!({ "publicKey": key('R'), "backup": BACKUP, "replace": false }),
    )
    .await
    .err(StatusCode::CONFLICT, "You already have a message key.");
    assert_eq!(
        ana.browser.get("/me/message-key").await.ok()["key"]["publicKey"],
        first
    );

    let replaced = put_key(
        &mut ana.browser,
        json!({ "publicKey": key('R'), "backup": BACKUP, "replace": true }),
    )
    .await
    .ok();
    assert_eq!(replaced["publicKey"], key('R'));
    assert_eq!(
        ana.browser.get("/me/message-key").await.ok()["key"]["publicKey"],
        key('R')
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn key_with_a_malformed_public_key_or_backup_is_refused(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let good = key('Q');
    let bad_keys = [
        "".to_owned(),
        "short".to_owned(),
        format!("{good}A"),
        good.replacen("BE", "CE", 1),
        format!("BE{}+", "Q".repeat(84)),
    ];
    for bad in bad_keys {
        let reply = put_key(
            &mut ana.browser,
            json!({ "publicKey": bad, "backup": BACKUP }),
        )
        .await;
        assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY, "{bad}");
    }
    let too_long = format!("v1.600000.a.b.{}", "c".repeat(2048));
    let mut bob = app.person(&db, "bob").await;
    let bad_backups = [
        "",
        "nope",
        "v1.600.a.b.c",
        "v1.600000.a.b",
        "v2.600000.a.b.c",
        too_long.as_str(),
    ];
    for bad in bad_backups {
        let reply = put_key(
            &mut bob.browser,
            json!({ "publicKey": good, "backup": bad }),
        )
        .await;
        assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY, "{bad}");
    }
    let rows: i64 = sqlx::query_scalar("select count(*) from message_keys")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(rows, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn key_is_cookie_only(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    turn_on(&mut ana, 'Q').await;
    let mut client = script(&app, &mut ana.browser).await;
    let refused = "API tokens can\u{2019}t be used for this. Sign in on the website instead.";
    client
        .get("/me/message-key")
        .await
        .err(StatusCode::FORBIDDEN, refused);
    put_key(
        &mut client,
        json!({ "publicKey": key('R'), "backup": BACKUP, "replace": true }),
    )
    .await
    .err(StatusCode::FORBIDDEN, refused);
    assert_eq!(
        ana.browser.get("/me/message-key").await.ok()["key"]["publicKey"],
        key('Q')
    );

    let mut stranger = app.browser();
    assert_eq!(
        stranger.get("/me/message-key").await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn key_changes_are_limited_to_ten_an_hour(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    for _ in 0..10 {
        put_key(
            &mut ana.browser,
            json!({ "publicKey": key('Q'), "backup": BACKUP, "replace": true }),
        )
        .await
        .ok();
    }
    let reply = put_key(
        &mut ana.browser,
        json!({ "publicKey": key('Q'), "backup": BACKUP, "replace": true }),
    )
    .await;
    assert_eq!(reply.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        ana.browser.get("/me/message-key").await.status,
        StatusCode::OK
    );
}
// ------------------------------------------------------------------ fixtures

const STALE: &str = "Encryption keys changed. Reload and try again.";
const MUST_BE_SEALED: &str = "Messages to this account must be end-to-end encrypted.";
const NOT_ENCRYPTED: &str = "This conversation is not encrypted.";
const SEALED_CONTENT: &str = "<p>Encrypted message. Open tuita to read it.</p>";

/// An app with `MESSAGES_E2EE_REQUIRED` set as asked.
fn app_for(db: &PgPool, required: bool) -> App {
    common::app_with(db.clone(), move |var| {
        (var == "MESSAGES_E2EE_REQUIRED").then(|| required.to_string())
    })
}

/// A remote account with a fresh profile, so lookups never go to the network.
async fn remote_user(db: &PgPool, name: &str, domain: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "insert into users (id, handle, domain, display_name, ap_id, inbox_url, fetched_at)
         values ($1, $2, $3, $2, $4, $5, now())",
    )
    .bind(id)
    .bind(name)
    .bind(domain)
    .bind(format!("https://{domain}/users/{name}"))
    .bind(format!("https://{domain}/users/{name}/inbox"))
    .execute(db)
    .await
    .expect("insert remote");
    id
}

/// `ana` (key `Q`) and `bob` (key `R`), both with encryption on.
struct Pair {
    ana: Person,
    bob: Person,
    ana_key: String,
    bob_key: String,
}

async fn keyed_pair(app: &App, db: &PgPool) -> Pair {
    let mut ana = app.person(db, "ana").await;
    let mut bob = app.person(db, "bob").await;
    let ana_key = turn_on(&mut ana, 'Q').await;
    let bob_key = turn_on(&mut bob, 'R').await;
    Pair {
        ana,
        bob,
        ana_key,
        bob_key,
    }
}

async fn view(person: &mut Person, handle: &str) -> Value {
    person
        .browser
        .get(&format!("/conversations/{handle}"))
        .await
        .ok()
}

fn drain(
    rx: &mut tokio::sync::broadcast::Receiver<tuita_server::realtime::Event>,
) -> Vec<StreamEvent> {
    let mut out = Vec::new();
    while let Ok(event) = rx.try_recv() {
        out.push((*event).clone());
    }
    out
}

async fn rows(db: &PgPool) -> Vec<(String, Option<String>, bool)> {
    sqlx::query_as("select text, ciphertext, deleted_at is not null from messages order by id")
        .fetch_all(db)
        .await
        .expect("messages")
}

async fn legacy_message(db: &PgPool, from: Uuid, to: Uuid, text: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query("insert into messages (id, sender_id, recipient_id, text) values ($1, $2, $3, $4)")
        .bind(id)
        .bind(from)
        .bind(to)
        .bind(text)
        .execute(db)
        .await
        .expect("insert message");
    id
}

// ------------------------------------------------------------------ mode

/// The mode of every pairing, with `MESSAGES_E2EE_REQUIRED` set to `required`.
async fn check_modes(db: PgPool, required: bool) {
    let app = app_for(&db, required);
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let mut cy = app.person(&db, "cy").await;
    let mut dee = app.person(&db, "dee").await;
    let ana_key = turn_on(&mut ana, 'Q').await;
    let bob_key = turn_on(&mut bob, 'R').await;
    let remote = remote_user(&db, "eli", "a.example").await;
    let unkeyed_ok = if required { "unavailable" } else { "plain" };

    let v = view(&mut ana, "eli@a.example").await;
    assert_eq!((&v["mode"], &v["peerKey"]), (&json!("plain"), &Value::Null));
    let v = view(&mut ana, "bob").await;
    assert_eq!(
        (&v["mode"], &v["peerKey"]),
        (&json!("sealed"), &json!(bob_key))
    );
    let v = view(&mut bob, "ana").await;
    assert_eq!(
        (&v["mode"], &v["peerKey"]),
        (&json!("sealed"), &json!(ana_key))
    );
    // The recipient has no key.
    let v = view(&mut ana, "cy").await;
    assert_eq!(
        (&v["mode"], &v["peerKey"]),
        (&json!(unkeyed_ok), &Value::Null)
    );
    // The sender has no key but the recipient does: plaintext is never allowed.
    let v = view(&mut cy, "ana").await;
    assert_eq!(
        (&v["mode"], &v["peerKey"]),
        (&json!("unavailable"), &json!(ana_key))
    );
    // Neither has one.
    let v = view(&mut cy, "dee").await;
    assert_eq!(
        (&v["mode"], &v["peerKey"]),
        (&json!(unkeyed_ok), &Value::Null)
    );
    let v = view(&mut dee, "cy").await;
    assert_eq!(v["mode"], unkeyed_ok);

    // The list carries each peer's key, one per row.
    legacy_message(&db, bob.id, ana.id, "old one").await;
    legacy_message(&db, cy.id, ana.id, "old two").await;
    legacy_message(&db, remote, ana.id, "old three").await;
    let list = ana.browser.get("/conversations").await.ok();
    let keys: std::collections::BTreeMap<&str, &Value> = list["items"]
        .as_array()
        .expect("items")
        .iter()
        .map(|item| {
            (
                item["user"]["handle"].as_str().expect("handle"),
                &item["peerKey"],
            )
        })
        .collect();
    let bob_key = json!(bob_key);
    let expected: std::collections::BTreeMap<&str, &Value> = [
        ("bob", &bob_key),
        ("cy", &Value::Null),
        ("eli", &Value::Null),
    ]
    .into_iter()
    .collect();
    assert_eq!(keys, expected);
}

#[sqlx::test(migrations = "./migrations")]
async fn mode_follows_the_keys_when_encryption_is_required(db: PgPool) {
    check_modes(db, true).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn mode_follows_the_keys_when_plaintext_is_allowed(db: PgPool) {
    check_modes(db, false).await;
}

// ------------------------------------------------------------------ send

#[sqlx::test(migrations = "./migrations")]
async fn send_stores_the_ciphertext_and_no_text(db: PgPool) {
    let app = app_for(&db, true);
    let mut p = keyed_pair(&app, &db).await;
    let sealed = sealed(&p.ana_key, &p.bob_key);
    let sent = p
        .ana
        .browser
        .post("/conversations/bob", json!({ "ciphertext": sealed }))
        .await
        .ok();
    assert_eq!(sent["text"], "");
    assert_eq!(sent["ciphertext"], sealed);
    assert_eq!(sent["emojis"], json!([]));
    assert_eq!(sent["mine"], true);
    assert_eq!(
        rows(&db).await,
        [(String::new(), Some(sealed.clone()), false)]
    );
    let queued: i64 = sqlx::query_scalar("select count(*) from ap_deliveries")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(queued, 0);

    let seen = view(&mut p.bob, "ana").await;
    assert_eq!(seen["messages"][0]["ciphertext"], sealed);
    assert_eq!(seen["messages"][0]["text"], "");
    assert_eq!(seen["messages"][0]["mine"], false);
}

/// Text, text with ciphertext and an empty body are all refused in a sealed conversation.
async fn check_text_refused(db: PgPool, required: bool) {
    let app = app_for(&db, required);
    let mut p = keyed_pair(&app, &db).await;
    let both = json!({ "text": "hello", "ciphertext": sealed(&p.ana_key, &p.bob_key) });
    for body in [json!({ "text": "hello" }), both, json!({})] {
        p.ana
            .browser
            .post("/conversations/bob", body)
            .await
            .err(StatusCode::UNPROCESSABLE_ENTITY, MUST_BE_SEALED);
    }
    assert!(rows(&db).await.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn send_refuses_text_to_a_sealed_conversation(db: PgPool) {
    check_text_refused(db, true).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn send_refuses_text_to_a_sealed_conversation_even_when_plaintext_is_allowed(db: PgPool) {
    check_text_refused(db, false).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn send_never_sends_plaintext_to_a_recipient_with_a_key(db: PgPool) {
    let app = app_for(&db, false);
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    turn_on(&mut bob, 'R').await;
    ana.browser
        .post("/conversations/bob", json!({ "text": "hello" }))
        .await
        .err(StatusCode::CONFLICT, "Turn on encrypted messages first.");
    assert!(rows(&db).await.is_empty());

    // The insert checks it too, in case the key turned up after the mode was decided.
    let refused = messages::send(
        &db,
        app.state.config(),
        ana.id,
        bob.id,
        messages::Content::Plain("hello"),
    )
    .await
    .expect_err("refused");
    assert_eq!(refused.to_string(), STALE);
    assert!(rows(&db).await.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn send_refuses_a_stale_key_part(db: PgPool) {
    let app = app_for(&db, true);
    let mut p = keyed_pair(&app, &db).await;
    let (a, b) = (p.ana_key.clone(), p.bob_key.clone());
    let post = async |browser: &mut Browser, sender: &str, recipient: &str| {
        browser
            .post(
                "/conversations/bob",
                json!({ "ciphertext": sealed(sender, recipient) }),
            )
            .await
    };
    // The sender's key part is not her current key.
    post(&mut p.ana.browser, &key('Z'), &b)
        .await
        .err(StatusCode::CONFLICT, STALE);
    // The recipient's key part is not his current key.
    post(&mut p.ana.browser, &a, &key('Z'))
        .await
        .err(StatusCode::CONFLICT, STALE);
    // Parts swapped.
    post(&mut p.ana.browser, &b, &a)
        .await
        .err(StatusCode::CONFLICT, STALE);
    // Bob replaced his key since ana loaded it.
    put_key(
        &mut p.bob.browser,
        json!({ "publicKey": key('S'), "backup": BACKUP, "replace": true }),
    )
    .await
    .ok();
    post(&mut p.ana.browser, &a, &b)
        .await
        .err(StatusCode::CONFLICT, STALE);
    assert!(rows(&db).await.is_empty());
    assert_eq!(view(&mut p.ana, "bob").await["peerKey"], key('S'));
    post(&mut p.ana.browser, &a, &key('S')).await.ok();
    assert_eq!(rows(&db).await.len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn send_refuses_a_malformed_ciphertext(db: PgPool) {
    let app = app_for(&db, true);
    let mut p = keyed_pair(&app, &db).await;
    let (a, b) = (p.ana_key.clone(), p.bob_key.clone());
    let too_long = format!("v1.{a}.{b}.c2FsdA.aXY.{}", "c".repeat(16 * 1024));
    let bad = [
        String::new(),
        "hello".to_owned(),
        format!("v2.{a}.{b}.c2FsdA.aXY.Y2lw"),
        format!("v1.{a}.{b}.c2FsdA.aXY"),
        format!("v1.{a}.{b}.c2FsdA.aXY.Y2lw.extra"),
        format!("v1.{a}.{b}.c2FsdA..Y2lw"),
        format!("v1.{a}.{b}.c2FsdA.aXY.not base64"),
        too_long,
    ];
    for ciphertext in bad {
        let reply = p
            .ana
            .browser
            .post("/conversations/bob", json!({ "ciphertext": ciphertext }))
            .await;
        assert_eq!(
            reply.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{ciphertext:.60}"
        );
    }
    assert!(rows(&db).await.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn send_is_unavailable_until_both_sides_have_a_key(db: PgPool) {
    let app = app_for(&db, true);
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let mut cy = app.person(&db, "cy").await;
    let ana_key = turn_on(&mut ana, 'Q').await;
    let theirs = "@bob hasn't turned on encrypted messages yet.";
    let mine = "Turn on encrypted messages first.";
    for body in [
        json!({ "text": "hi" }),
        json!({ "ciphertext": sealed(&ana_key, &key('R')) }),
    ] {
        ana.browser
            .post("/conversations/bob", body)
            .await
            .err(StatusCode::CONFLICT, theirs);
    }
    turn_on(&mut bob, 'R').await;
    for body in [
        json!({ "text": "hi" }),
        json!({ "ciphertext": sealed(&key('C'), &ana_key) }),
    ] {
        cy.browser
            .post("/conversations/ana", body)
            .await
            .err(StatusCode::CONFLICT, mine);
    }
    assert!(rows(&db).await.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn send_takes_text_when_plaintext_is_allowed_and_refuses_ciphertext(db: PgPool) {
    let app = app_for(&db, false);
    let mut ana = app.person(&db, "ana").await;
    app.person(&db, "bob").await;
    remote_user(&db, "eli", "a.example").await;
    let sent = ana
        .browser
        .post("/conversations/bob", json!({ "text": "hello" }))
        .await
        .ok();
    assert_eq!(sent["text"], "hello");
    assert_eq!(sent["ciphertext"], Value::Null);
    let any = sealed(&key('Q'), &key('R'));
    for handle in ["bob", "eli@a.example"] {
        ana.browser
            .post(
                &format!("/conversations/{handle}"),
                json!({ "ciphertext": any }),
            )
            .await
            .err(StatusCode::UNPROCESSABLE_ENTITY, NOT_ENCRYPTED);
    }
    ana.browser
        .post("/conversations/eli@a.example", json!({ "text": "psst" }))
        .await
        .ok();
    assert_eq!(rows(&db).await.len(), 2);
}

// ------------------------------------------------------------------ surfaces

#[sqlx::test(migrations = "./migrations")]
async fn surfaces_carry_no_sealed_text(db: PgPool) {
    let app = app_for(&db, true);
    let mut p = keyed_pair(&app, &db).await;
    let legacy = legacy_message(&db, p.bob.id, p.ana.id, "an old plain note").await;
    let mut bob_events = app.state.hub().subscribe(p.bob.id);
    let sealed = sealed(&p.ana_key, &p.bob_key);
    p.ana
        .browser
        .post("/conversations/bob", json!({ "ciphertext": sealed }))
        .await
        .ok();

    // The live event has no preview.
    let events = drain(&mut bob_events);
    let Some(StreamEvent::Message(event)) = events.first() else {
        panic!("expected a message event, got {events:?}");
    };
    assert_eq!(event.text, "");
    assert!(event.encrypted);

    // The thread and the list carry the ciphertext and an empty text; the legacy row is as it was.
    let thread = view(&mut p.ana, "bob").await;
    let messages = thread["messages"].as_array().expect("messages");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["id"], legacy.to_string());
    assert_eq!(messages[0]["text"], "an old plain note");
    assert_eq!(messages[0]["ciphertext"], Value::Null);
    assert_eq!(messages[1]["text"], "");
    assert_eq!(messages[1]["ciphertext"], sealed);
    let list = p.ana.browser.get("/conversations").await.ok();
    assert_eq!(list["items"][0]["last"]["text"], "");
    assert_eq!(list["items"][0]["last"]["ciphertext"], sealed);

    // The export is made on the server: ciphertext only.
    let export = p.ana.browser.get("/me/export").await.ok();
    let exported = export["messages"].as_array().expect("messages");
    assert_eq!(exported.len(), 2);
    assert_eq!(exported[0]["text"], "an old plain note");
    assert_eq!(exported[0]["ciphertext"], Value::Null);
    assert_eq!(exported[1]["text"], "");
    assert_eq!(exported[1]["ciphertext"], sealed);

    // The database never saw the text of the sealed message.
    let texts: Vec<String> =
        sqlx::query_scalar("select text from messages where ciphertext is not null")
            .fetch_all(&db)
            .await
            .expect("texts");
    assert_eq!(texts, [String::new()]);
}

#[sqlx::test(migrations = "./migrations")]
async fn surfaces_show_the_mastodon_api_a_placeholder(db: PgPool) {
    let env = Env::with(db.clone(), |var| {
        (var == "MESSAGES_E2EE_REQUIRED").then(|| "true".to_owned())
    });
    let mut p = keyed_pair(&env.app, &db).await;
    legacy_message(&db, p.bob.id, p.ana.id, "an old plain note").await;
    let sealed = sealed(&p.ana_key, &p.bob_key);
    p.ana
        .browser
        .post("/conversations/bob", json!({ "ciphertext": sealed }))
        .await
        .ok();
    let mut api = sign_in(&env.app, &mut p.ana, "read write", [203, 0, 113, 10]).await;

    let reply = api.get("/api/v1/conversations").await;
    let body = String::from_utf8_lossy(&reply.bytes).into_owned();
    assert!(!body.contains("Y2lwaGVy"), "the ciphertext leaked: {body}");
    let list = reply.ok();
    assert_eq!(list[0]["last_status"]["content"], SEALED_CONTENT);
    assert_eq!(list[0]["last_status"]["visibility"], "direct");

    // Once the sealed message is gone, the plaintext one shows as it always did.
    let id = list[0]["id"].as_str().expect("id").to_owned();
    api.delete(&format!("/api/v1/conversations/{id}"))
        .await
        .ok();
    let after = api.get("/api/v1/conversations").await.ok();
    assert!(
        after[0]["last_status"]["content"]
            .as_str()
            .expect("content")
            .contains("an old plain note")
    );
}

// ------------------------------------------------------------------ delete

#[sqlx::test(migrations = "./migrations")]
async fn delete_blanks_the_text_and_the_ciphertext(db: PgPool) {
    let app = app_for(&db, true);
    let mut p = keyed_pair(&app, &db).await;
    let legacy = legacy_message(&db, p.ana.id, p.bob.id, "an old plain note").await;
    let sealed = sealed(&p.ana_key, &p.bob_key);
    let sent = p
        .ana
        .browser
        .post("/conversations/bob", json!({ "ciphertext": sealed }))
        .await
        .ok();
    let id = sent["id"].as_str().expect("id");
    p.ana
        .browser
        .delete(&format!("/conversations/bob/messages/{id}"))
        .await
        .ok();
    assert_eq!(
        rows(&db).await,
        [
            ("an old plain note".to_owned(), None, false),
            (String::new(), None, true)
        ]
    );
    p.ana
        .browser
        .delete(&format!("/conversations/bob/messages/{legacy}"))
        .await
        .ok();
    assert_eq!(
        rows(&db).await,
        [(String::new(), None, true), (String::new(), None, true)]
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn delete_of_a_conversation_over_the_mastodon_api_scrubs_what_the_viewer_sent(db: PgPool) {
    let env = Env::with(db.clone(), |var| {
        (var == "MESSAGES_E2EE_REQUIRED").then(|| "true".to_owned())
    });
    let mut p = keyed_pair(&env.app, &db).await;
    legacy_message(&db, p.ana.id, p.bob.id, "an old plain note").await;
    legacy_message(&db, p.bob.id, p.ana.id, "bob's old note").await;
    p.ana
        .browser
        .post(
            "/conversations/bob",
            json!({ "ciphertext": sealed(&p.ana_key, &p.bob_key) }),
        )
        .await
        .ok();
    p.bob
        .browser
        .post(
            "/conversations/ana",
            json!({ "ciphertext": sealed(&p.bob_key, &p.ana_key) }),
        )
        .await
        .ok();
    let mut api = sign_in(&env.app, &mut p.ana, "read write", [203, 0, 113, 10]).await;
    api.delete(&format!("/api/v1/conversations/{}", p.bob.id))
        .await
        .ok();

    let by = |sender: Uuid| {
        sqlx::query_as::<_, (String, bool, bool)>(
            "select text, ciphertext is not null, deleted_at is not null from messages
              where sender_id = $1 order by id",
        )
        .bind(sender)
        .fetch_all(&db)
    };
    assert_eq!(
        by(p.ana.id).await.expect("ana's"),
        [(String::new(), false, true), (String::new(), false, true)]
    );
    // What bob sent is his, and stays.
    assert_eq!(
        by(p.bob.id).await.expect("bob's"),
        [
            ("bob's old note".to_owned(), false, false),
            (String::new(), true, false)
        ]
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn delete_by_a_remote_account_blanks_the_message(db: PgPool) {
    let app = app_for(&db, false);
    let ana = app.person(&db, "ana").await;
    let eli = remote_user(&db, "eli", "a.example").await;
    let note = "https://a.example/notes/1";
    let now = time::OffsetDateTime::now_utc();
    remote::insert_message(&db, eli, ana.id, note, "a secret", now)
        .await
        .expect("stored")
        .expect("new");
    // A row that somehow holds a ciphertext is blanked too.
    sqlx::query("update messages set text = '', ciphertext = 'v1.a.b.c.d.e' where ap_id = $1")
        .bind(note)
        .execute(&db)
        .await
        .expect("seal");
    let fay = remote_user(&db, "fay", "b.example").await;
    let hub = app.state.hub();
    assert!(
        !remote::delete_by_ap_id(&db, hub, fay, "https://b.example/users/fay", note)
            .await
            .expect("ignored"),
        "another account cannot delete it"
    );
    assert_eq!(
        rows(&db).await,
        [(String::new(), Some("v1.a.b.c.d.e".to_owned()), false)]
    );

    assert!(
        remote::delete_by_ap_id(&db, hub, eli, "https://a.example/users/eli", note)
            .await
            .expect("deleted")
    );
    assert_eq!(rows(&db).await, [(String::new(), None, true)]);

    // Plain text is blanked as well.
    let plain = "https://a.example/notes/2";
    remote::insert_message(&db, eli, ana.id, plain, "another secret", now)
        .await
        .expect("stored")
        .expect("new");
    remote::delete_by_ap_id(&db, hub, eli, "https://a.example/users/eli", plain)
        .await
        .expect("deleted");
    assert_eq!(
        rows(&db).await,
        [(String::new(), None, true), (String::new(), None, true)]
    );
}

/// The queued activities, oldest first.
async fn queue(db: &PgPool) -> Vec<(String, Value, Option<Uuid>)> {
    sqlx::query_as("select inbox_url, activity, signer_id from ap_deliveries order by id")
        .fetch_all(db)
        .await
        .expect("queue")
}

#[sqlx::test(migrations = "./migrations")]
async fn delete_cancels_the_queued_create_and_tells_the_remote_inbox(db: PgPool) {
    let app = app_for(&db, false);
    let mut ana = app.person(&db, "ana").await;
    remote_user(&db, "bob", "a.example").await;
    let sent = ana
        .browser
        .post("/conversations/bob@a.example", json!({ "text": "psst" }))
        .await
        .ok();
    let queued = queue(&db).await;
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0].1["type"], "Create");
    let note_id = queued[0].1["object"]["id"]
        .as_str()
        .expect("note id")
        .to_owned();

    let id = sent["id"].as_str().expect("id");
    ana.browser
        .delete(&format!("/conversations/bob@a.example/messages/{id}"))
        .await
        .ok();
    let queued = queue(&db).await;
    assert_eq!(
        queued.len(),
        1,
        "the Create is gone and a Delete is queued: {queued:?}"
    );
    let (inbox, activity, signer) = &queued[0];
    assert_eq!(inbox, "https://a.example/users/bob/inbox");
    assert_eq!(activity["type"], "Delete");
    assert_eq!(activity["actor"], format!("{}/users/ana", common::ORIGIN));
    assert_eq!(
        activity["object"],
        json!({ "id": note_id, "type": "Tombstone" })
    );
    assert_eq!(activity["to"], json!(["https://a.example/users/bob"]));
    assert_eq!(*signer, Some(ana.id));
    assert_eq!(rows(&db).await, [(String::new(), None, true)]);
}

#[sqlx::test(migrations = "./migrations")]
async fn delete_of_a_delivered_remote_message_still_queues_a_delete(db: PgPool) {
    let env = Env::with(db.clone(), |var| {
        (var == "MESSAGES_E2EE_REQUIRED").then(|| "false".to_owned())
    });
    let mut ana = env.app.person(&db, "ana").await;
    let bob = remote_user(&db, "bob", "a.example").await;
    ana.browser
        .post("/conversations/bob@a.example", json!({ "text": "psst" }))
        .await
        .ok();
    // The worker delivered the Create: it left the queue.
    sqlx::query("delete from ap_deliveries")
        .execute(&db)
        .await
        .expect("clear");
    let mut api = sign_in(&env.app, &mut ana, "read write", [203, 0, 113, 10]).await;
    api.delete(&format!("/api/v1/conversations/{bob}"))
        .await
        .ok();

    let queued = queue(&db).await;
    assert_eq!(queued.len(), 1, "{queued:?}");
    assert_eq!(queued[0].1["type"], "Delete");
    assert_eq!(queued[0].1["object"]["type"], "Tombstone");
    assert_eq!(rows(&db).await, [(String::new(), None, true)]);
}
