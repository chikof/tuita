//! Media uploads: metadata stripping, size limits, re-encoding, ranges, ownership and cleanup.
#![allow(missing_docs)]

mod common;

use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::Router;
use axum::body::{Body, Bytes};
use axum::http::header::CONTENT_TYPE;
use axum::http::{Method, StatusCode};
use axum::routing::get as serve_get;
use common::{App, Part, RawReply};
use futures_util::StreamExt as _;
use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{
    AnimationDecoder as _, ExtendedColorType, Frame, ImageEncoder as _, ImageFormat, Rgb, RgbImage,
    Rgba, RgbaImage,
};
use serde_json::{Value, json};
use sqlx::PgPool;
use tuita_core::ap::Document;
use tuita_server::federation::media::{copy_attachments, copy_avatar};
use tuita_server::media::MediaStore;
use tuita_server::workers::cleanup::run_once;
use uuid::Uuid;

const EXIF: &[u8] = b"Exif\0\0";

/// Where a test app keeps its media files.
#[derive(Clone, Copy, Debug)]
enum Backend {
    /// Files under a temp `DATA_DIR`.
    Local,
    /// The in-memory store.
    Memory,
}

/// Runs `$body(db, backend)` as one test per backend.
macro_rules! on_each_backend {
    ($body:ident => $local:ident, $memory:ident) => {
        #[sqlx::test(migrations = "./migrations")]
        async fn $local(db: PgPool) {
            $body(db, Backend::Local).await;
        }

        #[sqlx::test(migrations = "./migrations")]
        async fn $memory(db: PgPool) {
            $body(db, Backend::Memory).await;
        }
    };
}

/// An app over its own media store; a local one keeps its `DATA_DIR`, removed on drop.
struct Env {
    app: App,
    data: PathBuf,
}

impl Env {
    fn new(db: PgPool) -> Self {
        Self::on(Backend::Local, db)
    }

    fn on(backend: Backend, db: PgPool) -> Self {
        Self::build(backend, db, |_| None)
    }

    fn with(db: PgPool, extra: impl Fn(&str) -> Option<String>) -> Self {
        Self::build(Backend::Local, db, extra)
    }

    fn build(backend: Backend, db: PgPool, extra: impl Fn(&str) -> Option<String>) -> Self {
        let data = std::env::temp_dir().join(format!("tuita-media-{}", Uuid::now_v7()));
        let dir = data.display().to_string();
        let media = match backend {
            Backend::Local => None,
            Backend::Memory => Some(MediaStore::memory()),
        };
        let app = common::app_on(db, media, |var| {
            if var == "DATA_DIR" {
                Some(dir.clone())
            } else {
                extra(var)
            }
        });
        Self { app, data }
    }

    fn store(&self) -> &MediaStore {
        self.app.state.media()
    }

    /// Object keys in the store, sorted.
    async fn files(&self) -> Vec<String> {
        self.store().list().await.expect("list the store")
    }

    /// Waits for a request guard's delete, which runs in the background, then asserts the store
    /// is empty.
    async fn no_files(&self) {
        for _ in 0..200 {
            if self.files().await.is_empty() {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert_eq!(self.files().await, Vec::<String>::new());
    }

    /// The bytes behind the `url` of an upload's DTO.
    async fn stored(&self, dto: &Value) -> Vec<u8> {
        let url = dto["url"].as_str().expect("url");
        let name = url.strip_prefix("/media/").expect("media url");
        self.store().read(name).await.expect("stored file").to_vec()
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.data);
    }
}

fn gradient(width: u32, height: u32) -> RgbImage {
    RgbImage::from_fn(width, height, |x, y| {
        Rgb([
            (x * 255 / width.max(1)) as u8,
            (y * 255 / height.max(1)) as u8,
            90,
        ])
    })
}

fn jpeg(width: u32, height: u32) -> Vec<u8> {
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, 90)
        .encode(
            gradient(width, height).as_raw(),
            width,
            height,
            ExtendedColorType::Rgb8,
        )
        .expect("encode jpeg");
    out
}

fn png(width: u32, height: u32) -> Vec<u8> {
    let image = RgbaImage::from_fn(width, height, |x, y| Rgba([x as u8, y as u8, 7, 128]));
    let mut out = Vec::new();
    PngEncoder::new(&mut out)
        .write_image(image.as_raw(), width, height, ExtendedColorType::Rgba8)
        .expect("encode png");
    out
}

fn gif(frames: u8) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut encoder = GifEncoder::new(&mut out);
        encoder.set_repeat(Repeat::Infinite).expect("repeat");
        for step in 0..frames {
            let frame = RgbaImage::from_pixel(32, 24, Rgba([step * 60, 20, 200 - step * 60, 255]));
            encoder.encode_frame(Frame::new(frame)).expect("frame");
        }
    }
    out
}

/// A big-endian Exif block: orientation, and optionally a GPS IFD with a latitude and longitude.
fn exif_segment(orientation: u16, gps: bool) -> Vec<u8> {
    let mut tiff = Vec::new();
    tiff.extend_from_slice(b"MM\x00\x2a\x00\x00\x00\x08");
    let entries: u16 = if gps { 2 } else { 1 };
    tiff.extend_from_slice(&entries.to_be_bytes());
    tiff.extend_from_slice(&[0x01, 0x12, 0x00, 0x03, 0, 0, 0, 1]);
    tiff.extend_from_slice(&orientation.to_be_bytes());
    tiff.extend_from_slice(&[0, 0]);
    if gps {
        tiff.extend_from_slice(&[0x88, 0x25, 0x00, 0x04, 0, 0, 0, 1, 0, 0, 0, 38]);
    }
    tiff.extend_from_slice(&[0, 0, 0, 0]);
    if gps {
        assert_eq!(tiff.len(), 38);
        tiff.extend_from_slice(&[0x00, 0x04]);
        tiff.extend_from_slice(&[0, 1, 0, 2, 0, 0, 0, 2, b'N', 0, 0, 0]);
        tiff.extend_from_slice(&[0, 2, 0, 5, 0, 0, 0, 3, 0, 0, 0, 92]);
        tiff.extend_from_slice(&[0, 3, 0, 2, 0, 0, 0, 2, b'E', 0, 0, 0]);
        tiff.extend_from_slice(&[0, 4, 0, 5, 0, 0, 0, 3, 0, 0, 0, 116]);
        tiff.extend_from_slice(&[0, 0, 0, 0]);
        for degrees in [52_u32, 4] {
            for (num, den) in [(degrees, 1_u32), (30, 1), (0, 1)] {
                tiff.extend_from_slice(&num.to_be_bytes());
                tiff.extend_from_slice(&den.to_be_bytes());
            }
        }
        for (num, den) in [(13_u32, 1_u32), (24, 1), (0, 1)] {
            tiff.extend_from_slice(&num.to_be_bytes());
            tiff.extend_from_slice(&den.to_be_bytes());
        }
    }
    let mut segment = vec![0xff, 0xe1];
    let length = u16::try_from(2 + EXIF.len() + tiff.len()).expect("segment fits");
    segment.extend_from_slice(&length.to_be_bytes());
    segment.extend_from_slice(EXIF);
    segment.extend_from_slice(&tiff);
    segment
}

/// `jpeg` with `segment` right after the SOI marker.
fn with_segment(jpeg: &[u8], segment: &[u8]) -> Vec<u8> {
    let mut out = jpeg[..2].to_vec();
    out.extend_from_slice(segment);
    out.extend_from_slice(&jpeg[2..]);
    out
}

/// The committed fixture; written once when missing.
fn gps_fixture() -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gps.jpg");
    if !path.exists() {
        let bytes = with_segment(&jpeg(64, 48), &exif_segment(1, true));
        std::fs::write(&path, bytes).expect("write fixture");
    }
    std::fs::read(path).expect("read fixture")
}

fn mp4(len: usize) -> Vec<u8> {
    let mut out = b"\0\0\0\x18ftypmp42\0\0\0\0mp42isom".to_vec();
    out.resize(len, 0xab);
    out
}

fn dims(bytes: &[u8]) -> (u32, u32) {
    let image = image::load_from_memory(bytes).expect("decodes");
    (image.width(), image.height())
}

fn image_part(content_type: &'static str, bytes: Vec<u8>) -> Part {
    Part::file("file", content_type, bytes)
}

async fn count(db: &PgPool, table: &str) -> i64 {
    sqlx::query_scalar(sqlx::AssertSqlSafe(format!("select count(*) from {table}")))
        .fetch_one(db)
        .await
        .expect("count")
}

async fn get(browser: &mut common::Browser, url: &str, headers: &[(&str, &str)]) -> RawReply {
    browser
        .send_raw(Method::GET, url, headers, Body::empty())
        .await
}

async fn head(browser: &mut common::Browser, url: &str, headers: &[(&str, &str)]) -> RawReply {
    browser
        .send_raw(Method::HEAD, url, headers, Body::empty())
        .await
}

async fn exif_gps_is_stripped_from_a_jpeg(db: PgPool, backend: Backend) {
    let env = Env::on(backend, db.clone());
    let mut ana = env.app.person(&db, "ana").await;
    let fixture = gps_fixture();
    assert!(fixture.windows(EXIF.len()).any(|window| window == EXIF));

    let reply = ana
        .browser
        .upload(&[image_part("image/jpeg", fixture)])
        .await;
    let dto = reply.ok();
    assert_eq!(dto["kind"], "image");
    assert_eq!(
        (dto["width"].as_i64(), dto["height"].as_i64()),
        (Some(64), Some(48))
    );
    assert!(dto["url"].as_str().expect("url").ends_with(".jpg"));

    let stored = env.stored(&dto).await;
    assert!(!stored.windows(EXIF.len()).any(|window| window == EXIF));
    assert_eq!(
        image::guess_format(&stored).expect("format"),
        ImageFormat::Jpeg
    );
    assert_eq!(dims(&stored), (64, 48));
    let files = env.files().await;
    assert_eq!(files.len(), 1, "no temp file left: {files:?}");
}

on_each_backend!(exif_gps_is_stripped_from_a_jpeg => exif_on_local, exif_on_memory);

#[sqlx::test(migrations = "./migrations")]
async fn exif_orientation_is_applied_before_it_is_dropped(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;
    let rotated = with_segment(&jpeg(64, 32), &exif_segment(6, false));

    let dto = ana
        .browser
        .upload(&[image_part("image/jpeg", rotated)])
        .await
        .ok();
    assert_eq!(
        (dto["width"].as_i64(), dto["height"].as_i64()),
        (Some(32), Some(64))
    );
    assert_eq!(dims(&env.stored(&dto).await), (32, 64));
}

#[sqlx::test(migrations = "./migrations")]
async fn png_becomes_lossless_webp_and_keeps_alpha(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;

    let dto = ana
        .browser
        .upload(&[image_part("image/png", png(40, 30))])
        .await
        .ok();
    assert!(dto["url"].as_str().expect("url").ends_with(".webp"));
    let stored = env.stored(&dto).await;
    assert_eq!(
        image::guess_format(&stored).expect("format"),
        ImageFormat::WebP
    );
    let decoded = image::load_from_memory(&stored)
        .expect("decodes")
        .to_rgba8();
    assert_eq!(decoded.get_pixel(3, 4), &Rgba([3, 4, 7, 128]));

    let mime: String = sqlx::query_scalar("select mime from media where id = $1")
        .bind(
            dto["id"]
                .as_str()
                .expect("id")
                .parse::<Uuid>()
                .expect("uuid"),
        )
        .fetch_one(&db)
        .await
        .expect("row");
    assert_eq!(mime, "image/webp");
}

#[sqlx::test(migrations = "./migrations")]
async fn gif_stays_gif_with_all_its_frames(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;

    let dto = ana
        .browser
        .upload(&[image_part("image/gif", gif(3))])
        .await
        .ok();
    assert!(dto["url"].as_str().expect("url").ends_with(".gif"));
    let stored = env.stored(&dto).await;
    let frames = GifDecoder::new(Cursor::new(stored))
        .expect("gif")
        .into_frames()
        .count();
    assert_eq!(frames, 3);
}

#[sqlx::test(migrations = "./migrations")]
async fn big_images_are_scaled_down_and_small_ones_are_not_scaled_up(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;

    let big = ana
        .browser
        .upload(&[image_part("image/jpeg", jpeg(3000, 1000))])
        .await
        .ok();
    assert_eq!(
        (big["width"].as_i64(), big["height"].as_i64()),
        (Some(2048), Some(683))
    );
    assert_eq!(dims(&env.stored(&big).await), (2048, 683));

    let small = ana
        .browser
        .upload(&[image_part("image/jpeg", jpeg(100, 50))])
        .await
        .ok();
    assert_eq!(
        (small["width"].as_i64(), small["height"].as_i64()),
        (Some(100), Some(50))
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn avatars_and_banners_are_cropped_to_their_size_and_set_on_the_account(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;

    let crop = json!({ "x": 10, "y": 0, "w": 80, "h": 80 }).to_string();
    let avatar = ana
        .browser
        .upload(&[
            Part::text("purpose", "avatar"),
            Part::text("crop", &crop),
            image_part("image/jpeg", jpeg(200, 100)),
        ])
        .await
        .ok();
    assert_eq!(
        (avatar["width"].as_i64(), avatar["height"].as_i64()),
        (Some(400), Some(400))
    );
    assert_eq!(dims(&env.stored(&avatar).await), (400, 400));
    let id: Uuid = avatar["id"].as_str().expect("id").parse().expect("uuid");
    let set: Option<Uuid> = sqlx::query_scalar("select avatar_id from users where id = $1")
        .bind(ana.id)
        .fetch_one(&db)
        .await
        .expect("avatar");
    assert_eq!(set, Some(id));
    let me = ana.browser.get("/me").await.ok();
    assert_eq!(
        me["user"]["user"]["avatarUrl"],
        json!(format!("/media/{id}"))
    );

    let banner = ana
        .browser
        .upload(&[
            Part::text("purpose", "banner"),
            Part::text(
                "crop",
                &json!({ "x": 0, "y": 10, "w": 300, "h": 100 }).to_string(),
            ),
            image_part("image/png", png(300, 200)),
        ])
        .await
        .ok();
    assert_eq!(
        (banner["width"].as_i64(), banner["height"].as_i64()),
        (Some(1500), Some(500))
    );
    assert_eq!(dims(&env.stored(&banner).await), (1500, 500));

    let gif_avatar = ana
        .browser
        .upload(&[
            Part::text("purpose", "avatar"),
            Part::text(
                "crop",
                &json!({ "x": 0, "y": 0, "w": 24, "h": 24 }).to_string(),
            ),
            image_part("image/gif", gif(3)),
        ])
        .await
        .ok();
    assert!(gif_avatar["url"].as_str().expect("url").ends_with(".webp"));
    assert_eq!(dims(&env.stored(&gif_avatar).await), (400, 400));
}

#[sqlx::test(migrations = "./migrations")]
async fn the_crop_is_taken_after_the_rotation(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;

    // 200x100 turned 90 degrees is 100x200: only then does y 100 + h 100 fit.
    let crop = json!({ "x": 0, "y": 100, "w": 100, "h": 100, "rotate": 90 }).to_string();
    let dto = ana
        .browser
        .upload(&[
            Part::text("purpose", "avatar"),
            Part::text("crop", &crop),
            image_part("image/jpeg", jpeg(200, 100)),
        ])
        .await
        .ok();
    assert_eq!(dims(&env.stored(&dto).await), (400, 400));

    let banner_crop = json!({ "x": 0, "y": 0, "w": 100, "h": 33, "rotate": 270 }).to_string();
    let banner = ana
        .browser
        .upload(&[
            Part::text("purpose", "banner"),
            Part::text("crop", &banner_crop),
            image_part("image/jpeg", jpeg(200, 100)),
        ])
        .await
        .ok();
    assert_eq!(dims(&env.stored(&banner).await), (1500, 500));

    for crop in [
        json!({ "x": 0, "y": 100, "w": 100, "h": 100 }),
        json!({ "x": 0, "y": 0, "w": 0, "h": 10 }),
        json!({ "x": 0, "y": 0, "w": 10, "h": 10, "rotate": 45 }),
        json!({ "x": 4294967295_u32, "y": 0, "w": 10, "h": 10 }),
    ] {
        let reply = ana
            .browser
            .upload(&[
                Part::text("purpose", "avatar"),
                Part::text("crop", &crop.to_string()),
                image_part("image/jpeg", jpeg(200, 100)),
            ])
            .await;
        reply.err(StatusCode::BAD_REQUEST, "Pick the area to crop.");
    }
    let reply = ana
        .browser
        .upload(&[
            Part::text("purpose", "banner"),
            image_part("image/jpeg", jpeg(200, 100)),
        ])
        .await;
    reply.err(StatusCode::BAD_REQUEST, "Pick the area to crop.");
}

#[sqlx::test(migrations = "./migrations")]
async fn avatar_and_banner_can_be_removed(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;
    let crop = json!({ "x": 0, "y": 0, "w": 50, "h": 50 }).to_string();
    for purpose in ["avatar", "banner"] {
        ana.browser
            .upload(&[
                Part::text("purpose", purpose),
                Part::text("crop", &crop),
                image_part("image/jpeg", jpeg(100, 100)),
            ])
            .await
            .ok();
    }
    let me = ana.browser.get("/me").await.ok();
    assert!(me["user"]["user"]["avatarUrl"].is_string());
    assert!(me["user"]["user"]["bannerUrl"].is_string());

    let me = ana.browser.delete("/me/avatar").await.ok();
    assert!(me["user"]["avatarUrl"].is_null() && me["user"]["bannerUrl"].is_string());
    let me = ana.browser.delete("/me/banner").await.ok();
    assert!(me["user"]["bannerUrl"].is_null());
    assert_eq!(me["user"]["handle"], "ana");

    let mut anon = env.app.browser();
    anon.delete("/me/avatar")
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");
}

#[sqlx::test(migrations = "./migrations")]
async fn alt_is_trimmed_and_limited(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;

    let dto = ana
        .browser
        .upload(&[
            Part::text("alt", "  a cat  "),
            image_part("image/jpeg", jpeg(10, 10)),
        ])
        .await
        .ok();
    assert_eq!(dto["alt"], "a cat");

    let long = "x".repeat(1001);
    let reply = ana
        .browser
        .upload(&[
            Part::text("alt", &long),
            image_part("image/jpeg", jpeg(10, 10)),
        ])
        .await;
    reply.err(
        StatusCode::BAD_REQUEST,
        "Alt text must be at most 1000 characters.",
    );
    assert_eq!(count(&db, "media").await, 1);
    assert_eq!(env.files().await.len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_12_mib_image_is_refused_and_leaves_nothing(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;

    let reply = ana
        .browser
        .upload(&[image_part("image/jpeg", vec![0_u8; 12 * 1024 * 1024])])
        .await;
    reply.err(
        StatusCode::PAYLOAD_TOO_LARGE,
        "Image is too large (max 10 MB).",
    );
    assert_eq!(count(&db, "media").await, 0);
    env.no_files().await;
}

#[sqlx::test(migrations = "./migrations")]
async fn the_image_limit_is_enforced_while_the_body_streams(db: PgPool) {
    const CHUNK: usize = 64 * 1024;
    const CHUNKS: usize = 640;
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;

    let boundary = "stream";
    let head = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"a.jpg\"\r\nContent-Type: image/jpeg\r\n\r\n"
    );
    let polled = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&polled);
    // Yielding makes the stream pending between chunks, like a socket; an always-ready stream
    // would be drained by the parser in one go.
    let chunks = futures_util::stream::unfold(0_usize, move |sent| {
        let counter = Arc::clone(&counter);
        async move {
            tokio::task::yield_now().await;
            if sent == CHUNKS {
                return None;
            }
            counter.fetch_add(CHUNK, Ordering::SeqCst);
            let chunk = Bytes::from(vec![0_u8; CHUNK]);
            Some((Ok::<_, std::convert::Infallible>(chunk), sent + 1))
        }
    });
    let body = futures_util::stream::once(async move { Ok(Bytes::from(head)) }).chain(chunks);
    let content_type = format!("multipart/form-data; boundary={boundary}");
    let reply = ana
        .browser
        .send_raw(
            Method::POST,
            "/api/v1/media",
            &[("content-type", &content_type)],
            Body::from_stream(body),
        )
        .await;

    reply.err(
        StatusCode::PAYLOAD_TOO_LARGE,
        "Image is too large (max 10 MB).",
    );
    let read = polled.load(Ordering::SeqCst);
    assert!(
        read < 11 * 1024 * 1024 + 2 * CHUNK,
        "read {read} of {} bytes",
        CHUNK * CHUNKS
    );
    assert_eq!(count(&db, "media").await, 0);
    env.no_files().await;
}

#[sqlx::test(migrations = "./migrations")]
async fn a_huge_declared_length_is_refused_before_the_file_is_read(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;

    let (content_type, body) = common::multipart(&[image_part("image/jpeg", jpeg(10, 10))]);
    let reply = ana
        .browser
        .send_raw(
            Method::POST,
            "/api/v1/media",
            &[
                ("content-type", &content_type),
                ("content-length", "104857600"),
            ],
            Body::from(body),
        )
        .await;
    reply.err(
        StatusCode::PAYLOAD_TOO_LARGE,
        "Image is too large (max 10 MB).",
    );
    assert_eq!(count(&db, "media").await, 0);
    env.no_files().await;
}

#[sqlx::test(migrations = "./migrations")]
async fn bad_content_is_refused_whatever_the_header_says(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;
    let message =
        "That file isn’t an image or video tuita can use (PNG, JPEG, GIF, WebP, MP4, WebM or MOV).";

    for (content_type, bytes) in [
        ("image/png", b"definitely not a png".to_vec()),
        ("image/jpeg", Vec::new()),
        ("video/mp4", b"definitely not a video".to_vec()),
        ("video/mp4", Vec::new()),
        ("text/plain", b"hello".to_vec()),
        ("application/octet-stream", jpeg(10, 10)),
    ] {
        let reply = ana.browser.upload(&[image_part(content_type, bytes)]).await;
        reply.err(StatusCode::BAD_REQUEST, message);
    }
    let reply = ana.browser.upload(&[Part::text("alt", "no file")]).await;
    reply.err(StatusCode::BAD_REQUEST, message);

    let reply = ana
        .browser
        .upload(&[image_part("video/x-matroska", vec![1, 2, 3])])
        .await;
    reply.err(
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "Videos must be MP4, WebM or MOV.",
    );

    // An image dressed up as a video, and a video dressed up as an image.
    let reply = ana
        .browser
        .upload(&[image_part("video/mp4", jpeg(10, 10))])
        .await;
    reply.err(StatusCode::BAD_REQUEST, message);
    let reply = ana
        .browser
        .upload(&[image_part("image/png", mp4(200))])
        .await;
    reply.err(StatusCode::BAD_REQUEST, message);

    assert_eq!(count(&db, "media").await, 0);
    env.no_files().await;
}

/// A PNG header that claims 40000 x 40000 pixels; the decoder must refuse before allocating.
fn png_bomb() -> Vec<u8> {
    fn crc(bytes: &[u8]) -> u32 {
        let mut crc = 0xffff_ffff_u32;
        for byte in bytes {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = if crc & 1 == 1 {
                    0xedb8_8320 ^ (crc >> 1)
                } else {
                    crc >> 1
                };
            }
        }
        !crc
    }
    fn chunk(out: &mut Vec<u8>, kind: [u8; 4], data: &[u8]) {
        out.extend_from_slice(&u32::try_from(data.len()).expect("small").to_be_bytes());
        let mut body = kind.to_vec();
        body.extend_from_slice(data);
        out.extend_from_slice(&body);
        out.extend_from_slice(&crc(&body).to_be_bytes());
    }
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::new();
    header.extend_from_slice(&40_000_u32.to_be_bytes());
    header.extend_from_slice(&40_000_u32.to_be_bytes());
    header.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(&mut out, *b"IHDR", &header);
    chunk(&mut out, *b"IEND", &[]);
    out
}

#[sqlx::test(migrations = "./migrations")]
async fn a_decompression_bomb_is_refused(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;

    let reply = ana
        .browser
        .upload(&[image_part("image/png", png_bomb())])
        .await;
    assert_eq!(
        reply.status,
        StatusCode::BAD_REQUEST,
        "{}",
        String::from_utf8_lossy(&reply.bytes)
    );
    env.no_files().await;
}

#[sqlx::test(migrations = "./migrations")]
async fn sessions_and_origin_are_required(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;
    let part = || image_part("image/jpeg", jpeg(10, 10));

    let mut anon = env.app.browser();
    anon.upload(&[part()])
        .await
        .err(StatusCode::UNAUTHORIZED, "sign in required");

    ana.browser.origin = Some("https://evil.example".to_owned());
    ana.browser
        .upload(&[part()])
        .await
        .err(StatusCode::FORBIDDEN, "bad origin");
    assert_eq!(count(&db, "media").await, 0);
}

async fn videos_are_stored_and_served_with_ranges(db: PgPool, backend: Backend) {
    let env = Env::on(backend, db.clone());
    let mut ana = env.app.person(&db, "ana").await;

    let video = mp4(1000);
    let dto = ana
        .browser
        .upload(&[
            Part::text("alt", "a clip"),
            Part::file("poster", "image/png", png(20, 10)),
            image_part("video/mp4", video.clone()),
        ])
        .await
        .ok();
    assert_eq!(dto["kind"], "video");
    assert!(dto["width"].is_null() && dto["height"].is_null());
    assert_eq!(dto["alt"], "a clip");
    let url = dto["url"].as_str().expect("url").to_owned();
    assert!(url.ends_with(".mp4"));
    let poster_url = dto["posterUrl"].as_str().expect("poster url").to_owned();
    assert!(poster_url.ends_with(".webp"));
    assert_eq!(count(&db, "media").await, 2);
    let files = env.files().await;
    assert_eq!(files.len(), 2, "{files:?}");

    let anon = &mut env.app.browser();
    let partial = get(anon, &url, &[("range", "bytes=0-99")]).await;
    assert_eq!(partial.status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(partial.bytes, video[..100]);
    assert_eq!(partial.header("content-range"), "bytes 0-99/1000");
    assert_eq!(partial.header("content-type"), "video/mp4");

    let full = get(anon, &url, &[]).await;
    assert_eq!(full.status, StatusCode::OK);
    assert_eq!(full.bytes, video);
    assert_eq!(
        full.header("cache-control"),
        "public, max-age=31536000, immutable"
    );
    assert_eq!(full.header("content-disposition"), "inline");
    assert_eq!(full.header("accept-ranges"), "bytes");
    assert_eq!(full.header("content-type"), "video/mp4");

    let bare = url.trim_end_matches(".mp4");
    assert_eq!(get(anon, bare, &[]).await.bytes, video);
    let image = get(anon, &poster_url, &[]).await;
    assert_eq!(image.header("content-type"), "image/webp");
}

on_each_backend!(videos_are_stored_and_served_with_ranges => videos_on_local, videos_on_memory);

#[sqlx::test(migrations = "./migrations")]
async fn webm_and_mov_are_recognised_by_their_bytes(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;

    let mut webm = vec![0x1a, 0x45, 0xdf, 0xa3];
    webm.resize(64, 0);
    let dto = ana
        .browser
        .upload(&[image_part("video/webm", webm)])
        .await
        .ok();
    assert!(dto["url"].as_str().expect("url").ends_with(".webm"));

    let mut mov = b"\0\0\0\x14ftypqt  ".to_vec();
    mov.resize(64, 0);
    // The declared type is not trusted: an MP4 label on a MOV file still stores a MOV.
    let dto = ana
        .browser
        .upload(&[image_part("video/mp4", mov)])
        .await
        .ok();
    assert!(dto["url"].as_str().expect("url").ends_with(".mov"));
    let served = get(
        &mut env.app.browser(),
        dto["url"].as_str().expect("url"),
        &[],
    )
    .await;
    assert_eq!(served.header("content-type"), "video/quicktime");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_video_over_the_configured_cap_is_refused_and_its_temp_file_removed(db: PgPool) {
    let env = Env::with(db.clone(), |var| {
        (var == "MAX_VIDEO_MB").then(|| "1".to_owned())
    });
    let mut ana = env.app.person(&db, "ana").await;

    let reply = ana
        .browser
        .upload(&[image_part("video/mp4", mp4(2 * 1024 * 1024))])
        .await;
    reply.err(StatusCode::PAYLOAD_TOO_LARGE, "Videos can be up to 1 MB.");
    env.no_files().await;
    assert_eq!(count(&db, "media").await, 0);

    ana.browser
        .upload(&[image_part("video/mp4", mp4(1024 * 1024))])
        .await
        .ok();
}

#[sqlx::test(migrations = "./migrations")]
async fn videos_cannot_be_avatars_and_have_their_own_rate_limit(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;

    let reply = ana
        .browser
        .upload(&[
            Part::text("purpose", "avatar"),
            Part::text(
                "crop",
                &json!({ "x": 0, "y": 0, "w": 1, "h": 1 }).to_string(),
            ),
            image_part("video/mp4", mp4(100)),
        ])
        .await;
    reply.err(
        StatusCode::BAD_REQUEST,
        "Videos can’t be avatars or banners.",
    );
    env.no_files().await;

    // The refused upload above used one of the 20 hourly video uploads.
    for _ in 0..19 {
        ana.browser
            .upload(&[image_part("video/mp4", mp4(100))])
            .await
            .ok();
    }
    let reply = ana
        .browser
        .upload(&[image_part("video/mp4", mp4(100))])
        .await;
    assert_eq!(reply.status, StatusCode::TOO_MANY_REQUESTS);
    assert!(!reply.header("retry-after").is_empty());
    // Images have their own budget.
    ana.browser
        .upload(&[image_part("image/jpeg", jpeg(10, 10))])
        .await
        .ok();
}

async fn files_are_served_only_by_their_own_name(db: PgPool, backend: Backend) {
    let env = Env::on(backend, db.clone());
    let mut ana = env.app.person(&db, "ana").await;
    let dto = ana
        .browser
        .upload(&[image_part("image/jpeg", jpeg(30, 20))])
        .await
        .ok();
    let url = dto["url"].as_str().expect("url").to_owned();
    let id = dto["id"].as_str().expect("id").to_owned();

    let mut anon = env.app.browser();
    let reply = get(&mut anon, &url, &[]).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.header("content-type"), "image/jpeg");
    assert_eq!(
        get(&mut anon, &format!("/media/{id}"), &[]).await.status,
        StatusCode::OK
    );

    for wrong in [
        format!("/media/{id}.png"),
        format!("/media/{}.jpg", Uuid::now_v7()),
        "/media/not-a-uuid.jpg".to_owned(),
        "/media/..%2F..%2Fetc%2Fpasswd".to_owned(),
        format!("/media/{id}.jpg.part"),
    ] {
        let reply = get(&mut anon, &wrong, &[]).await;
        reply.err(StatusCode::NOT_FOUND, "not found");
    }

    env.store()
        .delete(&format!("{id}.jpg"))
        .await
        .expect("remove object");
    get(&mut anon, &url, &[])
        .await
        .err(StatusCode::NOT_FOUND, "not found");
}

on_each_backend!(files_are_served_only_by_their_own_name => names_on_local, names_on_memory);

#[sqlx::test(migrations = "./migrations")]
async fn alt_text_can_be_patched_by_the_owner_until_a_post_holds_the_upload(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;
    let mut ben = env.app.person(&db, "ben").await;
    let dto = ana
        .browser
        .upload(&[image_part("image/jpeg", jpeg(30, 20))])
        .await
        .ok();
    let id = dto["id"].as_str().expect("id").to_owned();

    let patched = ana
        .browser
        .patch(&format!("/media/{id}"), json!({ "alt": " sunset " }))
        .await
        .ok();
    assert_eq!(patched["alt"], "sunset");
    assert_eq!(patched["id"], json!(id));

    let reply = ana
        .browser
        .patch(&format!("/media/{id}"), json!({ "alt": "x".repeat(1001) }))
        .await;
    reply.err(
        StatusCode::BAD_REQUEST,
        "Alt text must be at most 1000 characters.",
    );

    ben.browser
        .patch(&format!("/media/{id}"), json!({ "alt": "mine now" }))
        .await
        .err(StatusCode::NOT_FOUND, "Upload not found.");
    ana.browser
        .patch(&format!("/media/{}", Uuid::now_v7()), json!({ "alt": "x" }))
        .await
        .err(StatusCode::NOT_FOUND, "Upload not found.");
    ana.browser
        .patch("/media/nope", json!({ "alt": "x" }))
        .await
        .err(StatusCode::NOT_FOUND, "Upload not found.");

    attach(&db, ana.id, id.parse().expect("uuid")).await;
    ana.browser
        .patch(&format!("/media/{id}"), json!({ "alt": "too late" }))
        .await
        .err(StatusCode::NOT_FOUND, "Upload not found.");
    let alt: String = sqlx::query_scalar("select alt from media where id = $1::uuid")
        .bind(&id)
        .fetch_one(&db)
        .await
        .expect("alt");
    assert_eq!(alt, "sunset");
}

/// Puts `media` on a new post of `author`, the way the post route does.
async fn attach(db: &PgPool, author: Uuid, media: Uuid) -> Uuid {
    let post = Uuid::now_v7();
    sqlx::query("insert into posts (id, author_id, text) values ($1, $2, 'p')")
        .bind(post)
        .bind(author)
        .execute(db)
        .await
        .expect("post");
    sqlx::query("insert into post_media (post_id, media_id, position) values ($1, $2, 0)")
        .bind(post)
        .bind(media)
        .execute(db)
        .await
        .expect("post_media");
    sqlx::query("update media set post_id = $1 where id = $2")
        .bind(post)
        .bind(media)
        .execute(db)
        .await
        .expect("attach");
    post
}

async fn an_old_orphan_is_cleaned_up_with_its_file_and_an_attached_upload_survives(
    db: PgPool,
    backend: Backend,
) {
    let env = Env::on(backend, db.clone());
    let mut ana = env.app.person(&db, "ana").await;
    let orphan = ana
        .browser
        .upload(&[image_part("image/jpeg", jpeg(30, 20))])
        .await
        .ok();
    let kept = ana
        .browser
        .upload(&[image_part("image/png", png(30, 20))])
        .await
        .ok();
    let fresh = ana
        .browser
        .upload(&[image_part("image/gif", gif(2))])
        .await
        .ok();
    assert_eq!(env.files().await.len(), 3);

    let id = |dto: &Value| {
        dto["id"]
            .as_str()
            .expect("id")
            .parse::<Uuid>()
            .expect("uuid")
    };
    attach(&db, ana.id, id(&kept)).await;
    sqlx::query("update media set created_at = now() - interval '25 hours' where id = any($1)")
        .bind([id(&orphan), id(&kept)].as_slice())
        .execute(&db)
        .await
        .expect("age");

    let stats = run_once(&env.app.state).await.expect("cleanup");
    assert_eq!((stats.media_rows, stats.media_files), (1, 1));
    let left: Vec<Uuid> = sqlx::query_scalar("select id from media order by id")
        .fetch_all(&db)
        .await
        .expect("media");
    let mut expected = vec![id(&kept), id(&fresh)];
    expected.sort();
    assert_eq!(left, expected);
    let files = env.files().await;
    assert_eq!(files.len(), 2);
    assert!(
        files
            .iter()
            .all(|name| !name.starts_with(&id(&orphan).to_string()))
    );
}

on_each_backend!(an_old_orphan_is_cleaned_up_with_its_file_and_an_attached_upload_survives => orphans_on_local, orphans_on_memory);

async fn card_images_are_re_encoded_and_owned_by_the_post_author(db: PgPool, backend: Backend) {
    let env = Env::on(backend, db.clone());
    let ana = env.app.person(&db, "ana").await;

    let exif = with_segment(&jpeg(64, 48), &exif_segment(1, true));
    let id = tuita_server::media::store_card_image(&env.app.state, ana.id, exif)
        .await
        .expect("stored");
    let (owner, path, kind, bytes): (Uuid, String, String, i64) =
        sqlx::query_as("select owner_id, path, kind, bytes from media where id = $1")
            .bind(id)
            .fetch_one(&db)
            .await
            .expect("row");
    assert_eq!((owner, kind.as_str()), (ana.id, "image"));
    let stored = env.store().read(&path).await.expect("file");
    assert_eq!(i64::try_from(stored.len()).expect("fits"), bytes);
    assert!(!stored.windows(EXIF.len()).any(|window| window == EXIF));

    let err = tuita_server::media::store_card_image(&env.app.state, ana.id, b"<html>".to_vec())
        .await
        .expect_err("not an image");
    assert!(
        matches!(err, tuita_server::AppError::BadRequest(_)),
        "{err:?}"
    );
    assert_eq!(env.files().await.len(), 1);
}

on_each_backend!(card_images_are_re_encoded_and_owned_by_the_post_author => card_images_on_local, card_images_on_memory);

#[sqlx::test(migrations = "./migrations")]
async fn a_tiny_png_that_decodes_to_hundreds_of_megapixels_is_refused(db: PgPool) {
    // 16384 x 16384 of one grey: a few hundred KB on the wire, 256 MiB once decoded.
    let side = 16384_u32;
    let pixels = vec![9_u8; side as usize * side as usize];
    let mut bomb = Vec::new();
    PngEncoder::new_with_quality(
        &mut bomb,
        image::codecs::png::CompressionType::Default,
        image::codecs::png::FilterType::Sub,
    )
    .write_image(&pixels, side, side, ExtendedColorType::L8)
    .expect("encode png");
    drop(pixels);
    assert!(bomb.len() < 10 * 1024 * 1024, "{} bytes", bomb.len());

    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;
    let reply = ana.browser.upload(&[image_part("image/png", bomb)]).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(count(&db, "media").await, 0);
    env.no_files().await;
}

async fn head_ranges_and_validators_come_from_the_store(db: PgPool, backend: Backend) {
    let env = Env::on(backend, db.clone());
    let mut ana = env.app.person(&db, "ana").await;
    let video = mp4(1000);
    let dto = ana
        .browser
        .upload(&[image_part("video/mp4", video.clone())])
        .await
        .ok();
    let url = dto["url"].as_str().expect("url").to_owned();
    let anon = &mut env.app.browser();
    let whole = head(anon, &url, &[]).await;
    assert_eq!(whole.status, StatusCode::OK);
    assert!(whole.bytes.is_empty());
    assert_eq!(whole.header("content-length"), "1000");
    assert_eq!(whole.header("content-type"), "video/mp4");
    assert_eq!(whole.header("accept-ranges"), "bytes");
    assert_eq!(
        whole.header("cache-control"),
        "public, max-age=31536000, immutable"
    );
    assert_eq!(whole.header("content-disposition"), "inline");
    assert!(!whole.header("last-modified").is_empty());
    let etag = whole.header("etag").to_owned();
    assert!(etag.starts_with('"') && etag.ends_with('"'), "{etag}");

    let part = head(anon, &url, &[("range", "bytes=10-19")]).await;
    assert_eq!(part.status, StatusCode::PARTIAL_CONTENT);
    assert!(part.bytes.is_empty());
    assert_eq!(part.header("content-length"), "10");
    assert_eq!(part.header("content-range"), "bytes 10-19/1000");

    let tail = get(anon, &url, &[("range", "bytes=-100")]).await;
    assert_eq!(tail.status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(tail.bytes, video[900..]);
    assert_eq!(tail.header("content-range"), "bytes 900-999/1000");
    let open = get(anon, &url, &[("range", "bytes=500-")]).await;
    assert_eq!(open.bytes, video[500..]);
    assert_eq!(open.header("content-range"), "bytes 500-999/1000");

    let past = get(anon, &url, &[("range", "bytes=1000-")]).await;
    assert_eq!(past.status, StatusCode::RANGE_NOT_SATISFIABLE);
    assert_eq!(past.header("content-range"), "bytes */1000");
    assert!(past.bytes.is_empty());
    let many = get(anon, &url, &[("range", "bytes=0-1,5-6")]).await;
    assert_eq!((many.status, many.bytes), (StatusCode::OK, video.clone()));

    let same = get(anon, &url, &[("if-none-match", &etag)]).await;
    assert_eq!(same.status, StatusCode::NOT_MODIFIED);
    assert!(same.bytes.is_empty());
    assert_eq!(
        same.header("cache-control"),
        "public, max-age=31536000, immutable"
    );
    let modified = whole.header("last-modified").to_owned();
    let fresh = get(anon, &url, &[("if-modified-since", &modified)]).await;
    assert_eq!(fresh.status, StatusCode::NOT_MODIFIED);
    let other = get(anon, &url, &[("if-none-match", "\"nope\"")]).await;
    assert_eq!((other.status, other.bytes), (StatusCode::OK, video));
}

on_each_backend!(head_ranges_and_validators_come_from_the_store => validators_on_local, validators_on_memory);

/// A server with one PNG and one MP4, for remote copies.
async fn remote_files() -> String {
    let app = Router::new()
        .route(
            "/a.png",
            serve_get(|| async { ([(CONTENT_TYPE, "image/png")], png(40, 30)) }),
        )
        .route(
            "/v.mp4",
            serve_get(|| async { ([(CONTENT_TYPE, "video/mp4")], mp4(2000)) }),
        )
        .route(
            "/geo.mp4",
            serve_get(|| async { ([(CONTENT_TYPE, "video/mp4")], tagged_mp4()) }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let base = format!("http://{}", listener.local_addr().expect("addr"));
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    base
}

async fn remote_media_is_copied_into_the_store(db: PgPool, backend: Backend) {
    let base = remote_files().await;
    let env = Env::build(backend, db.clone(), |var| {
        matches!(
            var,
            "FEDERATION_ALLOW_HTTP" | "FEDERATION_ALLOW_PRIVATE_NETS"
        )
        .then(|| "1".to_owned())
    });
    let owner = common::insert_user(&db, "remote").await;
    let state = &env.app.state;

    let avatar = copy_avatar(state, owner, &format!("{base}/a.png"))
        .await
        .expect("avatar copied");
    let docs: Vec<Document> = serde_json::from_value(json!([
        {"type": "Document", "mediaType": "video/mp4", "url": format!("{base}/v.mp4"), "name": "clip"}
    ]))
    .expect("documents");
    let attached = copy_attachments(state, owner, &docs).await;
    assert_eq!(attached.len(), 1);

    let rows: Vec<(Uuid, String, String, String)> =
        sqlx::query_as("select id, kind, mime, path from media order by kind")
            .fetch_all(&db)
            .await
            .expect("rows");
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert_eq!((rows[0].0, rows[0].1.as_str()), (avatar, "image"));
    assert_eq!(rows[0].2, "image/webp", "re-encoded, not stored as sent");
    assert_eq!((rows[1].0, rows[1].1.as_str()), (attached[0], "video"));
    let mut keys: Vec<String> = rows.iter().map(|row| row.3.clone()).collect();
    keys.sort();
    assert_eq!(env.files().await, keys);

    let anon = &mut env.app.browser();
    let video = get(
        anon,
        &format!("/media/{}.mp4", attached[0]),
        &[("range", "bytes=0-9")],
    )
    .await;
    assert_eq!(video.status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(video.bytes, mp4(2000)[..10]);
    let image = get(anon, &format!("/media/{avatar}.webp"), &[]).await;
    let (width, height) = dims(&image.bytes);
    assert_eq!(width, height, "the avatar is cropped square");
}

on_each_backend!(remote_media_is_copied_into_the_store => remote_on_local, remote_on_memory);

async fn a_video_whose_poster_is_refused_leaves_no_object(db: PgPool, backend: Backend) {
    let env = Env::on(backend, db.clone());
    let mut ana = env.app.person(&db, "ana").await;
    let reply = ana
        .browser
        .upload(&[
            image_part("video/mp4", mp4(1000)),
            Part::file("poster", "image/png", b"not a picture".to_vec()),
        ])
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(count(&db, "media").await, 0);
    env.no_files().await;
}

on_each_backend!(a_video_whose_poster_is_refused_leaves_no_object => poster_on_local, poster_on_memory);

const GPS_TEXT: &[u8] = b"+52.5200+013.4050/";
/// What a video must not keep once stored.
const VIDEO_LEAKS: [&[u8]; 4] = [
    GPS_TEXT,
    b"com.apple.quicktime.location.ISO6709",
    b"iPhone 15 Pro",
    b"GPSLatitude",
];
const FRAMES: &[u8] = b"frames frames";

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn mp4_box(kind: &[u8], payload: &[u8]) -> Vec<u8> {
    let size = u32::try_from(8 + payload.len()).expect("small box");
    [&size.to_be_bytes()[..], kind, payload].concat()
}

/// An MP4 with a GPS fix and device name in `moov`, and an XMP packet in a last `uuid` box.
fn tagged_mp4() -> Vec<u8> {
    let gps = mp4_box(b"\xa9xyz", &[&[0, 18, 0x55, 0xc4][..], GPS_TEXT].concat());
    let udta = mp4_box(b"udta", &gps);
    let meta = mp4_box(
        b"meta",
        &[
            &[0_u8; 4][..],
            &mp4_box(b"keys", VIDEO_LEAKS[1]),
            &mp4_box(b"ilst", VIDEO_LEAKS[2]),
        ]
        .concat(),
    );
    let xmp = mp4_box(
        b"uuid",
        &[&[1_u8; 16][..], b"<x:xmpmeta GPSLatitude=\"52.52\"/>"].concat(),
    );
    [
        mp4(24),
        mp4_box(b"moov", &[udta, meta].concat()),
        mp4_box(b"mdat", FRAMES),
        xmp,
    ]
    .concat()
}

fn assert_scrubbed(stored: &[u8], sent: &[u8]) {
    for leak in VIDEO_LEAKS {
        assert!(contains(sent, leak), "the fixture holds {leak:?}");
        assert!(!contains(stored, leak), "{leak:?} was stored");
    }
    assert_eq!(stored.len(), sent.len());
    assert!(contains(stored, FRAMES), "the media data is kept");
}

async fn media_bytes(db: &PgPool, id: Uuid) -> i64 {
    sqlx::query_scalar("select bytes from media where id = $1")
        .bind(id)
        .fetch_one(db)
        .await
        .expect("row")
}

async fn an_uploaded_video_is_stored_without_its_metadata(db: PgPool, backend: Backend) {
    let env = Env::on(backend, db.clone());
    let mut ana = env.app.person(&db, "ana").await;
    let video = tagged_mp4();
    let dto = ana
        .browser
        .upload(&[image_part("video/mp4", video.clone())])
        .await
        .ok();

    let stored = env.stored(&dto).await;
    assert_scrubbed(&stored, &video);
    let id: Uuid = dto["id"].as_str().expect("id").parse().expect("uuid");
    assert_eq!(media_bytes(&db, id).await, video.len() as i64);
    let url = dto["url"].as_str().expect("url");
    let served = get(&mut env.app.browser(), url, &[]).await;
    assert_eq!(served.bytes, stored);
    assert_eq!(env.files().await.len(), 1);
}

on_each_backend!(an_uploaded_video_is_stored_without_its_metadata => scrubbed_on_local, scrubbed_on_memory);

#[sqlx::test(migrations = "./migrations")]
async fn a_webm_upload_loses_its_tags_and_titles(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;
    let element = |id: &[u8], payload: &[u8]| {
        [
            id,
            &[0x80 | u8::try_from(payload.len()).expect("short")],
            payload,
        ]
        .concat()
    };
    let info = element(
        &[0x15, 0x49, 0xa9, 0x66],
        &element(&[0x7b, 0xa9], b"Holiday GPS"),
    );
    let tags = element(&[0x12, 0x54, 0xc3, 0x67], b"GPS 52.52N");
    let video = [
        element(&[0x1a, 0x45, 0xdf, 0xa3], b""),
        vec![
            0x18, 0x53, 0x80, 0x67, 0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ],
        info,
        element(&[0x1f, 0x43, 0xb6, 0x75], FRAMES),
        tags,
    ]
    .concat();
    let dto = ana
        .browser
        .upload(&[image_part("video/webm", video.clone())])
        .await
        .ok();
    assert!(dto["url"].as_str().expect("url").ends_with(".webm"));

    let stored = env.stored(&dto).await;
    assert_eq!(stored.len(), video.len());
    assert!(contains(&video, b"Holiday GPS") && contains(&video, b"GPS 52.52N"));
    assert!(!contains(&stored, b"Holiday GPS") && !contains(&stored, b"GPS 52.52N"));
    assert!(contains(&stored, FRAMES));
    let id: Uuid = dto["id"].as_str().expect("id").parse().expect("uuid");
    assert_eq!(media_bytes(&db, id).await, video.len() as i64);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_video_the_scrub_cannot_parse_is_still_stored_as_sent(db: PgPool) {
    let env = Env::new(db.clone());
    let mut ana = env.app.person(&db, "ana").await;
    // A box that claims more bytes than the file holds ends the walk before the `udta`.
    let video = [
        mp4(24),
        vec![0x7f, 0xff, 0xff, 0xff, b'm', b'o', b'o', b'v'],
        mp4_box(b"udta", GPS_TEXT),
    ]
    .concat();
    let dto = ana
        .browser
        .upload(&[image_part("video/mp4", video.clone())])
        .await
        .ok();
    assert_eq!(env.stored(&dto).await, video);
    let id: Uuid = dto["id"].as_str().expect("id").parse().expect("uuid");
    assert_eq!(media_bytes(&db, id).await, video.len() as i64);
}

async fn a_remote_video_is_stored_without_its_metadata(db: PgPool, backend: Backend) {
    let base = remote_files().await;
    let env = Env::build(backend, db.clone(), |var| {
        matches!(
            var,
            "FEDERATION_ALLOW_HTTP" | "FEDERATION_ALLOW_PRIVATE_NETS"
        )
        .then(|| "1".to_owned())
    });
    let owner = common::insert_user(&db, "remote").await;
    let docs: Vec<Document> = serde_json::from_value(json!([
        {"type": "Document", "mediaType": "video/mp4", "url": format!("{base}/geo.mp4"), "name": "clip"}
    ]))
    .expect("documents");
    let attached = copy_attachments(&env.app.state, owner, &docs).await;
    assert_eq!(attached.len(), 1);

    let sent = tagged_mp4();
    let stored = env
        .store()
        .read(&format!("{}.mp4", attached[0]))
        .await
        .expect("stored file")
        .to_vec();
    assert_scrubbed(&stored, &sent);
    assert_eq!(media_bytes(&db, attached[0]).await, sent.len() as i64);
}

on_each_backend!(a_remote_video_is_stored_without_its_metadata => remote_scrubbed_on_local, remote_scrubbed_on_memory);
