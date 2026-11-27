//! `SafeClient`: which URLs it refuses, how it follows redirects, and its size and time limits.
#![allow(missing_docs)]

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header::{CONTENT_TYPE, LOCATION, USER_AGENT};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use tokio::net::TcpListener;
use tuita_server::Config;
use tuita_server::net::{Fetch, NetError, Overflow, PostRequest, SafeClient};

fn config(allow_http: bool) -> Config {
    Config::from_lookup(|var| match var {
        "DATABASE_URL" => Some("postgres://unused".to_owned()),
        "PUBLIC_URL" => Some("https://social.example".to_owned()),
        "FEDERATION_ALLOW_HTTP" => Some(allow_http.to_string()),
        _ => None,
    })
    .expect("valid test config")
}

fn fetch(max_bytes: usize) -> Fetch<'static> {
    Fetch {
        accept: "*/*",
        max_bytes,
        timeout: Duration::from_secs(5),
        overflow: Overflow::Truncate,
    }
}

/// Resolves `internal.test` to a private address, `edge.test` to the local test server (with a
/// private address next to it), and everything else to a private address.
struct TestResolver;

impl Resolve for TestResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let addrs: Vec<SocketAddr> = match name.as_str() {
            "edge.test" => vec![
                "10.0.0.1:0".parse().expect("addr"),
                "127.0.0.1:0".parse().expect("addr"),
            ],
            _ => vec!["10.0.0.1:0".parse().expect("addr")],
        };
        Box::pin(async move { Ok(Box::new(addrs.into_iter()) as Addrs) })
    }
}

fn strict() -> SafeClient {
    SafeClient::with_resolver(&config(false), Arc::new(TestResolver))
}

fn loopback() -> SafeClient {
    strict().allow_loopback_for_tests()
}

#[derive(Clone, Default)]
struct Sent(Arc<AtomicUsize>);

async fn serve(app: Router) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    addr
}

fn redirect(to: &str) -> Response {
    (StatusCode::FOUND, [(LOCATION, to)]).into_response()
}

async fn hops(Path(n): Path<u32>) -> Response {
    match n {
        0 => "arrived".into_response(),
        n => redirect(&format!("/hops/{}", n - 1)),
    }
}

async fn endless(State(sent): State<Sent>) -> Response {
    const CHUNK: usize = 64 * 1024;
    const CHUNKS: usize = 4096;
    let stream = futures_util::stream::unfold(0usize, move |i| {
        let sent = sent.clone();
        async move {
            if i == CHUNKS {
                return None;
            }
            sent.0.fetch_add(CHUNK, Ordering::SeqCst);
            Some((Ok::<_, std::convert::Infallible>(vec![b'a'; CHUNK]), i + 1))
        }
    });
    Body::from_stream(stream).into_response()
}

async fn stall_in_body() -> Response {
    let stream = futures_util::stream::unfold(0u8, |i| async move {
        if i == 0 {
            Some((Ok::<_, std::convert::Infallible>(b"start".to_vec()), 1))
        } else {
            tokio::time::sleep(Duration::from_secs(60)).await;
            None
        }
    });
    Body::from_stream(stream).into_response()
}

async fn echo_headers(headers: HeaderMap) -> String {
    let agent = headers.get(USER_AGENT).and_then(|v| v.to_str().ok());
    let accept = headers.get("accept").and_then(|v| v.to_str().ok());
    format!(
        "{}|{}",
        agent.unwrap_or_default(),
        accept.unwrap_or_default()
    )
}

async fn test_server() -> (SocketAddr, Sent) {
    let sent = Sent::default();
    let app = Router::new()
        .route("/hops/{n}", get(hops))
        .route(
            "/hello",
            get(|| async { ([(CONTENT_TYPE, "text/plain")], "hello") }),
        )
        .route("/headers", get(echo_headers))
        .route("/missing", get(|| async { StatusCode::NOT_FOUND }))
        .route(
            "/to-metadata",
            get(|| async { redirect("http://169.254.169.254/latest") }),
        )
        .route(
            "/to-internal",
            get(|| async { redirect("http://internal.test/") }),
        )
        .route("/to-ftp", get(|| async { redirect("ftp://example.com/") }))
        .route(
            "/to-credentials",
            get(|| async { redirect("http://u:p@example.com/") }),
        )
        .route("/bare-redirect", get(|| async { StatusCode::FOUND }))
        .route("/exact", get(|| async { "x".repeat(1000) }))
        .route("/endless", get(endless))
        .route(
            "/stall",
            get(|| async {
                tokio::time::sleep(Duration::from_secs(60)).await;
                "late"
            }),
        )
        .route("/stall-body", get(stall_in_body))
        .with_state(sent.clone());
    (serve(app).await, sent)
}

#[tokio::test]
async fn refuses_ip_literals_before_connecting() {
    let client = strict();
    for url in [
        "http://127.0.0.1/",
        "http://127.0.0.1:80/",
        "http://169.254.169.254/latest/meta-data",
        "http://[::1]/",
        "http://[::ffff:127.0.0.1]/",
        "http://[fe80::1]/",
        "http://[fd00::1]/",
        "http://0.0.0.0/",
        "http://10.1.2.3/",
        "http://192.168.0.1/",
        "http://172.16.0.1/",
        "http://100.64.0.1/",
        "https://198.18.0.1/",
        "http://255.255.255.255/",
        "http://2130706433/",
        "http://0x7f.1/",
    ] {
        let err = client.get(url, fetch(1024)).await.expect_err(url);
        assert!(matches!(err, NetError::BlockedAddress), "{url}: {err:?}");
    }
}

#[tokio::test]
async fn refuses_host_that_resolves_to_a_private_address() {
    let err = strict()
        .get("http://internal.test/", fetch(1024))
        .await
        .expect_err("private address");
    assert!(matches!(err, NetError::BlockedAddress), "{err:?}");
    let err = strict()
        .get("https://internal.test/", fetch(1024))
        .await
        .expect_err("private address over https");
    assert!(matches!(err, NetError::BlockedAddress), "{err:?}");
}

#[tokio::test]
async fn refuses_unlisted_ports_schemes_and_credentials() {
    let client = strict();
    let err = client
        .get("http://example.com:8080/", fetch(1024))
        .await
        .expect_err("port 8080");
    assert!(matches!(err, NetError::BadPort(8080)), "{err:?}");
    let err = client
        .get("https://example.com:22/", fetch(1024))
        .await
        .expect_err("port 22");
    assert!(matches!(err, NetError::BadPort(22)), "{err:?}");
    let err = client
        .get("ftp://example.com/", fetch(1024))
        .await
        .expect_err("ftp");
    assert!(matches!(err, NetError::BadScheme), "{err:?}");
    let err = client
        .get("http://user:secret@example.com/", fetch(1024))
        .await
        .expect_err("credentials");
    assert!(matches!(err, NetError::BadUrl(_)), "{err:?}");
    let err = client
        .get("not a url", fetch(1024))
        .await
        .expect_err("garbage");
    assert!(matches!(err, NetError::BadUrl(_)), "{err:?}");
}

#[tokio::test]
async fn federation_dev_mode_opens_ports_but_not_addresses() {
    let client = SafeClient::with_resolver(&config(true), Arc::new(TestResolver));
    let err = client
        .get("http://127.0.0.1:8080/", fetch(1024))
        .await
        .expect_err("loopback");
    assert!(matches!(err, NetError::BlockedAddress), "{err:?}");
    let err = client
        .get("http://internal.test:8080/", fetch(1024))
        .await
        .expect_err("private");
    assert!(matches!(err, NetError::BlockedAddress), "{err:?}");
}

#[tokio::test]
async fn loopback_relaxation_keeps_other_classes_blocked() {
    for url in [
        "http://10.0.0.1/",
        "http://169.254.169.254/",
        "http://[::1]/",
    ] {
        let err = loopback().get(url, fetch(1024)).await.expect_err(url);
        assert!(matches!(err, NetError::BlockedAddress), "{url}: {err:?}");
    }
}

#[tokio::test]
async fn drops_blocked_addresses_and_connects_to_the_rest() {
    let (addr, _) = test_server().await;
    let url = format!("http://edge.test:{}/hello", addr.port());
    let got = loopback().get(&url, fetch(1024)).await.expect("fetched");
    assert_eq!(got.status, 200);
    assert_eq!(got.content_type, "text/plain");
    assert_eq!(got.body, b"hello");
    assert!(!got.truncated);
    assert_eq!(got.url.as_str(), url);
}

#[tokio::test]
async fn sends_bot_user_agent_and_accept() {
    let (addr, _) = test_server().await;
    let mut request = fetch(1024);
    request.accept = "text/html";
    let got = loopback()
        .get(
            &format!("http://127.0.0.1:{}/headers", addr.port()),
            request,
        )
        .await
        .expect("fetched");
    let text = String::from_utf8(got.body).expect("utf8");
    let expected = format!(
        "Mozilla/5.0 (compatible; tuitaBot/{}; +https://social.example)|text/html",
        env!("CARGO_PKG_VERSION")
    );
    assert_eq!(text, expected);
}

#[tokio::test]
async fn reports_non_success_status() {
    let (addr, _) = test_server().await;
    let base = format!("http://127.0.0.1:{}", addr.port());
    let err = loopback()
        .get(&format!("{base}/missing"), fetch(1024))
        .await
        .expect_err("404");
    assert!(matches!(err, NetError::Status(404)), "{err:?}");
    let err = loopback()
        .get(&format!("{base}/bare-redirect"), fetch(1024))
        .await
        .expect_err("redirect without location");
    assert!(matches!(err, NetError::Status(302)), "{err:?}");
}

#[tokio::test]
async fn follows_up_to_four_redirects() {
    let (addr, _) = test_server().await;
    let base = format!("http://127.0.0.1:{}", addr.port());
    let got = loopback()
        .get(&format!("{base}/hops/4"), fetch(1024))
        .await
        .expect("four redirects");
    assert_eq!(got.body, b"arrived");
    assert_eq!(got.url.path(), "/hops/0");
    let err = loopback()
        .get(&format!("{base}/hops/5"), fetch(1024))
        .await
        .expect_err("five redirects");
    assert!(matches!(err, NetError::TooManyRedirects), "{err:?}");
}

#[tokio::test]
async fn rechecks_every_redirect_hop() {
    let (addr, _) = test_server().await;
    let base = format!("http://127.0.0.1:{}", addr.port());
    for (path, ok) in [
        ("/to-metadata", matches_blocked as fn(&NetError) -> bool),
        ("/to-internal", matches_blocked),
        ("/to-ftp", |e| matches!(e, NetError::BadScheme)),
        ("/to-credentials", |e| matches!(e, NetError::BadUrl(_))),
    ] {
        let err = loopback()
            .get(&format!("{base}{path}"), fetch(1024))
            .await
            .expect_err(path);
        assert!(ok(&err), "{path}: {err:?}");
    }
}

fn matches_blocked(err: &NetError) -> bool {
    matches!(err, NetError::BlockedAddress)
}

#[tokio::test]
async fn cuts_an_oversize_body_without_reading_it_all() {
    let (addr, sent) = test_server().await;
    let url = format!("http://127.0.0.1:{}/endless", addr.port());
    let got = loopback().get(&url, fetch(100_000)).await.expect("fetched");
    assert_eq!(got.body.len(), 100_000);
    assert!(got.truncated);
    let produced = sent.0.load(Ordering::SeqCst);
    assert!(
        produced < 64 * 1024 * 1024,
        "server produced {produced} of 256 MiB"
    );
}

#[tokio::test]
async fn overflow_fail_refuses_instead_of_truncating() {
    let (addr, sent) = test_server().await;
    let base = format!("http://127.0.0.1:{}", addr.port());
    let mut request = fetch(999);
    request.overflow = Overflow::Fail;
    let err = loopback()
        .get(&format!("{base}/exact"), request)
        .await
        .expect_err("declared length over the cap");
    assert!(matches!(err, NetError::TooLarge), "{err:?}");
    request.max_bytes = 100_000;
    let err = loopback()
        .get(&format!("{base}/endless"), request)
        .await
        .expect_err("streamed body over the cap");
    assert!(matches!(err, NetError::TooLarge), "{err:?}");
    assert!(sent.0.load(Ordering::SeqCst) < 64 * 1024 * 1024);
}

#[tokio::test]
async fn a_body_of_exactly_the_cap_is_not_truncated() {
    let (addr, _) = test_server().await;
    let got = loopback()
        .get(
            &format!("http://127.0.0.1:{}/exact", addr.port()),
            fetch(1000),
        )
        .await
        .expect("fetched");
    assert_eq!(got.body.len(), 1000);
    assert!(!got.truncated);
}

#[tokio::test]
async fn times_out_on_a_stalled_server() {
    let (addr, _) = test_server().await;
    let base = format!("http://127.0.0.1:{}", addr.port());
    let mut request = fetch(1024);
    request.timeout = Duration::from_millis(300);
    for path in ["/stall", "/stall-body"] {
        let started = std::time::Instant::now();
        let err = loopback()
            .get(&format!("{base}{path}"), request)
            .await
            .expect_err(path);
        assert!(matches!(err, NetError::Timeout), "{path}: {err:?}");
        assert!(started.elapsed() < Duration::from_secs(3), "{path}");
    }
}

#[test]
fn unfurl_timeout_is_six_seconds() {
    assert_eq!(
        tuita_server::workers::unfurl::FETCH_TIMEOUT,
        Duration::from_secs(6)
    );
}

#[tokio::test]
async fn private_nets_flag_reaches_loopback_and_private_hosts_but_keeps_the_caps() {
    let config = Config::from_lookup(|var| match var {
        "DATABASE_URL" => Some("postgres://unused".to_owned()),
        "PUBLIC_URL" => Some("https://social.example".to_owned()),
        "FEDERATION_ALLOW_HTTP" => Some("true".to_owned()),
        "FEDERATION_ALLOW_PRIVATE_NETS" => Some("true".to_owned()),
        _ => None,
    })
    .expect("valid test config");
    let client = SafeClient::with_resolver(&config, Arc::new(TestResolver));
    let addr = serve(Router::new().route("/hops/{n}", get(hops))).await;
    let got = client
        .get(&format!("http://{addr}/hops/2"), fetch(1024))
        .await
        .expect("loopback is reachable with the flag");
    assert_eq!(got.body, b"arrived");
    let err = client
        .get(&format!("http://{addr}/hops/9"), fetch(1024))
        .await
        .expect_err("the redirect cap still applies");
    assert!(matches!(err, NetError::TooManyRedirects), "{err:?}");
    let refused = client
        .get("http://[::1]:9/", fetch(1024))
        .await
        .expect_err("nothing listens");
    assert!(!matches!(refused, NetError::BlockedAddress), "{refused:?}");
}

async fn echo_x_test(headers: axum::http::HeaderMap) -> String {
    let header = |name: &str| {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("-")
            .to_owned()
    };
    format!("{}|{}", header("x-test"), header("accept"))
}

async fn post_status(body: axum::body::Bytes) -> Response {
    (
        StatusCode::IM_A_TEAPOT,
        format!("got {} bytes {}", body.len(), "x".repeat(5000)),
    )
        .into_response()
}

#[tokio::test]
async fn get_with_sends_extra_headers_and_post_returns_any_status_capped() {
    let addr = serve(
        Router::new()
            .route("/echo", get(echo_x_test))
            .route("/teapot", axum::routing::post(post_status))
            .route(
                "/moved",
                axum::routing::post(|| async { redirect("/echo") }),
            ),
    )
    .await;
    let client = loopback();
    let got = client
        .get_with(
            &format!("http://{addr}/echo"),
            fetch(1024),
            &[("x-test", "hello".to_owned())],
        )
        .await
        .expect("fetched");
    assert_eq!(got.body, b"hello|*/*");

    let posted = client
        .post(
            &format!("http://{addr}/teapot"),
            tuita_server::net::PostRequest {
                body: b"abc".to_vec(),
                content_type: "application/activity+json",
                headers: &[("x-test", "1".to_owned())],
                max_response_bytes: 10,
                timeout: Duration::from_secs(5),
            },
        )
        .await
        .expect("any status comes back");
    assert_eq!(posted.status, 418);
    assert_eq!(posted.body, b"got 3 byte");

    let moved = client
        .post(
            &format!("http://{addr}/moved"),
            tuita_server::net::PostRequest {
                body: Vec::new(),
                content_type: "application/activity+json",
                headers: &[],
                max_response_bytes: 10,
                timeout: Duration::from_secs(5),
            },
        )
        .await
        .expect("a redirect is only a status");
    assert_eq!(moved.status, 302);

    let refused = strict()
        .post(
            &format!("https://{addr}/teapot"),
            tuita_server::net::PostRequest {
                body: Vec::new(),
                content_type: "application/activity+json",
                headers: &[],
                max_response_bytes: 10,
                timeout: Duration::from_secs(5),
            },
        )
        .await
        .expect_err("the port and address policy applies to POST");
    assert!(matches!(refused, NetError::BadPort(_)), "{refused:?}");
}

#[tokio::test]
async fn federation_requests_need_https_unless_the_instance_allows_http() {
    let strict = strict();
    let err = strict
        .get_with("http://example.com/actor", fetch(1024), &[])
        .await
        .expect_err("signed fetch over http");
    assert!(matches!(err, NetError::PlainHttp), "{err:?}");
    let err = strict
        .post(
            "http://example.com/inbox",
            PostRequest {
                body: Vec::new(),
                content_type: "application/activity+json",
                headers: &[],
                max_response_bytes: 1024,
                timeout: Duration::from_secs(5),
            },
        )
        .await
        .expect_err("post over http");
    assert!(matches!(err, NetError::PlainHttp), "{err:?}");
    // A link preview may still fetch plain http on port 80 (the address policy refuses this
    // host later, at connect time).
    let err = strict
        .get("http://example.com/page", fetch(1024))
        .await
        .expect_err("private address");
    assert!(matches!(err, NetError::BlockedAddress), "{err:?}");

    let dev = SafeClient::with_resolver(&config(true), Arc::new(TestResolver));
    let err = dev
        .get_with("http://example.com/actor", fetch(1024), &[])
        .await
        .expect_err("private address");
    assert!(matches!(err, NetError::BlockedAddress), "{err:?}");
}
