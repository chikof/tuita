//! Password policy and argon2id hashing.

use std::sync::OnceLock;

use argon2::Argon2;
use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher as _, PasswordVerifier as _};

use crate::error::AppError;

/// Shortest password.
pub const MIN_LEN: usize = 8;
/// Longest password.
pub const MAX_LEN: usize = 200;

/// Checks the length rules of a password.
///
/// # Errors
///
/// `400` with a message when the password is too short or too long.
pub fn validate(password: &str) -> Result<(), AppError> {
    let len = password.chars().count();
    if len < MIN_LEN {
        return Err(AppError::bad_request(
            "Password must be at least 8 characters.",
        ));
    }
    if len > MAX_LEN {
        return Err(AppError::bad_request("Password is too long."));
    }
    Ok(())
}

/// Hashes `password` with argon2id and default parameters on a blocking thread.
///
/// # Errors
///
/// Internal errors only.
pub async fn hash(password: String) -> Result<String, AppError> {
    let hashed = tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
    })
    .await
    .map_err(|err| AppError::Internal(err.into()))?;
    hashed.map_err(|err| AppError::Internal(anyhow::anyhow!("cannot hash password: {err}")))
}

/// Checks `password` against a stored PHC string. `None` (an account that cannot sign in) and a
/// malformed hash both fail, after the same amount of work as a real check so that timing does
/// not tell which handles exist.
pub async fn verify(password: String, stored: Option<String>) -> bool {
    tokio::task::spawn_blocking(move || {
        let (candidate, real) = match stored.as_deref().map(PasswordHash::new) {
            Some(Ok(hash)) => (hash, true),
            _ => (dummy().clone(), false),
        };
        let ok = Argon2::default()
            .verify_password(password.as_bytes(), &candidate)
            .is_ok();
        ok && real
    })
    .await
    .unwrap_or(false)
}

fn dummy() -> &'static PasswordHash {
    static DUMMY: OnceLock<PasswordHash> = OnceLock::new();
    DUMMY.get_or_init(|| {
        Argon2::default()
            .hash_password(b"tuita dummy password")
            .expect("hashing a fixed password cannot fail")
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn length_policy() {
        assert!(validate("1234567").is_err());
        assert!(validate("12345678").is_ok());
        assert!(validate(&"x".repeat(200)).is_ok());
        assert!(validate(&"x".repeat(201)).is_err());
        assert!(validate("ünïcödé!").is_ok());
    }

    #[tokio::test]
    async fn hash_then_verify() {
        let stored = hash("correct horse".to_owned()).await.expect("hashes");
        assert!(stored.starts_with("$argon2id$"));
        assert!(verify("correct horse".to_owned(), Some(stored.clone())).await);
        assert!(!verify("wrong".to_owned(), Some(stored)).await);
        assert!(!verify("anything".to_owned(), None).await);
        assert!(!verify("anything".to_owned(), Some("garbage".to_owned())).await);
    }
}
