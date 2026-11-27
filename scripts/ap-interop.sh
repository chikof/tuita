#!/usr/bin/env bash
# ActivityPub interop check: tuita against GoToSocial, both ways, over plain http.
#
# Run by hand, never from `check`:   scripts/ap-interop.sh
# Exit 0 only when every assertion passed. One PASS/FAIL line per assertion.
#
# What it does
#   1. Starts, on one docker network, containers (names tuita-interop-*):
#        pg      postgres:17-alpine, a throwaway database for tuita
#        tuita   the image `tuita-interop`, built from this checkout if missing
#                (alias tuita.test, PUBLIC_URL=http://tuita.test, port 80)
#        gts     superseriousbusiness/gotosocial:latest
#                (alias gts.test, GTS_HOST=gts.test, GTS_PROTOCOL=http, port 80)
#        curl    a curl-only helper. Every request runs through it with `docker exec`,
#                so gts.test and tuita.test resolve without touching /etc/hosts.
#   2. Creates one user on each side (GoToSocial through its admin CLI, an OAuth token
#      through the authorization-code flow, tuita through /auth/signup).
#   3. Drives follow, post, reply and like in both directions through the two public APIs and
#      polls (up to INTEROP_WAIT seconds each) until the effect shows on the other side.
#
# GoToSocial settings that make http federation work
#   GTS_PROTOCOL=http                        local ids and links use http://
#   GTS_HTTP_CLIENT_INSECURE_OUTGOING=true   WebFinger, host-meta and nodeinfo over http
#   GTS_HTTP_CLIENT_ALLOW_IPS=<private nets> docker network addresses are not dialable by default
#   GTS_TRUSTED_PROXIES=0.0.0.0/0            irrelevant to federation; silences client-IP warnings
# tuita side: FEDERATION_ALLOW_HTTP=1, FEDERATION_ALLOW_PRIVATE_NETS=1, PUBLIC_URL=http://tuita.test.
# Both servers listen on port 80 (sysctl net.ipv4.ip_unprivileged_port_start=0, as both images run
# unprivileged), so neither PUBLIC_URL nor GTS_HOST carries a port.
#
# Environment
#   INTEROP_WAIT      seconds to wait for each effect (default 30)
#   INTEROP_REBUILD   1 rebuilds the tuita image even when it exists
#   INTEROP_IMAGE     tuita image to run (default tuita-interop, built from this checkout when
#                     missing). Point it at a fixed image to test a fix without touching the tree.
#   INTEROP_KEEP      1 leaves containers and network running afterwards, for inspection
#                     (a later run removes them; or `docker rm -f` the tuita-interop-* ones)
#   DOCKER_FLAGS      replaces the default flags placed after `docker` in every call. With
#                     podman the default is `--root $TUITA_CONTAINERS/root --runroot
#                     $TUITA_CONTAINERS/run`; TUITA_CONTAINERS defaults to
#                     $HOME/Projects/tuita-containers. On this workstation / is a small tmpfs and
#                     podman's own storage would fill it, so keep that directory on a big disk.
#
# Disk: containers (with their volumes), the network and temp files are removed on exit. Images
# stay so reruns are fast (tuita ~40 MB, GoToSocial ~110 MB, postgres ~300 MB, curl ~25 MB, plus
# the rust and node build layers, ~2.8 GB in all). With podman they live under the storage root
# $TUITA_CONTAINERS (default $HOME/Projects/tuita-containers: root/ = image and container
# storage, run/ = runtime state, tmp/ = build scratch). To free the space:
#   docker --root "$HOME/Projects/tuita-containers/root" \
#     --runroot "$HOME/Projects/tuita-containers/run" image prune -a -f
# (with a plain docker daemon: `docker image prune -a -f`; `rm -rf` the directory to drop it all).
#
# If GoToSocial refuses http federation, the plan (§ Assumptions) puts both servers behind Caddy
# with `tls internal`, and falls back to the Mastodon sample fixtures plus the two-instance test.
set -euo pipefail

cd "$(dirname "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")")"

WAIT=${INTEROP_WAIT:-30}
NET=tuita-interop
PREFIX=tuita-interop
PG=$PREFIX-pg
TUITA=$PREFIX-tuita
GTS=$PREFIX-gts
HELPER=$PREFIX-curl
TUITA_IMAGE=${INTEROP_IMAGE:-tuita-interop}
GTS_IMAGE=docker.io/superseriousbusiness/gotosocial:latest
PG_IMAGE=docker.io/library/postgres:17-alpine
CURL_IMAGE=docker.io/curlimages/curl:latest
TUITA_ORIGIN=http://tuita.test
GTS_ORIGIN=http://gts.test
PASSWORD='correct horse battery 9!'
RUN=$(date +%s)

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

for tool in docker jq; do
  command -v "$tool" >/dev/null || { echo "ap-interop: $tool not found" >&2; exit 2; }
done

PASSED=0
FAILED=0
pass() { PASSED=$((PASSED + 1)); echo "PASS  $1"; }
fail() { FAILED=$((FAILED + 1)); echo "FAIL  $1"; }
step() { echo "--  $1"; }

remove_leftovers() {
  docker rm -fv "$HELPER" "$GTS" "$TUITA" "$PG" >/dev/null 2>&1 || true
  docker network rm "$NET" >/dev/null 2>&1 || true
}

show_logs() {
  echo "== tuita WARN/ERROR lines and federation debug lines" >&2
  docker logs "$TUITA" 2>&1 | jq -rR 'fromjson? | select(.level=="WARN" or .level=="ERROR" or (.target | startswith("tuita_server::federation"))) | [.timestamp, .level, .target, .fields.message, (.fields | del(.message) | tostring)] | join(" ")' 2>&1 | cut -c1-400 | tail -n 40 >&2 || true
  echo "== gotosocial WARN/ERROR lines and 4xx/5xx answers" >&2
  docker logs "$GTS" 2>&1 | grep -E 'level=(WARN|ERROR)|statusCode=[45]|msg="[45][0-9][0-9] ' | grep -v 'http-client-insecure-outgoing\|migration\|protocol was set' | cut -c1-400 | tail -n 40 >&2 || true
}

finish() {
  local rc=$?
  trap - EXIT
  if ((rc != 0 || FAILED != 0)); then
    show_logs
  fi
  if [[ ${INTEROP_KEEP:-0} != 1 ]]; then
    remove_leftovers
  fi
  echo "== $PASSED passed, $FAILED failed"
  if ((rc == 0 && FAILED != 0)); then rc=1; fi
  exit "$rc"
}
trap finish EXIT

# --- helpers that talk to the containers ------------------------------------------------------

# curl from inside the network.
hc() { docker exec "$HELPER" curl -sSg --max-time 25 "$@"; }

# tuita API: t METHOD PATH [JSON]. Cookie jar in the helper; the Origin header passes the CSRF guard.
t() {
  local method=$1 path=$2 body=${3:-}
  if [[ -n $body ]]; then
    hc -b /tmp/tuita.jar -c /tmp/tuita.jar -X "$method" -H "Origin: $TUITA_ORIGIN" \
      -H 'Content-Type: application/json' --data "$body" "$TUITA_ORIGIN/api/v1$path"
  else
    hc -b /tmp/tuita.jar -c /tmp/tuita.jar -X "$method" -H "Origin: $TUITA_ORIGIN" \
      "$TUITA_ORIGIN/api/v1$path"
  fi
}

# GoToSocial API: g METHOD PATH [curl args...]. Needs GTS_TOKEN.
g() {
  local method=$1 path=$2
  shift 2
  hc -X "$method" -H "Authorization: Bearer $GTS_TOKEN" "$@" "$GTS_ORIGIN$path"
}

# poll DESCRIPTION SECONDS COMMAND...: PASS when COMMAND succeeds in time, FAIL otherwise.
poll() {
  local desc=$1 limit=$((SECONDS + $2))
  shift 2
  until "$@"; do
    if ((SECONDS >= limit)); then
      fail "$desc"
      return 0
    fi
    sleep 1
  done
  pass "$desc"
}

wait_http() {
  local url=$1 limit=$((SECONDS + 90))
  until hc -f -o /dev/null "$url" 2>/dev/null; do
    if ((SECONDS >= limit)); then
      echo "ap-interop: $url did not come up" >&2
      return 1
    fi
    sleep 1
  done
}

# --- predicates polled above ---------------------------------------------------------------------

tuita_follows_gts() { [[ $(t GET "/users/guser@gts.test" | jq -r '.relation.following') == true ]]; }
gts_follows_tuita() { [[ $(g GET "/api/v1/accounts/relationships?id[]=$GTS_TUITA_ACCOUNT" | jq -r '.[0].following') == true ]]; }
tuita_sees_follower() { [[ $(t GET "/users/guser@gts.test" | jq -r '.relation.followsYou') == true ]]; }

# Sets FOUND to the id of the tuita post whose text contains $1.
tuita_has_post() {
  FOUND=$(t GET '/timelines/following?limit=40' | jq -r --arg m "$1" '[.items[]? | select(.text | contains($m)) | .id][0] // empty')
  [[ -n $FOUND ]]
}

# Sets FOUND to the id (on GoToSocial) of the home timeline status whose HTML contains $1.
gts_has_post() {
  FOUND=$(g GET '/api/v1/timelines/home?limit=40' | jq -r --arg m "$1" '[.[]? | select(.content | contains($m)) | .id][0] // empty')
  [[ -n $FOUND ]]
}

tuita_thread_has_reply() { t GET "/posts/$1" | jq -e --arg m "$2" '[.replies[]? | select(.text | contains($m))] | length > 0' >/dev/null; }
gts_thread_has_reply() { g GET "/api/v1/statuses/$1/context" | jq -e --arg m "$2" '[.descendants[]? | select(.content | contains($m))] | length > 0' >/dev/null; }
tuita_post_liked() { [[ $(t GET "/posts/$1" | jq -r '.post.counts.likes') -ge 1 ]]; }
gts_post_liked() { [[ $(g GET "/api/v1/statuses/$1" | jq -r '.favourites_count') -ge 1 ]]; }

tuita_post() { t POST /posts "$(jq -nc --arg text "$1" --arg reply "${2:-}" '{text: $text, cw: "", sensitive: false, mediaIds: []} + (if $reply == "" then {} else {replyTo: $reply} end)')"; }
gts_post() {
  local extra=()
  if [[ -n ${2:-} ]]; then extra=(-d "in_reply_to_id=$2"); fi
  g POST /api/v1/statuses --data-urlencode "status=$1" -d visibility=public "${extra[@]}"
}

# --- bring up the stack --------------------------------------------------------------------------

step "remove leftovers of a previous run"
remove_leftovers

step "images"
if [[ ${INTEROP_REBUILD:-0} == 1 ]] || ! docker image inspect "$TUITA_IMAGE" >/dev/null 2>&1; then
  docker build -q -t "$TUITA_IMAGE" .
fi
for image in "$GTS_IMAGE" "$PG_IMAGE" "$CURL_IMAGE"; do
  docker image inspect "$image" >/dev/null 2>&1 || docker pull "$image" >/dev/null
done

step "network and containers"
docker network create "$NET" >/dev/null
docker run -d --name "$HELPER" --network "$NET" --entrypoint sleep "$CURL_IMAGE" infinity >/dev/null
docker run -d --name "$PG" --network "$NET" \
  -e POSTGRES_USER=tuita -e POSTGRES_PASSWORD=tuita -e POSTGRES_DB=tuita "$PG_IMAGE" >/dev/null
until docker exec "$PG" pg_isready -q -U tuita -d tuita 2>/dev/null; do sleep 1; done

docker run -d --name "$TUITA" --network "$NET" --network-alias tuita.test \
  --sysctl net.ipv4.ip_unprivileged_port_start=0 \
  -e DATABASE_URL="postgres://tuita:tuita@$PG/tuita" \
  -e PUBLIC_URL="$TUITA_ORIGIN" -e BIND_ADDR=0.0.0.0:80 \
  -e FEDERATION_ALLOW_HTTP=1 -e FEDERATION_ALLOW_PRIVATE_NETS=1 \
  -e SECURE_COOKIES=false -e SOURCE_URL=https://example.invalid/tuita \
  -e RUST_LOG=info,tuita_server=debug "$TUITA_IMAGE" >/dev/null

docker run -d --name "$GTS" --network "$NET" --network-alias gts.test \
  --sysctl net.ipv4.ip_unprivileged_port_start=0 \
  -e GTS_HOST=gts.test -e GTS_PROTOCOL=http -e GTS_PORT=80 \
  -e GTS_DB_TYPE=sqlite -e GTS_DB_ADDRESS=/gotosocial/storage/sqlite.db \
  -e GTS_LETSENCRYPT_ENABLED=false -e GTS_TRUSTED_PROXIES=0.0.0.0/0 \
  -e GTS_ACCOUNTS_REGISTRATION_OPEN=false -e GTS_LOG_LEVEL=debug \
  -e GTS_HTTP_CLIENT_INSECURE_OUTGOING=true \
  -e GTS_HTTP_CLIENT_ALLOW_IPS=10.0.0.0/8,172.16.0.0/12,192.168.0.0/16 \
  "$GTS_IMAGE" >/dev/null

wait_http "$TUITA_ORIGIN/readyz"
wait_http "$GTS_ORIGIN/api/v1/instance"

# --- users ---------------------------------------------------------------------------------------

step "create gts user guser and tuita user tuser"
docker exec "$GTS" /gotosocial/gotosocial admin account create \
  --username guser --email guser@gts.test --password "$PASSWORD" >/dev/null
docker exec "$GTS" /gotosocial/gotosocial admin account confirm --username guser >/dev/null

app=$(hc -X POST "$GTS_ORIGIN/api/v1/apps" \
  -d client_name=interop -d redirect_uris=urn:ietf:wg:oauth:2.0:oob -d 'scopes=read write follow')
client_id=$(jq -r .client_id <<<"$app")
client_secret=$(jq -r .client_secret <<<"$app")
hc -b /tmp/gts.jar -c /tmp/gts.jar -o /dev/null -G "$GTS_ORIGIN/oauth/authorize" \
  -d "client_id=$client_id" -d redirect_uri=urn:ietf:wg:oauth:2.0:oob -d response_type=code --data-urlencode 'scope=read write follow'
hc -b /tmp/gts.jar -c /tmp/gts.jar -o /dev/null -X POST "$GTS_ORIGIN/auth/sign_in" \
  --data-urlencode username=guser@gts.test --data-urlencode "password=$PASSWORD"
code=$(hc -b /tmp/gts.jar -c /tmp/gts.jar -i -X POST "$GTS_ORIGIN/oauth/authorize" | tr -d '\r' | grep -i '^location:' | sed 's/.*code=//')
GTS_TOKEN=$(hc -X POST "$GTS_ORIGIN/oauth/token" -d "client_id=$client_id" -d "client_secret=$client_secret" \
  -d grant_type=authorization_code -d redirect_uri=urn:ietf:wg:oauth:2.0:oob -d "code=$code" | jq -r .access_token)
# New GoToSocial accounts are locked: a follow would wait for manual approval instead of an Accept.
[[ $(g PATCH /api/v1/accounts/update_credentials -d locked=false | jq -r .locked) == false ]] || { echo "ap-interop: could not unlock the gts account" >&2; exit 1; }

t POST /auth/signup "$(jq -nc --arg pw "$PASSWORD" '{handle: "tuser", displayName: "Tuita User", password: $pw, acceptTerms: true, add: false}')" >/dev/null
[[ $(t GET /me | jq -r .user.user.handle) == tuser ]] || { echo "ap-interop: tuita sign-up failed" >&2; exit 1; }

# --- follows -------------------------------------------------------------------------------------

step "follows"
t POST /users/guser@gts.test/follow >/dev/null
poll "tuita follows gts user (follow accepted)" "$WAIT" tuita_follows_gts

GTS_TUITA_ACCOUNT=$(g GET '/api/v2/search?resolve=true&type=accounts&q=tuser@tuita.test' | jq -r '.accounts[0].id // empty')
if [[ -n $GTS_TUITA_ACCOUNT ]]; then
  pass "gts resolves tuser@tuita.test over WebFinger"
  g POST "/api/v1/accounts/$GTS_TUITA_ACCOUNT/follow" >/dev/null
  poll "gts user follows tuita user" "$WAIT" gts_follows_tuita
  poll "tuita shows gts user as a follower" "$WAIT" tuita_sees_follower
else
  fail "gts resolves tuser@tuita.test over WebFinger"
  fail "gts user follows tuita user"
  fail "tuita shows gts user as a follower"
fi

# --- posts ---------------------------------------------------------------------------------------

step "posts"
T_MARK="from-tuita-$RUN"
G_MARK="from-gts-$RUN"
tuita_post "hello gts $T_MARK" >/dev/null
poll "tuita post reaches the gts home timeline" "$WAIT" gts_has_post "$T_MARK"
GTS_SIDE_OF_TUITA_POST=${FOUND:-}

GTS_POST=$(gts_post "hello tuita $G_MARK" | jq -r '.id // empty')
poll "gts post reaches the tuita following timeline" "$WAIT" tuita_has_post "$G_MARK"
TUITA_SIDE_OF_GTS_POST=${FOUND:-}

# --- replies -------------------------------------------------------------------------------------

step "replies"
TUITA_POST=$(t GET '/users/tuser/posts?limit=5' | jq -r --arg m "$T_MARK" '[.items[]? | select(.text | contains($m)) | .id][0] // empty')
R_TUITA="reply-from-tuita-$RUN"
R_GTS="reply-from-gts-$RUN"

if [[ -n $TUITA_SIDE_OF_GTS_POST ]]; then
  tuita_post "$R_TUITA" "$TUITA_SIDE_OF_GTS_POST" >/dev/null
  poll "tuita reply lands in the gts thread" "$WAIT" gts_thread_has_reply "$GTS_POST" "$R_TUITA"
else
  fail "tuita reply lands in the gts thread (no remote post to reply to)"
fi

if [[ -n $GTS_SIDE_OF_TUITA_POST && -n $TUITA_POST ]]; then
  gts_post "@tuser@tuita.test $R_GTS" "$GTS_SIDE_OF_TUITA_POST" >/dev/null
  poll "gts reply lands in the tuita thread" "$WAIT" tuita_thread_has_reply "$TUITA_POST" "$R_GTS"
else
  fail "gts reply lands in the tuita thread (no remote post to reply to)"
fi

# --- likes ---------------------------------------------------------------------------------------

step "likes"
if [[ -n $TUITA_SIDE_OF_GTS_POST ]]; then
  t POST "/posts/$TUITA_SIDE_OF_GTS_POST/like" >/dev/null
  poll "tuita like raises the gts favourite count" "$WAIT" gts_post_liked "$GTS_POST"
else
  fail "tuita like raises the gts favourite count (no remote post to like)"
fi

if [[ -n $GTS_SIDE_OF_TUITA_POST && -n $TUITA_POST ]]; then
  g POST "/api/v1/statuses/$GTS_SIDE_OF_TUITA_POST/favourite" >/dev/null
  poll "gts favourite raises the tuita like count" "$WAIT" tuita_post_liked "$TUITA_POST"
else
  fail "gts favourite raises the tuita like count (no remote post to like)"
fi

((FAILED == 0))
