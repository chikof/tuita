#!/usr/bin/env bash
# Mastodon client API smoke test: the core flow of a client such as Phanpy, driven with curl against a
# running tuita instance. Run by hand, never from `check`:   scripts/mastodon-smoke.sh <base-url>
#
# <base-url> is the instance's PUBLIC_URL (no path). It is also sent as `Origin`, which the CSRF guard wants
# on the cookie-session calls (sign-up, the consent answer).
# One PASS/FAIL/SKIP line per step. Exit 0 only when every step passed; the first failure stops the run and
# prints the request and the answer to stderr.
#
# Steps
#    1 GET /api/v2/instance advertises configuration.urls.streaming
#    2 two throwaway accounts sign up through /api/v1/auth/signup (cookie jars)
#    3 POST /api/v1/apps, scopes `read write follow`
#    4 PKCE S256 consent: POST /oauth/authorize as the signed-in user answers the out-of-band code
#    5 POST /oauth/token (public client, verifier); the same code again is 400 and revokes the token it made
#    6 a fresh PKCE round, then verify_credentials with the token
#    7 account two does the same flow as a confidential client (client_secret, no PKCE) and follows account one
#    8 account one posts with an Idempotency-Key; a retry returns the same status
#    9 account two reads it in timelines/home; limit=1 with two posts answers a Link header
#   10 account two favourites it; account one has a `favourite` notification
#   11 media: PNG upload through POST /api/v2/media, attached to a status, the file URL serves 200
#   12 streaming: websocat sees an `update` frame (SKIP when websocat is not installed)
#   13 DELETE the status; GET is then 404
#   14 POST /oauth/revoke; the token then gets 401
#   15 a token of a `read`-only app cannot POST /api/v1/statuses (403)
#
# Environment
#   SMOKE_XFF   1 (default) sends a random X-Forwarded-For for the whole run, so a server started with
#               TRUST_PROXY=1 does not count every run against the sign-up and token limits of one address.
#               0 sends none (a server without TRUST_PROXY ignores it anyway).
#   SMOKE_KEEP  1 leaves the two throwaway accounts in place (default: deleted on exit with DELETE /api/v1/me; on an
#               empty database the first account becomes the admin, which tuita refuses to delete).
#
# Needs curl, jq, openssl and base64. websocat (optional) enables step 12.
# shellcheck disable=SC2016 # `$name` in single quotes is a jq variable, not shell
set -euo pipefail

if (($# != 1)); then
  echo "usage: scripts/mastodon-smoke.sh <base-url>   e.g. http://localhost:8082" >&2
  exit 2
fi
BASE=${1%/}
for tool in curl jq openssl base64; do
  command -v "$tool" >/dev/null || { echo "mastodon-smoke: $tool not found" >&2; exit 2; }
done

OOB=urn:ietf:wg:oauth:2.0:oob
PASSWORD='correct horse battery 9!'
SCOPES='read write follow'
TMP=$(mktemp -d)
BODY=$TMP/body
HEADERS=$TMP/headers
STEPS=15
STEP=0
LAST=
CODE=

XFF=()
if [[ ${SMOKE_XFF:-1} == 1 ]]; then
  read -r o1 o2 o3 < <(openssl rand 3 | od -An -tu1)
  XFF=(-H "X-Forwarded-For: 10.$o1.$o2.$o3")
fi

cleanup() {
  local rc=$?
  trap - EXIT
  if [[ ${SMOKE_KEEP:-0} != 1 ]]; then
    local jar status
    for jar in "$TMP/a.jar" "$TMP/b.jar"; do
      [[ -s $jar ]] || continue
      status=$(curl -sS --max-time 15 -o /dev/null -w '%{http_code}' -b "$jar" -X DELETE "${XFF[@]}" -H "Origin: $BASE" \
        -H 'Content-Type: application/json' --data "$(jq -nc --arg pw "$PASSWORD" '{password: $pw}')" \
        "$BASE/api/v1/me" || true)
      [[ $status == 200 ]] || echo "mastodon-smoke: could not delete a throwaway account (HTTP $status)" >&2
    done
  fi
  rm -rf "$TMP"
  exit "$rc"
}
trap cleanup EXIT

pass() { printf 'PASS  %2d/%d  %s\n' "$STEP" "$STEPS" "$1"; }
skip() { printf 'SKIP  %2d/%d  %s\n' "$STEP" "$STEPS" "$1"; }
fail() {
  printf 'FAIL  %2d/%d  %s\n' "$STEP" "$STEPS" "$1"
  {
    echo "  request:  $LAST"
    echo "  answer:   HTTP ${CODE:-none}"
    [[ -s $BODY ]] && head -c 600 "$BODY" | sed 's/^/  body:     /'
  } >&2
  exit 1
}
step() { STEP=$1; }

# call METHOD URL [curl args...]: sets CODE; the answer is in $BODY and $HEADERS.
call() {
  local method=$1 url=$2
  shift 2
  LAST="$method $url"
  : >"$BODY"
  CODE=$(curl -sSg --max-time 30 -o "$BODY" -D "$HEADERS" -w '%{http_code}' -X "$method" "${XFF[@]}" "$@" "$url") ||
    fail "curl could not reach $url"
}

# api TOKEN METHOD PATH [curl args...]: a call with a bearer token.
api() {
  local token=$1 method=$2 path=$3
  shift 3
  call "$method" "$BASE$path" -H "Authorization: Bearer $token" "$@"
}

expect() { [[ $CODE == "$1" ]] || fail "$2"; }
# field FILTER: the jq result of the last answer as a raw string; empty when absent or null.
field() { jq -r "$1 // empty" "$BODY" 2>/dev/null || true; }
# need FILTER MESSAGE: the field, or FAIL when it is empty.
need() {
  local value
  value=$(field "$1")
  [[ -n $value ]] || fail "$2"
  printf '%s' "$value"
}
holds() { jq -e "$@" "$BODY" >/dev/null 2>&1; }

# retry COMMAND...: runs COMMAND up to 10 times, half a second apart.
retry() {
  local _
  for _ in $(seq 10); do
    "$@" && return 0
    sleep 0.5
  done
  return 1
}

# register_app NAME SCOPES: sets APP_ID and APP_SECRET.
register_app() {
  call POST "$BASE/api/v1/apps" --data-urlencode "client_name=$1" -d "redirect_uris=$OOB" --data-urlencode "scopes=$2"
  expect 200 "POST /api/v1/apps is not 200"
  APP_ID=$(need .client_id "the app has no client_id")
  APP_SECRET=$(need .client_secret "the app has no client_secret")
}

# new_pkce: sets VERIFIER and CHALLENGE (S256, base64url without padding).
new_pkce() {
  VERIFIER=$(openssl rand -hex 32)
  CHALLENGE=$(printf '%s' "$VERIFIER" | openssl dgst -sha256 -binary | openssl base64 -A | tr '+/' '-_' | tr -d '=')
}

# authorize JAR CLIENT_ID SCOPE [CHALLENGE]: the consent answer of the signed-in user; sets AUTH_CODE.
authorize() {
  local jar=$1 client=$2 scope=$3 challenge=${4:-} request
  request=$(jq -nc --arg client "$client" --arg oob "$OOB" --arg scope "$scope" --arg challenge "$challenge" \
    '{clientId: $client, redirectUri: $oob, scope: $scope, state: "smoke", responseType: "code", approve: true}
     + (if $challenge == "" then {} else {codeChallenge: $challenge, codeChallengeMethod: "S256"} end)')
  call POST "$BASE/oauth/authorize" -b "$jar" -H "Origin: $BASE" -H 'Content-Type: application/json' --data "$request"
  expect 200 "POST /oauth/authorize is not 200"
  AUTH_CODE=$(need .code "the consent answer has no code")
}

# exchange CLIENT_ID CODE [VERIFIER] [SECRET]: POST /oauth/token with the authorization_code grant.
exchange() {
  local args=(-d grant_type=authorization_code --data-urlencode "client_id=$1" --data-urlencode "code=$2"
    --data-urlencode "redirect_uri=$OOB")
  if [[ -n ${3:-} ]]; then args+=(--data-urlencode "code_verifier=$3"); fi
  if [[ -n ${4:-} ]]; then args+=(--data-urlencode "client_secret=$4"); fi
  call POST "$BASE/oauth/token" "${args[@]}"
}

# issued: the access token of the last /oauth/token answer, checked, into ACCESS_TOKEN.
issued() {
  expect 200 "POST /oauth/token is not 200"
  holds '.token_type == "Bearer"' || fail "the token answer is not token_type Bearer"
  ACCESS_TOKEN=$(need .access_token "the token answer has no access_token")
}

# --- 1 instance --------------------------------------------------------------------------------

step 1
call GET "$BASE/api/v2/instance"
expect 200 "GET /api/v2/instance is not 200"
STREAMING=$(need .configuration.urls.streaming "configuration.urls.streaming is missing")
[[ $STREAMING == ws://* || $STREAMING == wss://* ]] || fail "configuration.urls.streaming is not a websocket URL: $STREAMING"
pass "instance $(field .domain) advertises streaming at $STREAMING"

# --- 2 sign up two accounts ----------------------------------------------------------------------

step 2
signup() { # signup JAR HANDLE
  call POST "$BASE/api/v1/auth/signup" -c "$1" -H "Origin: $BASE" -H 'Content-Type: application/json' \
    --data "$(jq -nc --arg handle "$2" --arg pw "$PASSWORD" \
      '{handle: $handle, displayName: "Smoke Test", password: $pw, acceptTerms: true, add: false}')"
  [[ $CODE == 200 || $CODE == 201 ]] || fail "sign-up of $2 is not 200 or 201"
}
HANDLE_A=smoke$(openssl rand -hex 3)
HANDLE_B=smoke$(openssl rand -hex 3)
signup "$TMP/a.jar" "$HANDLE_A"
signup "$TMP/b.jar" "$HANDLE_B"
pass "signed up $HANDLE_A and $HANDLE_B"

# --- 3 register apps ----------------------------------------------------------------------------

step 3
register_app "smoke pkce" "$SCOPES"
APP_A=$APP_ID
APP_A_SECRET=$APP_SECRET
pass "app registered with client_id and client_secret (scopes: $SCOPES)"

# --- 4 PKCE consent -----------------------------------------------------------------------------

step 4
new_pkce
authorize "$TMP/a.jar" "$APP_A" "$SCOPES" "$CHALLENGE"
pass "consent answered an out-of-band code (PKCE S256, challenge ${CHALLENGE:0:8}...)"

# --- 5 token and replay -------------------------------------------------------------------------

step 5
exchange "$APP_A" "$AUTH_CODE" "$VERIFIER"
issued
FIRST_TOKEN=$ACCESS_TOKEN
exchange "$APP_A" "$AUTH_CODE" "$VERIFIER"
expect 400 "replaying a used code is not 400"
[[ $(field .error) == invalid_grant ]] || fail "a replayed code is not invalid_grant"
api "$FIRST_TOKEN" GET /api/v1/accounts/verify_credentials
expect 401 "the token of a replayed code still works (it must be revoked)"
pass "token issued with the verifier; the replay is 400 invalid_grant and revoked that token"

# --- 6 verify_credentials -----------------------------------------------------------------------

step 6
new_pkce
authorize "$TMP/a.jar" "$APP_A" "$SCOPES" "$CHALLENGE"
exchange "$APP_A" "$AUTH_CODE" "$VERIFIER"
issued
TOKEN_A=$ACCESS_TOKEN
api "$TOKEN_A" GET /api/v1/accounts/verify_credentials
expect 200 "verify_credentials is not 200"
ID_A=$(need .id "verify_credentials has no id")
[[ $(field .username) == "$HANDLE_A" ]] || fail "verify_credentials is not $HANDLE_A"
pass "verify_credentials is @$HANDLE_A ($ID_A)"

# --- 7 second account: confidential client, follow ----------------------------------------------

step 7
register_app "smoke confidential" "$SCOPES"
APP_B=$APP_ID
APP_B_SECRET=$APP_SECRET
authorize "$TMP/b.jar" "$APP_B" "$SCOPES"
exchange "$APP_B" "$AUTH_CODE" "" "$APP_B_SECRET"
issued
TOKEN_B=$ACCESS_TOKEN
api "$TOKEN_B" GET /api/v1/accounts/verify_credentials
expect 200 "verify_credentials of the second account is not 200"
ID_B=$(need .id "verify_credentials has no id")
api "$TOKEN_B" POST "/api/v1/accounts/$ID_A/follow"
expect 200 "follow is not 200"
holds '.following == true' || fail "the relationship does not say following"
pass "@$HANDLE_B (client_secret, no PKCE) follows @$HANDLE_A"

# --- 8 post with an idempotency key -------------------------------------------------------------

step 8
RUN=$(openssl rand -hex 4)
KEY=smoke-$RUN
api "$TOKEN_A" POST /api/v1/statuses -H "Idempotency-Key: $KEY" --data-urlencode "status=smoke first $RUN"
expect 200 "POST /api/v1/statuses is not 200"
STATUS_1=$(need .id "the status has no id")
holds --arg id "$ID_A" '.account.id == $id and .visibility == "public"' || fail "the status is not a public post of account one"
api "$TOKEN_A" POST /api/v1/statuses -H "Idempotency-Key: $KEY" --data-urlencode "status=smoke first $RUN"
expect 200 "the retry with the same Idempotency-Key is not 200"
[[ $(field .id) == "$STATUS_1" ]] || fail "the retry made another status ($(field .id) instead of $STATUS_1)"
api "$TOKEN_A" POST /api/v1/statuses --data-urlencode "status=smoke second $RUN"
expect 200 "the second status is not 200"
STATUS_2=$(need .id "the second status has no id")
[[ $STATUS_2 != "$STATUS_1" ]] || fail "a request without a key returned the first status"
pass "status $STATUS_1 created; the retry with Idempotency-Key returned the same id"

# --- 9 home timeline and Link -------------------------------------------------------------------

step 9
api "$TOKEN_B" GET "/api/v1/timelines/home?limit=1"
expect 200 "timelines/home?limit=1 is not 200"
holds 'length == 1' || fail "timelines/home?limit=1 does not hold exactly one status"
grep -qi '^link:.*rel="next"' "$HEADERS" || fail "timelines/home?limit=1 has no Link rel=\"next\" with two posts"
api "$TOKEN_B" GET /api/v1/timelines/home
expect 200 "timelines/home is not 200"
holds --arg id "$STATUS_1" 'any(.[]; .id == $id)' || fail "timelines/home does not contain status $STATUS_1"
pass "timelines/home of @$HANDLE_B holds the status; limit=1 carries a Link rel=next"

# --- 10 favourite and notification --------------------------------------------------------------

step 10
api "$TOKEN_B" POST "/api/v1/statuses/$STATUS_1/favourite"
expect 200 "favourite is not 200"
holds '.favourited == true' || fail "the status does not say favourited"
favourite_notified() {
  api "$TOKEN_A" GET "/api/v1/notifications?types[]=favourite"
  [[ $CODE == 200 ]] && holds --arg status "$STATUS_1" --arg by "$ID_B" \
    'any(.[]; .type == "favourite" and .status.id == $status and .account.id == $by)'
}
retry favourite_notified || fail "@$HANDLE_A has no favourite notification from @$HANDLE_B"
pass "@$HANDLE_A has a favourite notification from @$HANDLE_B"

# --- 11 media -----------------------------------------------------------------------------------

step 11
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
api "$TOKEN_A" POST /api/v2/media -F "file=@$TMP/pixel.png;type=image/png" -F 'description=a single pixel'
[[ $CODE == 200 || $CODE == 202 ]] || fail "POST /api/v2/media is not 200 or 202"
MEDIA=$(need .id "the upload has no id")
holds '.type == "image"' || fail "the upload is not an image"
api "$TOKEN_A" POST /api/v1/statuses --data-urlencode "status=smoke media $RUN" --data-urlencode "media_ids[]=$MEDIA"
expect 200 "the status with media is not 200"
MEDIA_URL=$(need '.media_attachments[0].url' "the status has no media_attachments[0].url")
call GET "$MEDIA_URL"
expect 200 "the media file $MEDIA_URL is not served with 200"
grep -qi '^content-type: *image/' "$HEADERS" || fail "the media file is not served as an image"
pass "PNG uploaded, attached and served at $MEDIA_URL"

# --- 12 streaming -------------------------------------------------------------------------------

step 12
if command -v websocat >/dev/null; then
  FRAMES=$TMP/frames
  : >"$FRAMES"
  # sleep keeps websocat's stdin open for the 12 s the stream is listened to.
  sleep 12 | websocat "$STREAMING/api/v1/streaming?stream=user&access_token=$TOKEN_B" >"$FRAMES" 2>"$TMP/websocat.err" &
  SOCKET=$!
  sleep 1
  api "$TOKEN_A" POST /api/v1/statuses --data-urlencode "status=smoke stream $RUN"
  expect 200 "the status for the stream is not 200"
  STREAMED=$(need .id "the streamed status has no id")
  seen_update() { jq -s -e --arg id "$STREAMED" 'any(.[]; .event == "update" and (.payload | fromjson | .id == $id))' "$FRAMES" >/dev/null 2>&1; }
  if ! retry seen_update; then
    kill "$SOCKET" 2>/dev/null || true
    {
      echo "  frames:   $(head -c 600 "$FRAMES")"
      echo "  websocat: $(head -c 300 "$TMP/websocat.err")"
    } >&2
    fail "no update frame for status $STREAMED arrived on $STREAMING"
  fi
  kill "$SOCKET" 2>/dev/null || true
  pass "websocat received the update frame of status $STREAMED"
else
  skip "websocat is not installed; the streaming socket was not checked"
fi

# --- 13 delete ----------------------------------------------------------------------------------

step 13
api "$TOKEN_A" DELETE "/api/v1/statuses/$STATUS_1"
expect 200 "DELETE /api/v1/statuses/$STATUS_1 is not 200"
api "$TOKEN_A" GET "/api/v1/statuses/$STATUS_1"
expect 404 "the deleted status is not 404"
pass "status $STATUS_1 deleted and then 404"

# --- 14 revoke ----------------------------------------------------------------------------------

step 14
call POST "$BASE/oauth/revoke" --data-urlencode "client_id=$APP_A" --data-urlencode "client_secret=$APP_A_SECRET" \
  --data-urlencode "token=$TOKEN_A"
expect 200 "POST /oauth/revoke is not 200"
api "$TOKEN_A" GET /api/v1/accounts/verify_credentials
expect 401 "the revoked token is not 401"
pass "token revoked and then 401"

# --- 15 read-only token -------------------------------------------------------------------------

step 15
register_app "smoke read-only" read
authorize "$TMP/b.jar" "$APP_ID" read
exchange "$APP_ID" "$AUTH_CODE" "" "$APP_SECRET"
issued
api "$ACCESS_TOKEN" GET /api/v1/accounts/verify_credentials
expect 200 "the read-only token cannot read verify_credentials"
api "$ACCESS_TOKEN" POST /api/v1/statuses --data-urlencode "status=smoke forbidden $RUN"
expect 403 "a read-only token posting a status is not 403"
pass "the read-only token reads and gets 403 on POST /api/v1/statuses"

echo "== mastodon-smoke: every step passed against $BASE"
