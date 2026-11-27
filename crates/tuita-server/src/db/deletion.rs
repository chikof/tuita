//! Account deletion: the account row stays, anonymised, so old references keep resolving.

use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use super::audit;
use super::users::UserRow;
use crate::config::Config;
use crate::error::AppError;
use crate::federation::outbound;

const LAST_ADMIN: &str =
    "You\u{2019}re the only admin. Make someone else an admin before deleting your account.";

/// Anonymises `user` and removes what they own, in one transaction: their posts are
/// soft-deleted (counters of replied-to and quoted posts follow), and likes, reposts,
/// bookmarks, follows, blocks, mutes, messages, notifications, sessions, API tokens, passkeys,
/// affiliations, lists and pending scheduled posts go. The handle becomes `deleted_` plus twelve
/// hex digits of the id. Nothing is sent to other servers: this is for removing a remote
/// account; a local account goes through [`delete_local_account`].
///
/// # Errors
///
/// `400` when `user` is the only active local admin; database failures.
pub async fn delete_account(db: &PgPool, user: &UserRow) -> Result<(), AppError> {
    let mut tx = db.begin().await?;
    erase(&mut tx, None, user).await?;
    tx.commit().await?;
    Ok(())
}

/// [`delete_account`] for a local account: the `Delete` of its actor is queued for every known
/// remote inbox in the same transaction.
///
/// # Errors
///
/// As [`delete_account`].
pub async fn delete_local_account(
    db: &PgPool,
    config: &Config,
    user: &UserRow,
) -> Result<(), AppError> {
    let mut tx = db.begin().await?;
    erase(&mut tx, Some(config), user).await?;
    tx.commit().await?;
    Ok(())
}

/// [`delete_account`] on behalf of an admin, with the `user.delete` audit row in the same
/// transaction.
///
/// # Errors
///
/// As [`delete_account`].
pub async fn delete_account_as_staff(
    db: &PgPool,
    config: &Config,
    actor: Uuid,
    user: &UserRow,
    reason: &str,
) -> Result<(), AppError> {
    let mut tx = db.begin().await?;
    erase(&mut tx, Some(config), user).await?;
    audit::record(
        &mut tx,
        audit::Entry::new(Some(actor), "user.delete")
            .target(format!("@{}", user.handle))
            .text(reason)
            .about(user.id),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

async fn erase(
    tx: &mut PgConnection,
    config: Option<&Config>,
    user: &UserRow,
) -> Result<(), AppError> {
    if user.role == "admin" {
        // Locking every admin row makes two admins deleting themselves at once take turns.
        let admins: Vec<Uuid> = sqlx::query_scalar(
            "select id from users
              where role = 'admin' and domain is null and deleted_at is null
                and suspended_at is null
              order by id for update",
        )
        .fetch_all(&mut *tx)
        .await?;
        if admins.iter().all(|id| *id == user.id) {
            return Err(AppError::bad_request(LAST_ADMIN));
        }
    }
    if let Some(config) = config.filter(|_| user.deleted_at.is_none() && user.domain.is_none()) {
        outbound::account_deleted(tx, config, user.id, &user.handle).await?;
    }
    fix_counters(tx, user.id).await?;
    remove_rows(tx, user.id).await?;
    sqlx::query("update users set bot_owner_id = null where bot_owner_id = $1")
        .bind(user.id)
        .execute(&mut *tx)
        .await?;
    // The last 12 hex digits are random; the first 12 are a millisecond clock that can repeat.
    sqlx::query(
        "update users set
            handle = 'deleted_' || substr(replace(id::text, '-', ''), 21, 12),
            display_name = 'Deleted account', bio = '', location = '', website = '',
            avatar_id = null, banner_id = null, pinned_post_id = null,
            password_hash = null, role = 'user', verified = false,
            muted_words = '{}', followers_count = 0, following_count = 0, posts_count = 0,
            is_org = false, is_bot = false, bot_owner_id = null, deleted_at = now(),
            invited_with = null, notif_prefs = '{}', webauthn_id = gen_random_uuid(),
            public_key_pem = case when exists (select 1 from ap_deliveries where signer_id = $1)
                                  then public_key_pem end,
            private_key_pem = case when exists (select 1 from ap_deliveries where signer_id = $1)
                                   then private_key_pem end
          where id = $1",
    )
    .bind(user.id)
    .execute(&mut *tx)
    .await?;
    Ok(())
}

/// Lowers the counters that count rows of `user`, before those rows go.
async fn fix_counters(conn: &mut PgConnection, user: Uuid) -> Result<(), AppError> {
    for (parent_column, count_column) in [
        ("reply_to_id", "reply_count"),
        ("quote_of_id", "quote_count"),
    ] {
        let sql = format!(
            "update posts t set {count_column} = greatest(t.{count_column} - c.n, 0)
               from (select {parent_column} as id, count(*)::int4 as n from posts
                      where author_id = $1 and deleted_at is null and {parent_column} is not null
                      group by 1) c
              where t.id = c.id"
        );
        sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(user)
            .execute(&mut *conn)
            .await?;
    }
    for (table, count_column) in [("likes", "like_count"), ("reposts", "repost_count")] {
        let sql = format!(
            "update posts p set {count_column} = greatest(p.{count_column} - 1, 0)
               from {table} e where e.user_id = $1 and e.post_id = p.id"
        );
        sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(user)
            .execute(&mut *conn)
            .await?;
    }
    sqlx::query(
        "update users u set followers_count = greatest(u.followers_count - 1, 0)
           from follows f where f.follower_id = $1 and f.followee_id = u.id",
    )
    .bind(user)
    .execute(&mut *conn)
    .await?;
    sqlx::query(
        "update users u set following_count = greatest(u.following_count - 1, 0)
           from follows f where f.followee_id = $1 and f.follower_id = u.id",
    )
    .bind(user)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Soft-deletes the posts of `user` and deletes every row that ties them to others or to
/// their own posts.
async fn remove_rows(conn: &mut PgConnection, user: Uuid) -> Result<(), AppError> {
    sqlx::query("update posts set deleted_at = now(), text = '', cw = '' where author_id = $1 and deleted_at is null")
        .bind(user)
        .execute(&mut *conn)
        .await?;
    let notes: Vec<String> = sqlx::query_scalar(
        "select ap_id from messages
          where (sender_id = $1 or recipient_id = $1) and ap_id is not null",
    )
    .bind(user)
    .fetch_all(&mut *conn)
    .await?;
    drop_message_deliveries(conn, &notes).await?;
    sqlx::query("delete from post_views where viewer = 'u' || $1::text")
        .bind(user)
        .execute(&mut *conn)
        .await?;
    for table in [
        "likes",
        "reposts",
        "bookmarks",
        "notifications",
        "post_tags",
        "mentions",
        "post_media",
        "post_edits",
        "post_views",
        "polls",
    ] {
        let sql = format!(
            "delete from {table} where post_id in (select id from posts where author_id = $1)"
        );
        sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(user)
            .execute(&mut *conn)
            .await?;
    }
    // Their uploads stay on disk until cleanup, but no post may keep them public.
    sqlx::query("update media set post_id = null where owner_id = $1")
        .bind(user)
        .execute(&mut *conn)
        .await?;
    for (table, columns) in [
        ("likes", &["user_id"][..]),
        ("reposts", &["user_id"]),
        ("bookmarks", &["user_id"]),
        ("follows", &["follower_id", "followee_id"]),
        ("blocks", &["user_id", "target_id"]),
        ("mutes", &["user_id", "target_id"]),
        ("messages", &["sender_id", "recipient_id"]),
        ("notifications", &["user_id", "actor_id"]),
        ("sessions", &["user_id"]),
        ("list_members", &["user_id"]),
        ("lists", &["owner_id"]),
        ("feed_snapshots", &["user_id"]),
        ("api_tokens", &["user_id"]),
        ("oauth_codes", &["user_id"]),
        ("passkeys", &["user_id"]),
        ("webauthn_challenges", &["user_id"]),
        ("poll_votes", &["user_id"]),
        ("status_idempotency", &["user_id"]),
        ("reports", &["reporter_id"]),
        ("affiliations", &["org_id", "user_id"]),
    ] {
        let condition = columns
            .iter()
            .map(|column| format!("{column} = $1"))
            .collect::<Vec<_>>()
            .join(" or ");
        let sql = format!("delete from {table} where {condition}");
        sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(user)
            .execute(&mut *conn)
            .await?;
    }
    sqlx::query("delete from scheduled_posts where author_id = $1")
        .bind(user)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Removes the queued deliveries that carry one of the direct messages `notes` (note ids, as
/// `messages.ap_id` holds them): the `Create` (`{note}/activity`), or any activity whose object
/// is the note. Returns how many it removed.
///
/// # Errors
///
/// Database failures.
pub async fn drop_message_deliveries(
    conn: &mut PgConnection,
    notes: &[String],
) -> Result<u64, AppError> {
    if notes.is_empty() {
        return Ok(0);
    }
    Ok(sqlx::query(
        "delete from ap_deliveries d
          where exists (select 1 from unnest($1::text[]) n
                         where starts_with(d.activity->>'id', n)
                            or d.activity #>> '{object,id}' = n
                            or d.activity->>'object' = n)",
    )
    .bind(notes)
    .execute(conn)
    .await?
    .rows_affected())
}

/// Wipes the federation key pair of every deleted account that no queued delivery needs to sign
/// with any more. Erasing an account keeps its key only while its `Delete` is still on the
/// way. Returns how many accounts it wiped.
///
/// # Errors
///
/// Database failures.
pub async fn wipe_idle_keys(db: &PgPool) -> Result<u64, AppError> {
    Ok(sqlx::query(
        "update users u set public_key_pem = null, private_key_pem = null
          where u.deleted_at is not null
            and (u.public_key_pem is not null or u.private_key_pem is not null)
            and not exists (select 1 from ap_deliveries d where d.signer_id = u.id)",
    )
    .execute(db)
    .await?
    .rows_affected())
}
