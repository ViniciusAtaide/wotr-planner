# Importer S1, 293c: the items target and the library run

Type: build (the importer, stage S1)
Status: resolved
Blocked by: 293b
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [293](293-the-importer.md)'s stage S1, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof and the files it shares are under its heading in [research/293/slices.md](../research/293/slices.md). The decisions are in [research/293/decisions.md](../research/293/decisions.md) and in that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as written in [research/293/slices.md](../research/293/slices.md) §293c.

- **What:** `areas/items.rs`: the game side of `compare` is `game_bonuses(ctx, out, json)`, and `Bonus`, `stacked`, `target` and `bonus_type` are `pub(crate)`; `compare`'s output is unchanged (the fixture's `items_*` tests and the real report, byte-identical, prove it). `areas/mod.rs`: `game_skill`, the inverse of `skill_stat`. `write/items.rs`: `changes(ctx)` (per non-usable gear item with a blueprint, in key order: `stacked(game_bonuses)` against `item.stats()` as sorted lists; equal writes nothing; else `components` is the game's bonuses through `gear_stat`, sorted, `kind: "bonuses"`) and `gear_stat`, the inverse of `target` and `bonus_type` (ability, saves, initiative, skills, `SpellDC`/`SpellDC <school>`, caster level, spell penetration, concentration; a type by its variant name, `Untyped` as `Descriptor::Untyped`). A bonus with no inverse leaves the item unwritten and listed (`Unexpressed`). `write/mod.rs`: `WriteArea`, `Outcome`, `run(dump, dir, areas, validate)`.
- **Decisions of mine:** `WriteArea` has `Items` only; the slice's `ClassTables` variant arrives with the slice that writes it (a variant that does nothing would be dead). `Outcome` also carries `applied`. A game bonus whose amount doesn't fit an `i8` or whose type or school has no variant is unexpressed.
- **Tests** (`tests/fixture/write.rs`; the expected values are the fixture's own blueprints, an invented game): `writing_items_leaves_only_the_slot_finding`, `a_second_items_run_writes_nothing`.
- **Proof:** the game check's stdout is byte-identical before and after (`cmp`). No real-data run yet.

**Review fixes, second commit:** `changes` leaves an item unwritten and lists it when the check skipped a bonus it couldn't read (a value worked out in play, a wearer's fact or enchantment not in the dump, an empty reference): writing the rest would delete the planner's bonus for it. Only the skips that name a number the planner doesn't count (AC) or a component of another kind are benign. `Unexpressed.what` lists every bonus with no inverse, not the first. `Out::skip_reasons` (in `report.rs`) gives the reasons. Tests: `an_item_with_a_bonus_the_importer_cannot_read_or_name_stays_unwritten_and_listed` (extra blueprints made in the test), `the_inverse_of_the_check_s_targets_names_each_stat_the_planner_counts` (the six abilities, three saves, initiative, SpellDC all and one school, caster level, spell penetration, concentration, the 11 skills, and the names with no inverse).
