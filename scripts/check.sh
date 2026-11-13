#!/usr/bin/env bash
# The per-change check: every step that can catch something a ticket breaks.
#
#   scripts/check.sh [--schema] [--full]
#
# Always: fmt, clippy (workspace and the frontend's wasm target), the workspace tests, the rules wasm build and the
# seed's --check-only (the data files, no database).
# --schema: also `cargo sqlx prepare --check`, after a change to the migration or a query, against a new SQLite file
#           migrated from libs/seeds/migrations (needs sqlx-cli with its `sqlite` feature, not the dev database).
# --full:   also the cargo-leptos dev and release builds. They are slow and rarely break, so run them once at the end
#           of a queue of tickets, not per ticket. Then scripts/build-mcp.sh, the standalone MCP server (`wotr-mcp`).
# Prints one line per step with its time, and ALL-OK at the end. Stops at the first failing step and shows its output.
set -euo pipefail

schema=false
full=false
for arg in "$@"; do
    case $arg in
    --schema) schema=true ;;
    --full) full=true ;;
    *) sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//' >&2 && exit 2 ;;
    esac
done

cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
log=$(mktemp)
schema_dir=$(mktemp -d)
trap 'rm -rf "$log" "$schema_dir"' EXIT

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
step clippy cargo clippy -q --workspace --all-targets -- -D warnings
step wasm-clippy cargo clippy -q -p frontend --target wasm32-unknown-unknown -- -D warnings
step test cargo test -q --workspace
step rules-wasm cargo build -q -p rules --target wasm32-unknown-unknown
step seed-check cargo run -q -p seeds --bin seed -- --check-only
if $schema; then
    schema_url=sqlite://$schema_dir/schema.db
    step sqlx-migrate sh -c "sqlx database create -D '$schema_url' && sqlx migrate run -D '$schema_url' --source libs/seeds/migrations"
    step sqlx-prepare sh -c "cd libs/seeds && cargo sqlx prepare --check -D '$schema_url'"
fi
if $full; then
    step leptos-dev env SQLX_OFFLINE=true cargo leptos build
    step leptos-release cargo leptos build --release
    step mcp-release scripts/build-mcp.sh
fi
echo ALL-OK
