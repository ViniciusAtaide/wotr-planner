# Importer S1, 293l: the class-tables run

Type: build (the importer, stage S1)
Status: resolved
Blocked by: 293g, 293h, 293i, 293j, 293k, 312
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [293](293-the-importer.md)'s stage S1, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof and the files it shares are under its heading in [research/293/slices.md](../research/293/slices.md). The decisions are in [research/293/decisions.md](../research/293/decisions.md) and in that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as written in `slices.md`, 293l. `game-check --write class-tables` wrote 29 fields in 23 files (10 class and prestige files, 13 archetypes), removing 62 findings; the log (the lane's scratchpad `293l/class-tables.log`) has 29 rows: `save_exceptions` 9, `class_skills` 9, `fort` 3, `will` 3, `reflex` 2, `skill_points` 2, `bab` 1. A second `--write class-tables` writes 0 files. The user's skill points and class skills were written under their `applied 261`/`applied 262` registry lines; nothing went "to the user". Note: `libs/game-data/notes/293-class-tables.md`.

- **Game check:** exit 0; class tables 235 compared, 234 agree, 1 differs (the Stalwart Defender's `CheckIntimidate`, now `known.txt` under ticket 312). Sections 261 and 262 of `known.txt` are deleted.
- **Two fixes from 293h's review** (`tools/game-check/src/write/tables.rs`): a class whose blueprint has no `SkillPoints`, and a skill-points value (a class's, or an archetype's total) out of `u8`'s range, are listed as unexpressed. Tests `a_class_whose_blueprint_has_no_skill_points_is_listed_not_planned_to_zero` and `a_skill_points_value_out_of_range_is_listed_not_skipped` (`tools/game-check/tests/fixture/write.rs`); both failed before the fix (nothing was listed).
- **Tests un-ignored and passing:** `class_tables.rs` `the_save_tables_are_the_games`, `the_feyspeakers_bab_is_low`, `skill_points_are_the_games`, `class_skills_are_the_games`; `archetype_saves.rs` `four_archetypes_replace_a_save_progression`. Their expected values are the game's (blueprint names and GUIDs cited in each file and in the note); all five failed before the write.
- **Census (form proof, base recorded in this lane before any edit, `dea972c9`; classifier `293l/classify`, std-only Rust):** numbers.txt 274 Builds / 2,537 lines differ, sheets.txt 4 / 55, slots.txt 2 / 5, builds.tsv 4 / 4; each differing Build holds a class or archetype with a change-log row, **0 unattributed**. By the changed fields the Build holds (Builds in numbers.txt): save_exceptions only 208, fort 16, class_skills 11, class_skills+save_exceptions 11, fort+reflex+will 9, will 8, bab 3, class_skills+three saves 3, save_exceptions+skill_points 3, skill_points 2. sheets.txt: bab 2 (the Feyspeaker, two Builds: Outflank for Improved Critical at level 13), skill_points 2 (Defender of the Hearth: one more skill). slots.txt: bab 2. builds.tsv: bab 2, skill_points 2. Old links: 1,998 the same, 0 through an heir, 2 refused, 0 changed; the 2 refused are the Feyspeaker Builds (Half BAB leaves Improved Critical's BAB +8 unmet at level 13).
- **Gold builds:** none changed (the gold tests pass unedited).
- **Decisions of mine outside the listed files:** (1) `tools/corpus/tests/links/306-refused.txt` lists the 2 refused Feyspeaker links (the S2 link test would otherwise fail on them); (2) `libs/seeds/tests/game_data/support.rs` `without_unproficient_picks` also drops a feat that fails only on base attack bonus, since the Feyspeaker's old link in `links-before-the-splits.tsv` now needs it. Reject either if the main session prefers another form.
- **Not done:** `research/skills-from-user.md`'s 15 rows (slices.md lists it, the brief did not).
- **Checks:** `cargo test -p seeds -p corpus` (seeds 547 passed, corpus links 5, all ok), `cargo test -p game-check` (108 passed), `check-data` (93 gear, 26 classes, 196 archetypes), the game check exits 0, `cargo fmt --all --check` and `scripts/test-modules.sh` (OK) pass.
