#!/usr/bin/env bash
# The full set of checks, run once per queue of tickets by the main session, after the last merge.
#
#   scripts/queue-checks.sh [<base>]
#
# guarded-files (scripts/guarded-files.sh since the queue's base, and its test: an expected value of the gold Builds or
# a held user fact changed only with the user's `User approved:` line), fmt, css-classes (scripts/css-classes.sh: every
# class in the stylesheet is used), test-modules (every test file is declared), clippy (workspace and the frontend's
# wasm target), the workspace tests, the rules wasm build, `check-data` (the data files), the cargo-leptos dev and
# release builds, and scripts/build-mcp.sh (the standalone MCP server, `wotr-mcp`).
# The queue's base (the commit `main` was at before the queue's first merge) is the argument, else `QUEUE_BASE`, else
# the merge base with the branch's upstream (the queue's base while the queue isn't pushed).
# Prints one line per step with its time, and ALL-OK at the end. Stops at the first failing step and shows its output.
# A ticket runs only the scoped checks of CLAUDE.md "Checks"; the self-checks' long runs and the game check are not here.
set -euo pipefail

if [[ $# -gt 1 || ${1:-} == -* ]]; then
    sed -n '2,14p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 2
fi

cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"

base=${1:-${QUEUE_BASE:-}}
if [[ -z $base ]]; then
    base=$(git merge-base HEAD '@{upstream}' 2>/dev/null) || {
        echo "queue-checks: no base: pass the queue's base or set QUEUE_BASE (the branch has no upstream)" >&2
        exit 2
    }
fi

log=$(mktemp)
trap 'rm -f "$log"' EXIT

# Runs `$2…` as step $1: quiet when it passes, its output when it fails.
step() {
    local name=$1 start=$SECONDS
    shift
    if "$@" >"$log" 2>&1; then
        echo "$name OK ($((SECONDS - start))s)"
    else
        cat "$log"
        echo "$name FAILED" >&2
        exit 1
    fi
}

step "guarded-files (since $(git rev-parse --short "$base" 2>/dev/null || echo "$base"))" scripts/guarded-files.sh "$base"
# Passing, it may have printed the user's approvals: shown, for the queue's review.
sed 's/^/  /' "$log"
step guarded-files-test scripts/guarded-files-test.sh
step fmt cargo fmt --all --check
step css-classes scripts/css-classes.sh
step test-modules scripts/test-modules.sh
step clippy cargo clippy -q --workspace --all-targets -- -D warnings
step wasm-clippy cargo clippy -q -p frontend --target wasm32-unknown-unknown -- -D warnings
step test cargo test -q --workspace
step rules-wasm cargo build -q -p rules --target wasm32-unknown-unknown
step data-check cargo run -q -p seeds --bin check-data
step leptos-dev cargo leptos build
step leptos-release cargo leptos build --release
step mcp-release scripts/build-mcp.sh
echo ALL-OK
