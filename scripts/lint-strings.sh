#!/usr/bin/env bash
# Flags user-visible text written as a literal inside tuita-web markup. Text goes through `t!(key)`.
# Usage: scripts/lint-strings.sh [root]   (default: crates/tuita-web/src)
# Only the bodies of `view! { .. }` and `attributes! { .. }` are scanned, and a file is scanned
# up to its first `#[cfg(test)]`. Skipped: src/design/ and src/app/design.rs.
# A line ending in `// lint-strings: allow` is skipped.
set -euo pipefail

cd "$(dirname "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")")"

root=${1:-crates/tuita-web/src}
[[ -d $root ]] || {
  echo "lint-strings: no such directory: $root" >&2
  exit 2
}

# Three or more letters, written out because POSIX awk has no interval expressions everywhere.
letters='[A-Za-z][A-Za-z][A-Za-z]'
# shellcheck disable=SC2016 # the awk program is single-quoted on purpose

hits=$(
  find "$root" -name '*.rs' -not -path '*/design/*' -not -path '*/app/design.rs' -print0 \
    | sort -z \
    | xargs -0 -r awk -v letters="$letters" '
      FNR == 1 { depth = 0; open = 0; done = 0 }
      done { next }
      /^[[:space:]]*#\[cfg\(test\)\]/ { done = 1; next }
      {
        line = $0
        if (!open && line ~ /(view|attributes)![[:space:]]*\{/) {
          open = 1
          depth = 0
          line = substr(line, index(line, "!") )
        }
        if (!open) next
        check = $0
        if (check !~ /\/\/ lint-strings: allow[[:space:]]*$/) {
          if (check ~ ("^[[:space:]]*\"[^\"]*" letters "[^\"]*\"[[:space:]]*$") \
            || check ~ ("(^|[^=-])>[[:space:]]*\"[^\"]*" letters) \
            || check ~ ("(aria-label|title|placeholder|alt)=\"[^\"]*" letters)) {
            print FILENAME ":" FNR ": literal text in markup"
          }
        }
        n = gsub(/\{/, "{", line)
        m = gsub(/\}/, "}", line)
        depth += n - m
        if (depth <= 0) open = 0
      }
    '
)

if [[ -n $hits ]]; then
  echo "$hits"
  echo "lint-strings: $(echo "$hits" | wc -l) literal(s); use t!(key), or end the line with '// lint-strings: allow'" >&2
  exit 1
fi
