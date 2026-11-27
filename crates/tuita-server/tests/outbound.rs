//! Outbound federation: every local action queues the right `ap_deliveries` rows, in the
//! transaction of the action. Remote accounts are inserted straight into the database; nothing
//! here touches the network.
#![allow(missing_docs)]

mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;
use tuita_server::db::deletion;
use tuita_server::db::scheduled;
use uuid::Uuid;

use common::{ORIGIN, PASSWORD, app, insert_user};

const PUBLIC: &str = "https://www.w3.org/ns/activitystreams#Public";

/// A queued delivery.
#[derive(Debug)]
struct Row {
    inbox: String,
    activity: Value,
    signer: Option<Uuid>,
}

async fn queue(db: &PgPool) -> Vec<Row> {
    let rows: Vec<(String, Value, Option<Uuid>)> = sqlx::query_as(
        "select inbox_url, activity, signer_id from ap_deliveries order by inbox_url, id",
    )
    .fetch_all(db)
    .await
    .expect("queue");
    rows.into_iter()
        .map(|(inbox, activity, signer)| Row {
            inbox,
            activity,
            signer,
        })
        .collect()
}

fn inboxes(rows: &[Row]) -> Vec<&str> {
    rows.iter().map(|row| row.inbox.as_str()).collect()
}

async fn clear(db: &PgPool) {
    sqlx::query("delete from ap_deliveries")
        .execute(db)
        .await
        .expect("clear");
}

/// A remote account with a fresh profile (so lookups never go to the network).
async fn remote(db: &PgPool, name: &str, domain: &str, shared: Option<&str>) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "insert into users (id, handle, domain, display_name, ap_id, inbox_url, shared_inbox_url,
                            followers_url, fetched_at)
         values ($1, $2, $3, $2, $4, $5, $6, $7, now())",
    )
    .bind(id)
    .bind(name)
    .bind(domain)
    .bind(format!("https://{domain}/users/{name}"))
    .bind(format!("https://{domain}/users/{name}/inbox"))
    .bind(shared)
    .bind(format!("https://{domain}/users/{name}/followers"))
    .execute(db)
    .await
    .expect("insert remote");
    id
}

fn actor_of(domain: &str, name: &str) -> String {
    format!("https://{domain}/users/{name}")
}

async fn follows(db: &PgPool, follower: Uuid, followee: Uuid) {
    sqlx::query(
        "insert into follows (follower_id, followee_id, state) values ($1, $2, 'accepted')",
    )
    .bind(follower)
    .bind(followee)
    .execute(db)
    .await
    .expect("follow");
}

/// A post of a remote author, as ingest stores it.
async fn remote_post(db: &PgPool, author: Uuid, domain: &str, name: &str) -> (Uuid, String) {
    let id = Uuid::now_v7();
    let ap_id = format!("https://{domain}/users/{name}/statuses/{id}");
    sqlx::query(
        "insert into posts (id, author_id, text, ap_id, visibility) values ($1, $2, 'hello', $3, 'public')",
    )
    .bind(id)
    .bind(author)
    .bind(&ap_id)
    .execute(db)
    .await
    .expect("remote post");
    (id, ap_id)
}

fn id_of(post: &Value) -> String {
    post["id"].as_str().expect("post id").to_owned()
}

fn kind(row: &Row) -> &str {
    row.activity["type"].as_str().expect("type")
}

/// Ana with a remote follower on `a.example` (shared inbox), a second follower on the same
/// shared inbox, and a third on `b.example` without one.
struct Net {
    shared: &'static str,
    dave_inbox: String,
}

async fn followed_by_remote(db: &PgPool, ana: Uuid) -> Net {
    let shared = "https://a.example/inbox";
    let bob = remote(db, "bob", "a.example", Some(shared)).await;
    let carol = remote(db, "carol", "a.example", Some(shared)).await;
    let dave = remote(db, "dave", "b.example", None).await;
    for follower in [bob, carol, dave] {
        follows(db, follower, ana).await;
    }
    Net {
        shared,
        dave_inbox: "https://b.example/users/dave/inbox".to_owned(),
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn a_post_goes_once_to_each_distinct_inbox_of_the_remote_followers(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let net = followed_by_remote(&db, ana.id).await;
    let local = insert_user(&db, "locale").await;
    follows(&db, local, ana.id).await;

    let post = ana.browser.post_text("hello fediverse #rust").await;
    let rows = queue(&db).await;
    assert_eq!(inboxes(&rows), [net.shared, net.dave_inbox.as_str()]);
    for row in &rows {
        assert_eq!(row.signer, Some(ana.id));
        assert_eq!(kind(row), "Create");
        assert_eq!(row.activity["actor"], format!("{ORIGIN}/users/ana"));
        assert_eq!(row.activity["to"], json!([PUBLIC]));
        assert_eq!(
            row.activity["cc"],
            json!([format!("{ORIGIN}/users/ana/followers")])
        );
        assert!(row.activity["@context"].is_array());
        assert_eq!(
            row.activity["object"]["id"],
            format!("{ORIGIN}/users/ana/statuses/{}", id_of(&post))
        );
        assert_eq!(
            row.activity["id"],
            format!("{ORIGIN}/users/ana/statuses/{}/activity", id_of(&post))
        );
    }
    assert_eq!(rows[0].activity, rows[1].activity);
}

#[sqlx::test(migrations = "./migrations")]
async fn an_action_with_only_local_audience_writes_no_rows(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut ben = app.person(&db, "ben").await;
    follows(&db, ben.id, ana.id).await;
    let post = ana.browser.post_text("local only @ben").await;
    let id = id_of(&post);
    ben.browser
        .post_empty(&format!("/posts/{id}/like"))
        .await
        .ok();
    ben.browser
        .post_empty(&format!("/posts/{id}/repost"))
        .await
        .ok();
    ana.browser
        .patch(&format!("/posts/{id}"), json!({ "text": "edited @ben" }))
        .await
        .ok();
    ben.browser
        .post("/conversations/ana", json!({ "text": "hi" }))
        .await
        .ok();
    ana.browser
        .patch("/me", json!({ "bio": "new bio" }))
        .await
        .ok();
    ben.browser.post_empty("/users/ana/follow").await.ok();
    ben.browser.delete("/users/ana/follow").await.ok();
    ana.browser.delete(&format!("/posts/{id}")).await.ok();
    assert!(queue(&db).await.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn mentions_replies_and_quotes_reach_the_remote_account_even_without_a_follow(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    remote(&db, "eve", "c.example", Some("https://c.example/inbox")).await;
    let frank = remote(&db, "frank", "d.example", None).await;
    let gina = remote(&db, "gina", "e.example", None).await;
    let (frank_post, frank_ap) = remote_post(&db, frank, "d.example", "frank").await;
    let (gina_post, _) = remote_post(&db, gina, "e.example", "gina").await;

    ana.browser.post_text("hello @eve@c.example").await;
    let rows = queue(&db).await;
    assert_eq!(inboxes(&rows), ["https://c.example/inbox"]);
    assert_eq!(rows[0].activity["to"], json!([PUBLIC]));
    assert!(
        rows[0].activity["cc"]
            .as_array()
            .expect("cc")
            .contains(&json!(actor_of("c.example", "eve")))
    );
    assert_eq!(
        rows[0].activity["object"]["tag"][0]["href"],
        actor_of("c.example", "eve")
    );
    clear(&db).await;

    ana.browser
        .post(
            "/posts",
            json!({ "text": "agreed", "replyTo": frank_post.to_string() }),
        )
        .await
        .ok();
    let rows = queue(&db).await;
    assert_eq!(inboxes(&rows), ["https://d.example/users/frank/inbox"]);
    assert_eq!(rows[0].activity["object"]["inReplyTo"], frank_ap);
    clear(&db).await;

    ana.browser
        .post(
            "/posts",
            json!({ "text": "look", "quoteOf": gina_post.to_string() }),
        )
        .await
        .ok();
    assert_eq!(
        inboxes(&queue(&db).await),
        ["https://e.example/users/gina/inbox"]
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn an_edit_sends_an_update_and_an_unchanged_edit_sends_nothing(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let net = followed_by_remote(&db, ana.id).await;
    let post = ana.browser.post_text("first").await;
    let id = id_of(&post);
    clear(&db).await;

    ana.browser
        .patch(&format!("/posts/{id}"), json!({ "text": "first" }))
        .await
        .ok();
    assert!(queue(&db).await.is_empty());

    ana.browser
        .patch(&format!("/posts/{id}"), json!({ "text": "second" }))
        .await
        .ok();
    let rows = queue(&db).await;
    assert_eq!(inboxes(&rows), [net.shared, net.dave_inbox.as_str()]);
    for row in &rows {
        assert_eq!(kind(row), "Update");
        assert!(
            row.activity["id"]
                .as_str()
                .expect("id")
                .contains("#updates/")
        );
        assert_eq!(row.activity["object"]["type"], "Note");
        assert!(
            row.activity["object"]["content"]
                .as_str()
                .expect("content")
                .contains("second")
        );
        assert!(row.activity["object"]["updated"].is_string());
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn a_delete_sends_one_tombstone_and_a_repeat_sends_none(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let net = followed_by_remote(&db, ana.id).await;
    remote(&db, "eve", "c.example", None).await;
    let post = ana.browser.post_text("bye @eve@c.example").await;
    let id = id_of(&post);
    clear(&db).await;

    ana.browser.delete(&format!("/posts/{id}")).await;
    let rows = queue(&db).await;
    assert_eq!(
        inboxes(&rows),
        [
            net.shared,
            net.dave_inbox.as_str(),
            "https://c.example/users/eve/inbox"
        ]
    );
    for row in &rows {
        assert_eq!(kind(row), "Delete");
        assert_eq!(row.activity["object"]["type"], "Tombstone");
        assert_eq!(
            row.activity["object"]["id"],
            format!("{ORIGIN}/users/ana/statuses/{id}")
        );
        assert_eq!(row.signer, Some(ana.id));
    }
    clear(&db).await;
    ana.browser
        .delete(&format!("/posts/{id}"))
        .await
        .err(StatusCode::NOT_FOUND, "Post not found.");
    assert!(queue(&db).await.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn staff_deletions_and_purges_are_federated_as_the_author(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut boss = app.person(&db, "boss").await;
    sqlx::query("update users set role = 'admin' where id = $1")
        .bind(boss.id)
        .execute(&db)
        .await
        .expect("admin");
    let net = followed_by_remote(&db, ana.id).await;
    let first = ana.browser.post_text("one").await;
    let second = ana.browser.post_text("two").await;
    let third = ana.browser.post_text("three").await;
    clear(&db).await;

    boss.browser
        .delete(&format!("/posts/{}", id_of(&first)))
        .await
        .ok();
    let rows = queue(&db).await;
    assert_eq!(inboxes(&rows), [net.shared, net.dave_inbox.as_str()]);
    assert!(rows.iter().all(|row| kind(row) == "Delete"
        && row.signer == Some(ana.id)
        && row.activity["actor"] == format!("{ORIGIN}/users/ana")));
    clear(&db).await;

    boss.browser.delete("/admin/users/ana/posts").await.ok();
    let rows = queue(&db).await;
    assert_eq!(rows.len(), 4, "two live posts, two inboxes each");
    let mut objects: Vec<String> = rows
        .iter()
        .map(|row| {
            row.activity["object"]["id"]
                .as_str()
                .expect("id")
                .to_owned()
        })
        .collect();
    objects.sort();
    objects.dedup();
    let mut wanted = vec![
        format!("{ORIGIN}/users/ana/statuses/{}", id_of(&second)),
        format!("{ORIGIN}/users/ana/statuses/{}", id_of(&third)),
    ];
    wanted.sort();
    assert_eq!(objects, wanted);
    clear(&db).await;

    boss.browser.delete("/admin/users/ana/posts").await.ok();
    assert!(queue(&db).await.is_empty(), "nothing left to delete");
}

#[sqlx::test(migrations = "./migrations")]
async fn resolving_a_report_by_deleting_the_post_federates_the_deletion(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut boss = app.person(&db, "boss").await;
    sqlx::query("update users set role = 'admin' where id = $1")
        .bind(boss.id)
        .execute(&db)
        .await
        .expect("admin");
    let net = followed_by_remote(&db, ana.id).await;
    let post = ana.browser.post_text("spam").await;
    let report = Uuid::now_v7();
    sqlx::query(
        "insert into reports (id, reporter_id, target_id, post_id, reason)
         values ($1, $2, $3, $4, 'spam')",
    )
    .bind(report)
    .bind(boss.id)
    .bind(ana.id)
    .bind(id_of(&post).parse::<Uuid>().expect("uuid"))
    .execute(&db)
    .await
    .expect("report");
    clear(&db).await;

    boss.browser
        .post(
            &format!("/admin/reports/{report}/resolve"),
            json!({ "action": "delete_post" }),
        )
        .await
        .ok();
    let rows = queue(&db).await;
    assert_eq!(inboxes(&rows), [net.shared, net.dave_inbox.as_str()]);
    assert!(rows.iter().all(|row| kind(row) == "Delete"));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_like_of_a_remote_post_is_sent_to_its_author_and_undone_by_the_same_id(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let bob = remote(&db, "bob", "a.example", Some("https://a.example/inbox")).await;
    let (post, ap_id) = remote_post(&db, bob, "a.example", "bob").await;
    let local = ana.browser.post_text("mine").await;

    ana.browser
        .post_empty(&format!("/posts/{post}/like"))
        .await
        .ok();
    let rows = queue(&db).await;
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].inbox, "https://a.example/users/bob/inbox",
        "a like goes to the author's own inbox"
    );
    assert_eq!(kind(&rows[0]), "Like");
    assert_eq!(rows[0].activity["object"], ap_id);
    assert_eq!(rows[0].activity["actor"], format!("{ORIGIN}/users/ana"));
    assert_eq!(rows[0].signer, Some(ana.id));
    let like_id = rows[0].activity["id"].as_str().expect("id").to_owned();

    ana.browser
        .post_empty(&format!("/posts/{post}/like"))
        .await
        .ok();
    assert_eq!(queue(&db).await.len(), 1, "a repeated like adds nothing");

    ana.browser
        .post_empty(&format!("/posts/{}/like", id_of(&local)))
        .await
        .ok();
    assert_eq!(
        queue(&db).await.len(),
        1,
        "a like of a local post stays home"
    );

    ana.browser
        .delete(&format!("/posts/{post}/like"))
        .await
        .ok();
    let rows = queue(&db).await;
    assert_eq!(rows.len(), 2);
    let undo = rows
        .iter()
        .find(|row| kind(row) == "Undo")
        .expect("an Undo row");
    assert_eq!(undo.activity["object"]["type"], "Like");
    assert_eq!(undo.activity["object"]["id"], like_id);
    assert_eq!(undo.activity["object"]["object"], ap_id);

    ana.browser
        .delete(&format!("/posts/{post}/like"))
        .await
        .ok();
    assert_eq!(queue(&db).await.len(), 2, "a repeated unlike adds nothing");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_repost_goes_to_the_followers_and_the_author_and_is_undone_by_the_same_id(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let net = followed_by_remote(&db, ana.id).await;
    let erin = remote(&db, "erin", "a.example", Some(net.shared)).await;
    let (post, ap_id) = remote_post(&db, erin, "a.example", "erin").await;
    let frank = remote(&db, "frank", "f.example", None).await;
    let (other, _) = remote_post(&db, frank, "f.example", "frank").await;

    ana.browser
        .post_empty(&format!("/posts/{post}/repost"))
        .await
        .ok();
    let rows = queue(&db).await;
    assert_eq!(
        inboxes(&rows),
        [net.shared, net.dave_inbox.as_str()],
        "the author shares an inbox with two followers"
    );
    for row in &rows {
        assert_eq!(kind(row), "Announce");
        assert_eq!(row.activity["object"], ap_id);
        assert_eq!(row.activity["to"], json!([PUBLIC]));
        assert_eq!(row.signer, Some(ana.id));
    }
    let announce_id = rows[0].activity["id"].as_str().expect("id").to_owned();
    clear(&db).await;

    ana.browser
        .delete(&format!("/posts/{post}/repost"))
        .await
        .ok();
    let rows = queue(&db).await;
    assert_eq!(inboxes(&rows), [net.shared, net.dave_inbox.as_str()]);
    for row in &rows {
        assert_eq!(kind(row), "Undo");
        assert_eq!(row.activity["object"]["type"], "Announce");
        assert_eq!(row.activity["object"]["id"], announce_id);
    }
    clear(&db).await;

    ana.browser
        .delete(&format!("/posts/{post}/repost"))
        .await
        .ok();
    assert!(
        queue(&db).await.is_empty(),
        "a repeated unrepost adds nothing"
    );

    ana.browser
        .post_empty(&format!("/posts/{other}/repost"))
        .await
        .ok();
    let rows = queue(&db).await;
    assert_eq!(
        inboxes(&rows),
        [
            net.shared,
            net.dave_inbox.as_str(),
            "https://f.example/users/frank/inbox"
        ]
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn following_a_remote_account_sends_follow_and_unfollow_sends_undo_with_the_same_id(
    db: PgPool,
) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let bob = remote(&db, "bob", "a.example", Some("https://a.example/inbox")).await;

    ana.browser
        .post_empty("/users/bob@a.example/follow")
        .await
        .ok();
    let rows = queue(&db).await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].inbox, "https://a.example/users/bob/inbox");
    assert_eq!(kind(&rows[0]), "Follow");
    assert_eq!(rows[0].activity["object"], actor_of("a.example", "bob"));
    assert_eq!(rows[0].signer, Some(ana.id));
    let follow_id = rows[0].activity["id"].as_str().expect("id").to_owned();
    let stored: (String, Option<String>) = sqlx::query_as(
        "select state, ap_id from follows where follower_id = $1 and followee_id = $2",
    )
    .bind(ana.id)
    .bind(bob)
    .fetch_one(&db)
    .await
    .expect("follow row");
    assert_eq!(stored, ("pending".to_owned(), Some(follow_id.clone())));

    ana.browser
        .post_empty("/users/bob@a.example/follow")
        .await
        .ok();
    assert_eq!(queue(&db).await.len(), 1, "a repeated follow adds nothing");

    ana.browser.delete("/users/bob@a.example/follow").await.ok();
    let rows = queue(&db).await;
    assert_eq!(rows.len(), 2, "cancelling a pending request is undone too");
    let undo = rows
        .iter()
        .find(|row| kind(row) == "Undo")
        .expect("an Undo row");
    assert_eq!(undo.activity["object"]["type"], "Follow");
    assert_eq!(undo.activity["object"]["id"], follow_id);
    assert_eq!(
        undo.activity["object"]["object"],
        actor_of("a.example", "bob")
    );

    ana.browser.delete("/users/bob@a.example/follow").await.ok();
    assert_eq!(
        queue(&db).await.len(),
        2,
        "a repeated unfollow adds nothing"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_vote_on_a_remote_poll_sends_one_note_per_choice(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let bob = remote(&db, "bob", "a.example", Some("https://a.example/inbox")).await;
    let (post, ap_id) = remote_post(&db, bob, "a.example", "bob").await;
    sqlx::query(
        "insert into polls (post_id, options, multiple, ends_at, counts)
         values ($1, '{red,green,blue}', true, now() + interval '1 day', '{0,0,0}')",
    )
    .bind(post)
    .execute(&db)
    .await
    .expect("poll");

    ana.browser
        .post(&format!("/posts/{post}/vote"), json!({ "choices": [0, 2] }))
        .await
        .ok();
    let rows = queue(&db).await;
    assert_eq!(rows.len(), 2);
    let mut names: Vec<&str> = rows
        .iter()
        .map(|row| row.activity["object"]["name"].as_str().expect("name"))
        .collect();
    names.sort_unstable();
    assert_eq!(names, ["blue", "red"]);
    for row in &rows {
        assert_eq!(row.inbox, "https://a.example/users/bob/inbox");
        assert_eq!(kind(row), "Create");
        assert_eq!(row.activity["object"]["type"], "Note");
        assert_eq!(row.activity["object"]["inReplyTo"], ap_id);
        assert_eq!(
            row.activity["object"]["attributedTo"],
            format!("{ORIGIN}/users/ana")
        );
        assert_eq!(row.activity["to"], json!([actor_of("a.example", "bob")]));
    }

    ana.browser
        .post(&format!("/posts/{post}/vote"), json!({ "choices": [1] }))
        .await
        .err(StatusCode::CONFLICT, "You already voted.");
    assert_eq!(queue(&db).await.len(), 2, "a refused vote sends nothing");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_direct_message_to_a_remote_account_is_a_direct_note(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    app.person(&db, "ben").await;
    remote(&db, "bob", "a.example", Some("https://a.example/inbox")).await;

    let sent = ana
        .browser
        .post("/conversations/bob@a.example", json!({ "text": "psst" }))
        .await
        .ok();
    let rows = queue(&db).await;
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.inbox, "https://a.example/users/bob/inbox");
    assert_eq!(kind(row), "Create");
    assert_eq!(row.signer, Some(ana.id));
    let remote_actor = actor_of("a.example", "bob");
    assert_eq!(row.activity["to"], json!([remote_actor]));
    assert!(row.activity.get("cc").is_none_or(|cc| cc == &json!([])));
    let note = &row.activity["object"];
    assert_eq!(note["type"], "Note");
    assert_eq!(note["to"], json!([remote_actor]));
    assert_eq!(note["tag"][0]["type"], "Mention");
    assert_eq!(note["tag"][0]["href"], remote_actor);
    let content = note["content"].as_str().expect("content");
    assert!(content.contains("psst") && content.contains("@<span>bob</span>"));
    let ap_id: Option<String> = sqlx::query_scalar("select ap_id from messages where id = $1")
        .bind(
            sent["id"]
                .as_str()
                .expect("id")
                .parse::<Uuid>()
                .expect("uuid"),
        )
        .fetch_one(&db)
        .await
        .expect("message");
    assert_eq!(ap_id.as_deref(), note["id"].as_str());

    clear(&db).await;
    ana.browser
        .post("/conversations/ben", json!({ "text": "local" }))
        .await
        .ok();
    assert!(queue(&db).await.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn a_profile_change_sends_update_person_to_the_followers_once_per_inbox(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let net = followed_by_remote(&db, ana.id).await;

    ana.browser
        .patch("/me", json!({ "bio": "I write about #birds" }))
        .await
        .ok();
    let rows = queue(&db).await;
    assert_eq!(inboxes(&rows), [net.shared, net.dave_inbox.as_str()]);
    for row in &rows {
        assert_eq!(kind(row), "Update");
        assert_eq!(row.signer, Some(ana.id));
        let actor = &row.activity["object"];
        assert_eq!(actor["type"], "Person");
        assert_eq!(actor["id"], format!("{ORIGIN}/users/ana"));
        assert!(
            actor["summary"]
                .as_str()
                .expect("summary")
                .contains("birds")
        );
        assert!(
            actor["publicKey"]["publicKeyPem"]
                .as_str()
                .expect("pem")
                .starts_with("-----BEGIN PUBLIC KEY-----")
        );
        assert!(actor.get("@context").is_none());
    }
    clear(&db).await;

    ana.browser
        .patch("/me", json!({ "mutedWords": ["spoilers"] }))
        .await
        .ok();
    assert!(
        queue(&db).await.is_empty(),
        "settings the actor does not show send nothing"
    );

    ana.browser
        .patch("/me/bot", json!({ "bot": true }))
        .await
        .ok();
    let rows = queue(&db).await;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].activity["object"]["type"], "Service");
}

#[sqlx::test(migrations = "./migrations")]
async fn deleting_an_account_sends_delete_person_to_every_known_remote_inbox(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.browser();
    ana.signup("ana").await;
    let ana_id: Uuid = sqlx::query_scalar("select id from users where handle = 'ana'")
        .fetch_one(&db)
        .await
        .expect("id");
    sqlx::query("update users set role = 'user' where id = $1")
        .bind(ana_id)
        .execute(&db)
        .await
        .expect("demote the first sign-up");
    let shared = "https://a.example/inbox";
    let bob = remote(&db, "bob", "a.example", Some(shared)).await;
    remote(&db, "carol", "a.example", Some(shared)).await;
    remote(&db, "dave", "b.example", None).await;
    follows(&db, bob, ana_id).await;
    clear(&db).await;

    ana.send(
        axum::http::Method::DELETE,
        "/me",
        Some(json!({ "password": PASSWORD })),
    )
    .await
    .ok();
    let rows = queue(&db).await;
    assert_eq!(
        inboxes(&rows),
        [shared, "https://b.example/users/dave/inbox"],
        "followers and strangers alike, each inbox once"
    );
    let actor = format!("{ORIGIN}/users/ana");
    for row in &rows {
        assert_eq!(kind(row), "Delete");
        assert_eq!(row.activity["actor"], actor);
        assert_eq!(row.activity["object"], actor);
        assert_eq!(row.activity["to"], json!([PUBLIC]));
        assert_eq!(row.signer, Some(ana_id));
    }
    let handle: String = sqlx::query_scalar("select handle from users where id = $1")
        .bind(ana_id)
        .fetch_one(&db)
        .await
        .expect("handle");
    assert!(handle.starts_with("deleted_"));
}

#[sqlx::test(migrations = "./migrations")]
async fn deleting_a_remote_account_row_sends_nothing(db: PgPool) {
    let app = app(db.clone());
    app.person(&db, "ana").await;
    let bob = remote(&db, "bob", "a.example", Some("https://a.example/inbox")).await;
    let row = tuita_server::db::users::by_id(&db, bob)
        .await
        .expect("load")
        .expect("row");
    deletion::delete_account(&db, &row).await.expect("delete");
    assert!(queue(&db).await.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn a_failing_action_leaves_no_rows(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    followed_by_remote(&db, ana.id).await;

    let reply = ana
        .browser
        .post(
            "/posts/thread",
            json!({ "posts": [
                { "text": "one" },
                { "text": "two", "mediaIds": [Uuid::now_v7().to_string()] },
            ] }),
        )
        .await;
    assert_ne!(reply.status, StatusCode::OK, "body: {}", reply.body);
    let posts: i64 = sqlx::query_scalar("select count(*) from posts")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(posts, 0, "the thread rolled back");
    assert!(
        queue(&db).await.is_empty(),
        "and so did the first post's delivery rows"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_scheduled_post_is_delivered_when_it_publishes(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let net = followed_by_remote(&db, ana.id).await;
    let at = (time::OffsetDateTime::now_utc() + time::Duration::minutes(5))
        .format(&time::format_description::well_known::Rfc3339)
        .expect("format");
    ana.browser
        .post("/scheduled", json!({ "text": "later", "publishAt": at }))
        .await
        .ok();
    assert!(
        queue(&db).await.is_empty(),
        "nothing goes out when it is scheduled"
    );

    sqlx::query("update scheduled_posts set publish_at = now() - interval '1 minute'")
        .execute(&db)
        .await
        .expect("make due");
    let published = scheduled::publish_due(&app.state).await.expect("publish");
    assert_eq!(published, 1);
    let rows = queue(&db).await;
    assert_eq!(inboxes(&rows), [net.shared, net.dave_inbox.as_str()]);
    assert!(rows.iter().all(|row| {
        kind(row) == "Create"
            && row.signer == Some(ana.id)
            && row.activity["object"]["content"]
                .as_str()
                .is_some_and(|text| text.contains("later"))
    }));
}

#[sqlx::test(migrations = "./migrations")]
async fn inboxes_of_suspended_instances_and_deleted_accounts_are_skipped(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let net = followed_by_remote(&db, ana.id).await;
    sqlx::query("insert into domain_blocks (domain, severity) values ('b.example', 'suspend')")
        .execute(&db)
        .await
        .expect("block");
    ana.browser.post_text("hello").await;
    assert_eq!(inboxes(&queue(&db).await), [net.shared]);

    sqlx::query("update users set deleted_at = now() where domain = 'a.example'")
        .execute(&db)
        .await
        .expect("gone");
    clear(&db).await;
    ana.browser.post_text("again").await;
    assert!(queue(&db).await.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn a_delete_cancels_the_create_and_update_still_waiting_in_the_queue(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let net = followed_by_remote(&db, ana.id).await;
    let doomed = id_of(&ana.browser.post_text("regrets").await);
    let kept = id_of(&ana.browser.post_text("fine").await);
    ana.browser
        .patch(
            &format!("/posts/{doomed}"),
            json!({"text": "regrets, edited"}),
        )
        .await
        .ok();
    let kinds = |rows: &[Row], id: &str| -> Vec<String> {
        rows.iter()
            .filter(|row| {
                let object = &row.activity["object"];
                object["id"] == format!("{ORIGIN}/users/ana/statuses/{id}")
            })
            .map(|row| kind(row).to_owned())
            .collect()
    };
    let before = queue(&db).await;
    assert_eq!(
        kinds(&before, &doomed).len(),
        4,
        "two inboxes, Create and Update"
    );

    ana.browser.delete(&format!("/posts/{doomed}")).await;
    let after = queue(&db).await;
    assert_eq!(
        kinds(&after, &doomed),
        ["Delete", "Delete"],
        "only the tombstones for {} and {} are left",
        net.shared,
        net.dave_inbox
    );
    assert_eq!(kinds(&after, &kept), ["Create", "Create"]);
}
