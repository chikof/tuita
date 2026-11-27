//! Settings read once from the environment, all set by `scripts/e2e.sh` (plus `E2E_CHROME` from devenv).

use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
    time::Duration,
};

/// Where nix puts Chromium on this workstation; used when `E2E_CHROME` is unset.
const DEFAULT_CHROME: &str = "/run/current-system/sw/bin/chromium";

/// Environment of one test process.
#[derive(Debug, Clone)]
pub struct Env {
    /// `E2E_BASE_URL`, without a trailing slash. The host MUST be `localhost`: WebAuthn refuses
    /// IP-literal origins. Defaults to `http://localhost:8081`, the port of `scripts/e2e.sh`.
    pub base_url: String,
    /// `E2E_DATABASE_URL`: the throwaway database the server under test uses.
    pub database_url: Option<String>,
    /// `E2E_TUITA_BIN`: the release binary, used for `admin promote` and for the federation peer.
    pub tuita_bin: Option<PathBuf>,
    /// `E2E_PEER_PORT`: port of the second instance (default 8083).
    pub peer_port: u16,
    /// `E2E_CHROME`, else the nix system Chromium when it exists.
    pub chrome: Option<PathBuf>,
    /// `E2E_TIMEOUT_MS`: how long an action or expectation keeps retrying (default 10000).
    pub timeout: Duration,
}

/// The process-wide [`Env`].
pub fn env() -> &'static Env {
    static ENV: OnceLock<Env> = OnceLock::new();
    ENV.get_or_init(|| {
        let var = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
        let chrome = var("E2E_CHROME").map(PathBuf::from).or_else(|| {
            Path::new(DEFAULT_CHROME)
                .exists()
                .then(|| PathBuf::from(DEFAULT_CHROME))
        });
        Env {
            base_url: var("E2E_BASE_URL")
                .unwrap_or_else(|| "http://localhost:8081".to_owned())
                .trim_end_matches('/')
                .to_owned(),
            database_url: var("E2E_DATABASE_URL"),
            tuita_bin: var("E2E_TUITA_BIN").map(PathBuf::from),
            peer_port: var("E2E_PEER_PORT")
                .and_then(|v| v.parse().ok())
                .unwrap_or(8083),
            chrome,
            timeout: Duration::from_millis(
                var("E2E_TIMEOUT_MS")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10_000),
            ),
        }
    })
}

/// Whether a Chromium binary was found. The self-test skips itself without one.
#[must_use]
pub fn chrome_available() -> bool {
    env().chrome.is_some()
}
