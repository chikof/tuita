---
name: service-ops
description: Load when running, configuring, probing, smoke-testing, containerizing or changing shutdown, health or middleware behaviour of this HTTP service.
---

# Service ops

HTTP service on axum and tokio in `crates/tuita-server`, started by the `tuita` binary. Facts below come from `crates/tuita-server/src/lib.rs`, `Dockerfile` and `docs/architecture.md`; read the latter for layout.

## Run locally

```sh
devenv up -d              # Postgres, plus `tuita serve` and `pnpm -C web dev` as processes
pnpm -C web build         # the server refuses to start without web/build/index.html
cargo run -p tuita        # migrate, then serve on 127.0.0.1:8080
```

Stop with Ctrl-C (SIGINT). Run it in the background or a second terminal while you probe it; do not block your session on it.

## Configuration

All settings are environment variables, read once by `Config::from_env()`. A bad or missing value stops startup with `invalid <VAR>: ...` naming the variable. `.env` is loaded by devenv; copy `.env.example` to `.env`. The full table is in `docs/architecture.md` § Configuration. The ones you touch most:

| Variable | Default | Meaning |
|---|---|---|
| `DATABASE_URL` | required | devenv sets `postgres://tuita@localhost/tuita` |
| `BIND_ADDR` | `127.0.0.1:8080` | listen address; the container image sets `0.0.0.0:8080` |
| `WEB_DIR` | `./web/build` | built SPA; the image sets `/srv/web` |
| `STORAGE` | `local` | `local` or `s3`: where media files live, see [Media storage](#media-storage) |
| `PUBLIC_URL` | `http://127.0.0.1:8080` | public origin, used for OG URLs, cookies and federation |
| `LOG_FORMAT` | `pretty` | `pretty` or `json` (image default: `json`) |
| `RUST_LOG` | `info` | tracing env-filter syntax, e.g. `info,tuita_server=debug` |

Add a setting: field on `Config`, a branch in `Config::from_lookup` returning `ConfigError { var, .. }`, a row in `docs/architecture.md`, a case in the `test` module, and `.env.example`.
Test config without touching the process environment: `Config::from_lookup(|name| ...)`.

## Routes

`docs/architecture.md` § Endpoints has the table. Errors: handlers return `Result<_, AppError>`, rendered `{"error":"<message>"}`. `Internal` logs the cause and sends only `internal error`. Timeouts return `408` with an empty body. `503` is only readiness.

Every response carries `x-request-id` (the caller's, else a UUID) and the security headers including the CSP.

## Smoke test

With the server running:

```sh
curl -si localhost:8080/healthz                # 200 {"status":"ok"}
curl -si localhost:8080/readyz                 # 200 {"status":"ready"}
curl -s localhost:8080/@ada | grep -c og:title # 1: the SPA with OG tags
curl -si localhost:8080/api/nope               # 404 {"error":"not found"}
curl -si -H 'x-request-id: abc' localhost:8080/healthz | grep -i x-request-id
devenv processes stop postgres; curl -si localhost:8080/readyz   # 503, then `devenv up -d`
```

Report each command and the status it returned (skill `verify`). A changed route needs a test too: in-process `router(..).oneshot(..)` in `crates/tuita-server/tests/`, no socket needed. `#[sqlx::test(migrations = "./migrations")]` gives each test its own database.

## Shutdown

`shutdown_signal()` completes on SIGINT or SIGTERM. `serve_on` then flips `/readyz` to 503, stops accepting, lets in-flight requests finish (bounded by `REQUEST_TIMEOUT_SECS`), logs `stopped` and returns `Ok(())`.

Check it by hand:

```sh
kill -TERM <pid>                        # logs "shutdown requested, draining" then "stopped"
echo $?                                 # 0 from the process
```

Rules when you change it:
- Keep the order: ready false, stop accepting, drain, return.
- A new background task (worker, consumer) needs its own stop signal and must be awaited inside the drain.
- Orchestrator kill timeout must exceed `REQUEST_TIMEOUT_SECS` (Docker `--stop-timeout`, Kubernetes `terminationGracePeriodSeconds`).
- `crates/tuita-server/tests/http.rs` covers the clean exit; keep that test green.

## Readiness for dependencies

Dependency checks live in `AppState::is_healthy`, called by `readyz`. Never in `healthz`: a dead dependency must not get the process killed.

## Container

```sh
docker build -t tuita .
docker run --rm --network host -e DATABASE_URL=postgres://tuita@localhost/tuita tuita
docker stop --time 35 <container>       # SIGTERM; time above REQUEST_TIMEOUT_SECS
```

- Stage 1 `node:24-slim` builds the SPA. Stage 2 `rust:1.99-slim` builds with `SQLX_OFFLINE=true cargo build --release --locked -p tuita`, so `Cargo.lock` and `.sqlx/` must be committed. Bump the rust tag together with `rust-toolchain.toml`.
- Runtime `gcr.io/distroless/cc-debian12:nonroot`: no shell, no curl, runs as non-root, `ENTRYPOINT` is `tuita serve`, port 8080, SPA in `/srv/web`. Debug from outside with `curl`, not `docker exec`.
- No `HEALTHCHECK` in the image: distroless has no tool to run one. Point orchestrator probes at `/healthz` (liveness) and `/readyz` (readiness).
- The Rust stage uses BuildKit cache mounts for the cargo registry and `target/`, so dependencies are reused between builds and `target/` never becomes an image layer (a layer of it filled the 7.7G tmpfs root here). `docker system prune -f` if the disk fills anyway.

## Media storage

`STORAGE=local` (default) keeps uploads under `DATA_DIR/media`. `STORAGE=s3` keeps them in an S3-compatible bucket (AWS S3, MinIO, R2, B2, Garage): set `S3_BUCKET`, `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY`, and for a non-AWS service `S3_ENDPOINT` (path-style addressing is then on by default; `S3_REGION`, `S3_PATH_STYLE` and `S3_PREFIX` are optional). Both keys are required; the ambient AWS credential chain is not read. Table and rules: `docs/architecture.md` § Configuration and § Deploy.

- The bucket stays private. `GET /media/{id}.{ext}` streams from the store (Range, HEAD, ETag), so the CSP stays `'self'`.
- Startup writes and deletes `.tuita-startup-check` in the bucket and exits with `invalid S3_BUCKET: ...`, `invalid S3_ACCESS_KEY_ID: ...` or `invalid S3_ENDPOINT: ...` when it cannot.
- A video streams to a temp file under `TMPDIR` (OS temp dir) before it uploads; size the temp dir for one `MAX_VIDEO_MB` file per concurrent upload. The distroless image has `/tmp`.
- Nothing is migrated between backends: copy `DATA_DIR/media/*` into the bucket under `S3_PREFIX` before switching.
- Proof: `scripts/s3-smoke.sh` (inside `devenv shell`, Postgres up) runs MinIO in podman, serves with `STORAGE=s3`, uploads an image and a 12 MiB video, checks 200/206/416/HEAD, the bucket's objects, that nothing reached `DATA_DIR/media`, and that cleanup deletes the objects. Images and container storage live under `$TUITA_CONTAINERS` (default `~/Projects/tuita-containers`); the official `minio/minio` images are no longer pullable, so the script pins a community build (`S3_SMOKE_IMAGE` overrides it).

## Logging

Logs go to stderr. `LOG_FORMAT=json` prints one JSON object per line. Filter with `RUST_LOG`. Handlers only emit events (`tracing::info!`, `error!`); `init_tracing`, called by the binary, owns output. Debug recipe: skill `debug`.
