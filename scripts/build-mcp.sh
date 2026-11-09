#!/usr/bin/env bash
# Builds the MCP server that `.mcp.json` runs: target/release/wotr-mcp.
#
#   scripts/build-mcp.sh
#
# `.mcp.json` runs the built binary rather than `cargo run`, so a session never waits on a build to connect (ticket
# 118). Run this once in a new clone, and again after a change to apps/mcp, libs/rules or libs/seeds; until then
# the server serves the old code and logs a warning naming the newer file. Game data is read at start, so a data
# change needs no rebuild. `scripts/check.sh --full` runs it too. Restart the server afterwards (`/mcp` in Claude
# Code).
set -euo pipefail

cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
SQLX_OFFLINE=true cargo build -q -p mcp --release
echo "built $PWD/target/release/wotr-mcp"
