//! Sign-up, sign-in, the account switcher, CSRF, rate limits, settings and sessions.

mod common;

use axum::http::StatusCode;
use common::{PASSWORD, app, app_with};
use serde_json::json;
use sqlx::PgPool;

fn signup_body(handle: &str) -> serde_json::Value {
    json!({
        "handle": handle,
        "displayName": "Ana Moreno",
        "password": PASSWORD,
        "acceptTerms": true,
    })
}

#[sqlx::test(migrations = "./migrations")]
async fn first_signup_is_admin_second_is_user(db: PgPool) {
    let app = app(db);
    let mut ana = app.browser();
    let me = ana.signup("ana").await;
    assert_eq!(me["role"], "admin");
    assert_eq!(me["user"]["handle"], "ana");
    assert_eq!(me["user"]["staff"], "admin");
    assert_eq!(me["federatedHandle"], "ana@social.example");
    assert_eq!(me["settings"]["dmPolicy"], "everyone");
    assert_eq!(me["settings"]["notifPrefs"]["like"], true);
    assert_eq!(me["user"]["counts"]["followers"], 0);

    let mut ben = app.browser_at([198, 51, 100, 2]);
    let me = ben.signup("ben").await;
    assert_eq!(me["role"], "user");
    assert_eq!(me["user"]["staff"], serde_json::Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn signup_sets_http_only_cookies_and_signs_in(db: PgPool) {
    let app = app(db);
    let mut ana = app.browser();
    let reply = ana.post("/auth/signup", signup_body("ana")).await;
    assert_eq!(reply.status, StatusCode::OK);
    let raw: Vec<&str> = reply
        .headers
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .collect();
    assert_eq!(raw.len(), 2);
    assert!(raw[0].starts_with("tuita_session="), "{raw:?}");
    assert!(raw[1].starts_with("tuita_accounts=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0"));
    for cookie in &raw {
        assert!(
            cookie.contains("HttpOnly") && cookie.contains("SameSite=Lax"),
            "{cookie}"
        );
        assert!(
            cookie.contains("Path=/") && cookie.contains("Secure"),
            "{cookie}"
        );
    }
    assert!(raw[0].contains("Max-Age=2592000"), "{}", raw[0]);

    let me = ana.get("/me").await.ok();
    assert_eq!(me["user"]["user"]["handle"], "ana");
}

#[sqlx::test(migrations = "./migrations")]
async fn me_is_null_when_signed_out(db: PgPool) {
    let app = app(db);
    let reply = app.browser().get("/me").await.ok();
    assert_eq!(reply, json!({ "user": null }));
}

#[sqlx::test(migrations = "./migrations")]
async fn admin_handle_setting_reserves_the_first_admin(db: PgPool) {
    let app = app_with(db, |var| {
        (var == "ADMIN_HANDLE").then(|| "@Boss".to_owned())
    });
    let mut ana = app.browser();
    assert_eq!(ana.signup("ana").await["role"], "user");
    let mut boss = app.browser_at([198, 51, 100, 2]);
    assert_eq!(boss.signup("boss").await["role"], "admin");
}

#[sqlx::test(migrations = "./migrations")]
async fn signup_validation_messages(db: PgPool) {
    common::block_words(&db, "gizmo").await;
    let app = app(db);
    let mut b = app.browser();
    let mut body = signup_body("ana");
    let cases: [(&str, serde_json::Value, StatusCode, &str); 11] = [
        (
            "handle",
            json!("ab"),
            StatusCode::BAD_REQUEST,
            "Handle must be at least 3 characters.",
        ),
        (
            "handle",
            json!("a".repeat(16)),
            StatusCode::BAD_REQUEST,
            "Handle must be at most 15 characters.",
        ),
        (
            "handle",
            json!("ana-b"),
            StatusCode::BAD_REQUEST,
            "Handles use 3\u{2013}15 letters, numbers or underscores.",
        ),
        (
            "handle",
            json!("admin"),
            StatusCode::BAD_REQUEST,
            "That handle is reserved. Please pick another.",
        ),
        (
            "handle",
            json!("deleted_7"),
            StatusCode::BAD_REQUEST,
            "That handle is reserved. Please pick another.",
        ),
        (
            "handle",
            json!("g1zm0"),
            StatusCode::BAD_REQUEST,
            "That handle isn\u{2019}t allowed. Please pick another.",
        ),
        (
            "displayName",
            json!("   "),
            StatusCode::BAD_REQUEST,
            "Name is required.",
        ),
        (
            "displayName",
            json!("x".repeat(51)),
            StatusCode::BAD_REQUEST,
            "Name must be at most 50 characters.",
        ),
        (
            "password",
            json!("short"),
            StatusCode::BAD_REQUEST,
            "Password must be at least 8 characters.",
        ),
        (
            "password",
            json!("x".repeat(201)),
            StatusCode::BAD_REQUEST,
            "Password is too long.",
        ),
        (
            "acceptTerms",
            json!(false),
            StatusCode::BAD_REQUEST,
            "Please accept the Terms of Service and Privacy Policy.",
        ),
    ];
    for (field, value, status, message) in cases {
        let original = body[field].clone();
        body[field] = value;
        b.post("/auth/signup", body.clone())
            .await
            .err(status, message);
        body[field] = original;
    }
    b.signup("ana").await;
    let mut other = app.browser_at([198, 51, 100, 2]);
    other
        .post("/auth/signup", signup_body("ANA"))
        .await
        .err(StatusCode::CONFLICT, "That handle is taken.");
}

#[sqlx::test(migrations = "./migrations")]
async fn closed_instance_refuses_signups_once_an_admin_exists(db: PgPool) {
    let app = app(db.clone());
    let mut ana = app.browser();
    ana.signup("ana").await;
    sqlx::query("insert into instance_settings (key, value) values ('signups', '\"closed\"')")
        .execute(&db)
        .await
        .expect("setting");
    let instance = ana.get("/instance").await.ok();
    assert_eq!(instance["signups"], "closed");
    assert_eq!(instance["registrationsOpen"], false);
    let mut ben = app.browser_at([198, 51, 100, 2]);
    ben.post("/auth/signup", signup_body("ben")).await.err(
        StatusCode::FORBIDDEN,
        "Sign-ups are closed on this instance right now.",
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn login_logout_and_wrong_password(db: PgPool) {
    let app = app(db);
    app.browser().signup("ana").await;

    let mut b = app.browser_at([198, 51, 100, 5]);
    b.post(
        "/auth/login",
        json!({ "handle": "ana", "password": "wrong wrong" }),
    )
    .await
    .err(StatusCode::UNAUTHORIZED, "Wrong handle or password.");
    b.post(
        "/auth/login",
        json!({ "handle": "nobody", "password": PASSWORD }),
    )
    .await
    .err(StatusCode::UNAUTHORIZED, "Wrong handle or password.");
    let me = b
        .post(
            "/auth/login",
            json!({ "handle": "@Ana", "password": PASSWORD }),
        )
        .await
        .ok();
    assert_eq!(me["user"]["user"]["handle"], "ana");
    assert_eq!(b.get("/me").await.ok()["user"]["role"], "admin");

    let out = b.post_empty("/auth/logout").await.ok();
    assert_eq!(out["switchedTo"], serde_json::Value::Null);
    assert_eq!(b.get("/me").await.ok(), json!({ "user": null }));
    assert!(b.cookie("tuita_session").is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn suspended_accounts_cannot_sign_in(db: PgPool) {
    let app = app(db.clone());
    app.browser().signup("ana").await;
    sqlx::query("update users set suspended_at = now() where handle = 'ana'")
        .execute(&db)
        .await
        .expect("suspend");
    let mut b = app.browser_at([198, 51, 100, 5]);
    b.post("/auth/login", json!({ "handle": "ana", "password": PASSWORD }))
        .await
        .err(
            StatusCode::FORBIDDEN,
            "This account has been suspended. Contact the instance admin if you think this is a mistake.",
        );
}

#[sqlx::test(migrations = "./migrations")]
async fn the_21st_auth_try_in_ten_minutes_is_429(db: PgPool) {
    let app = app(db);
    app.browser().signup("ana").await;
    let mut b = app.browser_at([203, 0, 113, 7]);
    for n in 1..=20 {
        let reply = b
            .post(
                "/auth/login",
                json!({ "handle": "ana", "password": "wrong wrong" }),
            )
            .await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "try {n}");
    }
    let reply = b
        .post(
            "/auth/login",
            json!({ "handle": "ana", "password": PASSWORD }),
        )
        .await;
    reply.err(
        StatusCode::TOO_MANY_REQUESTS,
        "Too many requests. Try again in a few minutes.",
    );
    let retry: u32 = reply
        .header("retry-after")
        .parse()
        .expect("Retry-After seconds");
    assert!((1..=31).contains(&retry), "{retry}");

    let mut other = app.browser_at([203, 0, 113, 8]);
    other
        .post(
            "/auth/login",
            json!({ "handle": "ana", "password": PASSWORD }),
        )
        .await
        .ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn client_ip_comes_from_the_last_forwarded_hop_behind_a_proxy(db: PgPool) {
    let app = app_with(db, |var| (var == "TRUST_PROXY").then(|| "true".to_owned()));
    app.browser().signup("ana").await;
    let wrong = json!({ "handle": "ana", "password": "wrong wrong" });
    // Same peer address, different last hops: separate limiter keys.
    for hop in 1..=25 {
        let mut b = app.browser();
        b.headers = vec![(
            "x-forwarded-for".to_owned(),
            format!("10.0.0.1, 192.0.2.{hop}"),
        )];
        let reply = b.post("/auth/login", wrong.clone()).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "hop {hop}");
    }
    // One last hop used 21 times is limited, whatever comes before it.
    let mut spammer = app.browser();
    for n in 1..=20 {
        spammer.headers = vec![(
            "x-forwarded-for".to_owned(),
            format!("10.0.0.{n}, 192.0.2.200"),
        )];
        assert_eq!(
            spammer.post("/auth/login", wrong.clone()).await.status,
            StatusCode::UNAUTHORIZED
        );
    }
    spammer.headers = vec![(
        "x-forwarded-for".to_owned(),
        "10.9.9.9, 192.0.2.200".to_owned(),
    )];
    assert_eq!(
        spammer.post("/auth/login", wrong).await.status,
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn cross_origin_cookie_post_is_403(db: PgPool) {
    let app = app(db);
    let mut ana = app.browser();
    ana.signup("ana").await;

    ana.origin = Some("https://evil.example".to_owned());
    ana.post("/posts", json!({ "text": "hi" }))
        .await
        .err(StatusCode::FORBIDDEN, "bad origin");
    ana.origin = None;
    ana.post("/posts", json!({ "text": "hi" }))
        .await
        .err(StatusCode::FORBIDDEN, "bad origin");
    ana.origin = Some("https://social.example".to_owned());
    assert_ne!(
        ana.post("/auth/logout", json!({})).await.status,
        StatusCode::FORBIDDEN
    );

    // Reads and requests without a session cookie are not guarded.
    ana.origin = Some("https://evil.example".to_owned());
    assert_eq!(ana.get("/me").await.status, StatusCode::OK);
    let mut anonymous = app.browser_at([198, 51, 100, 9]);
    anonymous.origin = Some("https://evil.example".to_owned());
    anonymous
        .post(
            "/auth/login",
            json!({ "handle": "ana", "password": PASSWORD }),
        )
        .await
        .ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn account_switcher_holds_five_accounts_and_rejects_a_sixth(db: PgPool) {
    let app = app(db);
    let mut b = app.browser();
    let handles = ["ana", "ben", "cy", "dee", "eli"];
    for handle in handles {
        let handle = format!("{handle}_sw");
        b.signup_as(&handle, true).await;
    }
    let accounts = b.get("/accounts").await.ok();
    assert_eq!(accounts["max"], 5);
    let list = accounts["accounts"].as_array().expect("list");
    assert_eq!(list.len(), 5);
    assert_eq!(list[0]["user"]["handle"], "eli_sw");
    assert_eq!(list[0]["active"], true);
    assert_eq!(list.iter().filter(|a| a["active"] == true).count(), 1);

    b.post(
        "/auth/signup",
        json!({ "handle": "fay_sw", "displayName": "Fay", "password": PASSWORD, "acceptTerms": true, "add": true }),
    )
    .await
    .err(StatusCode::BAD_REQUEST, "You can be signed in to up to 5 accounts at once.");
    b.post(
        "/auth/login",
        json!({ "handle": "ana_sw", "password": PASSWORD, "add": true }),
    )
    .await
    .ok();
    assert_eq!(
        b.get("/accounts").await.ok()["accounts"]
            .as_array()
            .expect("list")
            .len(),
        5
    );

    let ben = b.get("/accounts").await.ok()["accounts"]
        .as_array()
        .expect("list")
        .iter()
        .find(|a| a["user"]["handle"] == "ben_sw")
        .expect("ben")["user"]["id"]
        .clone();
    let switched = b.post("/accounts/switch", json!({ "id": ben })).await.ok();
    assert_eq!(switched["user"]["user"]["handle"], "ben_sw");
    assert_eq!(b.get("/me").await.ok()["user"]["user"]["handle"], "ben_sw");

    b.post("/accounts/switch", json!({ "id": uuid::Uuid::now_v7() }))
        .await
        .err(
            StatusCode::NOT_FOUND,
            "That account isn\u{2019}t signed in on this browser any more.",
        );

    let out = b.post("/auth/logout", json!({})).await.ok();
    assert!(out["switchedTo"]["user"]["handle"].is_string());
    assert_eq!(
        b.get("/accounts").await.ok()["accounts"]
            .as_array()
            .expect("list")
            .len(),
        4
    );
    let out = b.post("/auth/logout", json!({ "all": true })).await.ok();
    assert_eq!(out["switchedTo"], serde_json::Value::Null);
    assert_eq!(
        b.get("/accounts").await.ok()["accounts"]
            .as_array()
            .expect("list")
            .len(),
        0
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn patch_me_updates_settings_and_validates(db: PgPool) {
    common::block_words(&db, "gizmo").await;
    let app = app(db);
    let mut ana = app.browser();
    ana.signup("ana").await;

    let me = ana
        .patch(
            "/me",
            json!({
                "displayName": "  Ana M  ",
                "bio": "Makes buttons squish",
                "website": "example.org",
                "avatarShape": "octagon",
                "ink": "phosphor",
                "flags": ["rainbow"],
                "dmPolicy": "following",
                "discoverable": false,
                "language": "de",
                "mutedWords": ["Spoilers", " spoilers ", "x"],
                "notifPrefs": { "like": false, "repost": true, "follow": true, "reply": true, "mention": true, "quote": true },
            }),
        )
        .await
        .ok();
    assert_eq!(me["user"]["displayName"], "Ana M");
    assert_eq!(me["user"]["website"], "https://example.org");
    assert_eq!(me["user"]["avatarShape"], "octagon");
    assert_eq!(me["user"]["ink"], "phosphor");
    assert!(
        me["user"].get("flags").is_none(),
        "unknown field is ignored"
    );
    assert_eq!(me["settings"]["dmPolicy"], "following");
    assert_eq!(me["settings"]["discoverable"], false);
    assert_eq!(me["settings"]["language"], "de");
    assert_eq!(me["settings"]["mutedWords"], json!(["spoilers", "x"]));
    assert_eq!(me["settings"]["notifPrefs"]["like"], false);

    let me = ana.patch("/me", json!({ "language": "" })).await.ok();
    assert_eq!(me["settings"]["language"], serde_json::Value::Null);
    assert_eq!(me["user"]["displayName"], "Ana M");

    let cases: [(serde_json::Value, &str); 7] = [
        (json!({ "displayName": " " }), "Name is required."),
        (
            json!({ "displayName": "x".repeat(51) }),
            "Name must be at most 50 characters.",
        ),
        (
            json!({ "bio": "x".repeat(161) }),
            "Bio must be at most 160 characters.",
        ),
        (
            json!({ "location": "x".repeat(31) }),
            "Location must be at most 30 characters.",
        ),
        (
            json!({ "website": "not a url" }),
            "Website must be a valid URL.",
        ),
        (json!({ "language": "xx" }), "Unknown language."),
        (
            json!({ "mutedWords": ["x".repeat(41)] }),
            "Muted words can be at most 40 characters.",
        ),
    ];
    for (body, message) in cases {
        ana.patch("/me", body)
            .await
            .err(StatusCode::BAD_REQUEST, message);
    }
    ana.patch("/me", json!({ "displayName": "g1zm0" }))
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "That display name isn\u{2019}t allowed. Please change it.",
        );
    ana.patch("/me", json!({ "avatarShape": "hexagon" }))
        .await
        .err(StatusCode::BAD_REQUEST, "Invalid JSON.");
    let words: Vec<String> = (0..101).map(|n| format!("w{n}")).collect();
    ana.patch("/me", json!({ "mutedWords": words }))
        .await
        .err(StatusCode::BAD_REQUEST, "You can mute up to 100 words.");

    let mut anonymous = app.browser_at([198, 51, 100, 9]);
    anonymous
        .patch("/me", json!({ "bio": "x" }))
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");
}

#[sqlx::test(migrations = "./migrations")]
async fn password_change_ends_other_sessions(db: PgPool) {
    let app = app(db);
    let mut phone = app.browser();
    phone.signup("ana").await;
    let mut laptop = app.browser_at([198, 51, 100, 5]);
    laptop
        .post(
            "/auth/login",
            json!({ "handle": "ana", "password": PASSWORD }),
        )
        .await
        .ok();
    assert_eq!(
        laptop
            .get("/me/sessions")
            .await
            .ok()
            .as_array()
            .expect("list")
            .len(),
        2
    );

    phone
        .post(
            "/me/password",
            json!({ "current": "wrong wrong", "next": "a brand new pass" }),
        )
        .await
        .err(StatusCode::FORBIDDEN, "Your current password is wrong.");
    phone
        .post(
            "/me/password",
            json!({ "current": PASSWORD, "next": "short" }),
        )
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "Password must be at least 8 characters.",
        );
    phone
        .post(
            "/me/password",
            json!({ "current": PASSWORD, "next": "a brand new pass" }),
        )
        .await
        .ok();

    assert_eq!(laptop.get("/me").await.ok(), json!({ "user": null }));
    assert_eq!(phone.get("/me").await.ok()["user"]["user"]["handle"], "ana");
    let mut fresh = app.browser_at([198, 51, 100, 6]);
    fresh
        .post(
            "/auth/login",
            json!({ "handle": "ana", "password": "a brand new pass" }),
        )
        .await
        .ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn sessions_list_and_revoke(db: PgPool) {
    let app = app(db);
    let mut phone = app.browser();
    phone.signup("ana").await;
    phone.user_agent = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0)".to_owned();
    let mut laptop = app.browser_at([198, 51, 100, 5]);
    laptop
        .post(
            "/auth/login",
            json!({ "handle": "ana", "password": PASSWORD }),
        )
        .await
        .ok();

    let list = laptop.get("/me/sessions").await.ok();
    let list = list.as_array().expect("list");
    assert_eq!(list.len(), 2);
    assert_eq!(list[0]["current"], true);
    assert_eq!(list[0]["ip"], "198.51.100.5");
    assert_eq!(list[0]["client"], "linux");
    assert_eq!(list[1]["current"], false);
    assert_eq!(list[1]["ip"], "198.51.100.1");
    let current = list[0]["id"].as_str().expect("id").to_owned();
    let other = list[1]["id"].as_str().expect("id").to_owned();

    laptop.delete(&format!("/me/sessions/{current}")).await.err(
        StatusCode::BAD_REQUEST,
        "Use \u{201c}Log out\u{201d} to end this session.",
    );
    laptop
        .delete(&format!("/me/sessions/{}", uuid::Uuid::now_v7()))
        .await
        .err(StatusCode::NOT_FOUND, "Session not found.");
    laptop
        .delete("/me/sessions/not-a-uuid")
        .await
        .err(StatusCode::NOT_FOUND, "Session not found.");
    laptop.delete(&format!("/me/sessions/{other}")).await.ok();
    assert_eq!(phone.get("/me").await.ok(), json!({ "user": null }));

    let mut third = app.browser_at([198, 51, 100, 6]);
    third
        .post(
            "/auth/login",
            json!({ "handle": "ana", "password": PASSWORD }),
        )
        .await
        .ok();
    assert_eq!(
        laptop.delete("/me/sessions").await.ok(),
        json!({ "revoked": 1 })
    );
    assert_eq!(
        laptop
            .get("/me/sessions")
            .await
            .ok()
            .as_array()
            .expect("list")
            .len(),
        1
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn instance_is_public(db: PgPool) {
    let app = app(db);
    let mut anonymous = app.browser();
    let before = anonymous.get("/instance").await.ok();
    assert_eq!(before["name"], "tuita");
    assert_eq!(before["domain"], "social.example");
    assert_eq!(before["signups"], "open");
    assert_eq!(before["registrationsOpen"], true);
    assert_eq!(before["sourceUrl"], serde_json::Value::Null);
    assert_eq!(before["rules"].as_array().expect("rules").len(), 5);
    assert_eq!(before["minAge"], 13);
    assert_eq!(before["maxVideoMb"], 50);
    assert_eq!(before["stats"]["users"], 0);
    assert_eq!(before["stats"]["posts"], 0);
    assert_eq!(before["stats"]["user_count"], 0);
    assert_eq!(before["stats"]["status_count"], 0);
    assert_eq!(before["stats"]["domain_count"], 0);
    anonymous.signup("ana").await;
    assert_eq!(anonymous.get("/instance").await.ok()["stats"]["users"], 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn instance_reports_the_source_url(db: PgPool) {
    let app = app_with(db, |var| {
        (var == "SOURCE_URL").then(|| "https://git.example/tuita".to_owned())
    });
    let reply = app.browser().get("/instance").await.ok();
    assert_eq!(reply["sourceUrl"], "https://git.example/tuita");
}

#[sqlx::test(migrations = "./migrations")]
async fn api_errors_are_json(db: PgPool) {
    let app = app(db);
    let mut b = app.browser();
    b.get("/nope").await.err(StatusCode::NOT_FOUND, "not found");
    b.delete("/auth/login")
        .await
        .err(StatusCode::METHOD_NOT_ALLOWED, "method not allowed");
    b.get("/me/sessions")
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");
    b.post("/auth/login", json!("not an object"))
        .await
        .err(StatusCode::BAD_REQUEST, "Invalid JSON.");
}
