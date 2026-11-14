#!/usr/bin/env bash
# The per-change check: every step that can catch something a ticket breaks.
#
#   scripts/check.sh [--full | --ui]
#
# Always: fmt, clippy (workspace and the frontend's wasm target), the workspace tests, the rules wasm build and
# `check-data` (the data files).
# --full:   also the cargo-leptos dev and release builds. They are slow and rarely break, so run them once at the end
#           of a queue of tickets, not per ticket. Then scripts/build-mcp.sh, the standalone MCP server (`wotr-mcp`).
# --ui:     for a change only to the web UI: fmt, clippy and the tests of `ui` and `frontend` alone. Refuses when the
#           branch (since main) or the working tree changes a path outside libs/ui/, apps/frontend/, .scratch/ and *.md.
# Prints one line per step with its time, and ALL-OK at the end. Stops at the first failing step and shows its output.
set -euo pipefail

usage() {
    sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 2
}

full=false
ui=false
for arg in "$@"; do
    case $arg in
    --full) full=true ;;
    --ui) ui=true ;;
    *) usage ;;
    esac
done
if $full && $ui; then
    usage
fi

cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"

if $ui; then
    # Every path changed since the branch left main: its commits, staged and unstaged changes, and untracked files.
    changed=$(
        git diff --name-only --no-renames "$(git merge-base HEAD main)"
        git ls-files --others --exclude-standard
    )
    outside=$(grep -vE '^(libs/ui/|apps/frontend/|\.scratch/)|\.md$' <<<"$changed" | sort -u || true)
    if [[ -n $outside ]]; then
        echo "--ui covers only libs/ui/ and apps/frontend/, but these paths changed:" >&2
        sed 's/^/  /' <<<"$outside" >&2
        echo "Run scripts/check.sh without --ui." >&2
        exit 1
    fi
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

step fmt cargo fmt --all --check
if $ui; then
    step clippy cargo clippy -q -p ui -p frontend --all-targets -- -D warnings
    step wasm-clippy cargo clippy -q -p frontend --target wasm32-unknown-unknown -- -D warnings
    step test cargo test -q -p ui -p frontend
    echo ALL-OK
    exit 0
fi
step clippy cargo clippy -q --workspace --all-targets -- -D warnings
step wasm-clippy cargo clippy -q -p frontend --target wasm32-unknown-unknown -- -D warnings
step test cargo test -q --workspace
step rules-wasm cargo build -q -p rules --target wasm32-unknown-unknown
step data-check cargo run -q -p seeds --bin check-data
if $full; then
    step leptos-dev cargo leptos build
    step leptos-release cargo leptos build --release
    step mcp-release scripts/build-mcp.sh
fi
echo ALL-OK
