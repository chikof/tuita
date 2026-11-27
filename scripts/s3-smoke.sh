#!/usr/bin/env bash
# S3 storage proof: tuita with STORAGE=s3 against a real MinIO server in a container.
#
# Run by hand, never from `check` (inside `devenv shell`, Postgres up):   scripts/s3-smoke.sh
# Exit 0 only when every assertion passed. One PASS/FAIL line per assertion.
#
# What it does
#   1. Starts MinIO in a container (name tuita-s3-smoke-minio, S3 API on 127.0.0.1:$S3_SMOKE_S3_PORT).
#   2. Proves the startup check: `tuita serve` refuses a missing bucket, a wrong secret and an
#      unreachable endpoint, each with `invalid <VARIABLE>: ...` naming the variable to fix.
#   3. Creates the bucket and starts `tuita serve` with STORAGE=s3 and S3_PREFIX=uploads on a
#      throwaway database (tuita_s3_smoke), DATA_DIR and TMPDIR under a temp directory.
#   4. Signs up, uploads a PNG and a 12 MiB MP4 (large enough for a multipart upload) and fetches
#      them through /media: 200, 206 with Range (first bytes and a suffix), HEAD, a 416.
#   5. Lists the bucket with SigV4 curl: the objects are there under the prefix, with the right
#      size. Nothing was written under DATA_DIR/media or left in the temp dir.
#   6. Ages the uploads, restarts the server (the cleanup worker runs at start) and checks that
#      cleanup removed both objects from the bucket and the files answer 404.
#
# Environment
#   S3_SMOKE_IMAGE    MinIO image (default: a pinned community build of MinIO RELEASE.2025-10-15; the
#                     official minio/minio images are no longer published to docker.io or quay.io)
#   S3_SMOKE_S3_PORT  host port of the S3 API (default 9100)
#   S3_SMOKE_PORT     host port of tuita (default 8083)
#   S3_SMOKE_BIN      the tuita binary (default: `cargo build -p tuita`, target/debug/tuita)
#   S3_SMOKE_KEEP     1 leaves the container, server and temp directory in place
#   DOCKER_FLAGS      replaces the default flags placed after `docker` in every call. With podman
#                     the default is `--root $TUITA_CONTAINERS/root --runroot
#                     $TUITA_CONTAINERS/run`; TUITA_CONTAINERS defaults to
#                     $HOME/Projects/tuita-containers (a big disk: / is a small tmpfs here).
#
# Disk: the container and its data are removed on exit; the image stays for reruns (~170 MB under
# $TUITA_CONTAINERS).
set -euo pipefail

cd "$(dirname "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")")"

: "${DATABASE_URL:?DATABASE_URL is not set; run inside devenv shell}"
IMAGE=${S3_SMOKE_IMAGE:-ghcr.io/coollabsio/minio@sha256:4f75fd76598afa23919555d1363e1fb13c632d9bcd4ce8edcf21d3cca2ed0579}
S3_PORT=${S3_SMOKE_S3_PORT:-9100}
APP_PORT=${S3_SMOKE_PORT:-8083}
CONTAINER=tuita-s3-smoke-minio
BUCKET=tuita-media-smoke
PREFIX=uploads
REGION=us-east-1
ACCESS_KEY=smokeaccesskey
SECRET_KEY=$(openssl rand -hex 16)
S3_URL=http://127.0.0.1:$S3_PORT
BASE=http://127.0.0.1:$APP_PORT
SMOKE_DB=tuita_s3_smoke
SMOKE_DB_URL=${DATABASE_URL%/*}/$SMOKE_DB
PASSWORD='correct horse battery 9!'
VIDEO_BYTES=$((12 * 1024 * 1024))

if [[ -z ${DOCKER_FLAGS+x} ]]; then
  DOCKER_FLAGS=
  if docker version 2>/dev/null | grep -qi podman; then
    store=${TUITA_CONTAINERS:-$HOME/Projects/tuita-containers}
    mkdir -p "$store/root" "$store/run" "$store/tmp"
    DOCKER_FLAGS="--root $store/root --runroot $store/run"
    export TMPDIR=$store/tmp
  fi
fi

docker() {
  # Word splitting of DOCKER_FLAGS is the point.
  # shellcheck disable=SC2086
  command docker $DOCKER_FLAGS "$@"
}

for tool in docker curl jq psql openssl base64 cmp; do
  command -v "$tool" >/dev/null || { echo "s3-smoke: $tool not found" >&2; exit 2; }
done

PASSED=0
FAILED=0
pass() { PASSED=$((PASSED + 1)); echo "PASS  $1"; }
fail() { FAILED=$((FAILED + 1)); echo "FAIL  $1"; }
step() { echo "--  $1"; }
# check DESCRIPTION CMD...: PASS when the command succeeds.
check() {
  local description=$1
  shift
  if "$@"; then pass "$description"; else fail "$description"; fi
}

TMP=$(mktemp -d)
mkdir -p "$TMP/data" "$TMP/tmp"
SERVER_LOG=$TMP/server.log
SERVER=

server_stop() {
  if [[ -n $SERVER ]]; then
    kill -TERM "$SERVER" 2>/dev/null || true
    wait "$SERVER" 2>/dev/null || true
    SERVER=
  fi
}

finish() {
  local rc=$?
  trap - EXIT
  if ((rc != 0 || FAILED != 0)); then
    echo "== server log (last 40 lines)" >&2
    tail -n 40 "$SERVER_LOG" >&2 2>/dev/null || true
  fi
  if [[ ${S3_SMOKE_KEEP:-0} != 1 ]]; then
    server_stop
    docker rm -fv "$CONTAINER" >/dev/null 2>&1 || true
    psql -q "$DATABASE_URL" -c "DROP DATABASE IF EXISTS $SMOKE_DB WITH (FORCE)" >/dev/null 2>&1 || true
    rm -rf "$TMP"
  fi
  echo "== $PASSED passed, $FAILED failed"
  if ((rc == 0 && FAILED != 0)); then rc=1; fi
  exit "$rc"
}
trap finish EXIT

# s3 CURL_ARGS...: a SigV4-signed request to the bucket server.
s3() {
  curl -sS --max-time 60 --aws-sigv4 "aws:amz:$REGION:s3" --user "$ACCESS_KEY:$SECRET_KEY" "$@"
}

# bucket_keys: every object key in the bucket, one per line.
bucket_keys() {
  s3 "$S3_URL/$BUCKET?list-type=2" | grep -o '<Key>[^<]*</Key>' | sed -e 's/<Key>//' -e 's/<\/Key>//' || true
}

# serve_env: the environment of a `tuita serve` run, as arguments for env(1).
serve_env() {
  printf '%s\n' \
    "DATABASE_URL=$SMOKE_DB_URL" "PUBLIC_URL=$BASE" "BIND_ADDR=127.0.0.1:$APP_PORT" \
    "DATA_DIR=$TMP/data" "TMPDIR=$TMP/tmp" "LOG_FORMAT=json" \
    "SOURCE_URL=https://example.invalid/tuita" "STORAGE=s3" "S3_BUCKET=$BUCKET" \
    "S3_REGION=$REGION" "S3_ENDPOINT=$S3_URL" "S3_ACCESS_KEY_ID=$ACCESS_KEY" \
    "S3_SECRET_ACCESS_KEY=$SECRET_KEY" "S3_PREFIX=$PREFIX"
}

# serve_with VAR=VALUE...: runs `tuita serve` in the foreground with those variables replaced.
serve_with() {
  local -a vars
  mapfile -t vars < <(serve_env)
  env "${vars[@]}" "$@" timeout 90 "$BIN" serve
}

start_server() {
  local -a vars
  mapfile -t vars < <(serve_env)
  env "${vars[@]}" "$BIN" serve >>"$SERVER_LOG" 2>&1 &
  SERVER=$!
  for _ in $(seq 60); do
    curl -fs "$BASE/readyz" >/dev/null && return 0
    kill -0 "$SERVER" 2>/dev/null || { SERVER=; return 1; }
    sleep 0.5
  done
  return 1
}

# --- 0 build ------------------------------------------------------------------------------------

step "0 build"
if [[ -n ${S3_SMOKE_BIN:-} ]]; then
  BIN=$S3_SMOKE_BIN
else
  cargo build -p tuita
  BIN=${CARGO_TARGET_DIR:-target}/debug/tuita
fi
[[ -x $BIN ]] || { echo "s3-smoke: $BIN is not an executable" >&2; exit 2; }
psql -q "$DATABASE_URL" \
  -c "DROP DATABASE IF EXISTS $SMOKE_DB WITH (FORCE)" \
  -c "CREATE DATABASE $SMOKE_DB"
docker rm -fv "$CONTAINER" >/dev/null 2>&1 || true

# --- 1 MinIO ------------------------------------------------------------------------------------

step "1 start MinIO ($IMAGE)"
docker run -d --name "$CONTAINER" -p "127.0.0.1:$S3_PORT:9000" \
  -e MINIO_ROOT_USER="$ACCESS_KEY" -e MINIO_ROOT_PASSWORD="$SECRET_KEY" \
  "$IMAGE" server /data >/dev/null
for _ in $(seq 60); do
  curl -fs "$S3_URL/minio/health/live" >/dev/null && break
  sleep 0.5
done
curl -fs "$S3_URL/minio/health/live" >/dev/null || { echo "s3-smoke: MinIO did not come up" >&2; docker logs "$CONTAINER" >&2; exit 1; }
pass "MinIO answers on $S3_URL"

# --- 2 startup check ----------------------------------------------------------------------------

step "2 startup refuses bad settings"
refusal() { # refusal VARIABLE DESCRIPTION VAR=VALUE...
  local var=$1 description=$2 out status=0
  shift 2
  out=$(serve_with "$@" 2>&1) || status=$?
  if ((status != 0)) && grep -q "invalid $var:" <<<"$out"; then
    pass "$description: exit $status, \"$(grep -o "invalid $var:[^\"]*" <<<"$out" | head -n 1 | cut -c1-110)\""
  else
    fail "$description: exit $status, output: $(tail -n 3 <<<"$out" | cut -c1-200)"
  fi
}
refusal S3_BUCKET "a missing bucket is refused" "S3_BUCKET=$BUCKET"
s3 -X PUT "$S3_URL/$BUCKET" -o /dev/null -w '%{http_code}\n' | grep -qx 200 || fail "bucket $BUCKET was not created"
pass "bucket $BUCKET created"
refusal S3_ACCESS_KEY_ID "a wrong secret is refused" "S3_SECRET_ACCESS_KEY=not-the-secret"
refusal S3_ENDPOINT "an unreachable endpoint is refused" "S3_ENDPOINT=http://127.0.0.1:1"

# --- 3 serve ------------------------------------------------------------------------------------

step "3 serve with STORAGE=s3"
start_server || { fail "tuita serve did not become ready"; exit 1; }
pass "tuita serve is ready with STORAGE=s3 (prefix $PREFIX)"

jar=$TMP/jar
handle=smoke$(openssl rand -hex 3)
code=$(curl -sS -o "$TMP/signup.json" -w '%{http_code}' -c "$jar" -X POST "$BASE/api/v1/auth/signup" \
  -H "Origin: $BASE" -H 'Content-Type: application/json' \
  --data "$(jq -nc --arg handle "$handle" --arg pw "$PASSWORD" \
    '{handle: $handle, displayName: "Smoke Test", password: $pw, acceptTerms: true, add: false}')")
[[ $code == 200 || $code == 201 ]] || { fail "sign-up is $code"; exit 1; }
pass "signed up $handle"

# --- 4 upload and fetch -------------------------------------------------------------------------

step "4 upload and fetch"
# A 1x1 PNG. The base64 is split so a secret scanner does not mistake it for a key.
png=iVBORw0KGgo
png+=AAAANSUhEUgAA
png+=AAEAAAABCAYA
png+=AAAfFcSJAAAA
png+=DUlEQVR42mNk
png+=YPhfDwAChwGA
png+=60e6kgAAAABJ
png+=RU5ErkJggg==
printf '%s' "$png" | base64 -d >"$TMP/pixel.png"
{ printf '\0\0\0\030ftypmp42\0\0\0\0mp42isom'; head -c "$((VIDEO_BYTES - 24))" /dev/urandom; } >"$TMP/clip.mp4"
video_size=$(stat -c %s "$TMP/clip.mp4")

upload() { # upload FILE TYPE OUT
  curl -sS -o "$3" -w '%{http_code}' -b "$jar" -X POST "$BASE/api/v1/media" -H "Origin: $BASE" \
    -F "file=@$1;type=$2" -F 'alt=smoke'
}
[[ $(upload "$TMP/pixel.png" image/png "$TMP/image.json") == 200 ]] || { fail "the image upload is not 200"; exit 1; }
[[ $(upload "$TMP/clip.mp4" video/mp4 "$TMP/video.json") == 200 ]] || { fail "the video upload is not 200"; exit 1; }
image_url=$(jq -r .url "$TMP/image.json")
video_url=$(jq -r .url "$TMP/video.json")
image_key=${image_url#/media/}
video_key=${video_url#/media/}
pass "uploaded $image_url and $video_url ($video_size bytes)"

fetch() { # fetch URL [curl args...]: headers in $TMP/h, body in $TMP/b, prints the status
  local url=$1
  shift
  curl -sS -o "$TMP/b" -D "$TMP/h" -w '%{http_code}' "$@" "$BASE$url"
}
header() { grep -i "^$1:" "$TMP/h" | tr -d '\r' | cut -d' ' -f2-; }

# fetch_is STATUS URL [curl args...]: fetches and succeeds when the answer has that status.
fetch_is() {
  local wanted=$1
  shift
  [[ $(fetch "$@") == "$wanted" ]]
}
header_is() { [[ $(header "$1") == "$2" ]]; }

if fetch_is 200 "$image_url" && header_is content-type image/webp; then
  pass "GET $image_url: 200 image/webp"
else
  fail "GET $image_url is not 200 image/webp"
fi
if fetch_is 200 "$video_url" && cmp -s "$TMP/b" "$TMP/clip.mp4"; then
  pass "GET $video_url: 200, $video_size bytes identical to the upload"
else
  fail "GET $video_url is not 200 with the uploaded bytes"
fi
if header_is cache-control 'public, max-age=31536000, immutable' && header_is content-disposition inline; then
  pass "cache-control immutable, content-disposition inline"
else
  fail "cache or disposition headers differ"
fi
if fetch_is 206 "$video_url" -H 'Range: bytes=0-99' && header_is content-range "bytes 0-99/$video_size" \
  && cmp -s "$TMP/b" <(head -c 100 "$TMP/clip.mp4"); then
  pass "Range bytes=0-99: 206, content-range bytes 0-99/$video_size, first 100 bytes"
else
  fail "Range bytes=0-99 is not a correct 206"
fi
if fetch_is 206 "$video_url" -H 'Range: bytes=-1000' && cmp -s "$TMP/b" <(tail -c 1000 "$TMP/clip.mp4"); then
  pass "Range bytes=-1000: 206, last 1000 bytes"
else
  fail "Range bytes=-1000 is not a correct 206"
fi
if fetch_is 416 "$video_url" -H "Range: bytes=$video_size-" && header_is content-range "bytes */$video_size"; then
  pass "Range past the end: 416, content-range bytes */$video_size"
else
  fail "Range past the end is not a 416"
fi
if fetch_is 200 "$video_url" -I && header_is content-length "$video_size"; then
  pass "HEAD $video_url: 200, content-length $video_size"
else
  fail "HEAD $video_url is not 200 with the length"
fi

# --- 5 bucket contents --------------------------------------------------------------------------

step "5 the objects are in the bucket"
keys=$(bucket_keys | sort)
expected=$(printf '%s\n' "$PREFIX/$image_key" "$PREFIX/$video_key" | sort)
check "the bucket holds exactly $PREFIX/$image_key and $PREFIX/$video_key" test "$keys" = "$expected"
bucket_size=$(s3 -I "$S3_URL/$BUCKET/$PREFIX/$video_key" | tr -d '\r' | grep -i '^content-length:' | cut -d' ' -f2)
check "the video object is $video_size bytes in the bucket" test "${bucket_size:-}" = "$video_size"
check "nothing under DATA_DIR/media" test -z "$(find "$TMP/data" -type f 2>/dev/null)"
check "no upload temp file left in TMPDIR" test -z "$(find "$TMP/tmp" -type f 2>/dev/null)"

# --- 6 cleanup deletes the objects --------------------------------------------------------------

step "6 cleanup removes the objects"
server_stop
psql -q "$SMOKE_DB_URL" -c "update media set created_at = now() - interval '25 hours'"
start_server || { fail "tuita serve did not restart"; exit 1; }
for _ in $(seq 40); do
  [[ -z $(bucket_keys) ]] && break
  sleep 0.5
done
check "cleanup emptied the bucket" test -z "$(bucket_keys)"
check "the media rows are gone" test "$(psql -qAt "$SMOKE_DB_URL" -c 'select count(*) from media')" = 0
if fetch_is 404 "$image_url" && fetch_is 404 "$video_url"; then
  pass "both files answer 404"
else
  fail "a deleted file is still served"
fi
check "the server log has no ERROR line" test "$(jq -rR 'fromjson? | select(.level=="ERROR") | .level' "$SERVER_LOG" | wc -l)" = 0

if ((FAILED != 0)); then exit 1; fi
