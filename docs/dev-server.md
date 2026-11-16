# Dev server

Moved from `CLAUDE.md` (ticket 311): needed only when an agent starts the web app.

Start the web app with RustRover's **Leptos** run configuration (`cargo leptos watch`), not with Bash: `get_run_configurations`, then `execute_run_configuration` with `configurationName: "Leptos"` and `waitForExit: false`. Its log is at the `fullOutputPath` the call returns.

- **It allows multiple instances** (`singleton="false"`), so each agent starts its own and leaves the others alone. Port 3900 (reload 3901) belongs to the user's server in the main checkout. Any other instance passes `envs` with its own ports, for example `{"LEPTOS_SITE_ADDR": "127.0.0.1:3950", "LEPTOS_RELOAD_PORT": "3951", "WOTR_MCP_PORT": "0"}` (`0` gives its built-in MCP server a free port instead of the sessions' 3902).
- **In a worktree,** also pass `workingDirectory` with the worktree's absolute path: the configuration runs in the main checkout by default, the only one RustRover has open.
- **It counts as a build** (see "Parallel agents"): starting it, and every save it rebuilds on, compiles.
- **The server reads the game data once, at startup.** `cargo leptos watch` restarts it when a file under `libs/game-data/` changes (`watch-additional-files` in `Cargo.toml`). An instance started before that setting was added needs one restart by hand: stop it, then start it again.
- **Stop your own instance when you're done:** kill the `cargo-leptos` process whose working directory is your checkout (`lsof -a -d cwd -p <pid>`), then the server still listening on your port (`lsof -tiTCP:<port> -sTCP:LISTEN`).
