# Alchemist rows and discoveries

Type: build (data)
Status: open
Blocked by: [254](254-a-features-steps-follow-the-class-level.md)
Model: Opus to build: bomb ranks, discoveries' prerequisites and level 20.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half A `build_sheets.rs`, `borrowed_selections.rs`, `archetype_losses.ron`) differ on `alchemist.ron`:
- **Level 20** (`AlchemistProgression` efd55ff9be2fda34981f5b9c83afe4f1): Grand Discovery (`GrandDiscoverySelection` 2729af328ab46274394cedc3582d6e98) and two `DiscoverySelection` (cd86c437488386f438dcc9ae727ea2a6) picks; the Vivisectionist (`VivisectionistArchetype` 68cbcd9fbf1fb1d489562f829bb97e38) removes one, keeping one.
- **Bombs and Poison Resistance are ranks:** the game grants `AlchemistBombsFeature` (c59b2f256f5a70a4d896568658315b7d) again at each odd level and `PoisonResistance` (c9022272c87bd66429176ce5c597989c) at its steps; the planner grants each once. The Incense Synthesizer (`IncenseSynthesizerArchetype` 6faf49766d8156c419d08f556a40374f) removes the bomb die at 3, 7, 11, 15, 19 (its summary says bombs stay at 1d6) and its Incense Fog row is the planner's; the Reanimator's die stops at 5, 11, 15.
- **Discoveries' prerequisites:** the bomb discoveries need `AlchemistBombsFeature`; the mutagens need `MutagenFeature` or the Mutation Warrior's (`MutationWarriorArchetype`, with fighter levels); some exclude the Metamorph, the Incense Synthesizer or the Vivisectionist; the Underground Chemist's own discovery choice is [276](276-rogue-and-slayer-talents.md)'s.
- **The Metamorph** (`MetamorphArchetype` ac242ddeaa05af449bfc5230231fab84): its class skills feature and Shapechanger at 4; Adaptive Physiology's steps ([254](254-a-features-steps-follow-the-class-level.md)'s pass); the Reanimator's corpse studying; the Chirurgeon's Skill Focus is [255](255-a-parametrised-feat-at-one-value.md)'s.

## Findings (the game check, `known.txt` section 280)

| The check's group (area / kind) | Findings |
| --- | --- |
| progression rows / a further rank in game only | 57 |
| prerequisites / missing in planner | 34 |
| progression rows / in game only | 17 |
| progression rows / a further rank in planner only | 5 |
| prerequisites / not in game | 1 |
| progression rows / in planner only | 1 |
| **all** | **115** |

The 30 entries: `archetype alchemist/chirurgeon`, `archetype alchemist/grenadier`, `archetype alchemist/incense-synthesizer`, `archetype alchemist/metamorph`, `archetype alchemist/preservationist`, `archetype alchemist/reanimator`, `archetype alchemist/vivisectionist`, `class alchemist`, `selection option alchemist/discovery/acid-bomb`, `selection option alchemist/discovery/blinding-bombs`, `selection option alchemist/discovery/breath-weapon-bomb`, `selection option alchemist/discovery/choking-bomb`, `selection option alchemist/discovery/cognatogen`, `selection option alchemist/discovery/cursed-bomb`, `selection option alchemist/discovery/dispelling-bombs`, `selection option alchemist/discovery/explosive-bombs`, `selection option alchemist/discovery/fast-bombs`, `selection option alchemist/discovery/feral-mutagen`, `selection option alchemist/discovery/feral-wings`, `selection option alchemist/discovery/force-bombs`, `selection option alchemist/discovery/frost-bomb`, `selection option alchemist/discovery/grand-mutagen`, `selection option alchemist/discovery/greater-mutagen`, `selection option alchemist/discovery/holy-bombs`, `selection option alchemist/discovery/mummification`, `selection option alchemist/discovery/nauseating-flesh`, `selection option alchemist/discovery/precise-bomb`, `selection option alchemist/discovery/preserve-organs`, `selection option alchemist/discovery/shock-bomb`, `selection option alchemist/discovery/tanglefoot-bombs`.

The report lists each finding under its group, marked `[known: 280]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The rows and prerequisites become the game's, bombs and Poison Resistance as ranks (243's model); `archetype_losses.ron`'s incense-synthesizer and reanimator entries follow; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `build_sheets.rs`: `the_legend_sheet_runs_to_level_40_and_marks_the_picks_the_switch_takes_away` (the alchemist 20 row : two discovery slots, shown when filled).
- `borrowed_selections.rs`: `a_vivisectionist_picks_its_discoveries_and_talents_and_advanced_ones_from_10` (one discovery slot left at 20).
- `scaling_features.rs`: `the_alchemists_reanimator_and_metamorph_step`.
- `archetype_losses.rs` (incense-synthesizer, reanimator, vivisectionist).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 280 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
