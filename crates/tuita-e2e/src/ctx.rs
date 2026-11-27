//! An isolated browser context: its own cookies and storage, and its own client address.

use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{Context, Result};
use chromiumoxide::cdp::browser_protocol::{
    browser::{BrowserContextId, PermissionDescriptor, PermissionSetting, SetPermissionParams},
    network::CookieParam,
    storage::{GetCookiesParams, SetCookiesParams},
    target::{CreateBrowserContextParams, CreateTargetParams},
};
use serde_json::Value;
use tokio::sync::OnceCell;

use crate::{Page, browser::shared, env, page};

/// Options of a new [`Ctx`]; the defaults are a 1280x720 desktop in `E2E_BASE_URL`.
#[derive(Debug, Clone, Default)]
pub struct CtxOptions {
    /// Origin relative URLs resolve against; defaults to `E2E_BASE_URL`.
    pub base: Option<String>,
    /// Browser language, e.g. `pl-PL`: sets `Accept-Language` and `navigator.language`.
    pub locale: Option<String>,
    /// Viewport in CSS pixels, default 1280x720.
    pub viewport: Option<(u32, u32)>,
    /// Touch emulation (`TouchEvent` exists, `maxTouchPoints` is 1).
    pub touch: bool,
}

/// A browser context. Every one sends its own `X-Forwarded-For` (`10.<pid % 256>.<n >> 8>.<n & 255>`,
/// so the server's per-address auth limiter does
/// not see the whole suite as one client. Dropping the context disposes it, pages included.
#[derive(Debug)]
pub struct Ctx {
    pub(crate) id: BrowserContextId,
    pub(crate) options: CtxOptions,
    pub(crate) base: String,
    pub(crate) ip: String,
    api: OnceCell<Page>,
    closed: bool,
}

/// Contexts created in this process; the second and third address octets come from it.
static COUNT: AtomicU32 = AtomicU32::new(0);

impl Ctx {
    /// A fresh context on `E2E_BASE_URL`.
    pub async fn new() -> Result<Self> {
        Self::with(CtxOptions::default()).await
    }

    /// A fresh context on another origin (the federation peer, the self-test server).
    pub async fn with_base(base: &str) -> Result<Self> {
        Self::with(CtxOptions {
            base: Some(base.to_owned()),
            ..CtxOptions::default()
        })
        .await
    }

    /// A fresh context with explicit options.
    pub async fn with(options: CtxOptions) -> Result<Self> {
        let browser = &shared()?.browser;
        let id = browser
            .create_browser_context(CreateBrowserContextParams::default())
            .await
            .context("Target.createBrowserContext")?;
        let n = COUNT.fetch_add(1, Ordering::Relaxed) + 1;
        let ip = format!(
            "10.{}.{}.{}",
            std::process::id() % 256,
            (n >> 8) & 255,
            n & 255
        );
        let base = options
            .base
            .clone()
            .unwrap_or_else(|| env().base_url.clone());
        Ok(Self {
            id,
            options,
            base: base.trim_end_matches('/').to_owned(),
            ip,
            api: OnceCell::new(),
            closed: false,
        })
    }

    /// The `X-Forwarded-For` this context's pages send.
    #[must_use]
    pub fn ip(&self) -> &str {
        &self.ip
    }

    /// The origin relative URLs resolve against.
    #[must_use]
    pub fn base(&self) -> &str {
        &self.base
    }

    /// A new tab in this context. Pages share cookies, `localStorage` and IndexedDB.
    pub async fn page(&self) -> Result<Page> {
        let browser = &shared()?.browser;
        let target = CreateTargetParams::builder()
            .url("about:blank")
            .browser_context_id(self.id.clone())
            .build()
            .map_err(anyhow::Error::msg)?;
        let inner = browser
            .new_page(target)
            .await
            .context("Target.createTarget")?;
        page::setup(inner, self).await
    }

    /// The value of cookie `name` in this context, `HttpOnly` ones included.
    pub async fn cookie(&self, name: &str) -> Result<Option<String>> {
        let browser = &shared()?.browser;
        let cookies = browser
            .execute(
                GetCookiesParams::builder()
                    .browser_context_id(self.id.clone())
                    .build(),
            )
            .await
            .context("Storage.getCookies")?
            .result
            .cookies;
        Ok(cookies
            .into_iter()
            .find(|c| c.name == name)
            .map(|c| c.value))
    }

    /// A copy of this context's cookies in a new context (without
    /// IndexedDB): the same signed-in session on a "second device" that has no local keys.
    pub async fn fork(&self) -> Result<Self> {
        let copy = Self::with(self.options.clone()).await?;
        let browser = &shared()?.browser;
        let cookies = browser
            .execute(
                GetCookiesParams::builder()
                    .browser_context_id(self.id.clone())
                    .build(),
            )
            .await
            .context("Storage.getCookies")?
            .result
            .cookies;
        let params = cookies
            .into_iter()
            .map(|c| {
                let mut p = CookieParam::new(c.name, c.value);
                p.domain = Some(c.domain);
                p.path = Some(c.path);
                p.secure = Some(c.secure);
                p.http_only = Some(c.http_only);
                p.same_site = c.same_site;
                if c.expires > 0.0 {
                    p.expires = Some(
                        chromiumoxide::cdp::browser_protocol::network::TimeSinceEpoch::new(
                            c.expires,
                        ),
                    );
                }
                p
            })
            .collect::<Vec<_>>();
        browser
            .execute(
                SetCookiesParams::builder()
                    .cookies(params)
                    .browser_context_id(copy.id.clone())
                    .build()
                    .map_err(anyhow::Error::msg)?,
            )
            .await
            .context("Storage.setCookies")?;
        Ok(copy)
    }

    /// Grants browser permissions by Permissions API name (`clipboard-read`, `clipboard-write`,
    /// `notifications`) for every origin of this context.
    pub async fn grant(&self, permissions: &[&str]) -> Result<()> {
        let browser = &shared()?.browser;
        for name in permissions {
            let mut params = SetPermissionParams::new(
                PermissionDescriptor::new(*name),
                PermissionSetting::Granted,
            );
            params.browser_context_id = Some(self.id.clone());
            browser
                .execute(params)
                .await
                .with_context(|| format!("Browser.setPermission {name}"))?;
        }
        Ok(())
    }

    /// A hidden tab on `/healthz`, the origin without any app code: where [`Ctx::fetch`] runs.
    pub async fn api_page(&self) -> Result<&Page> {
        self.api
            .get_or_try_init(|| async {
                let page = self.page().await?;
                page.goto("/healthz").await?;
                Ok::<_, anyhow::Error>(page)
            })
            .await
    }

    /// `fetch(url, ...)` from a page of this context that sits on the origin, so cookies and
    /// `Origin` match a real client. Returns the status and the JSON body (`null` when empty, the
    /// text when it is not JSON). `url` is absolute-path form (`/api/v1/me`).
    pub async fn fetch(
        &self,
        method: &str,
        url: &str,
        body: Option<&Value>,
    ) -> Result<(u16, Value)> {
        self.api_page().await?.fetch(method, url, body).await
    }

    /// [`Ctx::fetch`] against `/api/v1{path}`.
    pub async fn call(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
    ) -> Result<(u16, Value)> {
        self.fetch(method, &format!("/api/v1{path}"), body).await
    }

    /// Disposes the context now and waits for it; dropping does the same in the background.
    pub async fn close(mut self) -> Result<()> {
        let shared = shared()?;
        let id = self.id.clone();
        self.closed = true;
        shared
            .browser
            .dispose_browser_context(id)
            .await
            .context("Target.disposeBrowserContext")?;
        Ok(())
    }
}

impl Drop for Ctx {
    fn drop(&mut self) {
        if self.closed {
            return;
        }
        if let Ok(shared) = shared() {
            let id = self.id.clone();
            shared.rt.spawn(async move {
                let _ = shared.browser.dispose_browser_context(id).await;
            });
        }
    }
}
