//! [`SafeClient`]: fetches a URL the server did not write, under the safety policy.

use std::fmt;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use reqwest::dns::Resolve;
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, HeaderMap, HeaderValue, LOCATION};
use reqwest::redirect;
use reqwest::{Client, Url};
use url::Host;

use super::error::NetError;
use super::resolve::{Guarded, Reach, SystemResolver, permits};
use crate::config::Config;

/// Redirects followed before a fetch fails.
const MAX_REDIRECTS: u8 = 4;

/// What happens when a body is larger than [`Fetch::max_bytes`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overflow {
    /// Return the first `max_bytes` bytes with [`Fetched::truncated`] set.
    Truncate,
    /// Fail with [`NetError::TooLarge`], before reading the body when `Content-Length` says so.
    Fail,
}

/// One request.
#[derive(Clone, Copy, Debug)]
pub struct Fetch<'a> {
    /// The `Accept` header.
    pub accept: &'a str,
    /// Most body bytes read.
    pub max_bytes: usize,
    /// Limit for the whole fetch: every hop, the headers and the body.
    pub timeout: Duration,
    /// What to do with a longer body.
    pub overflow: Overflow,
}

/// A fetched response.
#[derive(Clone, Debug)]
pub struct Fetched {
    /// The URL the body came from, after redirects.
    pub url: Url,
    /// The HTTP status, always a success.
    pub status: u16,
    /// The `Content-Type` header, empty when missing.
    pub content_type: String,
    /// At most [`Fetch::max_bytes`] bytes.
    pub body: Vec<u8>,
    /// The body had more bytes than were kept.
    pub truncated: bool,
}

/// One `POST`.
#[derive(Debug)]
pub struct PostRequest<'a> {
    /// The request body.
    pub body: Vec<u8>,
    /// The `Content-Type` header.
    pub content_type: &'a str,
    /// Extra headers, such as `Date`, `Digest` and `Signature`.
    pub headers: &'a [(&'a str, String)],
    /// Most response bytes read; the rest is dropped.
    pub max_response_bytes: usize,
    /// Limit for the whole request.
    pub timeout: Duration,
}

/// The answer to a `POST`, whatever its status.
#[derive(Clone, Debug)]
pub struct Posted {
    /// The HTTP status.
    pub status: u16,
    /// At most [`PostRequest::max_response_bytes`] bytes of the body.
    pub body: Vec<u8>,
}

/// Which URLs [`SafeClient`] accepts beyond the address classes.
#[derive(Clone, Copy, Debug)]
struct Policy {
    any_port: bool,
    reach: Reach,
}

/// A `reqwest` client that refuses private addresses, odd ports and endless redirects.
///
/// Redirects are followed here, not by `reqwest`, so that every hop is checked again.
#[derive(Clone)]
pub struct SafeClient {
    http: Client,
    resolver: Arc<dyn Resolve>,
    user_agent: Box<str>,
    policy: Policy,
}

impl fmt::Debug for SafeClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SafeClient")
            .field("policy", &self.policy)
            .finish_non_exhaustive()
    }
}

impl SafeClient {
    /// Builds the strict client: ports 80 and 443 only, unless the instance allows plain-http
    /// federation (development), and no non-public address in any case.
    ///
    /// # Panics
    ///
    /// When the bundled TLS settings are unusable, which a build of the server catches in every
    /// test that creates an `AppState`.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        Self::with_resolver(config, Arc::new(SystemResolver))
    }

    /// Like [`Self::new`], but names resolve through `resolver`. For tests that need a host name
    /// to resolve to a chosen address; the addresses it returns are still filtered.
    ///
    /// # Panics
    ///
    /// As [`Self::new`].
    #[must_use]
    pub fn with_resolver(config: &Config, resolver: Arc<dyn Resolve>) -> Self {
        let user_agent = format!(
            "Mozilla/5.0 (compatible; tuitaBot/{}; +{})",
            env!("CARGO_PKG_VERSION"),
            config.public_url
        );
        let policy = Policy {
            any_port: config.federation_allow_http,
            reach: if config.federation_allow_private_nets {
                Reach::Anywhere
            } else {
                Reach::Public
            },
        };
        Self::build(resolver, user_agent.into(), policy)
    }

    /// Lets this client reach `127.0.0.0/8` on any port, so tests can fetch from a local server.
    /// Production code builds its client with [`Self::new`] and never calls this.
    #[doc(hidden)]
    #[must_use]
    pub fn allow_loopback_for_tests(self) -> Self {
        let policy = Policy {
            any_port: true,
            reach: Reach::Loopback,
        };
        Self::build(self.resolver, self.user_agent, policy)
    }

    fn build(resolver: Arc<dyn Resolve>, user_agent: Box<str>, policy: Policy) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_static("en;q=0.9, *;q=0.5"),
        );
        let http = Client::builder()
            .tls_backend_preconfigured(tls_config())
            .dns_resolver(Guarded::new(Arc::clone(&resolver), policy.reach))
            .redirect(redirect::Policy::none())
            .no_proxy()
            .user_agent(&*user_agent)
            .default_headers(headers)
            .build()
            .expect("the TLS and resolver settings are fixed at build time");
        Self {
            http,
            resolver,
            user_agent,
            policy,
        }
    }

    /// Fetches `url`, following at most four redirects.
    ///
    /// # Errors
    ///
    /// A [`NetError`] naming the policy rule that refused the URL or a redirect hop, or the
    /// transport, status, size or time limit that ended the fetch.
    pub async fn get(&self, url: &str, fetch: Fetch<'_>) -> Result<Fetched, NetError> {
        self.fetch(url, fetch, &[], false).await
    }

    /// Like [`Self::get`], for a federation request (a signed actor or object fetch, WebFinger),
    /// with extra request headers (a signature, say). They go only to the host first asked: a
    /// redirect to another host is fetched without them.
    ///
    /// Federation traffic must use `https` on every hop, unless the instance allows plain http
    /// (`FEDERATION_ALLOW_HTTP`, development): a signed request or an actor over plain http can
    /// be read or forged by anyone on the path. [`Self::get`] still accepts plain http on port
    /// 80, for link previews.
    ///
    /// # Errors
    ///
    /// As [`Self::get`]; also [`NetError::PlainHttp`] for plain http and [`NetError::BadUrl`]
    /// for a header name or value that is invalid.
    pub async fn get_with(
        &self,
        url: &str,
        fetch: Fetch<'_>,
        extra_headers: &[(&str, String)],
    ) -> Result<Fetched, NetError> {
        self.fetch(url, fetch, extra_headers, true).await
    }

    async fn fetch(
        &self,
        url: &str,
        fetch: Fetch<'_>,
        extra_headers: &[(&str, String)],
        federation: bool,
    ) -> Result<Fetched, NetError> {
        let url = Url::parse(url).map_err(|_| NetError::BadUrl("not an absolute url"))?;
        tokio::time::timeout(
            fetch.timeout,
            self.follow(url, fetch, extra_headers, federation),
        )
        .await
        .map_err(|_| NetError::Timeout)?
    }

    /// `POST`s `request.body` to `url` under the same address and port policy as [`Self::get`].
    /// Redirects are not followed: a `3xx` is a status like any other. Every status is returned,
    /// and the caller decides what it means.
    ///
    /// # Errors
    ///
    /// A [`NetError`] for a refused URL, a transport failure, the time limit, or an invalid
    /// header.
    pub async fn post(&self, url: &str, request: PostRequest<'_>) -> Result<Posted, NetError> {
        let url = Url::parse(url).map_err(|_| NetError::BadUrl("not an absolute url"))?;
        let timeout = request.timeout;
        let work = async {
            self.check(&url, true)?;
            let mut builder = self
                .http
                .post(url)
                .header(reqwest::header::CONTENT_TYPE, request.content_type)
                .body(request.body);
            builder = with_headers(builder, request.headers)?;
            let response = builder.send().await?;
            let status = response.status().as_u16();
            let fetched = read_body(
                response,
                Url::parse("http://post.invalid/").map_err(|_| NetError::BadUrl("internal"))?,
                Fetch {
                    accept: "*/*",
                    max_bytes: request.max_response_bytes,
                    timeout,
                    overflow: Overflow::Truncate,
                },
            )
            .await?;
            Ok(Posted {
                status,
                body: fetched.body,
            })
        };
        tokio::time::timeout(timeout, work)
            .await
            .map_err(|_| NetError::Timeout)?
    }

    async fn follow(
        &self,
        mut url: Url,
        fetch: Fetch<'_>,
        extra_headers: &[(&str, String)],
        federation: bool,
    ) -> Result<Fetched, NetError> {
        let first_origin = url.origin();
        let mut redirects = 0;
        loop {
            self.check(&url, federation)?;
            let mut builder = self.http.get(url.clone()).header(ACCEPT, fetch.accept);
            if url.origin() == first_origin {
                builder = with_headers(builder, extra_headers)?;
            }
            let response = builder.send().await?;
            let status = response.status();
            if status.is_redirection() {
                let location = response
                    .headers()
                    .get(LOCATION)
                    .and_then(|value| value.to_str().ok())
                    .ok_or(NetError::Status(status.as_u16()))?;
                if redirects == MAX_REDIRECTS {
                    return Err(NetError::TooManyRedirects);
                }
                redirects += 1;
                url = url
                    .join(location)
                    .map_err(|_| NetError::BadUrl("bad redirect target"))?;
                continue;
            }
            if !status.is_success() {
                return Err(NetError::Status(status.as_u16()));
            }
            return read_body(response, url, fetch).await;
        }
    }

    /// Refuses the URL before any connection: scheme, credentials, port and IP literals.
    /// Names are checked when they resolve.
    fn check(&self, url: &Url, federation: bool) -> Result<(), NetError> {
        if !matches!(url.scheme(), "http" | "https") {
            return Err(NetError::BadScheme);
        }
        if federation && url.scheme() != "https" && !self.policy.any_port {
            return Err(NetError::PlainHttp);
        }
        if !url.username().is_empty() || url.password().is_some() {
            return Err(NetError::BadUrl("credentials in url"));
        }
        let port = url.port_or_known_default().unwrap_or(0);
        if !self.policy.any_port && port != 80 && port != 443 {
            return Err(NetError::BadPort(port));
        }
        let literal = match url.host() {
            None => return Err(NetError::BadUrl("no host")),
            Some(Host::Domain(_)) => return Ok(()),
            Some(Host::Ipv4(ip)) => IpAddr::V4(ip),
            Some(Host::Ipv6(ip)) => IpAddr::V6(ip),
        };
        if permits(literal, self.policy.reach) {
            Ok(())
        } else {
            Err(NetError::BlockedAddress)
        }
    }
}

/// Adds `headers` to `builder`; a name or value `reqwest` refuses is a [`NetError::BadUrl`].
fn with_headers(
    mut builder: reqwest::RequestBuilder,
    headers: &[(&str, String)],
) -> Result<reqwest::RequestBuilder, NetError> {
    for (name, value) in headers {
        let name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| NetError::BadUrl("invalid header name"))?;
        let value =
            HeaderValue::from_str(value).map_err(|_| NetError::BadUrl("invalid header value"))?;
        builder = builder.header(name, value);
    }
    Ok(builder)
}

async fn read_body(
    mut response: reqwest::Response,
    url: Url,
    fetch: Fetch<'_>,
) -> Result<Fetched, NetError> {
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    if fetch.overflow == Overflow::Fail
        && response
            .content_length()
            .is_some_and(|len| len > fetch.max_bytes as u64)
    {
        return Err(NetError::TooLarge);
    }
    let mut body = Vec::new();
    let mut truncated = false;
    while let Some(chunk) = response.chunk().await? {
        let room = fetch.max_bytes - body.len();
        if chunk.len() > room {
            if fetch.overflow == Overflow::Fail {
                return Err(NetError::TooLarge);
            }
            body.extend_from_slice(&chunk[..room]);
            truncated = true;
            break;
        }
        body.extend_from_slice(&chunk);
    }
    Ok(Fetched {
        url,
        status,
        content_type,
        body,
        truncated,
    })
}

/// rustls with the ring provider (the one sqlx links) and the bundled Mozilla roots, so the
/// server needs neither aws-lc nor a system certificate store.
pub(crate) fn tls_config() -> rustls::ClientConfig {
    let roots = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .expect("ring supports the default protocol versions")
        .with_root_certificates(roots)
        .with_no_client_auth();
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    config
}
