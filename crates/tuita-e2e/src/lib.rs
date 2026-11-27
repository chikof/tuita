//! Browser end-to-end harness: drives one shared headless Chromium over CDP against a running tuita.
//!
//! Tests are plain `#[tokio::test]` functions in `tests/e2e/*.rs`, started by `scripts/e2e.sh`,
//! which builds the server, starts it against a throwaway database and sets the `E2E_*`
//! environment ([`env()`]). Typical shape:
//!
//! ```ignore
//! let ctx = Ctx::new().await?;                       // isolated cookies, unique X-Forwarded-For
//! let page = ctx.page().await?;
//! page.goto("/signup").await?;
//! page.label("Handle").fill("ana").await?;
//! page.role("button", Name::Exact("Create account")).click().await?;
//! page.role("navigation", Name::Exact("Main")).first().expect_visible().await?;
//! ```
//!
//! Locators auto-wait: every action and expectation retries every 50 ms up to `E2E_TIMEOUT_MS`.
//! They are strict: an action on a locator that matches several elements fails
//! (narrow it with [`Loc::first`], [`Loc::nth`], [`Loc::has_text`] ...).
//!
//! Every fallible function returns [`anyhow::Result`] and fails when the browser, the page or the
//! awaited condition misbehaves; the crate allows `missing_errors_doc` for that reason.

#![allow(
    clippy::missing_errors_doc,
    reason = "every fallible function fails because the browser or the page misbehaves; the message says how"
)]

mod axe;
mod browser;
mod ctx;
mod env;
pub mod helpers;
mod loc;
mod name;
mod page;
mod poll;

pub use axe::{Violation, ViolationNode};
pub use ctx::{Ctx, CtxOptions};
pub use env::{Env, chrome_available, env};
pub use loc::{BoundingBox, Loc};
pub use name::Name;
pub use page::{Authenticator, Motion, Page, Scheme};
pub use poll::{INTERVAL, Poll, poll, poll_for};

/// Result of every harness call.
pub use anyhow::Result;
