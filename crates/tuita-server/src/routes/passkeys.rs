//! Passkeys (WebAuthn): registration and management for the signed-in account, and the
//! discoverable sign-in. `webauthn-rs` does the
//! cryptography. Every route except sign-in is cookie-only.

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse as _, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};
use webauthn_rs::prelude::{
    DiscoverableAuthentication, PasskeyRegistration, PublicKeyCredential,
    RegisterPublicKeyCredential,
};

use super::auth::text_field;
use crate::auth::{self, ApiJson, CurrentUser, Tokens};
use crate::cursor::parse_id;
use crate::db::passkeys::{self, EXPIRED, MAX_PASSKEYS};
use crate::db::sessions::ClientMeta;
use crate::db::users::me_dto;
use crate::dto::{
    AuthResponse, OkDto, PasskeyDto, PasskeyLoginRequest, PasskeyOptionsDto,
    PasskeyRegisterRequest, PasskeysDto, RenamePasskeyRequest,
};
use crate::error::AppError;
use crate::state::AppState;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/passkeys", get(list))
        .route("/passkeys/register/options", post(register_options))
        .route("/passkeys/register", post(register))
        .route(
            "/passkeys/{id}",
            axum::routing::patch(rename).delete(remove),
        )
        .route("/auth/passkey/options", post(login_options))
        .route("/auth/passkey", post(login))
}

const UNVERIFIED: &str = "That passkey could not be verified.";
const SUSPENDED: &str =
    "This account has been suspended. Contact the instance admin if you think this is a mistake.";

fn internal(err: impl std::error::Error + Send + Sync + 'static) -> AppError {
    AppError::Internal(err.into())
}

async fn list(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
) -> Result<Json<PasskeysDto>, AppError> {
    auth.session_id()?;
    Ok(Json(PasskeysDto {
        passkeys: passkeys::list(state.db(), auth.user.id).await?,
        rp_id: state.rp_id().map(str::to_owned),
    }))
}

async fn register_options(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
) -> Result<Json<PasskeyOptionsDto>, AppError> {
    auth.session_id()?;
    state.limits().check_passkey_register(auth.user.id)?;
    let relying_party = state.passkeys()?;
    let db = state.db();
    let existing = passkeys::credential_ids(db, auth.user.id).await?;
    if i64::try_from(existing.len()).unwrap_or(i64::MAX) >= MAX_PASSKEYS {
        return Err(AppError::bad_request("You can have up to 20 passkeys."));
    }
    let webauthn_id = passkeys::webauthn_id(db, auth.user.id).await?;
    let exclude = (!existing.is_empty()).then_some(existing);
    let (challenge, registration) = relying_party
        .webauthn()
        .start_passkey_registration(
            webauthn_id,
            &auth.user.handle,
            &auth.user.display_name,
            exclude,
        )
        .map_err(internal)?;
    let mut options = serde_json::to_value(&challenge.public_key).map_err(internal)?;
    // Passkeys sign people in without a handle, so the credential must be discoverable.
    if let Some(selection) = options
        .get_mut("authenticatorSelection")
        .and_then(Value::as_object_mut)
    {
        selection.insert("residentKey".to_owned(), json!("required"));
        selection.insert("requireResidentKey".to_owned(), json!(true));
    }
    let state_json = serde_json::to_value(&registration).map_err(internal)?;
    let challenge_id = passkeys::save_challenge(db, Some(auth.user.id), &state_json).await?;
    Ok(Json(PasskeyOptionsDto {
        challenge_id,
        public_key: options,
    }))
}

async fn register(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiJson(body): ApiJson<PasskeyRegisterRequest>,
) -> Result<Json<PasskeyDto>, AppError> {
    auth.session_id()?;
    state.limits().check_passkey_register(auth.user.id)?;
    let relying_party = state.passkeys()?;
    let name = match body.name.as_deref().map(str::trim) {
        Some(name) if !name.is_empty() => text_field(name, "Name", 1, 60)?,
        _ => "Passkey".to_owned(),
    };
    let stored = passkeys::take_challenge(state.db(), body.challenge_id, Some(auth.user.id))
        .await?
        .ok_or_else(|| AppError::bad_request(EXPIRED))?;
    let registration: PasskeyRegistration = serde_json::from_value(stored).map_err(internal)?;
    let credential: RegisterPublicKeyCredential =
        serde_json::from_value(body.credential).map_err(|_| AppError::bad_request(UNVERIFIED))?;
    let passkey = relying_party
        .webauthn()
        .finish_passkey_registration(&credential, &registration)
        .map_err(|err| {
            tracing::debug!(error = %err, "passkey registration refused");
            AppError::bad_request(UNVERIFIED)
        })?;
    Ok(Json(
        passkeys::insert(state.db(), auth.user.id, &passkey, &name).await?,
    ))
}

async fn rename(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
    ApiJson(body): ApiJson<RenamePasskeyRequest>,
) -> Result<Json<PasskeyDto>, AppError> {
    auth.session_id()?;
    let id = parse_id(&id, "Passkey not found.")?;
    let name = text_field(&body.name, "Name", 1, 60)?;
    passkeys::rename(state.db(), auth.user.id, id, &name)
        .await?
        .map(Json)
        .ok_or(AppError::Missing("Passkey not found."))
}

async fn remove(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<OkDto>, AppError> {
    auth.session_id()?;
    let id = parse_id(&id, "Passkey not found.")?;
    if !passkeys::delete(state.db(), auth.user.id, id).await? {
        return Err(AppError::Missing("Passkey not found."));
    }
    Ok(Json(OkDto { ok: true }))
}

async fn login_options(
    State(state): State<AppState>,
    meta: ClientMeta,
) -> Result<Json<PasskeyOptionsDto>, AppError> {
    state
        .limits()
        .check_passkey_options(meta.ip.unwrap_or(std::net::Ipv4Addr::UNSPECIFIED.into()))?;
    let relying_party = state.passkeys()?;
    let (challenge, authentication) = relying_party
        .webauthn()
        .start_discoverable_authentication()
        .map_err(internal)?;
    let options = serde_json::to_value(&challenge.public_key).map_err(internal)?;
    let state_json = serde_json::to_value(&authentication).map_err(internal)?;
    let challenge_id = passkeys::save_challenge(state.db(), None, &state_json).await?;
    Ok(Json(PasskeyOptionsDto {
        challenge_id,
        public_key: options,
    }))
}

async fn login(
    State(state): State<AppState>,
    meta: ClientMeta,
    headers: HeaderMap,
    ApiJson(body): ApiJson<PasskeyLoginRequest>,
) -> Result<Response, AppError> {
    state
        .limits()
        .check_auth(meta.ip.unwrap_or(std::net::Ipv4Addr::UNSPECIFIED.into()))?;
    let relying_party = state.passkeys()?;
    let db = state.db();
    let stored = passkeys::take_challenge(db, body.challenge_id, None)
        .await?
        .ok_or_else(|| AppError::bad_request(EXPIRED))?;
    let authentication: DiscoverableAuthentication =
        serde_json::from_value(stored).map_err(internal)?;
    let credential: PublicKeyCredential =
        serde_json::from_value(body.credential).map_err(|_| AppError::bad_request(UNVERIFIED))?;
    let webauthn = relying_party.webauthn();
    let (webauthn_id, credential_id) = webauthn
        .identify_discoverable_authentication(&credential)
        .map_err(|_| AppError::bad_request(UNVERIFIED))?;
    let found = passkeys::find(db, credential_id, webauthn_id)
        .await?
        .ok_or(AppError::Denied(
            "This passkey isn\u{2019}t registered here. It may have been removed.",
        ))?;
    let mut passkey = found.passkey.0;
    let result = webauthn
        .finish_discoverable_authentication_v2(&credential, authentication, &passkey)
        .map_err(|err| {
            tracing::debug!(error = %err, "passkey sign-in refused");
            AppError::Denied("Passkey sign-in failed.")
        })?;
    if found.user.suspended_at.is_some() {
        return Err(AppError::forbidden(SUSPENDED));
    }
    passkey.update_credential(&result);
    passkeys::record_use(db, found.passkey_id, &passkey).await?;
    let tokens = Tokens::from_headers(&headers);
    let cookies =
        auth::start_session(db, state.config(), &tokens, found.user.id, body.add, &meta).await?;
    let reply = AuthResponse {
        user: me_dto(db, &found.user, state.config().domain()).await?,
    };
    Ok((cookies, Json(reply)).into_response())
}
