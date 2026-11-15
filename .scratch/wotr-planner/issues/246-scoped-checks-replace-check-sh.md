# Scoped checks per ticket replace scripts/check.sh

Type: build
Status: resolved
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

Every builder and every review fix round runs `scripts/check.sh`, whose workspace test step takes 3 to 6 minutes. With two lanes and several fix rounds per ticket, that is most of a queue's wall-clock time.

The user, 2026-11-15: "let's try to focus as much as possible to pull the work into removing check.sh to be as early. having to run it make the migration proccess much slower". Ticket 234 had already said "we don't even need proof.sh and check.sh as they are today anymore" (235 removed the proof).

Decided (user, 2026-11-15, recommendations taken):
- A ticket runs scoped checks.
- The main session runs the full set once per queue, after the last merge.
- This ticket goes next, before the rest of the queue.

## Change

- **Delete `scripts/check.sh`**, with its `--full` and `--ui` modes.
- **Per ticket, scoped.**
  - The pre-commit hook (`.githooks/pre-commit`) keeps fmt and clippy on every commit that touches Rust. Its comment stops calling itself "the fast half of scripts/check.sh".
  - The builder runs, after its last edit, only what its diff can break:
    - `cargo test -p <crate>` for each crate the diff touches. A change to `rules` or `libs/game-data/` also runs `seeds`' tests, since the real-data tests live there.
    - `cargo run -p seeds --bin check-data` for a data change.
    - `cargo build -p rules --target wasm32-unknown-unknown` for a `rules` change.
    - `cargo clippy -p frontend --target wasm32-unknown-unknown -- -D warnings` and `scripts/css-classes.sh` for a change to `libs/ui/` or `apps/frontend/`.

    `CLAUDE.md` "Checks" holds this as a short table, from what the diff touches to what it runs.
- **Once per queue, the main session runs the full set** after the last merge:
  - fmt;
  - `scripts/css-classes.sh`;
  - clippy on the workspace and on the frontend's wasm target;
  - `cargo test --workspace`;
  - the rules wasm build;
  - `check-data`;
  - the cargo-leptos dev and release builds;
  - `scripts/build-mcp.sh`.

  These are listed as commands in `CLAUDE.md`, beside the self-checks' long runs and the game check (ticket 238). Choose the form for a list a human runs by hand, and say why: plain commands, or a small `scripts/queue-checks.sh` that runs them in order with timings.
- **CI** (`.github/workflows/check.yml`) runs the same full set as explicit steps, since CI's time isn't a queue's.
- **Every text that names `check.sh`** is rewritten:
  - `CLAUDE.md` ("Checks", "Behaviour checks", "Parallel agents", "Formatting");
  - `.claude/agents/ticket-builder.md` (step 7, the Builds rule, the report) and `ticket-reviewer.md`;
  - the `ticket-queue` skill (steps 8 and 9);
  - `README.md`;
  - `scripts/build-mcp.sh`'s comment;
  - `tools/corpus`'s docs ("their short runs are in scripts/check.sh");
  - anything `git grep -n 'check\.sh' -- ':!.scratch' ':!.agents'` finds. `.agents/` is a third-party skill with its own `check.sh`; leave it.

## Answer

**Built** (2026-11-15). `scripts/check.sh` is gone; a ticket runs scoped checks, the main session runs the full set once per queue.

- **`CLAUDE.md` "Checks":** the table from what the diff touches to what it runs (crate tests; `seeds` tests for `rules` or `game-data`; `check-data` for data; the rules wasm build for `rules`; wasm clippy and `css-classes.sh` for `ui` and `frontend`; `bash -n` for docs and scripts only), and the once-per-queue set. The sentence on the self-checks' short runs now names the `seeds` and `corpus` tests; the line on the game check (238) is left for that ticket.
- **`scripts/queue-checks.sh`:** the once-per-queue set as one script, one line per step with its time, stopping at the first failure: fmt, css-classes, clippy (workspace, frontend wasm), `cargo test --workspace`, rules wasm build, `check-data`, both cargo-leptos builds, `scripts/build-mcp.sh`.
- **CI** (`.github/workflows/check.yml`): the same steps, explicit, but the two cargo-leptos builds and the MCP release build, which the Pages workflow covers (as `check.sh` without `--full` already did).
- **Rewritten:** `ticket-builder.md` (the brief may allow `CLAUDE.md`, step 7, the Builds rule, the report), `ticket-reviewer.md` (no workspace run), the `ticket-queue` skill (steps 8 and 9), `.githooks/pre-commit`'s comment, `scripts/build-mcp.sh`'s comment. `README.md` and `tools/corpus` named `check.sh` nowhere in this base.

**Decisions:**
- The user, 2026-11-15: scoped checks per ticket, the full set once per queue, this ticket first.
- Ours, recommended: a small script rather than plain commands. The list has ten steps a person runs in order and wants timed, stopped at the first failure and quiet when green; a pasted list gives none of that, and the script is the same commands as the CI steps, so the two stay easy to compare. It takes no arguments.
- Ours: the CI workflow lists steps rather than calling the script, as the ticket says, and leaves out the builds Pages covers.

**Tests:** no Rust changed. `bash -n` passes on every script and the hook; `scripts/queue-checks.sh --help` prints its usage and exits 2 (not run in full: the main session's); `git grep -n 'check\.sh' -- ':!.scratch' ':!.agents'` finds nothing. No game fact changed, so no citation.

## Done when

- `scripts/check.sh` is gone, and `git grep -n 'check\.sh' -- ':!.scratch' ':!.agents'` finds nothing.
- `CLAUDE.md` says what a ticket runs, by what its diff touches, and what the main session runs once per queue.
- The CI workflow runs the full set.
- The scoped checks of this ticket's own diff pass. That is the hook and `bash -n` on any changed script; no Rust changes.
