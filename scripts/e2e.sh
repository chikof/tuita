#!/usr/bin/env bash
# Browser suite (crates/tuita-e2e) against the real stack: release server and asset bundle, throwaway
# database tuita_e2e, nix Chromium (E2E_CHROME) driven over CDP. Needs Postgres up (`devenv up -d`) and `topcoat`
# on PATH. Extra arguments are libtest filters (`scripts/e2e.sh design::`).
set -euo pipefail

cd "$(dirname "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")")"

# The suite needs a server that builds fast, not the production fat-LTO binary (one link of that takes
# minutes and gigabytes). Override with the same variables to test the real profile.
export CARGO_PROFILE_RELEASE_LTO=${CARGO_PROFILE_RELEASE_LTO:-off}
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=${CARGO_PROFILE_RELEASE_CODEGEN_UNITS:-16}

: "${DATABASE_URL:?DATABASE_URL is not set; run inside devenv shell}"
port=${E2E_PORT:-8081}
# The host must stay localhost: WebAuthn refuses IP-literal origins.
host=${E2E_HOST:-localhost}
base_url="http://$host:$port"
# Parallel runs (one per worktree) set E2E_PORT, E2E_PEER_PORT and E2E_DB_NAME to unique values.
e2e_db=${E2E_DB_NAME:-tuita_e2e}
e2e_db_url="${DATABASE_URL%/*}/$e2e_db"

psql -q "$DATABASE_URL" \
  -c "DROP DATABASE IF EXISTS $e2e_db WITH (FORCE)" \
  -c "CREATE DATABASE $e2e_db"

SQLX_OFFLINE=true cargo build --release --locked -p tuita

# `asset!` bakes absolute source paths into the binary, and a copied target/ or a shared compiler cache can
# reuse a build of another checkout: the bundle would then serve that checkout's scripts and styles.
foreign=$(strings -a "${CARGO_TARGET_DIR:-target}/release/tuita" | grep -oE '/[^ ]*/crates/tuita-web/assets/js/post\.js' | sort -u | grep -vxF "$PWD/crates/tuita-web/assets/js/post.js" || true)
if [[ -n $foreign ]]; then
  echo "e2e: the binary embeds assets of another checkout:" >&2
  echo "$foreign" >&2
  echo "e2e: run 'RUSTC_WRAPPER= cargo clean -p tuita-web -p tuita --release' and retry with RUSTC_WRAPPER= set" >&2
  exit 1
fi
# Writes target/release/assets, which the binary finds next to itself.
SQLX_OFFLINE=true topcoat asset bundle --release --package tuita

target=${CARGO_TARGET_DIR:-target}
log=$target/e2e-server.log
# Uploads belong to the throwaway database; drop them with it.
rm -rf "$target/e2e-data"
# Behind TRUST_PROXY the server reads the client address from X-Forwarded-For; the harness gives every
# browser context its own, so the per-address auth limiter does not see the whole suite as one client.
DATABASE_URL=$e2e_db_url PUBLIC_URL=$base_url BIND_ADDR="127.0.0.1:$port" FEDERATION_ALLOW_PRIVATE_NETS=1 TRUST_PROXY=1 \
  DATA_DIR="$target/e2e-data" SOURCE_URL=https://example.invalid/tuita \
  "$target/release/tuita" serve >"$log" 2>&1 &
server=$!
trap 'kill "$server" 2>/dev/null || true; wait "$server" 2>/dev/null || true' EXIT

for _ in $(seq 60); do
  curl -fs "$base_url/readyz" >/dev/null && break
  kill -0 "$server" 2>/dev/null || { cat "$log" >&2; exit 1; }
  sleep 0.5
done
curl -fs "$base_url/readyz" >/dev/null || {
  echo "e2e: server not ready after 30 s; log: $log" >&2
  exit 1
}
# Tests that need staff accounts run `"$E2E_TUITA_BIN" admin promote <handle>` against E2E_DATABASE_URL
# (the CLI reads DATABASE_URL). The suite target has `test = false`, so only `--test e2e` runs it.
E2E_BASE_URL=$base_url E2E_DATABASE_URL=$e2e_db_url E2E_TUITA_BIN=$(readlink -f "$target/release/tuita") \
  cargo test --release -p tuita-e2e --test e2e -- "$@"
