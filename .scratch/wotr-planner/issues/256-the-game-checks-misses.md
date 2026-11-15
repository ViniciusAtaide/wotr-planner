# The game check's misses: leaves only a pet fails, and the same-value feats

Type: build (tools)
Status: resolved
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

## Answer

**Built (2026-11-15):** two normalisations in `tools/game-check/src/areas/prerequisites.rs`. No data change.
- **Pet-only leaves** (`holds_for_every_build`, constants in `areas/mod.rs` `fixed`): the game's leaves every Build meets are dropped and counted ("not a pet, which always holds"), as 238's are. They are `PrerequisiteClassLevel` with `Not` and `PrerequisiteNoClassLevel` of `AnimalCompanionClass` 26b10d4340839004f960f9816f6109fe or `MonstrousHumanoidClass` 8a3c86893f383214da070e9c84c1e95b, and `PrerequisiteNoFeature` of `SubtypeElemental` 198fd8924dabcb5478d0f78bd453c586 or `BlizzardServantLockEquipmentFeature` daf893d14cc54e98a319fb121d7ac4d9. One a Build meets in an `Any` group holds the whole group (`BlueprintFeature.MeetsPrerequisites`, WrathDecompiled 6ce4a6b), so the group is dropped with it; the armour proficiencies' `Any` group of one `Not` leaf needs this. A non-negated `PrerequisiteClassLevel` of a pet class, and any other `NoFeature`, still compare.
- **Same-value feats** (`same_value`): `FeatForSameValue(X)` is the game's prerequisite on X's blueprint, `Has(X)`; when X's blueprint is a selection with `m_AllFeatures`, any of those. Elemental Focus `bb24cc01319528849b09a3ae8eec0b31` lists `ElementalFocusAcid`, `Cold`, `Electricity`, `Fire`, the four `GreaterElementalFocusSelection` 1c17446a3eb744f438488711b792ca4d asks for with `PrerequisiteFeature` of group `Any`. The skip is counted as "the value of a parametrized feature, compared as the feature".
- **Decision (me, as the ticket's builder; the ticket allowed "any of the per-element features"):** the per-element set is read from the planner feat's own blueprint (`m_AllFeatures`), not from the greater feat's prerequisites, so the rule is one for every same-value feat.
- **A third finding the rule removed:** `mythic feat skill-focus-mythic` (known line of 244, "Mythic Skill Focus asks for the same skill as its own value") now agrees, since `FeatForSameValue("skill-focus")` reads `SkillFocus`'s `m_AllFeatures`, the eleven features the game names. That is the check's miss, not a data difference, so 244's line and its comment are deleted with section 256.
- No finding of 256 turned out a real data difference; none moved to 263.

**`known.txt`:** section 256 (25 entries) and 244's Skill Focus line deleted. The real run's findings: 4,231 before, 4,191 after (the 29 of 256 and the 11 of Mythic Skill Focus), 0 not known; the group "missing in planner" [known: 256] and "a blueprint it doesn't map" [known: 256] are gone, and 244's group falls from 216 to 205.

**Tests** (`tools/game-check/tests/fixture/areas.rs`, a fixture of invented names; the pet-only classes and features use the game's real GUIDs, which is what the check matches):
- `prerequisites_drop_the_leaves_only_a_pet_can_fail_and_count_them`: `pet-wise` (every leaf of the four kinds, one in an `Any` group, beside a real `Feat` leaf) agrees; the count's line is in the report; `plain-bar`'s `NoFeature` of another feature is still "missing in planner".
- `prerequisites_read_a_same_value_feat_as_the_game_s_feature_or_any_of_a_selection_s_features`: `mastery` (`FeatForSameValue` of a feat; the game's `PrerequisiteFeature`) and `element-mastery` (of a selection of two features; the game's `Any` of the two) agree.
- Both failed first: `pet-wise` reported `not <GUID>` and `not <GUID> level 1`, and `element-mastery` reported both features missing.

**Citations from the game:** the rows above (blueprint names and GUIDs, from the pinned dump `blueprints_raw_2.7.0x_0.binz`); `HeavyArmorProficiency` 1b0f68188dcc435429fb87a022239681 carries `PrerequisiteClassLevel` (`Any`, `Not`) of `AnimalCompanionClass`, `IndomitableMount` 68e814f1f3ce55942a52c1dd536eaa5b `PrerequisiteNoClassLevel` of it and of `MonstrousHumanoidClass`, `HeavyBardingProficiency` aed0b33e17a3b3d44a718852e87305bd `PrerequisiteNoFeature` of `SubtypeElemental`, `WeaponFinesse` 90e54424d682d104ab36436bd527af09 `PrerequisiteNoFeature` of the servant's lock.

**Checks:** `cargo test -p game-check` (23 pass), `cargo clippy -p game-check --all-targets -- -D warnings`, `cargo fmt --all --check`, and the real run exits 0 twice with the two outputs `cmp` identical.

## Done when

- A fixture test per normalisation shows the planted leaf dropped or matched, and the real run reports none of this ticket's findings; its section of `known.txt` is gone and the game check exits 0 (two runs `cmp` identical).
- `cargo test -p game-check` and fmt pass.
