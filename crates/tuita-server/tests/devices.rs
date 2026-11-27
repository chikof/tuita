//! Device labels on sessions, and organization affiliations.

mod common;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn sessions_carry_the_country_and_the_client_the_request_declared(db: PgPool) {
    let app = common::app_with(db.clone(), |var| {
        (var == "TRUST_PROXY").then(|| "true".to_owned())
    });
    let mut ana = app.browser();
    ana.headers
        .push(("cf-ipcountry".to_owned(), "pl".to_owned()));
    ana.headers
        .push(("tuita-client".to_owned(), "android".to_owned()));
    ana.signup("ana").await;
    let mut other = app.browser_at([198, 51, 100, 9]);
    other
        .headers
        .push(("x-forwarded-for".to_owned(), "203.0.113.5".to_owned()));
    other.user_agent = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0)".to_owned();
    other
        .post(
            "/auth/login",
            json!({ "handle": "ana", "password": common::PASSWORD, "add": false }),
        )
        .await
        .ok();
    let sessions = ana.get("/me/sessions").await.ok();
    let rows = sessions.as_array().expect("sessions");
    assert_eq!(rows.len(), 2);
    let android = rows
        .iter()
        .find(|s| s["client"] == "android-app")
        .expect("android session");
    assert_eq!(android["country"], "PL");
    assert_eq!(android["current"], true);
    assert!(
        rows.iter()
            .any(|s| s["client"] == "iphone" && s["country"].is_null())
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn an_organization_invites_and_a_member_accepts(db: PgPool) {
    let app = common::app(db.clone());
    let mut org = app.person(&db, "acme").await;
    let mut ada = app.person(&db, "ada").await;
    let mut bea = app.person(&db, "bea").await;
    org.browser.post("/affiliations", json!({ "handle": "ada" })).await
        .err(StatusCode::FORBIDDEN, "Only organization accounts can add affiliates. Ask an admin to mark your account as an organization.");
    sqlx::query("update users set is_org = true where id = $1")
        .bind(org.id)
        .execute(&db)
        .await
        .expect("org");
    org.browser
        .post("/affiliations", json!({ "handle": "acme" }))
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "An organization can\u{2019}t be affiliated with itself.",
        );
    org.browser
        .post("/affiliations", json!({ "handle": "ghost" }))
        .await
        .err(
            StatusCode::NOT_FOUND,
            "No account with that handle on this instance.",
        );
    org.browser
        .post(
            "/affiliations",
            json!({ "handle": "@ada", "label": "Engineer" }),
        )
        .await
        .ok();
    org.browser
        .post("/affiliations", json!({ "handle": "ada" }))
        .await
        .err(
            StatusCode::CONFLICT,
            "They already have an invitation from you.",
        );
    org.browser
        .post(
            "/affiliations",
            json!({ "handle": "bea", "label": "x".repeat(31) }),
        )
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "Label must be at most 30 characters.",
        );

    let note = ada.browser.get("/notifications").await.ok();
    assert_eq!(note["items"][0]["kind"], "affiliation");
    let mine = ada.browser.get("/affiliations").await.ok();
    assert_eq!(mine["invites"][0]["user"]["handle"], "acme");
    assert_eq!(mine["current"], serde_json::Value::Null);
    let id = mine["invites"][0]["id"].as_str().expect("id").to_owned();
    bea.browser
        .post(&format!("/affiliations/{id}/accept"), json!({}))
        .await
        .err(StatusCode::NOT_FOUND, "Invitation not found.");
    ada.browser
        .post(&format!("/affiliations/{id}/accept"), json!({}))
        .await
        .ok();

    let profile = bea.browser.get("/users/ada").await.ok();
    assert_eq!(profile["affiliation"]["handle"], "acme");
    assert_eq!(profile["affiliation"]["label"], "Engineer");
    let list = bea.browser.get("/orgs/acme/affiliates").await.ok();
    assert_eq!(list["items"][0]["user"]["handle"], "ada");
    assert_eq!(list["items"][0]["label"], "Engineer");
    org.browser
        .post("/affiliations", json!({ "handle": "ada" }))
        .await
        .err(
            StatusCode::CONFLICT,
            "They\u{2019}re already affiliated with you.",
        );

    org.browser
        .patch(
            &format!("/affiliations/{id}"),
            json!({ "label": "Designer" }),
        )
        .await
        .ok();
    assert_eq!(
        bea.browser.get("/users/ada").await.ok()["affiliation"]["label"],
        "Designer"
    );
    ada.browser
        .patch(&format!("/affiliations/{id}"), json!({ "label": "x" }))
        .await
        .err(StatusCode::NOT_FOUND, "Affiliation not found.");
    ada.browser
        .delete(&format!("/affiliations/{id}"))
        .await
        .ok();
    assert_eq!(
        bea.browser.get("/users/ada").await.ok()["affiliation"],
        serde_json::Value::Null
    );
    ada.browser
        .delete(&format!("/affiliations/{id}"))
        .await
        .err(StatusCode::NOT_FOUND, "Affiliation not found.");
}

#[sqlx::test(migrations = "./migrations")]
async fn two_invitations_accepted_at_once_leave_one_affiliation_and_no_error(db: PgPool) {
    let app = common::app(db.clone());
    let mut ada = app.person(&db, "ada").await;
    let mut again = app.browser();
    again.sign_in_as(&db, ada.id).await;
    for handle in ["acme", "globex"] {
        let mut org = app.person(&db, handle).await;
        sqlx::query("update users set is_org = true where id = $1")
            .bind(org.id)
            .execute(&db)
            .await
            .expect("org");
        org.browser
            .post("/affiliations", json!({ "handle": "ada" }))
            .await
            .ok();
    }
    let invites = ada.browser.get("/affiliations").await.ok()["invites"].clone();
    let first = invites[0]["id"].as_str().expect("id").to_owned();
    let second = invites[1]["id"].as_str().expect("id").to_owned();
    let (path_a, path_b) = (
        format!("/affiliations/{first}/accept"),
        format!("/affiliations/{second}/accept"),
    );
    let (a, b) = tokio::join!(
        ada.browser.post(&path_a, json!({})),
        again.post(&path_b, json!({}))
    );
    assert_eq!(
        (a.status, b.status),
        (StatusCode::OK, StatusCode::OK),
        "{} {}",
        a.body,
        b.body
    );
    let active: i64 = sqlx::query_scalar(
        "select count(*) from affiliations where user_id = $1 and status = 'active'",
    )
    .bind(ada.id)
    .fetch_one(&db)
    .await
    .expect("count");
    assert_eq!(active, 1);
}
