//! The one headless Chromium every test in a process shares.
//!
//! `#[tokio::test]` gives each test its own runtime, so the browser and its CDP handler live on a
//! dedicated thread with a runtime of its own; test futures talk to it over channels and work from
//! any runtime.
//!
//! **Shutdown.** Statics are never dropped at process exit, so `kill_on_drop` cannot fire. Instead
//! a small detached `sh` watchdog waits for the test process to disappear, then terminates
//! Chromium and removes its profile directory. It also runs after Ctrl-C or a crash.

use std::{
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::OnceLock,
    time::Duration,
};

use anyhow::{Context, Result, anyhow};
use chromiumoxide::{Browser, BrowserConfig};
use futures_util::StreamExt;
use tokio::runtime::{Builder, Handle};

use crate::env;

/// The browser and the handle of the runtime that drives it.
pub(crate) struct Shared {
    pub(crate) browser: Browser,
    pub(crate) rt: Handle,
}

/// Waits for `$1` (the test process) to exit, then stops `$2` (Chromium) and removes `$3`.
const WATCHDOG: &str = r#"
while kill -0 "$1" 2>/dev/null; do sleep 0.3; done
kill -TERM "$2" 2>/dev/null
sleep 1
kill -0 "$2" 2>/dev/null && kill -KILL "$2" 2>/dev/null
rm -rf "$3"
"#;

/// Chromium's profile directory, also the harness's scratch space (uploads): the watchdog removes it
/// when the process ends.
pub(crate) fn profile_dir() -> PathBuf {
    std::env::temp_dir().join(format!("tuita-e2e-chrome-{}", std::process::id()))
}

/// The process-wide browser, launched on first use.
pub(crate) fn shared() -> Result<&'static Shared> {
    static SHARED: OnceLock<std::result::Result<Shared, String>> = OnceLock::new();
    SHARED
        .get_or_init(|| spawn_runtime().map_err(|error| format!("{error:#}")))
        .as_ref()
        .map_err(|message| anyhow!("launching Chromium failed: {message}"))
}

fn spawn_runtime() -> Result<Shared> {
    let chrome = env()
        .chrome
        .clone()
        .context("no Chromium found: set E2E_CHROME or run inside `devenv shell`")?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("tuita-e2e-browser".to_owned())
        .spawn(move || {
            let runtime = match Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    let _ = tx.send(Err(anyhow!(error)));
                    return;
                }
            };
            runtime.block_on(async move {
                match launch(&chrome).await {
                    Ok(browser) => {
                        let _ = tx.send(Ok(Shared {
                            browser,
                            rt: Handle::current(),
                        }));
                        // Park: dropping the runtime would stop the handler task.
                        std::future::pending::<()>().await;
                    }
                    Err(error) => {
                        let _ = tx.send(Err(error));
                    }
                }
            });
        })?;
    rx.recv()
        .context("the browser thread died before answering")?
}

async fn launch(chrome: &Path) -> Result<Browser> {
    let profile = profile_dir();
    let config = BrowserConfig::builder()
        .chrome_executable(chrome)
        .user_data_dir(&profile)
        .no_sandbox()
        .new_headless_mode()
        .launch_timeout(Duration::from_secs(30))
        .build()
        .map_err(|message| anyhow!(message))?;
    let (mut browser, mut handler) = Browser::launch(config).await.context("Browser::launch")?;
    tokio::spawn(async move { while handler.next().await.is_some() {} });
    let pid = browser
        .get_mut_child()
        .and_then(|child| child.as_mut_inner().id());
    watch(pid, &profile)?;
    Ok(browser)
}

fn watch(chrome_pid: Option<u32>, profile: &Path) -> Result<()> {
    let Some(chrome_pid) = chrome_pid else {
        return Ok(());
    };
    Command::new("sh")
        .args(["-c", WATCHDOG, "sh"])
        .arg(std::process::id().to_string())
        .arg(chrome_pid.to_string())
        .arg(profile)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .context("starting the Chromium watchdog")?;
    Ok(())
}
