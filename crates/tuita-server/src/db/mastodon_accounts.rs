//! The queries behind the Mastodon account routes: accounts by id, the people on the other side
//! of a follow or a list, and the statuses of a profile.
//!
//! Mastodon pages by the id of the listed thing, and here that is an account id or a status id,
//! while tuita's own lists page by the row of the join table. [`edge_accounts`] therefore reads
//! the bounds from the join table itself: the cursor account's edge decides where the page starts.

use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use super::suspended_domain;
use super::users::{COLS, UserRow};
use crate::error::AppError;

/// The accounts among `ids` that a viewer may look at: not deleted, not suspended (unless
/// `staff`), not on a `suspend`-blocked instance, and not the `tuita` system account. Unknown
/// and hidden ids are absent.
///
/// # Errors
///
/// Database failures.
pub async fn visible_accounts(
    db: &PgPool,
    ids: &[Uuid],
    staff: bool,
) -> Result<Vec<UserRow>, AppError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let suspended = suspended_domain("u");
    let sql = format!(
        "select {COLS} from users u
         where u.id = any($1) and u.deleted_at is null and ($2 or u.suspended_at is null)
           and not (u.domain is null and lower(u.handle) = 'tuita') and not {suspended}"
    );
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(ids)
        .bind(staff)
        .fetch_all(db)
        .await?)
}

/// [`visible_accounts`] for one id.
///
/// # Errors
///
/// Database failures.
pub async fn visible_account(
    db: &PgPool,
    id: Uuid,
    staff: bool,
) -> Result<Option<UserRow>, AppError> {
    Ok(visible_accounts(db, &[id], staff).await?.pop())
}

/// Where a page starts and how far it reads: Mastodon's `max_id`, `since_id`/`min_id` and
/// `limit`, as `Paging` hands them out.
#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    /// Rows older than this.
    pub upper: Option<Uuid>,
    /// Rows newer than this.
    pub lower: Option<Uuid>,
    /// Read the rows nearest `lower` first (`min_id`).
    pub ascending: bool,
    /// How many rows to read, the probe row included.
    pub fetch: i64,
}

impl Bounds {
    const fn direction(self) -> &'static str {
        if self.ascending { "asc" } else { "desc" }
    }
}

/// The join tables whose other side is a list of accounts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeSet {
    /// Accounts following the subject.
    Followers,
    /// Accounts the subject follows.
    Following,
    /// Members of the list that is the subject.
    ListMembers,
    /// Accounts the subject blocked.
    Blocks,
    /// Accounts the subject muted.
    Mutes,
}

impl EdgeSet {
    /// Table, the column holding the subject, the column holding the listed account and the
    /// extra row filter on the user `u` and the edge `e`.
    const fn parts(self) -> (&'static str, &'static str, &'static str, &'static str) {
        match self {
            Self::Followers => (
                "follows",
                "followee_id",
                "follower_id",
                "e.state = 'accepted' and u.suspended_at is null",
            ),
            Self::Following => (
                "follows",
                "follower_id",
                "followee_id",
                "e.state = 'accepted' and u.suspended_at is null",
            ),
            Self::ListMembers => (
                "list_members",
                "list_id",
                "user_id",
                "u.suspended_at is null",
            ),
            Self::Blocks => ("blocks", "user_id", "target_id", "true"),
            Self::Mutes => ("mutes", "user_id", "target_id", "true"),
        }
    }
}

/// The accounts on the other side of `subject`'s `set`, newest edge first (oldest first with
/// [`Bounds::ascending`]). The bounds are account ids; each one stands for the edge that joins
/// it to `subject`, so an account that is no longer on the list gives an empty page.
///
/// # Errors
///
/// Database failures.
pub async fn edge_accounts(
    db: &PgPool,
    set: EdgeSet,
    subject: Uuid,
    bounds: Bounds,
) -> Result<Vec<UserRow>, AppError> {
    let (table, subject_col, other_col, filter) = set.parts();
    let direction = bounds.direction();
    let edge = |cursor: &str| {
        format!(
            "(select c.created_at, c.{other_col} from {table} c
               where c.{subject_col} = $1 and c.{other_col} = {cursor})"
        )
    };
    let (after, before) = (edge("$2"), edge("$3"));
    let sql = format!(
        "select {COLS}
           from {table} e join users u on u.id = e.{other_col}
          where e.{subject_col} = $1 and u.deleted_at is null and {filter}
            and ($2::uuid is null or (e.created_at, e.{other_col}) < {after})
            and ($3::uuid is null or (e.created_at, e.{other_col}) > {before})
          order by e.created_at {direction}, e.{other_col} {direction} limit $4"
    );
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(subject)
        .bind(bounds.upper)
        .bind(bounds.lower)
        .bind(bounds.fetch)
        .fetch_all(db)
        .await?)
}

/// Which entries of a profile a status list keeps.
#[derive(Clone, Copy, Debug, Default)]
pub struct StatusFilter {
    /// Only posts with media; no reposts.
    pub only_media: bool,
    /// No replies.
    pub exclude_replies: bool,
    /// No reposts.
    pub exclude_reblogs: bool,
}

/// One entry of a profile: a post, or a repost of one.
#[derive(Clone, Copy, Debug, FromRow)]
pub struct ProfileEntry {
    /// The post id for an original, the repost row id for a repost: the Mastodon status id.
    pub key: Uuid,
    /// The post.
    pub post_id: Uuid,
    /// Whether the entry is a repost.
    pub reposted: bool,
}

/// The entries of `profile`'s statuses: its posts and replies (never direct posts) and its
/// reposts, newest first (oldest first with [`Bounds::ascending`]). Whether the viewer may see
/// the posts is decided when they are loaded, so a page can come back shorter than asked.
///
/// # Errors
///
/// Database failures.
pub async fn profile_entries(
    db: &PgPool,
    profile: Uuid,
    filter: StatusFilter,
    bounds: Bounds,
) -> Result<Vec<ProfileEntry>, AppError> {
    let replies = if filter.exclude_replies {
        " and p.reply_to_id is null"
    } else {
        ""
    };
    let media = if filter.only_media {
        " and exists (select 1 from post_media pm where pm.post_id = p.id)"
    } else {
        ""
    };
    let reposts = if filter.exclude_reblogs || filter.only_media {
        String::new()
    } else {
        " union all select r.id, r.post_id, true from reposts r
           where r.user_id = $1
             and ($2::uuid is null or r.id < $2) and ($3::uuid is null or r.id > $3)"
            .to_owned()
    };
    let direction = bounds.direction();
    let sql = format!(
        "select key, post_id, reposted from (
            select p.id as key, p.id as post_id, false as reposted from posts p
             where p.author_id = $1 and p.deleted_at is null and p.visibility <> 'direct'
               {replies} {media}
               and ($2::uuid is null or p.id < $2) and ($3::uuid is null or p.id > $3)
            {reposts}
         ) t order by key {direction} limit $4"
    );
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(profile)
        .bind(bounds.upper)
        .bind(bounds.lower)
        .bind(bounds.fetch)
        .fetch_all(db)
        .await?)
}

/// How often each of `tags` was used per day in the last 7 days, as `(tag, day, posts,
/// accounts)` where `day` is the Unix time of that UTC midnight. Only public posts of authors on
/// no silenced instance count, as in tag search. Days without use are absent.
///
/// # Errors
///
/// Database failures.
pub async fn tag_usage(
    db: &PgPool,
    tags: &[String],
) -> Result<Vec<(String, i64, i64, i64)>, AppError> {
    if tags.is_empty() {
        return Ok(Vec::new());
    }
    let discoverable = super::discoverable_post("p", "a");
    let sql = format!(
        "select pt.tag,
                extract(epoch from date_trunc('day', p.created_at at time zone 'UTC'))::bigint,
                count(*), count(distinct p.author_id)
           from post_tags pt join posts p on p.id = pt.post_id join users a on a.id = p.author_id
          where pt.tag = any($1) and p.deleted_at is null and {discoverable}
            and p.created_at >= date_trunc('day', now() at time zone 'UTC') at time zone 'UTC'
                                - interval '6 days'
          group by 1, 2"
    );
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(tags)
        .fetch_all(db)
        .await?)
}
