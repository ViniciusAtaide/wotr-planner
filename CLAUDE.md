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

The machine has 8 cores and 24 GB of RAM. Builds, tests, proofs and benchmarks are all CPU-bound and compete for them: three agents building at once drove the load to 85–250 and made each one slower. Reading, writing and reviewing cost almost no local CPU, so those run side by side for free.

- **Two at most:** no more than two agents building at once, each in its own worktree. Tickets whose diffs touch the same files run one after the other, however small.
- **Heavy runs one at a time:** `scripts/proof.sh` (the main session's, once per queue: see "Behaviour checks"), the full agreement sweeps (`--ignored`) and benchmarks. Time speed only on a quiet machine (`uptime` load near 8 or under), comparing medians of alternating runs on both sides.
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

Before reporting a change done, run `scripts/check.sh` (`--full`, the cargo-leptos builds, only at the end of a queue of tickets). Its tests include the snapshot check and the gold builds (see "Behaviour checks"). A pre-commit hook in `.githooks/` runs its fmt and clippy steps; enable it in a new clone with `git config core.hooksPath .githooks`.

- **A change only to `libs/ui/` and `apps/frontend/`:** `scripts/check.sh --ui` is enough: fmt, clippy and the tests of those two crates. It refuses when the branch or the working tree changes any other path (but `.scratch/` and `*.md`).

## Behaviour checks

A change to `rules` or `libs/game-data/` must change no behaviour but the intended one. There are two tiers (`libs/game-data/README.md` describes both tests):

- **Every ticket: the snapshot check.** `cargo test -p seeds --test game_data snapshots`, which `scripts/check.sh` runs. It compares the engine's output for a small corpus of Builds with snapshot files in `libs/seeds/tests/game_data/snapshots/`, and its message groups the differences by path.
  - A ticket that changes no behaviour leaves the snapshot files untouched. If the check reports a difference, find the cause.
  - A ticket that means to change behaviour regenerates them (`SNAPSHOTS=write cargo test -p seeds --test game_data snapshots`), shows the check's grouped message in its report, and commits the files. Every changed line must follow from the intended change.
  - The gold builds (`gold.rs` beside it) compare two real player builds with the engine. A known-gap entry is deleted by the ticket that fixes it, never added to get a test to pass.
- **Once per queue: the proof.** The main session runs `scripts/proof.sh` once, when a queue of tickets is merged, alongside `scripts/check.sh --full` (usage in the script's header and in `tools/proof`'s crate docs). It records about 2,000 Builds on two sides, so it catches what the small corpus can't. A ticket's agent doesn't run it.
  - `scripts/proof.sh <mode> <out> -- --light` records every Build without the `allows` battery and the checks, about 29 seconds a side against 45 for a full record, after the builds. Reach for it between the two tiers, to see a change's reach over the whole corpus before the queue's proof; it misses what only the battery and the checks see, so it never replaces that proof.
  - `scripts/proof.sh <mode> <out> <ref> -- --explain` also writes `explain.txt` when the sides differ: what changed, grouped by path and change with the largest group first, and whether the data or the code caused it. Attribute each group to a ticket from there. Delete `A.dump` and `B.dump` afterwards: about 1.5 MB a Build.

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
