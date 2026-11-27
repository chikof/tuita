//! Shared handler state.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use sqlx::PgPool;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::auth::passkey::RelyingParty;
use crate::config::Config;
use crate::error::AppError;
use crate::federation::keys::Generator;
use crate::limits::RateLimits;
use crate::mastodon::instance::NumbersCache;
use crate::media::MediaStore;
use crate::net::SafeClient;
use crate::realtime::Hub;
use crate::workers::unfurl::{Job, QUEUE_CAPACITY};

/// How often an open event stream re-checks that its session is still valid.
const DEFAULT_STREAM_RECHECK: Duration = Duration::from_secs(25);

/// Longest a readiness database ping may take.
const READY_PING_TIMEOUT: Duration = Duration::from_secs(1);

/// Shared handler state. Cheap to clone: one `Arc`.
#[derive(Clone)]
pub struct AppState(Arc<Inner>);
// 8: one Arc pointer; the null pointer is a niche.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<AppState>() == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<Option<AppState>>() == std::mem::size_of::<AppState>());

struct Inner {
    db: PgPool,
    config: Config,
    ready: AtomicBool,
    hub: Hub,
    limits: RateLimits,
    shutdown: CancellationToken,
    stream_recheck_ms: AtomicU64,
    safe_client: SafeClient,
    unfurl_tx: mpsc::Sender<Job>,
    unfurl_rx: Mutex<Option<mpsc::Receiver<Job>>>,
    passkeys: Option<RelyingParty>,
    keygen: Generator,
    numbers: NumbersCache,
    media: MediaStore,
}

impl AppState {
    /// Creates a state that is not ready yet; [`crate::serve_on`] flips it once listening.
    /// `media` is where uploaded files live, see [`MediaStore::open`].
    #[must_use]
    pub fn new(db: PgPool, config: Config, media: MediaStore) -> Self {
        let (unfurl_tx, unfurl_rx) = mpsc::channel(QUEUE_CAPACITY);
        let safe_client = SafeClient::new(&config);
        let passkeys = RelyingParty::new(&config.public_url);
        Self(Arc::new(Inner {
            db,
            config,
            ready: AtomicBool::new(false),
            hub: Hub::new(),
            limits: RateLimits::new(),
            shutdown: CancellationToken::new(),
            stream_recheck_ms: AtomicU64::new(DEFAULT_STREAM_RECHECK.as_millis() as u64),
            safe_client,
            keygen: Generator::default(),
            numbers: NumbersCache::default(),
            media,
            unfurl_tx,
            unfurl_rx: Mutex::new(Some(unfurl_rx)),
            passkeys,
        }))
    }

    /// First-use actor key generation: one at a time, counted.
    #[must_use]
    pub fn keygen(&self) -> &Generator {
        &self.0.keygen
    }

    /// The cached counts of the instance routes.
    pub(crate) fn numbers_cache(&self) -> &NumbersCache {
        &self.0.numbers
    }

    /// The one client for fetching URLs the server did not write.
    #[must_use]
    pub fn safe_client(&self) -> &SafeClient {
        &self.0.safe_client
    }

    /// Asks the unfurl worker to build a link card. Drops the job when the queue is full: the
    /// post simply has no card.
    pub fn queue_unfurl(&self, job: Job) {
        if let Err(err) = self.0.unfurl_tx.try_send(job) {
            tracing::debug!(%err, "unfurl queue full or closed; no card for this link");
        }
    }

    /// Hands the unfurl queue's receiving end to its taker: the worker, or a test that wants to read
    /// the jobs. `None` after the first call.
    pub fn take_unfurl_queue(&self) -> Option<mpsc::Receiver<Job>> {
        self.0
            .unfurl_rx
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
    }

    /// The WebAuthn relying party.
    ///
    /// # Errors
    ///
    /// `400` when `PUBLIC_URL` names an IP address, which WebAuthn cannot use.
    pub fn passkeys(&self) -> Result<&RelyingParty, AppError> {
        self.0.passkeys.as_ref().ok_or_else(|| {
            AppError::bad_request(
                "Passkeys need a site address with a domain name, not an IP address.",
            )
        })
    }

    /// The relying party id, when passkeys work on this instance.
    #[must_use]
    pub fn rp_id(&self) -> Option<&str> {
        self.0.passkeys.as_ref().map(RelyingParty::id)
    }

    /// The database pool.
    #[must_use]
    pub fn db(&self) -> &PgPool {
        &self.0.db
    }

    /// The settings the server started with.
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.0.config
    }

    /// Where uploaded and federated media files live.
    #[must_use]
    pub fn media(&self) -> &MediaStore {
        &self.0.media
    }

    /// The live-event hub.
    #[must_use]
    pub fn hub(&self) -> &Hub {
        &self.0.hub
    }

    /// The rate limiters.
    #[must_use]
    pub fn limits(&self) -> &RateLimits {
        &self.0.limits
    }

    /// Cancelled when the server starts draining: long-lived streams and workers end on it.
    #[must_use]
    pub fn shutdown(&self) -> &CancellationToken {
        &self.0.shutdown
    }

    /// How often an open event stream re-checks its session.
    #[must_use]
    pub fn stream_recheck(&self) -> Duration {
        Duration::from_millis(self.0.stream_recheck_ms.load(Ordering::Relaxed))
    }

    /// Changes the stream re-check interval; tests use a short one.
    pub fn set_stream_recheck(&self, every: Duration) {
        let ms = u64::try_from(every.as_millis()).unwrap_or(u64::MAX).max(1);
        self.0.stream_recheck_ms.store(ms, Ordering::Relaxed);
    }

    /// Sets whether the server accepts traffic.
    pub fn set_ready(&self, ready: bool) {
        self.0.ready.store(ready, Ordering::Release);
    }

    /// Whether the server accepts traffic. Ignores the database; see [`Self::is_healthy`].
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.0.ready.load(Ordering::Acquire)
    }

    /// Ready and the database answers `SELECT 1` within one second.
    pub async fn is_healthy(&self) -> bool {
        if !self.is_ready() {
            return false;
        }
        let ping = sqlx::query("SELECT 1").execute(&self.0.db);
        matches!(
            tokio::time::timeout(READY_PING_TIMEOUT, ping).await,
            Ok(Ok(_))
        )
    }
}
