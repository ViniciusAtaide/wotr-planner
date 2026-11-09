# The MCP server's start time and the mid-session disconnect

Type: task
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

On 2026-11-08, at the start of the Demon Greatsword run (see [112](112-effect-summaries.md)), Claude Code reported:
- `wotr-planner` as still connecting at session start;
- then, at the next turn, as disconnected, and shortly after as connecting and back.

`rustrover` dropped and came back at the same moment, so the client may have restarted both and our server may not be at fault. The cause is unknown.

`.mcp.json` launches `cargo run -q -p mcp --release`. After any source change, the first start compiles the release binary before the server can answer, so "still connecting" can be a build. Two agents in the repo can also contend for the `target/` lock.

## Scope

1. Find the cause: the server logs its start, its data hash and its exit reason to stderr, which Claude Code keeps in its MCP logs. Reproduce a cold start after a source change, and a client reconnect.
2. If the build is the delay: `.mcp.json` runs the built binary (`target/release/wotr-mcp`), built by `scripts/check.sh --full` or a small `scripts/build-mcp.sh`. If the binary is missing, it fails with a message that names the script.
3. Record the cause in this ticket, even if it is the client.

## Open questions (with recommendations)

1. **Prebuilt binary or `cargo run`?** *Recommend:* decide after step 1. `cargo run` never serves a stale binary, while a prebuilt one starts at once but can lag the source.

## Done when

The cause is recorded. A cold start connects without a build when the binary is current. `scripts/check.sh` passes.
