# The MCP server doesn't say which game data it serves

Type: bugfix
Status: open
Blocked by: [202](202-the-web-app-says-the-gear-notes-in-the-engines-words.md) (the Offers queue, 199 to 202, edits `apps/mcp`)
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

From the retrospective of the queue of tickets 188 to 193 (2026-11-14; user: "do every recommendation"). The servers read `libs/game-data/` once, at startup. After a data-only merge the dev server kept the old data: the code review's second pass could not reproduce through the MCP tools, and the QA session found a row the data no longer had. Nothing in a tool's answer says which data the server holds, so an agent can't tell a stale server from a bug.

The first half is done outside this ticket: `Cargo.toml`'s leptos section watches `libs/game-data` (`watch-additional-files`), so `cargo leptos watch` restarts the server when the data changes. The standalone `wotr-mcp` (`scripts/start-mcp.sh`) still reads once.

## Build

- The envelope every tool returns (`build`, `link`, `todo`) gains the game data's content hash, the one the web server already names its JSON by, in a short form. Find where the envelope is built (`apps/mcp/src/shapes.rs`) and where the hash is computed.
- `scripts/check.sh`'s `data-check` step, or `check-data`, prints the same short hash for the files on disk, so "is the server current?" is one comparison. The `qa-session` skill's first step and `CLAUDE.md`'s dev server section say so.
- `wotr-mcp`: decide between rereading when the files change and saying in its instructions that it must be restarted; the smaller one that leaves no stale answer unmarked.
- Ticket [197](197-leftovers-from-the-second-mcp-qa-session.md)'s item 2 (a valid code reported as a stale link while the server restarts) is the same area: fix it here and strike it there.

## Done when

A tool's answer carries the data's hash; `check-data` prints the files' hash; an MCP test pins that the two agree for the test server; the tool descriptions stay within the instructions' length test; `scripts/check.sh` passes.
