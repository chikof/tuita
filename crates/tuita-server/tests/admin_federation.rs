//! `/admin/federation`: domain blocks, the delivery queue, retry and the instance actor's key.
#![allow(missing_docs)]

mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use common::{App, Person, app};

async fn admin(app: &App, db: &PgPool) -> Person {
    let person = app.person(db, "boss").await;
    sqlx::query("update users set role = 'admin' where id = $1")
        .bind(person.id)
        .execute(db)
        .await
        .expect("admin");
    person
}

/// Queues one delivery to `inbox`.
async fn queue(db: &PgPool, inbox: &str, attempts: i32, error: Option<&str>, due_in_minutes: i32) {
    sqlx::query(
        "insert into ap_deliveries (id, inbox_url, activity, attempts, last_error, next_attempt_at)
         values ($1, $2, '{\"type\": \"Create\"}', $3, $4, now() + make_interval(mins => $5))",
    )
    .bind(Uuid::now_v7())
    .bind(inbox)
    .bind(attempts)
    .bind(error)
    .bind(due_in_minutes)
    .execute(db)
    .await
    .expect("queue");
}

async fn queued(db: &PgPool) -> i64 {
    sqlx::query_scalar("select count(*) from ap_deliveries")
        .fetch_one(db)
        .await
        .expect("count")
}

async fn audit(db: &PgPool, action: &str) -> Vec<(String, String, String)> {
    sqlx::query_as(
        "select u.handle, a.target, coalesce(a.detail->>'text', '') from audit_log a
           join users u on u.id = a.actor_id where a.action = $1 order by a.id",
    )
    .bind(action)
    .fetch_all(db)
    .await
    .expect("audit")
}

#[sqlx::test(migrations = "./migrations")]
async fn only_admins_open_the_console(db: PgPool) {
    let app = app(db.clone());
    let mut anon = app.browser();
    assert_eq!(
        anon.get("/admin/federation").await.status,
        StatusCode::UNAUTHORIZED
    );
    let mut ana = app.person(&db, "ana").await;
    ana.browser
        .get("/admin/federation")
        .await
        .err(StatusCode::FORBIDDEN, "admin only");
    sqlx::query("update users set role = 'moderator' where id = $1")
        .bind(ana.id)
        .execute(&db)
        .await
        .expect("moderator");
    let calls = [
        ana.browser.get("/admin/federation").await,
        ana.browser
            .post(
                "/admin/federation/blocks",
                json!({"domain": "a.example", "severity": "silence"}),
            )
            .await,
        ana.browser
            .patch("/admin/federation/blocks/a.example", json!({"reason": "x"}))
            .await,
        ana.browser
            .delete("/admin/federation/blocks/a.example")
            .await,
        ana.browser.post_empty("/admin/federation/retry").await,
    ];
    for reply in calls {
        reply.err(StatusCode::FORBIDDEN, "admin only");
    }
    assert_eq!(queued(&db).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn api_tokens_are_refused(db: PgPool) {
    let app = app(db.clone());
    let mut boss = admin(&app, &db).await;
    let created = boss
        .browser
        .post(
            "/tokens",
            json!({"name": "script", "scopes": ["read", "write"]}),
        )
        .await
        .ok();
    let secret = created["token"].as_str().expect("secret");
    let mut script = app.browser();
    script.origin = None;
    script
        .headers
        .push(("authorization".to_owned(), format!("Bearer {secret}")));
    let refused = "API tokens can\u{2019}t be used for this. Sign in on the website instead.";
    script
        .get("/admin/federation")
        .await
        .err(StatusCode::FORBIDDEN, refused);
    script
        .post(
            "/admin/federation/blocks",
            json!({"domain": "a.example", "severity": "suspend"}),
        )
        .await
        .err(StatusCode::FORBIDDEN, refused);
    script
        .post_empty("/admin/federation/retry")
        .await
        .err(StatusCode::FORBIDDEN, refused);
    assert_eq!(
        boss.browser.get("/admin/federation").await.status,
        StatusCode::OK
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn the_overview_shows_the_queue_the_actor_and_what_is_known(db: PgPool) {
    let app = app(db.clone());
    let mut boss = admin(&app, &db).await;

    let empty = boss.browser.get("/admin/federation").await.ok();
    assert_eq!(empty["domain"], "social.example");
    assert_eq!(
        empty["queue"],
        json!({"count": 0, "due": 0, "oldest": null, "failing": []})
    );
    assert_eq!(empty["blocks"], json!([]));
    assert_eq!(empty["knownDomains"], 0);
    assert_eq!(empty["remoteAccounts"], 0);
    assert_eq!(empty["remotePosts"], 0);
    assert_eq!(empty["actor"]["id"], "https://social.example/actor");
    assert_eq!(
        empty["actor"]["keyId"],
        "https://social.example/actor#main-key"
    );
    let pem = empty["actor"]["publicKeyPem"].as_str().expect("pem");
    assert!(pem.starts_with("-----BEGIN PUBLIC KEY-----"));
    let fingerprint = empty["actor"]["fingerprint"].as_str().expect("fingerprint");
    assert_eq!(fingerprint.len(), 64);
    assert!(
        fingerprint
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    );
    assert_eq!(
        fingerprint,
        tuita_core::ap::fingerprint(pem).expect("fingerprint")
    );

    let again = boss.browser.get("/admin/federation").await.ok();
    assert_eq!(again["actor"], empty["actor"]);
    // The public instance actor shows the very key the console fingerprints.
    let public = tuita_server::federation::keys::instance_public_pem(&app.state)
        .await
        .expect("pem");
    assert_eq!(public, pem);

    for (handle, domain) in [
        ("eve", "b.example"),
        ("fay", "b.example"),
        ("gus", "c.example:8081"),
    ] {
        let id = Uuid::now_v7();
        sqlx::query("insert into users (id, handle, domain) values ($1, $2, $3)")
            .bind(id)
            .bind(handle)
            .bind(domain)
            .execute(&db)
            .await
            .expect("remote user");
        sqlx::query("insert into posts (id, author_id, text, ap_id) values ($1, $2, 'hi', $3)")
            .bind(Uuid::now_v7())
            .bind(id)
            .bind(format!("https://{domain}/n/{handle}"))
            .execute(&db)
            .await
            .expect("remote post");
    }
    let known = boss.browser.get("/admin/federation").await.ok();
    assert_eq!(known["knownDomains"], 2);
    assert_eq!(known["remoteAccounts"], 3);
    assert_eq!(known["remotePosts"], 3);
    assert_eq!(known["actor"], empty["actor"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_queue_counts_waiting_and_lists_failing_instances(db: PgPool) {
    let app = app(db.clone());
    let mut boss = admin(&app, &db).await;
    queue(&db, "https://ok.example/inbox", 0, None, 0).await;
    queue(&db, "https://slow.example/inbox", 2, Some("older"), 10).await;
    queue(
        &db,
        "https://slow.example/users/a/inbox",
        5,
        Some("HTTP 503"),
        30,
    )
    .await;
    queue(&db, "https://slow.example/inbox", 1, None, 60).await;
    queue(
        &db,
        "https://down.example:8443/inbox",
        3,
        Some("connection refused"),
        5,
    )
    .await;

    let overview = boss.browser.get("/admin/federation").await.ok();
    let queue = &overview["queue"];
    assert_eq!(queue["count"], 5);
    assert_eq!(queue["due"], 1);
    assert!(queue["oldest"].as_str().is_some_and(|t| t.ends_with('Z')));
    let failing = queue["failing"].as_array().expect("failing");
    assert_eq!(failing.len(), 2);
    assert_eq!(failing[0]["domain"], "slow.example");
    assert_eq!(failing[0]["count"], 3);
    assert_eq!(failing[0]["attempts"], 5);
    assert_eq!(failing[0]["lastError"], "HTTP 503");
    assert_eq!(failing[1]["domain"], "down.example:8443");
    assert_eq!(failing[1]["count"], 1);
    assert_eq!(failing[1]["attempts"], 3);
    assert_eq!(failing[1]["lastError"], "connection refused");
    assert!(failing[1]["nextAttemptAt"].as_str().is_some());
}

#[sqlx::test(migrations = "./migrations")]
async fn retry_makes_everything_due_and_is_audited(db: PgPool) {
    let app = app(db.clone());
    let mut boss = admin(&app, &db).await;
    queue(&db, "https://a.example/inbox", 4, Some("boom"), 120).await;
    queue(&db, "https://b.example/inbox", 0, None, 300).await;
    let due: i64 =
        sqlx::query_scalar("select count(*) from ap_deliveries where next_attempt_at <= now()")
            .fetch_one(&db)
            .await
            .expect("due");
    assert_eq!(due, 0);

    assert_eq!(
        boss.browser
            .post_empty("/admin/federation/retry")
            .await
            .ok(),
        json!({"requeued": 2})
    );
    let due: i64 =
        sqlx::query_scalar("select count(*) from ap_deliveries where next_attempt_at <= now()")
            .fetch_one(&db)
            .await
            .expect("due");
    assert_eq!(due, 2);
    assert_eq!(queued(&db).await, 2);
    assert_eq!(
        boss.browser.get("/admin/federation").await.ok()["queue"]["due"],
        2
    );

    assert_eq!(
        boss.browser
            .post_empty("/admin/federation/retry")
            .await
            .ok(),
        json!({"requeued": 2})
    );
    let rows = audit(&db, "federation.retry").await;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].0, "boss");
    sqlx::query("delete from ap_deliveries")
        .execute(&db)
        .await
        .expect("empty");
    assert_eq!(
        boss.browser
            .post_empty("/admin/federation/retry")
            .await
            .ok(),
        json!({"requeued": 0})
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn blocks_validate_normalise_and_replace(db: PgPool) {
    let app = app(db.clone());
    let mut boss = admin(&app, &db).await;
    let post = |domain: &str, severity: &str, reason: Option<&str>| {
        let mut body = json!({"domain": domain, "severity": severity});
        if let Some(reason) = reason {
            body["reason"] = json!(reason);
        }
        body
    };
    for (domain, message) in [
        ("", "Enter a domain."),
        ("   ", "Enter a domain."),
        ("not a domain", "That domain isn\u{2019}t valid."),
        ("a.example/path", "That domain isn\u{2019}t valid."),
        ("user@a.example", "That domain isn\u{2019}t valid."),
        ("a.example:99999", "That domain isn\u{2019}t valid."),
        ("-bad-.example", "That domain isn\u{2019}t valid."),
        (
            "SOCIAL.example",
            "You can\u{2019}t block your own instance.",
        ),
        (
            "https://social.example/@me",
            "You can\u{2019}t block your own instance.",
        ),
    ] {
        boss.browser
            .post("/admin/federation/blocks", post(domain, "suspend", None))
            .await
            .err(StatusCode::BAD_REQUEST, message);
    }
    boss.browser
        .post(
            "/admin/federation/blocks",
            post("a.example", "suspend", Some(&"x".repeat(201))),
        )
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "Reason must be at most 200 characters.",
        );
    boss.browser
        .post(
            "/admin/federation/blocks",
            json!({"domain": "a.example", "severity": "ban"}),
        )
        .await
        .err(StatusCode::BAD_REQUEST, "Invalid JSON.");

    let created = boss
        .browser
        .post(
            "/admin/federation/blocks",
            post(
                "  HTTPS://Spam.Example/users/x ",
                "silence",
                Some("  spam  "),
            ),
        )
        .await
        .ok();
    assert_eq!(created["domain"], "spam.example");
    assert_eq!(created["severity"], "silence");
    assert_eq!(created["reason"], "spam");
    assert_eq!(created["accounts"], 0);
    assert!(created["createdAt"].as_str().is_some());
    let local = boss
        .browser
        .post(
            "/admin/federation/blocks",
            post("127.0.0.1:8081", "suspend", None),
        )
        .await
        .ok();
    assert_eq!(local["domain"], "127.0.0.1:8081");

    sqlx::query("insert into users (id, handle, domain) values ($1, 'eve', 'spam.example')")
        .bind(Uuid::now_v7())
        .execute(&db)
        .await
        .expect("remote");
    let replaced = boss
        .browser
        .post(
            "/admin/federation/blocks",
            post("spam.example", "suspend", Some("worse")),
        )
        .await
        .ok();
    assert_eq!(replaced["severity"], "suspend");
    assert_eq!(replaced["reason"], "worse");
    assert_eq!(replaced["accounts"], 1);
    let blocks = boss.browser.get("/admin/federation").await.ok()["blocks"].clone();
    let list = blocks.as_array().expect("blocks");
    assert_eq!(list.len(), 2);
    assert_eq!(
        list.iter()
            .filter(|block| block["domain"] == "spam.example")
            .count(),
        1
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_suspend_drops_the_queued_deliveries_of_that_domain_only(db: PgPool) {
    let app = app(db.clone());
    let mut boss = admin(&app, &db).await;
    queue(&db, "https://bad.example/inbox", 0, None, 0).await;
    queue(&db, "https://bad.example/users/a/inbox", 3, Some("x"), 5).await;
    queue(&db, "https://bad.example:8443/inbox", 0, None, 0).await;
    queue(&db, "https://good.example/inbox", 0, None, 0).await;
    queue(&db, "https://silenced.example/inbox", 0, None, 0).await;

    boss.browser
        .post(
            "/admin/federation/blocks",
            json!({"domain": "silenced.example", "severity": "silence"}),
        )
        .await
        .ok();
    assert_eq!(queued(&db).await, 5, "a silence keeps the queue");
    boss.browser
        .post(
            "/admin/federation/blocks",
            json!({"domain": "bad.example", "severity": "suspend", "reason": "abuse"}),
        )
        .await
        .ok();
    assert_eq!(queued(&db).await, 3);
    let left: Vec<String> =
        sqlx::query_scalar("select inbox_url from ap_deliveries order by inbox_url")
            .fetch_all(&db)
            .await
            .expect("left");
    assert_eq!(
        left,
        [
            "https://bad.example:8443/inbox",
            "https://good.example/inbox",
            "https://silenced.example/inbox"
        ]
    );
    // Raising a silence to a suspend drops the domain's rows too.
    boss.browser
        .patch(
            "/admin/federation/blocks/silenced.example",
            json!({"severity": "suspend"}),
        )
        .await
        .ok();
    assert_eq!(queued(&db).await, 2);

    let blocks = audit(&db, "domain.block").await;
    assert_eq!(
        blocks,
        [
            (
                "boss".to_owned(),
                "silenced.example".to_owned(),
                "silence".to_owned()
            ),
            (
                "boss".to_owned(),
                "bad.example".to_owned(),
                "suspend: abuse".to_owned()
            ),
            (
                "boss".to_owned(),
                "silenced.example".to_owned(),
                "suspend".to_owned()
            ),
        ]
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn patching_and_removing_blocks(db: PgPool) {
    let app = app(db.clone());
    let mut boss = admin(&app, &db).await;
    boss.browser
        .post(
            "/admin/federation/blocks",
            json!({"domain": "a.example:8080", "severity": "silence", "reason": "meh"}),
        )
        .await
        .ok();

    let both = boss
        .browser
        .patch(
            "/admin/federation/blocks/a.example:8080",
            json!({"severity": "suspend", "reason": "no"}),
        )
        .await
        .ok();
    assert_eq!(
        (both["severity"].clone(), both["reason"].clone()),
        (json!("suspend"), json!("no"))
    );
    let reason_only = boss
        .browser
        .patch(
            "/admin/federation/blocks/A.example%3A8080",
            json!({"reason": "  because  "}),
        )
        .await
        .ok();
    assert_eq!(
        (
            reason_only["severity"].clone(),
            reason_only["reason"].clone()
        ),
        (json!("suspend"), json!("because"))
    );
    let untouched = boss
        .browser
        .patch("/admin/federation/blocks/a.example:8080", json!({}))
        .await
        .ok();
    assert_eq!(untouched["reason"], "because");
    boss.browser
        .patch(
            "/admin/federation/blocks/a.example:8080",
            json!({"reason": "x".repeat(201)}),
        )
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "Reason must be at most 200 characters.",
        );
    boss.browser
        .patch(
            "/admin/federation/blocks/other.example",
            json!({"reason": "x"}),
        )
        .await
        .err(StatusCode::NOT_FOUND, "Block not found.");
    boss.browser
        .patch(
            "/admin/federation/blocks/not%20valid",
            json!({"reason": "x"}),
        )
        .await
        .err(StatusCode::NOT_FOUND, "Block not found.");

    assert_eq!(
        boss.browser
            .delete("/admin/federation/blocks/a.example%3A8080")
            .await
            .ok(),
        json!({"ok": true})
    );
    boss.browser
        .delete("/admin/federation/blocks/a.example:8080")
        .await
        .err(StatusCode::NOT_FOUND, "Block not found.");
    assert_eq!(
        boss.browser.get("/admin/federation").await.ok()["blocks"],
        json!([])
    );
    assert_eq!(
        audit(&db, "domain.unblock").await,
        [(
            "boss".to_owned(),
            "a.example:8080".to_owned(),
            String::new()
        )]
    );
    let _: Value = boss.browser.get("/admin/audit").await.ok();
}
