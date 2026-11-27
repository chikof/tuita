//! axe-core runs (WCAG 2 A and AA).

use std::fmt::{self, Write as _};

use anyhow::{Context, Result};
use serde_json::{Value, json};

use crate::Page;

/// axe-core 4.14.0 (MPL-2.0, see `js/axe.LICENSE`), injected through CDP so the page CSP does not apply.
const AXE: &str = include_str!("../js/axe.min.js");

/// The tags `axe()` checks.
const WCAG_AA: [&str; 2] = ["wcag2a", "wcag2aa"];

/// One axe rule that failed on the page.
#[derive(Debug, Clone)]
pub struct Violation {
    /// Rule id, e.g. `color-contrast`.
    pub id: String,
    /// `minor`, `moderate`, `serious` or `critical`.
    pub impact: Option<String>,
    /// One-line description of the rule.
    pub help: String,
    /// The offending elements.
    pub nodes: Vec<ViolationNode>,
}

/// One element that broke a rule.
#[derive(Debug, Clone)]
pub struct ViolationNode {
    /// CSS selector path, nested frames joined by a space.
    pub target: String,
    /// The element's outer HTML, truncated by axe.
    pub html: String,
    /// What to change, in axe's words.
    pub summary: String,
}

impl Violation {
    /// All violations as one assertion message.
    #[must_use]
    pub fn report(violations: &[Self]) -> String {
        let mut out = String::new();
        for v in violations {
            let _ = write!(out, "{v}");
        }
        out
    }

    fn parse(value: &Value) -> Option<Self> {
        let text = |v: &Value, key: &str| {
            v.get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned()
        };
        Some(Self {
            id: text(value, "id"),
            impact: value
                .get("impact")
                .and_then(Value::as_str)
                .map(str::to_owned),
            help: text(value, "help"),
            nodes: value
                .get("nodes")?
                .as_array()?
                .iter()
                .map(|n| ViolationNode {
                    target: text(n, "target"),
                    html: text(n, "html"),
                    summary: text(n, "summary"),
                })
                .collect(),
        })
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "[{}] {}: {}",
            self.impact.as_deref().unwrap_or("?"),
            self.id,
            self.help
        )?;
        for node in &self.nodes {
            writeln!(f, "    {}\n      {}", node.target, node.html)?;
        }
        Ok(())
    }
}

impl Page {
    /// Runs `axe.run(document, {runOnly: ['wcag2a','wcag2aa']})` and returns the violations
    /// (empty means clean).
    pub async fn axe(&self) -> Result<Vec<Violation>> {
        self.axe_with(None, &WCAG_AA).await
    }

    /// [`Page::axe`] with another rule set (`tags` as in axe's `runOnly`) and an optional CSS
    /// selector that limits the scan to one subtree.
    pub async fn axe_with(&self, context: Option<&str>, tags: &[&str]) -> Result<Vec<Violation>> {
        self.eval(&format!(
            "if (typeof window.axe === 'undefined') {{ {AXE}\n}} true"
        ))
        .await?;
        let script = format!(
            "(async (context, tags) => {{
                const root = context ? document.querySelector(context) : document;
                if (!root) throw new Error('axe context not found: ' + context);
                const result = await axe.run(root, {{ runOnly: tags }});
                return result.violations.map((v) => ({{
                    id: v.id, impact: v.impact, help: v.help,
                    nodes: v.nodes.map((n) => ({{
                        target: n.target.flat(Infinity).join(' '), html: n.html,
                        summary: n.failureSummary || ''
                    }}))
                }}));
            }})({}, {})",
            json!(context),
            json!(tags),
        );
        let raw = self.eval(&script).await.context("axe.run")?;
        Ok(raw
            .as_array()
            .map(|all| all.iter().filter_map(Violation::parse).collect())
            .unwrap_or_default())
    }
}

impl Page {
    /// [`Page::axe`] limited to the subtrees `selectors` match (axe's `include` list): the union of
    /// all of them, one run.
    pub async fn axe_include(&self, selectors: &[&str]) -> Result<Vec<Violation>> {
        self.eval(&format!(
            "if (typeof window.axe === 'undefined') {{ {AXE}\n}} true"
        ))
        .await?;
        let script = format!(
            "(async (selectors, tags) => {{
                const context = {{ include: selectors.map((s) => [s]) }};
                const result = await axe.run(context, {{ runOnly: tags }});
                return result.violations.map((v) => ({{
                    id: v.id, impact: v.impact, help: v.help,
                    nodes: v.nodes.map((n) => ({{
                        target: n.target.flat(Infinity).join(' '), html: n.html,
                        summary: n.failureSummary || ''
                    }}))
                }}));
            }})({}, {})",
            json!(selectors),
            json!(WCAG_AA),
        );
        let raw = self.eval(&script).await.context("axe.run")?;
        Ok(raw
            .as_array()
            .map(|all| all.iter().filter_map(Violation::parse).collect())
            .unwrap_or_default())
    }
}
