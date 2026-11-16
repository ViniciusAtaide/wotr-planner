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
- Ours: `skips_proficiency` on the Sword Saint's list rather than modelling its chosen-weapon feature; since the review fixes the pick also grants the proficiency, as the game's feature does.
- Ours: Fencing Grace, Slashing Grace, Point Blank Master and the mythic Weapon Focus and Weapon Specialization set the game's flag but take no weapon in the planner, so they aren't marked. The game's `Disabled` weapons (sling, siangham, hand and repeating crossbows, shuriken) aren't in the data.

**Tests.**
- Rules, hand-made data, `libs/rules/tests/engine/required_proficiency.rs`: `a_slot_that_ignores_prerequisites_still_takes_only_weapons_the_character_is_proficient_with` (the test world's warpriest slot refuses Weapon Focus (Longsword) with `NotProficient` and its text, takes the unarmed strike; the walk fails the pick; unflagged, any weapon), `proficiency_taken_at_the_same_level_opens_the_weapon` (EWP (Bastard Sword) in the general slot opens Weapon Focus in the fighter's class slot; `agreement::judge` clean), `a_list_whose_pick_skips_proficiency_takes_any_weapon`.
- Real data, `libs/seeds/tests/game_data/parameterised_feats.rs`: `a_warpriests_weapon_focus_slot_opens_only_the_weapons_he_is_proficient_with` off ignore (game: `WarpriestWeaponFocusSelection` ac384183dbfbbd7499410a21d749bef1, `WeaponFocus` 1e1f627d26ad36f43bbd26cc2bf8ac7e, the code above); `the_feats_that_require_proficiency_are_the_games` (each blueprint cited by name and GUID); `a_sword_saint_chooses_a_weapon_he_isnt_proficient_with_yet` (`SwordSaintChosenWeaponSelection` 3130e3083845b7b4fa2ac5b6f1a1ae6e, `SwordSaintChosenWeaponFeature` c0b4ec0175e3ff940a45fc21f318a39a).
- Re-read against the rule: tests that took Weapon Focus in a weapon the class isn't proficient with now take one it is (`class_rules.rs` the Crusader: heavy mace; `option_families.rs` Weapon Mastery: heavy mace, citing `OracleRevelationWeaponMasteryWeaponFocusSelection` 7e360e025592a734bb6c3ae1210824f3; `repeating_talents.rs`: short sword for the rogue); `bonus_feat_choices.rs` accepts `NotProficient` beside open for the Hunter's and Stormwalker's lists. The old-link fixtures (`first_ascension.rs`, `ranks_and_pools.rs`) hold Weapon Focus in weapons the classes aren't proficient with: they still decode to the same Build, and are checked without those picks (`support::without_unproficient_picks`); such a link is now stale, as the engine opens no Build the game refuses.

**Review fixes** (2026-11-15).
- **The Sword Saint's chosen weapon is proficient** (the review's regression: a Sword Saint with a martial chosen weapon was refused Improved Critical, Weapon Specialization, Greater Weapon Focus for it with `NotProficient`, which the game allows). The game's `SwordSaintChosenWeaponFeature` c0b4ec0175e3ff940a45fc21f318a39a carries `KensaiChosenWeapon`, whose `OnActivate` adds the chosen weapon to the unit's proficiencies (`Owner.Proficiencies.Add`) and gives Weapon Focus in it (`Designers/Mechanics/Facts/KensaiChosenWeapon.cs`, WrathDecompiled 6ce4a6b); no other blueprint carries that component. A pick from a list that `skips_proficiency` now holds proficiency with its weapon: `CharacterState::weapon_proficiencies` (counted, so a release takes it back), held through `Held::Proficiency` in `Walker::hold_pick`, read by `Catalog::proficient`, so the pick check and the sheet's weapons (`Engine::proficient_weapons`, the attack line's picker) see it. `judge::skips_proficiency` is the one test of the list, for the check and the grant. Tests: `parameterised_feats.rs` `a_sword_saint_is_proficient_with_his_chosen_weapon` (Improved Critical (Longsword) at magus 11 refused `NotProficient` without the chosen weapon, the Build possible with Longsword chosen at 1, the longsword proficient at 1 and 11; game: `KensaiChosenWeapon.OnActivate`, `SwordSaintChosenWeaponFeature` c0b4ec0175e3ff940a45fc21f318a39a, `ImprovedCritical` f4201c85a991369408740c6888362e20 needing BAB 8), failing before the fix with `possible(&saint)` false; the rules fixture `a_list_whose_pick_skips_proficiency_takes_any_weapon_and_makes_the_character_proficient_with_it` (was `a_list_whose_pick_skips_proficiency_takes_any_weapon`).
- **The old-link fixtures.** `ranks_and_pools.rs` `links_made_before_the_splits_open_with_the_same_picks` checks again that a link that opens opens as the Build it decodes to (`open_link(link) == decode_build(link)`), and that one that doesn't fails only for its weapon feats in weapons the class isn't proficient with. `without_unproficient_picks` strips something from 4 of the 8 rows (the Dual-Cursed and Possessed Oracles, the Stigmatized Witch, the Feyspeaker); the other 4 open unchanged, so the open check isn't vacuous: the test pins `(8, 4)`. In `first_ascension.rs` it strips picks from 3 of the 8 links (demon 2 picks, lich 4, trickster 4; the two aeon, the angel and the two azata links none).
- **The Hunter's and the Stormwalker's lists.** `bonus_feat_choices.rs` pins the options refused: none in the Hunter's and the Urban Hunter's lists; in the Stormwalker's archery list from 10 (at 10, 14 and 18) Improved Critical in each of the 17 exotic weapons, `NotProficient`, and nothing else. Game: `ImprovedCritical` f4201c85a991369408740c6888362e20 (`RequireProficiency`), offered by `RangerStyleArcherySelection10` 7ef950ca681955d47bc4efbe77073e2c; `RangerProficiencies` c5e479367d07d62428f2fe92f39c0341 gives simple and martial weapons, which `StormwalkerArchetype` a330c05e484d98748a462acd43654a3a keeps.
- **Docs.** `libs/game-data/README.md` documents `requires_proficiency` (feats and mythic feats) and `skips_proficiency` (feat lists); the note says the chosen weapon's proficiency is held.

## Done when

- The test above holds the game's answer; a fixture test shows a non-proficient value refused in a slot that ignores prerequisites.
- The scoped checks pass.
