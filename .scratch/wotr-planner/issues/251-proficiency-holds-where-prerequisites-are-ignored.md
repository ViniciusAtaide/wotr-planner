# Proficiency still holds where a slot ignores prerequisites

Type: build (engine rule)
Status: resolved
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

## Answer

**Built** (2026-11-15). The game checks `RequireProficiency` in `BlueprintParametrizedFeature.CanSelect` on the feature's own flag: the weapon must be among the unit's proficiencies unless the global `IgnorePrerequisites` context flag (a debug cheat) is on, while the selection's `IgnorePrerequisites` skips only the feature's prerequisites (`Blueprints/Classes/Selection/BlueprintParametrizedFeature.cs`, WrathDecompiled 6ce4a6b). The planner now does the same.

- **The data.** `Feat::requires_proficiency` and `MythicFeat::requires_proficiency`, set on the nine weapon-valued feats whose blueprints set the flag (Weapon Focus, Greater Weapon Focus, Weapon Specialization, Greater Weapon Specialization, Improved Critical, the Trickster's three critical feats, Improved Critical (Mythic)). `FeatList::skips_proficiency` on the Sword Saint's chosen-weapon list, whose game pick is `SwordSaintChosenWeaponFeature` (no `RequireProficiency`; its `KensaiChosenWeapon` gives Weapon Focus). Note `libs/game-data/notes/251-proficiency-holds-where-prerequisites-are-ignored.md`.
- **The rule.** `check_proficiency` in the walk's pick check (`engine/walk/judge.rs`): a feat that requires proficiency, picked for a weapon the character isn't proficient with, is refused with the new `Issue::NotProficient { weapon }` ("Not proficient with Longsword"; MCP code `not_proficient`), before the bonus list's prerequisite skip; the same for an option's chosen feat (the rogue talent Weapon Focus, Weapon Mastery) and a mythic feat. Proficiency is ticket 198's rule, now one predicate, `Catalog::proficient`, which `Engine::proficient_weapons` filters by, so the check and the attack line's weapon list can't disagree. The pick check takes the Build start (race, subrace, background weapons) for it.
- **With 248.** A weapon the character becomes proficient with through another pick of the same band at the level (Exotic Weapon Proficiency in the general slot) opens the feat in an earlier slot: `NotProficient` is retried in the band like an unmet prerequisite (`judge::deferrable`).

**Decisions.**
- Ours: a new `Issue` variant rather than an `Unmet` prerequisite, since the bonus list's prerequisite skip must not skip it.
- Ours: `skips_proficiency` on the Sword Saint's list rather than modelling its chosen-weapon feature; the sheet doesn't hold the proficiency it grants (a gap of the magus's data, noted).
- Ours: Fencing Grace, Slashing Grace, Point Blank Master and the mythic Weapon Focus and Weapon Specialization set the game's flag but take no weapon in the planner, so they aren't marked. The game's `Disabled` weapons (sling, siangham, hand and repeating crossbows, shuriken) aren't in the data.

**Tests.**
- Rules, hand-made data, `libs/rules/tests/engine/required_proficiency.rs`: `a_slot_that_ignores_prerequisites_still_takes_only_weapons_the_character_is_proficient_with` (the test world's warpriest slot refuses Weapon Focus (Longsword) with `NotProficient` and its text, takes the unarmed strike; the walk fails the pick; unflagged, any weapon), `proficiency_taken_at_the_same_level_opens_the_weapon` (EWP (Bastard Sword) in the general slot opens Weapon Focus in the fighter's class slot; `agreement::judge` clean), `a_list_whose_pick_skips_proficiency_takes_any_weapon`.
- Real data, `libs/seeds/tests/game_data/parameterised_feats.rs`: `a_warpriests_weapon_focus_slot_opens_only_the_weapons_he_is_proficient_with` off ignore (game: `WarpriestWeaponFocusSelection` ac384183dbfbbd7499410a21d749bef1, `WeaponFocus` 1e1f627d26ad36f43bbd26cc2bf8ac7e, the code above); `the_feats_that_require_proficiency_are_the_games` (each blueprint cited by name and GUID); `a_sword_saint_chooses_a_weapon_he_isnt_proficient_with_yet` (`SwordSaintChosenWeaponSelection` 3130e3083845b7b4fa2ac5b6f1a1ae6e, `SwordSaintChosenWeaponFeature` c0b4ec0175e3ff940a45fc21f318a39a).
- Re-read against the rule: tests that took Weapon Focus in a weapon the class isn't proficient with now take one it is (`class_rules.rs` the Crusader: heavy mace; `option_families.rs` Weapon Mastery: heavy mace, citing `OracleRevelationWeaponMasteryWeaponFocusSelection` 7e360e025592a734bb6c3ae1210824f3; `repeating_talents.rs`: short sword for the rogue); `bonus_feat_choices.rs` accepts `NotProficient` beside open for the Hunter's and Stormwalker's lists. The old-link fixtures (`first_ascension.rs`, `ranks_and_pools.rs`) hold Weapon Focus in weapons the classes aren't proficient with: they still decode to the same Build, and are checked without those picks (`support::without_unproficient_picks`); such a link is now stale, as the engine opens no Build the game refuses.

## Done when

- The test above holds the game's answer; a fixture test shows a non-proficient value refused in a slot that ignores prerequisites.
- The scoped checks pass.
