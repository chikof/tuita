//! Search and discovery: people, posts, hashtags and trends.

use sqlx::{PgPool, Postgres, QueryBuilder};
use time::Time;
use tuita_core::search::{Has, Query, like_escape};
use uuid::Uuid;

use super::users::{COLS, UserRow};
use super::{discoverable_post, silenced_author, visible_author, word_visible};
use crate::cursor::{Limit, page};
use crate::dto::{Page, TrendDto};
use crate::error::AppError;

/// Accounts matching `needle` (already trimmed, without a leading `@`) that `viewer` may see.
///
/// The handle and display name are searched, and the bio too when `with_bio`. Accounts that opted
/// out of discovery only match their exact handle. Exact handle first, then verified, then the
/// most followed.
///
/// # Errors
///
/// Database failures.
pub async fn people(
    db: &PgPool,
    viewer: Uuid,
    needle: &str,
    with_bio: bool,
    offset: i64,
    limit: i64,
) -> Result<Vec<UserRow>, AppError> {
    let visible = visible_author("u", "$1");
    let silenced = silenced_author("u");
    let bio = if with_bio { "or u.bio ilike $2" } else { "" };
    let sql = format!(
        "select {COLS} from users u
         where (u.domain is null or not {silenced}) and {visible}
           and not (u.domain is null and lower(u.handle) = 'tuita')
           and (u.handle ilike $2 or u.display_name ilike $2 {bio})
           and (coalesce(u.discoverable, true) or lower(u.handle) = lower($3))
         order by (lower(u.handle) = lower($3)) desc, u.verified desc, u.followers_count desc,
                  u.id asc
         limit $4 offset $5"
    );
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(format!("%{}%", like_escape(needle)))
        .bind(needle)
        .bind(limit)
        .bind(offset)
        .fetch_all(db)
        .await?)
}

/// One page of post ids matching `query`, newest first, written by authors `viewer` may see.
/// With `media_only` the posts must carry media.
///
/// # Errors
///
/// Database failures.
pub async fn posts(
    db: &PgPool,
    viewer: Uuid,
    query: &Query,
    media_only: bool,
    before: Option<Uuid>,
    limit: Limit,
) -> Result<Page<Uuid>, AppError> {
    let ids = post_ids(db, viewer, query, media_only, before, 0, limit.fetch()).await?;
    Ok(page(ids, limit, Uuid::to_string))
}

/// [`posts`] without paging: up to `fetch` ids after skipping `offset` of them, for clients that
/// page by offset.
///
/// # Errors
///
/// Database failures.
pub async fn post_ids(
    db: &PgPool,
    viewer: Uuid,
    query: &Query,
    media_only: bool,
    before: Option<Uuid>,
    offset: i64,
    fetch: i64,
) -> Result<Vec<Uuid>, AppError> {
    let mut sql = QueryBuilder::<Postgres>::new(
        "select p.id from posts p join users a on a.id = p.author_id where p.deleted_at is null and ",
    );
    // The first bind is `$1`, which `visible_author` refers to.
    sql.push_bind(viewer).push("::uuid is not null and ");
    sql.push(visible_author("a", "$1"));
    sql.push(" and ").push(word_visible("p", "$1"));
    sql.push(" and ").push(discoverable_post("p", "a"));

    for term in &query.terms {
        sql.push(" and p.text ilike ").push_bind(contains(term));
    }
    for term in &query.excluded {
        sql.push(" and p.text not ilike ").push_bind(contains(term));
    }
    let mut from: Vec<String> = query.from.iter().map(|h| h.to_lowercase()).collect();
    from.sort();
    from.dedup();
    match from.as_slice() {
        [] => {}
        [handle] => {
            sql.push(" and lower(a.handle) = ")
                .push_bind(handle.clone());
        }
        _ => {
            sql.push(" and false");
        }
    }
    for handle in &query.to {
        sql.push(
            " and (exists (select 1 from posts pp join users pu on pu.id = pp.author_id
                 where pp.id = p.reply_to_id and lower(pu.handle) = lower(",
        );
        sql.push_bind(handle.to_string())
            .push(")) or p.text ilike ");
        sql.push_bind(format!("%@{}%", like_escape(handle)))
            .push(")");
    }
    for has in &query.has {
        match has {
            Has::Media => {
                sql.push(" and exists (select 1 from post_media pm where pm.post_id = p.id)");
            }
            Has::Image => push_media_kind(&mut sql, "image"),
            Has::Video => push_media_kind(&mut sql, "video"),
            Has::Poll => {
                sql.push(" and exists (select 1 from polls pl where pl.post_id = p.id)");
            }
            Has::Links => {
                sql.push(" and p.text ilike '%http%'");
            }
        }
    }
    if let Some(since) = query.since {
        sql.push(" and p.created_at >= ")
            .push_bind(since.with_time(Time::MIDNIGHT).assume_utc());
    }
    // `next_day` is `None` on the last representable date; nothing lies beyond it, so no bound.
    if let Some(end) = query.until.and_then(|until| until.next_day()) {
        sql.push(" and p.created_at < ")
            .push_bind(end.with_time(Time::MIDNIGHT).assume_utc());
    }
    if media_only {
        sql.push(" and exists (select 1 from post_media pm where pm.post_id = p.id)");
    }
    if let Some(before) = before {
        sql.push(" and p.id < ").push_bind(before);
    }
    sql.push(" order by p.id desc limit ")
        .push_bind(fetch)
        .push(" offset ")
        .push_bind(offset);

    Ok(sql.build_query_scalar().fetch_all(db).await?)
}

fn contains(term: &str) -> String {
    format!("%{}%", like_escape(term))
}

fn push_media_kind(sql: &mut QueryBuilder<Postgres>, kind: &'static str) {
    sql.push(
        " and exists (select 1 from post_media pm join media m on m.id = pm.media_id
           where pm.post_id = p.id and m.kind = ",
    )
    .push_bind(kind)
    .push(")");
}

/// The most used tags that start with `prefix` (lower case, no `#`).
///
/// # Errors
///
/// Database failures.
pub async fn hashtags(db: &PgPool, prefix: &str) -> Result<Vec<TrendDto>, AppError> {
    let discoverable = discoverable_post("p", "a");
    let sql = format!(
        "select pt.tag, count(*) as posts
         from post_tags pt join posts p on p.id = pt.post_id join users a on a.id = p.author_id
         where p.deleted_at is null and pt.tag like $1 and {discoverable}
         group by pt.tag order by posts desc, pt.tag limit 6"
    );
    let rows: Vec<(String, i64)> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(format!("{}%", like_escape(prefix)))
        .fetch_all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|(tag, posts)| TrendDto { tag, posts })
        .collect())
}

/// How many of the most used tags are read before the letters-only filter.
const TREND_CANDIDATES: i64 = 40;
/// Trends shown.
const TRENDS: usize = 8;

/// The most used tags of the last 7 days. Numbers-only tags like `#2024` are not trends.
///
/// # Errors
///
/// Database failures.
pub async fn trends(db: &PgPool) -> Result<Vec<TrendDto>, AppError> {
    let discoverable = discoverable_post("p", "a");
    let sql = format!(
        "select pt.tag, count(*) as posts
         from post_tags pt join posts p on p.id = pt.post_id join users a on a.id = p.author_id
         where p.deleted_at is null and a.suspended_at is null and a.deleted_at is null
           and {discoverable} and p.created_at > now() - interval '7 days'
         group by pt.tag order by posts desc, pt.tag limit $1"
    );
    let rows: Vec<(String, i64)> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(TREND_CANDIDATES)
        .fetch_all(db)
        .await?;
    Ok(rows
        .into_iter()
        .filter(|(tag, _)| tag.chars().any(char::is_alphabetic))
        .take(TRENDS)
        .map(|(tag, posts)| TrendDto { tag, posts })
        .collect())
}

/// One page of post ids tagged `tag` (lower case), newest first, by authors `viewer` may see.
///
/// # Errors
///
/// Database failures.
pub async fn tag_posts(
    db: &PgPool,
    viewer: Uuid,
    tag: &str,
    before: Option<Uuid>,
    limit: Limit,
) -> Result<Page<Uuid>, AppError> {
    let visible = visible_author("a", "$1");
    let words = word_visible("p", "$1");
    let discoverable = discoverable_post("p", "a");
    let sql = format!(
        "select p.id from post_tags pt join posts p on p.id = pt.post_id
           join users a on a.id = p.author_id
         where pt.tag = $2 and p.deleted_at is null and {visible} and {words} and {discoverable}
           and ($3::uuid is null or p.id < $3)
         order by p.id desc limit $4"
    );
    let ids: Vec<Uuid> = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .bind(viewer)
        .bind(tag)
        .bind(before)
        .bind(limit.fetch())
        .fetch_all(db)
        .await?;
    Ok(page(ids, limit, Uuid::to_string))
}
