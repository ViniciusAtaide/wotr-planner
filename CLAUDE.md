# wotr-planner

The plan for this project lives in `.scratch/wotr-planner/map.md`. Read its Notes before starting any work, and its tickets under `issues/` for the task at hand.

- **Several tickets with subagents:** the `ticket-queue` skill. A builder is `.claude/agents/ticket-builder.md`, a reviewer `.claude/agents/ticket-reviewer.md`.
- **Reviewing:** `CODING_STANDARDS.md`, read at review, not while building.
- **A QA session through the MCP tools:** the `qa-session` skill.

## Game data

The `.ron` files in `libs/game-data/` are the game data; there is no database (ticket 187). Since ticket 244 each entry is its own file, in a folder by kind, named by its key (`features/sneak-attack.ron`); an entry holds no other, rows name what they grant by key, and `planner/` holds the entries with no game blueprint, each saying why (`libs/game-data/README.md`, Layout). The web server and the MCP server read and check them at startup (`seeds::check_game_data_files`). `cargo leptos watch` restarts its server when one changes; restart a standalone `wotr-mcp` yourself. `cargo run -p seeds --bin check-data` checks them on their own.

- **A data ticket's note** goes in its own file, `libs/game-data/notes/<ticket>-<slug>.md`, so data tickets share no file.
- **The guide as text:** `scripts/guide-text.sh` renders `guide-cache/*.html` to `guide-cache/txt/`, and its `INDEX.txt` leads from a `page#anchor` to the file and line. Read the guide there.
- Build in the worktree's own `target/` (the default). Never share `CARGO_TARGET_DIR` between worktrees: the servers and the tests find the workspace root, and with it `libs/game-data/`, from the path they were compiled in, so a shared build reads another worktree's data.
- A `.env` is optional; `.env.example` lists what it may set.

## Game sources

The game is the source of truth for rules (map Notes, ticket 234), at patch 2.7.0x. Two git-ignored folders at the repo root hold its sources; `scripts/game-sources.sh` fills them at the pins in `scripts/game-sources.pins` (the dump's SHA-256 and the code's commit) and checks a filled one against them.

- **`blueprints/`:** BubblePrintsData's release `2.7.0x_0`, one `.binz` file (the game's blueprints as JSON, in BubblePrints' container).
- **`decompiled/`:** xADDBx/WrathDecompiled, shallow, at the pinned commit: the game's code.
- **Citing:** a fact or a rule cites a blueprint by name and GUID, or the code by file and method with the WrathDecompiled commit.
- **Worktrees:** both folders live only in the main checkout; an agent in a worktree reads them there by absolute path and does not run the script.
- **Reading a blueprint:** `cargo run -p game-check -- show <name|guid>…` prints one, references resolved and no game text; `grep <regex>` lists the blueprints whose name or JSON match; `refs <name|guid>` lists those that reference one. They find the dump from a worktree on their own.
- **Never in the repo:** the game's text strings, the dump and its code, not even quoted in a ticket.

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

A ticket runs only what its diff can break, after its last edit, and fmt (`cargo fmt --all --check`). A pre-commit hook in `.githooks/` runs fmt and clippy on every commit that touches Rust; enable it in a new clone with `git config core.hooksPath .githooks`. Each row adds to the others for a diff that touches several.

| The diff touches | It runs |
| --- | --- |
| a crate's code or tests | `cargo test -p <crate>` for each crate touched (`ui`, `frontend`, `server`, `rules`, `seeds`, `corpus`, `mcp`, `game-check`) |
| `libs/rules/` or `libs/game-data/` | also `cargo test -p seeds -p corpus`: the real-data tests, the gold builds and the self-checks' short runs live there |
| `libs/game-data/` | also `cargo run -p seeds --bin check-data` |
| `libs/rules/` | also `cargo build -p rules --target wasm32-unknown-unknown` |
| `libs/ui/` or `apps/frontend/` | also `cargo clippy -p frontend --target wasm32-unknown-unknown -- -D warnings` and `scripts/css-classes.sh` |
| only `*.md`, `.scratch/` or scripts | `bash -n` on a changed script; nothing else |

**Once per queue,** after the last merge, the main session runs `scripts/queue-checks.sh`: fmt, `scripts/css-classes.sh`, clippy on the workspace and on the frontend's wasm target, `cargo test --workspace`, the rules wasm build, `check-data`, the cargo-leptos dev and release builds, and `scripts/build-mcp.sh`. It is one script so a person runs the list by hand in order, with each step's time and a stop at the first failure; CI (`.github/workflows/check.yml`) runs the same steps but the two builds, which the Pages workflow covers. Beside it, the self-checks' long runs (see "Behaviour checks").

## Behaviour checks

The game is the source of truth for rules (map Notes; ticket 234). A change to `rules` or `libs/game-data/` is checked against the game, never against the engine's earlier output.

- **Each changed fact has a real-data test** in `libs/seeds/tests/game_data/` whose expected value comes from the game, cited beside the assertion by blueprint name and GUID, or by the decompiled code's file and method with the WrathDecompiled commit (see "Game sources").
- **The engine's self-checks** check it against itself: every picker agrees with the walk (the agreement sweeps, ticket 150), each shortcut agrees with the full answer it stands for, and each option's status agrees with `allows` of its edit. Their short runs are in the `seeds` and `corpus` tests. The long ones, over the corpus's ~2,000 Builds (`tools/corpus`), run in release under `--ignored`, once per queue by the main session: `cargo test --release -p corpus -- --ignored` and `cargo test --release -p seeds --test game_data agreement -- --ignored`. A self-check never outranks the game: when one disagrees with it, the engine is fixed.
- **A ticket that changes only the data's or the model's form** proves nothing moved: `FORM_PROOF_OUT=<dir> cargo test --release -p corpus --test form_proof record -- --ignored --nocapture` on its base (in a temporary worktree of the base commit), then `FORM_PROOF_BEFORE=<dir> ... compare` on its tip (ticket 300; usage in the test's header). It puts the counts `compare` prints in its Answer. It replaces the snapshot check and a per-ticket proof, and runs only for such a ticket.
- **The gold builds** (`libs/seeds/tests/game_data/gold.rs`) compare two real player builds with the engine. A known-gap entry is deleted by the ticket that fixes it, never added to get a test to pass. A gold build's gain that the game contradicts goes to the user.
- **The game check** (ticket 238) compares each entry that names a blueprint with the game: `cargo run -p game-check -- --binz <main checkout>/blueprints/blueprints_raw_2.7.0x_0.binz` (the path is the default in the main checkout). It reads only the pinned dump and prints the same report for the same dump and data. It exits 0 when every difference is listed in `tools/game-check/known.txt` under the ticket that will remove it, 1 when one isn't or a listed one is stale, 2 when the dump is refused. The main session runs it after merging any data ticket and once per queue. Each new difference gets a ticket and is applied without asking, unless it touches one of the user's facts, which go to the user first.

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
