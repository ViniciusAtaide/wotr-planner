# Importer S1, 293j: the class tables' real-data tests, ignored

Type: build (the importer, stage S1)
Status: resolved
Blocked by: 293i
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [293](293-the-importer.md)'s stage S1, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof and the files it shares are under its heading in [research/293/slices.md](../research/293/slices.md). The decisions are in [research/293/decisions.md](../research/293/decisions.md) and in that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as written in `slices.md`, 293j: `libs/seeds/tests/game_data/class_tables.rs` (new, `mod class_tables;` in `main.rs`), four tests, each `#[ignore = "waits on ticket 293l"]` (293l writes the data and removes the ignores).

- `the_save_tables_are_the_games`: 17 (entry, save, level) rows, saves through `Class::save_bonus` (prestige formula for prestige classes), each row citing its class or archetype blueprint and GUID. Game values: SavesHigh ff4662bd... (12 at 20), SavesLow dc0c7c1a... (2 at 7), SavesPrestigeHigh 1f309006... and SavesPrestigeLow dc5257e1... (read with `game-check show`).
- `the_feyspeakers_bab_is_low`: druid 2 at level 3 (BABMedium 4c936de4...), Feyspeaker 1 (BABLow 05380888..., FeyspeakerArchetype da69747a...).
- `skill_points_are_the_games`: Arcane Trickster 3; Defender of the Hearth 3 (fighter's 2 plus AddSkillPoints 1).
- `class_skills_are_the_games`: the 13 entries of 262, compared as the effective list (the archetype's own list, else its class's), so a redundant field stays allowed. The Metamorph, Sohei, Empyreal and Sylvan Sorcerer have ReplaceClassSkills false in the game: their expected list is the class's, and they already pass; the other 9 fail today. The Stalwart's expectation omits Intimidate (no skill key; gap G4, ticket 312).

Decision (mine, from the section's table-driven wording): the first failing row panics, so each test shows its first difference only; 293l's run is what proves all rows. `archetype_saves.rs` is left for 293l.

Proof: the four names print in `cargo test -p seeds --test game_data -- --list`; run with `--ignored` all four fail, each on today's data (paladin Will at 20 is 20 not 12; Feyspeaker BAB 2 not 1; Arcane Trickster 4 not 3; Magic Deceiver lacks Persuasion).
