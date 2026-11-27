//! Account administration from the shell: the bodies of `tuita admin ...`.
//!
//! Every command names a local account by handle.

use sqlx::{FromRow, PgPool};
use thiserror::Error;

use crate::db::{accounts, moderation};
use crate::dto::Role;
use crate::error::AppError;

/// What `tuita admin <command> <handle>` does to the account.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Make an admin.
    Promote,
    /// Make a moderator.
    Moderator,
    /// Make a regular user.
    Demote,
    /// Give the verified badge.
    Verify,
    /// Remove the verified badge.
    Unverify,
    /// Suspend the account.
    Suspend,
    /// Lift a suspension.
    Unsuspend,
}

impl Action {
    /// The command name, as typed.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Promote => "promote",
            Self::Moderator => "moderator",
            Self::Demote => "demote",
            Self::Verify => "verify",
            Self::Unverify => "unverify",
            Self::Suspend => "suspend",
            Self::Unsuspend => "unsuspend",
        }
    }
}

/// Why an admin command failed.
#[derive(Debug, Error)]
pub enum AdminError {
    /// No local account has the handle.
    #[error("No local account @{0}")]
    NoSuchAccount(Box<str>),
    /// The database failed.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    /// A shared database function failed.
    #[error("{0}")]
    App(#[from] AppError),
}

/// Applies `action` to the local account `handle` (a leading `@` is dropped).
///
/// # Errors
///
/// [`AdminError::NoSuchAccount`] when there is no such account; database failures.
pub async fn apply(db: &PgPool, action: Action, handle: &str) -> Result<(), AdminError> {
    let handle = handle.strip_prefix('@').unwrap_or(handle);
    let user = accounts::find_for_login(db, handle)
        .await?
        .ok_or_else(|| AdminError::NoSuchAccount(handle.into()))?;
    // The routes call the same functions with an actor; the command line has none.
    let mut tx = db.begin().await?;
    match action {
        Action::Promote => moderation::set_role(&mut tx, None, &user, Role::Admin).await?,
        Action::Moderator => moderation::set_role(&mut tx, None, &user, Role::Moderator).await?,
        Action::Demote => moderation::set_role(&mut tx, None, &user, Role::User).await?,
        Action::Verify => moderation::set_verified(&mut tx, None, &user, true).await?,
        Action::Unverify => moderation::set_verified(&mut tx, None, &user, false).await?,
        Action::Suspend => moderation::set_suspended(&mut tx, None, &user, true, "").await?,
        Action::Unsuspend => moderation::set_suspended(&mut tx, None, &user, false, "").await?,
    }
    tx.commit().await?;
    Ok(())
}

/// One line of `tuita admin list`.
#[derive(Debug, FromRow, PartialEq, Eq)]
pub struct StaffRow {
    /// The handle.
    pub handle: String,
    /// `admin`, `moderator` or `user`.
    pub role: String,
    /// Carries the verified badge.
    pub verified: bool,
    /// Currently suspended.
    pub suspended: bool,
}

/// The staff and verified accounts, admins first.
///
/// # Errors
///
/// Database failures.
pub async fn list(db: &PgPool) -> Result<Vec<StaffRow>, AdminError> {
    Ok(sqlx::query_as(
        "select handle, role, verified, suspended_at is not null as suspended from users
         where domain is null and deleted_at is null and (role <> 'user' or verified)
         order by (role = 'admin') desc, (role = 'moderator') desc, lower(handle)",
    )
    .fetch_all(db)
    .await?)
}
