# Importer S1, 293h: class tables: skill points and class skills

Type: build (the importer, stage S1)
Status: resolved
Blocked by: 293g
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [293](293-the-importer.md)'s stage S1, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof and the files it shares are under its heading in [research/293/slices.md](../research/293/slices.md). The decisions are in [research/293/decisions.md](../research/293/decisions.md) and in that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as written in `research/293/slices.md`, 293h, with 293g's review folded in. No data file changed (293l runs it); no form proof.

- `write/tables.rs`: class `skill_points` and `class_skills` (the game's `SkillPoints`, and `ClassSkills` through `game_skill`; a stat with no planner skill, `CheckIntimidate`, is listed as unexpressed and the rest is written); archetype `skill_points` (class target + `AddSkillPoints`, `Some` only when it differs from the class's) and `class_skills` (`Some(game list)` when `ReplaceClassSkills` and it differs from the class's, else removed). Kinds are the check's `... (the user's facts)`, so `run` holds them unless an `applied` line says otherwise. `WriteArea::kinds` names them.
- Review fix: `tables::changes` takes the registry and plans the archetypes against the class as `decide` leaves it (a held class change keeps the old value in the target), so a held class value is never assumed by an archetype. Consequence worth knowing: with the class's skill points held, the Stoic's 5 already equals the held 4 + 1, so it isn't a change.
- The `TODO(293h)` in `facts.rs` is done: `Facts::kinds` and a test that every non-`*` kind of `user-facts.txt` appears as a literal in its area's check source.
- Fixture: `WardenClass`'s `ClassSkills` gains `CheckIntimidate`; `areas.rs` takes the new game value.
- Tests (print in `--list`): `the_users_class_tables_are_held_without_an_applied_line`, `applied_class_tables_are_written`, `save_exceptions_stay_for_a_save_no_formula_gives`, `an_unexpressed_class_table_is_left_and_listed`, `an_archetype_override_the_class_now_equals_is_removed`, `a_held_class_save_is_not_assumed_by_its_archetypes`, `every_change_is_of_a_kind_its_area_names` (the dry-run's kinds filter), `every_kind_in_the_registry_is_one_the_check_emits`; the two 293g tests that ran on default facts now hold the user's kinds. Expected values come from the fixture's invented dump (Warden `SkillPoints` 3, `ClassSkills` Athletics/Stealth/Intimidate; Stoic `AddSkillPoints` 1).
- Dry run on main's data with the embedded registry: 33 fields in 27 files (18 BAB/save, 2 skill points, 13 class skills), every user fact `applied`, none held; the one unexpressed item is `stalwart-defender`'s `CheckIntimidate`.
- Checks: `cargo test -p game-check` 91 passed; clippy `-p game-check --all-targets -D warnings` clean; `cargo fmt --all --check` ok; the check's report on main's data byte-identical to the base's (`cmp`).
