//! What staff do to accounts, posts and reports. Every function writes its audit row in the
//! caller's transaction. The admin routes and `tuita admin ...` both call these, so the two
//! cannot drift: the command line passes no actor.

use sqlx::PgConnection;
use uuid::Uuid;

use super::audit::{self, Entry};
use super::live::{self, Removal};
use super::post_actions::soft_delete_federated;
use super::users::{self, UserRow};
use crate::config::Config;
use crate::dto::{ReportAction, Role};
use crate::error::AppError;
use crate::realtime::Hub;

fn handle_of(user: &UserRow) -> String {
    format!("@{}", user.handle)
}

/// Gives or takes the verified badge.
///
/// # Errors
///
/// Database failures.
pub async fn set_verified(
    conn: &mut PgConnection,
    actor: Option<Uuid>,
    target: &UserRow,
    on: bool,
) -> Result<(), AppError> {
    sqlx::query("update users set verified = $2 where id = $1")
        .bind(target.id)
        .bind(on)
        .execute(&mut *conn)
        .await?;
    let action = if on { "user.verify" } else { "user.unverify" };
    audit::record(
        conn,
        Entry::new(actor, action)
            .target(handle_of(target))
            .about(target.id),
    )
    .await
}

/// Suspends an account, which ends its sessions, or lifts the suspension.
///
/// # Errors
///
/// Database failures.
pub async fn set_suspended(
    conn: &mut PgConnection,
    actor: Option<Uuid>,
    target: &UserRow,
    on: bool,
    why: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "update users set suspended_at = case when $2 then coalesce(suspended_at, now()) end
          where id = $1",
    )
    .bind(target.id)
    .bind(on)
    .execute(&mut *conn)
    .await?;
    if on {
        sqlx::query("delete from sessions where user_id = $1")
            .bind(target.id)
            .execute(&mut *conn)
            .await?;
    }
    let action = if on { "user.suspend" } else { "user.unsuspend" };
    audit::record(
        conn,
        Entry::new(actor, action)
            .target(handle_of(target))
            .text(why)
            .about(target.id),
    )
    .await
}

/// Sets an account's role.
///
/// # Errors
///
/// Database failures.
pub async fn set_role(
    conn: &mut PgConnection,
    actor: Option<Uuid>,
    target: &UserRow,
    role: Role,
) -> Result<(), AppError> {
    sqlx::query("update users set role = $2 where id = $1")
        .bind(target.id)
        .bind(role.as_str())
        .execute(&mut *conn)
        .await?;
    audit::record(
        conn,
        Entry::new(actor, "user.role")
            .target(handle_of(target))
            .text(role.as_str())
            .about(target.id),
    )
    .await
}

/// Marks an account as an organization or not. An organization has no affiliation as a member;
/// an account that stops being one loses the affiliations it handed out.
///
/// # Errors
///
/// Database failures.
pub async fn set_org(
    conn: &mut PgConnection,
    actor: Option<Uuid>,
    target: &UserRow,
    on: bool,
) -> Result<(), AppError> {
    sqlx::query("update users set is_org = $2 where id = $1")
        .bind(target.id)
        .bind(on)
        .execute(&mut *conn)
        .await?;
    let column = if on { "user_id" } else { "org_id" };
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "delete from affiliations where {column} = $1"
    )))
    .bind(target.id)
    .execute(&mut *conn)
    .await?;
    let action = if on { "user.org" } else { "user.unorg" };
    audit::record(
        conn,
        Entry::new(actor, action)
            .target(handle_of(target))
            .about(target.id),
    )
    .await
}

/// What a rename changes.
#[derive(Debug, Default)]
pub struct Rename {
    /// The new handle, already validated.
    pub handle: Option<String>,
    /// Set the display name to the handle.
    pub reset_name: bool,
    /// Clear the bio.
    pub reset_bio: bool,
}

/// Renames an account. `409 That handle is taken.` when the handle exists.
///
/// # Errors
///
/// The `409`; database failures.
pub async fn rename(
    conn: &mut PgConnection,
    actor: Uuid,
    target: &UserRow,
    change: &Rename,
) -> Result<(), AppError> {
    let new_handle = change.handle.as_deref().unwrap_or(&target.handle);
    let updated = sqlx::query(
        "update users set handle = $2,
                display_name = case when $3 then $2 else display_name end,
                bio = case when $4 then '' else bio end
          where id = $1",
    )
    .bind(target.id)
    .bind(new_handle)
    .bind(change.reset_name)
    .bind(change.reset_bio)
    .execute(&mut *conn)
    .await;
    match updated {
        Ok(_) => {}
        Err(err)
            if err
                .as_database_error()
                .and_then(|db| db.code())
                .is_some_and(|code| code == "23505") =>
        {
            return Err(AppError::conflict("That handle is taken."));
        }
        Err(err) => return Err(err.into()),
    }
    let mut parts: Vec<String> = Vec::new();
    if change.handle.is_some() {
        parts.push(format!("handle \u{2192} @{new_handle}"));
    }
    if change.reset_name {
        parts.push("name reset".to_owned());
    }
    if change.reset_bio {
        parts.push("bio cleared".to_owned());
    }
    audit::record(
        conn,
        Entry::new(Some(actor), "user.rename")
            .target(handle_of(target))
            .text(parts.join(", "))
            .about(target.id),
    )
    .await
}

/// Deletes every post of an account. Returns how many went and the live timeline entries that
/// went with them: hand them to [`live::publish_removal`] after the commit.
///
/// # Errors
///
/// Database failures.
pub async fn purge_posts(
    conn: &mut PgConnection,
    config: &Config,
    hub: &Hub,
    actor: Uuid,
    target: &UserRow,
) -> Result<(i32, Removal), AppError> {
    let ids: Vec<Uuid> =
        sqlx::query_scalar("select id from posts where author_id = $1 and deleted_at is null")
            .bind(target.id)
            .fetch_all(&mut *conn)
            .await?;
    let mut deleted = 0_i32;
    let mut removal = Removal::default();
    for id in ids {
        let entries = live::removal(&mut *conn, hub, id).await?;
        if soft_delete_federated(&mut *conn, config, id).await? {
            deleted += 1;
            removal.absorb(entries);
        }
    }
    audit::record(
        conn,
        Entry::new(Some(actor), "user.purge_posts")
            .target(handle_of(target))
            .text(format!("{deleted} posts"))
            .about(target.id),
    )
    .await?;
    Ok((deleted, removal))
}

/// Decides a report. In one transaction: the post is deleted and/or the account suspended, the
/// audit rows are written, and the open reports about the same post (about the same account
/// after a suspension) are resolved with it. Returns the live timeline entries of a deleted
/// post: hand them to [`live::publish_removal`] after the commit.
///
/// # Errors
///
/// `404 Report not found.`; `409 This report was already resolved.`; `403 Admins can’t be
/// suspended from a report.`; database failures.
pub async fn resolve_report(
    conn: &mut PgConnection,
    config: &Config,
    hub: &Hub,
    actor: Uuid,
    report: Uuid,
    action: ReportAction,
) -> Result<Removal, AppError> {
    let row: Option<(Uuid, Option<Uuid>, String)> =
        sqlx::query_as("select target_id, post_id, status from reports where id = $1 for update")
            .bind(report)
            .fetch_optional(&mut *conn)
            .await?;
    let Some((target_id, post_id, status)) = row else {
        return Err(AppError::Missing("Report not found."));
    };
    if status != "open" {
        return Err(AppError::conflict("This report was already resolved."));
    }
    let target = users::by_id(&mut *conn, target_id)
        .await?
        .ok_or(AppError::Missing("Report not found."))?;
    let deletes = matches!(
        action,
        ReportAction::DeletePost | ReportAction::DeleteAndSuspend
    );
    let suspends = matches!(
        action,
        ReportAction::SuspendUser | ReportAction::DeleteAndSuspend
    );
    if suspends && target.role() == Role::Admin {
        return Err(AppError::forbidden(
            "Admins can\u{2019}t be suspended from a report.",
        ));
    }
    let mut removal = Removal::default();
    if deletes && let Some(post) = post_id {
        let entries = live::removal(&mut *conn, hub, post).await?;
        if soft_delete_federated(&mut *conn, config, post).await? {
            removal = entries;
            audit::record(
                &mut *conn,
                Entry::new(Some(actor), "post.delete")
                    .target(handle_of(&target))
                    .text(format!("post {post} (report {report})"))
                    .about(target.id),
            )
            .await?;
        }
    }
    if suspends {
        set_suspended(
            &mut *conn,
            Some(actor),
            &target,
            true,
            &format!("report {report}"),
        )
        .await?;
    }
    sqlx::query(
        "update reports set status = 'resolved', resolution = $2, resolved_by = $3,
                resolved_at = now()
          where status = 'open'
            and (id = $1 or (target_id = $4 and ($5 or post_id is not distinct from $6)))",
    )
    .bind(report)
    .bind(action.as_str())
    .bind(actor)
    .bind(target_id)
    .bind(suspends)
    .bind(post_id)
    .execute(&mut *conn)
    .await?;
    if action == ReportAction::Dismiss {
        audit::record(
            conn,
            Entry::new(Some(actor), "report.dismiss")
                .target(format!("#{report}"))
                .about(target.id),
        )
        .await?;
    }
    Ok(removal)
}
