//! Custom emotes: the staff routes, the media pipeline behind them and the `emojis` array of
//! every DTO that carries text.
#![allow(missing_docs)]

mod common;

use std::io::Cursor;

use axum::body::Body;
use axum::http::{Method, StatusCode};
use common::{App, Browser, Part, Person, RawReply};
use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
use image::codecs::png::PngEncoder;
use image::{AnimationDecoder as _, ExtendedColorType, Frame, ImageEncoder as _, Rgba, RgbaImage};
use serde_json::{Value, json};
use sqlx::PgPool;
use tuita_server::db::emoji;
use tuita_server::media::MediaStore;
use tuita_server::workers::cleanup::run_once;
use uuid::Uuid;

fn app(db: &PgPool) -> App {
    common::app_on(db.clone(), Some(MediaStore::memory()), |_| None)
}

async fn staff(app: &App, db: &PgPool, handle: &str, role: &str) -> Person {
    let person = app.person(db, handle).await;
    sqlx::query("update users set role = $2, password_hash = 'x' where id = $1")
        .bind(person.id)
        .bind(role)
        .execute(db)
        .await
        .expect("role");
    person
}

fn png(width: u32, height: u32) -> Vec<u8> {
    let image = RgbaImage::from_fn(width, height, |x, y| Rgba([x as u8, y as u8, 7, 200]));
    let mut out = Vec::new();
    PngEncoder::new(&mut out)
        .write_image(image.as_raw(), width, height, ExtendedColorType::Rgba8)
        .expect("encode png");
    out
}

fn gif(width: u32, height: u32, frames: u8) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut encoder = GifEncoder::new(&mut out);
        encoder.set_repeat(Repeat::Infinite).expect("repeat");
        for step in 0..frames {
            let frame =
                RgbaImage::from_pixel(width, height, Rgba([step * 60, 20, 200 - step * 60, 255]));
            encoder.encode_frame(Frame::new(frame)).expect("frame");
        }
    }
    out
}

async fn create(browser: &mut Browser, parts: &[Part]) -> RawReply {
    let (content_type, body) = common::multipart(parts);
    browser
        .send_raw(
            Method::POST,
            "/api/v1/admin/emojis",
            &[("content-type", &content_type)],
            Body::from(body),
        )
        .await
}

fn named(shortcode: &str, category: Option<&str>, file: Part) -> Vec<Part> {
    let mut parts = vec![Part::text("shortcode", shortcode), file];
    if let Some(category) = category {
        parts.push(Part::text("category", category));
    }
    parts
}

fn image_file(content_type: &'static str, bytes: Vec<u8>) -> Part {
    Part::file("file", content_type, bytes)
}

/// Adds the emote `shortcode` as `browser`, a staff account.
async fn add(browser: &mut Browser, shortcode: &str, file: Part) -> Value {
    create(browser, &named(shortcode, None, file)).await.ok()
}

async fn stored(app: &App, dto: &Value) -> Vec<u8> {
    let url = dto["url"].as_str().expect("url");
    let name = url.strip_prefix("/media/").expect("media url");
    app.state
        .media()
        .read(name)
        .await
        .expect("stored file")
        .to_vec()
}

/// Waits for a failed request's guard to delete its file, which happens in the background.
async fn files_settle_at(app: &App, count: usize) {
    for _ in 0..200 {
        if app.state.media().list().await.expect("list").len() == count {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert_eq!(app.state.media().list().await.expect("list").len(), count);
}

fn shortcodes(emojis: &Value) -> Vec<&str> {
    emojis
        .as_array()
        .expect("emojis array")
        .iter()
        .map(|emoji| emoji["shortcode"].as_str().expect("shortcode"))
        .collect()
}

async fn rows(db: &PgPool, sql: &str) -> i64 {
    sqlx::query_scalar(sqlx::AssertSqlSafe(sql.to_owned()))
        .fetch_one(db)
        .await
        .expect("count")
}

#[sqlx::test(migrations = "./migrations")]
async fn staff_add_list_recategorise_and_delete_emotes(db: PgPool) {
    let app = app(&db);
    let mut moderator = staff(&app, &db, "mod", "moderator").await;

    let blob = create(
        &mut moderator.browser,
        &named(
            "blobcat",
            Some("Cats"),
            image_file("image/png", png(300, 200)),
        ),
    )
    .await
    .ok();
    assert_eq!(blob["shortcode"], "blobcat");
    assert_eq!(blob["category"], "Cats");
    assert!(blob["url"].as_str().expect("url").ends_with(".webp"));
    let image = image::load_from_memory(&stored(&app, &blob).await).expect("decodes");
    assert_eq!(image.width(), 128);
    assert!((85..=86).contains(&image.height()), "{}", image.height());

    let party = add(
        &mut moderator.browser,
        "party_gif",
        image_file("image/gif", gif(256, 128, 3)),
    )
    .await;
    assert!(party["url"].as_str().expect("url").ends_with(".gif"));
    assert_eq!(party["category"], Value::Null);
    let bytes = stored(&app, &party).await;
    let decoder = GifDecoder::new(Cursor::new(&bytes)).expect("gif");
    let frames = decoder
        .into_frames()
        .collect::<Result<Vec<_>, _>>()
        .expect("frames");
    assert_eq!(frames.len(), 3);
    assert_eq!(
        (frames[0].buffer().width(), frames[0].buffer().height()),
        (128, 64)
    );

    let mut visitor = app.browser();
    let listed = visitor.get("/emojis").await.ok();
    assert_eq!(shortcodes(&listed), ["party_gif", "blobcat"]);
    assert_eq!(listed[1]["url"], blob["url"]);
    assert_eq!(moderator.browser.get("/admin/emojis").await.ok(), listed);

    create(
        &mut moderator.browser,
        &named("BlobCat", None, image_file("image/png", png(8, 8))),
    )
    .await
    .err(StatusCode::BAD_REQUEST, "That shortcode is already taken.");
    files_settle_at(&app, 2).await;
    assert_eq!(rows(&db, "select count(*) from media").await, 2);

    let shortcode_rule = "A shortcode is 2 to 32 letters, digits or underscores.";
    let long_shortcode = "x".repeat(33);
    let long_category = "c".repeat(33);
    for bad in ["a", "bad-name", "", long_shortcode.as_str()] {
        create(
            &mut moderator.browser,
            &named(bad, None, image_file("image/png", png(8, 8))),
        )
        .await
        .err(StatusCode::BAD_REQUEST, shortcode_rule);
    }
    let missing = create(&mut moderator.browser, &[Part::text("shortcode", "nofile")]).await;
    assert_eq!(missing.status, StatusCode::BAD_REQUEST);
    let not_image = create(
        &mut moderator.browser,
        &named("text", None, image_file("image/png", b"hello".to_vec())),
    )
    .await;
    assert_eq!(not_image.status, StatusCode::BAD_REQUEST);
    let huge = create(
        &mut moderator.browser,
        &named("huge", None, image_file("image/png", vec![0; 300 * 1024])),
    )
    .await;
    assert_eq!(huge.status, StatusCode::PAYLOAD_TOO_LARGE);
    create(
        &mut moderator.browser,
        &named(
            "longcat",
            Some(long_category.as_str()),
            image_file("image/png", png(8, 8)),
        ),
    )
    .await
    .err(
        StatusCode::BAD_REQUEST,
        "A category is at most 32 characters.",
    );
    files_settle_at(&app, 2).await;

    let updated = moderator
        .browser
        .patch("/admin/emojis/BLOBCAT", json!({ "category": null }))
        .await
        .ok();
    assert_eq!(updated["shortcode"], "blobcat");
    assert_eq!(updated["category"], Value::Null);
    moderator
        .browser
        .patch("/admin/emojis/ghost", json!({ "category": "x" }))
        .await
        .err(StatusCode::NOT_FOUND, "Emote not found.");

    // Cleanup keeps files an emote uses, however old.
    sqlx::query("update media set created_at = now() - interval '2 days'")
        .execute(&db)
        .await
        .expect("age media");
    run_once(&app.state).await.expect("cleanup");
    assert_eq!(rows(&db, "select count(*) from media").await, 2);

    assert_eq!(
        moderator
            .browser
            .delete("/admin/emojis/party_gif")
            .await
            .ok()["ok"],
        true
    );
    moderator
        .browser
        .delete("/admin/emojis/party_gif")
        .await
        .err(StatusCode::NOT_FOUND, "Emote not found.");
    assert_eq!(shortcodes(&visitor.get("/emojis").await.ok()), ["blobcat"]);
    assert_eq!(rows(&db, "select count(*) from media").await, 1);
    files_settle_at(&app, 1).await;

    let actions: Vec<String> = sqlx::query_scalar("select action from audit_log order by id")
        .fetch_all(&db)
        .await
        .expect("audit");
    assert_eq!(
        actions,
        [
            "emoji.create",
            "emoji.create",
            "emoji.update",
            "emoji.delete"
        ]
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn only_staff_manage_emotes(db: PgPool) {
    let app = app(&db);
    let mut moderator = staff(&app, &db, "mod", "moderator").await;
    add(
        &mut moderator.browser,
        "blobcat",
        image_file("image/png", png(8, 8)),
    )
    .await;
    let mut plain = app.person(&db, "plain").await;
    let mut signed_out = app.browser();

    let file = || named("sneaky", None, image_file("image/png", png(8, 8)));
    create(&mut plain.browser, &file())
        .await
        .err(StatusCode::FORBIDDEN, "staff only");
    plain
        .browser
        .get("/admin/emojis")
        .await
        .err(StatusCode::FORBIDDEN, "staff only");
    plain
        .browser
        .patch("/admin/emojis/blobcat", json!({ "category": "x" }))
        .await
        .err(StatusCode::FORBIDDEN, "staff only");
    plain
        .browser
        .delete("/admin/emojis/blobcat")
        .await
        .err(StatusCode::FORBIDDEN, "staff only");
    assert_eq!(
        create(&mut signed_out, &file()).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        signed_out.get("/admin/emojis").await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(rows(&db, "select count(*) from custom_emojis").await, 1);

    // The list everyone reads is public, signed in or not.
    assert_eq!(
        shortcodes(&signed_out.get("/emojis").await.ok()),
        ["blobcat"]
    );
    assert_eq!(
        shortcodes(&plain.browser.get("/emojis").await.ok()),
        ["blobcat"]
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn an_instance_holds_at_most_one_thousand_local_emotes(db: PgPool) {
    let app = app(&db);
    let mut moderator = staff(&app, &db, "mod", "moderator").await;
    let first = add(
        &mut moderator.browser,
        "first",
        image_file("image/png", png(8, 8)),
    )
    .await;
    assert_eq!(first["shortcode"], "first");
    sqlx::query(
        "insert into custom_emojis (id, shortcode, media_id)
         select gen_random_uuid(), 'filler' || n, (select media_id from custom_emojis limit 1)
           from generate_series(1, 999) n",
    )
    .execute(&db)
    .await
    .expect("fill");
    create(
        &mut moderator.browser,
        &named("onetoomany", None, image_file("image/png", png(8, 8))),
    )
    .await
    .err(
        StatusCode::BAD_REQUEST,
        "This instance already has 1000 emotes.",
    );
}

/// Two emotes, one still and one animated, made through the API.
async fn two_emotes(app: &App, db: &PgPool) {
    let mut moderator = staff(app, db, "mod", "moderator").await;
    add(
        &mut moderator.browser,
        "blobcat",
        image_file("image/png", png(20, 20)),
    )
    .await;
    add(
        &mut moderator.browser,
        "party_gif",
        image_file("image/gif", gif(16, 16, 2)),
    )
    .await;
}

#[sqlx::test(migrations = "./migrations")]
async fn posts_users_and_messages_carry_the_emotes_they_mention(db: PgPool) {
    let app = app(&db);
    two_emotes(&app, &db).await;
    let mut ana = app.person(&db, "ana").await;
    let mut bob = app.person(&db, "bob").await;
    ana.browser
        .patch(
            "/me",
            json!({
                "displayName": "Ana :blobcat:",
                "bio": "party :party_gif: time",
                "location": ":nope:",
            }),
        )
        .await
        .ok();

    let post = ana
        .browser
        .post_text("hi :blobcat: and :PARTY_GIF: :nope: at 12:30:45 :blobcat:")
        .await;
    assert_eq!(shortcodes(&post["emojis"]), ["blobcat", "party_gif"]);
    assert_eq!(
        post["emojis"][0]["url"]
            .as_str()
            .map(|url| url.starts_with("/media/")),
        Some(true)
    );
    assert_eq!(shortcodes(&post["author"]["emojis"]), ["blobcat"]);

    let plain = ana.browser.post_text("nothing :nope: here").await;
    assert_eq!(plain["emojis"], json!([]));

    let warned = ana
        .browser
        .post(
            "/posts",
            json!({ "text": "spoiler", "cw": ":party_gif: hmm" }),
        )
        .await
        .ok();
    assert_eq!(shortcodes(&warned["emojis"]), ["party_gif"]);

    let poll = ana
        .browser
        .post(
            "/posts",
            json!({
                "text": "vote",
                "poll": { "options": [":blobcat: yes", "no"], "multiple": false, "durationHours": 1 },
            }),
        )
        .await
        .ok();
    assert_eq!(shortcodes(&poll["emojis"]), ["blobcat"]);

    let id = post["id"].as_str().expect("id");
    let thread = bob.browser.get(&format!("/posts/{id}")).await.ok();
    assert_eq!(
        shortcodes(&thread["post"]["emojis"]),
        ["blobcat", "party_gif"]
    );

    let quote = bob
        .browser
        .post("/posts", json!({ "text": "agreed", "quoteOf": id }))
        .await
        .ok();
    assert_eq!(quote["emojis"], json!([]));
    assert_eq!(
        shortcodes(&quote["quote"]["emojis"]),
        ["blobcat", "party_gif"]
    );
    assert_eq!(shortcodes(&quote["quote"]["author"]["emojis"]), ["blobcat"]);

    let profile = bob.browser.get("/users/ana").await.ok();
    assert_eq!(shortcodes(&profile["emojis"]), ["blobcat", "party_gif"]);
    let me = ana.browser.get("/me").await.ok();
    assert_eq!(
        shortcodes(&me["user"]["user"]["emojis"]),
        ["blobcat", "party_gif"]
    );

    let sent = ana
        .browser
        .post(
            "/conversations/bob",
            json!({ "text": "yo :blobcat: :ghost:" }),
        )
        .await
        .ok();
    assert_eq!(shortcodes(&sent["emojis"]), ["blobcat"]);
    let view = bob.browser.get("/conversations/ana").await.ok();
    assert_eq!(shortcodes(&view["messages"][0]["emojis"]), ["blobcat"]);
    assert_eq!(
        shortcodes(&view["user"]["emojis"]),
        ["blobcat", "party_gif"]
    );
    let list = bob.browser.get("/conversations").await.ok();
    assert_eq!(shortcodes(&list["items"][0]["last"]["emojis"]), ["blobcat"]);
    assert_eq!(shortcodes(&list["items"][0]["user"]["emojis"]), ["blobcat"]);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_deleted_emote_is_no_longer_attached(db: PgPool) {
    let app = app(&db);
    two_emotes(&app, &db).await;
    let mut ana = app.person(&db, "ana").await;
    let post = ana.browser.post_text("hi :blobcat:").await;
    assert_eq!(shortcodes(&post["emojis"]), ["blobcat"]);

    let mut moderator = app.browser();
    let id: Uuid = sqlx::query_scalar("select id from users where handle = 'mod'")
        .fetch_one(&db)
        .await
        .expect("mod");
    moderator.sign_in_as(&db, id).await;
    moderator.delete("/admin/emojis/blobcat").await.ok();

    let id = post["id"].as_str().expect("id");
    let again = ana.browser.get(&format!("/posts/{id}")).await.ok();
    assert_eq!(again["post"]["text"], "hi :blobcat:");
    assert_eq!(again["post"]["emojis"], json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_feed_of_many_posts_resolves_every_emote(db: PgPool) {
    let app = app(&db);
    two_emotes(&app, &db).await;
    let mut ana = app.person(&db, "ana").await;
    for n in 0..20 {
        ana.browser
            .post_text(&format!("post {n} :blobcat: :party_gif:"))
            .await;
    }
    let page = ana.browser.get("/users/ana/posts?limit=20").await.ok();
    let items = page["items"].as_array().expect("items");
    assert_eq!(items.len(), 20);
    for item in items {
        assert_eq!(shortcodes(&item["emojis"]), ["blobcat", "party_gif"]);
    }
}

async fn media_row(db: &PgPool, owner: Uuid, path: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "insert into media (id, owner_id, kind, mime, path, bytes)
         values ($1, $2, 'image', 'image/webp', $3, 1)",
    )
    .bind(id)
    .bind(owner)
    .bind(path)
    .execute(db)
    .await
    .expect("media");
    id
}

#[sqlx::test(migrations = "./migrations")]
async fn emotes_belong_to_one_origin_and_resolve_in_one_call(db: PgPool) {
    let owner = common::insert_user(&db, "mod").await;
    let remote = media_row(&db, owner, "remote.webp").await;
    let local = media_row(&db, owner, "local.webp").await;

    emoji::upsert_remote(
        &db,
        "Remote.Example",
        "Wave",
        remote,
        "https://remote.example/wave.png",
    )
    .await
    .expect("upsert");
    let mut conn = db.acquire().await.expect("conn");
    let made = emoji::create_local(&mut conn, owner, "wave", None, local, "local.webp")
        .await
        .expect("local twin of a remote shortcode");
    assert_eq!(made.url, "/media/local.webp");
    drop(conn);

    let from_remote = emoji::resolve(&db, Some("remote.example"), &["hi :wave:"])
        .await
        .expect("resolve");
    assert_eq!(from_remote.len(), 1);
    assert_eq!(
        (
            from_remote[0].shortcode.as_str(),
            from_remote[0].url.as_str()
        ),
        ("Wave", "/media/remote.webp")
    );
    let from_local = emoji::resolve(&db, None, &["hi :WAVE:"])
        .await
        .expect("resolve");
    assert_eq!(from_local.len(), 1);
    assert_eq!(from_local[0].url, "/media/local.webp");
    assert!(
        emoji::resolve(&db, Some("other.example"), &[":wave:"])
            .await
            .expect("resolve")
            .is_empty()
    );
    assert!(
        emoji::resolve(&db, None, &["no tokens", "12:30:45"])
            .await
            .expect("resolve")
            .is_empty()
    );

    let replaced = media_row(&db, owner, "remote2.webp").await;
    emoji::upsert_remote(
        &db,
        "remote.example",
        "wave",
        replaced,
        "https://remote.example/wave2.png",
    )
    .await
    .expect("replace");
    assert_eq!(rows(&db, "select count(*) from custom_emojis").await, 2);
    let again = emoji::resolve(&db, Some("remote.example"), &[":wave:"])
        .await
        .expect("resolve");
    assert_eq!(again[0].url, "/media/remote2.webp");

    // The local list never shows a remote copy.
    let listed = emoji::list_local(&db).await.expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].url, "/media/local.webp");

    let many = emoji::resolve(&db, None, &[":wave: and :nope:", "also :WAVE:"])
        .await
        .expect("resolve");
    assert_eq!(many.len(), 1);
}
