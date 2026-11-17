#!/usr/bin/env bash
# Lists the tests that pin the keys a slice touches, so a builder runs them filtered before its one full run
# (rescue retros of 306i and 306j: tests that pin a count or a key were found only by a full run, each costing
# about 5 minutes). A slice's research puts this script's output in its section's **Tests**.
#
#   scripts/pinning-tests.sh <key>…
#
# For each key (a selection, feat or option key, or a word such as a count's name), it prints each test file
# under `libs/seeds/tests/game_data/` and `libs/rules/tests/` that names it, with the `fn` names of the tests
# around each match. Plain shell: grep and awk only.
set -euo pipefail

if [ "$#" -eq 0 ]; then
    echo "usage: scripts/pinning-tests.sh <key>…" >&2
    exit 2
fi

root="$(cd "$(dirname "$0")/.." && pwd)"
dirs=("$root/libs/seeds/tests/game_data" "$root/libs/rules/tests")

for key in "$@"; do
    echo "== $key"
    grep -rlF --include='*.rs' --include='*.ron' -- "$key" "${dirs[@]}" 2>/dev/null | sort | while read -r file; do
        rel="${file#"$root"/}"
        # The name of the last `fn` above each line that names the key.
        tests="$(awk -v key="$key" '
            /^[[:space:]]*(pub[[:space:]]+)?fn[[:space:]]+[a-z_0-9]+/ {
                match($0, /fn[[:space:]]+[a-z_0-9]+/); name = substr($0, RSTART + 3, RLENGTH - 3); gsub(/[[:space:]]/, "", name)
            }
            index($0, key) && name != "" { seen[name] = 1 }
            END { for (n in seen) print n }
        ' "$file" | sort | tr '\n' ' ')"
        echo "  $rel: ${tests:-(data file)}"
    done
done
