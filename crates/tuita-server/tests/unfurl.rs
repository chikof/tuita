//! The unfurl worker: pages become link cards, failures are remembered, events reach the author.
#![allow(missing_docs)]

mod common;

use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::routing::get;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tuita_server::dto::{CardEventDto, StreamEvent};
use tuita_server::net::SafeClient;
use tuita_server::workers::unfurl::{Job, process_job, run};
use uuid::Uuid;

#[derive(Clone, Default)]
struct Hits {
    page: Arc<AtomicUsize>,
    image: Arc<AtomicUsize>,
}

fn png() -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    image::RgbImage::new(8, 6)
        .write_to(&mut out, image::ImageFormat::Png)
        .expect("encode png");
    out.into_inner()
}

const PAGE: &str = r#"<!doctype html><html><head>
<title>Ignored</title>
<meta property="og:title" content="Fish &amp; Chips">
<meta property="og:description" content="Best   in town">
<meta property="og:image" content="/img.png">
<meta property="og:image:width" content="1200">
</head><body>hi</body></html>"#;

async fn page(
    State(hits): State<Hits>,
) -> ([(axum::http::HeaderName, &'static str); 1], &'static str) {
    hits.page.fetch_add(1, Ordering::SeqCst);
    ([(CONTENT_TYPE, "text/html; charset=utf-8")], PAGE)
}

async fn image(State(hits): State<Hits>) -> ([(axum::http::HeaderName, &'static str); 1], Vec<u8>) {
    hits.image.fetch_add(1, Ordering::SeqCst);
    ([(CONTENT_TYPE, "image/png")], png())
}

async fn broken_image_page(
    State(hits): State<Hits>,
) -> ([(axum::http::HeaderName, &'static str); 1], &'static str) {
    hits.page.fetch_add(1, Ordering::SeqCst);
    (
        [(CONTENT_TYPE, "text/html")],
        r#"<meta property="og:title" content="No picture"><meta property="og:image" content="/nope.png">
           <meta property="og:site_name" content="Site Name">"#,
    )
}

async fn untitled(
    State(hits): State<Hits>,
) -> ([(axum::http::HeaderName, &'static str); 1], &'static str) {
    hits.page.fetch_add(1, Ordering::SeqCst);
    ([(CONTENT_TYPE, "text/html")], "<p>no title here</p>")
}

async fn missing(State(hits): State<Hits>) -> StatusCode {
    hits.page.fetch_add(1, Ordering::SeqCst);
    StatusCode::NOT_FOUND
}

async fn serve() -> (SocketAddr, Hits) {
    let hits = Hits::default();
    let app = Router::new()
        .route("/page", get(page))
        .route("/img.png", get(image))
        .route("/broken", get(broken_image_page))
        .route("/untitled", get(untitled))
        .route("/missing", get(missing))
        .route(
            "/json",
            get(|| async { ([(CONTENT_TYPE, "application/json")], r#"{"title":"x"}"#) }),
        )
        .with_state(hits.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    (addr, hits)
}

struct Fixture {
    app: common::App,
    client: SafeClient,
    author: Uuid,
    post: Uuid,
    data: std::path::PathBuf,
}

impl Fixture {
    async fn new(db: &PgPool) -> Self {
        let data = std::env::temp_dir().join(format!("tuita-unfurl-{}", Uuid::now_v7()));
        let dir = data.display().to_string();
        let app = common::app_with(db.clone(), |var| (var == "DATA_DIR").then(|| dir.clone()));
        let client = SafeClient::new(app.state.config()).allow_loopback_for_tests();
        let author = common::insert_user(db, "ana").await;
        let post = Uuid::now_v7();
        sqlx::query("insert into posts (id, author_id, text) values ($1, $2, 'link')")
            .bind(post)
            .bind(author)
            .execute(db)
            .await
            .expect("insert post");
        Self {
            app,
            client,
            author,
            post,
            data,
        }
    }

    fn job(&self, url: String) -> Job {
        Job {
            post_id: self.post,
            author_id: self.author,
            url,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.data);
    }
}

#[derive(sqlx::FromRow)]
struct CardRow {
    title: String,
    description: String,
    site: String,
    large: bool,
    ok: bool,
    image_id: Option<Uuid>,
}

async fn card_row(db: &PgPool, url: &str) -> Option<CardRow> {
    sqlx::query_as(
        "select title, description, site, large, ok, image_id from link_cards where url = $1",
    )
    .bind(url)
    .fetch_optional(db)
    .await
    .expect("select link card")
}

#[sqlx::test(migrations = "./migrations")]
async fn builds_a_card_stores_its_image_and_tells_the_author(db: PgPool) {
    let fx = Fixture::new(&db).await;
    let (addr, hits) = serve().await;
    let url = format!("http://127.0.0.1:{}/page", addr.port());
    let mut events = fx.app.state.hub().subscribe(fx.author);

    let card = process_job(&fx.app.state, &fx.client, &fx.job(url.clone()))
        .await
        .expect("processed")
        .expect("a card");

    let row = card_row(&db, &url).await.expect("link_cards row");
    assert!(row.ok && row.large);
    assert_eq!(row.title, "Fish & Chips");
    assert_eq!(row.description, "Best in town");
    assert_eq!(row.site, "127.0.0.1");
    let image_id = row.image_id.expect("image stored");
    let (owner, path, post_id): (Uuid, String, Option<Uuid>) =
        sqlx::query_as("select owner_id, path, post_id from media where id = $1")
            .bind(image_id)
            .fetch_one(&db)
            .await
            .expect("media row");
    assert_eq!(owner, fx.author);
    assert_eq!(post_id, None);
    assert!(
        fx.data.join("media").join(&path).is_file(),
        "image file on disk"
    );

    assert_eq!(card.url, url);
    assert_eq!(card.title, "Fish & Chips");
    assert_eq!(card.site, "127.0.0.1");
    assert_eq!(card.image, Some(format!("/media/{path}")));
    assert!(card.large);
    let event = events.try_recv().expect("card event");
    assert_eq!(
        *event,
        StreamEvent::Card(Box::new(CardEventDto {
            post_id: fx.post,
            card: card.clone(),
        }))
    );

    let again = process_job(&fx.app.state, &fx.client, &fx.job(url))
        .await
        .expect("processed again");
    assert_eq!(again, Some(card));
    assert_eq!(hits.page.load(Ordering::SeqCst), 1, "fresh row: no refetch");
    assert_eq!(hits.image.load(Ordering::SeqCst), 1);
    assert!(events.try_recv().is_ok(), "event published again");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_dead_image_only_costs_the_card_its_picture(db: PgPool) {
    let fx = Fixture::new(&db).await;
    let (addr, _) = serve().await;
    let url = format!("http://127.0.0.1:{}/broken", addr.port());
    let card = process_job(&fx.app.state, &fx.client, &fx.job(url.clone()))
        .await
        .expect("processed")
        .expect("a card");
    assert_eq!(card.title, "No picture");
    assert_eq!(card.site, "Site Name");
    assert_eq!(card.image, None);
    assert!(!card.large);
    let row = card_row(&db, &url).await.expect("row");
    assert!(row.ok);
    assert_eq!(row.image_id, None);
}

#[sqlx::test(migrations = "./migrations")]
async fn failures_are_stored_and_retried_after_six_hours(db: PgPool) {
    let fx = Fixture::new(&db).await;
    let (addr, hits) = serve().await;
    let url = format!("http://127.0.0.1:{}/missing", addr.port());
    let mut events = fx.app.state.hub().subscribe(fx.author);

    let first = process_job(&fx.app.state, &fx.client, &fx.job(url.clone()))
        .await
        .expect("processed");
    assert_eq!(first, None);
    let row = card_row(&db, &url).await.expect("failure row");
    assert!(!row.ok);
    assert_eq!(
        (
            row.title.as_str(),
            row.description.as_str(),
            row.site.as_str()
        ),
        ("", "", "")
    );
    assert!(!row.large);
    assert_eq!(row.image_id, None);
    assert!(events.try_recv().is_err(), "no event for a failure");

    process_job(&fx.app.state, &fx.client, &fx.job(url.clone()))
        .await
        .expect("processed");
    assert_eq!(hits.page.load(Ordering::SeqCst), 1, "failure is cached");

    sqlx::query("update link_cards set fetched_at = now() - interval '7 hours' where url = $1")
        .bind(&url)
        .execute(&db)
        .await
        .expect("age row");
    process_job(&fx.app.state, &fx.client, &fx.job(url))
        .await
        .expect("processed");
    assert_eq!(hits.page.load(Ordering::SeqCst), 2, "retried after 6 h");
}

#[sqlx::test(migrations = "./migrations")]
async fn refreshes_a_success_after_seven_days(db: PgPool) {
    let fx = Fixture::new(&db).await;
    let (addr, hits) = serve().await;
    let url = format!("http://127.0.0.1:{}/page", addr.port());
    process_job(&fx.app.state, &fx.client, &fx.job(url.clone()))
        .await
        .expect("processed");
    sqlx::query("update link_cards set fetched_at = now() - interval '6 days' where url = $1")
        .bind(&url)
        .execute(&db)
        .await
        .expect("age row");
    process_job(&fx.app.state, &fx.client, &fx.job(url.clone()))
        .await
        .expect("processed");
    assert_eq!(hits.page.load(Ordering::SeqCst), 1);
    sqlx::query("update link_cards set fetched_at = now() - interval '8 days' where url = $1")
        .bind(&url)
        .execute(&db)
        .await
        .expect("age row");
    process_job(&fx.app.state, &fx.client, &fx.job(url.clone()))
        .await
        .expect("processed");
    assert_eq!(hits.page.load(Ordering::SeqCst), 2);
    assert!(card_row(&db, &url).await.expect("row").ok);
}

#[sqlx::test(migrations = "./migrations")]
async fn pages_without_a_title_or_html_make_no_card(db: PgPool) {
    let fx = Fixture::new(&db).await;
    let (addr, _) = serve().await;
    for path in ["untitled", "json"] {
        let url = format!("http://127.0.0.1:{}/{path}", addr.port());
        let card = process_job(&fx.app.state, &fx.client, &fx.job(url.clone()))
            .await
            .expect("processed");
        assert_eq!(card, None, "{path}");
        assert!(!card_row(&db, &url).await.expect("row").ok, "{path}");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn blocked_urls_are_stored_as_failures(db: PgPool) {
    let fx = Fixture::new(&db).await;
    let strict = SafeClient::new(fx.app.state.config());
    for url in [
        "http://169.254.169.254/latest/meta-data",
        "http://127.0.0.1/",
    ] {
        let card = process_job(&fx.app.state, &strict, &fx.job(url.to_owned()))
            .await
            .expect("processed");
        assert_eq!(card, None, "{url}");
        assert!(!card_row(&db, url).await.expect("row").ok, "{url}");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn the_worker_loop_processes_jobs_and_stops_on_cancel(db: PgPool) {
    let fx = Fixture::new(&db).await;
    let (tx, rx) = mpsc::channel(8);
    let stop = CancellationToken::new();
    let worker = tokio::spawn(run(fx.app.state.clone(), rx, stop.clone()));
    let url = "http://169.254.169.254/".to_owned();
    tx.send(fx.job(url.clone())).await.expect("queue job");
    let stored = async {
        loop {
            if card_row(&db, &url).await.is_some() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    };
    tokio::time::timeout(Duration::from_secs(5), stored)
        .await
        .expect("job processed");
    stop.cancel();
    tokio::time::timeout(Duration::from_secs(5), worker)
        .await
        .expect("worker stops on cancel")
        .expect("worker did not panic");
}

#[sqlx::test(migrations = "./migrations")]
async fn the_worker_loop_ends_when_the_queue_closes(db: PgPool) {
    let fx = Fixture::new(&db).await;
    let (tx, rx) = mpsc::channel(1);
    let worker = tokio::spawn(run(fx.app.state.clone(), rx, CancellationToken::new()));
    drop(tx);
    tokio::time::timeout(Duration::from_secs(5), worker)
        .await
        .expect("worker stops when the queue closes")
        .expect("worker did not panic");
}
