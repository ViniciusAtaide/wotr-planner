# Remove the regression checks; keep the engine's self-checks

Type: build
Status: resolved
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

## Answer

**Decided** (main session, 2026-11-14, on the research report's open questions; the ticket's own decisions are the user's, in [234](234-game-blueprints-as-a-source.md)):

1. **The two shared files:** this ticket edits its own parts of `CLAUDE.md` (Parallel agents, Checks, Behaviour checks) and `libs/game-data/README.md` (the proof and snapshot text). [236](236-the-game-sources-in-the-repo.md) adds the "Game sources" section and the sources line; it merges first, and this branch merges `main` before its check.
2. **`rules`' `unchecked` feature goes:** it existed only so side A of a data proof could record data that fails `check`.
3. **The `allows` battery keeps only what was asserted:** each start and level option's status against `allows` of its edit. Its other edits (each first ascension, each held choice's withdrawal, Remove level, a name) were recorded for the comparison and asserted nowhere, so they go. A possible follow-up, not filed: new invariants for them (a first-ascension option's status against `allows`; a withdrawal's `allows` against `removable`, not yet known to hold).
4. **The shims for older crates go where the direct code makes the same Build** (`with_faith`, `faithful`, the mythic ability pick, the first ascension, the deities' keys, all through serde before); `fnv` seeding and the generator's quirks stay, since they define the corpus. Checked by a throwaway test, not committed, before and after: **2,163 Builds, names hash `50a7a1336a9f14d0`, Debug hash `0f53b5145c6d5a56`, on both sides.**
5. **`ticket-builder.md`'s "The guide wins over the game"** now says the game wins on rules (map Notes) and the guide supplies summaries and links; its step 5 checks against the game.
6. **The sweep's exclusions** are named as unrelated: `.agents/` (a third-party skill on insta snapshot testing) and `Cargo.lock` (`wasi-snapshot-preview1`); `.scratch/` and `libs/game-data/notes/` are history (see 8).
7. **The long self-check runs are a heavy run,** once per queue by the main session: in `CLAUDE.md`'s "Parallel agents" and "Behaviour checks" and the `ticket-queue` skill's closing step.
8. **`libs/game-data/notes/204-wizard-guide-link.md` stays as it is:** data notes are history, like tickets.

The builder's own decisions: the crate is renamed `tools/corpus` (library only), `links.rs` becomes `bench.rs` (only the benchmark's reader), the self-checks live in its `tests/self_checks.rs`, and the suite's sample is `builds::sample` (the bench Builds cut to 10 levels, as seeds' agreement sample, and ticket 55's extra-slot Builds for the shortcut check).

### Built (2026-11-14)

- **Removed:** `scripts/proof.sh`; `libs/seeds/tests/game_data/snapshots.rs`, its `mod` line and `snapshots/` (52 files); from the harness `build.rs`, `main.rs`, `compare.rs`, `dump.rs`, `explain.rs`, `fingerprint.rs` and the tests `round_trip.rs`, `dump.rs`, `explain.rs`; `rules`' `unchecked` feature and its `From<GameData> for CheckedGameData`.
- **`tools/corpus`** (was `tools/proof`): `builds::all` (the corpus, unchanged), `builds::sample`, `engine()` (the checked data of `seeds::game_data_dir`, so `$GAME_DATA_DIR` is honoured), `threads(var)`, `in_parallel`. `generate` and `bench` are private. `tests/agreement.rs` (moved, `every_picker_agrees_with_the_walk_on_the_corpus`, ignored) and `tests/self_checks.rs` (new): the withdrawals against `removable`, the start and level statuses against `allows`, and ticket 48's shortcut on the `SHORTCUT_CHECKED` Builds, as the record checked them, now with the Build's name in each failure.
- **`libs/seeds`' `agreement.rs`** stays: its short run in the suite, the whole bench Builds and the timing ignored; only its docs and the tsv's path changed.
- **Rewritten:** `CLAUDE.md` (Parallel agents, Checks, Behaviour checks), `.claude/agents/ticket-builder.md`, `.claude/agents/ticket-reviewer.md` (a step: each expected value against its citation in the game), `.claude/skills/ticket-queue/SKILL.md`, `CODING_STANDARDS.md` ("A status is behaviour"), `README.md` (a `corpus` row), `libs/game-data/README.md` ("Checking against the game" in place of "The snapshot check"; the gold standard's text), `gold.rs`'s comments (a gain the game contradicts goes to the user), and the comments naming the proof in `libs/seeds` and `libs/rules`.
- **`scripts/check.sh`** is unchanged: the snapshot test ran inside its `cargo test --workspace` step, which now runs the corpus's short run.
- **Tests:** `the_self_checks_hold_on_a_sample_of_the_corpus` (about 1 s in a debug build; asserts held choices of more than three kinds, some controls and some shortcut options were checked), `the_self_checks_hold_on_the_whole_corpus` (ignored) and `every_picker_agrees_with_the_walk_on_the_corpus` (ignored), each listed by `cargo test -p corpus -- --list`; no `snapshots` test is left in `cargo test -p seeds --test game_data -- --list`.
- **No engine output moves:** no `rules` or data code changed but the removed feature, and the corpus hashes above match.
- **The sweep:** `git grep -i -E 'snapshot|proof\.sh' -- ':!.scratch' ':!.agents' ':!Cargo.lock' ':!libs/game-data/notes'` finds nothing.
- **For the queue's checks:** the long runs, `cargo test --release -p corpus -- --ignored` and `cargo test --release -p seeds --test game_data agreement -- --ignored`, were not run here (a heavy run, the main session's). The whole-corpus self-checks have not run since the proof's last full record.
- **Not done:** sharing one bench reader and one 10-level sample between `corpus` and the seeds tests (they read the same tsv), and a `rules::self_checks` module beside `rules::agreement` that the test-world tests could drive too: both found by the simplify pass and left for a later ticket.
