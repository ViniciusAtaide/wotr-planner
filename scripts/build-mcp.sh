#!/usr/bin/env bash
# Builds the standalone MCP server: target/release/wotr-mcp.
#
#   scripts/build-mcp.sh
#
# `.mcp.json` connects over HTTP to http://localhost:3902/mcp, which the app's server serves (`cargo leptos watch`).
# To serve it without the app, run the binary with the port set: `WOTR_MCP_PORT=3902 target/release/wotr-mcp`;
# the project's SessionStart hook (`.claude/settings.json`) runs scripts/start-mcp.sh, which does that in the
# background when nothing listens on 3902.
# Without WOTR_MCP_PORT it serves stdio, for a client that starts it. Rebuild after a change to apps/mcp, libs/rules
# or libs/seeds; until then it serves the old code and logs a warning naming the newer file. Game data is read at
# start, so a data change needs no rebuild. `scripts/check.sh --full` runs it too.
set -euo pipefail

cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cargo build -q -p mcp --release
echo "built $PWD/target/release/wotr-mcp"
