//! Shared steps of the suite: sign-up and seeding, the peer instance and overflow checks.

use std::{os::unix::process::CommandExt as _, path::PathBuf, process::Stdio, time::Duration};

use anyhow::{Context, Result, bail};
use image::{ImageBuffer, Rgb, codecs::jpeg::JpegEncoder};
use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    process::{Child, Command},
};
use uuid::Uuid;

use crate::{Ctx, Loc, Name, Page, env};

/// Password every test account gets.
pub const PASSWORD: &str = "correct horse battery";
/// Passphrase of the encrypted-messages tests.
pub const PASSPHRASE: &str = "correct horse battery staple";

/// Turns encrypted messages on through the settings page.
pub async fn turn_on(page: &Page, passphrase: &str) -> Result<()> {
    page.goto("/settings/encryption").await?;
    page.label(Name::Exact("Passphrase"))
        .fill(passphrase)
        .await?;
    page.label("Confirm passphrase").fill(passphrase).await?;
    page.role("button", Name::Exact("Turn on")).click().await?;
    page.text("On and unlocked on this device.")
        .expect_visible()
        .await
}

/// The open conversation's bubbles; the conversation list shows previews of the same text.
#[must_use]
pub fn thread(page: &Page) -> Loc {
    page.role("list", Name::Re("^Messages with"))
}

/// Types `text` into the open conversation and sends it.
pub async fn send_dm(page: &Page, text: &str) -> Result<()> {
    page.label(Name::Exact("Message")).fill(text).await?;
    page.role("button", Name::Exact("Send")).click().await
}

/// Sends `text` and waits for it to show in the thread.
pub async fn say(page: &Page, text: &str) -> Result<()> {
    send_dm(page, text).await?;
    thread(page).text(text).expect_visible().await
}

/// A handle no other run uses, at most 15 characters. Hex after a fixed letter: random base36
/// letters sometimes spell a word the filter refuses, and a digits-only suffix makes `mod_` reserved.
#[must_use]
pub fn handle(prefix: &str) -> String {
    let hex = Uuid::now_v7().simple().to_string();
    let mut who = format!("{prefix}_k{}", &hex[hex.len() - 5..]);
    who.truncate(15);
    who
}

/// Signs up through the signup form and lands on the signed-in home.
pub async fn sign_up_via_ui(page: &Page, name: &str, who: &str) -> Result<()> {
    page.goto("/signup").await?;
    page.label("Display name").fill(name).await?;
    page.label("Handle").fill(who).await?;
    page.label("Password").fill(PASSWORD).await?;
    page.role("checkbox", Name::Re("I.m at least"))
        .check()
        .await?;
    submit_signup(page, &page.role("button", "Create account")).await
}

/// Clicks the sign-up button and waits for the app to open.
pub async fn submit_signup(page: &Page, button: &Loc) -> Result<()> {
    button.click().await?;
    page.role("navigation", Name::Exact("Main"))
        .first()
        .expect_visible()
        .await
}

/// A JSON request to `/api/v1{path}` from a page of `ctx` (cookies and `Origin` like a real
/// client). Fails unless the status is 2xx; returns the JSON body.
pub async fn call(ctx: &Ctx, method: &str, path: &str, data: Option<&Value>) -> Result<Value> {
    let (status, body) = ctx.call(method, path, data).await?;
    if !(200..300).contains(&status) {
        bail!("{method} {path}: {status} {body}");
    }
    Ok(body)
}

/// Signs `who` up through the API; the context keeps the session cookie.
pub async fn sign_up_via_api(ctx: &Ctx, who: &str, name: &str) -> Result<Value> {
    let body = json!({"handle": who, "displayName": name, "password": PASSWORD, "acceptTerms": true, "add": false});
    call(ctx, "POST", "/auth/signup", Some(&body)).await
}

/// A JPEG of exactly `width` x `height` with a gradient, so thumbnails are not flat colour.
pub fn make_image(width: u32, height: u32, hue: u32) -> Result<Vec<u8>> {
    let (from, to) = (
        hsl(f64::from(hue % 360), 0.80, 0.65),
        hsl(f64::from((hue + 90) % 360), 0.70, 0.40),
    );
    let (cx, cy, r) = (
        f64::from(width) * 0.7,
        f64::from(height) * 0.3,
        f64::from(height) * 0.15,
    );
    let image = ImageBuffer::from_fn(width, height, |x, y| {
        let (px, py) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
        // The SVG gradient runs corner to corner in object bounding box space.
        let t = ((px / f64::from(width)) + (py / f64::from(height))) / 2.0;
        let mut rgb = [0.0; 3];
        for (i, c) in rgb.iter_mut().enumerate() {
            *c = from[i] + (to[i] - from[i]) * t;
        }
        if (px - cx).hypot(py - cy) <= r {
            for c in &mut rgb {
                *c = *c * 0.2 + 255.0 * 0.8;
            }
        }
        Rgb([
            rgb[0].round() as u8,
            rgb[1].round() as u8,
            rgb[2].round() as u8,
        ])
    });
    let mut out = Vec::new();
    image.write_with_encoder(JpegEncoder::new_with_quality(&mut out, 90))?;
    Ok(out)
}

/// `hsl()` to RGB in 0..255.
fn hsl(h: f64, s: f64, l: f64) -> [f64; 3] {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match h as u32 / 60 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [(r + m) * 255.0, (g + m) * 255.0, (b + m) * 255.0]
}

/// The part of a `MediaDto` the suite reads.
#[derive(Debug, Clone)]
pub struct Uploaded {
    /// Media id for `mediaIds`.
    pub id: String,
    /// Stored width.
    pub width: u64,
    /// Stored height.
    pub height: u64,
}

/// `POST /api/v1/media` as multipart from a page of `ctx`; returns the media id and size.
pub async fn upload_image(ctx: &Ctx, bytes: &[u8], alt: &str) -> Result<Uploaded> {
    let script = format!(
        "(async (bytes, alt) => {{
            const form = new FormData();
            form.append('file', new Blob([Uint8Array.from(bytes)], {{ type: 'image/jpeg' }}), 'photo.jpg');
            form.append('purpose', 'post');
            form.append('alt', alt);
            const res = await fetch('/api/v1/media', {{ method: 'POST', credentials: 'same-origin', body: form }});
            return {{ status: res.status, body: await res.json().catch(() => null) }};
        }})({}, {})",
        json!(bytes),
        json!(alt),
    );
    let out = ctx.api_page().await?.eval(&script).await?;
    if out["status"]
        .as_u64()
        .is_none_or(|s| !(200..300).contains(&s))
    {
        bail!("upload: {out}");
    }
    let body = &out["body"];
    Ok(Uploaded {
        id: body["id"]
            .as_str()
            .context("upload returned no id")?
            .to_owned(),
        width: body["width"].as_u64().unwrap_or_default(),
        height: body["height"].as_u64().unwrap_or_default(),
    })
}

/// Everything [`seed`] made.
#[derive(Debug)]
pub struct Seeded {
    /// The seeded user's session. Open more pages with [`Ctx::page`]; they share its cookies and
    /// IndexedDB (the encryption key).
    pub ctx: Ctx,
    /// The seeded user's handle.
    pub handle: String,
    /// The second user's handle.
    pub other: String,
    /// The first post.
    pub post_id: String,
    /// The list.
    pub list_id: String,
}

/// One signed-in user with a post, a reply, a like from a second user and a follow, so that every
/// signed-in page has something to show. Costs two sign-ups.
pub async fn seed() -> Result<Seeded> {
    let me = handle("vis");
    let other = handle("oth");
    let post = |text: &str| json!({"text": text, "cw": "", "sensitive": false, "mediaIds": []});

    let mine = Ctx::new().await?;
    sign_up_via_api(&mine, &me, "Visual Tester").await?;
    let first = call(
        &mine,
        "POST",
        "/posts",
        Some(&post("Two inks, one pixel grid. #dither")),
    )
    .await?;
    let post_id = first["id"].as_str().context("post has no id")?.to_owned();

    let theirs = Ctx::new().await?;
    sign_up_via_api(&theirs, &other, "Other Person").await?;
    call(&theirs, "POST", &format!("/posts/{post_id}/like"), None).await?;
    call(&theirs, "POST", &format!("/users/{me}/follow"), None).await?;
    let reply = json!({"text": format!("@{me} a reply to you"), "cw": "", "sensitive": false, "replyTo": post_id, "mediaIds": []});
    call(&theirs, "POST", "/posts", Some(&reply)).await?;
    // Direct messages are sealed in the browser, so both sides turn encryption on and the key stays
    // in IndexedDB of the context for the pages that open the conversation.
    let mine_page = mine.page().await?;
    let their_page = theirs.page().await?;
    turn_on(&mine_page, PASSPHRASE).await?;
    turn_on(&their_page, PASSPHRASE).await?;
    their_page.goto(&format!("/messages/{me}")).await?;
    say(&their_page, "Hello from the other side.").await?;
    their_page.close().await?;
    mine_page.close().await?;
    let report = json!({"postId": post_id, "reason": "spam", "note": "Looks like an advert."});
    call(&theirs, "POST", "/reports", Some(&report)).await?;
    promote(&me).await?;
    call(
        &mine,
        "POST",
        "/tokens",
        Some(&json!({"name": "weather bot", "scopes": ["read", "write"]})),
    )
    .await?;
    staff_trail(&mine, &theirs, &other).await?;
    theirs.close().await?;

    let mut photos = Vec::new();
    for i in 0..2u32 {
        let image = make_image(900, 600, 30 + i * 120)?;
        photos.push(upload_image(&mine, &image, &format!("Gradient number {}", i + 1)).await?);
    }
    let media_ids: Vec<&str> = photos.iter().map(|p| p.id.as_str()).collect();
    let body = json!({"text": "Two photos, as uploaded.", "cw": "", "sensitive": false, "mediaIds": media_ids});
    call(&mine, "POST", "/posts", Some(&body)).await?;
    let poll = json!({
        "text": "Which ink?", "cw": "", "sensitive": false, "mediaIds": [],
        "poll": {"options": ["Bone", "Phosphor", "Amber"], "multiple": false, "durationHours": 24}
    });
    call(&mine, "POST", "/posts", Some(&poll)).await?;
    let list_body = json!({"name": "Pixel people", "description": "People who work in two inks.", "private": false});
    let list = call(&mine, "POST", "/lists", Some(&list_body)).await?;
    let list_id = list["id"].as_str().context("list has no id")?.to_owned();
    call(
        &mine,
        "POST",
        &format!("/lists/{list_id}/members/{other}"),
        None,
    )
    .await?;
    let later = mine
        .api_page()
        .await?
        .eval("new Date(Date.now() + 3 * 3600000).toISOString()")
        .await?;
    let scheduled = json!({"publishAt": later, "text": "Scheduled for later.", "cw": "", "sensitive": false, "mediaIds": []});
    call(&mine, "POST", "/scheduled", Some(&scheduled)).await?;

    Ok(Seeded {
        ctx: mine,
        handle: me,
        other,
        post_id,
        list_id,
    })
}

/// [`promote`] on another instance's database (a [`Peer`]'s).
pub async fn promote_on(database: &str, who: &str) -> Result<()> {
    let Some(bin) = &env().tuita_bin else {
        bail!("E2E_TUITA_BIN comes from scripts/e2e.sh");
    };
    let out = Command::new(bin)
        .args(["admin", "promote", who])
        .env("DATABASE_URL", database)
        .output()
        .await
        .with_context(|| format!("running {} admin promote", bin.display()))?;
    if !out.status.success() {
        bail!(
            "admin promote {who}: {}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

/// Makes an account an admin through the same command line operators use. Needs `scripts/e2e.sh`.
pub async fn promote(who: &str) -> Result<()> {
    let Some(database) = &env().database_url else {
        bail!("E2E_DATABASE_URL comes from scripts/e2e.sh");
    };
    promote_on(database, who).await
}

/// A fresh context signed up as a new handle with `prefix`, promoted to admin, with its cookies.
pub async fn new_admin(prefix: &str) -> Result<(Ctx, String)> {
    let who = handle(prefix);
    let ctx = Ctx::new().await?;
    sign_up_via_api(&ctx, &who, "Admin Person").await?;
    promote(&who).await?;
    Ok((ctx, who))
}

/// One of each kind of staff action, so the audit tab, the invite list and the filter scan have
/// something to show: verify, organization and role changes, a suspension, a dismissed report, a
/// deleted post, an announcement that was ended, invites (one revoked), a changed filter and an
/// account the new filter flags.
async fn staff_trail(staff: &Ctx, theirs: &Ctx, other: &str) -> Result<()> {
    let user = format!("/admin/users/{other}");
    for (path, body) in [
        (user.clone(), json!({"verified": true})),
        (user.clone(), json!({"verified": false})),
        (format!("{user}/org"), json!({"org": true})),
        (format!("{user}/org"), json!({"org": false})),
        (user.clone(), json!({"role": "moderator"})),
        (user.clone(), json!({"role": "user"})),
    ] {
        call(staff, "PATCH", &path, Some(&body)).await?;
    }

    for (text, action) in [
        ("Buy followers today", "dismiss"),
        ("Free gift cards inside", "delete_post"),
    ] {
        let body = json!({"text": text, "cw": "", "sensitive": false, "mediaIds": []});
        let post = call(theirs, "POST", "/posts", Some(&body)).await?;
        let post_id = post["id"].as_str().context("post has no id")?;
        call(
            staff,
            "POST",
            "/reports",
            Some(&json!({"postId": post_id, "reason": "spam", "note": ""})),
        )
        .await?;
        let open = call(staff, "GET", "/admin/reports?status=open", None).await?;
        let report = open["items"]
            .as_array()
            .and_then(|items| items.iter().find(|r| r["post"]["id"] == post_id))
            .context("the report just filed is not in the open queue")?;
        let id = report["id"].as_str().context("report has no id")?;
        call(
            staff,
            "POST",
            &format!("/admin/reports/{id}/resolve"),
            Some(&json!({"action": action})),
        )
        .await?;
    }

    // Last: a suspension ends the account's sessions, and `theirs` is done by now.
    call(staff, "PATCH", &user, Some(&json!({"suspended": true}))).await?;
    call(staff, "PATCH", &user, Some(&json!({"suspended": false}))).await?;

    let note = call(
        staff,
        "POST",
        "/admin/announcements",
        Some(&json!({"text": "Seeded announcement", "tone": "info", "days": 1})),
    )
    .await?;
    let note_id = note["id"].as_str().context("announcement has no id")?;
    call(
        staff,
        "DELETE",
        &format!("/admin/announcements/{note_id}"),
        None,
    )
    .await?;
    call(
        staff,
        "POST",
        "/admin/invites",
        Some(&json!({"note": "for the design team", "maxUses": 5, "days": 30})),
    )
    .await?;
    let spent = call(
        staff,
        "POST",
        "/admin/invites",
        Some(&json!({"note": "one use, revoked", "maxUses": 1, "days": 7})),
    )
    .await?;
    let code = spent["code"].as_str().context("invite has no code")?;
    call(staff, "DELETE", &format!("/admin/invites/{code}"), None).await?;

    // Every run seeds; once the first one set the filter, the name is refused, and one flagged
    // account is all the scan needs.
    let flagged = Ctx::new().await?;
    let signup = json!({"handle": handle("flg"), "displayName": "Zzquux Person", "password": PASSWORD, "acceptTerms": true, "add": false});
    let (status, body) = flagged.call("POST", "/auth/signup", Some(&signup)).await?;
    if !(200..300).contains(&status) && status != 400 {
        bail!("flagged sign-up: {status} {body}");
    }
    flagged.close().await?;
    let settings = call(staff, "GET", "/admin/settings", None).await?;
    let mut filter = settings["filter"].clone();
    filter["blocked"] = json!("zzquux");
    call(
        staff,
        "PATCH",
        "/admin/settings",
        Some(&json!({"filter": filter})),
    )
    .await?;
    Ok(())
}

/// Box metrics only, no pixels and no font dependence: the page must not scroll sideways, and no
/// element's text may spill out of its own box or be cut off by it. Containers that scroll on
/// purpose (`overflow-x: auto`) and text that truncates with an ellipsis are fine. Empty means fine.
pub async fn overflowing(page: &Page) -> Result<Vec<String>> {
    let found = page
        .eval(
            r#"(() => {
        const found = [];
        const root = document.documentElement;
        if (root.scrollWidth > root.clientWidth) {
            found.push(`page scrolls sideways: ${root.scrollWidth} > ${root.clientWidth}`);
        }
        const label = (el) =>
            `${el.tagName.toLowerCase()}${el.className && typeof el.className === 'string' ? `.${el.className.split(' ')[0]}` : ''} "${(el.textContent ?? '').trim().slice(0, 30)}"`;
        const scrolls = (el) => /auto|scroll/.test(getComputedStyle(el).overflowX);
        for (const el of document.body.querySelectorAll('*')) {
            if (el.closest('svg, canvas, dialog:not([open]), [aria-hidden="true"]')) continue;
            const style = getComputedStyle(el);
            if (style.display === 'none' || style.display === 'contents' || style.visibility === 'hidden') continue;
            // Visually hidden text is a 1px box by design.
            if (el.clientWidth <= 1 || el.clientHeight <= 1) continue;
            if (style.textOverflow === 'ellipsis' || scrolls(el)) continue;
            const direct = [...el.childNodes].some((n) => n.nodeType === 3 && n.textContent?.trim());
            if (!direct) continue;
            if (el.scrollWidth > el.clientWidth + 1) {
                found.push(`${label(el)} text is wider than its box: ${el.scrollWidth} > ${el.clientWidth}`);
            }
            const clips = /hidden|clip/.test(style.overflowY);
            if (clips && el.scrollHeight > el.clientHeight + 1) {
                found.push(`${label(el)} text is cut off: ${el.scrollHeight} > ${el.clientHeight}`);
            }
            const box = el.getBoundingClientRect();
            let inScroller = false;
            for (let up = el.parentElement; up; up = up.parentElement) inScroller ||= scrolls(up);
            if (!inScroller && (box.right > root.clientWidth + 1 || box.left < -1)) {
                found.push(`${label(el)} sits outside the viewport: ${box.left}..${box.right}`);
            }
        }
        return found;
    })()"#,
        )
        .await?;
    Ok(found
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default())
}

/// A second, real tuita instance with its own database, for federation tests.
#[derive(Debug)]
pub struct Peer {
    /// `http://localhost:8083`, also its `PUBLIC_URL`.
    pub base: String,
    /// `localhost:8083`: the domain its accounts have on the first instance.
    pub domain: String,
    /// `DATABASE_URL` of the peer, for the command line (`admin promote`).
    pub database: String,
    child: Child,
}

impl Peer {
    /// Stops the server, so deliveries to it fail.
    pub async fn stop(mut self) -> Result<()> {
        if let Some(pid) = self.child.id() {
            let _ = Command::new("kill")
                .args(["-TERM", &pid.to_string()])
                .status()
                .await;
        }
        if tokio::time::timeout(Duration::from_secs(10), self.child.wait())
            .await
            .is_err()
        {
            self.child.kill().await?;
        }
        Ok(())
    }
}

/// Starts `E2E_TUITA_BIN serve` on `E2E_PEER_PORT` (8083) against a database derived from
/// `E2E_DATABASE_URL`. Needs `scripts/e2e.sh`, which also lets the first instance reach loopback.
/// The peer dies with the returned [`Peer`] (and with the test process); only one can run at a time
/// per port: a suite that needs a peer for the whole process while another module starts its own
/// uses [`start_peer_on`].
pub async fn start_peer() -> Result<Peer> {
    start_peer_on(env().peer_port, "peer").await
}

/// [`start_peer`] on `port`, against the database `<E2E database>_<name>`. A `static` peer is never
/// dropped, so a watchdog stops the server when the test process ends.
pub async fn start_peer_on(port: u16, name: &str) -> Result<Peer> {
    let (Some(bin), Some(database)) = (&env().tuita_bin, &env().database_url) else {
        bail!("E2E_TUITA_BIN and E2E_DATABASE_URL come from scripts/e2e.sh");
    };
    let domain = format!("localhost:{port}");
    let base = format!("http://{domain}");
    let main_name = database
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .split('?')
        .next()
        .unwrap_or_default();
    let name = format!("{main_name}_{name}");
    let name = name.as_str();
    let peer_database = match database.rfind('/') {
        Some(at) => {
            let query = database[at..].find('?').map_or("", |q| &database[at + q..]);
            format!("{}/{name}{query}", &database[..at])
        }
        None => bail!("E2E_DATABASE_URL has no database name: {database}"),
    };

    let psql = Command::new("psql")
        .arg(database)
        .args([
            "-q",
            "-c",
            &format!("DROP DATABASE IF EXISTS {name} WITH (FORCE)"),
            "-c",
            &format!("CREATE DATABASE {name}"),
        ])
        .output()
        .await
        .context("running psql")?;
    if !psql.status.success() {
        bail!("psql: {}", String::from_utf8_lossy(&psql.stderr));
    }

    let dir: PathBuf = std::env::temp_dir().join(format!("tuita-peer-{}", Uuid::now_v7()));
    tokio::fs::create_dir_all(&dir).await?;
    let log_path = dir.join("peer.log");
    let log = std::fs::File::create(&log_path)?;
    let mut child = Command::new(bin)
        .arg("serve")
        .env("DATABASE_URL", &peer_database)
        .env("PUBLIC_URL", &base)
        .env("BIND_ADDR", format!("127.0.0.1:{port}"))
        .env("DATA_DIR", &dir)
        .env("SOURCE_URL", "https://example.invalid/tuita")
        .env("FEDERATION_ALLOW_HTTP", "1")
        .env("FEDERATION_ALLOW_PRIVATE_NETS", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log))
        .kill_on_drop(true)
        .spawn()
        .with_context(|| format!("starting {}", bin.display()))?;
    watch(child.id())?;

    let early = |log: &PathBuf| std::fs::read_to_string(log).unwrap_or_default();
    for _ in 0..120 {
        if let Some(status) = child.try_wait()? {
            bail!("peer exited early ({status}):\n{}", early(&log_path));
        }
        if ready(port).await {
            return Ok(Peer {
                base,
                domain,
                database: peer_database,
                child,
            });
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    bail!("peer not ready after 60 s:\n{}", early(&log_path))
}

/// `GET /readyz` over a bare socket: 200 means up.
async fn ready(port: u16) -> bool {
    let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)).await else {
        return false;
    };
    if stream
        .write_all(b"GET /readyz HTTP/1.0\r\nHost: localhost\r\n\r\n")
        .await
        .is_err()
    {
        return false;
    }
    let mut head = Vec::new();
    let _ = stream.take(64).read_to_end(&mut head).await;
    head.starts_with(b"HTTP/1.1 200") || head.starts_with(b"HTTP/1.0 200")
}

/// Waits for `$1` (the test process) to exit, then stops `$2` (the peer); returns early when the peer
/// is gone, so a recycled pid is never signalled.
const PEER_WATCHDOG: &str = r#"
while kill -0 "$1" 2>/dev/null; do
  kill -0 "$2" 2>/dev/null || exit 0
  sleep 0.3
done
kill -TERM "$2" 2>/dev/null
"#;

/// A detached `sh` that stops the peer when this process ends, which `kill_on_drop` cannot do for a
/// peer held in a `static`.
fn watch(peer: Option<u32>) -> Result<()> {
    let Some(peer) = peer else {
        return Ok(());
    };
    std::process::Command::new("sh")
        .args(["-c", PEER_WATCHDOG, "sh"])
        .arg(std::process::id().to_string())
        .arg(peer.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .context("starting the peer watchdog")?;
    Ok(())
}

/// Runs the known-answer vector `vector` (the source of an ES module exporting
/// `async function run(mod)`) against the island module the current document loaded as
/// `<script type="module" data-js="{name}">`. The vector throws on its first failed assertion,
/// which becomes the error. The page must have [`Page::bypass_csp`] set: the vector is imported
/// from a `blob:` URL.
pub async fn run_js_unit(page: &Page, name: &str, vector: &str) -> Result<()> {
    let selector = format!(r#"script[data-js="{name}"]"#);
    let script_tag = page.css(&selector);
    if script_tag.count().await? != 1 {
        bail!("expected one <script data-js=\"{name}\"> on the page");
    }
    let url = script_tag.attr("src").await?.unwrap_or_default();
    let script = format!(
        "(async () => {{
            const mod = await import({});
            const blob = new Blob([{}], {{ type: 'text/javascript' }});
            const vector = await import(URL.createObjectURL(blob));
            await vector.run(mod);
            return 'ok';
        }})()",
        json!(url),
        json!(vector),
    );
    page.eval(&script)
        .await
        .with_context(|| format!("js unit `{name}` failed"))?;
    Ok(())
}

/// Held for the whole test by every test that starts a [`Peer`]: they share one port and one
/// database, and the next peer replaces the previous one's database.
pub static PEER_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Runs `sql` against `database` (the main instance's or a [`Peer`]'s) with `psql`.
pub async fn psql(database: &str, sql: &str) -> Result<()> {
    let out = Command::new("psql")
        .arg(database)
        .args(["-q", "-v", "ON_ERROR_STOP=1", "-c", sql])
        .output()
        .await
        .context("running psql")?;
    if !out.status.success() {
        bail!("psql {sql}: {}", String::from_utf8_lossy(&out.stderr));
    }
    Ok(())
}

/// A TCP port nothing listens on: bound once to get a free number, then released.
pub async fn closed_port() -> Result<u16> {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
    Ok(listener.local_addr()?.port())
}

/// `POST /api/v1/admin/emojis` as multipart from a page of `ctx` (staff only).
pub async fn upload_emote(ctx: &Ctx, shortcode: &str, bytes: &[u8]) -> Result<Value> {
    let script = format!(
        "(async (bytes, shortcode) => {{
            const form = new FormData();
            form.append('shortcode', shortcode);
            form.append('file', new Blob([Uint8Array.from(bytes)], {{ type: 'image/jpeg' }}), 'e.jpg');
            const res = await fetch('/api/v1/admin/emojis', {{ method: 'POST', credentials: 'same-origin', body: form }});
            return {{ status: res.status, body: await res.json().catch(() => null) }};
        }})({}, {})",
        json!(bytes),
        json!(shortcode),
    );
    let out = ctx.api_page().await?.eval(&script).await?;
    if out["status"]
        .as_u64()
        .is_none_or(|s| !(200..300).contains(&s))
    {
        bail!("emote upload: {out}");
    }
    Ok(out["body"].clone())
}

/// [`psql`] for a query that returns one value (`psql -t -A`): that value, trimmed.
pub async fn psql_value(database: &str, sql: &str) -> Result<String> {
    let out = Command::new("psql")
        .arg(database)
        .args(["-q", "-t", "-A", "-v", "ON_ERROR_STOP=1", "-c", sql])
        .output()
        .await
        .context("running psql")?;
    if !out.status.success() {
        bail!("psql {sql}: {}", String::from_utf8_lossy(&out.stderr));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_owned())
}
