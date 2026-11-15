# Paladin rows and mercies

Type: build (data)
Status: open
Blocked by: [254](254-a-features-steps-follow-the-class-level.md)
Model: Opus to build: steps become the game's extra-use features.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half B `mercies_and_exploits.rs`, `scaling_features.rs`; half A `archetype_losses.ron`) differ on `paladin.ron`:
- **Smite Evil** (`SmiteEvilFeature` 3a6db57fce75b0244a6a5819528ddf26) is granted once; its uses at 4, 7, 10, 13, 16, 19 are `SmiteEvilAdditionalUse` (0f5c99ffb9c084545bbbe960b825d137), a feature of its own (`PaladinProgression` fd325cbba872e5f40b618970678db002); the planner grants Smite Evil again. The Hospitaler (`HospitalerArchetype` fcdb27e1a6cef834ea1d7fab7c6e2b61) removes only the uses at 4, 10, 16, keeps the class's Smite Evil at 1 and the uses at 7, 13, 19, and keeps the Divine Bond selection at 5. The Divine Scion, Martyr, Stonelord and Tortured Crusader remove all.
- **The Divine Hunter** (paladin, `DivineHunterArchetype` fec08c1a3187da549abd6b85f27e4432) removes the mercy (`SelectionMercy` 02b187038a8dce545bb34bbfb346428d) at 6 only: it keeps 18; its bond's +2 to +6 are the game's features.
- **Other rows:** the auras (courage, faith, resolve, righteousness) at the game's levels and the Martyr's own; the Tortured Crusader's All Is Darkness steps; the Warrior of the Holy Light's Power of Faith; the Stonelord's Stonebane and Heartstone; the Martyr's Inspire Courage (ranks).
- **Mercies:** three level-3 mercies (Fatigued, Shaken, Sickened) ask paladin 3, which the game doesn't.

## Findings (the game check, `known.txt` section 274)

| The check's group (area / kind) | Findings |
| --- | --- |
| progression rows / in game only | 49 |
| progression rows / a further rank in planner only | 37 |
| progression rows / in planner only | 7 |
| progression rows / a further rank in game only | 4 |
| prerequisites / not in game | 3 |
| **all** | **100** |

The 11 entries: `archetype paladin/divine-guardian`, `archetype paladin/divine-hunter-paladin`, `archetype paladin/hospitaler`, `archetype paladin/martyr`, `archetype paladin/stonelord`, `archetype paladin/tortured-crusader`, `archetype paladin/warrior-of-the-holy-light`, `class paladin`, `selection option paladin/mercy/fatigued`, `selection option paladin/mercy/shaken`, `selection option paladin/mercy/sickened`.

The report lists each finding under its group, marked `[known: 274]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The rows and mercies become the game's with [254](254-a-features-steps-follow-the-class-level.md)'s model; `archetype_losses.ron`'s paladin entries follow; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `mercies_and_exploits.rs`: `every_paladin_archetype_picks_a_mercy_at_its_tables_levels` (the Divine Hunter at 3, 9, 12, 15, 18).
- `scaling_features.rs`: `the_paladins_smite_evil_steps_every_three_levels_after_4` (the Hospitaler keeps the class's Smite Evil and the uses at 7, 13, 19), `the_paladin_archetypes_step_the_features_the_guide_gives_steps_to`.
- `archetype_losses.rs` (`archetype_losses.ron`'s divine-hunter-paladin and hospitaler entries).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 274 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
