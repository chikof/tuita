//! Accounts and sessions: passwords, session tokens, the cookie pair behind the account
//! switcher, request extractors and the CSRF guard.
//!
//! The browser holds two `HttpOnly` cookies. `tuita_session` carries the active account's token
//! and `tuita_accounts` up to four more, joined with `.`. Only the SHA-256 of a token is stored
//! (see [`crate::db::sessions`]). The account switcher adds an account's token to
//! `tuita_accounts` and switches by moving a token into `tuita_session`.

pub mod bearer;
pub mod csrf;
mod extract;
pub mod passkey;
pub mod password;

use axum::http::header::{COOKIE, HeaderName, SET_COOKIE};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::AppendHeaders;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest as _, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

pub use extract::{
    Admin, ApiJson, ApiJsonOrDefault, ApiQuery, Auth, Credential, CurrentUser, MaybeUser, Staff,
};

use crate::config::Config;
use crate::db::sessions::{self, ClientMeta, SESSION_DAYS};
use crate::db::users::UserRow;
use crate::error::AppError;

/// Cookie holding the active account's session token.
pub const SESSION_COOKIE: &str = "tuita_session";
/// Cookie holding the other signed-in accounts' tokens, joined with `.`.
pub const ACCOUNTS_COOKIE: &str = "tuita_accounts";
/// Accounts one browser may hold.
pub const MAX_ACCOUNTS: usize = 5;

/// How long a session cookie lives, in seconds.
const COOKIE_MAX_AGE: i64 = SESSION_DAYS * 24 * 3600;

/// A fresh session token: 32 random bytes, base64url without padding.
#[must_use]
pub fn new_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// The stored form of a token.
#[must_use]
pub fn hash_token(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

/// Whether `a` and `b` hold the same bytes. The comparison takes the same time wherever they
/// first differ; use it for secrets, hashes of secrets and PKCE challenges.
#[must_use]
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    use subtle::ConstantTimeEq as _;
    a.ct_eq(b).into()
}

/// The value of cookie `name` in a `Cookie` header, when present and not empty.
#[must_use]
pub fn cookie_value<'a>(header: &'a str, name: &str) -> Option<&'a str> {
    header.split(';').find_map(|part| {
        let (key, value) = part.split_once('=')?;
        (key.trim() == name && !value.trim().is_empty()).then(|| value.trim())
    })
}

/// A token as it may appear in a cookie: URL-safe base64, 20 to 64 characters.
fn plausible(token: &str) -> bool {
    (20..=64).contains(&token.len())
        && token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// The session tokens the browser sent.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tokens {
    /// Token of the active account.
    pub active: Option<String>,
    /// Tokens of the other accounts.
    pub others: Vec<String>,
}

impl Tokens {
    /// Reads both cookies from the request headers. Malformed tokens are dropped.
    #[must_use]
    pub fn from_headers(headers: &HeaderMap) -> Self {
        let mut tokens = Self::default();
        for raw in headers
            .get_all(COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
        {
            if tokens.active.is_none() {
                tokens.active = cookie_value(raw, SESSION_COOKIE)
                    .filter(|t| plausible(t))
                    .map(str::to_owned);
            }
            if let Some(list) = cookie_value(raw, ACCOUNTS_COOKIE) {
                tokens
                    .others
                    .extend(list.split('.').filter(|t| plausible(t)).map(str::to_owned));
            }
        }
        tokens
    }

    /// Every distinct token, the active one first.
    fn all(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        for token in self.active.iter().chain(&self.others) {
            if !out.contains(&token.as_str()) {
                out.push(token);
            }
        }
        out
    }
}

/// `Set-Cookie` headers that store `active` and `others`; an empty value ends the cookie.
/// At most [`MAX_ACCOUNTS`] tokens in all are kept.
pub fn set_cookies(
    config: &Config,
    active: Option<&str>,
    others: &[String],
) -> AppendHeaders<[(HeaderName, HeaderValue); 2]> {
    let mut kept: Vec<&str> = Vec::new();
    for token in others {
        if Some(token.as_str()) != active && !kept.contains(&token.as_str()) {
            kept.push(token);
        }
    }
    kept.truncate(MAX_ACCOUNTS - 1);
    let session = cookie(
        config,
        SESSION_COOKIE,
        active.unwrap_or(""),
        active.is_some(),
    );
    let accounts = cookie(config, ACCOUNTS_COOKIE, &kept.join("."), !kept.is_empty());
    AppendHeaders([(SET_COOKIE, session), (SET_COOKIE, accounts)])
}

fn cookie(config: &Config, name: &str, value: &str, live: bool) -> HeaderValue {
    let max_age = if live { COOKIE_MAX_AGE } else { 0 };
    let secure = if config.secure_cookies {
        "; Secure"
    } else {
        ""
    };
    let text = format!("{name}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure}");
    HeaderValue::from_str(&text).expect("tokens are URL-safe ASCII")
}

/// An account signed in on this browser.
pub struct BrowserAccount {
    /// The session token.
    pub token: String,
    /// The account.
    pub user: UserRow,
    /// Whether it is the active one.
    pub active: bool,
}

/// The accounts signed in on this browser, the active one first. Tokens that no longer map to
/// a live session are dropped, and so is a second token for the same account.
///
/// # Errors
///
/// Database failures.
pub async fn browser_accounts(
    db: &PgPool,
    tokens: &Tokens,
) -> Result<Vec<BrowserAccount>, AppError> {
    let mut out: Vec<BrowserAccount> = Vec::new();
    for token in tokens.all() {
        let Some(found) = sessions::find(db, &hash_token(token)).await? else {
            continue;
        };
        if out.iter().any(|account| account.user.id == found.user.id) {
            continue;
        }
        out.push(BrowserAccount {
            token: token.to_owned(),
            user: found.user,
            active: tokens.active.as_deref() == Some(token),
        });
    }
    Ok(out)
}

/// A new session for `user` and the cookies that make it active.
///
/// With `add` the accounts already signed in stay in the switcher (signing in to one that is
/// already there replaces its entry); without it they are dropped.
///
/// # Errors
///
/// `400` when `add` would exceed [`MAX_ACCOUNTS`]; database failures.
pub async fn start_session(
    db: &PgPool,
    config: &Config,
    tokens: &Tokens,
    user: Uuid,
    add: bool,
    meta: &ClientMeta,
) -> Result<AppendHeaders<[(HeaderName, HeaderValue); 2]>, AppError> {
    let keep: Vec<String> = if add {
        browser_accounts(db, tokens)
            .await?
            .into_iter()
            .filter(|account| account.user.id != user)
            .map(|account| account.token)
            .collect()
    } else {
        Vec::new()
    };
    if keep.len() >= MAX_ACCOUNTS {
        return Err(AppError::bad_request(format!(
            "You can be signed in to up to {MAX_ACCOUNTS} accounts at once."
        )));
    }
    let token = new_token();
    sessions::insert(db, user, &hash_token(&token), meta).await?;
    Ok(set_cookies(config, Some(&token), &keep))
}

/// Makes `target` the active account of this browser. `None` when it is not signed in here.
///
/// # Errors
///
/// Database failures.
pub async fn switch_account(
    db: &PgPool,
    config: &Config,
    tokens: &Tokens,
    target: Uuid,
) -> Result<Option<(UserRow, AppendHeaders<[(HeaderName, HeaderValue); 2]>)>, AppError> {
    let accounts = browser_accounts(db, tokens).await?;
    let Some(position) = accounts.iter().position(|a| a.user.id == target) else {
        return Ok(None);
    };
    let mut accounts = accounts;
    let chosen = accounts.remove(position);
    let others: Vec<String> = accounts.into_iter().map(|a| a.token).collect();
    let headers = set_cookies(config, Some(&chosen.token), &others);
    Ok(Some((chosen.user, headers)))
}

/// Signs the active account out (every account with `all`). Returns the account that became
/// active, if any, and the cookies to set.
///
/// # Errors
///
/// Database failures.
pub async fn end_session(
    db: &PgPool,
    config: &Config,
    tokens: &Tokens,
    all: bool,
) -> Result<
    (
        Option<UserRow>,
        AppendHeaders<[(HeaderName, HeaderValue); 2]>,
    ),
    AppError,
> {
    let accounts = browser_accounts(db, tokens).await?;
    let doomed: Vec<&str> = if all {
        tokens.all()
    } else {
        tokens.active.as_deref().into_iter().collect()
    };
    for token in doomed {
        sessions::delete_token(db, &hash_token(token)).await?;
    }
    let mut rest: Vec<BrowserAccount> = if all {
        Vec::new()
    } else {
        accounts.into_iter().filter(|a| !a.active).collect()
    };
    let next = (!rest.is_empty()).then(|| rest.remove(0));
    let others: Vec<String> = rest.into_iter().map(|a| a.token).collect();
    let headers = set_cookies(config, next.as_ref().map(|a| a.token.as_str()), &others);
    Ok((next.map(|a| a.user), headers))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tokens_are_43_url_safe_characters() {
        let token = new_token();
        assert_eq!(token.len(), 43);
        assert!(plausible(&token));
        assert_ne!(token, new_token());
        assert_eq!(hash_token(&token).len(), 32);
    }

    #[test]
    fn reads_both_cookies_and_drops_junk() {
        let (a, b, c) = (new_token(), new_token(), new_token());
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_str(&format!(
                "other=1; tuita_session={a}; tuita_accounts={b}.junk!.{c}"
            ))
            .expect("valid header"),
        );
        let tokens = Tokens::from_headers(&headers);
        assert_eq!(tokens.active.as_deref(), Some(a.as_str()));
        assert_eq!(tokens.others, [b, c]);
    }

    #[test]
    fn empty_cookie_values_count_as_absent() {
        assert_eq!(cookie_value("tuita_session=; a=b", SESSION_COOKIE), None);
        assert_eq!(
            cookie_value("a=b; tuita_session=x", SESSION_COOKIE),
            Some("x")
        );
    }
}
