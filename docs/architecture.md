# Architecture

tuita: a small social platform with a 1-bit dither look that federates over ActivityPub.

Cargo workspace. Every crate lives under `crates/`; the root `Cargo.toml` owns versions, metadata and lints. The web client is the server-rendered Rust crate `tuita-web`; there is no JavaScript toolchain.

## Crate graph

```
crates/tuita         (bin: clap; `serve`, `migrate`, `admin`, `seed`)
        |
        v
crates/tuita-web     (lib: Topcoat pages, components, CSS, JS islands, l10n)
        |
        v
crates/tuita-server  (lib: Config, AppState, router, routes, db, workers)
        |
        v
crates/tuita-core    (lib: parsing, validation, feed scoring, ActivityPub types; no I/O)

crates/tuita-e2e     (lib + test target: Chromium browser suite; talks HTTP to a running server)
```

| Crate | Kind | Role |
|---|---|---|
| `tuita-core` | lib | Pure domain logic. Typed errors (`thiserror`). Documented, doctested API. Never depends on the other crates. |
| `tuita-server` | lib | HTTP server on axum and tokio, Postgres through sqlx. Hosts the JSON API, ActivityPub and background workers, and hands everything else to the web service passed to `router`. Testable in-process with a stub web service. |
| `tuita-web` | lib | The web client: server-rendered Topcoat pages, components, CSS, hand-written JS modules and Fluent translations. Calls the JSON API in-process; never touches the database. |
| `tuita` | bin | Parses arguments, reads `Config`, installs tracing, runs migrations, builds the web router from the asset bundle, dispatches the subcommand. |
| `tuita-e2e` | lib | The browser suite: drives Chromium over CDP (`chromiumoxide`) against a running server. Run it with `scripts/e2e.sh`; `cargo test --workspace` does not run it (`test = false`). |

Dependencies point one way: bin -> web -> server -> core (the bin also uses server directly). `tuita-server` never depends on `tuita-web`.

### `tuita-server` layout

| Path | Role |
|---|---|
| `src/lib.rs` | `backend` (health, `/api/v1`, `/api/v2` and the outside-API routes), `router(state, web)` (backend plus the web service as fallback behind the gate, plus the outer layers), `serve`, `serve_on`, `connect`, `migrate`, `shutdown_signal`, `init_tracing`, `CONTENT_SECURITY_POLICY`. The one place that wires routes and middleware. |
| `src/config.rs` | `Config`, `ConfigError`, `LogFormat`. |
| `src/error.rs` | `AppError`, rendered as `{"error": "..."}`. |
| `src/state.rs` | `AppState(Arc<Inner>)`: pool, config, readiness flag, realtime hub, rate limiters, shutdown token. |
| `src/dto.rs` | Every JSON request and response body. `tuita-web` shares them as Rust types. |
| `src/cursor.rs` | `before`/`limit` query parsing, opaque cursors, `Page` building. |
| `src/auth/` | Passwords (argon2id), session tokens and the `tuita_session`/`tuita_accounts` cookies, the account switcher, extractors (`MaybeUser`, `CurrentUser`, `Staff`, `Admin`, `ApiJson`, `ApiQuery`), the CSRF guard, the bearer-token gate (`bearer.rs`), the WebAuthn relying party (`passkey.rs`). |
| `src/limits.rs` | `RateLimits`: keyed `governor` limiters (`auth:{ip}`, `pw:{user}`, `post:{user}`, `follow:{user}`, and for OAuth `oauth-app:{ip}`, `oauth-token:{ip}`, `oauth-authorize:{user}`). |
| `src/realtime.rs` | `Hub`: one broadcast channel per user, feeds `GET /api/v1/stream`. |
| `src/db/` | SQL, one module per area: users, sessions, accounts, settings (incl. the content filter), posts (loader), post actions, compose (create, thread, edit, hide), polls, scheduled, views, media, messages, lists, reports, export, deletion, graph, timeline, search, notifications, audit, moderation (staff writes, shared with the CLI), admin_reads (console reads and stats), announcements, invites, tokens, passkeys, affiliations, emoji (custom emotes and the `emojis` hydration of DTOs), oauth (apps, codes, OAuth tokens), mastodon (lookups for the Mastodon mappers), and for federation: ap (local posts as AP data), remote (remote actors and posts), inbound (inbox writes), deliveries (the queue), federation (domain blocks, queue stats). Handlers never write SQL. |
| `src/media/` | Upload pipeline: type sniffing, re-encoding without metadata, crops, video streaming to a temp file. `store.rs` is `MediaStore`, the one interface over `object_store` (local disk, S3-compatible bucket, memory for tests): put, put from a file, head, ranged get, delete. A file's key is its `media.path`, `{id}.{ext}`. |
| `src/storage.rs` | `Storage`, `S3`, `S3Credentials`: the `STORAGE` and `S3_*` settings, parsed and validated by `Config::from_lookup`. |
| `src/net/` | `SafeClient`, the only way the server fetches a URL it did not write: a resolver that drops private address classes, port policy, redirect cap 4 with every hop re-checked, streamed body caps. |
| `src/federation/` | ActivityPub, server half. `ids` (every local object id from `PUBLIC_URL`), `keys` (RSA-2048 per local account, created at sign-up, lazily for older accounts, and the instance actor key in `instance_settings.instance_actor`), `fetch` (GETs signed by the instance actor), `lookup` (WebFinger, actor fetch, upsert, 24 h stale refresh, outbox backfill, `find_user` for `user@domain` handles), `ingest` (remote Note and Question into posts, DMs into messages, remote media copied through `SafeClient`), `render` (local actors and notes as AP JSON), `mastodon` (status JSON), `outbound` (one enqueue function per local action), `inbox/` (verify, dedupe, dispatch). The pure parts (types, signatures, HTML) are `tuita_core::ap`. |
| `src/routes/` | Handlers, one module per area. `head.rs` renders the Open Graph and link tags for profile and post URLs (the web layout embeds them). `fallback.rs` is the gate in front of the web service: backend prefixes without a route answer JSON 404, ActivityPub `Accept` on `/@...` is negotiated. `routes::api` and `routes::api_v2` build the two API routers with the layers every API route sits behind (CORS, CSRF guard, bearer gate, body limit). |
| `src/mastodon/` | The Mastodon-compatible client API, see [Mastodon API](#mastodon-api). One file per area; `dto` has the Mastodon entities and the mappers from tuita values, `ctx` the batched lookups they need, `page` the `Link` pagination, `params` the query/form/JSON parameters, `oauth` the OAuth2 provider. |
| `src/workers/` | Background workers. `spawn_workers` starts them; `serve_on` awaits them in the drain. `cleanup` runs hourly (sessions, snapshots, orphaned uploads, passkey challenges, `ap_seen` after 7 days, `ap_tombstones` after 30, OAuth codes a day past their expiry), `scheduler` every 5 s (`FOR UPDATE SKIP LOCKED`), `unfurl` drains an `mpsc` queue of links to preview, `delivery` sends the `ap_deliveries` queue every 2 s. |
| `src/admin.rs`, `src/seed.rs` | Bodies of `tuita admin ...` and `tuita seed`. |
| `migrations/` | sqlx migrations, embedded with `sqlx::migrate!`. |
| `tests/` | Integration tests. `#[sqlx::test]` gives each test a fresh database; `tests/common` has an in-process fake browser. |

### Typed API contract

DTOs in `src/dto.rs` derive `Serialize` and `Deserialize`; `tuita-web` reads and writes the JSON API through the same types.

## Web client

Request flow: `tuita_server::router(state, web)` matches the health, API and outside-API routes first. Everything else reaches the gate (`routes::fallback::gate`), which answers JSON `404` for a backend prefix without a route (`GET /oauth/authorize` is exempt), negotiates ActivityPub for `/@...`, and passes the rest to the Topcoat router that `tuita_web::router` builds. Pages read data and write through `tuita_web::api`, which calls the backend `Router` in-process with the browser's cookies and headers, so authentication, CSRF, rate limits and business rules stay in the API handlers. The developer guide is [`docs/web-client.md`](web-client.md); Topcoat behaviour is in [`docs/topcoat.md`](topcoat.md).

### `tuita-web` layout

| Path | Role |
|---|---|
| `src/lib.rs` | `router(state, assets)` and `Assets` (the hashed-asset bundle). |
| `src/app.rs`, `src/app/` | The module router: pages are modules discovered by Topcoat, file names are URL segments (`handle.rs` is `/@{handle}`), `_member/` holds the layer that sends guests to `/login`. `app.rs` has the root layout, the pathless `edge` layer (locale, `Cache-Control: no-store`, branded 404) and `error_page`. A request with the header `x-tuita-fragment: 1` renders only the page, and a paged page only its list items. |
| `src/api.rs`, `session.rs`, `paths.rs`, `next.rs`, `fragment.rs`, `cookies.rs`, `locale.rs`, `fmt.rs`, `icons.rs` | Shared helpers: the in-process API client, the per-request viewer, URL builders, safe `?next=` handling, fragment requests, cookies, locale resolution, ICU number and date formatting, the icon set. |
| `src/l10n.rs`, `src/l10n/<locale>.ftl` | Fluent messages in 12 locales read with `t!(key)`, and `CLIENT_STRINGS`, the subset sent to the browser. `scripts/lint-strings.sh` rejects literal text in markup. |
| `src/ui/`, `src/shell/` | Presentational components; the member frame (rail, bars, drawer, right rail, global dialogs). |
| `src/{admin,auth,compose,design,docs,explore,feed,lists,messages,notifications,oauth,post,profile,settings}/` | One module per area: its components and its `/_tuita/...` fragment routes. |
| `src/assets.rs`, `src/static_files.rs` | Every `asset!` (stylesheets, island modules, fonts, clips) and the fixed-URL files (favicon, app icons, web manifest, `robots.txt`). |
| `assets/` | `css/` (one stylesheet per area plus tokens, base, dither and `ui`), `js/` (islands), `fonts/`, `icons/`, `media/`, `static/`. |
| `tests/` | Page tests: the whole app in-process over Postgres with a cookie-keeping fake browser (`tests/common`). |

Islands: browser-only behavior (theme preferences, toasts, the event stream, the composer, end-to-end encryption, dithering, the post card, follow buttons, the lightbox and more: 30 modules) is plain ES2022 modules in `crates/tuita-web/assets/js/`, loaded on every page. A module attaches to `[data-island="<name>"]` and `data-act` elements, reaches the server only through the JSON API or `/_tuita/...` fragment routes, and talks to Topcoat through bubbling `CustomEvent`s. Topcoat hydrates only the initial page, so markup an island inserts is plain HTML that islands drive. There is no JS build step.

Assets: every CSS, JS, font and clip file is an `asset!` in the crate's `src/assets.rs`. `topcoat asset bundle --package tuita` (add `--release` for release) writes the content-hashed files and `manifest.toml` into an `assets/` directory beside the built binary; the server loads it at startup (`tuita_web::Assets::next_to_binary()`) and refuses to start without it. Editing an asset changes its URL, so re-bundle and restart. The `tuita` process of `devenv up`, `check`, `scripts/e2e.sh` and the Dockerfile run the bundle step.

CSP: one static policy, `tuita_server::CONTENT_SECURITY_POLICY`. `script-src 'self' 'unsafe-eval'` because Topcoat compiles its `$(...)` expressions in the browser with `new Function`; no inline `<script>` runs anywhere. Pages and fragments carry `Cache-Control: no-store`; `/_topcoat/assets/*` is immutable.

### Tests

- `cargo test --workspace` (part of `check`) runs the unit tests, the server integration tests and the `tuita-web` page tests. Render tests use `crate::testing`; page tests need Postgres and the bundle in `target/debug/assets`.
- `scripts/e2e.sh [filter]` runs the browser suite, `crates/tuita-e2e`: a harness that drives one headless Chromium over CDP (`chromiumoxide`) with strict, auto-waiting locators, and the suite in `tests/e2e/` (one module per area, plus `visual`, `languages`, `federation` and `js_units`, the known-answer vectors of the islands in `js/units/`). The script builds a release server, bundles the assets, starts the server on a throwaway database (`E2E_DB_NAME`, default `tuita_e2e`) and a federation peer port (`E2E_PEER_PORT`), then runs `cargo test --release -p tuita-e2e --test e2e`. The suite is outside `cargo test` (`test = false`) and so not part of `check`; `.github/workflows/e2e-nightly.yml` runs it once a day and on manual dispatch instead.

## Run

```sh
topcoat asset bundle --package tuita   # once, and after any asset change: the server needs it
devenv up -d                      # Postgres (and the server as a process)
cargo run -p tuita                # migrate, then serve on 127.0.0.1:8080
cargo run -p tuita -- migrate     # migrations only
curl -i localhost:8080/readyz
check                             # the full gate (scripts/check.sh)
scripts/e2e.sh                    # Rust + Chromium suite (crates/tuita-e2e) against a release server and a throwaway database
```

Logs go to stderr. `RUST_LOG=debug cargo run -p tuita` turns on debug logs.

## Configuration

All settings are environment variables, read once at startup by `Config::from_env()`. An invalid or missing value stops startup with an error that names the variable (`invalid DATABASE_URL: not set`). Copy `.env.example` to `.env` to override (devenv loads it).

| Variable | Default | Meaning |
|---|---|---|
| `DATABASE_URL` | required | Postgres URL. devenv sets `postgres://tuita@localhost/tuita`. |
| `BIND_ADDR` | `127.0.0.1:8080` | Socket address to listen on. The container image sets `0.0.0.0:8080`. |
| `REQUEST_TIMEOUT_SECS` | `30` | Per-request limit, whole seconds in 1..=3600. |
| `LOG_FORMAT` | `pretty` | `pretty` or `json`. Use `json` in production. |
| `RUST_LOG` | `info` | Log filter, `tracing-subscriber` env-filter syntax (`info,tuita_server=debug`). |
| `PUBLIC_URL` | `http://127.0.0.1:8080` | Public origin, no path. Its host is the federation domain. |
| `DATA_DIR` | `./data` | With `STORAGE=local`, media files live under `DATA_DIR/media`. |
| `STORAGE` | `local` | Where media files are kept: `local` (disk, for development) or `s3`. |
| `S3_BUCKET` | required for `s3` | Bucket name (3 to 63 lowercase letters, digits, dots, dashes). |
| `S3_REGION` | `us-east-1` | Signing region (`auto` for Cloudflare R2). |
| `S3_ENDPOINT` | unset | URL of an S3-compatible service (MinIO, R2, B2, Garage). Unset means AWS S3. No credentials, query or fragment in it. |
| `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY` | required for `s3` | The access key pair. Both are required: the ambient AWS credential chain (instance profiles, `AWS_*` variables) is not read. Neither shows in logs, errors or `Debug` output. |
| `S3_PATH_STYLE` | true when `S3_ENDPOINT` is set | Put the bucket in the URL path (`https://host/bucket/key`) instead of the host name (`https://bucket.host/key`). AWS S3 itself wants `false`. |
| `S3_PREFIX` | unset | Key prefix inside the bucket, for a bucket tuita shares. Slashes at the ends are dropped. |
| `SECURE_COOKIES` | true when `PUBLIC_URL` is https | Adds `Secure` to cookies. |
| `TRUST_PROXY` | `false` | Read the client IP from the last `X-Forwarded-For` hop. |
| `ADMIN_HANDLE` | unset | Only this handle may become the first admin. |
| `MAX_VIDEO_MB` | `50` | Largest video upload, 1..=1024 MiB. |
| `FEDERATION_ALLOW_HTTP` | true when `PUBLIC_URL` is http | Allow plain-http federation peers. Development only. |
| `FEDERATION_ALLOW_PRIVATE_NETS` | `false` | Let federation fetches and deliveries reach loopback and private addresses (the address-class filter of `net::SafeClient`; redirect, size and time caps stay). Development, tests and the Docker interop only; never inferred from `PUBLIC_URL`; logs a warning at startup. Pair it with `FEDERATION_ALLOW_HTTP` for `http://localhost:PORT` peers. |
| `SOURCE_URL` | unset | AGPL source link shown to users. Unset logs a startup warning. |
| `MESSAGES_E2EE_REQUIRED` | `true` | Direct messages between two accounts on this instance must be end-to-end encrypted. `false` lets them exchange plaintext until both have a message key; a recipient who has a key is never sent plaintext. See [Direct messages](#direct-messages). |
| `SECRET_KEY` | unset | 32 bytes as 64 hex characters or standard base64. Encrypts the ActivityPub signing keys in the database (AES-256-GCM); plaintext rows are sealed at startup. Once set, keep it: without it the sealed keys cannot be read and startup fails naming `SECRET_KEY`. Unset logs a startup warning. |

Add a setting: field on `Config`, a branch in `Config::from_lookup` that returns `ConfigError { var, .. }`, a row here, a case in the `test` module, and `.env.example`.

## Endpoints

| Route | Answer |
|---|---|
| `GET /healthz` | Liveness. `200 {"status":"ok"}` whenever the process answers. Checks no dependency, so a dead database never gets the pod killed. |
| `GET /readyz` | Readiness. `200 {"status":"ready"}` while serving and Postgres answers `SELECT 1` within 1 s; `503 {"status":"not ready"}` otherwise, including before startup and while draining. |
| Anything else | Handed to the web client service passed to `router`, after the fallback gate: `GET`/`HEAD` on `/@handle` with an ActivityPub `Accept` gets the actor or note instead. |
| `GET /media/{id}.{ext}`, `HEAD` | An uploaded file streamed from the media store with one `Range` (`206`; a range past the end is `416`, several ranges or a malformed header get the whole file), `ETag` and `Last-Modified` with `If-None-Match` and `If-Modified-Since` (`304`), `Cache-Control: public, max-age=31536000, immutable`, `Content-Disposition: inline`. Avatars and banners are also served at `/media/{id}` without the extension. |
| `GET /u/{handle}/rss.xml` | RSS 2.0 of the 30 newest top-level posts of a local account, `Cache-Control: public, max-age=300`. |
| Backend prefixes (`/api`, `/media`, `/.well-known`, `/users`, `/inbox`, `/nodeinfo`, `/actor`, `/oauth`, `/u/*/rss.xml`) without a route, any method | `404 {"error":"not found"}`. |
| `GET /oauth/authorize` | The web client's consent page, an exception to the `/oauth` backend prefix. `POST /oauth/authorize` (session cookie and CSRF guard), `POST /oauth/token`, `POST /oauth/revoke` and `GET /.well-known/oauth-authorization-server` are the OAuth endpoints; see [Mastodon API](#mastodon-api). |
| `GET /.well-known/webfinger`, `/.well-known/host-meta`, `/.well-known/nodeinfo`, `/nodeinfo/2.0`, `/nodeinfo/2.1` | Discovery, with CORS (any origin, preflights answered). WebFinger answers `acct:{handle}@{domain}` and `acct:{domain}@{domain}` (the instance actor) with `self` and `profile-page` links. `/.well-known/nodeinfo` links both NodeInfo versions; 2.1 adds `software.repository`. |
| `GET /users/{handle}` (+ `/outbox`, `/followers`, `/following`, `/featured`, `/statuses/{id}`, `/statuses/{id}/activity`), `GET /actor` | ActivityPub objects as `application/activity+json`. Followers and following are counts only. A deleted account is `410` with a `Tombstone`. Public and unlisted posts only. |
| `POST /users/{handle}/inbox`, `POST /inbox` | Signed activities. `202` once the signature verified, even when the activity is dropped; `401` for a missing or bad signature; `413` over 1 MiB; limits `inbox:{ip}` 300/min and `inbox-domain:{domain}` 1200/min. |
| `GET /@{handle}`, `/@{handle}/{post}` with an ActivityPub `Accept` | `302` to the canonical `/users/...` URL for local accounts; the web client's page otherwise. |
| `GET /oembed?url=`, `GET /api/v1/statuses/{id}` | oEmbed `rich` (a blockquote, no iframe) for public local posts; Mastodon status JSON for public and unlisted ones. Post pages carry `<link rel="alternate">` to both and to the AP note. |

Errors: handlers return `Result<_, AppError>`. `AppError` renders `{"error": "<message>"}` with the matching status. `Internal` logs the cause and sends only `internal error`. No `unwrap()` outside tests.

## JSON API

Everything under `/api/v1`, JSON in camelCase, errors `{"error": "<message>"}`. The route table is `crates/tuita-server/src/routes/mod.rs`; the types are `crates/tuita-server/src/dto.rs`.

| Area | Routes |
|---|---|
| Instance | `GET /instance` |
| Session | `POST /auth/signup`, `/auth/login`, `/auth/logout`; `GET /accounts`, `POST /accounts/switch` |
| Account | `GET /me`, `PATCH /me`, `POST /me/password`, `GET|DELETE /me/sessions`, `DELETE /me/sessions/{id}`, `GET /me/blocks`, `/me/mutes`, `GET /badges`, `GET /suggestions` |
| Timelines | `GET /timelines/following`, `/timelines/foryou`, `/timelines/{feed}/new`, `GET /bookmarks` |
| Posts | `POST /posts`, `GET|DELETE /posts/{id}`, `POST|DELETE /posts/{id}/like|repost|bookmark|pin`, `GET /posts/{id}/likes|reposts|quotes` |
| Users | `GET /users/{handle}`, `/posts`, `/followers`, `/following`, `/followed_by_known`, `POST|DELETE /users/{handle}/follow|block|mute` |
| Discovery | `GET /search`, `/hashtags`, `/trends`, `/tags/{tag}/posts` |
| Notifications | `GET /notifications`, `POST /notifications/read` |
| Media | `POST /media` (multipart), `PATCH /media/{id}`, `DELETE /me/avatar`, `DELETE /me/banner` |
| Posting extras | `POST /posts/thread`, `PATCH /posts/{id}`, `GET /posts/{id}/edits`, `POST /posts/{id}/vote`, `POST|DELETE /posts/{id}/hide`, `POST /views`, `GET|POST /scheduled`, `PATCH|DELETE /scheduled/{id}`, `POST /scheduled/{id}/publish` |
| Messages | `GET /conversations`, `GET|POST /conversations/{handle}`, `POST /conversations/{handle}/read`, `DELETE /conversations/{handle}/messages/{id}` |
| Lists | `GET|POST /lists`, `GET|PATCH|DELETE /lists/{id}`, `GET /lists/{id}/timeline|members`, `POST|DELETE /lists/{id}/members/{handle}` |
| Safety and data | `POST /reports`, `GET /me/export`, `DELETE /me` |
| Public | `GET /announcements` |
| Bots and tokens | `PATCH /me/bot`, `GET|POST /tokens`, `DELETE /tokens/{id}` |
| Custom emotes | `GET /emojis` (public); staff: `GET|POST /admin/emojis` (multipart), `PATCH|DELETE /admin/emojis/{shortcode}` |
| Affiliations | `GET|POST /affiliations`, `POST /affiliations/{id}/accept`, `PATCH|DELETE /affiliations/{id}`, `GET /orgs/{handle}/affiliates` |
| Passkeys | `GET /passkeys`, `POST /passkeys/register/options`, `POST /passkeys/register`, `PATCH|DELETE /passkeys/{id}`, `POST /auth/passkey/options`, `POST /auth/passkey` |
| Admin (staff or admin, cookie only) | `GET /admin/stats|users|users/{ref}|reports|filter/scan`, `PATCH|DELETE /admin/users/{ref}`, `DELETE /admin/users/{ref}/posts`, `POST /admin/users/{ref}/rename`, `PATCH /admin/users/{ref}/org`, `POST /admin/reports/{id}/resolve`; admin only: `GET|PATCH /admin/settings`, `GET /admin/audit`, `GET|POST /admin/announcements`, `DELETE /admin/announcements/{id}`, `GET|POST /admin/invites`, `DELETE /admin/invites/{code}`, `GET /admin/federation`, `POST /admin/federation/blocks`, `PATCH|DELETE /admin/federation/blocks/{domain}`, `POST /admin/federation/retry` |
| Realtime | `GET /stream` (server-sent events) |

- Sessions: random 32-byte token, only its SHA-256 stored, 30 days. Cookies `tuita_session` (active) and `tuita_accounts` (up to four more) are `HttpOnly; SameSite=Lax`, `Secure` when `SECURE_COOKIES`. One browser holds five accounts.
- CSRF: a non-GET request with a session cookie must send `Origin` equal to the `PUBLIC_URL` origin, else `403 bad origin`. The guard keys on the cookie only, so a script that sends just `Authorization: Bearer` is never checked and a request that carries a cookie always is.
- API tokens: `Authorization: Bearer tuita_<43 chars>` (scopes `read`, `write`, `messages`; made at `POST /tokens`, the secret is shown once, only its SHA-256 is stored, at most 20 per account, 600 requests per 10 minutes each). A bearer that is malformed, unknown or revoked is `401`, never a cookie fallback; a valid one identifies the request and cookies are ignored for identity. Tokens are refused (`403`) on `/tokens`, `/admin`, `/auth`, `/passkeys`, `/oauth` (the consent card), exactly `/accounts` and `/accounts/switch`, `/me/password`, `/me/sessions`, `/me/export` and `DELETE /me`; `/conversations` needs `messages`, other `GET` need `read`, the rest `write` (`src/auth/bearer.rs`). OAuth tokens (see Mastodon API) share the table and these rules, except that they never carry `messages`: `/conversations` needs plain `read` or `write` for them.
- Passkeys (WebAuthn, `webauthn-rs`, no OpenSSL): relying party id is the host of `PUBLIC_URL` (an IP address has none), origin is `PUBLIC_URL`, user verification is required, discoverable credentials are requested. A ceremony's state is a `webauthn_challenges` row (5 minutes, single use, expired rows removed by `cleanup`). Passkey sign-in sets the same cookies as password sign-in.
- Staff actions (`/admin/...`) write an `audit_log` row in the action's own transaction; `tuita admin ...` calls the same `db::moderation` functions with a null actor.
- The word filter (`filter::ContentFilter`) blocks nothing until the admin adds words to `instance_settings.content_filter` (blocked, allowed, reserved); then it guards sign-up, rename, profile edit, lists and posts.
- Rate limits answer `429` with `Retry-After`. The client IP is the peer address, or, when `TRUST_PROXY` is set, the last value of the last `X-Forwarded-For` header line (the proxy must overwrite that header or append to it; everything before its value is client-supplied). `CF-IPCountry` is read only under the same trust.
- Lists return `{items, next}`. Send `next` back as `before`; the For you timeline uses `cursor`. Cursors are opaque.
- `GET /stream` sends `retry: 5000`, a `badges` event, then live events (`notification`, `read`, ...). Each frame has `event: <type>` and `data: {type, data}`. A comment keeps the connection alive every 25 s. Streams end when the server drains. The hub is in-process, so run one replica until it moves to Postgres `LISTEN/NOTIFY`.
- Media: images (10 MiB; JPEG, PNG, WebP, GIF) are decoded and re-encoded, so EXIF is gone: JPEG stays JPEG (quality 82), PNG and WebP become lossless WebP, GIF stays an animated GIF, long edge at most 2048 px. Videos (MP4, WebM, MOV, up to `MAX_VIDEO_MB`) stream to a temp file under the OS temp dir and are sniffed by magic bytes, then scrubbed in place before they are stored (`media/scrub.rs`): MP4/MOV `udta`, `meta`, `uuid` and `XMP_` boxes and WebM `Tags` and `Info` title, date and tool elements are retyped to `free`/`Void` with their payload zeroed, and the `mvhd`/`tkhd`/`mdhd` times are zeroed, so size and offsets stay the same and playback is unchanged; content the scrub cannot parse is stored as it came, from the first bad box on. A remote video is scrubbed the same way. Both are then stored in the `MediaStore` (`DATA_DIR/media` or an S3-compatible bucket, see `STORAGE`); an image is put from memory, a video is uploaded from its temp file (multipart above 10 MiB) and the temp file is removed on every path. `GET /media/{id}.{ext}` streams from the store, so a bucket is never exposed. An upload no post, avatar, banner or link card holds is deleted after 24 h by `cleanup`.
- Outbound fetches (link previews now, federation later) go through `net::SafeClient` only; see `src/net`. A link card is built by the `unfurl` worker after a post is created or edited and arrives as a `card` stream event.
- Muted words are phrases (`cat` hides `concatenate`); a muted `#tag` matches `post_tags`. Timelines, search, tag lists, list timelines, thread replies and the post loader all apply them in SQL.
- Custom emotes: `:shortcode:` images shared by the whole instance. The `custom_emojis` table (`shortcode`, `domain` null for local, `media_id`, `category`, `source_url` for remote copies, `created_by`) is unique on `(lower(shortcode), coalesce(domain, ''))`, so a local and a remote emote may share a name. A shortcode is 2 to 32 ASCII letters, digits or underscores; a token in text is `:shortcode:` with the opening colon at the start or after a character that is not a word character or `/`, and the closing colon followed by a non-word character (`tuita_core::emoji`, Mastodon's rule; `12:30:45` holds none). Staff (moderators and admins) manage the local emotes: `GET /emojis` is public, `POST /admin/emojis` takes multipart `shortcode`, optional `category` (32 characters) and `file` (PNG, JPEG, WebP or GIF up to 256 KiB, re-encoded to at most 128 px on the long edge through the media pipeline, animated GIFs stay animated), `PATCH /admin/emojis/{shortcode}` sets `{category}` and `DELETE` removes the row, its `media` row and file. At most 1000 local emotes; every write is audited (`emoji.create|update|delete`). Cleanup never reaps media a `custom_emojis` row uses. Renaming is not supported, and deleting leaves `:name:` as plain text in stored posts. Hydration: `PostDto`, `QuoteDto`, `UserDto`, `UserRefDto` and `MessageDto` carry `emojis: EmojiDto[]` (`shortcode`, `url`, `category`), the distinct emotes their text mentions (post text, content warning and poll options; display name, bio and location; message text). `db::emoji::hydrate` resolves a whole response with one query, matching an author's text against that author's origin only: local emotes for a local author, the copied emotes of that domain for a remote one; unknown shortcodes are absent. Federation and the Mastodon API reuse `db::emoji::{resolve, upsert_remote}`.
- The `tuita-server` crate also serves `/@{handle}` and `/@{handle}/{post}` with real Open Graph tags; deleted, suspended or unknown targets get the generic ones.

## Mastodon API

Mastodon clients (Phanpy, Ice Cubes, Tusky, Elk, Ivory) sign in with OAuth2 and call a subset of the v1 and v2 API. The routes live in `src/mastodon/`, below `/api/v1` and `/api/v2`, behind the same CSRF guard and bearer gate as tuita's own API. JSON is snake_case and follows <https://docs.joinmastodon.org/entities/>; ids are UUID strings (a post id, a repost entry id for a boost, a user id).

- OAuth2 provider (`mastodon/oauth.rs`, `db/oauth.rs`, migration `0007_oauth.sql`): `POST /api/v1/apps` registers a client (JSON or form; limited to 30 per hour per address), `GET /api/v1/apps/verify_credentials`, `GET /oauth/authorize` (the web client's consent card), `GET /api/v1/oauth/authorization` and `POST /oauth/authorize` (the two calls the card makes, session cookie only, the second behind the CSRF guard), `POST /oauth/token` (`authorization_code` and `client_credentials`; 60 per 10 minutes per address), `POST /oauth/revoke`, `GET /.well-known/oauth-authorization-server`.
- Safety rules: a redirect URI is compared by exact string at the authorization request and at the exchange (allowed: `https`, `http` to loopback, the out-of-band URI, an app scheme `app://`; never a fragment, credentials or `javascript:`/`data:`/`file:`). A code is stored as a SHA-256, single use, valid 10 minutes; presenting a used code again deletes the token it made. PKCE is `S256` only: a code made with a challenge is only redeemed with the verifier, a code made without one needs the client secret. Client secrets are stored as a SHA-256 and compared in constant time. Scopes are `read`, `write`, `follow` (stored as `write`) and `push` (dropped); granular scopes are refused and a request may not exceed what the app registered. Nothing logs a code, a secret or a token.
- OAuth tokens are `api_tokens` rows with `app_id` set. They never expire, there are no refresh tokens, an account keeps at most 100 (the oldest go), and they show in the user's token list under the app's name. A `client_credentials` token has no user (`user_id` null): it only passes `GET /apps/verify_credentials`. `Auth::oauth_app()` tells a handler that an OAuth token made the request.
- Where a Mastodon route shares its method and path with a tuita route (`GET /instance`, `/notifications`, `/lists`, `/bookmarks`, `/conversations`, `/announcements`, `/suggestions`), `mastodon::by_credential(tuita_handler, mastodon_handler)` registers it once: a request authenticated by an OAuth token gets the Mastodon handler, sessions and personal tokens keep the tuita one.
- CORS: `/api/v1`, `/api/v2`, `/oauth/token`, `/oauth/revoke`, the OAuth metadata and the discovery documents (`/.well-known/webfinger`, `/.well-known/host-meta`, `/.well-known/nodeinfo`, `/nodeinfo/2.0`, `/nodeinfo/2.1`) answer `Access-Control-Allow-Origin: *` (never credentials), allow `Authorization`, `Content-Type` and `Idempotency-Key`, expose `Link`, and answer preflights, all from the one `mastodon::cors()` layer. `/oauth/authorize` is not a CORS endpoint.
- Start-up and screen routes (`mastodon/misc.rs`, `db/mastodon_misc.rs`): `preferences`, `followed_tags`, `follow_requests`, `endorsements`, `domain_blocks`, `blocks`, `mutes`, `filters` (v1 and v2, the viewer's muted words), `announcements`, `trends/tags|statuses|links`, `suggestions` (v1 and v2), `accounts/familiar_followers`, `instance/rules` and `GET /api/v2/notifications` (one group per notification). Real tuita data where there is some (block and mute lists, muted words, announcements, trending tags with a week of daily use, suggestions, known followers, rules); `[]` where there is none (tuita has no locked accounts, followed tags, endorsements, per-user domain blocks, trending posts or links).
- Pagination (`mastodon/page.rs`): `max_id`, `since_id`, `min_id` and `limit` (default 20, at most 40) become UUID or opaque cursors; the answer carries `Link` with absolute `next` and `prev` URLs built from `PUBLIC_URL`. `mastodon/params.rs` reads query, form and JSON into one flat parameter map.
- Mapping: `mastodon::dto` has every entity and the mappers (`status`, `account`, `notification`, ...); a handler builds one `mastodon::ctx::Ctx` per response (`Ctx::for_posts`) so the mappers never query. Status HTML is the one federation sends (`federation::render::content_html`). A `visibility` of `followers` is `private`; a repost entry is a boost status whose `reblog` holds the post; `fields` holds a profile's website, location and pronouns; `filtered` is empty; a remote post's `uri` is its page URL. Custom emotes: `GET /api/v1/custom_emojis` lists the local emotes as `CustomEmoji` (`shortcode`, absolute `url`, `static_url` equal to `url` because tuita keeps one image per emote, `visible_in_picker` true, `category`); `Account.emojis` comes from `UserDto.emojis` (`Ctx::load` hydrates its account batch with one `db::emoji::hydrate` query), `Status.emojis` and `Poll.emojis` from `PostDto.emojis`, a direct-message status from `MessageDto.emojis`, and every edit-history entry from one `db::emoji::resolve` over all versions' text and content warning; a boost wrapper's own `emojis` is `[]` (the boosted status carries them).
- Streaming (`mastodon/streaming.rs`): `GET /api/v1/streaming?stream=user&access_token=` upgrades to a websocket with `update`, `delete` and `notification` events (`stream` `user` and `user:notification`, `subscribe`/`unsubscribe` messages, anything else an `error` event). Only a token opens it (query, `Sec-WebSocket-Protocol` echoed back, or `Authorization`; a cookie alone is `401`). It counts toward the 12 streams per user that SSE has, ends on revocation or suspension (same recheck), pings every 25 s and drops a client that misses three. The hub carries ids only: `db::live` publishes `StreamEvent::Update` (new top-level posts to the author and online followers, boosts to the booster's followers, after commit, from `compose::Effects` and `post_actions::react`) and `Delete` (`post_actions::delete`, unboost); the socket loads each post for the connected user, so blocks, mutes, muted words and visibility apply there. The SSE stream drops both kinds.
- Scopes: `read`, `write` and `follow` are granted (`follow` is stored as `write`); `push` is accepted and dropped; granular scopes (`read:statuses`) are `invalid_scope`. For an OAuth token a `GET` or `HEAD` needs `read`, any other method `write`, and `GET /api/v1/streaming` needs `read`. OAuth tokens never carry `messages`, so `/conversations` needs plain `read` or `write` for them. A token without the scope is `403`; a revoked or unknown one is `401`.
- OAuth allowlist (fail closed, `mastodon::ALLOWED` in `mastodon/mod.rs`): below `/api/v1` the bearer gate admits an OAuth token only on a path and method that table names, and answers `403` for any other, so a `read` or `write` grant never reaches tuita-only routes (`/me`, `/posts`, `/users/..`, `/reports`, `/affiliations`, `GET|POST /conversations/{handle}`, tuita's `POST /media`, ...). Everything below `/api/v2` is Mastodon and needs no row. A new Mastodon route needs a row; `every_registered_mastodon_route_is_open_to_oauth_tokens` in `tests/oauth.rs` fails when one is missing. Personal tokens and cookies are not affected.
- Supported route families (all need a sign-in, `401` otherwise; tuita has no guest reads):

| Family | Routes |
|---|---|
| Instance | `GET /api/v1/instance` (tuita's JSON plus the Mastodon v1 fields), `GET /api/v2/instance`, `/api/v1/instance/rules`, `/api/v1/custom_emojis`, `GET /api/v2/search` |
| Accounts | `verify_credentials`, `PATCH update_credentials`, `/accounts/{id}`, `lookup`, `relationships`, `familiar_followers`, `/accounts/{id}/statuses`, `followers`, `following`, `lists`, `POST /accounts/{id}/follow`, `unfollow`, `block`, `unblock`, `mute`, `unmute`, `GET /blocks`, `/mutes` |
| Statuses | `POST /statuses` (`Idempotency-Key`, scheduled), `GET`, `PUT`, `DELETE /statuses/{id}`, `context`, `source`, `history`, `favourite`, `unfavourite`, `reblog`, `unreblog`, `bookmark`, `unbookmark`, `pin`, `unpin`, `favourited_by`, `reblogged_by`, `/scheduled_statuses`, `/polls/{id}`, `/polls/{id}/votes` |
| Timelines | `/timelines/home`, `public`, `tag/{tag}`, `list/{id}`, `GET /bookmarks`, `/favourites` |
| Notifications | `GET /api/v1/notifications`, `/notifications/{id}`, `POST /notifications/clear`, `/notifications/{id}/dismiss`, `GET /api/v2/notifications` |
| Lists and conversations | `/lists`, `/lists/{id}`, `/lists/{id}/accounts`, `GET /conversations`, `DELETE /conversations/{id}`, `POST /conversations/{id}/read` |
| Media | `POST /api/v2/media` (multipart), `PUT /api/v1/media/{id}` |
| Screens | `preferences`, `filters` (v1 and v2, read only), `followed_tags`, `follow_requests`, `endorsements`, `domain_blocks`, `announcements`, `trends/tags|statuses|links`, `suggestions` (v1 and v2) |
| Streaming | `GET /api/v1/streaming` (websocket) |

- Known gaps, because tuita has no equivalent or support stops there:
  - Statuses: `visibility` other than `public` is `422` (direct messages are tuita's `/conversations`); poll `expires_in` under an hour and `hide_totals` are `422`; edits change text and `spoiler_text` only, within the 30 minute window (`media_ids`, poll and `sensitive` changes are `422`, and removing all media is not detected); `language`, `focus` and `with_rate_limit` are ignored; the `quote` entity is not mapped; pin keeps one post, pinning another replaces it; boosting a non-public status is `422` and a boost's own id is `404` (act on the original id); `timelines/public` and `tag` ignore `any[]`, `all[]` and `none[]`; `POST /statuses/{id}/mute`, `GET /api/v1/media/{id}`, v1 `POST /media` (tuita's own shape), `markers` and filter writes are not offered.
  - Accounts: no `locked` accounts and no follow requests (`follow_requests_count` 0); no `source[privacy|sensitive|language]`; `fields_attributes` holds Website and Location only; follow `reblogs|notify|languages`, mute `duration|notifications`, list `replies_policy` and `exclusive`, and search `account_id` are ignored; `/follow_requests`, `/followed_tags`, `/endorsements`, `/domain_blocks`, `trends/statuses` and `trends/links` answer `[]`; `DELETE /suggestions/{id}`, announcement dismissal and reactions are not offered; announcements' `emojis` is always `[]`.
  - Notifications and conversations: `affiliation` rows never appear; `GET /api/v2/notifications/{group_key}`, `unread_count` and `policy` are not offered; each notification is its own group; `GET /conversations` pages by `max_id` only, and `DELETE` removes the viewer's own messages only.
  - OAuth: OAuth tokens do not expire and have no refresh tokens; registered apps are never pruned (bounded by the registration limit of 30 per hour per address).
  - Streaming: only `user` and `user:notification` streams exist (no `public`, `hashtag`, `list` or `direct`); suspending an account without purging its posts sends no `delete` to followers' sockets, and a `status.update` (edit) event is not sent; the Phanpy web client was run by hand against a tunnelled instance (sign-in, home timeline, post, favourite, notification: see spec 0022 Log), the other clients (Ice Cubes, Tusky, Elk) only through `scripts/mastodon-smoke.sh` and the test suites.
- Smoke test: `scripts/mastodon-smoke.sh <base-url>` drives the core client flow with curl and jq against a running instance (instance document, sign-up of two throwaway accounts, app registration, PKCE consent and token exchange, a confidential client, follow, post with `Idempotency-Key`, home timeline and `Link`, favourite and notification, media upload, websocket `update` frame when `websocat` is installed, delete, revoke, read-only token) and prints one `PASS` or `FAIL` line per step. It stops at the first failure. `<base-url>` must be the instance's `PUBLIC_URL`, because it is sent as `Origin`. `SMOKE_XFF=1` (default) sends a random `X-Forwarded-For` so an instance with `TRUST_PROXY=1` does not count every run against one address; `SMOKE_KEEP=1` keeps the throwaway accounts. Run it by hand after a deploy; it is not part of `check`.

## Federation

tuita speaks ActivityPub, hand-written on axum (no `activitypub_federation`). Mastodon, Misskey and GoToSocial payload shapes are fixtures in `crates/tuita-core/tests/fixtures/ap/`.

- Ids: actor `{PUBLIC_URL}/users/{handle}`, note `{PUBLIC_URL}/users/{handle}/statuses/{uuid}`, page `{PUBLIC_URL}/@{handle}/{uuid}`, instance actor `{PUBLIC_URL}/actor`. Local rows keep a null `ap_id`; the id is computed. A remote object that claims an id on this instance's authority is refused.
- Signatures: draft-cavage-12 `rsa-sha256` (the Mastodon dialect). POST signs `(request-target) host date digest content-type`, GET `(request-target) host date accept`. Verification rejects a `Date` more than 1 h off, a digest mismatch and a missing `(request-target)`, `host`, `date` (or `digest` on POST). The `keyId` actor must equal the activity `actor`.
- Keys: private keys live in `users.private_key_pem` and `instance_settings.instance_actor`. They never leave the server: not in a DTO, an export or a log. With `SECRET_KEY` set they are stored as `enc:v1:<nonce>:<ciphertext>` (AES-256-GCM, the row as additional data, so a value cannot be moved to another account); `federation::keys::protect_at_rest` seals plaintext rows at startup and is idempotent. A sealed row with `SECRET_KEY` missing or wrong stops startup. Rotating `SECRET_KEY` is not supported. Erasing an account keeps its key until the queued `Delete` it signs has been sent; `cleanup` then wipes it, and an erased account never gets a new one. `rsa` 0.10.0 fixed RUSTSEC-2023-0071; its `crypto-glue` dependency (webauthn-rs) asks for a feature 0.10.0 dropped, so `vendor/crypto-glue` is a two-line patched copy wired through `[patch.crates-io]`. Drop the patch when crypto-glue publishes a compatible release.
- Inbound: verify, then drop a `suspend`ed domain (before any fetch), dedupe on `ap_seen`, dispatch (Follow auto-accepted, Accept, Reject, Undo, Create, Update, Delete, Like, Announce, Block, Flag into a report filed by the `tuita` system account, Move ignored). A `Create` is stored only when it is addressed to a local user, replies to a local post, comes from an actor a local user follows or mentions a local user. Remote content is sanitised with `ammonia` and stored as plain text. Remote media (4 attachments, avatars, banners) is copied into `media` through `SafeClient`; readers never contact remote servers.
- Outbound: each local action writes `ap_deliveries` rows in its own transaction, one per distinct inbox (shared inbox when known). The `delivery` worker claims up to 50 due rows every 2 s under a 60 s lease, sends 8 at a time, deletes on 2xx, deletes on 4xx except 429, and otherwise retries at `min(2^attempts x 15 s, 6 h)` and gives up at 12 attempts (about two days). Rows for a `suspend`ed domain are deleted unsent. Admin > Federation shows the queue and has "Retry now".
- Lookup: `user@domain` in any `{handle}` route runs WebFinger, fetches the actor, upserts it and backfills 20 outbox posts. A profile older than 24 h is served stale while it refreshes in the background. Lookups are limited to 30 per 10 minutes per user (`lookup:{user}`) when the network is used. A `suspend`ed domain is a 404 without a request.
- Emotes: `tuita_core::ap::Tag::Emoji { id, name, icon, media_type }` is the `{"type":"Emoji","id","name":":shortcode:","icon":{"type":"Image","mediaType","url"}}` tag Mastodon, Misskey and GoToSocial send on notes and actors. It is read only when the name is `:shortcode:` with a valid shortcode (`tuita_core::emoji`) and the icon (a string, an object or the first object of an array) is an http(s) URL within `MAX_URL_LEN`; anything else is `Tag::Other`, and the sender's `id` falls back to the icon URL. `Actor` has a `tag` list for the emotes of its display name and bio; `ap::context()` defines `Emoji` as `toot:Emoji`. Outbound: `db::ap::load_posts_in` resolves the local emotes a post's text, content warning and poll options mention with one extra query for the whole batch (`LocalPost.emojis`), and `federation::render` writes one `Emoji` tag per emote after the mentions and hashtags, with an absolute `{PUBLIC_URL}/media/{file}` icon and `{PUBLIC_URL}/emojis/{shortcode}` as id; the actor document and the actor `Update` do the same for the display name and bio (`db::ap::local_emojis`, `mentioned_emojis`). A change to an emote is not re-sent. Inbound: for a note (text, content warning, poll options, and a direct message's text, also on an edit) and an actor (display name, bio, on every lookup, refresh and `Update`), `federation::ingest::copy_emotes` looks at the first 16 `Emoji` tags and keeps those whose `:shortcode:` the text uses; a copy is fetched again only when the tag's icon URL differs from the `custom_emojis.source_url` it was made from (Mastodon icon URLs carry a cache-buster, so a replaced image has a new URL); the replacement swaps `media_id` and `source_url` and the old `media` row, now unreferenced, is reaped by `cleanup`. Each icon is fetched through `SafeClient` (through the shared `federation::media::download`: http(s) only, never this instance's own authority; 256 KiB cap, 10 s, `image/*`, at most 4 emotes at a time process-wide), sniffed and re-encoded by the media pipeline as `Target::Emote` (128 px, animated GIF kept), stored as a `media` row owned by the author and recorded with `db::emoji::upsert_remote(domain, shortcode, media, source_url)`. Any failure skips that emote and leaves `:name:` as text. Readers then see the copy through the same `emojis` arrays as local emotes.
- Visibility: `public`, `unlisted`, `followers`, `direct`. Local posts are always public. Timelines, search, tags and trends read public posts of instances that are not `silence`d; the following feed and profile tabs never show `direct`; the post loader hides non-public posts from anyone who may not see them.
- Domain blocks: `silence` hides an instance's accounts from public timelines and search; `suspend` hides them like a suspension and drops all traffic to and from them.
- A domain block covers its subdomains (`evil.example` blocks `x.evil.example`) and ignores a trailing dot and capitals: `ap::url_authority` and the handle parser drop the dot, and `db::domain_in_block` does the suffix match in SQL for every check (lookup, inbox, delivery, enqueue, visibility). Account counts in the admin list stay exact-domain.
- Deleted remote notes: `delete_by_ap_id` records the note id in `ap_tombstones` (the author's own deletes, and an unknown id on the author's own instance). `store_note` and `insert_note` refuse a tombstoned id, so a `Create` or backfill that arrives after its `Delete` stays deleted; `cleanup` prunes rows after 30 days. A local delete also removes the note's `Create` and `Update` still waiting in `ap_deliveries`.
- Retention of deleted data: deleting a post removes its edits, polls, votes, views and uploads (row and stored object) at once and blanks the text of its scheduled-post record. A deleted message is scrubbed on delete and its tombstone is removed after 7 days with any queued delivery; finished scheduled posts go after 7 days; `cleanup` deletes a stored object before its `media` row and keeps the row when the object cannot be removed. Erasing an account also nulls its personal columns and deletes its poll votes, idempotency keys, view rows, filed reports and scheduled posts. A reply shows its parent's id and handle only to viewers who may see the parent, and delete, pin, hide, report and thread answer `404 Post not found.` for a post the viewer cannot see, exactly like a missing one.
- Development and tests: `FEDERATION_ALLOW_HTTP` allows plain http and odd ports; `FEDERATION_ALLOW_PRIVATE_NETS` also lets fetches reach loopback and private addresses. `tests/federation.rs` runs two instances against each other this way. `scripts/ap-interop.sh` (manual, not in `check`) runs GoToSocial against tuita in containers.

## Direct messages

Messages between two accounts on this instance are end-to-end encrypted. The browser makes an ECDH P-256 key pair, seals the private half under a passphrase (PBKDF2-SHA-256, 600 000 iterations, AES-256-GCM) and uploads the public key and that opaque backup to `message_keys` (`GET|PUT /me/message-key`, cookie only). Each message is sealed to the other person's public key (ECDH, HKDF-SHA-256 with a random salt, AES-256-GCM, padded to 128 bytes, additional data binds both account ids and both public keys) and stored as `messages.ciphertext` with `text = ''`. The server cannot open it. The code is `crates/tuita-web/assets/js/e2ee.js`.

- Mode: `ConversationViewDto.mode` says how a message to one person travels. `sealed` when both have a key; `plain` for a person on another server (ActivityPub has no encryption, the UI says so); `unavailable` while a local person has no key and `MESSAGES_E2EE_REQUIRED` is true. A recipient who has a key is never sent plaintext. A client that ever pinned a local person's key refuses `plain` for them, whatever the server says about where that person lives.
- Keys are pinned on first use in the browser (IndexedDB `tuita-e2ee`), which keeps every key the person has trusted for a contact. A different key from the server blocks sending until the person trusts it; trusting a reset key keeps the earlier messages readable. Safety numbers let two people compare keys out of band.
- Protected: message content against a database dump, a backup, and anyone who can read the database or the server's storage, administrators with SQL access included. Not protected: metadata (who, when, size to 128 bytes, read receipts); messages sent before a person turned encryption on; a server operator who changes the web app the server serves (the passphrase and the unlocked key live in that code, and no pin or CSP stops it); a server that serves a false key to someone who never opened the conversation; a compromised browser. There is no forward secrecy.
- Sealed messages never appear in plaintext on the SSE `message` event, the export (`ciphertext` only), the Mastodon conversations (placeholder text) or logs. Deleting a message scrubs its text and ciphertext. `PUT /me/message-key` is limited to 10 attempts per hour, valid or not.

## Maintenance commands

```sh
cargo run -p tuita -- admin promote <handle>      # also: moderator, demote, verify, unverify, suspend, unsuspend
cargo run -p tuita -- admin list                  # admins, moderators and verified accounts
cargo run -p tuita -- seed                        # 6 demo accounts and 40 posts, only in an empty database
```

## Middleware

Outermost first: request id (`x-request-id`: keep the caller's, else a UUID), `TraceLayer` (span per request with method, path, request id), request id copied to the response, security headers, timeout.

Security headers on every response: `X-Content-Type-Options: nosniff`, `Referrer-Policy: strict-origin-when-cross-origin`, `X-Frame-Options: DENY`, `Permissions-Policy: camera=(), microphone=(), geolocation=()` and a `Content-Security-Policy` (see [Web client](#web-client)), plus `Strict-Transport-Security: max-age=31536000` when `SECURE_COOKIES` is on (no `includeSubDomains`, no preload).

Under `/api/v1` the CSRF guard, the bearer-token gate and a 1 MiB body limit sit inside those layers, in that order. Outside them, `guarded()` adds `Cache-Control: no-store` and `Vary: Cookie, Authorization` to every API answer whose handler set no `Cache-Control` itself (the anonymous `GET /api/v1/statuses/{id}` keeps its public caching), so private JSON never sits in a shared or history cache. The request span logs the path with the handle of `/api/v1/conversations/{handle}...` replaced by `{handle}`: who messages whom is not for a log.

Timeouts return `408 Request Timeout` with an empty body, always. `503` is reserved for readiness, so the two never get confused in dashboards.

## Shutdown

`shutdown_signal()` completes on SIGINT or SIGTERM. Then `serve_on` flips `/readyz` to 503, stops accepting connections, lets in-flight requests finish (bounded by `REQUEST_TIMEOUT_SECS`), and returns `Ok(())`. Background workers get their own stop signal and are awaited inside the drain. Open event streams end on the same signal, or the drain would wait on them forever. A second signal is not handled specially; the orchestrator's kill timeout is the backstop. Set it above `REQUEST_TIMEOUT_SECS` (Kubernetes: `terminationGracePeriodSeconds`, Docker: `--stop-timeout`). `crates/tuita-server/tests/http.rs` proves the clean exit.

## Deploy

```sh
docker build -t tuita .
docker run --rm --network host -v tuita-data:/data -e DATABASE_URL=postgres://tuita@localhost/tuita tuita
```

- Stage 1 `rust:1.99-slim` installs `topcoat-cli` 0.12.1, runs `SQLX_OFFLINE=true cargo build --release --locked -p tuita` and `topcoat asset bundle --release --package tuita` in one `RUN`, so `Cargo.lock` and `.sqlx/` must be committed.
- Runtime `gcr.io/distroless/cc-debian12:nonroot`: the binary plus the asset bundle in `/usr/local/bin/assets` (beside the binary, where `next_to_binary()` looks). No shell, runs as non-root, listens on 8080, `LOG_FORMAT=json`. With the default `STORAGE=local`, uploads go to `DATA_DIR=/data`, a `VOLUME` owned by `nonroot`; mount a named volume there or media is lost with the container.
- Object storage: with `STORAGE=s3` the container needs no volume for media (nothing is written under `DATA_DIR/media`). Set `S3_BUCKET`, `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY` and, for a non-AWS service, `S3_ENDPOINT` (and `S3_REGION` where the service wants one, `auto` for R2). The bucket stays private: tuita streams every file through `GET /media/...`, so the CSP's `'self'` is enough and no public bucket policy or CDN URL is involved. The key pair needs put, get, head and delete on the keys under `S3_PREFIX`; the server never lists the bucket. Startup writes and deletes one marker object (`.tuita-startup-check`) and refuses to start with `invalid S3_BUCKET: ...`, `invalid S3_ACCESS_KEY_ID: ...` or `invalid S3_ENDPOINT: ...` when the bucket is missing, the key is refused or the endpoint does not answer. A video upload streams to a temp file under the OS temp dir (`TMPDIR`, `/tmp` in the image) before it uploads, so that directory must hold one `MAX_VIDEO_MB` file per concurrent video upload. Existing local files are not migrated: copy `DATA_DIR/media/*` into the bucket under `S3_PREFIX` (`aws s3 sync`, `mc mirror`, `rclone`) before switching, or the old `media` rows answer 404. `scripts/s3-smoke.sh` proves the whole path against MinIO in podman.
- The `media` row's `path` (`{id}.{ext}`) is the object key, so the same bucket layout works for both backends. Orphaned uploads are deleted by `cleanup` through the same store.
- The image has no Docker `HEALTHCHECK`: distroless has no shell or HTTP client. Point the orchestrator's probes at `/healthz` (liveness) and `/readyz` (readiness).
- Bump the `rust:` tag together with `rust-toolchain.toml`.

## Add a crate

1. `cargo new --lib crates/<name>` (or `--bin`).
2. Replace the generated `[package]` fields with the shared ones and opt into the lints:

   ```toml
   [package]
   name = "<name>"
   version.workspace = true
   edition.workspace = true
   rust-version.workspace = true
   publish.workspace = true
   license.workspace = true

   [lints]
   workspace = true
   ```

3. Add `<name> = { path = "crates/<name>" }` to `[workspace.dependencies]` in the root `Cargo.toml` and use `<name>.workspace = true` in the crates that need it.
4. Update the crate graph above.

`members = ["crates/*"]` picks the new crate up automatically.

## Dependency policy

- Declare every external dependency once, in the root `[workspace.dependencies]`, with its version and features. Members write `name.workspace = true`.
- A member may add features on top (`name = { workspace = true, features = [..] }`) but never pins its own version.
- Add a dependency only when a few lines of std cannot do the job. `cargo deny check` (part of `check`) gates licences, advisories and sources; see `deny.toml`.
- Binary tests use `std::process::Command` with `env!("CARGO_BIN_EXE_<bin>")`, not extra crates.

## Conventions

- Lints: `[workspace.lints]` in the root manifest, inherited with `[lints] workspace = true`. Details in `docs/rust-style.md`. `exhaustive_enums` and `exhaustive_structs` are allowed because the crates are application code.
- Errors: libraries expose typed errors (`thiserror`); the binary adds context with `anyhow::Context` and returns `anyhow::Result` from `main`.
- Logging: `tracing`; the subscriber is installed only by the binary through `tuita_server::init_tracing`, writes to stderr and reads `RUST_LOG`. Libraries emit events, never configure output.
- Panics: no `unwrap()` outside tests. Use `expect("why it cannot fail")` for real invariants.
- Release profile: fat LTO, one codegen unit, stripped, `panic = "abort"`.
- Tests: unit tests in a `mod test` in the same file; integration tests in `crates/<name>/tests/`.

## Workflow

Rules and commands: `AGENTS.md`. Gate: `check`.

## CI

`.github/workflows/check.yml` runs `check` (the same `scripts/check.sh` a developer runs locally, via `devenv up -d` then `devenv shell -- check`) on every push to `main` and every pull request, so a regression is caught before merge instead of whenever someone next remembers to run it by hand. It installs Nix and devenv rather than hand-rolling the toolchain, so CI and a local run diverge only in how the environment is provisioned, never in what is checked.

`.github/workflows/e2e-nightly.yml` runs the browser suite (`scripts/e2e.sh`) once a day and on manual dispatch, in the same devenv shell, since it is release-profile and drives a real Chromium so it is too slow to run on every push.
