# Remove the regression checks; keep the engine's self-checks

Type: build
Status: open
Blocked by: none
Model: Opus to build: it moves the self-checks out of `tools/proof` and rewrites the project's checking rules.
Map: [WotR Build Planner](../map.md)

## Problem

Decided in [234](234-game-blueprints-as-a-source.md) (user, 2026-11-14): the game is the source of truth for rules, so a check against the game replaces the checks that compared the engine with its own earlier output.
- The user: "I would argue we don't even need proof.sh and check.sh as they are today anymore". Asked which checks remain, they chose "c".
- On timing: "the queue stopped so c remove them now".

On the oracle: "if they contradict the game rules we go with the game rules. assume the game code works always."

## Change

**Remove** the two regression checks:
- **The snapshot check:** `libs/seeds/tests/game_data/snapshots.rs`, its `mod` line and the `snapshots/` folder (the class and bench Builds' recorded sheets).
- **The two-sided proof:** `scripts/proof.sh`, and in `tools/proof` the recording of two sides, `compare`, `explain`, the dumps and the fingerprints that serve only the comparison (`fingerprint.rs`, `dump.rs`, `compare.rs`, `explain.rs`, as far as nothing kept needs them).

**Keep**, moved out of `tools/proof` into tests or a crate of their own: these check the engine against itself, not against an earlier version.
- **The agreement sweeps** (`libs/seeds/tests/game_data/agreement.rs`, ticket 150): every picker agrees with the walk. The suite's short run stays. The whole bench Builds and the corpus's ~2,000 Builds stay under `--ignored`.
- **The record's consistency and shortcut checks, and the `allows` battery**, as tests over the corpus. The long run is under `--ignored`.
- **The corpus:** the bench links (`bench-links.tsv`) and the generated Builds (`generate.rs`, `links.rs`, `builds/`), which the benchmarks and the sweeps read.

`scripts/check.sh` keeps its steps: fmt, css-classes, clippy, the workspace tests, the rules wasm build and `check-data`, with `--full` and `--ui` as they are. Only the snapshot test leaves it, with the tests.

**Rewrite** the rules that name the removed checks:
- **`CLAUDE.md`, "Behaviour checks":**
  - A change to `rules` or `libs/game-data/` is checked against the game: each changed fact has a real-data test whose expected value comes from the game, cited by blueprint name and GUID or by the decompiled code's file and method.
  - The engine's self-checks never outrank the game. When one disagrees with the game, the engine is fixed.
  - The gold builds stay.
  - The game check (ticket 238) runs by hand once it exists.
- **The tooling that names them:** the `ticket-queue` skill, `.claude/agents/ticket-builder.md` (its step 5, "Snapshots", and what it reports) and `.claude/agents/ticket-reviewer.md`.
- **The documentation that names them:** `CODING_STANDARDS.md` ("A status is behaviour" cites the proof's hashing), `README.md` and `libs/game-data/README.md` (its two tiers).
- **The gold builds:** `gold.rs`'s comments, where they name the snapshot check. A gold build's gain that contradicts the game goes to the user.

## Done when

- The snapshot check and the two-sided proof are gone, and nothing names them: `git grep -i -E 'snapshot|proof\.sh'` finds only history in `.scratch/` and this ticket's own notes.
- The self-checks run from their new home, their short runs in `scripts/check.sh` and the long ones under `--ignored`, each listed by `cargo test … -- --list`.
- `CLAUDE.md`, the skill and the agents describe checking against the game.
- `scripts/check.sh` passes.
