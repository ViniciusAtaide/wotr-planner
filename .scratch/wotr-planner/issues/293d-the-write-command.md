# Importer S1, 293d: the command, --dry-run and the change log

Type: build (the importer, stage S1)
Status: resolved
Blocked by: 293c
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [293](293-the-importer.md)'s stage S1, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof and the files it shares are under its heading in [research/293/slices.md](../research/293/slices.md). The decisions are in [research/293/decisions.md](../research/293/decisions.md) and in that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as written in [research/293/slices.md](../research/293/slices.md) §293d.

- **What:** `src/main.rs`: `game-check --write <area>[,<area>…] [--dry-run] [--log <file>] [--data <dir>] [--binz <file>]` (doc header and `USAGE`): the check before, `write::run` with `seeds::check_game_data_files` as the validate step, per area the fields written and the files touched, the unexpressed list, "nothing to write"; exit 0 written or nothing to write, 2 refused (an unknown area is usage, exit 2). `src/write/log.rs`: `tsv(changes, before, after)`: a header and one row per change in entry then field order, `findings removed` = the report groups (`area / kind`) that named the entry before and no longer after, `fact` `-` until 293e. `src/write/mod.rs`: `run(dump, dir, areas, dry_run, validate)`, `plan` (the guards and the span writer, nothing written), `WriteArea::{parse, name, kind}`.
- **Decisions of mine:** `--write` takes `items` only; `class-tables` joins when its slice lands. `--log` with `--dry-run` is a usage error (a dry run has no report after, so its findings column would be empty). `findings removed` is read from the report's text, not a typed report.
- **Record of 293c's addition:** `Out::skip_reasons` in `src/report.rs`, outside 293c's file list, was added in its review fix so `changes` can see why the check skipped a bonus.
- **Tests** (`tests/fixture/write.rs`; the fixture's own invented game): `two_runs_on_two_copies_give_the_same_files_and_logs`, `the_log_names_the_findings_each_change_removed`, `a_dry_run_writes_nothing`.
- **Proof:** the game check's stdout is byte-identical before and after (`cmp`). By hand, `game-check --write items --dry-run` on main's data printed 4 fields in 4 files (belt-of-the-venerated-champion, dirge-for-sarkoris, half-plate-of-vigor, trusted-friend), wrote nothing, exit 0.
