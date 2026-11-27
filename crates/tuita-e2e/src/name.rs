//! How a locator compares text.

use serde_json::{Value, json};

/// A text pattern for [`Page::role`](crate::Page::role), `label`, `text` and the `expect_*`
/// methods. Whitespace runs collapse to one space and U+2068/U+2069 (Fluent isolation marks) are
/// stripped from the page text before comparing.
#[derive(Debug, Clone, Copy)]
pub enum Name<'a> {
    /// The whole text, case-sensitive (`exact: true`).
    Exact(&'a str),
    /// A case-insensitive substring: what a bare string means. `&str` converts to this.
    Contains(&'a str),
    /// A JS `RegExp` source, no flags.
    Re(&'a str),
    /// A JS `RegExp` source with the `i` flag.
    ReI(&'a str),
    /// Anything.
    Any,
}

impl<'a> From<&'a str> for Name<'a> {
    fn from(text: &'a str) -> Self {
        Self::Contains(text)
    }
}

impl Name<'_> {
    /// The wire form `find.js` reads.
    pub(crate) fn json(&self) -> Value {
        match self {
            Self::Exact(v) => json!({"t": "exact", "v": v}),
            Self::Contains(v) => json!({"t": "sub", "v": v}),
            Self::Re(v) => json!({"t": "re", "v": v}),
            Self::ReI(v) => json!({"t": "ci", "v": v}),
            Self::Any => json!({"t": "re", "v": ""}),
        }
    }

    /// Short form for failure messages.
    pub(crate) fn describe(&self) -> String {
        match self {
            Self::Exact(v) => format!("{v:?} (exact)"),
            Self::Contains(v) => format!("{v:?}"),
            Self::Re(v) => format!("/{v}/"),
            Self::ReI(v) => format!("/{v}/i"),
            Self::Any => "*".to_owned(),
        }
    }
}
