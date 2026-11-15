# A selection or a parametrised feat may be taken again

Type: build (engine rule)
Status: open
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

## Done when

- The tests above hold the game's answers with the citations above.
- A fixture test in `rules` shows a selection option offered again and a parametrised option refused only for its held value.
- The scoped checks pass, the self-checks' short runs among them.
