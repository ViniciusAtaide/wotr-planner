# The game check's misses: leaves only a pet fails, and the same-value feats

Type: build (tools)
Status: open
Blocked by: none
Model: Sonnet to build: two normalisations in `tools/game-check`, each with a fixture test.
Map: [WotR Build Planner](../map.md)

## Problem

Some of the first run's findings are the check's, not the data's: both sides hold the same rule and the check can't see it.
- **Leaves only a pet can fail.** The check already drops "not a pet" and "no animal class levels" (238's normalisations), but not these, which hold for every Build the planner makes: `not AnimalCompanionClass level 1` (`PrerequisiteClassLevel` of `AnimalCompanionClass`, `Not`), `not SubtypeElemental`, `not MonstrousHumanoidClass level 1`, and `not BlizzardServantLockEquipmentFeature` (the Winter Child's servant's lock, on Weapon Finesse).
- **The same-value feats.** The planner writes "Weapon Focus in the same weapon" as `FeatForSameValue("weapon-focus")`, which the check drops as having no game form; the game writes `PrerequisiteFeature WeaponFocus` (or `PrerequisiteParametrizedFeature`). So Weapon Specialization, Greater Weapon Focus, Greater Weapon Specialization, Greater Spell Focus, Greater Elemental Focus (whose game leaves are the four `ElementalFocus<Element>` features), the three Improved² Critical feats, and the mythic Improved Critical and Spell Focus are reported "missing in planner".

## Findings (the game check, `known.txt` section 256)

| The check's group (area / kind) | Findings |
| --- | --- |
| prerequisites / missing in planner, a blueprint it doesn't map | 20 |
| prerequisites / missing in planner | 9 |
| **all** | **29** |

The 25 entries: `feat armor-proficiency-heavy-armor`, `feat armor-proficiency-heavy-barding`, `feat armor-proficiency-light-armor`, `feat armor-proficiency-light-barding`, `feat armor-proficiency-medium-armor`, `feat armor-proficiency-medium-barding`, `feat exotic-weapon-proficiency`, `feat focused-strike`, `feat greater-elemental-focus`, `feat greater-spell-focus`, `feat greater-weapon-focus`, `feat greater-weapon-specialization`, `feat improved-improved-critical`, `feat improved-improved-improved-critical`, `feat improved-improved-improved-critical-improved`, `feat indomitable-mount`, `feat martial-weapons-proficiency`, `feat mounted-combat`, `feat shield-proficiency`, `feat simple-weapon-proficiency`, `feat tower-shield-proficiency`, `feat weapon-finesse`, `feat weapon-specialization`, `mythic feat improved-critical-mythic`, `mythic feat spell-focus-mythic`.

The report lists each finding under its group, marked `[known: 256]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The pet-only leaves above are dropped and counted, as "not a pet" is (a leaf the check is sure every Build meets).
- `FeatForSameValue(X)` compares as the game's prerequisite on X's blueprint (for Greater Elemental Focus, any of the per-element features `GreaterElementalFocusSelection` 1c17446a3eb744f438488711b792ca4d names), so these entries agree.
- No data change.

## Done when

- A fixture test per normalisation shows the planted leaf dropped or matched, and the real run reports none of this ticket's findings; its section of `known.txt` is gone and the game check exits 0 (two runs `cmp` identical).
- `cargo test -p game-check` and fmt pass.
