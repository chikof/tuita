//! Lazy, auto-waiting element locators (the harness's `getByRole`, `getByLabel`, `getByText`,
//! `locator`). A [`Loc`] stores how to find elements; every action and expectation re-resolves it
//! in the page, retrying until it holds or `E2E_TIMEOUT_MS` passes.

use std::{fmt::Display, path::Path};

use anyhow::{Context, Result, anyhow};
use chromiumoxide::cdp::browser_protocol::page::Viewport;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    Name, Page,
    poll::{Poll, poll_for},
};

/// One step of a locator, as `find.js` reads it.
pub(crate) type Step = Value;

/// Position and size of an element in page (document) coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
    /// Width.
    pub width: f64,
    /// Height.
    pub height: f64,
}

/// A way to find elements on a [`Page`]. Cheap to clone and to refine.
#[derive(Clone)]
pub struct Loc {
    page: Page,
    steps: Vec<Step>,
    desc: String,
}

impl std::fmt::Debug for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.desc)
    }
}

impl Loc {
    pub(crate) fn root(page: &Page) -> Self {
        Self {
            page: page.clone(),
            steps: Vec::new(),
            desc: "page".to_owned(),
        }
    }

    pub(crate) fn steps_json(steps: &[Step]) -> String {
        Value::Array(steps.to_vec()).to_string()
    }

    fn then(&self, step: Step, what: impl Display) -> Self {
        let mut steps = self.steps.clone();
        steps.push(step);
        Self {
            page: self.page.clone(),
            steps,
            desc: format!("{} >> {what}", self.desc),
        }
    }

    // --- refining ---------------------------------------------------------------------------------

    /// Descendants with this role and accessible name (see [`Page::role`]).
    pub fn role<'a>(&self, role: &str, name: impl Into<Name<'a>>) -> Self {
        let name = name.into();
        self.then(
            json!({"k": "role", "role": role, "name": name.json()}),
            format!("role={role}[name={}]", name.describe()),
        )
    }

    /// Descendant controls whose label matches.
    pub fn label<'a>(&self, name: impl Into<Name<'a>>) -> Self {
        let name = name.into();
        self.then(
            json!({"k": "label", "name": name.json()}),
            format!("label={}", name.describe()),
        )
    }

    /// The deepest visible descendants whose text matches.
    pub fn text<'a>(&self, name: impl Into<Name<'a>>) -> Self {
        let name = name.into();
        self.then(
            json!({"k": "text", "name": name.json()}),
            format!("text={}", name.describe()),
        )
    }

    /// Descendants matching a CSS selector (hidden ones included).
    #[must_use]
    pub fn css(&self, selector: &str) -> Self {
        self.then(
            json!({"k": "css", "sel": selector}),
            format!("css={selector}"),
        )
    }

    /// This locator searched inside every element `parent` matches.
    #[must_use]
    pub fn within(&self, parent: &Self) -> Self {
        let mut steps = parent.steps.clone();
        steps.extend(self.steps.iter().cloned());
        Self {
            page: self.page.clone(),
            steps,
            desc: format!(
                "{} >> {}",
                parent.desc,
                self.desc.trim_start_matches("page >> ")
            ),
        }
    }

    /// Keeps the matches whose text matches (`filter({ hasText })`).
    pub fn has_text<'a>(&self, name: impl Into<Name<'a>>) -> Self {
        let name = name.into();
        self.then(
            json!({"k": "has_text", "name": name.json()}),
            format!("has-text={}", name.describe()),
        )
    }

    /// Keeps the matches that contain a match of `inner` (`filter({ has })`).
    #[must_use]
    pub fn has(&self, inner: &Self) -> Self {
        self.then(
            json!({"k": "has", "steps": inner.steps}),
            format!("has=({})", inner.desc.trim_start_matches("page >> ")),
        )
    }

    /// The `index`th match, 0-based.
    #[must_use]
    pub fn nth(&self, index: usize) -> Self {
        self.then(json!({"k": "nth", "n": index}), format!("nth={index}"))
    }

    /// The first match.
    #[must_use]
    pub fn first(&self) -> Self {
        self.nth(0)
    }

    /// The last match.
    #[must_use]
    pub fn last(&self) -> Self {
        self.then(json!({"k": "nth", "n": -1}), "last")
    }

    // --- plumbing ---------------------------------------------------------------------------------

    /// One evaluation of `__e2e.act(steps, op, arg)`.
    async fn act(&self, op: &str, arg: &Value) -> Result<Value> {
        let script = format!(
            "__e2e.act({}, {}, {arg})",
            Self::steps_json(&self.steps),
            json!(op)
        );
        self.page.eval(&script).await
    }

    /// Retries `op` until it answers `ok` and `accept` is satisfied; `what` names the attempt in the timeout error.
    async fn until<T>(
        &self,
        what: String,
        op: &str,
        arg: &Value,
        accept: impl Fn(&Value) -> Option<T>,
    ) -> Result<T> {
        poll_for(what, self.page.timeout(), || async {
            let r = self.act(op, arg).await?;
            if r["ok"] != Value::Bool(true) {
                return Ok(Poll::Retry(
                    r["reason"].as_str().unwrap_or("not ok").to_owned(),
                ));
            }
            Ok(match accept(&r) {
                Some(value) => Poll::Ready(value),
                None => Poll::Retry(format!("got {}", r["actual"])),
            })
        })
        .await
    }

    /// Retries `op` until it is `ok`.
    async fn ok(&self, what: &str, op: &str, arg: &Value) -> Result<Value> {
        self.until(format!("{what} {}", self.desc), op, arg, |r| {
            Some(r.clone())
        })
        .await
    }

    // --- reading ----------------------------------------------------------------------------------

    /// How many elements match right now.
    pub async fn count(&self) -> Result<usize> {
        let r = self.act("count", &Value::Null).await?;
        r["value"]
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .context("count returned no number")
    }

    /// Text content of every match, whitespace collapsed.
    pub async fn texts(&self) -> Result<Vec<String>> {
        let r = self.act("texts", &Value::Null).await?;
        Ok(r["value"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Whether exactly one element matches and is visible, right now.
    pub async fn is_visible(&self) -> Result<bool> {
        let r = self.act("info", &Value::Null).await?;
        Ok(r["count"] == 1 && r["visible"][0] == Value::Bool(true))
    }

    async fn read(&self, what: &str, op: &str, arg: &Value) -> Result<Value> {
        Ok(self.ok(what, op, arg).await?["value"].clone())
    }

    /// Text content of the single match, whitespace collapsed, isolation marks removed.
    pub async fn text_content(&self) -> Result<String> {
        Ok(self
            .read("reading the text of", "text", &Value::Null)
            .await?
            .as_str()
            .unwrap_or_default()
            .to_owned())
    }

    /// `innerText` of the single match (line breaks kept).
    pub async fn inner_text(&self) -> Result<String> {
        Ok(self
            .read("reading the inner text of", "inner_text", &Value::Null)
            .await?
            .as_str()
            .unwrap_or_default()
            .to_owned())
    }

    /// `value` of the single input.
    pub async fn value(&self) -> Result<String> {
        Ok(self
            .read("reading the value of", "value", &Value::Null)
            .await?
            .as_str()
            .unwrap_or_default()
            .to_owned())
    }

    /// An attribute of the single match.
    pub async fn attr(&self, name: &str) -> Result<Option<String>> {
        Ok(self
            .read("reading an attribute of", "attr", &json!(name))
            .await?
            .as_str()
            .map(str::to_owned))
    }

    /// A computed style property of the single match.
    pub async fn css_value(&self, property: &str) -> Result<String> {
        Ok(self
            .read("reading a style of", "css", &json!(property))
            .await?
            .as_str()
            .unwrap_or_default()
            .to_owned())
    }

    /// Whether the single checkbox, radio or switch is checked.
    pub async fn is_checked(&self) -> Result<bool> {
        Ok(self
            .read("reading the checked state of", "checked", &Value::Null)
            .await?
            == Value::Bool(true))
    }

    /// Where the single match is, in document coordinates.
    pub async fn bounding_box(&self) -> Result<BoundingBox> {
        let r = self.ok("measuring", "box", &Value::Null).await?;
        let n = |k: &str| r[k].as_f64().unwrap_or_default();
        Ok(BoundingBox {
            x: n("x"),
            y: n("y"),
            width: n("width"),
            height: n("height"),
        })
    }

    // --- acting -----------------------------------------------------------------------------------

    /// Scrolls the single match into view, waits for it to be visible, enabled, still and
    /// uncovered, then clicks its centre with real mouse events.
    pub async fn click(&self) -> Result<()> {
        let r = self.ok("clicking", "click", &Value::Null).await?;
        self.page
            .mouse_click(
                r["x"].as_f64().unwrap_or_default(),
                r["y"].as_f64().unwrap_or_default(),
            )
            .await
    }

    /// Moves the mouse over the single match.
    pub async fn hover(&self) -> Result<()> {
        let r = self.ok("hovering", "hover", &Value::Null).await?;
        self.page
            .mouse_move(
                r["x"].as_f64().unwrap_or_default(),
                r["y"].as_f64().unwrap_or_default(),
            )
            .await
    }

    /// Focuses the single match.
    pub async fn focus(&self) -> Result<()> {
        self.ok("focusing", "focus", &Value::Null).await.map(|_| ())
    }

    /// Replaces the content of the single input, textarea or contenteditable with `text`.
    pub async fn fill(&self, text: &str) -> Result<()> {
        let r = self.ok("filling", "fill", &json!(text)).await?;
        if r["insert"] == Value::Bool(true) {
            self.page.insert_text(text).await?;
        }
        Ok(())
    }

    /// Focuses the single match and presses a key or chord (see [`Page::press`]).
    pub async fn press(&self, chord: &str) -> Result<()> {
        self.focus().await?;
        self.page.press(chord).await
    }

    /// Focuses the single match and types `text` one key press at a time.
    pub async fn press_sequentially(&self, text: &str) -> Result<()> {
        self.focus().await?;
        for c in text.chars() {
            self.page.press(&c.to_string()).await?;
        }
        Ok(())
    }

    /// Checks the single checkbox, radio or switch; a no-op when it already is.
    pub async fn check(&self) -> Result<()> {
        self.set_checked(true).await
    }

    /// Unchecks the single checkbox or switch; a no-op when it already is off.
    pub async fn uncheck(&self) -> Result<()> {
        self.set_checked(false).await
    }

    async fn set_checked(&self, want: bool) -> Result<()> {
        if self.is_checked().await? != want {
            self.click().await?;
        }
        poll_for(
            format!(
                "waiting for {} to be {}checked",
                self.desc,
                if want { "" } else { "un" }
            ),
            self.page.timeout(),
            || async {
                Ok(if self.is_checked().await? == want {
                    Poll::Ready(())
                } else {
                    Poll::Retry("state did not change".to_owned())
                })
            },
        )
        .await
    }

    /// Selects the option of the single `<select>` whose value or label is `choice`.
    pub async fn select(&self, choice: &str) -> Result<()> {
        self.ok("selecting in", "select", &json!(choice))
            .await
            .map(|_| ())
    }

    /// Gives the single file input these files (an empty slice clears it).
    pub async fn set_files<P: AsRef<Path>>(&self, files: &[P]) -> Result<()> {
        if files.is_empty() {
            // CDP drops an empty `files` list from the request, so clear from inside the page.
            return self
                .ok("clearing the files of", "clear_files", &Value::Null)
                .await
                .map(|_| ());
        }
        let files = files
            .iter()
            .map(|f| f.as_ref().to_string_lossy().into_owned())
            .collect();
        self.page.set_input_files(&self.steps, files).await
    }

    /// Gives the single file input a file with this name and content (written to a temp file).
    pub async fn set_file_bytes(&self, name: &str, bytes: &[u8]) -> Result<()> {
        let dir = crate::browser::profile_dir()
            .join("uploads")
            .join(Uuid::now_v7().to_string());
        tokio::fs::create_dir_all(&dir).await?;
        let path = dir.join(name);
        tokio::fs::write(&path, bytes).await?;
        self.set_files(&[path]).await
    }

    /// Writes a PNG of the single match.
    pub async fn screenshot(&self, path: impl AsRef<Path>) -> Result<()> {
        let b = self.bounding_box().await?;
        let clip = Viewport {
            x: b.x,
            y: b.y,
            width: b.width,
            height: b.height,
            scale: 1.0,
        };
        self.page.shoot(path.as_ref(), Some(clip), false).await
    }

    // --- expecting --------------------------------------------------------------------------------

    /// Retries until exactly one element matches and it is visible.
    pub async fn expect_visible(&self) -> Result<()> {
        let what = format!("expecting {} to be visible", self.desc);
        poll_for(what, self.page.timeout(), || async {
            let r = self.act("info", &Value::Null).await?;
            Ok(match (r["count"].as_u64().unwrap_or(0), &r["visible"][0]) {
                (1, Value::Bool(true)) => Poll::Ready(()),
                (1, _) => Poll::Retry("the element is not visible".to_owned()),
                (0, _) => Poll::Retry("no element matches".to_owned()),
                (n, _) => Poll::Retry(format!("strict mode violation: {n} elements match")),
            })
        })
        .await
    }

    /// Retries until no match is visible (none at all counts).
    pub async fn expect_hidden(&self) -> Result<()> {
        let what = format!("expecting {} to be hidden", self.desc);
        poll_for(what, self.page.timeout(), || async {
            let r = self.act("info", &Value::Null).await?;
            let shown = r["visible"]
                .as_array()
                .map_or(0, |v| v.iter().filter(|b| **b == Value::Bool(true)).count());
            Ok(if shown == 0 {
                Poll::Ready(())
            } else {
                Poll::Retry(format!("{shown} matching element(s) are visible"))
            })
        })
        .await
    }

    /// Retries until exactly `n` elements match (visible ones only for role, label and text finders).
    pub async fn expect_count(&self, n: usize) -> Result<()> {
        let what = format!("expecting {} to match {n} element(s)", self.desc);
        poll_for(what, self.page.timeout(), || async {
            let have = self.count().await?;
            Ok(if have == n {
                Poll::Ready(())
            } else {
                Poll::Retry(format!("{have} match"))
            })
        })
        .await
    }

    async fn expect_field(&self, what: &str, field: &str, name: Name<'_>) -> Result<()> {
        let arg = json!({"field": field, "name": name.json()});
        let what = format!("expecting {} {what} {}", self.desc, name.describe());
        poll_for(what, self.page.timeout(), || async {
            let r = self.act("match", &arg).await?;
            if r["ok"] != Value::Bool(true) {
                return Ok(Poll::Retry(
                    r["reason"].as_str().unwrap_or("not ok").to_owned(),
                ));
            }
            Ok(if r["match"] == Value::Bool(true) {
                Poll::Ready(())
            } else {
                Poll::Retry(format!("got {}", r["actual"]))
            })
        })
        .await
    }

    /// Retries until the single match's text (whitespace collapsed) matches `name`. `Exact` is the
    /// whole text, `Contains` and `Re` the substring and regex forms of `toContainText`.
    pub async fn expect_text(&self, name: Name<'_>) -> Result<()> {
        self.expect_field("to have text", "text", name).await
    }

    /// Retries until the single input's value matches (exactly, not whitespace-collapsed, for `Exact`).
    pub async fn expect_value(&self, name: Name<'_>) -> Result<()> {
        self.expect_field("to have value", "value", name).await
    }

    /// Retries until the attribute matches; a missing attribute never matches.
    pub async fn expect_attr(&self, attribute: &str, name: Name<'_>) -> Result<()> {
        self.expect_field(
            &format!("to have attribute {attribute} matching"),
            &format!("attr:{attribute}"),
            name,
        )
        .await
    }

    /// Retries until the computed style property matches.
    pub async fn expect_css(&self, property: &str, name: Name<'_>) -> Result<()> {
        self.expect_field(
            &format!("to have style {property} matching"),
            &format!("css:{property}"),
            name,
        )
        .await
    }

    /// Retries until the single checkbox, radio or switch has this state.
    pub async fn expect_checked(&self, checked: bool) -> Result<()> {
        let what = format!(
            "expecting {} to be {}checked",
            self.desc,
            if checked { "" } else { "un" }
        );
        poll_for(what, self.page.timeout(), || async {
            Ok(if self.is_checked().await? == checked {
                Poll::Ready(())
            } else {
                Poll::Retry("other state".to_owned())
            })
        })
        .await
    }

    /// Retries until the single match is enabled.
    pub async fn expect_enabled(&self) -> Result<()> {
        self.expect_enabled_is(true).await
    }

    /// Retries until the single match is disabled.
    pub async fn expect_disabled(&self) -> Result<()> {
        self.expect_enabled_is(false).await
    }

    async fn expect_enabled_is(&self, want: bool) -> Result<()> {
        let what = format!(
            "expecting {} to be {}",
            self.desc,
            if want { "enabled" } else { "disabled" }
        );
        self.until(what, "enabled", &Value::Null, |r| {
            (r["value"] == Value::Bool(want)).then_some(())
        })
        .await
        .map_err(|e| anyhow!("{e}"))
    }
}
