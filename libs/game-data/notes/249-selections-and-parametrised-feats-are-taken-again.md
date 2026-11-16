# 249: a selection or a parametrised feat may be taken again

**What the data says now.** `rogue/rogue-talents/weapon-focus` (in `classes/rogue.ron`) has `repeat: PerValue`: the
talent gives Weapon Focus for the weapon its pick names, once per weapon, and is refused for a weapon Weapon Focus is
already held for, however it was taken (a feat, another talent). Its summary says so. It was `Once`, the user's
decision of 2026-11-02; the user, 2026-11-15, let the game's rule win.

Game: `RogueTalentSelection` c074a5d615200494b8f2a9c845799d93 offers `WeaponFocus` 1e1f627d26ad36f43bbd26cc2bf8ac7e
itself, a `BlueprintParametrizedFeature`. `BlueprintFeatureSelection.CanSelect` skips the rank check for it and
`BlueprintParametrizedFeature.CanSelect` refuses only a value already held (`unit.GetFeature(this, param)`), both in
`Blueprints/Classes/Selection/`, WrathDecompiled 6ce4a6b.

**The mythic abilities.** `MythicAbility` has a `repeat` field now, as feats and mythic feats have (default `Once`).
`favorite-metamagic` and `ascendant-element` (in `mythic/abilities.ron`), the two that take a value, have
`repeat: PerValue`: each is taken again for another value and refused for a held one. Game: `FavoriteMetamagicSelection`
503fb196aa222b24cb6cfdc9a284e838 and `AscendantElementSelection` 535d4e5d8a5664f4797560c280941782, offered by
`MythicAbilitySelection` ba0e5a900b775be4a99702f1ed08914d (`Mode` Default), are selections whose options each have
`Ranks` 1 (`FavoriteMetamagicQuicken` 4ecf1a86ddc45d04c8b2df3a1c318e78, `AscendantElementAcid`
d808baf783a063e4d84a1aaadbdd1cb7, …). A pick that leaves its value open (an older link) is held once, as before.

**Tests.** `repeating_talents.rs`: `the_rogue_weapon_focus_talent_is_taken_once_per_weapon`; `pick_values.rs`:
`favorite_metamagic_is_taken_again_for_another_metamagic`, `ascendant_element_is_taken_again_for_another_element_but_not_the_same`.
