# Warpriest rows

Type: build (data)
Status: open
Blocked by: [254](254-a-features-steps-follow-the-class-level.md)
Model: Opus to build: each row is read from the warpriest's and the archetypes' progressions.
Map: [WotR Build Planner](../map.md)

## Problem

The game check's progression rows of the warpriest (`WarpriestProgression` 062b215705430ad47b7174c1e75d285e) and its archetypes differ:
- **Repeated grants against the game's own steps** ([254](254-a-features-steps-follow-the-class-level.md)'s shape): the planner grants Fervor, Sacred Weapon and Sacred Armor again at each step; the game grants `WarpriestFervor` once with its heal/harm selection (`WarpriestFervorHealDamageSelection` b0dc39bfbc6f85646adbfc6413f843b0) and rank, and the sacred weapon's and armour's enchant features (`SacredWeaponEnchantPlus2` 4ee3e74f79d5e4f47b7a0091984b2518 to `Plus5`; `SacredArmorEnchantPlus2` ec327c67f6a6b2f49a8ca218466a8818 to `Plus5`) at their levels. The archetypes' own scaling features (the Proclaimer's Cleanser of Evil, the Champion's smite, the Disenchanter's Mystic Interference, the Mantis Zealot's Fading and Swarm, the Shieldbearer's Sacred Shield) likewise.
- **Channel energy** (`WarpriestChannelEnergySelection` 1a871294410971745bba12ef11e41f6e and the Shieldbearer's) is in the planner's rows at levels the game grants nothing.
- **The archetypes' removals** (242, `archetype_losses.ron`): the Cult Leader (`CultLeaderArchetype` cd2d419961bb019438b77d4aecc2fc04) keeps shield proficiency; the Mantis Zealot (`MantisZealotArchetype` fc403240fdca4a52a413578169ea7117) keeps both blessings; the Proclaimer (`ProclaimerArchetype` aaf7ace9642192d4389eab95e6cd04fd) removes the heal/harm selection at 2 and the ranks at 5, 11, 14, 17, 20, keeping one at 8; the Shieldbearer (`ShieldbearerArchetype` d34dcc10e74961f44a81f72b54377b24) keeps sacred weapon and the Weapon Focus slot, swaps only the level-4 enchant for the shield's, and its Sacred Shield (4, 8, 12, 16, 20) replaces Sacred Armor.
- **Fading** (`MantisZealotFadingFeature` 724a017c96d643729f65d2d1eac54673) steps at 7, 10, 13, 16 (no 19).
- **The Disenchanter's lists** ignore no prerequisites in the game (their Skill Focus and Spell Focus are [255](255-a-parametrised-feat-at-one-value.md)'s).

## Findings (the game check, `known.txt` section 268)

| The check's group (area / kind) | Findings |
| --- | --- |
| progression rows / a further rank in planner only | 129 |
| progression rows / in game only | 73 |
| progression rows / in planner only | 25 |
| option lists / ignores prerequisites or not | 3 |
| progression rows / at another level | 1 |
| **all** | **231** |

The 11 entries: `archetype warpriest/champion-of-the-faith`, `archetype warpriest/cult-leader`, `archetype warpriest/disenchanter`, `archetype warpriest/feral-champion`, `archetype warpriest/mantis-zealot`, `archetype warpriest/proclaimer`, `archetype warpriest/shieldbearer`, `class warpriest`, `feat list warpriest/disenchanter/bonus-feats-12`, `feat list warpriest/disenchanter/bonus-feats-3`, `feat list warpriest/disenchanter/bonus-feats-9`.

The report lists each finding under its group, marked `[known: 268]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The rows become the game's with [254](254-a-features-steps-follow-the-class-level.md)'s model; `archetype_losses.ron`'s warpriest entries follow; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `archetype_losses.rs` (`archetype_losses.ron`'s entries for cult-leader, mantis-zealot, proclaimer, shieldbearer).
- `scaling_features.rs`: the Fervor test (every archetype keeps it; the Proclaimer's own series) and the warpriest archetypes' steps test (Fading at 7, 10, 13, 16).
- `parameterised_feats.rs`: `a_warpriest_takes_weapon_focus_in_a_weapon_of_his_choosing` waits on [251](251-proficiency-holds-where-prerequisites-are-ignored.md).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 268 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
