#![allow(missing_docs)]

mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::sync::broadcast::Receiver;
use tuita_server::dto::{BadgesDto, ReadKind, StreamEvent};
use tuita_server::realtime::Event;
use uuid::Uuid;

/// Events waiting on a subscription, in arrival order.
fn drain(rx: &mut Receiver<Event>) -> Vec<StreamEvent> {
    let mut out = Vec::new();
    while let Ok(event) = rx.try_recv() {
        out.push((*event).clone());
    }
    out
}

fn badges(messages: i64) -> StreamEvent {
    StreamEvent::Badges(BadgesDto {
        notifications: 0,
        messages,
        reports: None,
    })
}

async fn set_policy(db: &PgPool, handle: &str, policy: &str) {
    sqlx::query("update users set dm_policy = $2 where handle = $1")
        .bind(handle)
        .bind(policy)
        .execute(db)
        .await
        .expect("set dm policy");
}

fn texts(messages: &Value) -> Vec<&str> {
    messages
        .as_array()
        .expect("messages")
        .iter()
        .map(|message| message["text"].as_str().expect("text"))
        .collect()
}

#[sqlx::test(migrations = "./migrations")]
async fn a_recipient_who_accepts_no_messages_is_refused(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    app.person(&db, "bob").await;
    set_policy(&db, "bob", "none").await;

    ana.browser
        .post("/conversations/bob", json!({"text": "hi"}))
        .await
        .err(
            StatusCode::FORBIDDEN,
            "@bob isn\u{2019}t accepting messages.",
        );
    let view = ana.browser.get("/conversations/bob").await.ok();
    assert_eq!(
        view["blockedReason"],
        "@bob isn\u{2019}t accepting messages."
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn the_following_policy_refuses_strangers_until_they_are_followed(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let bob = app.person(&db, "bob").await;
    set_policy(&db, "bob", "following").await;
    let refused = "@bob only accepts messages from people they follow.";

    ana.browser
        .post("/conversations/bob", json!({"text": "hi"}))
        .await
        .err(StatusCode::FORBIDDEN, refused);
    assert_eq!(
        ana.browser.get("/conversations/bob").await.ok()["blockedReason"],
        refused
    );

    sqlx::query("insert into follows (follower_id, followee_id) values ($1, $2)")
        .bind(bob.id)
        .bind(ana.id)
        .execute(&db)
        .await
        .expect("follow");
    ana.browser
        .post("/conversations/bob", json!({"text": "hi"}))
        .await
        .ok();
    assert_eq!(
        ana.browser.get("/conversations/bob").await.ok()["blockedReason"],
        Value::Null
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_conversation_they_started_stays_open_under_the_following_policy(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    bob.browser
        .post("/conversations/ana", json!({"text": "first"}))
        .await
        .ok();
    set_policy(&db, "bob", "following").await;
    ana.browser
        .post("/conversations/bob", json!({"text": "reply"}))
        .await
        .ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn blocks_and_yourself_are_refused(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    ana.browser
        .post("/conversations/ana", json!({"text": "me"}))
        .await
        .err(StatusCode::FORBIDDEN, "You can\u{2019}t message yourself.");
    sqlx::query("insert into blocks (user_id, target_id) values ($1, $2)")
        .bind(ana.id)
        .bind(bob.id)
        .execute(&db)
        .await
        .expect("block");
    let message = "You can\u{2019}t message this account.";
    ana.browser
        .post("/conversations/bob", json!({"text": "hi"}))
        .await
        .err(StatusCode::FORBIDDEN, message);
    bob.browser
        .post("/conversations/ana", json!({"text": "hi"}))
        .await
        .err(StatusCode::FORBIDDEN, message);
    ana.browser
        .post("/conversations/nobody", json!({"text": "hi"}))
        .await
        .err(StatusCode::NOT_FOUND, "User not found.");
}

#[sqlx::test(migrations = "./migrations")]
async fn messages_are_one_to_two_thousand_characters(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    app.person(&db, "bob").await;
    ana.browser
        .post("/conversations/bob", json!({"text": "   "}))
        .await
        .err(StatusCode::BAD_REQUEST, "Message is required.");
    ana.browser
        .post("/conversations/bob", json!({"text": "a".repeat(2001)}))
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "Message must be at most 2000 characters.",
        );
    let sent = ana
        .browser
        .post(
            "/conversations/bob",
            json!({"text": format!("  {}  ", "a".repeat(2000))}),
        )
        .await
        .ok();
    assert_eq!(sent["text"], "a".repeat(2000));
}

#[sqlx::test(migrations = "./migrations")]
async fn sending_stores_the_message_and_tells_the_recipient(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let mut bob_events = app.state.hub().subscribe(bob.id);
    let mut ana_events = app.state.hub().subscribe(ana.id);

    let sent = ana
        .browser
        .post("/conversations/bob", json!({"text": "x".repeat(300)}))
        .await
        .ok();
    assert_eq!(sent["mine"], true);
    assert_eq!(sent["readAt"], Value::Null);
    assert!(Uuid::parse_str(sent["id"].as_str().expect("id")).is_ok());

    let events = drain(&mut bob_events);
    assert_eq!(events.len(), 2, "{events:?}");
    let StreamEvent::Message(message) = &events[0] else {
        panic!("expected a message event, got {:?}", events[0]);
    };
    assert_eq!(message.from.handle, "ana");
    assert_eq!(message.text, "x".repeat(280));
    assert_eq!(events[1], badges(1));
    assert!(
        drain(&mut ana_events).is_empty(),
        "the sender hears nothing"
    );

    let counts = bob.browser.get("/badges").await.ok();
    assert_eq!(counts["messages"], 1);

    let view = bob.browser.get("/conversations/ana").await.ok();
    assert_eq!(view["user"]["handle"], "ana");
    assert_eq!(view["messages"][0]["mine"], false);
    assert_eq!(view["messages"][0]["text"], "x".repeat(300));
    assert_eq!(view["blockedReason"], Value::Null);
    assert_eq!(
        bob.browser.get("/badges").await.ok()["messages"],
        1,
        "opening a conversation does not mark it read"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn reading_marks_messages_and_tells_only_the_reader(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    for text in ["one", "two"] {
        ana.browser
            .post("/conversations/bob", json!({"text": text}))
            .await
            .ok();
    }
    // bob's own message to ana stays untouched by his reading.
    bob.browser
        .post("/conversations/ana", json!({"text": "mine"}))
        .await
        .ok();
    let mut bob_events = app.state.hub().subscribe(bob.id);
    let mut ana_events = app.state.hub().subscribe(ana.id);

    assert_eq!(bob.browser.get("/badges").await.ok()["messages"], 2);
    assert_eq!(
        bob.browser.post_empty("/conversations/ana/read").await.ok(),
        json!({"ok": true})
    );
    let unread: i64 = sqlx::query_scalar(
        "select count(*) from messages where recipient_id = $1 and read_at is null",
    )
    .bind(bob.id)
    .fetch_one(&db)
    .await
    .expect("count");
    assert_eq!(unread, 0);
    let ana_unread: i64 = sqlx::query_scalar(
        "select count(*) from messages where recipient_id = $1 and read_at is null",
    )
    .bind(ana.id)
    .fetch_one(&db)
    .await
    .expect("count");
    assert_eq!(ana_unread, 1);

    assert_eq!(
        drain(&mut bob_events),
        [
            StreamEvent::Read(tuita_server::dto::ReadEventDto {
                kind: ReadKind::Messages
            }),
            badges(0)
        ]
    );
    assert!(drain(&mut ana_events).is_empty());
    assert_eq!(bob.browser.get("/badges").await.ok()["messages"], 0);

    // Nothing left to read: ok, and no event.
    bob.browser.post_empty("/conversations/ana/read").await.ok();
    assert!(drain(&mut bob_events).is_empty());

    let view = ana.browser.get("/conversations/bob").await.ok();
    assert_eq!(view["messages"][0]["text"], "one");
    assert_ne!(view["messages"][0]["readAt"], Value::Null);
    assert_eq!(view["messages"][2]["text"], "mine");
    assert_eq!(view["messages"][2]["readAt"], Value::Null);
    bob.browser
        .post_empty("/conversations/nobody/read")
        .await
        .err(StatusCode::NOT_FOUND, "User not found.");
}

#[sqlx::test(migrations = "./migrations")]
async fn only_the_sender_deletes_and_deleted_messages_disappear(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let first = ana
        .browser
        .post("/conversations/bob", json!({"text": "keep"}))
        .await
        .ok();
    let second = ana
        .browser
        .post("/conversations/bob", json!({"text": "oops"}))
        .await
        .ok();
    let id = second["id"].as_str().expect("id").to_owned();
    assert_eq!(bob.browser.get("/badges").await.ok()["messages"], 2);
    let mut bob_events = app.state.hub().subscribe(bob.id);

    bob.browser
        .delete(&format!("/conversations/ana/messages/{id}"))
        .await
        .err(StatusCode::NOT_FOUND, "Message not found.");
    ana.browser
        .delete("/conversations/bob/messages/not-a-uuid")
        .await
        .err(StatusCode::NOT_FOUND, "Message not found.");
    assert_eq!(
        ana.browser
            .delete(&format!("/conversations/bob/messages/{id}"))
            .await
            .ok(),
        json!({"ok": true})
    );
    ana.browser
        .delete(&format!("/conversations/bob/messages/{id}"))
        .await
        .err(StatusCode::NOT_FOUND, "Message not found.");

    let seen_by_ana = ana.browser.get("/conversations/bob").await.ok();
    assert_eq!(texts(&seen_by_ana["messages"]), ["keep"]);
    let seen_by_bob = bob.browser.get("/conversations/ana").await.ok();
    assert_eq!(texts(&seen_by_bob["messages"]), ["keep"]);
    assert_eq!(bob.browser.get("/badges").await.ok()["messages"], 1);
    assert_eq!(drain(&mut bob_events), [badges(1)]);
    let list = bob.browser.get("/conversations").await.ok();
    assert_eq!(list["items"][0]["last"]["id"], first["id"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn deleting_a_message_blanks_its_text_in_the_database(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    app.person(&db, "bob").await;
    let kept = ana
        .browser
        .post("/conversations/bob", json!({"text": "keep this"}))
        .await
        .ok();
    let gone = ana
        .browser
        .post("/conversations/bob", json!({"text": "secret words"}))
        .await
        .ok();
    let id = gone["id"].as_str().expect("id");
    ana.browser
        .delete(&format!("/conversations/bob/messages/{id}"))
        .await
        .ok();

    let rows: Vec<(String, bool)> =
        sqlx::query_as("select text, deleted_at is not null from messages order by id")
            .fetch_all(&db)
            .await
            .expect("rows");
    assert_eq!(
        rows,
        [("keep this".to_owned(), false), (String::new(), true)]
    );
    assert_ne!(kept["id"], gone["id"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_suspended_or_deleted_recipient_is_refused(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let bob = app.person(&db, "bob").await;
    sqlx::query("update users set role = 'admin' where id = $1")
        .bind(ana.id)
        .execute(&db)
        .await
        .expect("admin");
    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(bob.id)
        .execute(&db)
        .await
        .expect("suspend");
    let refused = "You can\u{2019}t message this account.";

    // Staff can still open a suspended account's conversation, but not write to it.
    assert_eq!(
        ana.browser.get("/conversations/bob").await.ok()["blockedReason"],
        refused
    );
    ana.browser
        .post("/conversations/bob", json!({"text": "hi"}))
        .await
        .err(StatusCode::FORBIDDEN, refused);
    let stored: i64 = sqlx::query_scalar("select count(*) from messages")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(stored, 0);

    sqlx::query("update users set suspended_at = null, deleted_at = now() where id = $1")
        .bind(bob.id)
        .execute(&db)
        .await
        .expect("delete");
    let row = tuita_server::db::users::by_id(&db, bob.id)
        .await
        .expect("loads")
        .expect("the row stays");
    let reason = tuita_server::db::messages::blocked_reason(&db, ana.id, &row)
        .await
        .expect("checks");
    assert_eq!(reason.as_deref(), Some(refused));
}

#[sqlx::test(migrations = "./migrations")]
async fn the_list_is_newest_first_with_unread_counts_and_pages(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let mut cat = app.person(&db, "cat").await;
    app.person(&db, "dan").await;
    for text in ["b1", "b2"] {
        bob.browser
            .post("/conversations/ana", json!({"text": text}))
            .await
            .ok();
    }
    cat.browser
        .post("/conversations/ana", json!({"text": "c1"}))
        .await
        .ok();
    ana.browser
        .post("/conversations/dan", json!({"text": "d1"}))
        .await
        .ok();

    let list = ana.browser.get("/conversations").await.ok();
    let rows = list["items"].as_array().expect("items");
    let summary: Vec<(&str, i64, &str, bool)> = rows
        .iter()
        .map(|row| {
            (
                row["user"]["handle"].as_str().expect("handle"),
                row["unread"].as_i64().expect("unread"),
                row["last"]["text"].as_str().expect("text"),
                row["last"]["mine"].as_bool().expect("mine"),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            ("dan", 0, "d1", true),
            ("cat", 1, "c1", false),
            ("bob", 2, "b2", false)
        ]
    );
    assert_eq!(list["next"], Value::Null);

    let first = ana.browser.get("/conversations?limit=2").await.ok();
    assert_eq!(first["items"].as_array().expect("items").len(), 2);
    assert_eq!(first["items"][1]["user"]["handle"], "cat");
    assert_eq!(first["next"], first["items"][1]["last"]["id"]);
    let next = first["next"].as_str().expect("cursor");
    let second = ana
        .browser
        .get(&format!("/conversations?limit=2&before={next}"))
        .await
        .ok();
    assert_eq!(second["items"].as_array().expect("items").len(), 1);
    assert_eq!(second["items"][0]["user"]["handle"], "bob");
    assert_eq!(second["next"], Value::Null);
    ana.browser
        .get("/conversations?before=nope")
        .await
        .err(StatusCode::BAD_REQUEST, "Invalid cursor.");
}

#[sqlx::test(migrations = "./migrations")]
async fn the_list_leaves_out_blocked_and_suspended_people_but_keeps_muted_ones(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    for handle in ["bob", "cat", "dan", "eve"] {
        let mut other = app.person(&db, handle).await;
        other
            .browser
            .post("/conversations/ana", json!({"text": handle}))
            .await
            .ok();
    }
    let id = |handle: &'static str| {
        let db = db.clone();
        async move {
            sqlx::query_scalar::<_, Uuid>("select id from users where handle = $1")
                .bind(handle)
                .fetch_one(&db)
                .await
                .expect("user")
        }
    };
    let (bob, cat, dan) = (id("bob").await, id("cat").await, id("dan").await);
    sqlx::query("insert into blocks (user_id, target_id) values ($1, $2)")
        .bind(ana.id)
        .bind(bob)
        .execute(&db)
        .await
        .expect("block");
    sqlx::query("insert into blocks (user_id, target_id) values ($1, $2)")
        .bind(cat)
        .bind(ana.id)
        .execute(&db)
        .await
        .expect("blocked by");
    sqlx::query("update users set suspended_at = now() where id = $1")
        .bind(dan)
        .execute(&db)
        .await
        .expect("suspend");
    sqlx::query("insert into mutes (user_id, target_id) values ($1, $2)")
        .bind(ana.id)
        .bind(id("eve").await)
        .execute(&db)
        .await
        .expect("mute");

    let list = ana.browser.get("/conversations").await.ok();
    let handles: Vec<&str> = list["items"]
        .as_array()
        .expect("items")
        .iter()
        .map(|row| row["user"]["handle"].as_str().expect("handle"))
        .collect();
    assert_eq!(handles, ["eve"]);
    // Their unread messages do not count either, except the muted account's.
    assert_eq!(ana.browser.get("/badges").await.ok()["messages"], 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_conversation_pages_back_and_forward_oldest_first(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let mut ids = Vec::new();
    for n in 1..=5 {
        let sender = if n % 2 == 1 { &mut ana } else { &mut bob };
        let to = if n % 2 == 1 { "bob" } else { "ana" };
        let sent = sender
            .browser
            .post(
                &format!("/conversations/{to}"),
                json!({"text": format!("m{n}")}),
            )
            .await
            .ok();
        ids.push(sent["id"].as_str().expect("id").to_owned());
    }

    let newest = ana.browser.get("/conversations/bob?limit=2").await.ok();
    assert_eq!(texts(&newest["messages"]), ["m4", "m5"]);
    assert_eq!(newest["messages"][1]["mine"], true);
    assert_eq!(newest["next"], ids[3]);
    let middle = ana
        .browser
        .get(&format!("/conversations/bob?limit=2&before={}", ids[3]))
        .await
        .ok();
    assert_eq!(texts(&middle["messages"]), ["m2", "m3"]);
    assert_eq!(middle["next"], ids[1]);
    let oldest = ana
        .browser
        .get(&format!("/conversations/bob?limit=2&before={}", ids[1]))
        .await
        .ok();
    assert_eq!(texts(&oldest["messages"]), ["m1"]);
    assert_eq!(oldest["next"], Value::Null);

    let newer = ana
        .browser
        .get(&format!("/conversations/bob?after={}", ids[2]))
        .await
        .ok();
    assert_eq!(texts(&newer["messages"]), ["m4", "m5"]);
    ana.browser
        .get("/conversations/bob?after=nope")
        .await
        .err(StatusCode::BAD_REQUEST, "Invalid cursor.");
}

#[sqlx::test(migrations = "./migrations")]
async fn the_scheduler_worker_publishes_within_six_seconds_of_the_due_time(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let at = (time::OffsetDateTime::now_utc() + time::Duration::minutes(5))
        .format(&time::format_description::well_known::Rfc3339)
        .expect("format");
    let made = ana
        .browser
        .post("/scheduled", json!({ "text": "on time", "publishAt": at }))
        .await
        .ok();
    let id: Uuid = made["id"].as_str().expect("id").parse().expect("uuid");

    let stop = tokio_util::sync::CancellationToken::new();
    let mut workers = tuita_server::workers::spawn_workers(&app.state, &stop);
    let due = sqlx::query_scalar::<_, time::OffsetDateTime>(
        "update scheduled_posts set publish_at = now() + interval '1 second'
          where id = $1 returning publish_at",
    )
    .bind(id)
    .fetch_one(&db)
    .await
    .expect("set due time");

    let mut published = None;
    for _ in 0..70 {
        let row: (String, Option<Uuid>) =
            sqlx::query_as("select status, post_id from scheduled_posts where id = $1")
                .bind(id)
                .fetch_one(&db)
                .await
                .expect("row");
        if row.0 == "published" {
            published = row.1;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    let late = time::OffsetDateTime::now_utc() - due;
    assert!(published.is_some(), "not published after 7 s");
    assert!(
        late < time::Duration::seconds(6),
        "published {late} after it was due"
    );
    let post = ana
        .browser
        .get(&format!("/posts/{}", published.expect("post id")))
        .await
        .ok();
    assert_eq!(post["post"]["text"], "on time");

    stop.cancel();
    while workers.join_next().await.is_some() {}
}

#[sqlx::test(migrations = "./migrations")]
async fn an_edit_after_30_minutes_is_refused(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let post = ana.browser.post_text("typo").await;
    let id = post["id"].as_str().expect("id").to_owned();
    ana.browser
        .patch(&format!("/posts/{id}"), json!({ "text": "fixed" }))
        .await
        .ok();
    sqlx::query("update posts set created_at = now() - interval '31 minutes' where id = $1")
        .bind(id.parse::<Uuid>().expect("uuid"))
        .execute(&db)
        .await
        .expect("age post");
    ana.browser
        .patch(&format!("/posts/{id}"), json!({ "text": "too late" }))
        .await
        .err(
            StatusCode::FORBIDDEN,
            "Posts can only be edited for 30 minutes after posting.",
        );
}
