//! `/admin/...`: the staff console. The
//! database work and the audit rows are in `db::moderation`, `db::admin_reads` and friends.

use axum::extract::{Path, State};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use serde::Deserialize;
use sqlx::PgConnection;
use tuita_core::handle::{HandleError, parse_handle};
use uuid::Uuid;

use super::auth::text_field;
use crate::auth::{Admin, ApiJson, ApiJsonOrDefault, ApiQuery, Auth, Staff};
use crate::cursor::{Limit, PageQuery, parse_id};
use crate::db::moderation::{self, Rename};
use crate::db::users::{self, UserRow};
use crate::db::{admin_reads, announcements, audit, deletion, invites, live, settings};
use crate::dto::{
    AdminAnnouncementDto, AdminReportDto, AdminSettingsDto, AdminStatsDto, AdminUserDetailDto,
    AdminUserDto, AnnouncementTone, AuditEntryDto, CreateAnnouncementRequest, CreateInviteRequest,
    DeleteUserRequest, FilterScanDto, InviteDto, OkDto, Page, PatchAdminUserRequest,
    PatchSettingsRequest, PurgedDto, RenameUserRequest, ReportStatus, ResolveReportRequest, Role,
    SetOrgRequest, UserFilter,
};
use crate::error::AppError;
use crate::state::AppState;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/stats", get(stats))
        .route("/admin/users", get(users_list))
        .route(
            "/admin/users/{reference}",
            get(user_detail).patch(patch_user).delete(delete_user),
        )
        .route("/admin/users/{reference}/posts", delete(purge_posts))
        .route("/admin/users/{reference}/rename", post(rename_user))
        .route("/admin/users/{reference}/org", patch(set_org))
        .route("/admin/reports", get(reports))
        .route("/admin/reports/{id}/resolve", post(resolve_report))
        .route("/admin/filter/scan", get(filter_scan))
        .route("/admin/settings", get(get_settings).patch(patch_settings))
        .route("/admin/audit", get(audit_log))
        .route(
            "/admin/announcements",
            get(list_announcements).post(create_announcement),
        )
        .route("/admin/announcements/{id}", delete(end_announcement))
        .route("/admin/invites", get(list_invites).post(create_invite))
        .route("/admin/invites/{code}", delete(revoke_invite))
}

fn is_admin(auth: &Auth) -> bool {
    auth.user.role() == Role::Admin
}

async fn reload(state: &AppState, id: Uuid) -> Result<AdminUserDto, AppError> {
    let row = users::by_id(state.db(), id)
        .await?
        .ok_or(AppError::Missing("User not found."))?;
    admin_reads::admin_user(state.db(), row).await
}

async fn stats(
    State(state): State<AppState>,
    Staff(_): Staff,
) -> Result<Json<AdminStatsDto>, AppError> {
    Ok(Json(admin_reads::stats(state.db()).await?))
}

#[derive(Deserialize)]
struct UsersQuery {
    q: Option<String>,
    filter: Option<UserFilter>,
    before: Option<String>,
    limit: Option<u32>,
}

async fn users_list(
    State(state): State<AppState>,
    Staff(_): Staff,
    ApiQuery(query): ApiQuery<UsersQuery>,
) -> Result<Json<Page<AdminUserDto>>, AppError> {
    let paging = PageQuery {
        before: query.before,
        limit: query.limit,
    };
    let page = admin_reads::users_page(
        state.db(),
        query.filter.unwrap_or(UserFilter::Local),
        query.q.as_deref().unwrap_or_default(),
        paging.offset()?,
        paging.limit()?,
    )
    .await?;
    Ok(Json(page))
}

async fn user_detail(
    State(state): State<AppState>,
    Staff(_): Staff,
    Path(reference): Path<String>,
) -> Result<Json<AdminUserDetailDto>, AppError> {
    let row = admin_reads::find_user(state.db(), &reference).await?;
    Ok(Json(admin_reads::user_detail(state.db(), row).await?))
}

/// Only an admin may change an admin account (verification, name, organization flag).
fn spare_admin(auth: &Auth, target: &UserRow) -> Result<(), AppError> {
    if target.role() == Role::Admin && !is_admin(auth) {
        return Err(AppError::forbidden(
            "Only admins can change admin accounts.",
        ));
    }
    Ok(())
}

/// Refuses to suspend or demote the last active admin. Locking every active admin row makes two
/// admins who act on each other at once take turns, so the second one sees the first one's result.
async fn keep_one_admin(tx: &mut PgConnection, target: Uuid) -> Result<(), AppError> {
    let admins: Vec<Uuid> = sqlx::query_scalar(
        "select id from users
          where role = 'admin' and domain is null and deleted_at is null and suspended_at is null
          order by id for update",
    )
    .fetch_all(tx)
    .await?;
    if admins.iter().all(|id| *id == target) {
        return Err(AppError::bad_request(
            "You can\u{2019}t remove the last active admin.",
        ));
    }
    Ok(())
}
async fn patch_user(
    State(state): State<AppState>,
    Staff(auth): Staff,
    Path(reference): Path<String>,
    ApiJson(body): ApiJson<PatchAdminUserRequest>,
) -> Result<Json<AdminUserDto>, AppError> {
    let target = admin_reads::find_user(state.db(), &reference).await?;
    if target.deleted_at.is_some() {
        return Err(AppError::bad_request("That account was deleted."));
    }
    let me = &auth.user;
    if body.verified.is_some() {
        spare_admin(&auth, &target)?;
    }
    if body.suspended.is_some() {
        if target.id == me.id {
            return Err(AppError::bad_request("You can\u{2019}t suspend yourself."));
        }
        if target.role() == Role::Admin && !is_admin(&auth) {
            return Err(AppError::forbidden("Only admins can suspend admins."));
        }
    }
    if let Some(role) = body.role {
        if !is_admin(&auth) {
            return Err(AppError::forbidden("Only admins can change roles."));
        }
        if target.domain.is_some() || target.password_hash.is_none() {
            return Err(AppError::bad_request(
                "Only local accounts can have a role.",
            ));
        }
        if target.id == me.id && role != Role::Admin {
            return Err(AppError::bad_request(
                "You can\u{2019}t remove your own admin role.",
            ));
        }
    }
    let mut tx = state.db().begin().await?;
    let demotes = body.role.is_some_and(|role| role != Role::Admin);
    if target.role() == Role::Admin && (body.suspended == Some(true) || demotes) {
        keep_one_admin(&mut tx, target.id).await?;
    }
    if let Some(on) = body.verified {
        moderation::set_verified(&mut tx, Some(me.id), &target, on).await?;
    }
    if let Some(on) = body.suspended {
        moderation::set_suspended(&mut tx, Some(me.id), &target, on, "").await?;
    }
    if let Some(role) = body.role {
        moderation::set_role(&mut tx, Some(me.id), &target, role).await?;
    }
    tx.commit().await?;
    Ok(Json(reload(&state, target.id).await?))
}

async fn purge_posts(
    State(state): State<AppState>,
    Staff(auth): Staff,
    Path(reference): Path<String>,
) -> Result<Json<PurgedDto>, AppError> {
    if !is_admin(&auth) {
        return Err(AppError::forbidden("Only admins can do this."));
    }
    let target = admin_reads::find_user(state.db(), &reference).await?;
    let mut tx = state.db().begin().await?;
    let (deleted, removal) =
        moderation::purge_posts(&mut tx, state.config(), state.hub(), auth.user.id, &target)
            .await?;
    tx.commit().await?;
    live::publish_removal(state.hub(), removal);
    Ok(Json(PurgedDto { deleted }))
}

async fn delete_user(
    State(state): State<AppState>,
    Staff(auth): Staff,
    Path(reference): Path<String>,
    ApiJsonOrDefault(body): ApiJsonOrDefault<DeleteUserRequest>,
) -> Result<Json<OkDto>, AppError> {
    if !is_admin(&auth) {
        return Err(AppError::forbidden("Only admins can delete accounts."));
    }
    let target = admin_reads::find_user(state.db(), &reference).await?;
    if target.id == auth.user.id {
        return Err(AppError::bad_request(
            "Delete your own account from Settings \u{2192} Account.",
        ));
    }
    if target.role() == Role::Admin {
        return Err(AppError::forbidden("Remove their admin role first."));
    }
    if target.deleted_at.is_some() {
        return Err(AppError::bad_request("Already deleted."));
    }
    let reason: String = body
        .reason
        .as_deref()
        .unwrap_or_default()
        .trim()
        .chars()
        .take(200)
        .collect();
    deletion::delete_account_as_staff(state.db(), state.config(), auth.user.id, &target, &reason)
        .await?;
    Ok(Json(OkDto { ok: true }))
}

async fn rename_user(
    State(state): State<AppState>,
    Staff(auth): Staff,
    Path(reference): Path<String>,
    ApiJson(body): ApiJson<RenameUserRequest>,
) -> Result<Json<AdminUserDto>, AppError> {
    let target = admin_reads::find_user(state.db(), &reference).await?;
    spare_admin(&auth, &target)?;
    if target.domain.is_some() {
        return Err(AppError::bad_request(
            "Remote accounts are renamed on their own instance.",
        ));
    }
    if target.deleted_at.is_some() {
        return Err(AppError::bad_request("That account was deleted."));
    }
    let not_allowed = || AppError::bad_request("That handle isn\u{2019}t allowed either.");
    let handle = match body.handle.as_deref() {
        None => None,
        Some(raw) => {
            let handle = match parse_handle(raw) {
                Ok(handle) => handle.to_string(),
                Err(HandleError::Reserved) => return Err(not_allowed()),
                Err(err) => return Err(AppError::bad_request(err.user_message())),
            };
            let filter = settings::content_filter(state.db()).await?;
            if filter.is_reserved(&handle) || filter.blocks_handle(&handle) {
                return Err(not_allowed());
            }
            Some(handle)
        }
    };
    if handle.is_none() && !body.reset_name && !body.reset_bio {
        return Err(AppError::bad_request("Nothing to change."));
    }
    let change = Rename {
        handle,
        reset_name: body.reset_name,
        reset_bio: body.reset_bio,
    };
    let mut tx = state.db().begin().await?;
    moderation::rename(&mut tx, auth.user.id, &target, &change).await?;
    tx.commit().await?;
    Ok(Json(reload(&state, target.id).await?))
}

async fn set_org(
    State(state): State<AppState>,
    Staff(auth): Staff,
    Path(reference): Path<String>,
    ApiJson(body): ApiJson<SetOrgRequest>,
) -> Result<Json<AdminUserDto>, AppError> {
    let target = admin_reads::find_user(state.db(), &reference).await?;
    if target.domain.is_some() || target.deleted_at.is_some() {
        return Err(AppError::Missing(
            "Only local accounts can be organizations.",
        ));
    }
    spare_admin(&auth, &target)?;
    let mut tx = state.db().begin().await?;
    moderation::set_org(&mut tx, Some(auth.user.id), &target, body.org).await?;
    tx.commit().await?;
    Ok(Json(reload(&state, target.id).await?))
}

#[derive(Deserialize)]
struct ReportsQuery {
    status: Option<ReportStatus>,
    before: Option<String>,
    limit: Option<u32>,
}

async fn reports(
    State(state): State<AppState>,
    Staff(_): Staff,
    ApiQuery(query): ApiQuery<ReportsQuery>,
) -> Result<Json<Page<AdminReportDto>>, AppError> {
    let paging = PageQuery {
        before: query.before,
        limit: query.limit,
    };
    let page = admin_reads::reports_page(
        state.db(),
        query.status.unwrap_or(ReportStatus::Open),
        paging.id_cursor()?,
        paging.limit()?,
    )
    .await?;
    Ok(Json(page))
}

async fn resolve_report(
    State(state): State<AppState>,
    Staff(auth): Staff,
    Path(id): Path<String>,
    ApiJson(body): ApiJson<ResolveReportRequest>,
) -> Result<Json<OkDto>, AppError> {
    let id = parse_id(&id, "Report not found.")?;
    let mut tx = state.db().begin().await?;
    let removal = moderation::resolve_report(
        &mut tx,
        state.config(),
        state.hub(),
        auth.user.id,
        id,
        body.action,
    )
    .await?;
    tx.commit().await?;
    live::publish_removal(state.hub(), removal);
    Ok(Json(OkDto { ok: true }))
}

async fn filter_scan(
    State(state): State<AppState>,
    Staff(auth): Staff,
) -> Result<Json<FilterScanDto>, AppError> {
    // The scan reads every account; one staff member may run it a few times a minute.
    state.limits().check_scan(auth.user.id)?;
    let filter = settings::content_filter(state.db()).await?;
    Ok(Json(admin_reads::filter_scan(state.db(), &filter).await?))
}

async fn get_settings(
    State(state): State<AppState>,
    Admin(_): Admin,
) -> Result<Json<AdminSettingsDto>, AppError> {
    Ok(Json(
        settings::admin_view(state.db(), state.config().domain()).await?,
    ))
}

/// `local@domain.tld` with something on each side of a dot in the domain.
fn valid_email(email: &str) -> bool {
    if email.chars().any(|c| c.is_whitespace()) {
        return false;
    }
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    let domain: Vec<char> = domain.chars().collect();
    !local.is_empty()
        && !domain.contains(&'@')
        && domain.len() >= 3
        && domain[1..domain.len() - 1].contains(&'.')
}

/// Checks a settings patch and returns the changes with the names the audit row lists.
fn validate_settings(
    body: PatchSettingsRequest,
) -> Result<(settings::Changes, Vec<&'static str>), AppError> {
    let mut changes = settings::Changes::default();
    let mut names = Vec::new();
    if let Some(name) = body.name {
        changes.name = Some(text_field(&name, "Instance name", 1, 40)?);
        names.push("name");
    }
    if let Some(description) = body.description {
        changes.description = Some(text_field(&description, "Description", 0, 300)?);
        names.push("description");
    }
    if let Some(contact) = body.contact {
        let email = text_field(&contact, "Contact email", 0, 120)?;
        if !email.is_empty() && !valid_email(&email) {
            return Err(AppError::bad_request("Enter a valid email."));
        }
        changes.contact = Some(email);
        names.push("contact");
    }
    if let Some(rules) = body.rules {
        let rules: Vec<String> = rules
            .iter()
            .map(|rule| rule.trim().to_owned())
            .filter(|rule| !rule.is_empty())
            .collect();
        if rules.join("\n").chars().count() > 3000 {
            return Err(AppError::bad_request(
                "Rules must be at most 3000 characters.",
            ));
        }
        changes.rules = Some(rules);
        names.push("rules");
    }
    if let Some(age) = body.min_age {
        let age = u8::try_from(age)
            .ok()
            .filter(|age| (13..=21).contains(age))
            .ok_or_else(|| AppError::bad_request("Minimum age must be 13\u{2013}21."))?;
        changes.min_age = Some(age);
        names.push("minAge");
    }
    if let Some(mode) = body.signups {
        changes.signups = Some(mode);
        names.push("signups");
    }
    if let Some(mut filter) = body.filter {
        filter.blocked = text_field(&filter.blocked, "Blocked words", 0, 5000)?;
        filter.allowed = text_field(&filter.allowed, "Allowed words", 0, 5000)?;
        filter.reserved = text_field(&filter.reserved, "Reserved handles", 0, 5000)?;
        changes.filter = Some(filter);
        names.push("filter");
    }
    Ok((changes, names))
}

async fn patch_settings(
    State(state): State<AppState>,
    Admin(auth): Admin,
    ApiJson(body): ApiJson<PatchSettingsRequest>,
) -> Result<Json<AdminSettingsDto>, AppError> {
    let (changes, names) = validate_settings(body)?;
    let mut tx = state.db().begin().await?;
    settings::apply(&mut tx, &changes).await?;
    if !names.is_empty() {
        audit::record(
            &mut tx,
            audit::Entry::new(Some(auth.user.id), "instance.settings").text(names.join(", ")),
        )
        .await?;
    }
    tx.commit().await?;
    Ok(Json(
        settings::admin_view(state.db(), state.config().domain()).await?,
    ))
}

async fn audit_log(
    State(state): State<AppState>,
    Admin(_): Admin,
    ApiQuery(paging): ApiQuery<PageQuery>,
) -> Result<Json<Page<AuditEntryDto>>, AppError> {
    let limit: Limit = paging.limit()?;
    Ok(Json(
        audit::list(state.db(), paging.id_cursor()?, limit).await?,
    ))
}

async fn list_announcements(
    State(state): State<AppState>,
    Admin(_): Admin,
) -> Result<Json<Vec<AdminAnnouncementDto>>, AppError> {
    Ok(Json(announcements::list(state.db()).await?))
}

/// A link that starts with `https://`, `http://` or a single `/` (not `//` or `/\`, which a
/// browser reads as another site) and has no spaces, `<`, `>` or quotes.
fn valid_link(link: &str) -> bool {
    let lower = link.to_ascii_lowercase();
    let local = lower.starts_with('/') && !lower.starts_with("//") && !lower.starts_with("/\\");
    (lower.starts_with("https://") || lower.starts_with("http://") || local)
        && !link
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '<' | '>' | '"'))
}

async fn create_announcement(
    State(state): State<AppState>,
    Admin(auth): Admin,
    ApiJson(body): ApiJson<CreateAnnouncementRequest>,
) -> Result<Json<AdminAnnouncementDto>, AppError> {
    let text = text_field(&body.text, "Announcement", 1, 280)?;
    let link = text_field(body.link.as_deref().unwrap_or_default(), "Link", 0, 200)?;
    if !link.is_empty() && !valid_link(&link) {
        return Err(AppError::bad_request(
            "Links must start with http://, https:// or /.",
        ));
    }
    let new = announcements::New {
        text,
        tone: body.tone.unwrap_or(AnnouncementTone::Info),
        link,
        days: body.days.unwrap_or(0).clamp(0, announcements::MAX_DAYS),
    };
    let mut tx = state.db().begin().await?;
    let created = announcements::create(&mut tx, auth.user.id, new).await?;
    tx.commit().await?;
    Ok(Json(created))
}

async fn end_announcement(
    State(state): State<AppState>,
    Admin(auth): Admin,
    Path(id): Path<String>,
) -> Result<Json<OkDto>, AppError> {
    let id = parse_id(&id, "Announcement not found.")?;
    let mut tx = state.db().begin().await?;
    announcements::end(&mut tx, auth.user.id, id).await?;
    tx.commit().await?;
    Ok(Json(OkDto { ok: true }))
}

async fn list_invites(
    State(state): State<AppState>,
    Admin(_): Admin,
) -> Result<Json<Vec<InviteDto>>, AppError> {
    Ok(Json(invites::list(state.db()).await?))
}

async fn create_invite(
    State(state): State<AppState>,
    Admin(auth): Admin,
    ApiJsonOrDefault(body): ApiJsonOrDefault<CreateInviteRequest>,
) -> Result<Json<InviteDto>, AppError> {
    let note = text_field(body.note.as_deref().unwrap_or_default(), "Note", 0, 80)?;
    let new = invites::New {
        max_uses: body.max_uses.unwrap_or(1).clamp(1, 1000),
        days: body.days.unwrap_or(0).clamp(0, 365),
        note,
    };
    let mut tx = state.db().begin().await?;
    let created = invites::create(&mut tx, auth.user.id, new).await?;
    tx.commit().await?;
    Ok(Json(created))
}

async fn revoke_invite(
    State(state): State<AppState>,
    Admin(auth): Admin,
    Path(code): Path<String>,
) -> Result<Json<OkDto>, AppError> {
    let code = invites::normalize(&code).ok_or(AppError::Missing("Invite not found."))?;
    let mut tx = state.db().begin().await?;
    invites::revoke(&mut tx, auth.user.id, &code).await?;
    tx.commit().await?;
    Ok(Json(OkDto { ok: true }))
}
