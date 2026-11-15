# Feats' prerequisites

Type: build (data)
Status: open
Blocked by: [247](247-a-selection-counts-from-the-level-that-grants-it.md) (Warrior Priest names selections), [256](256-the-game-checks-misses.md) (so the report shows only the data's differences)
Model: Opus to build: each feat's prerequisites are read from its blueprint and judged against the planner's forms.
Map: [WotR Build Planner](../map.md)

## Problem

The game check's prerequisites of the general feats (`feats.ron`) differ from the blueprints in these patterns; each finding in the report names the feat, its blueprint and GUID, and the leaf:
- **Leaves the planner leaves to the chain:** the game repeats a lower tier's prerequisites at each tier (Strength 13 and Power Attack on the Cleave and Bull Rush chains; Intelligence 13 and Combat Expertise or Kinetic Warrior on the Greater manoeuvres; Improved Unarmed Strike on the style feats; Point-Blank Shot on the Snap Shot tiers; BAB on Bashing Finish, the Diva feats, Ranged Feint, Energized Wild Shape, Frightful Shape, the Improved³ Critical feats).
- **Grouping:** Improved Dirty Trick, Disarm and Trip need Intelligence 13 and one of Combat Expertise or Kinetic Warrior; the planner asks Intelligence 13 of a Kinetic Warrior only.
- **Leaves the game has and the planner doesn't:** races (Blood Feaster, Ironguts, Murmurs of Earth, Steel Soul's Hardy, Improved Elven Immunities' Elven Immunities), Fencing and Slashing Grace's and Shifter's Edge's "not Weapon Finesse (Mythic)", Diehard's "not Half-Orc Ferocity", the Wings feat's "not the Angel sorcerer's wings", Agile Maneuvers' Improved Disarm, Point-Blank Master's fighter 4, Spell Chain's Spell Penetration, Spell Specialization's own progression.
- **Leaves the planner has and the game doesn't:** Manyshot's and Rapid Shot's Dexterity and Point-Blank Shot, the shield feats' Shield Proficiency, the Trickster feats' Perception tier 2, Fencing and Slashing Grace's Weapon Focus (see below), Critical Mastery's two critical feats, Point-Blank Master's Weapon Specialization, Accomplished Sneak Attacker's Sneak Attack.
- **A blueprint the planner doesn't map:** Boon Companion needs `AnimalCompanionRank` where the planner lists 15 companion selections; the wild-shape feats name `MajorFormFeature` and the Feral Champion's wolf; Natural Spell, Raging Brutality, Selective Channel, Lingering Performance, Discordant Voice and Disruptive name features of other classes; Warrior Priest (`WarriorPriest` b9bee4e4e15573546b76a8d942ce914b) needs `DomainsSelection`, `DruidDomainSelection` or `OracleMysterySelection`, which [247](247-a-selection-counts-from-the-level-that-grants-it.md)'s form expresses.
- **The barding proficiencies** need the taker to be an animal (`AnimalClass` or `AnimalCompanionClass` level 1): a Build can't take them.
- **Advanced Weapon Training** (`AdvancedWeaponTraining1` 3aa4cbdd4af5ba54888b0dc7f07f80c4) needs the fighter's `WeaponTrainingSelection` and fighter 5 only; the Sohei's and the Disciple of the Pike's weapon training don't meet it (242, `class_skills.rs`).
- **The general feat pool** (`BasicFeatSelection` 247a4068296e8be42890143f451b4b45) holds six feats the planner's pool lacks (Bleeding Critical, the Martyr's and the skald's Extra Performance, the Instinctual Warrior's Extra Rage, the Vivisectionist's Extra Discovery, …); the Extra ones go with [264](264-the-extra-feats.md)'s split where they are its feats.
- Fencing Grace's game prerequisite is Weapon Focus in a one-handed piercing weapon (`PrerequisiteParametrizedWeaponSubcategory`, which the check doesn't compare): the value list is [289](289-weapons-and-finesse-training.md)'s.

## Findings (the game check, `known.txt` section 263)

| The check's group (area / kind) | Findings |
| --- | --- |
| prerequisites / missing in planner | 87 |
| prerequisites / not in game | 66 |
| prerequisites / missing in planner, a blueprint it doesn't map | 25 |
| option lists / in game only | 6 |
| prerequisites / grouping differs | 3 |
| **all** | **187** |

The 84 entries: `feat accomplished-sneak-attacker`, `feat advanced-weapon-training`, `feat agile-maneuvers`, `feat armor-proficiency-heavy-barding`, `feat armor-proficiency-light-barding`, `feat armor-proficiency-medium-barding`, `feat bashing-finish`, `feat blood-feaster`, `feat boar-ferocity`, `feat boar-shred`, `feat boon-companion`, `feat bull-rush`, `feat cleave`, `feat cleaving-finish`, `feat combat-mobility`, `feat crane-riposte`, `feat crane-wing`, `feat critical-mastery`, `feat crushing-blow`, `feat diehard`, `feat dirty-trick`, `feat disarm`, `feat disarming-strike`, `feat discordant-voice`, `feat disruptive`, `feat diva-advance`, `feat diva-strike`, `feat diva-style`, `feat dragon-ferocity`, `feat dragon-roar`, `feat energized-wild-shape`, `feat extended-aspects`, `feat favored-enemy-spellcasting`, `feat feint`, `feat fencing-grace`, `feat final-feint`, `feat frightful-shape`, `feat great-cleave`, `feat greater-bull-rush`, `feat greater-dirty-trick`, `feat greater-disarm`, `feat greater-penetrating-strike`, `feat greater-shield-focus`, `feat greater-snap-shot`, `feat greater-sunder-armor`, `feat greater-trip`, `feat improved-cleaving-finish`, `feat improved-elven-immunities`, `feat improved-improved-critical`, `feat improved-improved-improved-critical`, `feat improved-improved-improved-critical-improved`, `feat improved-snap-shot`, `feat ironguts`, `feat lingering-performance`, `feat manyshot`, `feat metamagic-completely-normal-spell`, `feat murmurs-of-earth`, `feat natural-spell`, `feat point-blank-master`, `feat pummeling-bully`, `feat pummeling-style`, `feat raging-brutality`, `feat raking-claws`, `feat ranged-feint`, `feat rapid-shot`, `feat selective-channel`, `feat shaitan-earthblast`, `feat shaitan-skin`, `feat shield-bash`, `feat shield-focus`, `feat shield-wall`, `feat shifters-edge`, `feat shifters-rush`, `feat slashing-grace`, `feat spell-chain`, `feat spell-specialization`, `feat stat-focus`, `feat steel-soul`, `feat sunder-armor`, `feat trip`, `feat two-weapon-feint`, `feat warrior-priest`, `feat wings`, `pool general feats`.

The report lists each finding under its group, marked `[known: 263]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- Each feat's prerequisite becomes the game's, in the planner's forms; a leaf the planner can't express (a pet-only leaf, a story leaf) is recorded in the data's note with its reason. The barding proficiencies leave what a Build can pick (or carry a prerequisite no Build meets), as the game's rule says.
- A data note lists every feat changed.

## Tests 242 marks as waiting on this ticket

- `class_skills.rs`: `features_sharing_a_name_read_apart_in_a_reason` (Advanced Weapon Training's reasons name only the fighter's Weapon Training; the test needs another pair of same-named features for its read-apart half).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 263 (exit 0, run by the main session).
- A real-data test per pattern asserts the game's prerequisite, cited by blueprint and GUID.
- The scoped checks pass.
