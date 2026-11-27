#!/usr/bin/env bash
# The one gate: `check` (devenv) runs this. Done means this exits 0.
# -e: every step below gates; a failing one (fmt diff, ...) must fail `check`.
set -euo pipefail
trap 'echo "check: failed: $BASH_COMMAND" >&2' ERR

cd "$(dirname "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")")"

cargo fmt --all --check
# Formats only the bodies of Topcoat macros; rustfmt skips them.
topcoat fmt --check

# Compile-time query checks read .sqlx/, never a live database.
export SQLX_OFFLINE=true

# Two independent groups run at once: clippy and doc share Cargo's build graph and target-dir
# lock, so they gain little from running against each other and stay sequential inside their
# group; deny, shellcheck and typos touch neither Cargo's target dir nor Postgres and do not
# depend on clippy/doc's output, so they run as the other group, overlapping the first.
clippy_doc_log=$(mktemp)
lint_log=$(mktemp)
trap 'rm -f "$clippy_doc_log" "$lint_log"' EXIT

(
  cargo clippy --workspace --all-targets --all-features --locked -- -D warnings &&
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features --locked
) >"$clippy_doc_log" 2>&1 &
clippy_doc_pid=$!

(
  cargo deny check &&
    shellcheck scripts/*.sh &&
    typos
) >"$lint_log" 2>&1 &
lint_pid=$!

fail=0
wait "$clippy_doc_pid" || fail=1
cat "$clippy_doc_log"
wait "$lint_pid" || fail=1
cat "$lint_log"
if [[ $fail -ne 0 ]]; then
  echo "check: clippy/doc or deny/shellcheck/typos failed, see output above" >&2
  exit 1
fi

pg_isready -q -d "${DATABASE_URL:-}" || {
  echo "check: Postgres not reachable; run 'devenv up -d'" >&2
  exit 1
}
cargo sqlx prepare --workspace --check -- --all-targets
# tuita-web tests load the bundle from target/debug/assets.
topcoat asset bundle --package tuita
cargo test --workspace --all-features --locked

# A used key missing from a locale, or a `$var` that `t!` does not pass, exits 1.
topcoat l10n check --package tuita
scripts/lint-strings.sh
