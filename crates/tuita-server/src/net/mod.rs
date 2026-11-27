//! Outbound HTTP: [`SafeClient`] is the only way the server fetches a URL it did not write.
//!
//! A fetch is checked three times: the URL (scheme, port, credentials, IP literal) before every
//! request and redirect hop, the resolved addresses at connect time, and the body size while it
//! streams in.

mod client;
mod error;
mod resolve;

pub(crate) use client::tls_config;
pub use client::{Fetch, Fetched, Overflow, PostRequest, Posted, SafeClient};
pub use error::NetError;
pub use resolve::is_blocked_ip;
