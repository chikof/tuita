//! OAuth2 storage: registered apps, authorization codes and the tokens a code is exchanged for.
//!
//! Secrets (client secrets, codes, access tokens) are stored as SHA-256 only. A code is single
//! use: [`exchange`] locks its row, so two requests with one code take turns, and the second one
//! sees `used_at` and deletes the token the first one got.

use sqlx::{FromRow, PgConnection, PgPool};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::tokens;
use crate::auth::{ct_eq, hash_token, new_token};
use crate::dto::TokenScope;
use crate::error::AppError;

/// How long an authorization code lives.
pub const CODE_TTL: Duration = Duration::minutes(10);
/// OAuth tokens one account may hold; the oldest are dropped past this.
pub const MAX_USER_TOKENS: i64 = 100;
/// `client_credentials` tokens one app may hold; the oldest are dropped past this.
pub const MAX_APP_TOKENS: i64 = 100;

/// A registered client.
#[derive(Clone, Debug, FromRow)]
pub struct App {
    /// The app id.
    pub id: Uuid,
    /// The public client id.
    pub client_id: String,
    /// SHA-256 of the client secret.
    pub client_secret_hash: Vec<u8>,
    /// The name the client gave.
    pub name: String,
    /// Redirect URIs as registered.
    pub redirect_uris: Vec<String>,
    /// Scope texts as registered.
    pub scopes: Vec<String>,
    /// The client's website.
    pub website: Option<String>,
    /// When it registered.
    pub created_at: OffsetDateTime,
}

impl App {
    /// Whether `secret` is this app's client secret. Compares the hashes in constant time.
    #[must_use]
    pub fn secret_matches(&self, secret: &str) -> bool {
        ct_eq(&hash_token(secret), &self.client_secret_hash)
    }
}

/// What a client sent to register.
pub struct NewApp<'a> {
    /// The client's name.
    pub name: &'a str,
    /// Validated redirect URIs.
    pub redirect_uris: &'a [String],
    /// Validated scope texts.
    pub scopes: &'a [String],
    /// The client's website, when it gave one.
    pub website: Option<&'a str>,
}

/// Registers an app. The second value is the client secret, which is never stored.
///
/// # Errors
///
/// Database failures.
pub async fn create_app(db: &PgPool, new: &NewApp<'_>) -> Result<(App, String), AppError> {
    let secret = tokens::new_secret();
    let app: App = sqlx::query_as(
        "insert into oauth_apps (id, client_id, client_secret_hash, name, redirect_uris, scopes, website)
         values ($1, $2, $3, $4, $5, $6, $7)
         returning id, client_id, client_secret_hash, name, redirect_uris, scopes, website, created_at",
    )
    .bind(Uuid::now_v7())
    .bind(new_token())
    .bind(hash_token(&secret).as_slice())
    .bind(new.name)
    .bind(new.redirect_uris)
    .bind(new.scopes)
    .bind(new.website)
    .fetch_one(db)
    .await?;
    Ok((app, secret))
}

/// The app with public id `client_id`.
///
/// # Errors
///
/// Database failures.
pub async fn app_by_client_id(db: &PgPool, client_id: &str) -> Result<Option<App>, AppError> {
    Ok(sqlx::query_as(
        "select id, client_id, client_secret_hash, name, redirect_uris, scopes, website, created_at
           from oauth_apps where client_id = $1",
    )
    .bind(client_id)
    .fetch_optional(db)
    .await?)
}

/// The app with id `id`.
///
/// # Errors
///
/// Database failures.
pub async fn app_by_id(db: &PgPool, id: Uuid) -> Result<Option<App>, AppError> {
    Ok(sqlx::query_as(
        "select id, client_id, client_secret_hash, name, redirect_uris, scopes, website, created_at
           from oauth_apps where id = $1",
    )
    .bind(id)
    .fetch_optional(db)
    .await?)
}

/// Stores a code for `user` and returns it; only its hash is kept.
///
/// # Errors
///
/// Database failures.
pub async fn create_code(
    db: &PgPool,
    app: Uuid,
    user: Uuid,
    scopes: &[TokenScope],
    redirect_uri: &str,
    code_challenge: Option<&str>,
) -> Result<String, AppError> {
    let code = new_token();
    let scope_texts: Vec<&str> = scopes.iter().map(|scope| scope.as_str()).collect();
    sqlx::query(
        "insert into oauth_codes (code_hash, app_id, user_id, scopes, redirect_uri, code_challenge, expires_at)
         values ($1, $2, $3, $4, $5, $6, now() + make_interval(secs => $7))",
    )
    .bind(hash_token(&code).as_slice())
    .bind(app)
    .bind(user)
    .bind(&scope_texts)
    .bind(redirect_uri)
    .bind(code_challenge)
    .bind(CODE_TTL.as_seconds_f64())
    .execute(db)
    .await?;
    Ok(code)
}

/// A token just made. The secret exists only here.
pub struct Issued {
    /// The secret.
    pub token: String,
    /// What it may do.
    pub scopes: Vec<TokenScope>,
    /// When it was made.
    pub created_at: OffsetDateTime,
}

/// What a client presents to trade a code for a token.
pub struct CodeExchange<'a> {
    /// The authenticated app.
    pub app: &'a App,
    /// The code as received.
    pub code: &'a str,
    /// The `redirect_uri` parameter.
    pub redirect_uri: Option<&'a str>,
    /// The S256 challenge computed from the presented verifier, when the request had a valid one.
    pub verifier_challenge: Option<&'a str>,
    /// The request carried this app's client secret.
    pub secret_verified: bool,
}

/// The answer to [`exchange`].
pub enum Exchanged {
    /// The code was good; here is the token.
    Issued(Issued),
    /// The code is refused; the text is the `error_description`.
    Refused(&'static str),
}

#[derive(FromRow)]
struct CodeRow {
    app_id: Uuid,
    user_id: Uuid,
    scopes: Vec<String>,
    redirect_uri: String,
    code_challenge: Option<String>,
    expired: bool,
    used: bool,
    token_id: Option<Uuid>,
}

/// Trades a code for a token, once.
///
/// The code row is locked for the whole transaction, so a second request with the same code
/// waits and then finds it used. A used code is refused and the token it produced is deleted.
/// Wrong parameters leave the code alone; only a successful exchange (or a replay) changes it.
///
/// The rules, in order: the code exists, was not used, has not expired, belongs to this app,
/// the redirect URI equals the one it was issued for, and the proof matches. A code issued
/// with a PKCE challenge needs the matching verifier. A code issued without one needs the
/// client secret, and a verifier is refused for it.
///
/// # Errors
///
/// Database failures.
pub async fn exchange(db: &PgPool, request: &CodeExchange<'_>) -> Result<Exchanged, AppError> {
    let mut tx = db.begin().await?;
    let hash = hash_token(request.code);
    let row: Option<CodeRow> = sqlx::query_as(
        "select app_id, user_id, scopes, redirect_uri, code_challenge, expires_at <= now() as expired,
                used_at is not null as used, token_id
           from oauth_codes where code_hash = $1 for update",
    )
    .bind(hash.as_slice())
    .fetch_optional(&mut *tx)
    .await?;
    let Some(row) = row else {
        return Ok(Exchanged::Refused("The authorization code is not valid."));
    };
    if row.used {
        if let Some(token) = row.token_id {
            sqlx::query("delete from api_tokens where id = $1")
                .bind(token)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        return Ok(Exchanged::Refused(
            "The authorization code was already used.",
        ));
    }
    if row.expired {
        return Ok(Exchanged::Refused("The authorization code has expired."));
    }
    if row.app_id != request.app.id {
        return Ok(Exchanged::Refused(
            "The authorization code was issued to another client.",
        ));
    }
    if request.redirect_uri != Some(row.redirect_uri.as_str()) {
        return Ok(Exchanged::Refused(
            "The redirect URI does not match the authorization request.",
        ));
    }
    match (&row.code_challenge, request.verifier_challenge) {
        (Some(stored), Some(presented)) => {
            if !ct_eq(stored.as_bytes(), presented.as_bytes()) {
                return Ok(Exchanged::Refused("The code verifier does not match."));
            }
        }
        (Some(_), None) => {
            return Ok(Exchanged::Refused(
                "A code verifier is required for this authorization code.",
            ));
        }
        (None, Some(_)) => {
            return Ok(Exchanged::Refused(
                "This authorization code was not issued with a code challenge.",
            ));
        }
        (None, None) => {
            if !request.secret_verified {
                return Ok(Exchanged::Refused(
                    "The client secret is required for this authorization code.",
                ));
            }
        }
    }
    let scopes: Vec<TokenScope> = row
        .scopes
        .iter()
        .filter_map(|scope| TokenScope::parse(scope))
        .collect();
    let (id, issued) = issue(&mut tx, Some(row.user_id), request.app, scopes).await?;
    sqlx::query("update oauth_codes set used_at = now(), token_id = $2 where code_hash = $1")
        .bind(hash.as_slice())
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "delete from api_tokens
          where user_id = $1 and app_id is not null
            and id not in (select id from api_tokens where user_id = $1 and app_id is not null
                            order by id desc limit $2)",
    )
    .bind(row.user_id)
    .bind(MAX_USER_TOKENS)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Exchanged::Issued(issued))
}

/// A token for the app itself (`client_credentials`): no user, so it only identifies the app.
///
/// # Errors
///
/// Database failures.
pub async fn issue_app_token(
    db: &PgPool,
    app: &App,
    scopes: Vec<TokenScope>,
) -> Result<Issued, AppError> {
    let mut tx = db.begin().await?;
    let (_, issued) = issue(&mut tx, None, app, scopes).await?;
    sqlx::query(
        "delete from api_tokens
          where app_id = $1 and user_id is null
            and id not in (select id from api_tokens where app_id = $1 and user_id is null
                            order by id desc limit $2)",
    )
    .bind(app.id)
    .bind(MAX_APP_TOKENS)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(issued)
}

async fn issue(
    conn: &mut PgConnection,
    user: Option<Uuid>,
    app: &App,
    scopes: Vec<TokenScope>,
) -> Result<(Uuid, Issued), AppError> {
    let secret = tokens::new_secret();
    let id = Uuid::now_v7();
    let scope_texts: Vec<&str> = scopes.iter().map(|scope| scope.as_str()).collect();
    let created_at: OffsetDateTime = sqlx::query_scalar(
        "insert into api_tokens (id, user_id, name, scopes, token_hash, prefix, app_id)
         values ($1, $2, $3, $4, $5, $6, $7) returning created_at",
    )
    .bind(id)
    .bind(user)
    .bind(&app.name)
    .bind(&scope_texts)
    .bind(hash_token(&secret).as_slice())
    .bind(tokens::shown_prefix(&secret))
    .bind(app.id)
    .fetch_one(&mut *conn)
    .await?;
    Ok((
        id,
        Issued {
            token: secret,
            scopes,
            created_at,
        },
    ))
}

/// Deletes the token `secret` when it belongs to `app`. `false` when there was none.
///
/// # Errors
///
/// Database failures.
pub async fn revoke(db: &PgPool, app: Uuid, secret: &str) -> Result<bool, AppError> {
    Ok(
        sqlx::query("delete from api_tokens where token_hash = $1 and app_id = $2")
            .bind(hash_token(secret).as_slice())
            .bind(app)
            .execute(db)
            .await?
            .rows_affected()
            > 0,
    )
}

/// Removes codes a day past their expiry. The day is how long a used code stays recognisable.
///
/// # Errors
///
/// Database failures.
pub async fn expire_codes(db: &PgPool) -> Result<u64, AppError> {
    Ok(
        sqlx::query("delete from oauth_codes where expires_at < now() - interval '1 day'")
            .execute(db)
            .await?
            .rows_affected(),
    )
}
