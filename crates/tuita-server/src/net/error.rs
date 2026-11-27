//! Why a [`SafeClient`](super::SafeClient) fetch failed.

use std::error::Error;

use thiserror::Error;

use super::resolve::BlockedHost;

/// A refused or failed fetch.
#[derive(Debug, Error)]
pub enum NetError {
    /// The host is, or resolves only to, a loopback, private, link-local or otherwise
    /// non-public address.
    #[error("blocked address")]
    BlockedAddress,
    /// The scheme is not `http` or `https`.
    #[error("only http and https are fetched")]
    BadScheme,
    /// A federation request names plain `http` and the instance does not allow it.
    #[error("plain http is not used for federation")]
    PlainHttp,
    /// The port is neither 80 nor 443.
    #[error("port {0} is not allowed")]
    BadPort(u16),
    /// The URL cannot be fetched; the text says why.
    #[error("bad url: {0}")]
    BadUrl(&'static str),
    /// The server redirected more often than allowed.
    #[error("too many redirects")]
    TooManyRedirects,
    /// The body is larger than the caller accepts.
    #[error("response too large")]
    TooLarge,
    /// The final response was not a success.
    #[error("unexpected status {0}")]
    Status(u16),
    /// The whole fetch took longer than allowed.
    #[error("timed out")]
    Timeout,
    /// Connecting, TLS or reading failed.
    #[error("transport error")]
    Transport(#[source] reqwest::Error),
}

impl From<reqwest::Error> for NetError {
    fn from(err: reqwest::Error) -> Self {
        let mut cause: Option<&(dyn Error + 'static)> = Some(&err);
        while let Some(inner) = cause {
            if inner.is::<BlockedHost>() {
                return Self::BlockedAddress;
            }
            cause = inner.source();
        }
        if err.is_timeout() {
            Self::Timeout
        } else {
            Self::Transport(err)
        }
    }
}
// 24: the tag plus the largest payload, `&'static str` (16); `reqwest::Error` is one box.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<NetError>() == 24);
