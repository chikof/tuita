//! Polls: validating a poll request, storing it with its post, tallying votes and reading a
//! poll as one viewer sees it.

use std::collections::HashMap;

use sqlx::{FromRow, PgConnection, PgPool};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::config::Config;
use crate::dto::{PollDto, PollInput, PollOptionDto};
use crate::error::AppError;
use crate::federation::outbound;

/// Fewest answers a poll has.
pub const MIN_OPTIONS: usize = 2;
/// Most answers a poll has.
pub const MAX_OPTIONS: usize = 4;
/// Longest answer in characters.
pub const MAX_OPTION_CHARS: usize = 25;
/// Longest poll in hours.
pub const MAX_HOURS: u32 = 168;

/// A poll request that passed validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidPoll {
    /// The trimmed, distinct answers.
    pub options: Vec<String>,
    /// A voter may pick several.
    pub multiple: bool,
    /// How long it runs, 1 to 168.
    pub hours: u32,
}

/// Checks a poll request and trims its answers.
///
/// # Errors
///
/// `400` with a message: 2 to 4 answers, at most 25 characters each, no repeats,
/// 1 to 168 hours.
pub fn validate(input: &PollInput) -> Result<ValidPoll, AppError> {
    let options: Vec<String> = input
        .options
        .iter()
        .map(|option| option.trim().to_owned())
        .filter(|option| !option.is_empty())
        .collect();
    if !(MIN_OPTIONS..=MAX_OPTIONS).contains(&options.len()) {
        return Err(AppError::bad_request("Polls need 2 to 4 choices."));
    }
    if options
        .iter()
        .any(|option| option.chars().count() > MAX_OPTION_CHARS)
    {
        return Err(AppError::bad_request(
            "Poll choices can be at most 25 characters.",
        ));
    }
    let mut lower: Vec<String> = options.iter().map(|option| option.to_lowercase()).collect();
    lower.sort_unstable();
    lower.dedup();
    if lower.len() != options.len() {
        return Err(AppError::bad_request("Poll choices must be different."));
    }
    if !(1..=MAX_HOURS).contains(&input.duration_hours) {
        return Err(AppError::bad_request(
            "Polls can run from 1 hour to 7 days.",
        ));
    }
    Ok(ValidPoll {
        options,
        multiple: input.multiple,
        hours: input.duration_hours,
    })
}

/// Stores the poll of a new post; it ends `hours` from now.
///
/// # Errors
///
/// Database failures.
pub async fn insert(conn: &mut PgConnection, post: Uuid, poll: &ValidPoll) -> Result<(), AppError> {
    let ends_at = OffsetDateTime::now_utc() + Duration::hours(i64::from(poll.hours));
    let counts = vec![0_i32; poll.options.len()];
    sqlx::query(
        "insert into polls (post_id, options, multiple, ends_at, counts) values ($1, $2, $3, $4, $5)",
    )
    .bind(post)
    .bind(&poll.options)
    .bind(poll.multiple)
    .bind(ends_at)
    .bind(counts)
    .execute(conn)
    .await?;
    Ok(())
}

#[derive(FromRow)]
struct PollRow {
    post_id: Uuid,
    options: Vec<String>,
    multiple: bool,
    ends_at: OffsetDateTime,
    counts: Vec<i32>,
    voters: i32,
}

/// The polls among `posts`, as `viewer` sees them. Posts without a poll are absent.
///
/// # Errors
///
/// Database failures.
pub async fn for_posts(
    db: &PgPool,
    viewer: Uuid,
    posts: &[Uuid],
) -> Result<HashMap<Uuid, PollDto>, AppError> {
    if posts.is_empty() {
        return Ok(HashMap::new());
    }
    let rows: Vec<PollRow> = sqlx::query_as(
        "select post_id, options, multiple, ends_at, counts, voters from polls where post_id = any($1)",
    )
    .bind(posts)
    .fetch_all(db)
    .await?;
    if rows.is_empty() {
        return Ok(HashMap::new());
    }
    let ids: Vec<Uuid> = rows.iter().map(|row| row.post_id).collect();
    let votes: Vec<(Uuid, i16)> = sqlx::query_as(
        "select post_id, choice from poll_votes where user_id = $1 and post_id = any($2)
         order by choice",
    )
    .bind(viewer)
    .bind(&ids)
    .fetch_all(db)
    .await?;
    let now = OffsetDateTime::now_utc();
    Ok(rows
        .into_iter()
        .map(|row| {
            let voted = votes
                .iter()
                .filter(|(post, _)| *post == row.post_id)
                .filter_map(|(_, choice)| u8::try_from(*choice).ok())
                .collect();
            let options: Vec<PollOptionDto> = row
                .options
                .into_iter()
                .zip(row.counts.iter().copied().chain(std::iter::repeat(0)))
                .map(|(text, votes)| PollOptionDto { text, votes })
                .collect();
            let total = options.iter().map(|option| option.votes).sum();
            (
                row.post_id,
                PollDto {
                    options,
                    total,
                    voters: row.voters,
                    multiple: row.multiple,
                    ends_at: row.ends_at,
                    closed: now >= row.ends_at,
                    voted,
                },
            )
        })
        .collect())
}

/// What a vote needs to know about the post.
#[derive(FromRow)]
struct VoteTarget {
    options: Vec<String>,
    multiple: bool,
    ends_at: OffsetDateTime,
    author_id: Uuid,
}

/// Records `voter`'s vote and bumps the tallies in one transaction.
///
/// # Errors
///
/// `404 This post has no poll.`; `400 This poll has ended.`, `Pick a choice.`,
/// `Pick one choice.`; `403 You can’t vote on this poll.` when the author and voter blocked each
/// other; `409 You already voted.`; database failures.
pub async fn vote(
    db: &PgPool,
    config: &Config,
    voter: Uuid,
    post: Uuid,
    choices: &[u8],
) -> Result<Uuid, AppError> {
    let mut tx = db.begin().await?;
    let sql = format!(
        "select pl.options, pl.multiple, pl.ends_at, p.author_id
           from polls pl join posts p on p.id = pl.post_id
          where pl.post_id = $1 and p.deleted_at is null and {} for update of pl",
        super::visible_post("p", "$2")
    );
    let target: VoteTarget = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(post)
        .bind(voter)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::Missing("This post has no poll."))?;
    if OffsetDateTime::now_utc() >= target.ends_at {
        return Err(AppError::bad_request("This poll has ended."));
    }
    let blocked: bool = sqlx::query_scalar(
        "select exists (select 1 from blocks
            where (user_id = $1 and target_id = $2) or (user_id = $2 and target_id = $1))",
    )
    .bind(voter)
    .bind(target.author_id)
    .fetch_one(&mut *tx)
    .await?;
    if blocked {
        return Err(AppError::forbidden("You can’t vote on this poll."));
    }
    let voted: bool = sqlx::query_scalar(
        "select exists (select 1 from poll_votes where post_id = $1 and user_id = $2)",
    )
    .bind(post)
    .bind(voter)
    .fetch_one(&mut *tx)
    .await?;
    if voted {
        return Err(AppError::conflict("You already voted."));
    }
    let mut picked: Vec<i16> = choices.iter().map(|choice| i16::from(*choice)).collect();
    picked.sort_unstable();
    picked.dedup();
    if picked.is_empty()
        || picked
            .iter()
            .any(|choice| usize::from(choice.unsigned_abs()) >= target.options.len())
    {
        return Err(AppError::bad_request("Pick a choice."));
    }
    if !target.multiple && picked.len() > 1 {
        return Err(AppError::bad_request("Pick one choice."));
    }
    sqlx::query(
        "insert into poll_votes (post_id, user_id, choice)
         select $1, $2, c from unnest($3::int2[]) c",
    )
    .bind(post)
    .bind(voter)
    .bind(&picked)
    .execute(&mut *tx)
    .await?;
    let picked_idx: Vec<i32> = picked.iter().map(|choice| i32::from(*choice) + 1).collect();
    sqlx::query(
        "update polls set voters = voters + 1,
            counts = (select array_agg(case when i = any($2) then c + 1 else c end order by i)
                        from unnest(counts) with ordinality as t(c, i))
          where post_id = $1",
    )
    .bind(post)
    .bind(&picked_idx)
    .execute(&mut *tx)
    .await?;
    outbound::vote(&mut tx, config, voter, post, &picked).await?;
    tx.commit().await?;
    Ok(target.author_id)
}

#[cfg(test)]
mod test {
    use super::*;

    fn input(options: &[&str], hours: u32) -> PollInput {
        PollInput {
            options: options.iter().map(|option| (*option).to_owned()).collect(),
            multiple: false,
            duration_hours: hours,
        }
    }

    fn message(result: Result<ValidPoll, AppError>) -> String {
        result.expect_err("invalid poll").to_string()
    }

    #[test]
    fn a_poll_has_two_to_four_distinct_short_answers_for_up_to_a_week() {
        assert_eq!(
            validate(&input(&[" Yes ", "", "No"], 24))
                .expect("valid")
                .options,
            ["Yes", "No"]
        );
        assert_eq!(
            message(validate(&input(&["Only"], 24))),
            "Polls need 2 to 4 choices."
        );
        assert_eq!(
            message(validate(&input(&["a", "b", "c", "d", "e"], 24))),
            "Polls need 2 to 4 choices."
        );
        assert_eq!(
            message(validate(&input(&["a", &"x".repeat(26)], 24))),
            "Poll choices can be at most 25 characters."
        );
        assert_eq!(
            message(validate(&input(&["Yes", "yes"], 24))),
            "Poll choices must be different."
        );
        assert_eq!(
            message(validate(&input(&["a", "b"], 0))),
            "Polls can run from 1 hour to 7 days."
        );
        assert_eq!(
            message(validate(&input(&["a", "b"], 169))),
            "Polls can run from 1 hour to 7 days."
        );
        assert!(validate(&input(&["a", "b"], 168)).is_ok());
    }
}
