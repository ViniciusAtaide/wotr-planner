# Importer S1, 293g: class tables: BAB, saves and the save exceptions

Type: build (the importer, stage S1)
Status: resolved
Blocked by: 293e, 293f
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [293](293-the-importer.md)'s stage S1, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof and the files it shares are under its heading in [research/293/slices.md](../research/293/slices.md). The decisions are in [research/293/decisions.md](../research/293/decisions.md) and in that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as written in `research/293/slices.md`, 293g. No data file changed (293l runs it); no form proof.

- `src/write/tables.rs` (new): `bab_progression`, `save_progression` (the variant whose formula gives `table[1..=levels]`; none when no formula does) and `changes`. Classes in key order: a BAB or save is written only when the effective value (`Class::base_attack_bonus`, `save_bonus`) differs from the game's table at some level; a save with a formula drops its `save_exceptions` (a save no formula gives keeps them; none left removes the field). Archetypes in key order, against the class as it will be written: `Some(p)` only where the game's table is not the class's, and a field equal to the class's becomes `None` (the data check refuses it). A table with no formula, or no table, is listed as unexpressed and its finding stays.
- `class_tables.rs`: `bonuses(dump, json, field)` lifted from `Row::progression`, `game_skills` and `SAVES` `pub(crate)`. `write/mod.rs`: `WriteArea::ClassTables` (`--write class-tables`); `WriteArea::kind` became `kinds` (this area has four) and `main.rs` filters by it (not in the section's list; needed for the dry run's listing).
- Fixture: `TestBabLow` (`...56`), `StrayArchetype` names it; `areas.rs` asserts `BAB differs` on `archetype warden/stray`.
- Tests (all print in `cargo test -p game-check -- --list`): `write::writing_class_tables_brings_bab_and_saves_to_the_game`, `write::a_save_table_no_formula_gives_is_none`, `write::a_class_table_run_is_idempotent`; `areas::class_tables_find_…` takes the new finding. Expected values come from the fixture's invented dump (warden's `m_ReflexSave` = TestSavesHigh, stoic's null `m_WillSave` = the class's TestSavesLow, stray's `m_BaseAttackBonus` = TestBabLow); the formulas are the game's (Full = L, Half = L/2, good base save 2+L/2, poor L/3).
- Dry run on main's data (`--write class-tables --dry-run`): 18 fields in 14 files. 9 classes lose `save_exceptions` (paladin, hunter, shifter, warpriest, arcane-trickster, loremaster, mystic-theurge, student-of-war, winter-witch), `stalwart-defender` gets `fort: Good, reflex: Poor, will: Good`; `cleric/priest-of-balance` fort Poor, `kineticist/dark-elementalist` fort Poor, reflex Poor, will Good, `monk/traditional-monk` will Good, `druid/feyspeaker` bab Half. No archetype of the four other classes is written. Nothing unexpressed.
- Checks: `cargo test -p game-check` 83 passed; clippy `-p game-check --all-targets -D warnings` clean; `cargo fmt --all --check` ok; the check's report on main's data is byte-identical to the base's but for its timing line (`cmp` differs only on `game-check: 0.8 s` / `0.7 s`).
