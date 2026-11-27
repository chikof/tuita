//! A tab: navigation, evaluation, input, emulation and the collected console errors.

use std::{
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use anyhow::{Context, Result, anyhow, bail};
use chromiumoxide::{
    Page as CdpPage,
    cdp::{
        browser_protocol::{
            dom::SetFileInputFilesParams,
            emulation::{
                MediaFeature, SetDeviceMetricsOverrideParams, SetEmulatedMediaParams,
                SetLocaleOverrideParams, SetTouchEmulationEnabledParams,
            },
            input::{
                DispatchKeyEventParams, DispatchKeyEventType, DispatchMouseEventParams,
                DispatchMouseEventType, InsertTextParams, MouseButton,
            },
            log::{EventEntryAdded, LogEntryLevel, LogEntrySource},
            network::{
                EnableParams as NetworkEnable, Headers, SetExtraHttpHeadersParams,
                SetUserAgentOverrideParams,
            },
            page::{
                AddScriptToEvaluateOnNewDocumentParams, CaptureScreenshotFormat,
                SetBypassCspParams, Viewport,
            },
            web_authn::{
                AddVirtualAuthenticatorParams, AuthenticatorId, AuthenticatorProtocol,
                AuthenticatorTransport, EnableParams as WebAuthnEnable, GetCredentialsParams,
                RemoveVirtualAuthenticatorParams, VirtualAuthenticatorOptions,
            },
        },
        js_protocol::runtime::{
            ConsoleApiCalledType, EvaluateParams, EventConsoleApiCalled, EventExceptionThrown,
        },
    },
    page::ScreenshotParams,
};
use futures_util::StreamExt;
use serde_json::{Value, json};

use crate::{
    Ctx, Loc, Name,
    loc::Step,
    poll::{Poll, poll_for},
};

/// `find.js`, installed on every new document of every page.
const FIND_JS: &str = include_str!("../js/find.js");

/// `prefers-color-scheme` for [`Page::emulate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    /// `light`.
    Light,
    /// `dark`.
    Dark,
}

/// `prefers-reduced-motion` for [`Page::emulate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    /// `no-preference`.
    NoPreference,
    /// `reduce`.
    Reduce,
}

pub(crate) struct State {
    pub(crate) base: String,
    errors: Mutex<Vec<String>>,
    timeout_ms: AtomicU64,
}

/// A tab of a [`Ctx`]. Cheap to clone; clones are the same tab.
#[derive(Clone)]
pub struct Page {
    pub(crate) inner: CdpPage,
    pub(crate) st: Arc<State>,
}

impl std::fmt::Debug for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Page")
            .field("base", &self.st.base)
            .finish_non_exhaustive()
    }
}

fn spec<T, E: std::fmt::Display>(built: std::result::Result<T, E>) -> Result<T> {
    built.map_err(|e| anyhow!("{e}"))
}

/// Wires up a fresh target: error collection, init scripts, headers, viewport, locale.
pub(crate) async fn setup(inner: CdpPage, ctx: &Ctx) -> Result<Page> {
    let st = Arc::new(State {
        base: ctx.base.clone(),
        errors: Mutex::new(Vec::new()),
        timeout_ms: AtomicU64::new(
            crate::env()
                .timeout
                .as_millis()
                .try_into()
                .unwrap_or(10_000),
        ),
    });

    inner.execute(NetworkEnable::default()).await?;
    inner.enable_runtime().await?;
    inner.enable_log().await?;
    // A page of a background context has no focus, which WebAuthn requires.
    inner
        .execute(
            chromiumoxide::cdp::browser_protocol::emulation::SetFocusEmulationEnabledParams::new(
                true,
            ),
        )
        .await?;
    inner.bring_to_front().await?;

    let mut console = inner.event_listener::<EventConsoleApiCalled>().await?;
    let mut thrown = inner.event_listener::<EventExceptionThrown>().await?;
    let mut log = inner.event_listener::<EventEntryAdded>().await?;
    let sink = Arc::clone(&st);
    tokio::spawn(async move {
        while let Some(ev) = console.next().await {
            if ev.r#type == ConsoleApiCalledType::Error {
                let text = ev
                    .args
                    .iter()
                    .map(|a| match (&a.value, &a.description) {
                        (Some(Value::String(s)), _) => s.clone(),
                        (Some(v), _) => v.to_string(),
                        (None, Some(d)) => d.clone(),
                        (None, None) => String::new(),
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                sink.push(format!("console.error: {text}"));
            }
        }
    });
    let sink = Arc::clone(&st);
    tokio::spawn(async move {
        while let Some(ev) = thrown.next().await {
            let d = &ev.exception_details;
            let text = d
                .exception
                .as_ref()
                .and_then(|e| e.description.clone())
                .unwrap_or_else(|| d.text.clone());
            sink.push(format!("uncaught: {text}"));
        }
    });
    let sink = Arc::clone(&st);
    tokio::spawn(async move {
        while let Some(ev) = log.next().await {
            let e = &ev.entry;
            let wanted = e.level == LogEntryLevel::Error
                || (e.source == LogEntrySource::Security
                    && e.level != LogEntryLevel::Verbose
                    && e.level != LogEntryLevel::Info);
            let favicon = e
                .url
                .as_deref()
                .is_some_and(|u| u.ends_with("/favicon.ico"))
                || e.text.contains("/favicon.ico");
            if wanted && !favicon {
                let at = e
                    .url
                    .as_deref()
                    .map(|u| format!(" ({u})"))
                    .unwrap_or_default();
                sink.push(format!("{:?}: {}{at}", e.source, e.text));
            }
        }
    });

    inner
        .execute(AddScriptToEvaluateOnNewDocumentParams::new(FIND_JS))
        .await?;
    inner
        .execute(SetExtraHttpHeadersParams::new(Headers::new(
            json!({ "x-forwarded-for": ctx.ip }),
        )))
        .await?;
    let page = Page { inner, st };
    let (w, h) = ctx.options.viewport.unwrap_or((1280, 720));
    page.set_viewport(w, h).await?;
    if ctx.options.touch {
        page.inner
            .execute(
                SetTouchEmulationEnabledParams::builder()
                    .enabled(true)
                    .max_touch_points(1)
                    .build()
                    .map_err(anyhow::Error::msg)?,
            )
            .await?;
    }
    if let Some(locale) = &ctx.options.locale {
        let ua = crate::browser::shared()?.browser.user_agent().await?;
        page.inner
            .execute(SetLocaleOverrideParams {
                locale: Some(locale.clone()),
            })
            .await?;
        let mut over = SetUserAgentOverrideParams::new(ua);
        over.accept_language = Some(locale.clone());
        page.inner.execute(over).await?;
    }
    Ok(page)
}

impl State {
    fn push(&self, message: String) {
        if let Ok(mut errors) = self.errors.lock() {
            errors.push(message);
        }
    }
}

impl Page {
    /// How long actions and expectations on this page retry.
    #[must_use]
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.st.timeout_ms.load(Ordering::Relaxed))
    }

    /// Changes the retry window of this page (and every locator made from it afterwards).
    pub fn set_timeout(&self, timeout: Duration) {
        self.st.timeout_ms.store(
            timeout.as_millis().try_into().unwrap_or(u64::MAX),
            Ordering::Relaxed,
        );
    }

    /// The origin of the context this page belongs to.
    #[must_use]
    pub fn base(&self) -> &str {
        &self.st.base
    }

    fn absolute(&self, target: &str) -> String {
        if target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with("about:")
        {
            target.to_owned()
        } else {
            format!(
                "{}{}{target}",
                self.st.base,
                if target.starts_with('/') { "" } else { "/" }
            )
        }
    }

    /// Navigates (`/path` resolves against the context's origin) and waits for the load event.
    pub async fn goto(&self, target: &str) -> Result<()> {
        let url = self.absolute(target);
        self.inner
            .goto(url.as_str())
            .await
            .with_context(|| format!("goto {url}"))?;
        self.wait_for_js("document.readyState === 'complete'")
            .await?;
        // Another tab of the context (the hidden API page) may have taken the focus; WebAuthn needs it.
        self.inner.bring_to_front().await?;
        Ok(())
    }

    /// Reloads and waits for the load event.
    pub async fn reload(&self) -> Result<()> {
        self.inner.reload().await?;
        self.wait_for_js("document.readyState === 'complete'")
            .await?;
        Ok(())
    }

    /// `location.href`.
    pub async fn url(&self) -> Result<String> {
        Ok(self
            .eval("location.href")
            .await?
            .as_str()
            .unwrap_or_default()
            .to_owned())
    }

    /// Evaluates a JS expression (promises are awaited) and returns its JSON value; `undefined`
    /// is `null`. A throw becomes an error carrying the exception description.
    pub async fn eval(&self, expression: &str) -> Result<Value> {
        let params = spec(
            EvaluateParams::builder()
                .expression(expression)
                .await_promise(true)
                .return_by_value(true)
                .build(),
        )?;
        let out = self.inner.execute(params).await?.result;
        if let Some(d) = out.exception_details {
            let text = d.exception.and_then(|e| e.description).unwrap_or(d.text);
            bail!("{text}");
        }
        Ok(out.result.value.unwrap_or(Value::Null))
    }

    /// Calls the JS function source `function` with `arg` as JSON: `eval_fn("(n) => n + 1", json!(1))`.
    pub async fn eval_fn(&self, function: &str, arg: &Value) -> Result<Value> {
        self.eval(&format!("({function})({arg})")).await
    }

    /// Polls `expression` until it is truthy.
    pub async fn wait_for_js(&self, expression: &str) -> Result<()> {
        poll_for(
            format!("waiting for `{expression}`"),
            self.timeout(),
            || async {
                Ok(
                    if self.eval(&format!("Boolean({expression})")).await? == Value::Bool(true) {
                        Poll::Ready(())
                    } else {
                        Poll::Retry("still falsy".to_owned())
                    },
                )
            },
        )
        .await
    }

    /// `fetch` from inside the page (so cookies and `Origin` match a real client): the status and
    /// the JSON body (`null` when empty, the text when it is not JSON). The page must already sit
    /// on the origin; `url` is in absolute-path form (`/api/v1/me`).
    pub async fn fetch(
        &self,
        method: &str,
        url: &str,
        body: Option<&Value>,
    ) -> Result<(u16, Value)> {
        let script = format!(
            "(async (method, url, body) => {{
                const init = {{ method, credentials: 'same-origin', headers: {{}} }};
                if (body !== null) {{ init.headers['content-type'] = 'application/json'; init.body = JSON.stringify(body); }}
                const res = await fetch(url, init);
                const text = await res.text();
                let json = null;
                try {{ json = text ? JSON.parse(text) : null; }} catch {{ json = text; }}
                return {{ status: res.status, body: json }};
            }})({}, {}, {})",
            json!(method),
            json!(url),
            body.cloned().unwrap_or(Value::Null),
        );
        let out = self
            .eval(&script)
            .await
            .with_context(|| format!("{method} {url}"))?;
        let status = out["status"]
            .as_u64()
            .and_then(|s| u16::try_from(s).ok())
            .context("fetch returned no status")?;
        Ok((status, out["body"].clone()))
    }

    /// [`Page::fetch`] against `/api/v1{path}`.
    pub async fn call(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
    ) -> Result<(u16, Value)> {
        self.fetch(method, &format!("/api/v1{path}"), body).await
    }

    /// Runs `script` before any script of every document this page loads from now on
    /// (`addInitScript`).
    pub async fn init_script(&self, script: &str) -> Result<()> {
        self.inner
            .execute(AddScriptToEvaluateOnNewDocumentParams::new(script))
            .await?;
        Ok(())
    }

    /// Elements with this accessible role and name, e.g. `page.role("button", Name::Exact("Send"))`.
    /// Only visible elements match. Implicit roles come from the tag; `Name::Any` ignores the name.
    pub fn role<'a>(&self, role: &str, name: impl Into<Name<'a>>) -> Loc {
        Loc::root(self).role(role, name)
    }

    /// Form controls and `aria-label`ed elements whose label matches.
    pub fn label<'a>(&self, name: impl Into<Name<'a>>) -> Loc {
        Loc::root(self).label(name)
    }

    /// The deepest visible elements whose text matches.
    pub fn text<'a>(&self, name: impl Into<Name<'a>>) -> Loc {
        Loc::root(self).text(name)
    }

    /// A CSS selector. Unlike the other finders it also matches hidden elements.
    #[must_use]
    pub fn css(&self, selector: &str) -> Loc {
        Loc::root(self).css(selector)
    }

    /// Retries until the URL (`location.href`) matches. An `Exact` pattern starting with `/` is
    /// resolved against the origin.
    pub async fn expect_url(&self, name: Name<'_>) -> Result<()> {
        let name = match name {
            Name::Exact(path) if path.starts_with('/') => {
                Name::Exact(&format!("{}{path}", self.st.base)).json()
            }
            other => other.json(),
        };
        let what = format!("expecting the URL to match {name}");
        poll_for(what, self.timeout(), || async {
            let r = self
                .eval_fn("(n) => __e2e.match(n, location.href, true)", &name)
                .await?;
            Ok(if r["ok"] == Value::Bool(true) {
                Poll::Ready(())
            } else {
                Poll::Retry(format!("url is {}", r["actual"]))
            })
        })
        .await
    }

    /// Console errors since the page was opened or last cleared: `console.error`, uncaught
    /// exceptions, security (CSP) entries and failed requests. The `/favicon.ico` 404 is ignored.
    pub async fn console_errors(&self) -> Vec<String> {
        // One round trip flushes events the browser already sent; then let the listeners run.
        let _ = self.eval("0").await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        self.st.errors.lock().map(|e| e.clone()).unwrap_or_default()
    }

    /// Forgets the console errors so far.
    pub fn clear_console_errors(&self) {
        if let Ok(mut errors) = self.st.errors.lock() {
            errors.clear();
        }
    }

    /// Writes a viewport PNG, creating parent directories.
    pub async fn screenshot(&self, path: impl AsRef<Path>) -> Result<()> {
        self.shoot(path.as_ref(), None, false).await
    }

    /// Writes a PNG of the whole scrollable page.
    pub async fn screenshot_full(&self, path: impl AsRef<Path>) -> Result<()> {
        self.shoot(path.as_ref(), None, true).await
    }

    pub(crate) async fn shoot(
        &self,
        path: &Path,
        clip: Option<Viewport>,
        full: bool,
    ) -> Result<()> {
        let mut params = ScreenshotParams::builder().format(CaptureScreenshotFormat::Png);
        if full {
            params = params.full_page(true);
        }
        if let Some(clip) = clip {
            params = params.clip(clip).capture_beyond_viewport(true);
        }
        let png = self.inner.screenshot(params.build()).await?;
        if let Some(dir) = path.parent() {
            tokio::fs::create_dir_all(dir).await?;
        }
        tokio::fs::write(path, png)
            .await
            .with_context(|| format!("writing {}", path.display()))
    }

    /// Sets the viewport (CSS pixels, device scale 1, not mobile).
    pub async fn set_viewport(&self, width: u32, height: u32) -> Result<()> {
        // `mobile` must stay false: true makes innerWidth jump to the layout viewport (1560).
        let params =
            SetDeviceMetricsOverrideParams::new(i64::from(width), i64::from(height), 1.0, false);
        self.inner.execute(params).await?;
        Ok(())
    }

    /// Emulates `prefers-color-scheme` and `prefers-reduced-motion`.
    pub async fn emulate(&self, scheme: Scheme, motion: Motion) -> Result<()> {
        let scheme = match scheme {
            Scheme::Light => "light",
            Scheme::Dark => "dark",
        };
        let motion = match motion {
            Motion::NoPreference => "no-preference",
            Motion::Reduce => "reduce",
        };
        let params = SetEmulatedMediaParams::builder()
            .features(vec![
                MediaFeature::new("prefers-color-scheme", scheme),
                MediaFeature::new("prefers-reduced-motion", motion),
            ])
            .build();
        self.inner.execute(params).await?;
        Ok(())
    }

    /// Adds a software passkey authenticator (CTAP2, internal, resident keys, always verified).
    /// WebAuthn only works on a `localhost` origin, so `E2E_BASE_URL` must use that host.
    pub async fn virtual_authenticator(&self) -> Result<Authenticator> {
        self.inner.execute(WebAuthnEnable::default()).await?;
        let options = spec(
            VirtualAuthenticatorOptions::builder()
                .protocol(AuthenticatorProtocol::Ctap2)
                .transport(AuthenticatorTransport::Internal)
                .has_resident_key(true)
                .has_user_verification(true)
                .is_user_verified(true)
                .automatic_presence_simulation(true)
                .build(),
        )?;
        let id = self
            .inner
            .execute(AddVirtualAuthenticatorParams::new(options))
            .await?
            .result
            .authenticator_id;
        Ok(Authenticator {
            page: self.inner.clone(),
            id,
        })
    }

    /// Presses a key or chord: `Enter`, `Escape`, `Control+k`, `Shift+Tab`, `a`. Goes to whatever has focus.
    pub async fn press(&self, chord: &str) -> Result<()> {
        let (mods, key) = match chord.rfind('+') {
            Some(at) if at > 0 && at + 1 < chord.len() => (&chord[..at], &chord[at + 1..]),
            _ => ("", chord),
        };
        let mut modifiers = 0;
        for m in mods.split('+').filter(|m| !m.is_empty()) {
            modifiers |= match m {
                "Alt" => 1,
                "Control" | "Ctrl" => 2,
                "Meta" | "Cmd" => 4,
                "Shift" => 8,
                other => bail!("unknown modifier {other} in {chord}"),
            };
        }
        let def = KeyDef::of(key).with_context(|| format!("unknown key {key}"))?;
        // Ctrl, Alt and Meta chords are shortcuts: no character is typed.
        let text = if modifiers & 0b0111 == 0 {
            def.text.clone()
        } else {
            None
        };
        let kind = if text.is_some() {
            DispatchKeyEventType::KeyDown
        } else {
            DispatchKeyEventType::RawKeyDown
        };
        let event = |kind| {
            spec(
                DispatchKeyEventParams::builder()
                    .r#type(kind)
                    .key(def.key.clone())
                    .code(def.code.clone())
                    .windows_virtual_key_code(def.vk)
                    .modifiers(modifiers)
                    .build(),
            )
        };
        let mut down = event(kind)?;
        down.text = text;
        self.inner.execute(down).await?;
        self.inner
            .execute(event(DispatchKeyEventType::KeyUp)?)
            .await?;
        Ok(())
    }

    pub(crate) async fn insert_text(&self, text: &str) -> Result<()> {
        self.inner.execute(InsertTextParams::new(text)).await?;
        Ok(())
    }

    pub(crate) async fn mouse_move(&self, x: f64, y: f64) -> Result<()> {
        let params = spec(
            DispatchMouseEventParams::builder()
                .r#type(DispatchMouseEventType::MouseMoved)
                .x(x)
                .y(y)
                .build(),
        )?;
        self.inner.execute(params).await?;
        Ok(())
    }

    pub(crate) async fn mouse_click(&self, x: f64, y: f64) -> Result<()> {
        self.mouse_move(x, y).await?;
        for kind in [
            DispatchMouseEventType::MousePressed,
            DispatchMouseEventType::MouseReleased,
        ] {
            let params = spec(
                DispatchMouseEventParams::builder()
                    .r#type(kind)
                    .x(x)
                    .y(y)
                    .button(MouseButton::Left)
                    .click_count(1)
                    .build(),
            )?;
            self.inner.execute(params).await?;
        }
        Ok(())
    }

    pub(crate) async fn set_input_files(&self, steps: &[Step], files: Vec<String>) -> Result<()> {
        let expression = format!("__e2e.element({})", Loc::steps_json(steps));
        let object_id = poll_for("resolving the file input", self.timeout(), || async {
            let params = spec(
                EvaluateParams::builder()
                    .expression(expression.as_str())
                    .build(),
            )?;
            let out = self.inner.execute(params).await?.result;
            if let Some(d) = out.exception_details {
                return Ok(Poll::Retry(
                    d.exception.and_then(|e| e.description).unwrap_or(d.text),
                ));
            }
            Ok(match out.result.object_id {
                Some(id) => Poll::Ready(id),
                None => Poll::Retry("no element".to_owned()),
            })
        })
        .await?;
        let mut params = spec(SetFileInputFilesParams::builder().files(files).build())?;
        params.object_id = Some(object_id);
        self.inner.execute(params).await?;
        Ok(())
    }

    /// Lets this tab ignore its Content-Security-Policy (blob and data module imports). Only for
    /// tests that load scripts, never for tests that check the policy.
    pub async fn bypass_csp(&self) -> Result<()> {
        self.inner.execute(SetBypassCspParams::new(true)).await?;
        Ok(())
    }

    /// Closes the tab.
    pub async fn close(self) -> Result<()> {
        self.inner.close().await?;
        Ok(())
    }
}

/// A virtual passkey authenticator on a page.
#[derive(Debug)]
pub struct Authenticator {
    page: CdpPage,
    id: AuthenticatorId,
}

impl Authenticator {
    /// How many credentials it holds.
    pub async fn credentials(&self) -> Result<usize> {
        let out = self
            .page
            .execute(GetCredentialsParams::new(self.id.clone()))
            .await?;
        Ok(out.result.credentials.len())
    }

    /// Removes the authenticator.
    pub async fn remove(self) -> Result<()> {
        self.page
            .execute(RemoveVirtualAuthenticatorParams::new(self.id))
            .await?;
        Ok(())
    }
}

/// How a key press is sent.
struct KeyDef {
    key: String,
    code: String,
    vk: i64,
    text: Option<String>,
}

impl KeyDef {
    fn of(key: &str) -> Option<Self> {
        let named = |key: &str, code: &str, vk: i64, text: Option<&str>| Self {
            key: key.to_owned(),
            code: code.to_owned(),
            vk,
            text: text.map(str::to_owned),
        };
        Some(match key {
            "Enter" => named("Enter", "Enter", 13, Some("\r")),
            "Escape" => named("Escape", "Escape", 27, None),
            "Tab" => named("Tab", "Tab", 9, None),
            "Backspace" => named("Backspace", "Backspace", 8, None),
            "Delete" => named("Delete", "Delete", 46, None),
            "ArrowLeft" => named("ArrowLeft", "ArrowLeft", 37, None),
            "ArrowUp" => named("ArrowUp", "ArrowUp", 38, None),
            "ArrowRight" => named("ArrowRight", "ArrowRight", 39, None),
            "ArrowDown" => named("ArrowDown", "ArrowDown", 40, None),
            "Home" => named("Home", "Home", 36, None),
            "End" => named("End", "End", 35, None),
            "PageUp" => named("PageUp", "PageUp", 33, None),
            "PageDown" => named("PageDown", "PageDown", 34, None),
            " " | "Space" => named(" ", "Space", 32, Some(" ")),
            _ => {
                let mut chars = key.chars();
                let (c, None) = (chars.next()?, chars.next()) else {
                    return None;
                };
                let upper = c.to_ascii_uppercase();
                let (code, vk) = if c.is_ascii_alphabetic() {
                    (format!("Key{upper}"), i64::from(u32::from(upper)))
                } else if c.is_ascii_digit() {
                    (format!("Digit{c}"), i64::from(u32::from(c)))
                } else {
                    (String::new(), 0)
                };
                Self {
                    key: c.to_string(),
                    code,
                    vk,
                    text: Some(c.to_string()),
                }
            }
        })
    }
}
