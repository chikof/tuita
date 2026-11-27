//! Passkeys: the whole ceremony with a software authenticator, and the limits around it.

mod common;

use axum::http::StatusCode;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde_json::{Value, json};
use sqlx::PgPool;
use webauthn_authenticator_rs::WebauthnAuthenticator as _;
use webauthn_authenticator_rs::prelude::{
    CreationChallengeResponse, RequestChallengeResponse, Url,
};
use webauthn_authenticator_rs::softpasskey::SoftPasskey;

use common::Browser;

fn origin() -> Url {
    Url::parse(common::ORIGIN).expect("origin")
}

/// The browser's part of registration. The software token cannot store resident keys, so the
/// requirement the server sent (asserted by the callers) is dropped before it signs.
fn ceremony_create(authenticator: &mut SoftPasskey, options: &Value) -> Value {
    let mut public_key = options["publicKey"].clone();
    if let Some(selection) = public_key["authenticatorSelection"].as_object_mut() {
        selection.remove("residentKey");
        selection.insert("requireResidentKey".to_owned(), json!(false));
    }
    let ccr: CreationChallengeResponse =
        serde_json::from_value(json!({ "publicKey": public_key })).expect("options");
    let credential = authenticator
        .do_registration(origin(), ccr)
        .expect("software token signs");
    serde_json::to_value(credential).expect("json")
}

/// The browser's part of sign-in. A discoverable credential returns the user handle with the
/// assertion; the software token does not, so it is added here (the signature does not cover it).
async fn ceremony_get(
    authenticator: &mut SoftPasskey,
    db: &PgPool,
    options: &Value,
    handle: &str,
) -> Value {
    // A resident-key authenticator finds its credential itself; the software token is told which.
    let mut public_key = options["publicKey"].clone();
    let cred_id: Vec<u8> = sqlx::query_scalar("select cred_id from passkeys limit 1")
        .fetch_one(db)
        .await
        .expect("a stored passkey");
    public_key["allowCredentials"] =
        json!([{ "type": "public-key", "id": URL_SAFE_NO_PAD.encode(cred_id) }]);
    let rcr: RequestChallengeResponse =
        serde_json::from_value(json!({ "publicKey": public_key })).expect("options");
    let mut assertion = serde_json::to_value(
        authenticator
            .do_authentication(origin(), rcr)
            .expect("assertion"),
    )
    .expect("json");
    let webauthn_id: uuid::Uuid =
        sqlx::query_scalar("select webauthn_id from users where handle = $1")
            .bind(handle)
            .fetch_one(db)
            .await
            .expect("webauthn id");
    assertion["response"]["userHandle"] = json!(URL_SAFE_NO_PAD.encode(webauthn_id.as_bytes()));
    assertion
}

/// Runs the registration ceremony for the signed-in `browser`; returns the stored passkey.
async fn register(browser: &mut Browser, authenticator: &mut SoftPasskey, name: &str) -> Value {
    let options = browser.post_empty("/passkeys/register/options").await.ok();
    assert_eq!(
        options["publicKey"]["authenticatorSelection"]["residentKey"],
        "required"
    );
    let credential = ceremony_create(authenticator, &options);
    browser
        .post(
            "/passkeys/register",
            json!({ "challengeId": options["challengeId"], "credential": credential, "name": name }),
        )
        .await
        .ok()
}

#[sqlx::test(migrations = "./migrations")]
async fn register_then_sign_in_with_a_discoverable_passkey(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut authenticator = SoftPasskey::new(true);
    let stored = register(&mut ana.browser, &mut authenticator, "Laptop").await;
    assert_eq!(stored["name"], "Laptop");
    let listed = ana.browser.get("/passkeys").await.ok();
    assert_eq!(listed["rpId"], "social.example");
    assert_eq!(listed["passkeys"][0]["id"], stored["id"]);

    // A fresh browser signs in with no handle, like the login field's conditional UI does.
    let mut visitor = app.browser_at([198, 51, 100, 7]);
    let options = visitor.post_empty("/auth/passkey/options").await.ok();
    let assertion = ceremony_get(&mut authenticator, &db, &options, "ana").await;
    let signed_in = visitor
        .post(
            "/auth/passkey",
            json!({ "challengeId": options["challengeId"], "credential": assertion, "add": false }),
        )
        .await
        .ok();
    assert_eq!(signed_in["user"]["user"]["handle"], "ana");
    assert!(
        visitor.cookie("tuita_session").is_some(),
        "a session like password login"
    );
    assert_eq!(
        visitor.get("/me").await.ok()["user"]["user"]["handle"],
        "ana"
    );
    let used: Option<time::OffsetDateTime> =
        sqlx::query_scalar("select last_used_at from passkeys")
            .fetch_one(&db)
            .await
            .expect("row");
    assert!(used.is_some());

    // The challenge worked once.
    visitor
        .post(
            "/auth/passkey",
            json!({ "challengeId": options["challengeId"], "credential": assertion, "add": false }),
        )
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "This passkey request expired. Please try again.",
        );

    // Rename, delete, and sign-in stops working with the passkey gone.
    let renamed = ana
        .browser
        .patch(
            &format!("/passkeys/{}", stored["id"].as_str().expect("id")),
            json!({ "name": "Phone" }),
        )
        .await
        .ok();
    assert_eq!(renamed["name"], "Phone");
    ana.browser
        .patch(
            &format!("/passkeys/{}", stored["id"].as_str().expect("id")),
            json!({ "name": "x".repeat(61) }),
        )
        .await
        .err(
            StatusCode::BAD_REQUEST,
            "Name must be at most 60 characters.",
        );
    ana.browser
        .delete(&format!("/passkeys/{}", stored["id"].as_str().expect("id")))
        .await
        .ok();
    let options = visitor.post_empty("/auth/passkey/options").await.ok();
    visitor
        .post(
            "/auth/passkey",
            json!({ "challengeId": options["challengeId"], "credential": assertion }),
        )
        .await
        .err(
            StatusCode::UNAUTHORIZED,
            "This passkey isn\u{2019}t registered here. It may have been removed.",
        );
}

#[sqlx::test(migrations = "./migrations")]
async fn challenges_expire_work_once_and_belong_to_their_user(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    let mut authenticator = SoftPasskey::new(true);
    let options = ana
        .browser
        .post_empty("/passkeys/register/options")
        .await
        .ok();
    let credential = ceremony_create(&mut authenticator, &options);
    let body = json!({ "challengeId": options["challengeId"], "credential": credential });
    let expired = "This passkey request expired. Please try again.";
    bob.browser
        .post("/passkeys/register", body.clone())
        .await
        .err(StatusCode::BAD_REQUEST, expired);

    // Expired: five minutes are up.
    let id = options["challengeId"].as_str().expect("id");
    sqlx::query("update webauthn_challenges set expires_at = now() - interval '1 second' where id = $1::uuid").bind(id).execute(&db).await.expect("expire");
    ana.browser
        .post("/passkeys/register", body.clone())
        .await
        .err(StatusCode::BAD_REQUEST, expired);

    // Used up: a failed attempt burns the challenge too.
    let options = ana
        .browser
        .post_empty("/passkeys/register/options")
        .await
        .ok();
    let junk = json!({ "challengeId": options["challengeId"], "credential": { "id": "x" } });
    ana.browser.post("/passkeys/register", junk).await.err(
        StatusCode::BAD_REQUEST,
        "That passkey could not be verified.",
    );
    let again = json!({ "challengeId": options["challengeId"], "credential": credential });
    ana.browser
        .post("/passkeys/register", again)
        .await
        .err(StatusCode::BAD_REQUEST, expired);
    ana.browser
        .post(
            "/passkeys/register",
            json!({ "challengeId": uuid::Uuid::nil(), "credential": {} }),
        )
        .await
        .err(StatusCode::BAD_REQUEST, expired);
}

#[sqlx::test(migrations = "./migrations")]
async fn an_account_holds_at_most_twenty_passkeys_and_tokens_cannot_touch_them(db: PgPool) {
    let app = common::app(db.clone());
    let mut ana = app.person(&db, "ana").await;
    for n in 0..20_u8 {
        sqlx::query("insert into passkeys (id, user_id, cred_id, passkey, name) values ($1, $2, $3, '{}', 'k')")
            .bind(uuid::Uuid::now_v7())
            .bind(ana.id)
            .bind(vec![n; 16])
            .execute(&db)
            .await
            .expect("passkey");
    }
    ana.browser
        .post_empty("/passkeys/register/options")
        .await
        .err(StatusCode::BAD_REQUEST, "You can have up to 20 passkeys.");

    let created = ana
        .browser
        .post(
            "/tokens",
            json!({ "name": "t", "scopes": ["read", "write"] }),
        )
        .await
        .ok();
    let mut client = app.browser();
    client.origin = None;
    client.headers.push((
        "authorization".to_owned(),
        format!("Bearer {}", created["token"].as_str().expect("token")),
    ));
    let refused = "API tokens can\u{2019}t be used for this. Sign in on the website instead.";
    client
        .get("/passkeys")
        .await
        .err(StatusCode::FORBIDDEN, refused);
    client
        .post_empty("/passkeys/register/options")
        .await
        .err(StatusCode::FORBIDDEN, refused);
    client
        .delete("/passkeys/00000000-0000-0000-0000-000000000000")
        .await
        .err(StatusCode::FORBIDDEN, refused);
    let mut nobody = app.browser();
    nobody
        .get("/passkeys")
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");
    ana.browser
        .delete("/passkeys/00000000-0000-0000-0000-000000000000")
        .await
        .err(StatusCode::NOT_FOUND, "Passkey not found.");
}

#[sqlx::test(migrations = "./migrations")]
async fn an_ip_address_has_no_relying_party(db: PgPool) {
    let config = tuita_server::Config::from_lookup(|var| match var {
        "DATABASE_URL" => Some("postgres://unused".to_owned()),
        "PUBLIC_URL" => Some("http://127.0.0.1:8080".to_owned()),
        _ => None,
    })
    .expect("config");
    let media = tuita_server::media::MediaStore::from_config(&config).expect("media store");
    let state = tuita_server::AppState::new(db.clone(), config, media);
    assert_eq!(state.rp_id(), None);
    let router = tuita_server::router(state.clone(), common::stub_web());
    let mut ana = common::App { state, router }.browser();
    ana.origin = Some("http://127.0.0.1:8080".to_owned());
    let id = common::insert_user(&db, "ana").await;
    ana.sign_in_as(&db, id).await;
    assert_eq!(ana.get("/passkeys").await.ok()["rpId"], Value::Null);
    ana.post_empty("/passkeys/register/options").await.err(
        StatusCode::BAD_REQUEST,
        "Passkeys need a site address with a domain name, not an IP address.",
    );
}
