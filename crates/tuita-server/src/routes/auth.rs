//! Sign-up, sign-in, sign-out and the account switcher.

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{AppendHeaders, IntoResponse as _, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use tuita_core::handle::parse_handle;
use tuita_core::locale::LANGS;

use crate::auth::{self, ApiJson, ApiJsonOrDefault, MAX_ACCOUNTS, Tokens, password};
use crate::db::sessions::ClientMeta;
use crate::db::settings;
use crate::db::users::{me_dto, user_ref};
use crate::db::{accounts, emoji, invites, users};
use crate::dto::{
    AccountDto, AccountsDto, AuthResponse, AvatarShape, LoginRequest, LogoutRequest,
    LogoutResponse, SignupRequest, SignupsMode, SwitchRequest,
};
use crate::error::AppError;
use crate::state::AppState;

const SHAPES: [AvatarShape; 4] = [
    AvatarShape::Square,
    AvatarShape::Circle,
    AvatarShape::Squircle,
    AvatarShape::Octagon,
];

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/signup", post(signup))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/accounts", get(list_accounts))
        .route("/accounts/switch", post(switch))
}

async fn signup(
    State(state): State<AppState>,
    meta: ClientMeta,
    headers: HeaderMap,
    ApiJson(body): ApiJson<SignupRequest>,
) -> Result<Response, AppError> {
    state
        .limits()
        .check_auth(meta.ip.unwrap_or(std::net::Ipv4Addr::UNSPECIFIED.into()))?;
    let db = state.db();
    let admin_exists = {
        let mut conn = db.acquire().await?;
        accounts::has_admin(&mut conn).await?
    };
    // Until an admin exists the first account must be able to join, whatever the mode is.
    let invite = if admin_exists {
        match settings::load(db).await?.signups {
            SignupsMode::Open => None,
            SignupsMode::Closed => {
                return Err(AppError::forbidden(
                    "Sign-ups are closed on this instance right now.",
                ));
            }
            SignupsMode::Invite => Some(usable_invite(db, body.invite.as_deref()).await?),
        }
    } else {
        None
    };

    let filter = settings::content_filter(db).await?;
    let handle = parse_handle(&body.handle)
        .map_err(|err| AppError::bad_request(err.user_message()))?
        .to_string();
    if filter.is_reserved(&handle) {
        return Err(AppError::bad_request(
            "That handle is reserved. Please pick another.",
        ));
    }
    if filter.blocks_handle(&handle) {
        return Err(AppError::bad_request(
            "That handle isn\u{2019}t allowed. Please pick another.",
        ));
    }
    let display_name = text_field(&body.display_name, "Name", 1, 50)?;
    if filter.blocks_text(&display_name) {
        return Err(AppError::bad_request(
            "That display name isn\u{2019}t allowed. Please pick another.",
        ));
    }
    password::validate(&body.password)?;
    if !body.accept_terms {
        return Err(AppError::bad_request(
            "Please accept the Terms of Service and Privacy Policy.",
        ));
    }
    let tokens = Tokens::from_headers(&headers);
    if body.add && auth::browser_accounts(db, &tokens).await?.len() >= MAX_ACCOUNTS {
        return Err(too_many_accounts());
    }
    if accounts::handle_taken(db, &handle).await? {
        return Err(AppError::conflict("That handle is taken."));
    }
    let password_hash = password::hash(body.password).await?;
    let keys = crate::federation::keys::new_pair().await?;
    let language = body
        .language
        .filter(|language| LANGS.contains(&language.as_str()));
    let user = Box::pin(accounts::create(
        db,
        accounts::NewUser {
            handle,
            display_name,
            password_hash,
            avatar_shape: SHAPES[rand::random_range(0..SHAPES.len())],
            language,
            invite,
            keys,
        },
        state.config(),
    ))
    .await?;
    let cookies =
        auth::start_session(db, state.config(), &tokens, user.id, body.add, &meta).await?;
    let user = users::by_id(db, user.id).await?.unwrap_or(user);
    let reply = AuthResponse {
        user: me_dto(db, &user, state.config().domain()).await?,
    };
    Ok((cookies, Json(reply)).into_response())
}

/// The normalised invite code of a sign-up in invite-only mode, once it checks out.
async fn usable_invite(db: &sqlx::PgPool, sent: Option<&str>) -> Result<String, AppError> {
    let sent = sent.unwrap_or_default().trim();
    if sent.is_empty() {
        return Err(AppError::forbidden(
            "Sign-ups are closed on this instance right now. You need an invite code.",
        ));
    }
    match invites::normalize(sent) {
        Some(code) if invites::is_usable(db, &code).await? => Ok(code),
        _ => Err(AppError::forbidden(accounts::INVITE_INVALID)),
    }
}

fn too_many_accounts() -> AppError {
    AppError::bad_request(format!(
        "You can be signed in to up to {MAX_ACCOUNTS} accounts at once."
    ))
}

/// Trims `value` and checks its length in characters, with a message per field.
pub(super) fn text_field(
    value: &str,
    name: &str,
    min: usize,
    max: usize,
) -> Result<String, AppError> {
    let value = value.trim();
    let len = value.chars().count();
    if len < min {
        return Err(AppError::bad_request(if min == 1 {
            format!("{name} is required.")
        } else {
            format!("{name} must be at least {min} characters.")
        }));
    }
    if len > max {
        return Err(AppError::bad_request(format!(
            "{name} must be at most {max} characters."
        )));
    }
    Ok(value.to_owned())
}

async fn login(
    State(state): State<AppState>,
    meta: ClientMeta,
    headers: HeaderMap,
    ApiJson(body): ApiJson<LoginRequest>,
) -> Result<Response, AppError> {
    state
        .limits()
        .check_auth(meta.ip.unwrap_or(std::net::Ipv4Addr::UNSPECIFIED.into()))?;
    let db = state.db();
    let handle = body.handle.trim();
    let handle = handle.strip_prefix('@').unwrap_or(handle);
    let found = accounts::find_for_login(db, handle).await?;
    let stored = found.as_ref().and_then(|user| user.password_hash.clone());
    let verified = password::verify(body.password, stored).await;
    let Some(user) = found.filter(|_| verified) else {
        return Err(AppError::BadCredentials);
    };
    if user.suspended_at.is_some() {
        return Err(AppError::forbidden(
            "This account has been suspended. Contact the instance admin if you think this is a mistake.",
        ));
    }
    let tokens = Tokens::from_headers(&headers);
    let cookies =
        auth::start_session(db, state.config(), &tokens, user.id, body.add, &meta).await?;
    let reply = AuthResponse {
        user: me_dto(db, &user, state.config().domain()).await?,
    };
    Ok((cookies, Json(reply)).into_response())
}

async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    ApiJsonOrDefault(body): ApiJsonOrDefault<LogoutRequest>,
) -> Result<
    (
        AppendHeaders<[(axum::http::HeaderName, axum::http::HeaderValue); 2]>,
        Json<LogoutResponse>,
    ),
    AppError,
> {
    let tokens = Tokens::from_headers(&headers);
    let (next, cookies) = auth::end_session(state.db(), state.config(), &tokens, body.all).await?;
    let switched_to = match next {
        Some(user) => Some(me_dto(state.db(), &user, state.config().domain()).await?),
        None => None,
    };
    Ok((cookies, Json(LogoutResponse { switched_to })))
}

async fn list_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AccountsDto>, AppError> {
    let tokens = Tokens::from_headers(&headers);
    let mut accounts: Vec<AccountDto> = auth::browser_accounts(state.db(), &tokens)
        .await?
        .into_iter()
        .map(|account| AccountDto {
            user: user_ref(&account.user),
            active: account.active,
        })
        .collect();
    let mut cards: Vec<_> = accounts
        .iter_mut()
        .map(|account| &mut account.user)
        .collect();
    emoji::hydrate(state.db(), &mut cards).await?;
    Ok(Json(AccountsDto {
        accounts,
        max: u8::try_from(MAX_ACCOUNTS).unwrap_or(5),
    }))
}

async fn switch(
    State(state): State<AppState>,
    headers: HeaderMap,
    ApiJson(body): ApiJson<SwitchRequest>,
) -> Result<Response, AppError> {
    let tokens = Tokens::from_headers(&headers);
    let Some((user, cookies)) =
        auth::switch_account(state.db(), state.config(), &tokens, body.id).await?
    else {
        return Err(AppError::Missing(
            "That account isn\u{2019}t signed in on this browser any more.",
        ));
    };
    let reply = AuthResponse {
        user: me_dto(state.db(), &user, state.config().domain()).await?,
    };
    Ok((cookies, Json(reply)).into_response())
}
