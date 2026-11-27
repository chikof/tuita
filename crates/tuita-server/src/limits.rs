//! Rate limits, one keyed limiter per key family.
//!
//! These are GCRA limiters from `governor`: the same count may go through at once, then one more
//! cell frees up every `window / count`.

use std::net::IpAddr;
use std::num::NonZeroU32;
use std::time::Duration;

use governor::clock::Clock as _;
use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};
use uuid::Uuid;

use crate::error::AppError;

const MINUTE: u64 = 60;
const HOUR: u64 = 60 * MINUTE;

/// `auth:{ip}`: signup and login share it. 20 per 10 minutes.
const AUTH: (u32, u64) = (20, 10 * MINUTE);
/// `pw:{user}`: password changes. 10 per 10 minutes.
const PASSWORD: (u32, u64) = (10, 10 * MINUTE);
/// `post:{user}`: new posts. 60 per 10 minutes.
const POST: (u32, u64) = (60, 10 * MINUTE);
/// `follow:{user}`: follows. 200 per hour.
const FOLLOW: (u32, u64) = (200, HOUR);
/// `dm:{user}`: direct messages. 120 per 10 minutes.
const DM: (u32, u64) = (120, 10 * MINUTE);
/// `msgkey:{user}`: message key changes. 10 per hour.
const MESSAGE_KEY: (u32, u64) = (10, HOUR);
/// `report:{user}`: reports. 20 per hour.
const REPORT: (u32, u64) = (20, HOUR);
/// `export:{user}`: data exports. 5 per hour.
const EXPORT: (u32, u64) = (5, HOUR);
/// `lists:{user}`: list creation. 30 per hour.
const LISTS: (u32, u64) = (30, HOUR);
/// `schedule:{user}`: scheduled posts. 30 per hour.
const SCHEDULE: (u32, u64) = (30, HOUR);
/// `views:{user}`: view reports. 120 per minute.
const VIEWS: (u32, u64) = (120, MINUTE);
/// `video:{user}`: video uploads. 20 per hour.
const VIDEO: (u32, u64) = (20, HOUR);
/// `edit:{user}`: post edits. 30 per 10 minutes.
const EDIT: (u32, u64) = (30, 10 * MINUTE);
/// `upload:{user}`: image uploads. 60 per 10 minutes.
const UPLOAD: (u32, u64) = (60, 10 * MINUTE);
/// `token:{id}`: requests of one API token. 600 per 10 minutes.
const TOKEN: (u32, u64) = (600, 10 * MINUTE);
/// `tokens:{user}`: new API tokens. 20 per hour.
const TOKEN_CREATE: (u32, u64) = (20, HOUR);
/// `aff:{user}`: affiliation invitations. 60 per hour.
const AFFILIATE: (u32, u64) = (60, HOUR);
/// `pk-reg:{user}`: passkey registration steps. 30 per 10 minutes.
const PASSKEY_REGISTER: (u32, u64) = (30, 10 * MINUTE);
/// `pk-auth:{ip}`: passkey sign-in options. 60 per 10 minutes.
const PASSKEY_OPTIONS: (u32, u64) = (60, 10 * MINUTE);
/// `inbox:{ip}`: deliveries to the inboxes from one address. 300 per minute.
const INBOX: (u32, u64) = (300, MINUTE);
/// `inbox-domain:{domain}`: deliveries from one verified instance. 1200 per minute.
const INBOX_DOMAIN: (u32, u64) = (1200, MINUTE);
/// `scan:{user}`: word-filter scans of every account, by staff. 6 per minute.
const SCAN: (u32, u64) = (6, MINUTE);
/// `bot:{user}`: bot flag and owner changes. 10 per hour.
const BOT: (u32, u64) = (10, HOUR);
/// `lookup:{user}`: remote lookups that reach the network. 30 per 10 minutes.
const LOOKUP: (u32, u64) = (30, 10 * MINUTE);
/// `key-refresh:{actor}`: re-fetches of an actor whose signature failed. One per 5 minutes.
const KEY_REFRESH: (u32, u64) = (1, 5 * MINUTE);
/// `oauth-app:{ip}`: app registrations (`POST /api/v1/apps`, no sign-in needed). 30 per hour.
const OAUTH_APP: (u32, u64) = (30, HOUR);
/// `oauth-token:{ip}`: `POST /oauth/token` and `POST /oauth/revoke`. 60 per 10 minutes.
const OAUTH_TOKEN: (u32, u64) = (60, 10 * MINUTE);
/// `oauth-authorize:{user}`: approvals at `POST /oauth/authorize`. 30 per 10 minutes.
const OAUTH_AUTHORIZE: (u32, u64) = (30, 10 * MINUTE);

fn limiter<K>((count, window_secs): (u32, u64)) -> DefaultKeyedRateLimiter<K>
where
    K: std::hash::Hash + Eq + Clone,
{
    let count = NonZeroU32::new(count).expect("limit counts are not zero");
    let period = Duration::from_secs(window_secs) / count.get();
    let quota = Quota::with_period(period)
        .expect("limit periods are not zero")
        .allow_burst(count);
    RateLimiter::keyed(quota)
}

/// The limiters the API consults. Lives in [`crate::AppState`].
#[derive(Debug)]
pub struct RateLimits {
    auth: DefaultKeyedRateLimiter<IpAddr>,
    password: DefaultKeyedRateLimiter<Uuid>,
    post: DefaultKeyedRateLimiter<Uuid>,
    follow: DefaultKeyedRateLimiter<Uuid>,
    dm: DefaultKeyedRateLimiter<Uuid>,
    message_key: DefaultKeyedRateLimiter<Uuid>,
    report: DefaultKeyedRateLimiter<Uuid>,
    export: DefaultKeyedRateLimiter<Uuid>,
    lists: DefaultKeyedRateLimiter<Uuid>,
    schedule: DefaultKeyedRateLimiter<Uuid>,
    views: DefaultKeyedRateLimiter<Uuid>,
    video: DefaultKeyedRateLimiter<Uuid>,
    upload: DefaultKeyedRateLimiter<Uuid>,
    edit: DefaultKeyedRateLimiter<Uuid>,
    token: DefaultKeyedRateLimiter<Uuid>,
    token_create: DefaultKeyedRateLimiter<Uuid>,
    affiliate: DefaultKeyedRateLimiter<Uuid>,
    passkey_register: DefaultKeyedRateLimiter<Uuid>,
    passkey_options: DefaultKeyedRateLimiter<IpAddr>,
    inbox: DefaultKeyedRateLimiter<IpAddr>,
    inbox_domain: DefaultKeyedRateLimiter<String>,
    scan: DefaultKeyedRateLimiter<Uuid>,
    bot: DefaultKeyedRateLimiter<Uuid>,
    lookup: DefaultKeyedRateLimiter<Uuid>,
    key_refresh: DefaultKeyedRateLimiter<String>,
    oauth_app: DefaultKeyedRateLimiter<IpAddr>,
    oauth_token: DefaultKeyedRateLimiter<IpAddr>,
    oauth_authorize: DefaultKeyedRateLimiter<Uuid>,
}

impl Default for RateLimits {
    fn default() -> Self {
        Self::new()
    }
}

/// Defines `check_*` methods that count one hit on a per-user limiter.
macro_rules! user_checks {
    ($($(#[$doc:meta])* $method:ident => $field:ident),+ $(,)?) => {
        $(
            $(#[$doc])*
            ///
            /// # Errors
            ///
            /// [`AppError::TooManyRequests`] once the limit is used up.
            pub fn $method(&self, user: Uuid) -> Result<(), AppError> {
                check(&self.$field, &user)
            }
        )+
    };
}

impl RateLimits {
    /// Creates empty limiters with the configured counts and windows.
    #[must_use]
    pub fn new() -> Self {
        Self {
            auth: limiter(AUTH),
            password: limiter(PASSWORD),
            post: limiter(POST),
            follow: limiter(FOLLOW),
            dm: limiter(DM),
            message_key: limiter(MESSAGE_KEY),
            report: limiter(REPORT),
            export: limiter(EXPORT),
            lists: limiter(LISTS),
            schedule: limiter(SCHEDULE),
            views: limiter(VIEWS),
            video: limiter(VIDEO),
            upload: limiter(UPLOAD),
            edit: limiter(EDIT),
            token: limiter(TOKEN),
            token_create: limiter(TOKEN_CREATE),
            affiliate: limiter(AFFILIATE),
            passkey_register: limiter(PASSKEY_REGISTER),
            passkey_options: limiter(PASSKEY_OPTIONS),
            inbox: limiter(INBOX),
            inbox_domain: limiter(INBOX_DOMAIN),
            scan: limiter(SCAN),
            bot: limiter(BOT),
            lookup: limiter(LOOKUP),
            key_refresh: limiter(KEY_REFRESH),
            oauth_app: limiter(OAUTH_APP),
            oauth_token: limiter(OAUTH_TOKEN),
            oauth_authorize: limiter(OAUTH_AUTHORIZE),
        }
    }

    /// Counts one signup or login try from `ip`.
    ///
    /// # Errors
    ///
    /// [`AppError::TooManyRequests`] once `ip` made 20 tries in 10 minutes.
    pub fn check_auth(&self, ip: IpAddr) -> Result<(), AppError> {
        check(&self.auth, &ip)
    }

    /// Counts one request for passkey sign-in options from `ip`.
    ///
    /// # Errors
    ///
    /// [`AppError::TooManyRequests`] once `ip` made 60 in 10 minutes.
    pub fn check_passkey_options(&self, ip: IpAddr) -> Result<(), AppError> {
        check(&self.passkey_options, &ip)
    }

    /// Counts one delivery to an inbox from `ip`. An IPv6 address counts as its whole `/64`,
    /// the smallest block one subscriber holds, so rotating addresses inside it gains nothing.
    ///
    /// # Errors
    ///
    /// [`AppError::TooManyRequests`] once `ip` made 300 in a minute.
    pub fn check_inbox(&self, ip: IpAddr) -> Result<(), AppError> {
        check(&self.inbox, &network_of(ip))
    }

    /// Counts one app registration from `ip` (an IPv6 address counts as its `/64`).
    ///
    /// # Errors
    ///
    /// [`AppError::TooManyRequests`] once `ip` made 30 in an hour.
    pub fn check_oauth_app(&self, ip: IpAddr) -> Result<(), AppError> {
        check(&self.oauth_app, &network_of(ip))
    }

    /// Counts one request to `/oauth/token` or `/oauth/revoke` from `ip` (an IPv6 address
    /// counts as its `/64`).
    ///
    /// # Errors
    ///
    /// [`AppError::TooManyRequests`] once `ip` made 60 in 10 minutes.
    pub fn check_oauth_token(&self, ip: IpAddr) -> Result<(), AppError> {
        check(&self.oauth_token, &network_of(ip))
    }

    /// Counts one re-fetch of `actor` after a signature that failed: one per 5 minutes, so a
    /// stream of bad signatures cannot make this server hammer the actor's instance.
    ///
    /// # Errors
    ///
    /// [`AppError::TooManyRequests`] while the last re-fetch is younger than 5 minutes.
    pub fn check_key_refresh(&self, actor: &str) -> Result<(), AppError> {
        check(&self.key_refresh, &actor.to_owned())
    }

    /// Counts one verified delivery from `domain`.
    ///
    /// # Errors
    ///
    /// [`AppError::TooManyRequests`] once `domain` made 1200 in a minute.
    pub fn check_inbox_domain(&self, domain: &str) -> Result<(), AppError> {
        check(&self.inbox_domain, &domain.to_owned())
    }

    user_checks! {
        /// Counts one password change by `user`: 10 in 10 minutes.
        check_password => password,
        /// Counts one new post by `user`: 60 in 10 minutes.
        check_post => post,
        /// Counts one follow by `user`: 200 in an hour.
        check_follow => follow,
        /// Counts one direct message by `user`: 120 in 10 minutes.
        check_dm => dm,
        /// Counts one message key change by `user`: 10 in an hour.
        check_message_key => message_key,
        /// Counts one report by `user`: 20 in an hour.
        check_report => report,
        /// Counts one data export by `user`: 5 in an hour.
        check_export => export,
        /// Counts one new list by `user`: 30 in an hour.
        check_lists => lists,
        /// Counts one scheduled post by `user`: 30 in an hour.
        check_schedule => schedule,
        /// Counts one view report by `user`: 120 in a minute.
        check_views => views,
        /// Counts one video upload by `user`: 20 in an hour.
        check_video => video,
        /// Counts one image upload by `user`: 60 in 10 minutes.
        check_upload => upload,
        /// Counts one post edit by `user`: 30 in 10 minutes.
        check_edit => edit,
        /// Counts one API token request of token `id`: 600 in 10 minutes.
        check_token => token,
        /// Counts one new API token of `user`: 20 in an hour.
        check_token_create => token_create,
        /// Counts one affiliation invitation by organization `user`: 60 in an hour.
        check_affiliate => affiliate,
        /// Counts one passkey registration step by `user`: 30 in 10 minutes.
        check_passkey_register => passkey_register,
        /// Counts one word-filter scan by staff member `user`: 6 in a minute.
        check_scan => scan,
        /// Counts one bot-flag change by `user`: 10 in an hour.
        check_bot => bot,
        /// Counts one remote lookup by `user` that reaches the network: 30 in 10 minutes.
        check_lookup => lookup,
        /// Counts one approval at `POST /oauth/authorize` by `user`: 30 in 10 minutes.
        check_oauth_authorize => oauth_authorize,
    }

    /// Forgets keys whose limit has fully recovered. Called by the cleanup worker.
    pub fn sweep(&self) {
        self.auth.retain_recent();
        self.password.retain_recent();
        self.post.retain_recent();
        self.follow.retain_recent();
        self.dm.retain_recent();
        self.message_key.retain_recent();
        self.report.retain_recent();
        self.export.retain_recent();
        self.lists.retain_recent();
        self.schedule.retain_recent();
        self.views.retain_recent();
        self.video.retain_recent();
        self.upload.retain_recent();
        self.edit.retain_recent();
        self.token.retain_recent();
        self.token_create.retain_recent();
        self.affiliate.retain_recent();
        self.passkey_register.retain_recent();
        self.passkey_options.retain_recent();
        self.inbox.retain_recent();
        self.inbox_domain.retain_recent();
        self.scan.retain_recent();
        self.bot.retain_recent();
        self.lookup.retain_recent();
        self.key_refresh.retain_recent();
        self.oauth_app.retain_recent();
        self.oauth_token.retain_recent();
        self.oauth_authorize.retain_recent();
    }
}

fn check<K>(limiter: &DefaultKeyedRateLimiter<K>, key: &K) -> Result<(), AppError>
where
    K: std::hash::Hash + Eq + Clone,
{
    limiter.check_key(key).map_err(|denied| {
        let wait = denied.wait_time_from(limiter.clock().now());
        AppError::TooManyRequests {
            retry_after_secs: u32::try_from(wait.as_secs().saturating_add(1)).unwrap_or(u32::MAX),
        }
    })
}

/// The key a rate limit counts `ip` under: IPv4 as it is, IPv6 as its `/64`.
fn network_of(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V4(_) => ip,
        IpAddr::V6(v6) => IpAddr::V6((u128::from(v6) & (u128::MAX << 64)).into()),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn the_21st_auth_try_in_a_window_is_refused() {
        let limits = RateLimits::new();
        let ip = IpAddr::from([203, 0, 113, 9]);
        for _ in 0..20 {
            assert!(limits.check_auth(ip).is_ok());
        }
        match limits.check_auth(ip) {
            Err(AppError::TooManyRequests { retry_after_secs }) => {
                assert!((1..=31).contains(&retry_after_secs), "{retry_after_secs}");
            }
            other => panic!("expected 429, got {other:?}"),
        }
        assert!(limits.check_auth(IpAddr::from([203, 0, 113, 10])).is_ok());
    }

    #[test]
    fn inbox_limits_count_per_address_and_per_domain() {
        let limits = RateLimits::new();
        let ip = IpAddr::from([203, 0, 113, 9]);
        for _ in 0..300 {
            assert!(limits.check_inbox(ip).is_ok());
        }
        assert!(matches!(
            limits.check_inbox(ip),
            Err(AppError::TooManyRequests { .. })
        ));
        for _ in 0..1200 {
            assert!(limits.check_inbox_domain("a.example").is_ok());
        }
        assert!(matches!(
            limits.check_inbox_domain("a.example"),
            Err(AppError::TooManyRequests { .. })
        ));
        assert!(limits.check_inbox_domain("b.example").is_ok());
    }

    #[test]
    fn inbox_counts_an_ipv6_slash_64_as_one_address() {
        let limits = RateLimits::new();
        let ip = |last: u16| IpAddr::from([0x2001, 0xdb8, 1, 2, 0, 0, 0, last]);
        for n in 0..300 {
            assert!(limits.check_inbox(ip(n)).is_ok());
        }
        assert!(matches!(
            limits.check_inbox(ip(999)),
            Err(AppError::TooManyRequests { .. })
        ));
        assert!(
            limits
                .check_inbox(IpAddr::from([0x2001, 0xdb8, 1, 3, 0, 0, 0, 1]))
                .is_ok()
        );
    }

    #[test]
    fn a_key_refresh_is_allowed_once_per_actor_per_window() {
        let limits = RateLimits::new();
        assert!(
            limits
                .check_key_refresh("https://a.example/users/x")
                .is_ok()
        );
        assert!(matches!(
            limits.check_key_refresh("https://a.example/users/x"),
            Err(AppError::TooManyRequests { .. })
        ));
        assert!(
            limits
                .check_key_refresh("https://a.example/users/y")
                .is_ok()
        );
    }
}
