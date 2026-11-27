//! Two full servers on two real sockets federate with each other, driven through the HTTP API.
//!
//! Each test makes a database of its own, named with a fresh uuid. `#[sqlx::test]` would name it
//! after the test path alone, so two `cargo test` runs at once (two agents, two worktrees) would
//! drop each other's databases. Instance `a` lives in it and instance `b` in a schema `fed_b`
//! (a second database would cost another forced checkpoint on drop). Each instance has its own
//! `AppState` and the real `router` served by `axum::serve`. Their delivery workers are driven
//! by hand (`run_once` in a polling loop, with a 5 s deadline) so every test is deterministic
//! and never sleeps a fixed time.
#![allow(missing_docs)]

mod common;

use std::net::SocketAddr;
use std::time::Duration;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use tokio::net::TcpListener;
use tokio::time::Instant;
use tuita_server::workers::delivery;
use tuita_server::{AppState, Config, router};

use common::{App, Browser};

/// How long anything may take to converge.
const DEADLINE: Duration = Duration::from_secs(5);

/// The database of one test, dropped with it even when the test panics.
struct Scratch {
    name: String,
    admin: PgConnectOptions,
}

impl Scratch {
    /// Creates the database through `DATABASE_URL`.
    async fn create() -> Self {
        use sqlx::ConnectOptions as _;
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL is set by devenv");
        let admin: PgConnectOptions = url.parse().expect("DATABASE_URL parses");
        let name = format!("tuita_fed_{}", uuid::Uuid::now_v7().simple());
        let mut conn = admin.connect().await.expect("connect DATABASE_URL");
        sqlx::query(sqlx::AssertSqlSafe(format!("create database \"{name}\"")))
            .execute(&mut conn)
            .await
            .expect("create the test database");
        Self { name, admin }
    }

    fn options(&self) -> PgConnectOptions {
        self.admin.clone().database(&self.name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let (name, admin) = (self.name.clone(), self.admin.clone());
        let _ = std::thread::spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            runtime.block_on(async {
                use sqlx::ConnectOptions as _;
                if let Ok(mut conn) = admin.connect().await {
                    let _ = sqlx::query(sqlx::AssertSqlSafe(format!(
                        "drop database if exists \"{name}\" with (force)"
                    )))
                    .execute(&mut conn)
                    .await;
                }
            });
        })
        .join();
    }
}

/// One running instance.
struct Node {
    state: AppState,
    app: App,
    db: PgPool,
    /// `http://127.0.0.1:{port}`.
    base: String,
    /// `127.0.0.1:{port}`.
    domain: String,
}

impl Node {
    async fn start(db: PgPool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let port = listener.local_addr().expect("addr").port();
        let base = format!("http://127.0.0.1:{port}");
        let config = Config::from_lookup(|var| match var {
            "DATABASE_URL" => Some("postgres://unused".to_owned()),
            "PUBLIC_URL" => Some(base.clone()),
            "FEDERATION_ALLOW_HTTP" | "FEDERATION_ALLOW_PRIVATE_NETS" => Some("1".to_owned()),
            _ => None,
        })
        .expect("valid config");
        let media = tuita_server::media::MediaStore::from_config(&config).expect("media store");
        let state = AppState::new(db.clone(), config, media);
        state.set_ready(true);
        let router = router(state.clone(), common::stub_web());
        let served = router.clone();
        tokio::spawn(async move {
            let _ = axum::serve(
                listener,
                served.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await;
        });
        Self {
            app: App {
                state: state.clone(),
                router,
            },
            state,
            db,
            domain: format!("127.0.0.1:{port}"),
            base,
        }
    }

    /// A browser signed up as `handle`.
    async fn user(&self, handle: &str) -> Browser {
        let mut browser = self.app.browser();
        browser.origin = Some(self.base.clone());
        browser.signup(handle).await;
        browser
    }

    /// Makes `handle` an admin and returns a browser signed in as it.
    async fn admin(&self, handle: &str) -> Browser {
        let browser = self.user(handle).await;
        sqlx::query("update users set role = 'admin' where handle = $1")
            .bind(handle)
            .execute(&self.db)
            .await
            .expect("promote");
        browser
    }

    async fn queued(&self) -> i64 {
        sqlx::query_scalar("select count(*) from ap_deliveries")
            .fetch_one(&self.db)
            .await
            .expect("count")
    }

    /// `handle@domain` of an account of this instance, as the other instance names it.
    fn remote(&self, handle: &str) -> String {
        format!("{handle}@{}", self.domain)
    }
}

/// Two instances: `a` (the test's database) and `b` (its schema `fed_b`).
struct Fed {
    a: Node,
    b: Node,
    _scratch: Scratch,
}

impl Fed {
    async fn new() -> Self {
        let scratch = Scratch::create().await;
        let db_a = PgPoolOptions::new()
            .max_connections(4)
            .connect_with(scratch.options())
            .await
            .expect("connect the test database");
        tuita_server::migrate(&db_a).await.expect("migrate a");
        sqlx::query("create schema fed_b")
            .execute(&db_a)
            .await
            .expect("create the second schema");
        let db_b = PgPoolOptions::new()
            .max_connections(4)
            .after_connect(|conn, _| {
                Box::pin(async move {
                    sqlx::query("set search_path to fed_b, public")
                        .execute(conn)
                        .await?;
                    Ok(())
                })
            })
            .connect_with(scratch.options())
            .await
            .expect("connect the second schema");
        tuita_server::migrate(&db_b).await.expect("migrate b");
        Self {
            a: Node::start(db_a).await,
            b: Node::start(db_b).await,
            _scratch: scratch,
        }
    }

    /// Runs both delivery workers until both queues are empty.
    async fn settle(&self) {
        let deadline = Instant::now() + DEADLINE;
        loop {
            let handled =
                delivery::run_once(&self.a.state).await + delivery::run_once(&self.b.state).await;
            if self.a.queued().await + self.b.queued().await == 0 {
                return;
            }
            if Instant::now() > deadline {
                let mut rows = Vec::new();
                for node in [&self.a, &self.b] {
                    let found: Vec<(String, i32, Option<String>)> =
                        sqlx::query_as("select inbox_url, attempts, last_error from ap_deliveries")
                            .fetch_all(&node.db)
                            .await
                            .expect("rows");
                    rows.extend(found);
                }
                panic!("deliveries did not drain within {DEADLINE:?}: {rows:?}");
            }
            if handled == 0 {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }
    }

    /// Settles, then polls `check` (settling before each try) until it returns `Some`.
    async fn eventually<T>(&self, what: &str, mut check: impl AsyncFnMut() -> Option<T>) -> T {
        let deadline = Instant::now() + DEADLINE;
        loop {
            self.settle().await;
            if let Some(found) = check().await {
                return found;
            }
            assert!(Instant::now() < deadline, "not within {DEADLINE:?}: {what}");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
}

/// The first post of `page` whose text is `text`.
fn find<'a>(page: &'a Value, text: &str) -> Option<&'a Value> {
    page["items"]
        .as_array()?
        .iter()
        .find(|item| item["text"] == text)
}

/// Follows `target` and waits until the follow is accepted.
async fn follow_and_accept(fed: &Fed, who: &mut Browser, target: &str) {
    let reply = who.post_empty(&format!("/users/{target}/follow")).await;
    let user = reply.ok();
    assert_eq!(user["relation"]["followPending"], true, "{user}");
    assert_eq!(user["relation"]["following"], false, "{user}");
    fed.settle().await;
    let user = who.get(&format!("/users/{target}")).await.ok();
    assert_eq!(user["relation"]["following"], true, "{user}");
    assert_eq!(user["relation"]["followPending"], false, "{user}");
}

#[tokio::test]
async fn the_whole_story() {
    let fed = Fed::new().await;
    let mut ada = fed.a.user("ada").await;
    let mut bob = fed.b.user("bob").await;
    let bob_at = fed.b.remote("bob");
    let ada_at = fed.a.remote("ada");

    let looked_up = ada.get(&format!("/users/{bob_at}")).await.ok();
    assert_eq!(looked_up["handle"], "bob");
    assert_eq!(looked_up["domain"], fed.b.domain);
    assert_eq!(looked_up["displayName"], "Name of bob");

    follow_and_accept(&fed, &mut ada, &bob_at).await;
    let bob_seen = bob.get(&format!("/users/{ada_at}")).await.ok();
    assert_eq!(bob_seen["relation"]["followsYou"], true, "{bob_seen}");
    assert_eq!(bob_seen["counts"]["following"], 1);
    assert_eq!(
        ada.get(&format!("/users/{bob_at}")).await.ok()["counts"]["followers"],
        1
    );
    let notes = bob.get("/notifications").await.ok();
    assert_eq!(notes["items"][0]["kind"], "follow", "{notes}");
    assert_eq!(notes["items"][0]["actors"][0]["handle"], "ada");
    assert_eq!(notes["items"][0]["actors"][0]["domain"], fed.a.domain);

    let posted = bob.post_text("hello from the other side").await;
    let timeline = fed
        .eventually("bob's post reaches ada's following timeline", async || {
            let page = ada.get("/timelines/following").await.ok();
            find(&page, "hello from the other side").cloned()
        })
        .await;
    assert_eq!(timeline["author"]["handle"], "bob");
    assert_eq!(timeline["author"]["domain"], fed.b.domain);
    assert_ne!(timeline["id"], posted["id"], "ada's copy has its own id");
    let on_a = timeline["id"].as_str().expect("id").to_owned();
    let on_b = posted["id"].as_str().expect("id").to_owned();

    let liked = ada.post_empty(&format!("/posts/{on_a}/like")).await.ok();
    assert_eq!(liked["viewer"]["liked"], true, "{liked}");
    let counted = fed
        .eventually("the like counts on bob's post", async || {
            let post = bob.get(&format!("/posts/{on_b}")).await.ok();
            (post["post"]["counts"]["likes"] == 1).then_some(post)
        })
        .await;
    assert_eq!(counted["post"]["counts"]["likes"], 1);
    let notes = bob.get("/notifications").await.ok();
    let like = notes["items"]
        .as_array()
        .expect("items")
        .iter()
        .find(|n| n["kind"] == "like")
        .unwrap_or_else(|| panic!("no like notification: {notes}"));
    assert_eq!(like["actors"][0]["handle"], "ada");
    assert_eq!(like["post"]["id"], on_b);

    bob.delete(&format!("/posts/{on_b}")).await.ok();
    fed.eventually("bob's post leaves ada's timeline", async || {
        let page = ada.get("/timelines/following").await.ok();
        find(&page, "hello from the other side")
            .is_none()
            .then_some(())
    })
    .await;
    ada.get(&format!("/posts/{on_a}"))
        .await
        .err(StatusCode::NOT_FOUND, "Post not found.");

    let sent = ada
        .post(
            &format!("/conversations/{bob_at}"),
            json!({"text": "psst, a secret"}),
        )
        .await
        .ok();
    assert_eq!(sent["mine"], true);
    let inbox = fed
        .eventually("the direct message reaches bob", async || {
            let page = bob.get("/conversations").await.ok();
            (page["items"].as_array().is_some_and(|i| !i.is_empty())).then_some(page)
        })
        .await;
    assert_eq!(inbox["items"][0]["user"]["handle"], "ada");
    assert_eq!(inbox["items"][0]["user"]["domain"], fed.a.domain);
    assert_eq!(inbox["items"][0]["last"]["text"], "psst, a secret");
    assert_eq!(inbox["items"][0]["last"]["mine"], false);
    assert_eq!(inbox["items"][0]["unread"], 1);

    bob.post(
        &format!("/conversations/{ada_at}"),
        json!({"text": "psst back"}),
    )
    .await
    .ok();
    let thread = fed
        .eventually("bob's reply reaches ada", async || {
            let view = ada.get(&format!("/conversations/{bob_at}")).await.ok();
            (view["messages"].as_array().map(Vec::len) == Some(2)).then_some(view)
        })
        .await;
    let all: Vec<&str> = thread["messages"]
        .as_array()
        .expect("messages")
        .iter()
        .filter_map(|m| m["text"].as_str())
        .collect();
    assert!(
        all.contains(&"psst, a secret") && all.contains(&"psst back"),
        "{all:?}"
    );
}

/// Whether `page` has an item with this text.
fn has(page: &Value, text: &str) -> bool {
    find(page, text).is_some()
}

/// The id of the copy of a post by `author` that `viewer` has stored, found on the following
/// timeline once it arrived.
async fn arrived(fed: &Fed, viewer: &mut Browser, text: &str) -> String {
    let post = fed
        .eventually(
            &format!("{text:?} reaches the following timeline"),
            async || {
                let page = viewer.get("/timelines/following").await.ok();
                find(&page, text).cloned()
            },
        )
        .await;
    post["id"].as_str().expect("id").to_owned()
}

/// Both sides of one follow: `ada` on `a` follows `bob` on `b`.
async fn pair() -> (Fed, Browser, Browser) {
    let fed = Fed::new().await;
    let mut ada = fed.a.user("ada").await;
    let bob = fed.b.user("bob").await;
    let bob_at = fed.b.remote("bob");
    follow_and_accept(&fed, &mut ada, &bob_at).await;
    (fed, ada, bob)
}

#[tokio::test]
async fn unfollowing_undoes_the_follow_on_both_sides() {
    let (fed, mut ada, mut bob) = Box::pin(pair()).await;
    let (bob_at, ada_at) = (fed.b.remote("bob"), fed.a.remote("ada"));
    assert_eq!(bob.get("/users/bob").await.ok()["counts"]["followers"], 1);
    assert_eq!(ada.get("/users/ada").await.ok()["counts"]["following"], 1);

    let user = ada.delete(&format!("/users/{bob_at}/follow")).await.ok();
    assert_eq!(user["relation"]["following"], false, "{user}");
    fed.settle().await;

    assert_eq!(bob.get("/users/bob").await.ok()["counts"]["followers"], 0);
    assert_eq!(
        bob.get(&format!("/users/{ada_at}")).await.ok()["counts"]["following"],
        0
    );
    assert_eq!(ada.get("/users/ada").await.ok()["counts"]["following"], 0);
    let seen = ada.get(&format!("/users/{bob_at}")).await.ok();
    assert_eq!(seen["counts"]["followers"], 0, "{seen}");
    assert_eq!(seen["relation"]["following"], false);

    bob.post_text("nobody follows me now").await;
    fed.settle().await;
    let page = ada.get("/timelines/following").await.ok();
    assert!(!has(&page, "nobody follows me now"), "{page}");
    let queued: i64 = sqlx::query_scalar("select count(*) from follows")
        .fetch_one(&fed.b.db)
        .await
        .expect("count");
    assert_eq!(queued, 0, "no follow row is left on bob's side");
}

#[tokio::test]
async fn replies_cross_instances_in_both_directions() {
    let (fed, mut ada, mut bob) = Box::pin(pair()).await;
    let root_b = bob.post_text("the root").await["id"]
        .as_str()
        .expect("id")
        .to_owned();
    let root_a = arrived(&fed, &mut ada, "the root").await;

    let reply = ada
        .post(
            "/posts",
            json!({"text": "a reply from ada", "replyTo": root_a}),
        )
        .await
        .ok();
    let thread = fed
        .eventually("ada's reply shows in bob's thread", async || {
            let thread = bob.get(&format!("/posts/{root_b}")).await.ok();
            (thread["replies"].as_array().map(Vec::len) == Some(1)).then_some(thread)
        })
        .await;
    let seen = &thread["replies"][0];
    assert_eq!(seen["text"], "a reply from ada");
    assert_eq!(seen["author"]["handle"], "ada");
    assert_eq!(seen["author"]["domain"], fed.a.domain);
    assert_eq!(thread["post"]["counts"]["replies"], 1, "{thread}");
    let notes = bob.get("/notifications").await.ok();
    let note = notes["items"]
        .as_array()
        .expect("items")
        .iter()
        .find(|n| n["kind"] == "reply")
        .unwrap_or_else(|| panic!("no reply notification: {notes}"));
    assert_eq!(note["actors"][0]["handle"], "ada");

    let answer = bob
        .post(
            "/posts",
            json!({"text": "bob answers", "replyTo": seen["id"]}),
        )
        .await
        .ok();
    assert_eq!(answer["replyTo"]["id"], seen["id"], "{answer}");
    let thread = fed
        .eventually("bob's answer shows in ada's thread", async || {
            let id = reply["id"].as_str().expect("id");
            let thread = ada.get(&format!("/posts/{id}")).await.ok();
            (thread["replies"].as_array().map(Vec::len) == Some(1)).then_some(thread)
        })
        .await;
    assert_eq!(thread["replies"][0]["text"], "bob answers");
    assert_eq!(thread["replies"][0]["author"]["domain"], fed.b.domain);
    assert_eq!(thread["ancestors"][0]["text"], "the root", "{thread}");
    let notes = ada.get("/notifications").await.ok();
    assert!(
        notes["items"]
            .as_array()
            .expect("items")
            .iter()
            .any(|n| n["kind"] == "reply" && n["actors"][0]["handle"] == "bob"),
        "{notes}"
    );
}

#[tokio::test]
async fn reposts_count_and_undo() {
    let (fed, mut ada, mut bob) = Box::pin(pair()).await;
    let on_b = bob.post_text("worth sharing").await["id"]
        .as_str()
        .expect("id")
        .to_owned();
    let on_a = arrived(&fed, &mut ada, "worth sharing").await;

    let shared = ada.post_empty(&format!("/posts/{on_a}/repost")).await.ok();
    assert_eq!(shared["viewer"]["reposted"], true, "{shared}");
    let post = fed
        .eventually("the repost counts on bob's post", async || {
            let post = bob.get(&format!("/posts/{on_b}")).await.ok();
            (post["post"]["counts"]["reposts"] == 1).then_some(post)
        })
        .await;
    assert_eq!(post["post"]["counts"]["reposts"], 1);
    let notes = bob.get("/notifications").await.ok();
    assert!(
        notes["items"]
            .as_array()
            .expect("items")
            .iter()
            .any(|n| n["kind"] == "repost" && n["actors"][0]["handle"] == "ada"),
        "{notes}"
    );

    ada.delete(&format!("/posts/{on_a}/repost")).await.ok();
    fed.eventually("the undone repost leaves the count", async || {
        let post = bob.get(&format!("/posts/{on_b}")).await.ok();
        (post["post"]["counts"]["reposts"] == 0).then_some(())
    })
    .await;
}

#[tokio::test]
async fn unliking_takes_the_like_back() {
    let (fed, mut ada, mut bob) = Box::pin(pair()).await;
    let on_b = bob.post_text("like and unlike").await["id"]
        .as_str()
        .expect("id")
        .to_owned();
    let on_a = arrived(&fed, &mut ada, "like and unlike").await;
    ada.post_empty(&format!("/posts/{on_a}/like")).await.ok();
    fed.eventually("liked", async || {
        let post = bob.get(&format!("/posts/{on_b}")).await.ok();
        (post["post"]["counts"]["likes"] == 1).then_some(())
    })
    .await;
    ada.delete(&format!("/posts/{on_a}/like")).await.ok();
    fed.eventually("unliked", async || {
        let post = bob.get(&format!("/posts/{on_b}")).await.ok();
        (post["post"]["counts"]["likes"] == 0).then_some(())
    })
    .await;
}

#[tokio::test]
async fn a_vote_on_a_remote_poll_counts_on_its_instance() {
    let (fed, mut ada, mut bob) = Box::pin(pair()).await;
    let made = bob
        .post(
            "/posts",
            json!({
                "text": "tea or coffee?",
                "poll": {"options": ["tea", "coffee"], "multiple": false, "durationHours": 24},
            }),
        )
        .await
        .ok();
    let on_b = made["id"].as_str().expect("id").to_owned();
    let on_a = arrived(&fed, &mut ada, "tea or coffee?").await;
    let copy = ada.get(&format!("/posts/{on_a}")).await.ok();
    assert_eq!(
        copy["post"]["poll"]["options"][1]["text"], "coffee",
        "{copy}"
    );

    let voted = ada
        .post(&format!("/posts/{on_a}/vote"), json!({"choices": [1]}))
        .await
        .ok();
    assert_eq!(voted["poll"]["voted"], json!([1]), "{voted}");
    let post = fed
        .eventually("the vote counts on bob's poll", async || {
            let post = bob.get(&format!("/posts/{on_b}")).await.ok();
            (post["post"]["poll"]["total"] == 1).then_some(post)
        })
        .await;
    let poll = &post["post"]["poll"];
    assert_eq!(poll["options"][0]["votes"], 0, "{poll}");
    assert_eq!(poll["options"][1]["votes"], 1, "{poll}");
    assert_eq!(poll["voters"], 1);
}

#[tokio::test]
async fn a_profile_update_reaches_the_stored_copy() {
    let (fed, mut ada, mut bob) = Box::pin(pair()).await;
    let bob_at = fed.b.remote("bob");
    bob.patch(
        "/me",
        json!({"displayName": "Bobby the Builder", "bio": "now with a bio"}),
    )
    .await
    .ok();
    let seen = fed
        .eventually("ada's copy of bob has the new name", async || {
            let seen = ada.get(&format!("/users/{bob_at}")).await.ok();
            (seen["displayName"] == "Bobby the Builder").then_some(seen)
        })
        .await;
    assert_eq!(seen["bio"], "now with a bio", "{seen}");
    assert_eq!(seen["handle"], "bob");
}

#[tokio::test]
async fn deleting_an_account_hides_its_posts_remotely() {
    let (fed, mut ada, mut bob) = Box::pin(pair()).await;
    fed.b.admin("keeper").await;
    let bob_at = fed.b.remote("bob");
    bob.post_text("before I leave").await;
    let on_a = arrived(&fed, &mut ada, "before I leave").await;

    bob.send(
        axum::http::Method::DELETE,
        "/me",
        Some(json!({"password": common::PASSWORD})),
    )
    .await
    .ok();
    fed.eventually("bob's posts are gone from ada's timeline", async || {
        let page = ada.get("/timelines/following").await.ok();
        (!has(&page, "before I leave")).then_some(())
    })
    .await;
    ada.get(&format!("/posts/{on_a}"))
        .await
        .err(StatusCode::NOT_FOUND, "Post not found.");
    ada.get(&format!("/users/{bob_at}"))
        .await
        .err(StatusCode::NOT_FOUND, "User not found.");
}

#[tokio::test]
async fn a_block_severs_the_remote_follower() {
    let (fed, mut ada, mut bob) = Box::pin(pair()).await;
    let (bob_at, ada_at) = (fed.b.remote("bob"), fed.a.remote("ada"));
    let blocked = bob.post_empty(&format!("/users/{ada_at}/block")).await.ok();
    assert_eq!(blocked["relation"]["blocked"], true, "{blocked}");
    fed.settle().await;
    assert_eq!(bob.get("/users/bob").await.ok()["counts"]["followers"], 0);
    let seen = fed
        .eventually("ada no longer follows bob", async || {
            let seen = ada.get(&format!("/users/{bob_at}")).await.ok();
            (seen["relation"]["following"] == false).then_some(seen)
        })
        .await;
    assert_eq!(seen["relation"]["followPending"], false, "{seen}");
    assert_eq!(ada.get("/users/ada").await.ok()["counts"]["following"], 0);

    bob.post_text("said after the block").await;
    fed.settle().await;
    let page = ada.get("/timelines/following").await.ok();
    assert!(!has(&page, "said after the block"), "{page}");

    ada.post_empty(&format!("/users/{bob_at}/follow")).await;
    fed.settle().await;
    let seen = ada.get(&format!("/users/{bob_at}")).await.ok();
    assert_eq!(seen["relation"]["following"], false, "{seen}");
    assert_eq!(seen["relation"]["followPending"], false, "{seen}");
    assert_eq!(bob.get("/users/bob").await.ok()["counts"]["followers"], 0);
}

#[tokio::test]
async fn a_suspended_domain_is_dropped_both_ways() {
    let (fed, mut ada, mut bob) = Box::pin(pair()).await;
    let bob_at = fed.b.remote("bob");
    let mut root = fed.a.admin("boss").await;
    let _cy = fed.b.user("cyd").await;
    root.post(
        "/admin/federation/blocks",
        json!({"domain": fed.b.domain, "severity": "suspend", "reason": "test"}),
    )
    .await
    .ok();

    ada.get(&format!("/users/{bob_at}"))
        .await
        .err(StatusCode::NOT_FOUND, "User not found.");
    ada.get(&format!("/users/{}", fed.b.remote("cyd")))
        .await
        .err(StatusCode::NOT_FOUND, "User not found.");
    let known: i64 = sqlx::query_scalar("select count(*) from users where handle = 'cyd'")
        .fetch_one(&fed.a.db)
        .await
        .expect("count");
    assert_eq!(known, 0, "cyd was never fetched");

    bob.post_text("sent into the void").await;
    fed.settle().await;
    let stored: i64 =
        sqlx::query_scalar("select count(*) from posts where text = 'sent into the void'")
            .fetch_one(&fed.a.db)
            .await
            .expect("count");
    assert_eq!(stored, 0, "the suspended instance's post was dropped");

    let ada_post = ada.post_text("never sent").await;
    fed.settle().await;
    let theirs: i64 = sqlx::query_scalar("select count(*) from posts where text = 'never sent'")
        .fetch_one(&fed.b.db)
        .await
        .expect("count");
    assert_eq!(
        theirs, 0,
        "nothing is sent to a suspended instance: {ada_post}"
    );

    root.delete(&format!("/admin/federation/blocks/{}", fed.b.domain))
        .await
        .ok();
    let page = ada.get("/timelines/following").await.ok();
    assert!(!has(&page, "sent into the void"), "still dropped: {page}");
    ada.get(&format!("/users/{bob_at}")).await.ok();
}

#[tokio::test]
async fn a_silenced_domain_leaves_public_lists_but_not_the_following_feed() {
    let (fed, mut ada, mut bob) = Box::pin(pair()).await;
    let mut root = fed.a.admin("boss").await;
    bob.post_text("zebra crossing ahead").await;
    arrived(&fed, &mut ada, "zebra crossing ahead").await;

    let foryou = ada.get("/timelines/foryou").await.ok();
    assert!(has(&foryou, "zebra crossing ahead"), "{foryou}");
    let found = ada.get("/search?q=zebra&type=posts").await.ok();
    assert!(has(
        &json!({"items": found["posts"]}),
        "zebra crossing ahead"
    ));
    let people = ada.get("/search?q=bob&type=people").await.ok();
    assert_eq!(people["users"][0]["handle"], "bob", "{people}");

    root.post(
        "/admin/federation/blocks",
        json!({"domain": fed.b.domain, "severity": "silence"}),
    )
    .await
    .ok();
    let foryou = ada.get("/timelines/foryou").await.ok();
    assert!(!has(&foryou, "zebra crossing ahead"), "{foryou}");
    let found = ada.get("/search?q=zebra&type=posts").await.ok();
    assert_eq!(found["posts"], json!([]), "{found}");
    let people = ada.get("/search?q=bob&type=people").await.ok();
    assert_eq!(people["users"], json!([]), "{people}");
    let following = ada.get("/timelines/following").await.ok();
    assert!(has(&following, "zebra crossing ahead"), "{following}");

    bob.post_text("zebra again").await;
    arrived(&fed, &mut ada, "zebra again").await;
    let foryou = ada.get("/timelines/foryou").await.ok();
    assert!(!has(&foryou, "zebra again"), "{foryou}");
}
