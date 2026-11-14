# wotr-planner

The plan for this project lives in `.scratch/wotr-planner/map.md`. Read its Notes before starting any work, and its tickets under `issues/` for the task at hand.

- **Several tickets with subagents:** the `ticket-queue` skill. A builder is `.claude/agents/ticket-builder.md`, a reviewer `.claude/agents/ticket-reviewer.md`.
- **Reviewing:** `CODING_STANDARDS.md`, read at review, not while building.
- **A QA session through the MCP tools:** the `qa-session` skill.

## Game data

The `.ron` files in `libs/game-data/` are the game data; there is no database (ticket 187). The web server and the MCP server read and check them at startup (`seeds::check_game_data_files`). `cargo leptos watch` restarts its server when one changes; restart a standalone `wotr-mcp` yourself. `cargo run -p seeds --bin check-data` checks them on their own.

- **A data ticket's note** goes in its own file, `libs/game-data/notes/<ticket>-<slug>.md`, so data tickets share no file.
- **The guide as text:** `scripts/guide-text.sh` renders `guide-cache/*.html` to `guide-cache/txt/`, and its `INDEX.txt` leads from a `page#anchor` to the file and line. Read the guide there.
- Build in the worktree's own `target/` (the default). Never share `CARGO_TARGET_DIR` between worktrees: the servers and the tests find the workspace root, and with it `libs/game-data/`, from the path they were compiled in, so a shared build reads another worktree's data.
- A `.env` is optional; `.env.example` lists what it may set.

## Parallel agents

The machine has 8 cores and 24 GB of RAM. Builds, tests, the long self-check runs and benchmarks are all CPU-bound and compete for them: three agents building at once drove the load to 85–250 and made each one slower. Reading, writing and reviewing cost almost no local CPU, so those run side by side for free.

- **Two at most:** no more than two agents building at once, each in its own worktree. Tickets whose diffs touch the same files run one after the other, however small.
- **Heavy runs one at a time:** the self-checks' long runs (`--ignored`: the agreement sweeps and the corpus's checks; the main session's, once per queue: see "Behaviour checks") and benchmarks. Time speed only on a quiet machine (`uptime` load near 8 or under), comparing medians of alternating runs on both sides.
- **Before spawning an agent into a checkout:**
  - its blockers are merged;
  - no other agent is still live there; an agent "waiting on its own background work" is live;
  - no process of another agent runs under that checkout's path.
- **A new worktree starts from current `main`:** confirm with `git merge-base --is-ancestor <main's commit> HEAD` before the first edit, since a new worktree can start from an older commit.

## Rust tests

Cargo compiles only the test modules that are declared:

- **Folder test crates** (`<crate>/tests/<name>/`) compile as one crate rooted at `main.rs`. Each new file in the folder gets a `mod <file>;` line in that `main.rs`, and shared helpers get `mod support;`.
- **Top-level test files** (`<crate>/tests/*.rs`) are each their own crate. Each one that uses the shared helpers in `tests/common/mod.rs` declares `mod common;`.

A new test is done when its name shows up in `cargo test -p <crate> -- --list`.

## Checks

Before reporting a change done, run `scripts/check.sh` (`--full`, the cargo-leptos builds, only at the end of a queue of tickets). Its tests include the self-checks' short runs and the gold builds (see "Behaviour checks"). A pre-commit hook in `.githooks/` runs its fmt and clippy steps; enable it in a new clone with `git config core.hooksPath .githooks`.

- **A change only to `libs/ui/` and `apps/frontend/`:** `scripts/check.sh --ui` is enough: fmt, clippy and the tests of those two crates. It refuses when the branch or the working tree changes any other path (but `.scratch/` and `*.md`).

## Behaviour checks

The game is the source of truth for rules (map Notes; ticket 234). A change to `rules` or `libs/game-data/` is checked against the game, never against the engine's earlier output.

- **Each changed fact has a real-data test** in `libs/seeds/tests/game_data/` whose expected value comes from the game, cited beside the assertion by blueprint name and GUID, or by the decompiled code's file and method with the WrathDecompiled commit.
- **The engine's self-checks** check it against itself: every picker agrees with the walk (the agreement sweeps, ticket 150), each shortcut agrees with the full answer it stands for, and each option's status agrees with `allows` of its edit. Their short runs are in `scripts/check.sh`. The long ones, over the corpus's ~2,000 Builds (`tools/corpus`), run in release under `--ignored`, once per queue by the main session: `cargo test --release -p corpus -- --ignored` and `cargo test --release -p seeds --test game_data agreement -- --ignored`. A self-check never outranks the game: when one disagrees with it, the engine is fixed.
- **The gold builds** (`libs/seeds/tests/game_data/gold.rs`) compare two real player builds with the engine. A known-gap entry is deleted by the ticket that fixes it, never added to get a test to pass. A gold build's gain that the game contradicts goes to the user.
- **The game check** (ticket 238) runs by hand once it exists.

## Formatting

Rust is formatted with rustfmt, the official formatter, through `cargo fmt`. Its style is pinned in `rustfmt.toml` (`style_edition = "2024"`), so `cargo fmt`, the rustfmt hook and RustRover's Reformat Code all produce the same output.

- **After an edit:** a hook runs rustfmt on every `.rs` file an agent writes.
- **Before you finish:** `cargo fmt --all --check` passes.
- **RustRover's `reformat_file` tool** uses the IDE's built-in formatter rather than rustfmt, so Rust formatting goes through `cargo fmt`.

## Dev server

Start the web app with RustRover's **Leptos** run configuration (`cargo leptos watch`), not with Bash: `get_run_configurations`, then `execute_run_configuration` with `configurationName: "Leptos"` and `waitForExit: false`. Its log is at the `fullOutputPath` the call returns.

- **It allows multiple instances** (`singleton="false"`), so each agent starts its own and leaves the others alone. Port 3900 (reload 3901) belongs to the user's server in the main checkout. Any other instance passes `envs` with its own ports, for example `{"LEPTOS_SITE_ADDR": "127.0.0.1:3950", "LEPTOS_RELOAD_PORT": "3951", "WOTR_MCP_PORT": "0"}` (`0` gives its built-in MCP server a free port instead of the sessions' 3902).
- **In a worktree,** also pass `workingDirectory` with the worktree's absolute path: the configuration runs in the main checkout by default, the only one RustRover has open.
- **It counts as a build** (see "Parallel agents"): starting it, and every save it rebuilds on, compiles.
- **The server reads the game data once, at startup.** `cargo leptos watch` restarts it when a file under `libs/game-data/` changes (`watch-additional-files` in `Cargo.toml`). An instance started before that setting was added needs one restart by hand: stop it, then start it again.
- **Stop your own instance when you're done:** kill the `cargo-leptos` process whose working directory is your checkout (`lsof -a -d cwd -p <pid>`), then the server still listening on your port (`lsof -tiTCP:<port> -sTCP:LISTEN`).
