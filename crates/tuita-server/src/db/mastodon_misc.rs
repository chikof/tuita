//! What the Mastodon trends need beyond tuita's own trend list: the daily use of a tag.

use std::collections::HashMap;

use sqlx::PgPool;
use time::OffsetDateTime;

use super::discoverable_post;
use crate::error::AppError;

/// How many days a tag's history holds, today included.
pub const HISTORY_DAYS: i64 = 7;
const DAY: i64 = 86_400;

/// The use of one tag on one UTC day.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DayUse {
    /// The start of the day as a Unix timestamp.
    pub day: i64,
    /// Posts that carry the tag.
    pub uses: i64,
    /// Accounts that wrote them.
    pub accounts: i64,
}

/// The last [`HISTORY_DAYS`] days of each tag in `tags` (lower case), today first, a day without
/// use counted as zero. Counts the posts a trend counts: discoverable posts of live accounts.
///
/// # Errors
///
/// Database failures.
pub async fn tag_history(
    db: &PgPool,
    tags: &[String],
) -> Result<HashMap<String, Vec<DayUse>>, AppError> {
    let discoverable = discoverable_post("p", "a");
    let sql = format!(
        "select pt.tag,
                extract(epoch from date_trunc('day', p.created_at at time zone 'UTC'))::bigint,
                count(*), count(distinct p.author_id)
           from post_tags pt
           join posts p on p.id = pt.post_id
           join users a on a.id = p.author_id
          where pt.tag = any($1) and p.deleted_at is null
            and a.suspended_at is null and a.deleted_at is null and {discoverable}
            and p.created_at >=
                (date_trunc('day', now() at time zone 'UTC') - make_interval(days => $2))
                    at time zone 'UTC'
          group by 1, 2"
    );
    let rows: Vec<(String, i64, i64, i64)> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(tags)
        .bind(i32::try_from(HISTORY_DAYS - 1).unwrap_or(i32::MAX))
        .fetch_all(db)
        .await?;
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let today = now - now.rem_euclid(DAY);
    let mut history: HashMap<String, Vec<DayUse>> = tags
        .iter()
        .map(|tag| {
            let days = (0..HISTORY_DAYS)
                .map(|ago| DayUse {
                    day: today - ago * DAY,
                    uses: 0,
                    accounts: 0,
                })
                .collect();
            (tag.clone(), days)
        })
        .collect();
    for (tag, day, uses, accounts) in rows {
        if let Some(slot) = history
            .get_mut(&tag)
            .and_then(|days| days.iter_mut().find(|entry| entry.day == day))
        {
            slot.uses = uses;
            slot.accounts = accounts;
        }
    }
    Ok(history)
}
