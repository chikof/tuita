//! Accounts: the full row, lookups and the conversion to [`UserDto`], [`UserRefDto`], [`MeDto`].

use std::collections::HashMap;

use serde_json::Value;
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::emoji;
use crate::dto::{
    AffiliationBadgeDto, AvatarShape, DmPolicy, Ink, MeDto, NotifPrefsDto, NotificationKindDto,
    Relation, Role, SettingsDto, UserCounts, UserDto, UserRefDto,
};
use crate::error::AppError;

/// Column list that fills a [`UserRow`] from a users row aliased `u`.
///
/// `bot_owner` is the handle of the account that runs a bot. `affiliation` is a JSON object
/// (`handle`, `displayName`, `avatarId`, `avatarShape`, `label`) of the organization the account
/// accepted, or null.
pub const COLS: &str = "u.id, u.handle, u.domain, u.display_name, u.bio, u.location, u.website, \
    u.avatar_id, u.banner_id, u.avatar_shape, u.ink, u.password_hash, u.role, \
    u.verified, u.suspended_at, u.deleted_at, u.is_org, u.is_bot, u.dm_policy, u.discoverable, \
    u.show_client, u.muted_words, u.notif_prefs, u.language, u.pinned_post_id, u.followers_count, \
    u.following_count, u.posts_count, u.created_at, u.ap_id, u.inbox_url, u.shared_inbox_url, \
    u.url, u.fetched_at, u.manually_approves, \
    (select bo.handle from users bo where bo.id = u.bot_owner_id) as bot_owner, \
    (select jsonb_build_object('handle', og.handle, 'displayName', og.display_name, \
            'avatarId', og.avatar_id, 'avatarShape', og.avatar_shape, 'label', af.label) \
       from affiliations af join users og on og.id = af.org_id \
      where af.user_id = u.id and af.status = 'active') as affiliation";

/// A users row. No `Debug`: it holds the password hash.
#[derive(FromRow, Clone)]
pub struct UserRow {
    /// Account id.
    pub id: Uuid,
    /// Handle without `@`.
    pub handle: String,
    /// Home domain; `None` for local accounts.
    pub domain: Option<String>,
    /// Display name.
    pub display_name: String,
    /// Biography.
    pub bio: String,
    /// Location.
    pub location: String,
    /// Website.
    pub website: String,
    /// Avatar media.
    pub avatar_id: Option<Uuid>,
    /// Banner media.
    pub banner_id: Option<Uuid>,
    /// Stored avatar shape text.
    pub avatar_shape: String,
    /// Stored ink text.
    pub ink: String,
    /// Argon2 PHC string; `None` cannot sign in.
    pub password_hash: Option<String>,
    /// Stored role text.
    pub role: String,
    /// Verified badge.
    pub verified: bool,
    /// Suspension time.
    pub suspended_at: Option<OffsetDateTime>,
    /// Deletion time.
    pub deleted_at: Option<OffsetDateTime>,
    /// Organization.
    pub is_org: Option<bool>,
    /// Bot.
    pub is_bot: Option<bool>,
    /// Handle of the account that runs this bot.
    pub bot_owner: Option<String>,
    /// The accepted affiliation, see [`COLS`].
    pub affiliation: Option<Value>,
    /// Stored DM policy text.
    pub dm_policy: String,
    /// Shown in suggestions and search.
    pub discoverable: Option<bool>,
    /// Show posting device.
    pub show_client: Option<bool>,
    /// Muted words.
    pub muted_words: Option<Vec<String>>,
    /// Notification preferences, a JSON object of kind to bool.
    pub notif_prefs: Option<Value>,
    /// Interface language.
    pub language: Option<String>,
    /// Pinned post.
    pub pinned_post_id: Option<Uuid>,
    /// Followers counter.
    pub followers_count: i32,
    /// Following counter.
    pub following_count: i32,
    /// Posts counter.
    pub posts_count: i32,
    /// Sign-up time.
    pub created_at: OffsetDateTime,
    /// The remote actor's id; `None` for local accounts, whose id is computed from the handle.
    pub ap_id: Option<String>,
    /// The remote actor's personal inbox.
    pub inbox_url: Option<String>,
    /// The remote actor's shared inbox.
    pub shared_inbox_url: Option<String>,
    /// The remote profile page.
    pub url: Option<String>,
    /// When a remote profile was last fetched.
    pub fetched_at: Option<OffsetDateTime>,
    /// A remote account that approves follows by hand.
    pub manually_approves: Option<bool>,
}

impl UserRow {
    /// The stored role.
    #[must_use]
    pub fn role(&self) -> Role {
        Role::parse(&self.role).unwrap_or(Role::User)
    }

    /// Moderator or admin.
    #[must_use]
    pub fn is_staff(&self) -> bool {
        self.role() != Role::User
    }

    /// Whether `kind` notifications are on. Everything is on unless switched off.
    #[must_use]
    pub fn wants(&self, kind: NotificationKindDto) -> bool {
        notif_prefs_allow(self.notif_prefs.as_ref(), kind)
    }
}

/// Whether the stored preference object allows `kind`.
#[must_use]
pub fn notif_prefs_allow(prefs: Option<&Value>, kind: NotificationKindDto) -> bool {
    prefs
        .and_then(|prefs| prefs.get(kind.as_str()))
        .and_then(Value::as_bool)
        .unwrap_or(true)
}

/// The URL of a stored media file.
#[must_use]
pub fn media_url(id: Uuid) -> String {
    format!("/media/{id}")
}

/// Loads a user by id, whatever its state.
///
/// # Errors
///
/// Database failures.
pub async fn by_id<'e>(
    db: impl sqlx::PgExecutor<'e>,
    id: Uuid,
) -> Result<Option<UserRow>, AppError> {
    let sql = format!("select {COLS} from users u where u.id = $1");
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .fetch_optional(db)
        .await?)
}

/// Finds the local account a `{handle}` path segment names, as the viewer may see it.
///
/// A leading `@` is dropped. `user@domain` is a remote handle, which 404s until federation
/// exists. Suspended and deleted accounts 404 too, except for staff. The `tuita` system
/// account never exists here.
///
/// # Errors
///
/// `404 User not found.` or a database failure.
pub async fn find_local(
    db: &PgPool,
    handle: &str,
    viewer_is_staff: bool,
) -> Result<UserRow, AppError> {
    let handle = handle.strip_prefix('@').unwrap_or(handle);
    if handle.contains('@') {
        return Err(AppError::Missing("User not found."));
    }
    let sql = format!(
        "select {COLS} from users u where lower(u.handle) = lower($1) and u.domain is null \
         and lower(u.handle) <> 'tuita' \
         and u.deleted_at is null and ($2 or u.suspended_at is null)"
    );
    sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(handle)
        .bind(viewer_is_staff)
        .fetch_optional(db)
        .await?
        .ok_or(AppError::Missing("User not found."))
}

/// How the viewer relates to each of `ids`. Ids without a row are absent from the map.
///
/// # Errors
///
/// Database failures.
pub async fn relations(
    db: &PgPool,
    viewer: Uuid,
    ids: &[Uuid],
) -> Result<HashMap<Uuid, Relation>, AppError> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows: Vec<(Uuid, bool, bool, bool, bool, bool, bool)> = sqlx::query_as(
        "select t.id,
            exists (select 1 from follows where follower_id = $1 and followee_id = t.id and state = 'accepted'),
            exists (select 1 from follows where follower_id = t.id and followee_id = $1 and state = 'accepted'),
            exists (select 1 from blocks where user_id = $1 and target_id = t.id),
            exists (select 1 from blocks where user_id = t.id and target_id = $1),
            exists (select 1 from mutes where user_id = $1 and target_id = t.id),
            exists (select 1 from follows where follower_id = $1 and followee_id = t.id and state = 'pending')
         from unnest($2::uuid[]) as t(id)",
    )
    .bind(viewer)
    .bind(ids)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(id, following, follows_you, blocked, blocks_you, muted, follow_pending)| {
                (
                    id,
                    Relation {
                        following,
                        follows_you,
                        blocked,
                        blocks_you,
                        muted,
                        follow_pending,
                    },
                )
            },
        )
        .collect())
}

/// Profile cards for `rows`, in the same order, with each one's relation to `viewer`.
///
/// # Errors
///
/// Database failures.
pub async fn dtos(db: &PgPool, viewer: Uuid, rows: &[UserRow]) -> Result<Vec<UserDto>, AppError> {
    let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
    let relations = relations(db, viewer, &ids).await?;
    let mut cards: Vec<UserDto> = rows
        .iter()
        .map(|row| user_dto(row, relations.get(&row.id).copied().unwrap_or_default()))
        .collect();
    emoji::hydrate(db, &mut cards).await?;
    Ok(cards)
}

/// One profile card.
///
/// # Errors
///
/// Database failures.
pub async fn dto(db: &PgPool, viewer: Uuid, row: &UserRow) -> Result<UserDto, AppError> {
    let relation = relations(db, viewer, &[row.id])
        .await?
        .remove(&row.id)
        .unwrap_or_default();
    let mut card = user_dto(row, relation);
    emoji::hydrate(db, std::slice::from_mut(&mut card)).await?;
    Ok(card)
}

/// Builds the profile card of `row` with a known `relation`.
#[must_use]
pub fn user_dto(row: &UserRow, relation: Relation) -> UserDto {
    UserDto {
        id: row.id,
        handle: row.handle.clone(),
        domain: row.domain.clone(),
        display_name: row.display_name.clone(),
        bio: row.bio.clone(),
        location: row.location.clone(),
        website: row.website.clone(),
        avatar_url: row.avatar_id.map(media_url),
        banner_url: row.banner_id.map(media_url),
        avatar_shape: AvatarShape::parse(&row.avatar_shape).unwrap_or(AvatarShape::Square),
        ink: Ink::parse(&row.ink).unwrap_or(Ink::Bone),
        verified: row.verified,
        staff: Some(row.role()).filter(|role| *role != Role::User),
        org: row.is_org.unwrap_or(false),
        bot: row.is_bot.unwrap_or(false),
        bot_owner: row.bot_owner.clone(),
        affiliation: row.affiliation.as_ref().and_then(affiliation_badge),
        pinned_post_id: row.pinned_post_id,
        created_at: row.created_at,
        counts: UserCounts {
            followers: row.followers_count,
            following: row.following_count,
            posts: row.posts_count,
        },
        relation,
        suspended: row.suspended_at.is_some(),
        url: row.url.clone(),
        locked: row.manually_approves.unwrap_or(false),
        emojis: Vec::new(),
    }
}

/// The badge a [`COLS`] `affiliation` object describes; `None` when it is malformed.
fn affiliation_badge(value: &Value) -> Option<AffiliationBadgeDto> {
    let text = |key: &str| value.get(key).and_then(Value::as_str);
    Some(AffiliationBadgeDto {
        handle: text("handle")?.to_owned(),
        display_name: text("displayName")?.to_owned(),
        avatar_url: text("avatarId")
            .and_then(|id| Uuid::parse_str(id).ok())
            .map(media_url),
        avatar_shape: text("avatarShape")
            .and_then(AvatarShape::parse)
            .unwrap_or(AvatarShape::Square),
        label: text("label").unwrap_or_default().to_owned(),
    })
}

/// The small card of `row`.
#[must_use]
pub fn user_ref(row: &UserRow) -> UserRefDto {
    UserRefDto {
        id: row.id,
        handle: row.handle.clone(),
        domain: row.domain.clone(),
        display_name: row.display_name.clone(),
        avatar_url: row.avatar_id.map(media_url),
        avatar_shape: AvatarShape::parse(&row.avatar_shape).unwrap_or(AvatarShape::Square),
        ink: Ink::parse(&row.ink).unwrap_or(Ink::Bone),
        verified: row.verified,
        bot: row.is_bot.unwrap_or(false),
        emojis: Vec::new(),
    }
}

/// Small cards for `ids`, keyed by id. Missing ids are absent.
///
/// # Errors
///
/// Database failures.
pub async fn refs(db: &PgPool, ids: &[Uuid]) -> Result<HashMap<Uuid, UserRefDto>, AppError> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let sql = format!("select {COLS} from users u where u.id = any($1)");
    let rows: Vec<UserRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(ids)
        .fetch_all(db)
        .await?;
    let mut cards: HashMap<Uuid, UserRefDto> =
        rows.iter().map(|row| (row.id, user_ref(row))).collect();
    let mut shown: Vec<&mut UserRefDto> = cards.values_mut().collect();
    emoji::hydrate(db, &mut shown).await?;
    Ok(cards)
}

/// The signed-in account with its private settings. `domain` is the instance domain.
///
/// # Errors
///
/// Database failures.
pub async fn me_dto(db: &PgPool, row: &UserRow, domain: &str) -> Result<MeDto, AppError> {
    let mut user = user_dto(row, Relation::default());
    emoji::hydrate(db, std::slice::from_mut(&mut user)).await?;
    Ok(MeDto {
        user,
        role: row.role(),
        settings: SettingsDto {
            dm_policy: DmPolicy::parse(&row.dm_policy).unwrap_or(DmPolicy::Everyone),
            discoverable: row.discoverable.unwrap_or(true),
            language: row.language.clone(),
            show_client: row.show_client.unwrap_or(true),
            notif_prefs: NotifPrefsDto {
                like: row.wants(NotificationKindDto::Like),
                repost: row.wants(NotificationKindDto::Repost),
                follow: row.wants(NotificationKindDto::Follow),
                reply: row.wants(NotificationKindDto::Reply),
                mention: row.wants(NotificationKindDto::Mention),
                quote: row.wants(NotificationKindDto::Quote),
            },
            muted_words: row.muted_words.clone().unwrap_or_default(),
        },
        federated_handle: format!("{}@{domain}", row.handle),
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn notification_prefs_default_to_on() {
        let prefs = json!({"like": false, "repost": true});
        assert!(!notif_prefs_allow(Some(&prefs), NotificationKindDto::Like));
        assert!(notif_prefs_allow(Some(&prefs), NotificationKindDto::Repost));
        assert!(notif_prefs_allow(Some(&prefs), NotificationKindDto::Follow));
        assert!(notif_prefs_allow(None, NotificationKindDto::Like));
    }
}
