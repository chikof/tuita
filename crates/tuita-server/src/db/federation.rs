//! The federation console: domain blocks, the delivery queue and what this server knows of
//! other instances. Every change writes its audit row in the caller's transaction.

use sqlx::{FromRow, PgConnection, PgPool};
use time::OffsetDateTime;
use tuita_core::ap::url_authority;
use uuid::Uuid;

use super::audit::{self, Entry};
use crate::dto::{DeliveryFailureDto, DomainBlockDto, DomainBlockSeverity, FederationQueueDto};
use crate::error::AppError;

/// Longest reason of a block, in characters.
pub const MAX_REASON_CHARS: usize = 200;
/// Instances listed as failing.
const FAILING_LISTED: i64 = 50;

/// The `400` for a domain that cannot be one.
const INVALID_DOMAIN: &str = "That domain isn\u{2019}t valid.";

/// Whether `text` is `host` or `host:port`: dot-separated labels of letters, digits and inner
/// hyphens, and a port from 1 to 65535.
fn is_host_and_port(text: &str) -> bool {
    let (host, port) = match text.rsplit_once(':') {
        Some((host, port)) => (host, Some(port)),
        None => (text, None),
    };
    if let Some(port) = port
        && (!port.bytes().all(|b| b.is_ascii_digit()) || !port.parse::<u16>().is_ok_and(|n| n != 0))
    {
        return false;
    }
    !host.is_empty()
        && host.len() <= 253
        && host.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}

/// The domain `input` names, lower case, as `host` or `host:port`; `None` when it is not one.
/// A URL is reduced to its host (and a port that is not the scheme's default).
///
/// ```
/// # use tuita_server::db::federation::parse_domain;
/// assert_eq!(parse_domain("  Social.Example ").as_deref(), Some("social.example"));
/// assert_eq!(parse_domain("https://Social.Example:443/@a").as_deref(), Some("social.example"));
/// assert_eq!(parse_domain("127.0.0.1:8081").as_deref(), Some("127.0.0.1:8081"));
/// assert_eq!(parse_domain("not a domain"), None);
/// assert_eq!(parse_domain("a.example/path"), None);
/// ```
#[must_use]
pub fn parse_domain(input: &str) -> Option<String> {
    let input = input.trim();
    let authority = if input.contains("://") {
        url_authority(input)?
    } else {
        input.to_ascii_lowercase()
    };
    is_host_and_port(&authority).then_some(authority)
}

/// Normalises the domain of a new block.
///
/// # Errors
///
/// `400 Enter a domain.`, `400 That domain isn’t valid.`, `400 You can’t block your own
/// instance.`
pub fn new_block_domain(input: &str, own_domain: &str) -> Result<String, AppError> {
    if input.trim().is_empty() {
        return Err(AppError::bad_request("Enter a domain."));
    }
    let domain = parse_domain(input).ok_or_else(|| AppError::bad_request(INVALID_DOMAIN))?;
    if domain.eq_ignore_ascii_case(own_domain) {
        return Err(AppError::bad_request(
            "You can\u{2019}t block your own instance.",
        ));
    }
    Ok(domain)
}

/// Trims `reason` and checks its length.
///
/// # Errors
///
/// `400 Reason must be at most 200 characters.`
pub fn check_reason(reason: &str) -> Result<String, AppError> {
    let reason = reason.trim();
    if reason.chars().count() > MAX_REASON_CHARS {
        return Err(AppError::bad_request(
            "Reason must be at most 200 characters.",
        ));
    }
    Ok(reason.to_owned())
}

#[derive(FromRow)]
struct BlockRow {
    domain: String,
    severity: String,
    reason: String,
    accounts: i32,
    created_at: OffsetDateTime,
}

impl From<BlockRow> for DomainBlockDto {
    fn from(row: BlockRow) -> Self {
        Self {
            domain: row.domain,
            severity: DomainBlockSeverity::parse(&row.severity)
                .unwrap_or(DomainBlockSeverity::Suspend),
            reason: row.reason,
            accounts: row.accounts,
            created_at: row.created_at,
        }
    }
}

const BLOCK_SELECT: &str = "select b.domain, b.severity, b.reason, b.created_at,
        (select count(*)::int4 from users u where u.domain = b.domain) as accounts
    from domain_blocks b";

/// Every block, newest first.
///
/// # Errors
///
/// Database failures.
pub async fn blocks(db: &PgPool) -> Result<Vec<DomainBlockDto>, AppError> {
    let rows: Vec<BlockRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{BLOCK_SELECT} order by b.created_at desc, b.domain"
    )))
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

async fn block_row(
    conn: &mut PgConnection,
    domain: &str,
) -> Result<Option<DomainBlockDto>, AppError> {
    let row: Option<BlockRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{BLOCK_SELECT} where b.domain = $1"
    )))
    .bind(domain)
    .fetch_optional(conn)
    .await?;
    Ok(row.map(Into::into))
}

fn detail(severity: DomainBlockSeverity, reason: &str) -> String {
    if reason.is_empty() {
        severity.as_str().to_owned()
    } else {
        format!("{}: {reason}", severity.as_str())
    }
}

/// Queued deliveries to a `suspend`ed instance, or to a subdomain of it, go nowhere.
async fn drop_deliveries(conn: &mut PgConnection, domain: &str) -> Result<(), AppError> {
    let sql = format!(
        "delete from ap_deliveries where {}",
        super::domain_in_block("domain", "$1::text")
    );
    sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(domain)
        .execute(conn)
        .await?;
    Ok(())
}

/// Blocks `domain`, replacing an earlier block of it. A `suspend` also deletes the deliveries
/// queued for the domain. Audit row `domain.block`.
///
/// # Errors
///
/// Database failures.
pub async fn block(
    conn: &mut PgConnection,
    actor: Uuid,
    domain: &str,
    severity: DomainBlockSeverity,
    reason: &str,
) -> Result<DomainBlockDto, AppError> {
    sqlx::query(
        "insert into domain_blocks (domain, severity, reason) values ($1, $2, $3)
         on conflict (domain) do update
            set severity = excluded.severity, reason = excluded.reason, created_at = now()",
    )
    .bind(domain)
    .bind(severity.as_str())
    .bind(reason)
    .execute(&mut *conn)
    .await?;
    if severity == DomainBlockSeverity::Suspend {
        drop_deliveries(conn, domain).await?;
    }
    audit::record(
        conn,
        Entry::new(Some(actor), "domain.block")
            .target(domain)
            .text(detail(severity, reason)),
    )
    .await?;
    block_row(conn, domain)
        .await?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("block vanished")))
}

/// Changes the severity and the reason of the block on `domain`; `None` leaves a field as it
/// is. Audit row `domain.block`.
///
/// # Errors
///
/// `404 Block not found.`; database failures.
pub async fn patch_block(
    conn: &mut PgConnection,
    actor: Uuid,
    domain: &str,
    severity: Option<DomainBlockSeverity>,
    reason: Option<&str>,
) -> Result<DomainBlockDto, AppError> {
    let updated: Option<(String, String)> = sqlx::query_as(
        "update domain_blocks
            set severity = coalesce($2, severity), reason = coalesce($3, reason)
          where domain = $1 returning severity, reason",
    )
    .bind(domain)
    .bind(severity.map(DomainBlockSeverity::as_str))
    .bind(reason)
    .fetch_optional(&mut *conn)
    .await?;
    let (stored, stored_reason) = updated.ok_or(AppError::Missing("Block not found."))?;
    let stored = DomainBlockSeverity::parse(&stored).unwrap_or(DomainBlockSeverity::Suspend);
    if stored == DomainBlockSeverity::Suspend {
        drop_deliveries(conn, domain).await?;
    }
    audit::record(
        conn,
        Entry::new(Some(actor), "domain.block")
            .target(domain)
            .text(detail(stored, &stored_reason)),
    )
    .await?;
    block_row(conn, domain)
        .await?
        .ok_or(AppError::Missing("Block not found."))
}

/// Lifts the block on `domain`. Audit row `domain.unblock`.
///
/// # Errors
///
/// `404 Block not found.`; database failures.
pub async fn unblock(conn: &mut PgConnection, actor: Uuid, domain: &str) -> Result<(), AppError> {
    let removed = sqlx::query("delete from domain_blocks where domain = $1")
        .bind(domain)
        .execute(&mut *conn)
        .await?
        .rows_affected();
    if removed == 0 {
        return Err(AppError::Missing("Block not found."));
    }
    audit::record(
        conn,
        Entry::new(Some(actor), "domain.unblock").target(domain),
    )
    .await
}

#[derive(FromRow)]
struct QueueRow {
    count: i32,
    due: i32,
    oldest: Option<OffsetDateTime>,
}

#[derive(FromRow)]
struct FailingRow {
    domain: String,
    count: i32,
    attempts: i32,
    last_error: Option<String>,
    next_attempt_at: OffsetDateTime,
}

/// The delivery queue: its size, how much is due, the oldest entry and the instances whose
/// deliveries have failed, most waiting first.
///
/// # Errors
///
/// Database failures.
pub async fn queue(db: &PgPool) -> Result<FederationQueueDto, AppError> {
    let totals: QueueRow = sqlx::query_as(
        "select count(*)::int4 as count,
                (count(*) filter (where next_attempt_at <= now()))::int4 as due,
                min(created_at) as oldest
           from ap_deliveries",
    )
    .fetch_one(db)
    .await?;
    let failing: Vec<FailingRow> = sqlx::query_as(
        "select domain, count(*)::int4 as count, max(attempts) as attempts,
                (array_agg(last_error order by attempts desc, next_attempt_at desc)
                    filter (where last_error is not null))[1] as last_error,
                min(next_attempt_at) as next_attempt_at
           from ap_deliveries
          group by domain having max(attempts) > 0
          order by count(*) desc, domain limit $1",
    )
    .bind(FAILING_LISTED)
    .fetch_all(db)
    .await?;
    Ok(FederationQueueDto {
        count: totals.count,
        due: totals.due,
        oldest: totals.oldest,
        failing: failing
            .into_iter()
            .map(|row| DeliveryFailureDto {
                domain: row.domain,
                count: row.count,
                attempts: row.attempts,
                last_error: row.last_error,
                next_attempt_at: row.next_attempt_at,
            })
            .collect(),
    })
}

/// Makes every queued delivery due now. Returns how many there are. Audit row
/// `federation.retry`.
///
/// # Errors
///
/// Database failures.
pub async fn retry_now(conn: &mut PgConnection, actor: Uuid) -> Result<i32, AppError> {
    let requeued = sqlx::query("update ap_deliveries set next_attempt_at = now()")
        .execute(&mut *conn)
        .await?
        .rows_affected();
    let requeued = i32::try_from(requeued).unwrap_or(i32::MAX);
    audit::record(
        conn,
        Entry::new(Some(actor), "federation.retry")
            .target("deliveries")
            .text(format!("{requeued} queued")),
    )
    .await?;
    Ok(requeued)
}

/// How many instances, remote accounts and remote posts this server knows.
///
/// # Errors
///
/// Database failures.
pub async fn known(db: &PgPool) -> Result<(i32, i32, i32), AppError> {
    Ok(sqlx::query_as(
        "select (select count(distinct domain) from users where domain is not null)::int4,
                (select count(*) from users where domain is not null)::int4,
                (select count(*) from posts p join users u on u.id = p.author_id
                  where u.domain is not null and p.deleted_at is null)::int4",
    )
    .fetch_one(db)
    .await?)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn domains_are_normalised_or_refused() {
        for (input, want) in [
            ("Example.COM", Some("example.com")),
            ("  a.b-c.example  ", Some("a.b-c.example")),
            ("https://Example.com/users/x?y#z", Some("example.com")),
            ("http://127.0.0.1:8081/x", Some("127.0.0.1:8081")),
            ("localhost:3000", Some("localhost:3000")),
            ("example.com:0", None),
            ("example.com:70000", None),
            ("example.com:", None),
            ("-a.example", None),
            ("a..example", None),
            ("a.example/", None),
            ("user@a.example", None),
            ("[::1]:80", None),
            ("ftp://a.example", None),
            ("", None),
            ("exämple.com", None),
        ] {
            assert_eq!(parse_domain(input).as_deref(), want, "{input:?}");
        }
    }

    #[test]
    fn a_new_block_names_a_valid_foreign_domain() {
        let message = |input: &str| {
            new_block_domain(input, "me.example")
                .expect_err("refused")
                .to_string()
        };
        assert_eq!(message("  "), "Enter a domain.");
        assert_eq!(message("what is this"), "That domain isn\u{2019}t valid.");
        assert_eq!(
            message("ME.example"),
            "You can\u{2019}t block your own instance."
        );
        assert_eq!(
            new_block_domain("https://Other.Example/x", "me.example")
                .ok()
                .as_deref(),
            Some("other.example")
        );
    }

    #[test]
    fn reasons_are_trimmed_and_capped() {
        assert_eq!(check_reason("  spam  ").ok().as_deref(), Some("spam"));
        assert!(check_reason(&"x".repeat(200)).is_ok());
        assert_eq!(
            check_reason(&"x".repeat(201))
                .expect_err("too long")
                .to_string(),
            "Reason must be at most 200 characters."
        );
    }
}
