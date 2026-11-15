#!/usr/bin/env bash
# The full set of checks, run once per queue of tickets by the main session, after the last merge.
#
#   scripts/queue-checks.sh
#
# fmt, css-classes (scripts/css-classes.sh: every class in the stylesheet is used), clippy (workspace and the
# frontend's wasm target), the workspace tests, the rules wasm build, `check-data` (the data files), the cargo-leptos
# dev and release builds, and scripts/build-mcp.sh (the standalone MCP server, `wotr-mcp`).
# Prints one line per step with its time, and ALL-OK at the end. Stops at the first failing step and shows its output.
# A ticket runs only the scoped checks of CLAUDE.md "Checks"; the self-checks' long runs and the game check are not here.
set -euo pipefail

if [[ $# -gt 0 ]]; then
    sed -n '2,9p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 2
fi

cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"

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

step fmt cargo fmt --all --check
step css-classes scripts/css-classes.sh
step clippy cargo clippy -q --workspace --all-targets -- -D warnings
step wasm-clippy cargo clippy -q -p frontend --target wasm32-unknown-unknown -- -D warnings
step test cargo test -q --workspace
step rules-wasm cargo build -q -p rules --target wasm32-unknown-unknown
step data-check cargo run -q -p seeds --bin check-data
step leptos-dev cargo leptos build
step leptos-release cargo leptos build --release
step mcp-release scripts/build-mcp.sh
echo ALL-OK
