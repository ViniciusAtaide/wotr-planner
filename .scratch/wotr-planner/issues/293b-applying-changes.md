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

**Review fixes, third commit:**
- A restore that fails is collected: `WriteError::RestoreFailed { cause, files }` names every file left written.
- Each file is written to `.<name>.tmp` in its folder and renamed over the original (`atomic_write`); the `apply` doc says the set of files isn't atomic and git is the recovery.
- `NotOwned { file, field }`: in the plan phase a change to a field outside the folder's owned list (design §1.2's table: classes and prestige, archetypes, gear's `components`) is refused before any write; a folder with no list (the single files, every other kind) owns nothing yet.
- `read_back(file, before, after, change)` is per change and also compares the bytes between the fields: the texts equal with the span cut out (for an insert or removal, with the separators the writer adds or drops), so a dropped comment, comma or white space fails. `apply` also compares the file as read from the disk with the text built. Changes to one file apply in order; of two to one field the later wins.
- `apply_with` takes the write function, the seam for the Io and ReadBack tests. `entry_file`'s doc says it doesn't detect `planner/` entries.
- Tests: `read_back_keeps_the_bytes_between_the_fields`, `a_field_the_importer_does_not_own_is_refused_before_any_write`, `a_write_that_lands_wrong_is_a_read_back_failure_and_every_file_is_restored`, `a_write_that_fails_is_an_io_error_and_every_file_is_restored`, `a_restore_that_fails_names_the_files_left_written` (unix: the folder turns read-only), `two_changes_to_one_field_apply_in_order_and_the_later_wins`, `a_write_that_parses_but_breaks_a_data_rule_is_rejected_by_the_real_check` (`GameData::check` through the seeds reader; the Stoic's `fort: Good` equals its class's: `SaveProgressionSameAsClass`).
