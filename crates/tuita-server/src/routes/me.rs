//! The signed-in account: profile and settings, password, login sessions.

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::http::header::CONTENT_DISPOSITION;
use axum::response::{IntoResponse as _, Response};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use serde_json::json;
use tuita_core::locale::LANGS;
use uuid::Uuid;

use super::auth::text_field;
use crate::auth::{self, ApiJson, CurrentUser, MaybeUser, Tokens, password};
use crate::cursor::parse_id;
use crate::db::accounts::{self, LanguageChange, ProfileChanges};
use crate::db::sessions::{self, SessionRow};
use crate::db::users::{UserRow, me_dto};
use crate::db::{deletion, export, settings, tokens};
use crate::dto::{
    DeleteAccountRequest, MeDto, MeResponse, NotifPrefsDto, OkDto, PasswordRequest,
    PatchBotRequest, PatchMeRequest, RevokedDto, SessionDto,
};
use crate::error::AppError;
use crate::filter::{ContentFilter, REASON};
use crate::state::AppState;

/// Most muted words.
const MAX_MUTED_WORDS: usize = 100;
/// Longest muted word, in characters.
const MAX_MUTED_WORD_LEN: usize = 40;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/me", get(me).patch(patch_me).delete(delete_me))
        .route("/me/bot", patch(patch_bot))
        .route("/me/export", get(export_data))
        .route("/me/password", post(change_password))
        .route("/me/sessions", get(list_sessions).delete(revoke_others))
        .route("/me/sessions/{id}", delete(revoke_one))
}

async fn me(
    State(state): State<AppState>,
    MaybeUser(auth): MaybeUser,
) -> Result<Json<MeResponse>, AppError> {
    let user = match auth {
        Some(auth) => Some(me_dto(state.db(), &auth.user, state.config().domain()).await?),
        None => None,
    };
    Ok(Json(MeResponse { user }))
}

async fn export_data(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
) -> Result<Response, AppError> {
    state.limits().check_export(auth.user.id)?;
    let data = export::export(state.db(), &auth.user, state.config().domain()).await?;
    let filename = format!(
        "tuita-{}-{}.json",
        auth.user.handle,
        data.exported_at.date()
    );
    Ok((
        [(
            CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )],
        Json(data),
    )
        .into_response())
}

async fn delete_me(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    headers: HeaderMap,
    ApiJson(body): ApiJson<DeleteAccountRequest>,
) -> Result<Response, AppError> {
    if !password::verify(body.password, auth.user.password_hash.clone()).await {
        return Err(AppError::forbidden("Wrong password."));
    }
    deletion::delete_local_account(state.db(), state.config(), &auth.user).await?;
    // The account's sessions are gone; this drops their cookies and keeps other signed-in accounts.
    let tokens = Tokens::from_headers(&headers);
    let (_, cookies) = auth::end_session(state.db(), state.config(), &tokens, false).await?;
    Ok((cookies, Json(OkDto { ok: true })).into_response())
}

async fn patch_me(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiJson(body): ApiJson<PatchMeRequest>,
) -> Result<Json<MeDto>, AppError> {
    let filter = settings::content_filter(state.db()).await?;
    let changes = validate(&auth.user, body, &filter)?;
    let user = Box::pin(accounts::update_profile(
        state.db(),
        state.config(),
        auth.user.id,
        changes,
    ))
    .await?;
    Ok(Json(
        me_dto(state.db(), &user, state.config().domain()).await?,
    ))
}

async fn patch_bot(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiJson(body): ApiJson<PatchBotRequest>,
) -> Result<Json<MeDto>, AppError> {
    // Naming an owner needs no consent (as with any account), so the claim itself is rate limited.
    state.limits().check_bot(auth.user.id)?;
    let db = state.db();
    let owner = match body
        .owner
        .as_deref()
        .map(|handle| handle.trim().trim_start_matches('@'))
    {
        Some(handle) if body.bot && !handle.is_empty() => {
            let owner = accounts::find_for_login(db, handle)
                .await?
                .filter(|owner| owner.suspended_at.is_none())
                .ok_or(AppError::Missing(
                    "No account with that handle on this instance.",
                ))?;
            if owner.id == auth.user.id {
                return Err(AppError::bad_request(
                    "A bot can\u{2019}t be run by itself.",
                ));
            }
            Some(owner.id)
        }
        _ => None,
    };
    let user = accounts::set_bot(db, state.config(), auth.user.id, body.bot, owner).await?;
    Ok(Json(
        me_dto(state.db(), &user, state.config().domain()).await?,
    ))
}

/// Checks a settings patch against the profile rules and turns it into column changes.
pub(crate) fn validate(
    current: &UserRow,
    body: PatchMeRequest,
    filter: &ContentFilter,
) -> Result<ProfileChanges, AppError> {
    let blocked = |text: &str, message: &str| -> Result<(), AppError> {
        if filter.blocks_text(text) {
            return Err(AppError::bad_request(format!(
                "{message} Please change it."
            )));
        }
        Ok(())
    };
    let mut changes = ProfileChanges::default();

    if let Some(name) = body.display_name {
        let name = text_field(&name, "Name", 1, 50)?;
        if name != current.display_name {
            blocked(&name, "That display name isn\u{2019}t allowed.")?;
        }
        changes.display_name = Some(name);
    }
    if let Some(bio) = body.bio {
        let bio = text_field(&bio, "Bio", 0, 160)?;
        if bio != current.bio && filter.blocks_text(&bio) {
            return Err(AppError::bad_request(format!(
                "Your bio contains {REASON}, which isn\u{2019}t allowed here."
            )));
        }
        changes.bio = Some(bio);
    }
    if let Some(location) = body.location {
        let location = text_field(&location, "Location", 0, 30)?;
        if location != current.location {
            blocked(&location, "That location isn\u{2019}t allowed.")?;
        }
        changes.location = Some(location);
    }
    if let Some(website) = body.website {
        changes.website = Some(normalize_website(&text_field(
            &website, "Website", 0, 100,
        )?)?);
    }
    changes.avatar_shape = body.avatar_shape;
    changes.ink = body.ink.map(crate::dto::Ink::as_str);
    changes.dm_policy = body.dm_policy.map(crate::dto::DmPolicy::as_str);
    changes.discoverable = body.discoverable;
    changes.show_client = body.show_client;
    if let Some(language) = body.language {
        let language = language.trim();
        changes.language = if language.is_empty() {
            LanguageChange::Clear
        } else if LANGS.contains(&language) {
            LanguageChange::Set(language.to_owned())
        } else {
            return Err(AppError::bad_request("Unknown language."));
        };
    }
    if let Some(words) = body.muted_words {
        let mut clean: Vec<String> = Vec::new();
        for word in words {
            let word = word.trim().to_lowercase();
            if !word.is_empty() && !clean.contains(&word) {
                clean.push(word);
            }
        }
        if clean.len() > MAX_MUTED_WORDS {
            return Err(AppError::bad_request("You can mute up to 100 words."));
        }
        if clean.iter().any(|w| w.chars().count() > MAX_MUTED_WORD_LEN) {
            return Err(AppError::bad_request(
                "Muted words can be at most 40 characters.",
            ));
        }
        changes.muted_words = Some(clean);
    }
    if let Some(prefs) = body.notif_prefs {
        changes.notif_prefs = Some(prefs_json(prefs));
    }
    Ok(changes)
}

fn prefs_json(prefs: NotifPrefsDto) -> serde_json::Value {
    json!({
        "like": prefs.like,
        "repost": prefs.repost,
        "follow": prefs.follow,
        "reply": prefs.reply,
        "mention": prefs.mention,
        "quote": prefs.quote,
    })
}

/// Adds `https://` when no scheme is given and checks the result is one URL without spaces,
/// `<`, `>` or quotes.
fn normalize_website(website: &str) -> Result<String, AppError> {
    if website.is_empty() {
        return Ok(String::new());
    }
    let lower = website.to_ascii_lowercase();
    let url = if lower.starts_with("http://") || lower.starts_with("https://") {
        website.to_owned()
    } else {
        format!("https://{website}")
    };
    let rest = url.split_once("://").map_or("", |(_, rest)| rest);
    if rest.is_empty()
        || rest
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '<' | '>' | '"'))
    {
        return Err(AppError::bad_request("Website must be a valid URL."));
    }
    Ok(url)
}

async fn change_password(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    ApiJson(body): ApiJson<PasswordRequest>,
) -> Result<Json<OkDto>, AppError> {
    let session = auth.session_id()?;
    state.limits().check_password(auth.user.id)?;
    if !password::verify(body.current, auth.user.password_hash.clone()).await {
        return Err(AppError::forbidden("Your current password is wrong."));
    }
    password::validate(&body.next)?;
    let hash = password::hash(body.next).await?;
    accounts::set_password(state.db(), auth.user.id, &hash).await?;
    sessions::delete_others(state.db(), auth.user.id, session).await?;
    tokens::delete_all(state.db(), auth.user.id).await?;
    Ok(Json(OkDto { ok: true }))
}

async fn list_sessions(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
) -> Result<Json<Vec<SessionDto>>, AppError> {
    let rows = sessions::list(state.db(), auth.user.id, auth.session_id()?).await?;
    Ok(Json(rows.into_iter().map(session_dto).collect()))
}

fn session_dto(row: SessionRow) -> SessionDto {
    SessionDto {
        id: row.id,
        current: row.current,
        created_at: row.created_at,
        last_used_at: row.last_used_at,
        expires_at: row.expires_at,
        user_agent: row.user_agent,
        ip: row.ip,
        country: row.country,
        client: row.client,
    }
}

async fn revoke_others(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
) -> Result<Json<RevokedDto>, AppError> {
    let revoked = sessions::delete_others(state.db(), auth.user.id, auth.session_id()?).await?;
    Ok(Json(RevokedDto {
        revoked: i64::try_from(revoked).unwrap_or(i64::MAX),
    }))
}

async fn revoke_one(
    State(state): State<AppState>,
    CurrentUser(auth): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<OkDto>, AppError> {
    let id: Uuid = parse_id(&id, "Session not found.")?;
    if id == auth.session_id()? {
        return Err(AppError::bad_request(
            "Use \u{201c}Log out\u{201d} to end this session.",
        ));
    }
    if !sessions::delete_one(state.db(), auth.user.id, id).await? {
        return Err(AppError::Missing("Session not found."));
    }
    Ok(Json(OkDto { ok: true }))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn websites_get_a_scheme_and_reject_junk() {
        assert_eq!(normalize_website("").ok().as_deref(), Some(""));
        assert_eq!(
            normalize_website("example.org/a").ok().as_deref(),
            Some("https://example.org/a")
        );
        assert_eq!(
            normalize_website("HTTP://example.org").ok().as_deref(),
            Some("HTTP://example.org")
        );
        assert!(normalize_website("exa mple.org").is_err());
        assert!(normalize_website("https://").is_err());
        assert!(normalize_website("a<b").is_err());
    }
}
