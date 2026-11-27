//! Account writes: sign-up, profile and password changes.

use serde_json::Value;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use super::users::{COLS, UserRow};
use super::{audit, invites};
use crate::config::Config;
use crate::dto::AvatarShape;
use crate::error::AppError;
use crate::federation::outbound;
use crate::secrets;

/// Serialises first-admin decisions so two simultaneous first sign-ups cannot both win.
const ADMIN_LOCK: i64 = 0x7475_6974_6100_0001;

/// Whether a local account with a password can administer the instance.
///
/// # Errors
///
/// Database failures.
pub async fn has_admin(conn: &mut PgConnection) -> Result<bool, AppError> {
    Ok(sqlx::query_scalar(
        "select exists (select 1 from users where role = 'admin' and domain is null
                        and deleted_at is null and password_hash is not null)",
    )
    .fetch_one(conn)
    .await?)
}

/// The `403` for an invite code that does not work.
pub const INVITE_INVALID: &str = "That invite code is invalid or has been used up.";

/// What a sign-up stores.
pub struct NewUser {
    /// The validated handle.
    pub handle: String,
    /// The validated display name.
    pub display_name: String,
    /// Argon2 PHC string.
    pub password_hash: String,
    /// The generated avatar's shape.
    pub avatar_shape: AvatarShape,
    /// Preferred language, when valid.
    pub language: Option<String>,
    /// The invite code (normalised) this sign-up uses up.
    pub invite: Option<String>,
    /// The account's ActivityPub key pair, made before the transaction opens.
    pub keys: tuita_core::ap::KeyPair,
}

/// Creates a local account. The first account becomes admin when no admin exists yet; with
/// `config.admin_handle` set, only that handle (ignoring case) may. The private key is stored
/// sealed when `config` has a `SECRET_KEY`.
///
/// # Errors
///
/// `409 That handle is taken.` when the handle exists; database failures.
pub async fn create(db: &PgPool, new: NewUser, config: &Config) -> Result<UserRow, AppError> {
    let admin_handle = config.admin_handle.as_deref();
    let id = Uuid::now_v7();
    let mut tx = db.begin().await?;
    sqlx::query("select pg_advisory_xact_lock($1)")
        .bind(ADMIN_LOCK)
        .execute(&mut *tx)
        .await?;
    let admin = !has_admin(&mut tx).await?
        && admin_handle.is_none_or(|only| {
            only.trim_start_matches('@')
                .eq_ignore_ascii_case(&new.handle)
        });
    let role = if admin { "admin" } else { "user" };
    if let Some(code) = &new.invite
        && !invites::consume(&mut tx, code).await?
    {
        return Err(AppError::forbidden(INVITE_INVALID));
    }
    let sql = format!(
        "insert into users as u (id, handle, display_name, password_hash, avatar_shape, role, language, invited_with,
            public_key_pem, private_key_pem)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) returning {COLS}"
    );
    let inserted = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .bind(&new.handle)
        .bind(&new.display_name)
        .bind(&new.password_hash)
        .bind(new.avatar_shape.as_str())
        .bind(role)
        .bind(new.language.as_deref())
        .bind(new.invite.as_deref())
        .bind(&new.keys.public_pem)
        .bind(secrets::store(
            config.secret_key.as_deref(),
            &secrets::user_context(id),
            &new.keys.private_pem,
        ))
        .fetch_one(&mut *tx)
        .await;
    let user: UserRow = match inserted {
        Ok(user) => user,
        Err(err) if is_unique_violation(&err) => {
            return Err(AppError::conflict("That handle is taken."));
        }
        Err(err) => return Err(err.into()),
    };
    if admin {
        audit::record(
            &mut tx,
            audit::Entry::new(Some(user.id), "admin.bootstrap")
                .target(&user.handle)
                .text("first account became admin")
                .about(user.id),
        )
        .await?;
    }
    tx.commit().await?;
    Ok(user)
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .and_then(|db| db.code())
        .is_some_and(|code| code == "23505")
}

/// Whether a local account with this handle exists (ignoring case, any state).
///
/// # Errors
///
/// Database failures.
pub async fn handle_taken(db: &PgPool, handle: &str) -> Result<bool, AppError> {
    Ok(sqlx::query_scalar(
        "select exists (select 1 from users where lower(handle) = lower($1) and domain is null)",
    )
    .bind(handle)
    .fetch_one(db)
    .await?)
}

/// The local account a login names, including suspended ones; deleted accounts are gone.
///
/// # Errors
///
/// Database failures.
pub async fn find_for_login(db: &PgPool, handle: &str) -> Result<Option<UserRow>, AppError> {
    let sql = format!(
        "select {COLS} from users u
         where lower(u.handle) = lower($1) and u.domain is null and u.deleted_at is null"
    );
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(handle)
        .fetch_optional(db)
        .await?)
}

/// What a settings patch does to the preferred language.
#[derive(Default)]
pub enum LanguageChange {
    /// Leave it.
    #[default]
    Keep,
    /// Remove it.
    Clear,
    /// Set it.
    Set(String),
}

/// Validated profile and settings changes; `None` leaves a column alone.
#[derive(Default)]
pub struct ProfileChanges {
    /// Display name.
    pub display_name: Option<String>,
    /// Bio.
    pub bio: Option<String>,
    /// Location.
    pub location: Option<String>,
    /// Website.
    pub website: Option<String>,
    /// Avatar shape.
    pub avatar_shape: Option<AvatarShape>,
    /// Ink tone text.
    pub ink: Option<&'static str>,
    /// DM policy text.
    pub dm_policy: Option<&'static str>,
    /// Discoverable.
    pub discoverable: Option<bool>,
    /// Show posting device.
    pub show_client: Option<bool>,
    /// What to do with the language.
    pub language: LanguageChange,
    /// Muted words.
    pub muted_words: Option<Vec<String>>,
    /// Notification preferences object.
    pub notif_prefs: Option<Value>,
}

impl ProfileChanges {
    /// Whether a change shows in the account's ActivityPub actor, so followers on other
    /// servers need an `Update`.
    fn touches_actor(&self) -> bool {
        self.display_name.is_some()
            || self.bio.is_some()
            || self.website.is_some()
            || self.discoverable.is_some()
    }
}

/// Applies `changes` to `user` and returns the fresh row.
///
/// # Errors
///
/// Database failures.
pub async fn update_profile(
    db: &PgPool,
    config: &Config,
    user: Uuid,
    changes: ProfileChanges,
) -> Result<UserRow, AppError> {
    let federates = changes.touches_actor();
    let (language_set, language) = match changes.language {
        LanguageChange::Keep => (false, None),
        LanguageChange::Clear => (true, None),
        LanguageChange::Set(language) => (true, Some(language)),
    };
    let sql = format!(
        "update users u set
            display_name = coalesce($2, u.display_name),
            bio = coalesce($3, u.bio),
            location = coalesce($4, u.location),
            website = coalesce($5, u.website),
            avatar_shape = coalesce($6, u.avatar_shape),
            ink = coalesce($7, u.ink),
            dm_policy = coalesce($8, u.dm_policy),
            discoverable = coalesce($9, u.discoverable),
            show_client = coalesce($10, u.show_client),
            language = case when $11 then $12 else u.language end,
            muted_words = coalesce($13, u.muted_words),
            notif_prefs = coalesce($14, u.notif_prefs)
         where u.id = $1 returning {COLS}"
    );
    let mut tx = db.begin().await?;
    let row = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(user)
        .bind(changes.display_name)
        .bind(changes.bio)
        .bind(changes.location)
        .bind(changes.website)
        .bind(changes.avatar_shape.map(AvatarShape::as_str))
        .bind(changes.ink)
        .bind(changes.dm_policy)
        .bind(changes.discoverable)
        .bind(changes.show_client)
        .bind(language_set)
        .bind(language)
        .bind(changes.muted_words)
        .bind(changes.notif_prefs)
        .fetch_one(&mut *tx)
        .await?;
    if changes.show_client == Some(false) {
        sqlx::query("update posts set client = null where author_id = $1")
            .bind(user)
            .execute(&mut *tx)
            .await?;
    }
    if federates {
        Box::pin(outbound::profile_updated(&mut tx, config, user)).await?;
    }
    tx.commit().await?;
    Ok(row)
}

/// Stores a new password hash.
///
/// # Errors
///
/// Database failures.
pub async fn set_password(db: &PgPool, user: Uuid, hash: &str) -> Result<(), AppError> {
    sqlx::query("update users set password_hash = $2 where id = $1")
        .bind(user)
        .bind(hash)
        .execute(db)
        .await?;
    Ok(())
}

/// Marks the account as automated (or not) and names the account that runs it. The owner is
/// dropped when `bot` is false.
///
/// # Errors
///
/// Database failures.
pub async fn set_bot(
    db: &PgPool,
    config: &Config,
    user: Uuid,
    bot: bool,
    owner: Option<Uuid>,
) -> Result<UserRow, AppError> {
    let sql = format!(
        "update users u set is_bot = $2, bot_owner_id = case when $2 then $3 end
          where u.id = $1 returning {COLS}"
    );
    let mut tx = db.begin().await?;
    let row = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(user)
        .bind(bot)
        .bind(owner)
        .fetch_one(&mut *tx)
        .await?;
    Box::pin(outbound::profile_updated(&mut tx, config, user)).await?;
    tx.commit().await?;
    Ok(row)
}
