# The MCP server over HTTP, and the app's server serving it

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The user asked (2026-11-09) for the MCP server to work over HTTP, ideally on a localhost port of this project: "make it an environment variable and make the deployed process also serve the mcp port". [104](104-mcp-server-plan.md) had chosen stdio only (its open question 5), noting that Streamable HTTP could be added later without a redesign because the server holds no state between calls.

## Choices settled

1. **The transport is rmcp's streamable HTTP server** (`transport-streamable-http-server`, rmcp 3.9.0) on an axum router at `/mcp`. The shared code is `apps/mcp/src/http.rs` (`mcp::http`): `Listener::bind` binds `127.0.0.1:<port>` so a busy port is reported first, and `serve` runs until a stop future completes, then ends every session. Each client gets its own session (rmcp's `LocalSessionManager`) and its own clone of the `Server`, which shares the planner.
2. **Loopback only.** It binds the loopback address, and rmcp's default `allowed_hosts` (localhost, 127.0.0.1, ::1) refuse any other `Host` header with 403, its guard against DNS rebinding. No authentication: nothing outside the machine can reach it.
3. **The port is `WOTR_MCP_PORT`.** An empty value counts as unset; anything other than a port number is an error.
   - `wotr-mcp` serves HTTP when it's set and stdio otherwise, so a client that starts the binary still works. Over HTTP it logs the URL and stops on SIGINT or SIGTERM (tokio's `signal` feature).
   - `wotr-mcp` doesn't read `.env`, so a port set there can't switch a stdio launch to HTTP.
4. **The app's server serves it too** (`apps/backend`, `serve_mcp`): on `WOTR_MCP_PORT`, else `mcp::http::DEFAULT_PORT`, 3902. 3900 is the site and 3901 is cargo-leptos's live reload. It reads the game data files (`seeds::game_data_dir`), as `wotr-mcp` does, not the database. A failure (a bad port value, the data, a busy port) is a warning and the site is served anyway.
5. **`.mcp.json` connects over HTTP** to `http://localhost:3902/mcp`. Something must be listening: the dev server (`cargo leptos watch`) or `WOTR_MCP_PORT=3902 target/release/wotr-mcp`. The dev server restarts on every source change, which drops the connection until it's back, and its debug build of the rules engine is slower than the release binary.
6. **GitHub Pages can't host it.** Pages serves static files on 443 only, with no process and no POST handling, so the deployed site has no MCP server. Hosting it would need a host that runs a process, and authentication in place of the loopback guard.

## Progress log

- 2026-11-09: Built and ran both:
  - `WOTR_MCP_PORT=3912 target/release/wotr-mcp`: `initialize` gave a session, `tools/list` the eight tools, `search` "BFT" found `arcanist/brown-fur-transmuter`, `Host: evil.example` got 403, SIGINT closed the sessions and exited 0.
  - The release app server with `WOTR_MCP_PORT=3913`: the site answered 200 on its own port and MCP `initialize` on 3913.
  - `wotr-mcp` without the variable: `initialize` over stdio still answers.
- `SQLX_OFFLINE=true scripts/check.sh`: ALL-OK.

## Answer

Done 2026-11-09. `WOTR_MCP_PORT` names the port. The app's server always serves MCP at `http://localhost:<port>/mcp` (3902 by default), and `wotr-mcp` serves it when the variable is set and stdio otherwise. `.mcp.json` connects to 3902 over HTTP.
