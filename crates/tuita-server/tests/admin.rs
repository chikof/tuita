//! `tuita admin` and `tuita seed` bodies, and the staff console routes.

mod common;

use sqlx::PgPool;
use tuita_server::admin::{self, Action, AdminError};
use tuita_server::seed::{self, Outcome};

async fn count(db: &PgPool, sql: &str) -> i64 {
    sqlx::query_scalar(sqlx::AssertSqlSafe(sql.to_owned()))
        .fetch_one(db)
        .await
        .expect("count")
}

#[sqlx::test(migrations = "./migrations")]
async fn seed_creates_six_users_and_forty_posts_once(db: PgPool) {
    assert_eq!(
        seed::run(&db).await.expect("seeds"),
        Outcome::Seeded {
            users: 6,
            posts: 40
        }
    );
    assert_eq!(
        count(&db, "select count(*) from users where handle <> 'tuita'").await,
        6
    );
    assert_eq!(count(&db, "select count(*) from posts").await, 40);
    assert_eq!(
        count(
            &db,
            "select count(*) from posts where reply_to_id is not null"
        )
        .await,
        8
    );
    assert_eq!(
        count(
            &db,
            "select count(*) from users where password_hash is not null"
        )
        .await,
        0
    );

    // Counters agree with the rows they count.
    assert_eq!(
        count(
            &db,
            "select count(*) from users u where posts_count <> (select count(*) from posts where author_id = u.id)
               or followers_count <> (select count(*) from follows where followee_id = u.id)
               or following_count <> (select count(*) from follows where follower_id = u.id)"
        )
        .await,
        0
    );
    assert_eq!(
        count(
            &db,
            "select count(*) from posts p where like_count <> (select count(*) from likes where post_id = p.id)
               or repost_count <> (select count(*) from reposts where post_id = p.id)
               or reply_count <> (select count(*) from posts r where r.reply_to_id = p.id)"
        )
        .await,
        0
    );
    // Ids sort like created_at, which keyset pagination relies on.
    assert_eq!(
        count(
            &db,
            "select count(*) from posts a join posts b on a.id < b.id and a.created_at > b.created_at"
        )
        .await,
        0
    );
    assert_eq!(
        seed::run(&db).await.expect("second run"),
        Outcome::AlreadyPopulated
    );
    assert_eq!(count(&db, "select count(*) from posts").await, 40);
}

#[sqlx::test(migrations = "./migrations")]
async fn admin_commands_change_roles_badges_and_suspension(db: PgPool) {
    seed::run(&db).await.expect("seeds");
    let role = |handle: &'static str| {
        let db = db.clone();
        async move {
            sqlx::query_scalar::<_, String>("select role from users where handle = $1")
                .bind(handle)
                .fetch_one(&db)
                .await
                .expect("role")
        }
    };
    admin::apply(&db, Action::Promote, "@Ada_Builds")
        .await
        .expect("promote");
    assert_eq!(role("ada_builds").await, "admin");
    admin::apply(&db, Action::Moderator, "kofi")
        .await
        .expect("moderator");
    assert_eq!(role("kofi").await, "moderator");
    admin::apply(&db, Action::Demote, "kofi")
        .await
        .expect("demote");
    assert_eq!(role("kofi").await, "user");
    admin::apply(&db, Action::Verify, "jonas")
        .await
        .expect("verify");
    admin::apply(&db, Action::Suspend, "jonas")
        .await
        .expect("suspend");

    let rows = admin::list(&db).await.expect("list");
    let line = |row: &admin::StaffRow| {
        (
            row.handle.clone(),
            row.role.clone(),
            row.verified,
            row.suspended,
        )
    };
    assert_eq!(
        rows.iter().map(line).collect::<Vec<_>>(),
        [
            ("ada_builds".to_owned(), "admin".to_owned(), false, false),
            ("jonas".to_owned(), "user".to_owned(), true, true),
            ("lena".to_owned(), "user".to_owned(), true, false),
            ("priya".to_owned(), "user".to_owned(), true, false),
        ]
    );
    admin::apply(&db, Action::Unsuspend, "jonas")
        .await
        .expect("unsuspend");
    admin::apply(&db, Action::Unverify, "jonas")
        .await
        .expect("unverify");
    assert_eq!(admin::list(&db).await.expect("list").len(), 3);

    match admin::apply(&db, Action::Promote, "nobody").await {
        Err(AdminError::NoSuchAccount(handle)) => assert_eq!(&*handle, "nobody"),
        other => panic!("expected NoSuchAccount, got {other:?}"),
    }
}

// ---------------------------------------------------------------- the staff console

use axum::http::StatusCode;
use serde_json::{Value, json};

async fn staff(app: &common::App, db: &PgPool, handle: &str, role: &str) -> common::Person {
    let person = app.person(db, handle).await;
    sqlx::query("update users set role = $2, password_hash = 'x' where id = $1")
        .bind(person.id)
        .bind(role)
        .execute(db)
        .await
        .expect("role");
    person
}

fn filter(blocked: &str, reserved: &str) -> Value {
    json!({ "filter": { "blocked": blocked, "allowed": "", "reserved": reserved } })
}

async fn audit_actions(db: &PgPool) -> Vec<String> {
    sqlx::query_scalar("select action from audit_log order by id")
        .fetch_all(db)
        .await
        .expect("audit")
}

#[sqlx::test(migrations = "./migrations")]
async fn a_moderator_cannot_change_settings_but_an_admin_can(db: PgPool) {
    let app = common::app(db.clone());
    let mut moderator = staff(&app, &db, "moddy", "moderator").await;
    let mut admin = staff(&app, &db, "boss", "admin").await;
    moderator
        .browser
        .patch("/admin/settings", json!({ "name": "Nope" }))
        .await
        .err(StatusCode::FORBIDDEN, "admin only");
    moderator
        .browser
        .get("/admin/audit")
        .await
        .err(StatusCode::FORBIDDEN, "admin only");
    let settings = admin
        .browser
        .patch(
            "/admin/settings",
            json!({ "name": "Quiet corner", "minAge": 16, "signups": "invite" }),
        )
        .await
        .ok();
    assert_eq!(settings["name"], "Quiet corner");
    assert_eq!(settings["minAge"], 16);
    assert_eq!(settings["signups"], "invite");
    admin
        .browser
        .patch("/admin/settings", json!({ "minAge": 30 }))
        .await
        .err(StatusCode::BAD_REQUEST, "Minimum age must be 13\u{2013}21.");
    assert_eq!(audit_actions(&db).await, ["instance.settings"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_console_is_closed_to_everyone_else(db: PgPool) {
    let app = common::app(db.clone());
    let mut nobody = app.browser();
    nobody
        .get("/admin/stats")
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");
    let mut user = app.person(&db, "plain").await;
    user.browser
        .get("/admin/stats")
        .await
        .err(StatusCode::FORBIDDEN, "staff only");
    let mut moderator = staff(&app, &db, "moddy", "moderator").await;
    let stats = moderator.browser.get("/admin/stats").await.ok();
    assert_eq!(stats["days"].as_array().map(Vec::len), Some(14));
    let heatmap = stats["heatmap"].as_array().expect("heatmap");
    assert_eq!(heatmap.len(), 7);
    assert!(
        heatmap
            .iter()
            .all(|row| row.as_array().map(Vec::len) == Some(24))
    );
    assert_eq!(stats["users"]["local"], 1);
    assert_eq!(stats["users"]["demo"], 1);
    let users = moderator.browser.get("/admin/users?filter=all").await.ok();
    assert_eq!(users["items"].as_array().map(Vec::len), Some(2));
    moderator
        .browser
        .get("/admin/users?filter=nope")
        .await
        .err(StatusCode::BAD_REQUEST, "Invalid query.");
}

#[sqlx::test(migrations = "./migrations")]
async fn delete_and_suspend_deletes_the_post_suspends_the_user_and_writes_two_audit_rows(
    db: PgPool,
) {
    let app = common::app(db.clone());
    let mut moderator = staff(&app, &db, "moddy", "moderator").await;
    let mut author = app.person(&db, "troll").await;
    let mut reporter = app.person(&db, "witness").await;
    let post = author.browser.post_text("something awful").await;
    let post_id = post["id"].as_str().expect("id").to_owned();
    reporter
        .browser
        .post(
            "/reports",
            json!({ "postId": post_id, "reason": "harassment", "note": "" }),
        )
        .await
        .ok();
    let queue = moderator.browser.get("/admin/reports").await.ok();
    let report = &queue["items"][0];
    assert_eq!(report["target"]["handle"], "troll");
    assert_eq!(report["post"]["id"], post_id.as_str());
    let report_id = report["id"].as_str().expect("report id").to_owned();
    moderator
        .browser
        .post(
            &format!("/admin/reports/{report_id}/resolve"),
            json!({ "action": "delete_and_suspend" }),
        )
        .await
        .ok();
    let deleted: bool =
        sqlx::query_scalar("select deleted_at is not null from posts where id = $1::uuid")
            .bind(&post_id)
            .fetch_one(&db)
            .await
            .expect("post");
    assert!(deleted);
    let suspended: bool =
        sqlx::query_scalar("select suspended_at is not null from users where id = $1")
            .bind(author.id)
            .fetch_one(&db)
            .await
            .expect("user");
    assert!(suspended);
    assert_eq!(audit_actions(&db).await, ["post.delete", "user.suspend"]);
    let actors: i64 = sqlx::query_scalar("select count(*) from audit_log where actor_id = $1")
        .bind(moderator.id)
        .fetch_one(&db)
        .await
        .expect("actors");
    assert_eq!(actors, 2);
    // The suspended author is signed out and the report is resolved, not resolvable twice.
    author.browser.get("/me").await.ok();
    moderator
        .browser
        .post(
            &format!("/admin/reports/{report_id}/resolve"),
            json!({ "action": "dismiss" }),
        )
        .await
        .err(StatusCode::CONFLICT, "This report was already resolved.");
    let resolved = moderator
        .browser
        .get("/admin/reports?status=resolved")
        .await
        .ok();
    assert_eq!(resolved["items"][0]["resolution"], "delete_and_suspend");
}

#[sqlx::test(migrations = "./migrations")]
async fn an_invite_with_one_use_refuses_the_second_signup(db: PgPool) {
    let app = common::app(db.clone());
    let mut admin = staff(&app, &db, "boss", "admin").await;
    admin
        .browser
        .patch("/admin/settings", json!({ "signups": "invite" }))
        .await
        .ok();
    let invite = admin
        .browser
        .post("/admin/invites", json!({ "maxUses": 1, "note": "for ada" }))
        .await
        .ok();
    let code = invite["code"].as_str().expect("code").to_owned();
    assert_eq!(code.len(), 11);

    let body = |handle: &str, invite: Option<&str>| json!({ "handle": handle, "displayName": handle, "password": common::PASSWORD, "acceptTerms": true, "invite": invite });
    let mut first = app.browser();
    first.post("/auth/signup", body("adaaa", None)).await.err(
        StatusCode::FORBIDDEN,
        "Sign-ups are closed on this instance right now. You need an invite code.",
    );
    first
        .post("/auth/signup", body("adaaa", Some("WRONG-CODE0")))
        .await
        .err(
            StatusCode::FORBIDDEN,
            "That invite code is invalid or has been used up.",
        );
    first
        .post("/auth/signup", body("adaaa", Some(&code.to_lowercase())))
        .await
        .ok();
    let mut second = app.browser_at([198, 51, 100, 2]);
    second
        .post("/auth/signup", body("bobbb", Some(&code)))
        .await
        .err(
            StatusCode::FORBIDDEN,
            "That invite code is invalid or has been used up.",
        );

    let listed = admin.browser.get("/admin/invites").await.ok();
    assert_eq!(listed[0]["uses"], 1);
    assert_eq!(listed[0]["usedBy"], json!(["adaaa"]));
    let none: i64 = sqlx::query_scalar("select count(*) from users where handle = 'bobbb'")
        .fetch_one(&db)
        .await
        .expect("count");
    assert_eq!(none, 0);

    let spare = admin.browser.post("/admin/invites", json!({})).await.ok();
    let spare_code = spare["code"].as_str().expect("code").to_owned();
    admin
        .browser
        .delete(&format!("/admin/invites/{spare_code}"))
        .await
        .ok();
    second
        .post("/auth/signup", body("bobbb", Some(&spare_code)))
        .await
        .err(
            StatusCode::FORBIDDEN,
            "That invite code is invalid or has been used up.",
        );
    admin
        .browser
        .delete("/admin/invites/NOPE-NOPE0")
        .await
        .err(StatusCode::NOT_FOUND, "Invite not found.");
}

#[sqlx::test(migrations = "./migrations")]
async fn announcements_are_public_until_ended(db: PgPool) {
    let app = common::app(db.clone());
    let mut admin = staff(&app, &db, "boss", "admin").await;
    let made = admin
        .browser
        .post("/admin/announcements", json!({ "text": "Maintenance tonight", "tone": "warning", "link": "/status", "days": 2 }))
        .await
        .ok();
    admin
        .browser
        .post(
            "/admin/announcements",
            json!({ "text": "x", "link": "javascript:alert(1)" }),
        )
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "Links must start with http://, https:// or /.",
        );
    let mut visitor = app.browser();
    let shown = visitor.get("/announcements").await.ok();
    assert_eq!(shown[0]["text"], "Maintenance tonight");
    assert_eq!(shown[0]["tone"], "warning");
    let id = made["id"].as_str().expect("id");
    admin
        .browser
        .delete(&format!("/admin/announcements/{id}"))
        .await
        .ok();
    assert_eq!(visitor.get("/announcements").await.ok(), json!([]));
    assert_eq!(
        audit_actions(&db).await,
        ["announcement.create", "announcement.end"]
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn the_filter_scan_and_the_admin_lists_follow_the_settings(db: PgPool) {
    let app = common::app(db.clone());
    let mut admin = staff(&app, &db, "boss", "admin").await;
    let rude = common::insert_user(&db, "rudeone").await;
    sqlx::query(
        "update users set display_name = 'what a gizmo', password_hash = 'x' where id = $1",
    )
    .bind(rude)
    .execute(&db)
    .await
    .expect("update");
    let mut poster = app.person(&db, "poster").await;

    // With no admin words nothing is flagged or refused, not even a slur.
    let scan = admin.browser.get("/admin/filter/scan").await.ok();
    assert_eq!(scan["flagged"], json!([]));
    poster
        .browser
        .post("/posts", json!({ "text": "a gizmo and f4gg0t" }))
        .await
        .ok();
    poster
        .browser
        .patch("/me", json!({ "bio": "gizmo fan" }))
        .await
        .ok();
    poster
        .browser
        .patch("/me", json!({ "bio": "plain" }))
        .await
        .ok();

    admin
        .browser
        .patch("/admin/settings", filter("gizmo", "council"))
        .await
        .ok();
    let scan = admin.browser.get("/admin/filter/scan").await.ok();
    assert_eq!(scan["flagged"][0]["user"]["user"]["handle"], "rudeone");
    assert_eq!(scan["flagged"][0]["field"], "name");
    assert_eq!(scan["flagged"][0]["reason"], "blocked words");

    let mut visitor = app.browser();
    let signup = |handle: &str, name: &str| json!({ "handle": handle, "displayName": name, "password": common::PASSWORD, "acceptTerms": true });
    visitor
        .post("/auth/signup", signup("council", "Council"))
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "That handle is reserved. Please pick another.",
        );
    visitor
        .post("/auth/signup", signup("gizmo_fan", "Fan"))
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "That handle isn\u{2019}t allowed. Please pick another.",
        );
    let member = visitor
        .post("/auth/signup", signup("fine_one", "Fine"))
        .await
        .ok();
    assert_eq!(member["user"]["user"]["handle"], "fine_one");

    poster
        .browser
        .post("/posts", json!({ "text": "a g1zm0 appears" }))
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "Your post contains blocked words, which isn\u{2019}t allowed on this instance.",
        );
    poster
        .browser
        .patch("/me", json!({ "bio": "gizmo fan" }))
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "Your bio contains blocked words, which isn\u{2019}t allowed here.",
        );
}

#[sqlx::test(migrations = "./migrations")]
async fn user_actions_write_audit_rows_and_respect_roles(db: PgPool) {
    let app = common::app(db.clone());
    let mut admin = staff(&app, &db, "boss", "admin").await;
    let mut moderator = staff(&app, &db, "moddy", "moderator").await;
    let victim = app.person(&db, "victim").await;
    let verified = moderator
        .browser
        .patch("/admin/users/victim", json!({ "verified": true }))
        .await
        .ok();
    assert_eq!(verified["user"]["verified"], true);
    moderator
        .browser
        .patch("/admin/users/victim", json!({ "role": "moderator" }))
        .await
        .err(StatusCode::FORBIDDEN, "Only admins can change roles.");
    moderator
        .browser
        .patch("/admin/users/boss", json!({ "suspended": true }))
        .await
        .err(StatusCode::FORBIDDEN, "Only admins can suspend admins.");
    admin
        .browser
        .patch("/admin/users/boss", json!({ "suspended": true }))
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "You can\u{2019}t suspend yourself.",
        );
    admin
        .browser
        .patch(
            &format!("/admin/users/id:{}", victim.id),
            json!({ "suspended": true }),
        )
        .await
        .ok();
    let detail = admin.browser.get("/admin/users/victim").await.ok();
    assert_eq!(detail["user"]["user"]["suspended"], true);
    assert_eq!(detail["history"][0]["action"], "user.suspend");
    admin
        .browser
        .post(
            "/admin/users/victim/rename",
            json!({ "handle": "renamed", "resetBio": true }),
        )
        .await
        .ok();
    admin
        .browser
        .post("/admin/users/renamed/rename", json!({}))
        .await
        .err(StatusCode::BAD_REQUEST, "Nothing to change.");
    admin
        .browser
        .patch("/admin/users/renamed/org", json!({ "org": true }))
        .await
        .ok();
    admin.browser.delete("/admin/users/renamed").await.ok();
    let after = admin
        .browser
        .get("/admin/users/id:".to_owned().as_str())
        .await;
    assert_eq!(after.status, StatusCode::NOT_FOUND);
    assert_eq!(
        audit_actions(&db).await,
        [
            "user.verify",
            "user.suspend",
            "user.rename",
            "user.org",
            "user.delete"
        ]
    );
    let log = admin.browser.get("/admin/audit?limit=2").await.ok();
    assert_eq!(log["items"][0]["action"], "user.delete");
    assert!(log["next"].is_string());
}

#[sqlx::test(migrations = "./migrations")]
async fn the_command_line_uses_the_same_functions_and_logs_a_null_actor(db: PgPool) {
    let app = common::app(db.clone());
    let mut user = app.person(&db, "someone").await;
    admin::apply(&db, Action::Suspend, "@someone")
        .await
        .expect("suspend");
    user.browser.get("/me").await.ok();
    let sessions: i64 = sqlx::query_scalar("select count(*) from sessions")
        .fetch_one(&db)
        .await
        .expect("sessions");
    assert_eq!(
        sessions, 0,
        "suspending ends the sessions, as the route does"
    );
    let row: (String, Option<uuid::Uuid>, String) =
        sqlx::query_as("select action, actor_id, detail->>'text' from audit_log")
            .fetch_one(&db)
            .await
            .expect("row");
    assert_eq!(
        row,
        ("user.suspend".to_owned(), None, "command line".to_owned())
    );
}

// ---------------------------------------------------------------- review fixes

#[sqlx::test(migrations = "./migrations")]
async fn admin_words_apply_to_list_names_and_edits_follow_the_post_filter(db: PgPool) {
    let app = common::app(db.clone());
    let mut admin = staff(&app, &db, "boss", "admin").await;
    let mut user = app.person(&db, "lister").await;
    let post = user.browser.post_text("a fine post").await;
    let id = post["id"].as_str().expect("id").to_owned();
    admin
        .browser
        .patch("/admin/settings", filter("gizmo", ""))
        .await
        .ok();
    let refused = "That list name or description isn\u{2019}t allowed.";
    user.browser
        .post("/lists", json!({ "name": "gizmo club", "description": "" }))
        .await
        .err(StatusCode::BAD_REQUEST, refused);
    let list = user
        .browser
        .post("/lists", json!({ "name": "fine club", "description": "" }))
        .await
        .ok();
    user.browser
        .patch(
            &format!("/lists/{}", list["id"].as_str().expect("id")),
            json!({ "description": "all about a gizmo" }),
        )
        .await
        .err(StatusCode::BAD_REQUEST, refused);
    user.browser
        .patch(&format!("/posts/{id}"), json!({ "text": "now a gizmo" }))
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "Your post contains blocked words, which isn\u{2019}t allowed on this instance.",
        );
    user.browser
        .patch(&format!("/posts/{id}"), json!({ "text": "a calmer edit" }))
        .await
        .ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn moderators_cannot_touch_admin_accounts_and_the_last_admin_stays(db: PgPool) {
    let app = common::app(db.clone());
    let mut moderator = staff(&app, &db, "moddy", "moderator").await;
    let _boss = staff(&app, &db, "boss", "admin").await;
    let denied = "Only admins can change admin accounts.";
    moderator
        .browser
        .patch("/admin/users/boss", json!({ "verified": true }))
        .await
        .err(StatusCode::FORBIDDEN, denied);
    moderator
        .browser
        .post("/admin/users/boss/rename", json!({ "resetBio": true }))
        .await
        .err(StatusCode::FORBIDDEN, denied);
    moderator
        .browser
        .patch("/admin/users/boss/org", json!({ "org": true }))
        .await
        .err(StatusCode::FORBIDDEN, denied);

    // Two admins acting on each other at once: one of them wins, never both.
    for round in 0..5 {
        let mut a = staff(&app, &db, &format!("alpha{round}"), "admin").await;
        let mut b = staff(&app, &db, &format!("beta{round}"), "admin").await;
        sqlx::query("update users set role = 'user' where role = 'admin' and handle not like $1 and handle not like $2 and handle <> 'boss'")
            .bind(format!("alpha{round}"))
            .bind(format!("beta{round}"))
            .execute(&db)
            .await
            .expect("tidy");
        sqlx::query("update users set role = 'user' where handle = 'boss'")
            .execute(&db)
            .await
            .expect("boss");
        let (to_beta, to_alpha) = (
            format!("/admin/users/beta{round}"),
            format!("/admin/users/alpha{round}"),
        );
        let (x, y) = tokio::join!(
            a.browser.patch(&to_beta, json!({ "suspended": true })),
            b.browser.patch(&to_alpha, json!({ "role": "user" })),
        );
        let oks = [x.status, y.status]
            .iter()
            .filter(|s| **s == StatusCode::OK)
            .count();
        assert_eq!(oks, 1, "round {round}: {} / {}", x.body, y.body);
        let left: i64 = sqlx::query_scalar(
            "select count(*) from users where role = 'admin' and suspended_at is null",
        )
        .fetch_one(&db)
        .await
        .expect("admins");
        assert!(left >= 1, "round {round}: no active admin left");
        sqlx::query("update users set role = 'user'")
            .execute(&db)
            .await
            .expect("reset");
        sqlx::query("update users set role = 'admin' where handle = 'moddy'")
            .execute(&db)
            .await
            .expect("keep one");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn announcement_links_stay_on_the_site_or_http_and_scans_are_rate_limited(db: PgPool) {
    let app = common::app(db.clone());
    let mut admin = staff(&app, &db, "boss", "admin").await;
    let message = "Links must start with http://, https:// or /.";
    for link in [
        "//evil.example/x",
        "/\\evil.example",
        "ftp://x",
        "javascript:alert(1)",
    ] {
        admin
            .browser
            .post(
                "/admin/announcements",
                json!({ "text": "hi", "link": link }),
            )
            .await
            .err(StatusCode::BAD_REQUEST, message);
    }
    for link in ["/status", "http://example.org/a", "https://example.org/a"] {
        admin
            .browser
            .post(
                "/admin/announcements",
                json!({ "text": "hi", "link": link }),
            )
            .await
            .ok();
    }
    for _ in 0..6 {
        admin.browser.get("/admin/filter/scan").await.ok();
    }
    admin.browser.get("/admin/filter/scan").await.err(
        StatusCode::TOO_MANY_REQUESTS,
        "Too many requests. Try again in a few minutes.",
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn the_bot_flag_is_rate_limited(db: PgPool) {
    let app = common::app(db.clone());
    let mut bot = app.person(&db, "newsbot").await;
    for _ in 0..10 {
        bot.browser
            .patch("/me/bot", json!({ "bot": true }))
            .await
            .ok();
    }
    bot.browser
        .patch("/me/bot", json!({ "bot": true }))
        .await
        .err(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many requests. Try again in a few minutes.",
        );
}

#[sqlx::test(migrations = "./migrations")]
async fn the_overview_buckets_days_in_utc_whatever_the_session_time_zone_is(db: PgPool) {
    let author = common::insert_user(&db, "writer").await;
    sqlx::query(
        "insert into posts (id, author_id, text, created_at)
         values ($1, $2, 'early', (date_trunc('day', now() at time zone 'UTC') at time zone 'UTC') + interval '1 hour')",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(author)
    .execute(&db)
    .await
    .expect("post");
    // sqlx starts every session in UTC; the one connection of this pool is switched by hand and
    // keeps its zone when it goes back to the pool.
    let auckland = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect_with((*db.connect_options()).clone())
        .await
        .expect("pool");
    sqlx::query("set timezone = '<+13>-13'")
        .execute(&auckland)
        .await
        .expect("zone");
    let zone: String = sqlx::query_scalar("show timezone")
        .fetch_one(&auckland)
        .await
        .expect("zone");
    assert_eq!(zone, "<+13>-13");
    let utc = tuita_server::db::admin_reads::stats(&db)
        .await
        .expect("stats");
    let shifted = tuita_server::db::admin_reads::stats(&auckland)
        .await
        .expect("stats");
    assert_eq!(utc.days, shifted.days);
    assert_eq!(shifted.days.len(), 14);
    assert_eq!(shifted.days[13].posts, 1, "{:?}", shifted.days[13]);
}
