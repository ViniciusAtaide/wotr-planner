# The MCP server's start time and the mid-session disconnect

Type: task
Status: resolved
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

## Choices settled

1. **The launch is the built binary** (open question 1). `.mcp.json` runs `target/release/wotr-mcp` through `sh -c`, from the checkout's root (`git rev-parse --show-toplevel`), so a session started in a subfolder finds it too. When the binary is missing, it exits 1 with `wotr-mcp: no <path>: build it with scripts/build-mcp.sh` on stderr. `SQLX_OFFLINE` is gone from `.mcp.json`; only the build needs it.
2. **`scripts/build-mcp.sh`** runs `SQLX_OFFLINE=true cargo build -q -p mcp --release` from the root. `scripts/check.sh --full` runs it as its last step (`mcp-release`), so the binary is current after a queue of tickets.
3. **The lag is logged, not prevented.** At start the server compares its binary's modification time with the `.rs` files under `apps/mcp/src`, `libs/rules/src` and `libs/seeds/src`. If one is newer, it warns on stderr and names the file and the script. Game data is read at start, so a data change never needs a rebuild.
4. **stderr gets one line per event** (`main.rs`; stdout stays the protocol):
   - the start: version, pid and binary;
   - the staleness warning;
   - the data hash, as before;
   - the client, once it has initialized: name, version, protocol and the time since start;
   - the exit: `exiting after N s: the client closed stdin`, `cancelled`, `the service task failed: …`, `exiting before the client initialized: …`, or a data load error.

   A signal isn't logged. Claude Code stops a server with SIGINT and logs that itself (`Sending SIGINT to MCP server process`). Catching it would need tokio's `signal` feature in `apps/mcp/Cargo.toml`, outside this ticket's files.

## Progress log

- 2026-11-08: Read Claude Code's MCP logs for this project (`~/Library/Caches/claude-cli-nodejs/-Users-thaynamarcela-code-wotr-planner/mcp-logs-{wotr-planner,rustrover}/`), the desktop app's `~/Library/Logs/Claude/main.log` and the two sessions' transcripts. Read only. Times are UTC (local is UTC−3). The run spans two CLI sessions of one desktop session (`local_7f715d77`):
  - **Session 5edf5d1d** started at 22:01:29.799, "check if you have an mcp connection available". The first turn was assembled at 22:01:29.836, 37 ms after the server's start, and listed `pendingMcpServers: ["wotr-planner"]`, "still connecting". wotr-planner: `Successfully connected (transport: stdio) in 1297ms` at 22:01:31.039; the timeout is `30000ms`. Then idle for 3½ hours.
  - **01:35:23**, desktop log: `OS resume` (the Mac had slept). At 01:35:54 the user sent "can you make the strongest martial build possible…" to 5edf5d1d.
  - **01:36:15**, `[Request interrupted by user]` and desktop `LocalSessions.interrupt`. At 01:36:16: `LocalSessions.rewindV2 … Stopping session`. The CLI stopped every server:
    - wotr-planner: `Sending SIGINT to MCP server process`, `STDIO connection closed after 12885s (cleanly)`, `MCP server process exited cleanly`;
    - rustrover: `HTTP connection closed after 12893s (cleanly)`, `Cleared connection cache for reconnection`, the same millisecond.
  - **01:36:23**, desktop: `[Rewind] resumeSessionAt=… + forkSession`, `CLI session ID changed from 5edf5d1d… to 3a262c66…`. The edited prompt ("…in wotr-planner?") went to the new process. Its first turn listed `claude-artifacts`, `rustrover` and `wotr-planner` as pending and announced `mcp__rustrover__*` and `mcp__wotr-planner__*` as disconnected. rustrover connected in 141 ms and wotr-planner in 1379 ms (01:36:25.018). The next turn's attachment (01:36:29.815) announced them reconnected. That session then ran the whole Demon Greatsword build without another drop.
  - No error from the server in either log. Its only stderr is the data line, which Claude Code files under `"error"` as `Server stderr: …`.
- Measured the time to answer an `initialize` piped on stdin (a scratchpad Python script; Apple Silicon, `nightly-2026-10-30`), in this worktree:

  | Start | Time |
  | --- | --- |
  | `cargo run -q -p mcp --release`, clean checkout (no `target/`) | 89.3 s |
  | `cargo run`, after `touch apps/mcp/src/main.rs` (bin only) | 8.6 s, 6.3 s |
  | `cargo run`, after `touch apps/mcp/src/server.rs` (lib and bin) | 12.0 s |
  | `cargo run`, after `touch libs/rules/src/lib.rs` | 28.7 s |
  | the same, while another release `cargo build` holds the lock | 37.5 s, 33.3 s (`Blocking waiting for file lock on build directory`) |
  | `cargo run`, while a debug `cargo build` runs (another profile) | 0.72 s |
  | `cargo run`, warm (nothing changed) | 0.36–0.48 s |
  | `target/release/wotr-mcp`, warm | 0.02–0.03 s |
  | the new `.mcp.json` launch (`sh`, `git`, binary) | 0.06–0.08 s (0.45 s on a new binary's first run) |

- The build lock: nothing in the main checkout was building during the check. The only process was the running server (`target/release/wotr-mcp`, pid 59115: `cargo run` execs the binary). Cargo's build-directory lock is per profile:
  - `cargo leptos watch` (the dev server on 3900) builds the `dev` profile, so it never blocks the release `cargo run`;
  - `cargo leptos build --release` (`check.sh --full`) and any release build in the main checkout do block it;
  - agents in worktrees build in their own `target/`, so they never hold the main checkout's lock.
- Wrote the stderr lines, `scripts/build-mcp.sh` and the new `.mcp.json`. Tried the launch four ways:
  - current binary, from the root and from `apps/mcp`: it answers, then logs `serving time-init 0 (protocol 2025-06-18) after 118 ms` and `exiting after 0 s: the client closed stdin`, exit 0;
  - after `touch libs/rules/src/lib.rs`: the warning names the file;
  - binary moved away: `wotr-mcp: no …/target/release/wotr-mcp: build it with scripts/build-mcp.sh`, exit 1, nothing on stdout;
  - stdin empty, or a request before `initialize`: `exiting before the client initialized: connection closed: initialize request`, exit 1.
- `SQLX_OFFLINE=true scripts/check.sh`: ALL-OK.

## Answer

Done 2026-11-08. **Both symptoms came from the client, not the server.**

1. **"Still connecting" at session start** is how Claude Code starts: it assembles the first turn while the servers are still starting (37 ms after the start). It listed rustrover and claude-artifacts as pending too. With `cargo run`, wotr-planner took 1.3 s to connect (cargo's freshness check plus the start), so it was always pending on the first turn. With the binary it takes 0.06 s. That can still be pending on a first turn assembled within 37 ms, as rustrover's 141 ms was, but it is ready by the next tool call.
2. **The disconnect and reconnect** was the user's interrupt and rewind. Editing the sent message makes the desktop app stop the Claude Code process and resume a fork of the session in a new one. Stopping the process stops every MCP server: wotr-planner by SIGINT and rustrover by closing its HTTP connection, both "cleanly", in the same millisecond. Then they start again in the new process. Our server exited cleanly and logged no error. Any rewind does this again; it isn't a fault to fix here.

**The launch is now the built binary anyway**, because of the measurements. After a source change, `cargo run` makes every session start wait on a build:
- 6–12 s after an `apps/mcp` edit;
- 29 s after a `rules` edit, against Claude Code's 30 s connect timeout;
- 33–38 s while another release build holds the lock, over the timeout;
- 89 s on a clean checkout.

A rewind restarts the server too, so a restart after a merge would wait as well. The binary starts in 0.06 s. Its cost is that it can lag the source, so the server warns on stderr when a source file is newer. Run `scripts/build-mcp.sh` after merging a change to `apps/mcp`, `libs/rules` or `libs/seeds`; `check.sh --full` runs it too.

**After this merges**, run `scripts/build-mcp.sh` once in the main checkout before the next session. Claude Code reads `.mcp.json` from there, and until the binary exists `/mcp` shows wotr-planner as failed; the reason (the script's name) is in its MCP log.
