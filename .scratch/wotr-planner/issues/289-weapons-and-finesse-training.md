# Weapons and Finesse Training

Type: build (data)
Status: open
Blocked by: [251](251-proficiency-holds-where-prerequisites-are-ignored.md) (the proficiency check per value)
Model: Opus to build: the weapon table, categories and Finesse Training's options and gates.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half A `finesse_training.rs`; half B part 4 `weapons.rs`, `parameterised_feats.rs`) differ on `weapons.ron` and the weapon values:
- **The weapon table** (the user's of 2026-11-08). **The user's decision (2026-11-15),** applied without asking again: the heavy shield bash is 1d6 (`WeaponHeavyShield` be9b6408e6101cb4997a8996484baf19), the light 1d4 (`WeaponLightShield` 1fd965e522502fe479fdd423cca07684), and the unarmed strike is finessable (`EmptyHand` a8b38f6b734daa44087ec0ec2e80c1cd; Weapon Finesse's `AttackStatReplacement` keys on `Finessable`), so Weapon Finesse puts Dexterity on unarmed attacks.
- **Categories** (`Enums/WeaponCategoryExtension`, WrathDecompiled 6ce4a6b): 29 martial, the spiked light and heavy shields added; the natural ones also Sting and Tentacle (low priority: record why not if no player source has them).
- **Fencing Grace** (`FencingGrace` 47b352ea0f73c354aba777945760b441) takes Weapon Focus in any one-handed piercing weapon (`PrerequisiteParametrizedWeaponSubcategory.CheckFeature`): also shortspear, trident, light and heavy pick, tongi, the spiked shields, bite and claw (the siangham is disabled).
- **Finesse Training** (`FinesseTrainingSelection` b78d146cea711a84598f0acef69462ea) offers 31 options: the 21 and Gore, Hoof, Slam, Spike, Spiked Light Shield, Tail, Talon, Tentacle, Wing and the Sawtooth Sabre; each once, each needing proficiency in its weapon (a natural one, the natural weapon). A plain rogue (`RogueProficiencies` 33e2a7e4ad9daa54eaf808e1483bb43c) is open to Dagger, Light Mace, Punching Dagger, Rapier, Short Sword, Sickle (and Unarmed, if everyone's unarmed proficiency counts: an inference to settle).
- **Seen, not asserted:** a Human-Raised half-orc keeps the double axe as martial; the halfling's Sling Staff as martial is missing: [291](291-races.md).

## Findings (the game check, `known.txt` section 289)

| The check's group (area / kind) | Findings |
| --- | --- |
| option lists / in game only | 31 |
| **all** | **31** |

The 1 entries: `selection rogue/finesse-training`.

The report lists each finding under its group, marked `[known: 289]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The table, categories, Fencing Grace's values and Finesse Training's options and gates become the game's; a data note lists every change, and `weapons.rs`' table is the game's.

## Tests 242 marks as waiting on this ticket

- `weapons.rs`: `every_weapons_stats_are_the_games_table` (the shield bashes, unarmed finesse).
- `parameterised_feats.rs`: `the_weapons_are_the_guides_proficiency_lists_and_what_weapon_focus_names` (29 martial; the naturals), `prerequisites_name_the_values_the_guide_names` (Fencing Grace's values).
- `finesse_training.rs`: `finesse_trainings_weapons_are_the_guides_list` (31, each once, gated), `a_rogue_chooses_a_weapon_at_3_11_and_19` (the open set; the Light Shield pick needs shield proficiency).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 289 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
