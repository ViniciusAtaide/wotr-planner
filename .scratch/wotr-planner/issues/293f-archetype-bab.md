# Importer S1, 293f: Archetype::bab

Type: build (the importer, stage S1)
Status: resolved
Blocked by: 305
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [293](293-the-importer.md)'s stage S1, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof and the files it shares are under its heading in [research/293/slices.md](../research/293/slices.md). The decisions are in [research/293/decisions.md](../research/293/decisions.md) and in that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as written in `research/293/slices.md`, 293f. No game data moved, so no form proof.

- `Archetype::bab: Option<BabProgression>` (between `casting_ability` and `fort`); `Class::base_attack_bonus(level, archetype)` takes the archetype's when set. Callers: `walk/level.rs` `recompute_tables` (archetype looked up above the BAB line), `apps/mcp/src/describe.rs`, `game-check` `class_tables.rs` (classes `None`; every archetype now compared, the game's table being the archetype's when it names one, else its class's, as the saves).
- `DataIssue::BabProgressionSameAsClass` beside `SaveProgressionSameAsClass`. README line 67 names `bab: Some(Half)`. Struct literals: `nested.rs`, `alignment.rs`.
- Tests: `derived::an_archetypes_bab_replaces_its_classes` (Full class at 4: 4; `Some(Half)` archetype: 2), `check::an_archetype_bab_equal_to_its_classs_is_refused`. Both print in `cargo test -p rules --test engine -- --list`.
- Decision (builder, per slices.md): the game check compares the BAB of every archetype; Feyspeaker's "BAB differs" stays until 293l.
- Citation: the values come from the BAB formulas (Full = level, Half = level/2). Game: archetype Feyspeaker (`da69747aa3dd0044ebff5f3d701cdde3`) names `m_BaseAttackBonus` = BABLow (`0538081888b2d8c41893d25d098dee99`), the half progression.
- Checks: `cargo test -p rules -p mcp -p game-check -p seeds -p corpus` all ok; rules wasm build ok; fmt ok; game check exit 0.
