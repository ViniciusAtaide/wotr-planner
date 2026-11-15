# Proficiency still holds where a slot ignores prerequisites

Type: build (engine rule)
Status: open
Blocked by: none
Model: Opus to build: a check that survives the ignore flag.
Map: [WotR Build Planner](../map.md)

## Problem

Found by [242](242-the-tests-expected-values-from-the-game.md)'s research (2026-11-15): the game's code contradicts a rule of the engine.
- **The game:** Weapon Focus, Weapon Specialization and Improved Critical carry `RequireProficiency`. `BlueprintParametrizedFeature.CanSelect` checks it, and the item's `Disabled` flag, on the feature's own `IgnorePrerequisites`, not the selection's (`Blueprints/Classes/Selection/BlueprintParametrizedFeature.cs`, WrathDecompiled 6ce4a6b). So a slot that skips prerequisites still offers only weapons the character is proficient with.
- **Example:** the warpriest's level-1 slot (`WarpriestWeaponFocusSelection` ac384183dbfbbd7499410a21d749bef1, granted by `WarpriestProgression` 062b215705430ad47b7174c1e75d285e at 1, `IgnorePrerequisites`, one option: `WeaponFocus`) offers only the warpriest's proficient weapons; the planner offers every weapon.

## Change

- A parametrised feat whose blueprint requires proficiency offers only proficient values, in every slot, whatever the slot ignores.
- The data marks which parametrised feats require it (Weapon Focus, Weapon Specialization, Improved Critical, and any other the game flags), each cited.

## Tests 242 marks as waiting on this rule

- `parameterised_feats.rs`: `a_warpriest_takes_weapon_focus_in_a_weapon_of_his_choosing` (only proficient weapons are open).

## Done when

- The test above holds the game's answer; a fixture test shows a non-proficient value refused in a slot that ignores prerequisites.
- The scoped checks pass.
