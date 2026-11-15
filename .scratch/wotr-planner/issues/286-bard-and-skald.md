# Bard and skald rows

Type: build (data)
Status: superseded by 293 (the user, 2026-11-15): the importer writes its mechanical data; what it can't express becomes a gap ticket
Blocked by: [254](254-a-features-steps-follow-the-class-level.md)
Model: Opus to build: performances' ranks and the Battle Scion's selection.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half B `skald_rage_powers.rs`, `scaling_features.rs`; half A `archetype_losses.ron`) differ on `bard.ron` and `skald.ron`:
- **Performances are ranks:** the game grants Inspire Courage (`InspireCourageFeature` acb4df34b25ca9043a6aba1a4c92bc69) and Inspire Competence (`InspireCompetenceFeature` 6d3fcfab6d935754c918eb0e004b5ef7) again at their steps (`BardProgression` 8127f5ff40f5b484b8be98609358b9d2), the Beast Tamer's Inspire Ferocity and the Mind Games likewise; the planner grants each once. The archetypes' own scaling features (Devastating Aria at 2, 4, 8, 12, 16, 20; Haunting Refrain, Deadly Vibrato, Insightful Contemplation, Fiendish Maw, Battle Prowess) follow [254](254-a-features-steps-follow-the-class-level.md).
- **Rows at 1:** the bard's cantrips (`BardCantripsFeature` 4f422e8490ec7d94592a8069cce47f98) and Detect Magic for the class and its archetypes.
- **Archetypes' removals:** the Beast Tamer (`BeastTamerArchetype` 912b0495d9485124bb2c0e4aa6ac2efd) removes Well-Versed (`BardWellVersed` 8f4060852a4c8604290037365155662f) at 2; the Flame Dancer (`FlameDancerArchetype` e7914f2adcdb8fc46af5b65d1e06c539) and the Thundercaller (`ThundercallerArchetype` 3c67fb6458752a1419fb8cd4efaf8eaa) remove Frightening Tune (`FrighteningTuneFeature` cfd8940869a304f4aa9077415f93febe) at 14; the Battle Scion, the Herald of the Horn and the Inciter keep Raging Song (`RagingSong` 334cb75aa673d1d4bb279761c2ef5cf1) at 1; the Demon Dancer's greater fiend totem at 9, not 10.
- **The Battle Scion** (`BattleScionArchetype` 71d41dd6309ea994886908c34c87ccd9): its selection is Battle Prowess (`BattleProwessSelection` 29b480a26a88f9e47a10d8c9fab84ee6), 21 rage powers (not the Celestial or Daemon totems, Rolling Dodge, Crippling Blows, Powerful Stance, Wrecking Blows, Clear Mind, Come and Get Me, nor the monk powers) and 12 teamwork feats; the feature (`BattleProwessFeature` 225940b509289224e83cd900f15b852e) is on row 3 only. Extra Rage Power's half is [264](264-the-extra-feats.md)'s.
- The bard's and skald's talent lists are [276](276-rogue-and-slayer-talents.md)'s; the separate-selection rows [244](244-the-game-data-laid-out-by-blueprint.md)'s.

## Findings (the game check, `known.txt` section 286)

| The check's group (area / kind) | Findings |
| --- | --- |
| progression rows / a further rank in game only | 29 |
| progression rows / a further rank in planner only | 28 |
| progression rows / in game only | 21 |
| option lists / in planner only | 17 |
| option lists / in game only | 12 |
| progression rows / in planner only | 4 |
| progression rows / at another level | 1 |
| **all** | **112** |

The 14 entries: `archetype bard/archaeologist`, `archetype bard/beast-tamer`, `archetype bard/chelaxian-diva`, `archetype bard/dirge-bard`, `archetype bard/flame-dancer`, `archetype bard/thundercaller`, `archetype bard/tranquil-whisperer`, `archetype skald/battle-scion`, `archetype skald/court-poet`, `archetype skald/demon-dancer`, `archetype skald/herald-of-the-horn`, `archetype skald/inciter`, `class bard`, `selection skald/battle-scion/rage-power-or-teamwork-feat`.

The report lists each finding under its group, marked `[known: 286]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The rows and the Battle Scion's selection become the game's; `archetype_losses.ron`'s beast-tamer, flame-dancer, thundercaller, battle-scion, herald-of-the-horn and inciter entries follow; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `skald_rage_powers.rs`: `the_battle_scions_selection_shares_every_rage_power_and_adds_a_teamwork_feat` (21 and 12; its name), `the_skald_and_each_archetype_have_a_rage_power_slot_at_their_tables_levels` (Battle Prowess on row 3).
- `scaling_features.rs`: `the_bard_archetypes_step_the_features_the_guide_gives_steps_to` (Devastating Aria), `the_heralds_horn_call_steps_and_keeps_what_the_skalds_rows_give`, the skald archetypes' steps test (Caustic Ridicule, Prompted Backstabbing: [254](254-a-features-steps-follow-the-class-level.md)'s pass).
- `archetype_losses.rs` (the six entries above).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 286 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
