# Importer S1, 293b: applying changes, the guard's refusals, the write fixture's tree

Type: build (the importer, stage S1)
Status: resolved
Blocked by: 293a
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [293](293-the-importer.md)'s stage S1, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof and the files it shares are under its heading in [research/293/slices.md](../research/293/slices.md). The decisions are in [research/293/decisions.md](../research/293/decisions.md) and in that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as written in [research/293/slices.md](../research/293/slices.md) §293b.

- **What:** `tools/game-check/src/write/mod.rs`: `Change`, `Applied`, `WriteError` (`Planner`, `Missing`, `Span`, `ReadBack`, `Invalid`, `Io`, with `Display`), `entry_file(data, at)` and `apply(dir, changes, validate)`. `apply` groups by file, refuses a `planner/` or missing file, builds every new text before writing (a `SpanError` refuses all), writes, reads each file back as a `ron::Value` (only the written fields differ, and hold the written values), runs `validate`, and restores every written file on any failure.
- **Decisions of mine:** `entry_file` takes `&GameData` and `&DataAt` and covers every kind (the single files for the progression and skills), not only the three the slice names, so later slices need no second function. A file's insert anchor order comes from its folder (`classes/`, `prestige/`, `archetypes/`, `gear/`); another folder has none, so only a replace or removal works there. `Applied` lists the files whose text changed.
- **The tree:** `tests/fixture/data/` (progression, skills, warden, sentinel, two archetypes, six gear files mixing pretty and one-line, `planner/gear/test-trinket.ron`); `support::data_copy()`.
- **Tests** (`tests/fixture/write.rs`; no game data, so no game citation): `the_write_fixture_reads_as_game_data`, `applying_a_change_writes_only_its_field`, `a_planner_file_is_refused_and_nothing_is_written`, `a_failed_validation_restores_every_file`.
- **Proof:** the game check's stdout is byte-identical before and after (`cmp`).

**Read-back, second commit:** `ron::Value` reads every unit variant alike (a `bab: Half` file read back as `bab: Full`), so `read_back` (now public) also compares text: each field not written has the same span bytes before and after, and each written field's span equals the change's text up to white space and trailing commas. `read_back_tells_unit_variants_apart` plants a wrong variant, a change in another field, and the value in an unrelated field; all are refused.
