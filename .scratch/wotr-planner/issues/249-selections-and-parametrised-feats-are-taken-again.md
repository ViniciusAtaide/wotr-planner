# A selection or a parametrised feat may be taken again

Type: build (engine rule)
Status: resolved
Blocked by: none
Model: Opus to build: it changes what the pickers offer as already held.
Map: [WotR Build Planner](../map.md)

## Problem

Found by [242](242-the-tests-expected-values-from-the-game.md)'s research (2026-11-15): the game's code contradicts a rule of the engine.
- **A selection skips the rank check:** an option that is itself a selection may be picked again, at any rank (`Blueprints/Classes/Selection/BlueprintFeatureSelection.cs`, `CanSelect`, WrathDecompiled 6ce4a6b). Favorite Metamagic (`FavoriteMetamagicSelection` 503fb196aa222b24cb6cfdc9a284e838, offered by `MythicAbilitySelection` ba0e5a900b775be4a99702f1ed08914d) can be taken a second time for another metamagic; the planner refuses it as held.
- **A parametrised feature repeats per value:** it is refused only for a value already held (`Blueprints/Classes/Selection/BlueprintParametrizedFeature.cs`, `CanSelect`). The rogue talent Weapon Focus is the parametrised `WeaponFocus` (1e1f627d26ad36f43bbd26cc2bf8ac7e), so it can be taken once per weapon.
- **The user's decision (2026-11-15):** the game wins over the user's decision of 2026-11-02 (the rogue talent Weapon Focus taken once): it repeats per weapon. Applied without asking again.

## Change

- The pickers and the walk offer again an option that is a selection, and a parametrised feat or talent for a value not yet held, wherever its slot allows it.
- The caps a few of these have are [250](250-the-extra-feats-caps.md)'s.

## Tests 242 marks as waiting on this rule

- `repeating_talents.rs`: `the_rogue_weapon_focus_talent_cant_be_taken_twice` (Weapon Focus (Longsword) then (Dagger) through the talent is allowed), and the row of `the_skill_focus_talents_repeat_once_per_skill_and_weapon_focus_doesnt` that says Weapon Focus doesn't repeat.
- `pick_values.rs`: `an_older_link_with_favorite_metamagic_opens_with_its_metamagic_left_open` (a second Favorite Metamagic is legal).

## Answer

**Built** (2026-11-15). Data note: `libs/game-data/notes/249-selections-and-parametrised-feats-are-taken-again.md`.

- **The rogue talent Weapon Focus repeats per weapon** (data): `rogue/rogue-talents/weapon-focus` has `repeat: PerValue`, which the engine already read for an option with a chosen feat (the Skill Focus talents): refused only for a weapon Weapon Focus is already held for, however it was taken. Game: `WeaponFocus` 1e1f627d26ad36f43bbd26cc2bf8ac7e is a `BlueprintParametrizedFeature`, offered by `RogueTalentSelection` c074a5d615200494b8f2a9c845799d93; `BlueprintFeatureSelection.CanSelect` skips the rank check for it and `BlueprintParametrizedFeature.CanSelect` refuses only a held value (WrathDecompiled 6ce4a6b).
- **A mythic ability may repeat per value** (engine and data): `MythicAbility::repeat` (`game_data.rs`, default `Once`), as feats and mythic feats have it; `favorite-metamagic` and `ascendant-element` have `PerValue`. The state holds the values (`CharacterState::mythic_ability_values`, `has_mythic_ability_for`; `Held::MythicAbility` carries the value, released per value); the walk's judge (`judge.rs`) refuses only a held value, the picker's "held already" reason (`holds_the_same`, `pickers.rs`) only a once-only ability; `GameData::check` refuses `PerValue` on an ability that takes no value. A pick with its value left open (an older link) is held once, as before. Game: `FavoriteMetamagicSelection` 503fb196aa222b24cb6cfdc9a284e838 and `AscendantElementSelection` 535d4e5d8a5664f4797560c280941782 are selections offered by `MythicAbilitySelection` ba0e5a900b775be4a99702f1ed08914d (`Mode` Default), whose options each have `Ranks` 1 (`FavoriteMetamagicQuicken` 4ecf1a86ddc45d04c8b2df3a1c318e78, `AscendantElementAcid` d808baf783a063e4d84a1aaadbdd1cb7); `CanSelect` lets a selection repeat and refuses a held one-rank option (and `NestedFeatureSelectionUtils.AllNestedFeaturesUnavailable` refuses the selection once none is left).
- **The planner's other "selections"** (Combat Trick, the Battle Scion's teamwork feat) already repeat as `Unlimited` options.

- **Review fixes** (2026-11-15): releasing a valued mythic ability pick (a Path switch dropping a later Favorite Metamagic (Empower)) no longer removes the ability while an older valueless pick still holds it: `CharacterState::mythic_ability_open_holds` counts the valueless holds, and `hold.rs` removes the key only when both it and the values are empty. Test: `pick_values.rs`'s `releasing_a_valued_pick_keeps_an_older_valueless_one_held` (hand-made data; Legend switch keeps MR 1, drops MR 3).

**Decisions.**
- The user (2026-11-15): the rogue talent Weapon Focus repeats per weapon, the game over the decision of 2026-11-02.
- 242's research: a second Favorite Metamagic is legal.
- Ours: Ascendant Element repeats too, as the same rule applies to its blueprint (a selection of one-rank features).
- Ours, at the simplify review: a `repeat` field on `MythicAbility` rather than reading "takes a value" as "repeats per value", so the data says it as it does for feats and mythic feats.

**Tests.**
- Rules, hand-made data: `pick_values.rs`'s `favorite_metamagic_is_taken_again_for_another_metamagic_but_not_the_same` (replaces `favorite_metamagic_is_held_once_whatever_its_metamagic`: the picker offers Empower at MR 3 and refuses the held Persistent, the walk holds both values, `agreement::judge` finds no disagreement, a second Persistent is `AlreadyHeld`); `parameters.rs`'s existing `an_option_once_per_value_repeats_for_another_value_of_its_chosen_feat` is the parametrised option refused only for its held value; `check.rs`'s `a_mythic_ability_repeats_per_value_only_with_a_value`.
- Real data, off `#[ignore]`: `repeating_talents.rs`'s `the_rogue_weapon_focus_talent_is_taken_once_per_weapon` (its "Longsword then Dagger" half checked `why_not` of a possible Build, which the helper refuses; it now asserts the Build possible) and `pick_values.rs`'s `favorite_metamagic_is_taken_again_for_another_metamagic`. New: `pick_values.rs`'s `ascendant_element_is_taken_again_for_another_element_but_not_the_same`, which also asserts both abilities' `repeat`. The ticket's "row of `the_skill_focus_talents_repeat_once_per_skill_and_weapon_focus_doesnt`" is already gone: that test is `the_skill_focus_talents_repeat_once_per_skill` and says nothing of Weapon Focus's repeat.

## Done when

- The tests above hold the game's answers with the citations above.
- A fixture test in `rules` shows a selection option offered again and a parametrised option refused only for its held value.
- The scoped checks pass, the self-checks' short runs among them.
