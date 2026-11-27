//! What the admin console reads: account lists, the report queue, one account's detail, the
//! word-filter scan and the overview numbers. The writes are in [`super::moderation`].

use std::collections::HashMap;

use futures_util::TryStreamExt as _;
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use tuita_core::search::like_escape;
use uuid::Uuid;

use super::users::{self, COLS, UserRow, user_dto};
use super::{audit, emoji, posts};
use crate::cursor::{Limit, page};
use crate::dto::{
    AdminReportDto, AdminStatsDto, AdminUserDetailDto, AdminUserDto, DayStatsDto, FilterField,
    FilterScanDto, FlaggedUserDto, Page, Relation, ReportAction, ReportReason, ReportStatus,
    StatsPostsDto, StatsUsersDto, UserFilter, UserReportDto,
};
use crate::error::AppError;
use crate::filter::{ContentFilter, REASON};

/// Longest search text the user list takes, in characters.
const MAX_QUERY_CHARS: usize = 100;
/// Most accounts the word-filter scan returns.
const SCAN_LIMIT: usize = 200;

/// Finds the account a `{ref}` path segment names: `id:<uuid>` or a local handle, whatever its
/// state (suspended and deleted accounts included).
///
/// # Errors
///
/// `404 User not found.`; database failures.
pub async fn find_user(db: &PgPool, reference: &str) -> Result<UserRow, AppError> {
    const NOT_FOUND: &str = "User not found.";
    if let Some(id) = reference.strip_prefix("id:") {
        let id = Uuid::parse_str(id).map_err(|_| AppError::Missing(NOT_FOUND))?;
        return users::by_id(db, id)
            .await?
            .ok_or(AppError::Missing(NOT_FOUND));
    }
    let handle = reference.strip_prefix('@').unwrap_or(reference);
    let sql = format!(
        "select {COLS} from users u where lower(u.handle) = lower($1) and u.domain is null"
    );
    sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(handle)
        .fetch_optional(db)
        .await?
        .ok_or(AppError::Missing(NOT_FOUND))
}

#[derive(FromRow)]
struct Extras {
    id: Uuid,
    open_reports: i32,
    last_active_at: Option<OffsetDateTime>,
}

/// The staff view of `rows`, in the same order.
///
/// # Errors
///
/// Database failures.
pub async fn admin_users(db: &PgPool, rows: Vec<UserRow>) -> Result<Vec<AdminUserDto>, AppError> {
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
    let extras: Vec<Extras> = sqlx::query_as(
        "select t.id,
                (select count(*) from reports r where r.target_id = t.id and r.status = 'open')::int4
                    as open_reports,
                (select max(s.last_used_at) from sessions s where s.user_id = t.id) as last_active_at
           from unnest($1::uuid[]) as t(id)",
    )
    .bind(&ids)
    .fetch_all(db)
    .await?;
    let extras: HashMap<Uuid, Extras> = extras.into_iter().map(|e| (e.id, e)).collect();
    let mut items: Vec<AdminUserDto> = rows
        .into_iter()
        .map(|row| {
            let extra = extras.get(&row.id);
            AdminUserDto {
                open_reports: extra.map_or(0, |e| e.open_reports),
                last_active_at: extra.and_then(|e| e.last_active_at),
                role: row.role(),
                deleted: row.deleted_at.is_some(),
                user: user_dto(&row, Relation::default()),
            }
        })
        .collect();
    let mut cards: Vec<_> = items.iter_mut().map(|item| &mut item.user).collect();
    emoji::hydrate(db, &mut cards).await?;
    Ok(items)
}

/// The staff view of one account.
///
/// # Errors
///
/// Database failures.
pub async fn admin_user(db: &PgPool, row: UserRow) -> Result<AdminUserDto, AppError> {
    admin_users(db, vec![row])
        .await?
        .pop()
        .ok_or(AppError::Missing("User not found."))
}

fn user_condition(filter: UserFilter) -> &'static str {
    match filter {
        UserFilter::Local => "u.domain is null and u.password_hash is not null",
        UserFilter::Remote => "u.domain is not null",
        UserFilter::Staff => "u.role in ('admin', 'moderator')",
        UserFilter::Verified => "u.verified",
        UserFilter::Suspended => "u.suspended_at is not null",
        UserFilter::All => "true",
    }
}

/// One page of accounts that are not deleted, newest first. `offset` is the page cursor.
///
/// # Errors
///
/// Database failures.
pub async fn users_page(
    db: &PgPool,
    filter: UserFilter,
    needle: &str,
    offset: i64,
    limit: Limit,
) -> Result<Page<AdminUserDto>, AppError> {
    let needle: String = needle
        .trim()
        .trim_start_matches('@')
        .chars()
        .take(MAX_QUERY_CHARS)
        .collect();
    let condition = user_condition(filter);
    let sql = format!(
        "select {COLS} from users u
          where {condition} and u.deleted_at is null and not (u.domain is null and u.handle = 'tuita')
            and ($1 = '' or u.handle ilike $2 or u.display_name ilike $2)
          order by u.created_at desc, u.id desc limit $3 offset $4"
    );
    let rows: Vec<UserRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(&needle)
        .bind(format!("%{}%", like_escape(&needle)))
        .bind(limit.fetch())
        .bind(offset)
        .fetch_all(db)
        .await?;
    let more = rows.len() > limit.size();
    let rows: Vec<UserRow> = rows.into_iter().take(limit.size()).collect();
    let items = admin_users(db, rows).await?;
    let next = more.then(|| offset.saturating_add(i64::from(limit.get())).to_string());
    Ok(Page { items, next })
}

#[derive(FromRow)]
struct ReportRow {
    id: Uuid,
    reason: String,
    note: String,
    status: String,
    resolution: Option<String>,
    created_at: OffsetDateTime,
    resolved_at: Option<OffsetDateTime>,
    reporter_id: Uuid,
    target_id: Uuid,
    post_id: Option<Uuid>,
    resolved_by: Option<Uuid>,
    same_target_open: i32,
}

/// One page of reports with the given status, newest first. `before` is a report id.
///
/// # Errors
///
/// Database failures.
pub async fn reports_page(
    db: &PgPool,
    status: ReportStatus,
    before: Option<Uuid>,
    limit: Limit,
) -> Result<Page<AdminReportDto>, AppError> {
    let rows: Vec<ReportRow> = sqlx::query_as(
        "select r.id, r.reason, r.note, r.status, r.resolution, r.created_at, r.resolved_at,
                r.reporter_id, r.target_id, r.post_id, r.resolved_by,
                (select count(*) from reports o
                  where o.target_id = r.target_id and o.status = 'open')::int4 as same_target_open
           from reports r
          where r.status = $1 and ($2::uuid is null or r.id < $2)
          order by r.id desc limit $3",
    )
    .bind(status.as_str())
    .bind(before)
    .bind(limit.fetch())
    .fetch_all(db)
    .await?;

    let mut user_ids: Vec<Uuid> = Vec::new();
    for row in &rows {
        user_ids.extend([row.reporter_id, row.target_id]);
        user_ids.extend(row.resolved_by);
    }
    user_ids.sort_unstable();
    user_ids.dedup();
    let cards = users::refs(db, &user_ids).await?;
    let suspended: Vec<Uuid> =
        sqlx::query_scalar("select id from users where id = any($1) and suspended_at is not null")
            .bind(&user_ids)
            .fetch_all(db)
            .await?;
    let post_ids: Vec<Uuid> = rows.iter().filter_map(|row| row.post_id).collect();
    // A nil viewer has no blocks, mutes or muted words: staff see what was reported.
    let mut posts: HashMap<Uuid, _> = posts::load(db, Uuid::nil(), &post_ids, None)
        .await?
        .into_iter()
        .map(|post| (post.id, post))
        .collect();

    let items = rows
        .into_iter()
        .filter_map(|row| {
            Some(AdminReportDto {
                reason: ReportReason::parse(&row.reason).unwrap_or(ReportReason::Other),
                status: ReportStatus::parse(&row.status).unwrap_or(ReportStatus::Open),
                resolution: row.resolution.as_deref().and_then(ReportAction::parse),
                reporter: cards.get(&row.reporter_id).cloned(),
                target: cards.get(&row.target_id).cloned()?,
                target_suspended: suspended.contains(&row.target_id),
                resolved_by: row.resolved_by.and_then(|id| cards.get(&id).cloned()),
                post: row.post_id.and_then(|id| posts.remove(&id)),
                id: row.id,
                note: row.note,
                created_at: row.created_at,
                resolved_at: row.resolved_at,
                same_target_open: row.same_target_open,
            })
        })
        .collect::<Vec<_>>();
    Ok(page(items, limit, |report: &AdminReportDto| {
        report.id.to_string()
    }))
}

#[derive(FromRow)]
struct UserReportRow {
    id: Uuid,
    reason: String,
    note: String,
    status: String,
    resolution: Option<String>,
    reporter: Option<String>,
    created_at: OffsetDateTime,
}

/// Everything staff see on one account's page.
///
/// # Errors
///
/// Database failures.
pub async fn user_detail(db: &PgPool, row: UserRow) -> Result<AdminUserDetailDto, AppError> {
    let id = row.id;
    let invited_with =
        sqlx::query_scalar::<_, Option<String>>("select invited_with from users where id = $1")
            .bind(id)
            .fetch_one(db)
            .await?;
    let (passkeys, sessions, tokens, following): (i32, i32, i32, i32) = sqlx::query_as(
        "select (select count(*) from passkeys where user_id = $1)::int4,
                (select count(*) from sessions where user_id = $1 and expires_at > now())::int4,
                (select count(*) from api_tokens where user_id = $1)::int4,
                (select count(*) from follows where follower_id = $1)::int4",
    )
    .bind(id)
    .fetch_one(db)
    .await?;
    let recent: Vec<Uuid> = sqlx::query_scalar(
        "select id from posts where author_id = $1 and deleted_at is null order by id desc limit 5",
    )
    .bind(id)
    .fetch_all(db)
    .await?;
    let reports: Vec<UserReportRow> = sqlx::query_as(
        "select r.id, r.reason, r.note, r.status, r.resolution, x.handle as reporter, r.created_at
           from reports r left join users x on x.id = r.reporter_id
          where r.target_id = $1 order by r.id desc limit 20",
    )
    .bind(id)
    .fetch_all(db)
    .await?;
    Ok(AdminUserDetailDto {
        user: admin_user(db, row).await?,
        invited_with,
        passkeys,
        sessions,
        tokens,
        following,
        posts: posts::load(db, Uuid::nil(), &recent, None).await?,
        reports: reports
            .into_iter()
            .map(|report| UserReportDto {
                id: report.id,
                reason: ReportReason::parse(&report.reason).unwrap_or(ReportReason::Other),
                note: report.note,
                status: ReportStatus::parse(&report.status).unwrap_or(ReportStatus::Open),
                resolution: report.resolution.as_deref().and_then(ReportAction::parse),
                reporter: report.reporter,
                created_at: report.created_at,
            })
            .collect(),
        history: audit::history(db, id, 20).await?,
    })
}

/// Accounts whose handle, name, location or bio trip the word filter, newest first.
///
/// # Errors
///
/// Database failures.
pub async fn filter_scan(db: &PgPool, filter: &ContentFilter) -> Result<FilterScanDto, AppError> {
    let sql = format!(
        "select {COLS} from users u
          where u.deleted_at is null and (u.domain is not null or u.password_hash is not null)
          order by u.created_at desc, u.id desc"
    );
    let mut flagged: Vec<(UserRow, FilterField, &'static str)> = Vec::new();
    {
        let mut rows = sqlx::query_as::<_, UserRow>(sqlx::AssertSqlSafe(sql)).fetch(db);
        while let Some(row) = rows.try_next().await? {
            if let Some((field, reason)) = first_hit(filter, &row) {
                flagged.push((row, field, reason));
                if flagged.len() >= SCAN_LIMIT {
                    break;
                }
            }
        }
    }
    let rows: Vec<UserRow> = flagged.iter().map(|(row, _, _)| row.clone()).collect();
    let views = admin_users(db, rows).await?;
    Ok(FilterScanDto {
        flagged: views
            .into_iter()
            .zip(flagged)
            .map(|(user, (_, field, reason))| FlaggedUserDto {
                user,
                field,
                reason: reason.to_owned(),
            })
            .collect(),
    })
}

fn first_hit(filter: &ContentFilter, row: &UserRow) -> Option<(FilterField, &'static str)> {
    let checks = [
        (FilterField::Handle, filter.blocks_handle(&row.handle)),
        (FilterField::Name, filter.blocks_text(&row.display_name)),
        (FilterField::Location, filter.blocks_text(&row.location)),
        (FilterField::Bio, filter.blocks_text(&row.bio)),
    ];
    checks
        .into_iter()
        .find_map(|(field, hit)| hit.then_some((field, REASON)))
}

/// The overview numbers.
///
/// # Errors
///
/// Database failures.
pub async fn stats(db: &PgPool) -> Result<AdminStatsDto, AppError> {
    let (local, demo, suspended, active7d): (i32, i32, i32, i32) = sqlx::query_as(
        "select (select count(*) from users
                  where domain is null and deleted_at is null and password_hash is not null)::int4,
                (select count(*) from users
                  where domain is null and deleted_at is null and password_hash is null and handle <> 'tuita')::int4,
                (select count(*) from users where suspended_at is not null and deleted_at is null)::int4,
                (select count(distinct user_id) from sessions
                  where last_used_at > now() - interval '7 days')::int4",
    )
    .fetch_one(db)
    .await?;
    let (total, last24h, reports_open): (i32, i32, i32) = sqlx::query_as(
        "select (select count(*) from posts where deleted_at is null)::int4,
                (select count(*) from posts
                  where deleted_at is null and created_at > now() - interval '24 hours')::int4,
                (select count(*) from reports where status = 'open')::int4",
    )
    .fetch_one(db)
    .await?;
    let days: Vec<(String, i32, i32, i32, i32)> = sqlx::query_as(
        "select to_char(g.d, 'YYYY-MM-DD'),
                (select count(*) from users
                  where domain is null and created_at >= g.d at time zone 'UTC'
                    and created_at < (g.d + interval '1 day') at time zone 'UTC')::int4,
                (select count(*) from posts
                  where created_at >= g.d at time zone 'UTC'
                    and created_at < (g.d + interval '1 day') at time zone 'UTC')::int4,
                (select count(*) from likes
                  where created_at >= g.d at time zone 'UTC'
                    and created_at < (g.d + interval '1 day') at time zone 'UTC')::int4,
                (select count(*) from reports
                  where created_at >= g.d at time zone 'UTC'
                    and created_at < (g.d + interval '1 day') at time zone 'UTC')::int4
           from generate_series(((now() at time zone 'UTC')::date - 13)::timestamp,
                                (now() at time zone 'UTC')::date::timestamp, interval '1 day') as g(d)
          order by g.d",
    )
    .fetch_all(db)
    .await?;
    let cells: Vec<(i32, i32, i32)> = sqlx::query_as(
        "select (extract(isodow from t at time zone 'UTC')::int - 1) as weekday,
                extract(hour from t at time zone 'UTC')::int as hour, count(*)::int4
           from (select created_at as t from posts where created_at >= now() - interval '28 days'
                 union all
                 select created_at from likes where created_at >= now() - interval '28 days') x
          group by 1, 2",
    )
    .fetch_all(db)
    .await?;
    let mut heatmap = vec![vec![0_i32; 24]; 7];
    for (weekday, hour, count) in cells {
        let slot = usize::try_from(weekday)
            .ok()
            .and_then(|row| heatmap.get_mut(row))
            .and_then(|row| row.get_mut(usize::try_from(hour).unwrap_or(usize::MAX)));
        if let Some(slot) = slot {
            *slot = count;
        }
    }
    Ok(AdminStatsDto {
        users: StatsUsersDto {
            local,
            demo,
            suspended,
            active7d,
        },
        posts: StatsPostsDto { total, last24h },
        reports_open,
        days: days
            .into_iter()
            .map(|(day, signups, posts, likes, reports)| DayStatsDto {
                day,
                signups,
                posts,
                likes,
                reports,
            })
            .collect(),
        heatmap,
    })
}
