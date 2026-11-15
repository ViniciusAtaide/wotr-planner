# Prestige classes

Type: build (data)
Status: open
Blocked by: [254](254-a-features-steps-follow-the-class-level.md)
Model: Opus to build: entry prerequisites and rows read from each class's blueprint and progression.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half B part 3 `option_levels.rs`, part 4 `report_gaps.rs`) differ on `libs/game-data/prestige/`:
- **No character-level prerequisite:** the game's prestige classes ask none (`Blueprints/Classes/BlueprintCharacterClass.cs`, `MeetsPrerequisites`, WrathDecompiled 6ce4a6b); the planner asks character level 5 to 8 of the Aldori Swordlord, Arcane Trickster, Assassin (`AssassinClass` eb284ea8d40a2d045911f525eb96c43d), Dragon Disciple (`DragonDiscipleClass` 72051275b1dbb2d42ba9118237794f7c), Duelist (`DuelistClass` 4e0ea99612ae87a499c7fb0588e31828), Eldritch Knight, Hellknight (`HellknightClass` ed246f1680e667b47b7427d51e651059), Hellknight Signifer, Loremaster, Mystic Theurge, Stalwart Defender, Student of War and Winter Witch. A sorcerer 5 / Dragon Disciple at character level 6 is open.
- **Other entry prerequisites:** the Mystic Theurge (`MysticTheurgeClass` 0920ea7e4fd7a404282e3d8b0ac41838) needs 3 ranks of Knowledge (Arcana) and Lore (Religion), not 5, and refuses the Magic Deceiver, as the Loremaster (`LoremasterClass` 4a7c05adfbaf05446a6bf664d28fb103) does; the Loremaster's metamagic list has no Intensified or Piercing Spell; the Duelist needs Dodge; the Stalwart Defender Endurance; the Hellknight no Heavy Armor Proficiency; the Arcane Trickster no Sneak Attack or Accomplished Sneak Attacker; the Dragon Disciple's draconic bloodlines aren't a prerequisite (none or a draconic one); the Winter Witch needs the Winter Child's domain selection, not the archetype.
- **The Stalwart Defender:** its defensive powers (`StalwartDefenderDefensivePowerSelection` 2cd91c501bda80b47ac2df0d51b02973) offer Smash, not Clear Mind; Clear Mind's gate is the rage powers' four classes ([269](269-barbarian-and-rage-powers.md)); Fearless Defense and Increased Damage Reduction gate on the Stonelord's paladin levels; its AC bonus and damage reduction are ranks (`StalwartDefenderACBonus` 60b4e1f781e9aac439b8d2ad4c635524), and its rows grant the armour proficiencies and Body of Stone.
- **Rows:** the Arcane Trickster's Impromptu Sneak Attack uses; the Dragon Disciple's bloodline feat selection, second bite and ultimate form, and Blind Sense ranks (`DragonDiscipleProgression` 69fc2bad2eb331346a6c777423e0d0f7), and its `bloodline` row; the Loremaster's secret offers the Trickster's combat feat selection too; the Student of War's Know Your Enemy (`StudentOfWarKnowYourEnemyFeature` 846c2094392076a4f9f3c84f0ecfe180) is a feature, not a choice, with ranks; the Hellknight's, the Signifer's, the Aldori Swordlord's (Defensive Parry, Adaptive Tactics, Improved Reaction) and the Winter Witch's (Cold Specialization, Freezing Cold) ranks and steps ([254](254-a-features-steps-follow-the-class-level.md)).

## Findings (the game check, `known.txt` section 287)

| The check's group (area / kind) | Findings |
| --- | --- |
| prerequisites / not in game | 29 |
| progression rows / in game only | 23 |
| prerequisites / missing in planner | 14 |
| progression rows / a further rank in game only | 11 |
| progression rows / a further rank in planner only | 3 |
| option lists / in game only | 2 |
| prerequisites / value differs | 2 |
| option lists / a choice the game doesn't offer | 1 |
| option lists / in planner only | 1 |
| prerequisites / missing in planner, a blueprint it doesn't map | 1 |
| progression rows / in planner only | 1 |
| **all** | **88** |

The 20 entries: `class aldori-swordlord`, `class arcane-trickster`, `class assassin`, `class dragon-disciple`, `class duelist`, `class eldritch-knight`, `class hellknight`, `class hellknight-signifer`, `class loremaster`, `class mystic-theurge`, `class stalwart-defender`, `class student-of-war`, `class winter-witch`, `selection loremaster/secret`, `selection option loremaster/secret/combat-feat`, `selection option stalwart-defender/defensive-powers/clear-mind`, `selection option stalwart-defender/defensive-powers/fearless-defense`, `selection option stalwart-defender/defensive-powers/increased-damage-reduction`, `selection stalwart-defender/defensive-powers`, `selection student-of-war/know-your-enemy`.

The report lists each finding under its group, marked `[known: 287]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- Entry prerequisites, powers and rows become the game's; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `option_levels.rs`: `a_sorcerer_5_dragon_disciple_5_has_bloodline_level_10` (the entry at character level 6 is open).
- `report_gaps.rs`: `the_dragon_disciples_no_bloodline_yet_reads_once` (the text loses "character level 6").
- `scaling_features.rs`: `the_assassins_cold_minded_and_the_winter_witchs_cold_specialization_step`.

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 287 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
