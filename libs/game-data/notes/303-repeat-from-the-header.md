# 303: repeat from the header

**What the data says now.** No entry says how often it is taken: `repeat` is gone from feats, options, mythic
abilities and mythic feats, and `rules` derives it (`Repeat::derive`) from what the data holds of the blueprint's
header, as the game's `BlueprintFeatureSelection.CanSelect` does (`Blueprints/Classes/Selection/
BlueprintFeatureSelection.cs`, WrathDecompiled 6ce4a6b): it refuses a feature held at its `Ranks`, unless the feature
is an `IFeatureSelection` (a `BlueprintFeatureSelection`, or a `BlueprintParametrizedFeature`, which picks a value), which
it refuses only once none of its items can be picked (`NestedFeatureSelectionUtils.AllNestedFeaturesUnavailable`).

- **`ranks`** (left out for 1): the blueprint's `Ranks`, how often a feature that isn't a selection is taken. Written on
  the nine Extra feats that are plain features (Extra Ki and Extra Reservoir 20, the other seven 10), the eight Extra
  talent feats (3, their header; a selection ignores it), the options of 2 or 3 ranks (Increased Damage Reduction,
  Swift Foot, Preserve Organs, the six incense fogs, the Stalwart Defender's Increased Damage Reduction) and New
  Strain (`PlagueHexIncreaseDamageStepFeature` 505648df95f5489fa13ddbd1af9473fd, Ranks 4: it was `Unlimited`).
- **A selection** is a feat, mythic feat or mythic ability that takes a value or grants a `selection` (302's nested
  form), and an option with `is_selection: true`: the rogue talents Weapon Focus (`WeaponFocus`
  1e1f627d26ad36f43bbd26cc2bf8ac7e, parametrised), the two Skill Focus talents (`SkillFocusSelection`
  c9629ef9eebb88b479b2fbc5e836656a), Combat Trick (`CombatTrick` c5158a6622d0b694a99efb1d0025d2c1), and two planner
  options, Finesse Training and the Battle Scion's Teamwork Feat, each with its `why`. One that takes a value is taken
  once per value; another while its selection (an option's: the one selection its lone level-1 row grants) has an
  option left. Not Witch Wandering Hex: its `selection` is the planner's (302), while the game's `WitchWanderingHex`
  fb8490d1721bd5a4883c3b6559310351 is a plain feature of one rank, so it says `plain_feature: true` and is taken once.
- **Its own conditions** (`Requires`, Group All): `HasNot` of itself makes it once-only, the game's own
  `PrerequisiteNoFeature`, now written on the selections that have one: Advanced Weapon Training, Second Bloodline,
  Second Bloodline (Bloodrager), Second Mystery, Second Spirit, and ten options (Pentamic Faith, Elemental Whispers,
  the Loremaster's Combat Feat, Rogue Secret and Wizard Feat, the oracle's Animal Companion, the rogue's Familiar,
  the slayer's three combat style talents). `Not(FactRank(<itself>, 3))` caps the eight Extra talent feats, the game's
  `PrerequisiteCondition` (`Not` of `ContextConditionBuffRank` on itself); `FactRank` takes any fact now, and a rank
  above 1 of anything but a feature compiles to the new prerequisite `Rank`.

**What moved** (decisions, the main session's, `game/303/decisions.md`; each from the game):

- Extra Feat (mythic) (`ExtraFeatMythicFeat` e10c4f18a6c8b4342afe6954bde0587b) and Extra Mythic Ability
  (`ExtraMythicAbilityMythicFeat` 8a6a511c55e67d04db328cc49aaad2b8), offered by `MythicFeatSelection`
  9ee0f6745f555484299b0a1563b99d81 (Mode Default), are taken again: once-only was ticket 55's reading of the guide's
  silence.
- Impossible Domain (`ExtraDomain` 213a8480d22206b45acbfa0619ca5aaf, Mode OnlyNew) is taken again for another domain.
- New Strain stops at four (no Build reaches a fifth: its five slots come at 2, 6, 10, 14 and 18 and it needs Prophet
  of Pestilence 10).
- Extra Rage Power, Combat Trick and the Battle Scion's Teamwork Feat repeat while their selection has an option left,
  as before but for a selection emptied.

**The planner's, under 293** (Q2, Q3; known to the game check): the options the planner keeps as rows of their own
though the game's are selections that repeat (Maneuver Mastery, Life-Bonding Friendship, and 13 more: the druid's
Animal Companion and Domain, Expanded Defense, three Loremaster spell secrets, the five ranger combat styles, Terrain
Mastery, the shaman's Hex Secret) stay once. The entries holding several blueprints have a planner `ranks`: Magical
Tail 8, the elements 2, the Primalist's rage powers 5; Beneficial Curse a planner `HasNot` of itself (once, over the
oracle's curses, where the game's selection lists the nine NoPenalty progressions).

**What it does not model.** `CanSelectAny` asks each item's prerequisites; the planner's "no option left" counts only
what is held (inference: an option whose prerequisite a later pick of the level meets still counts), and skips a
selection that lists nothing of its own (the first-had Extra feats', 293) or takes its lists from an option. The
selection modes `OnlyNew`/`OnlyRankUp` are the items' own repeats here (every domain, bloodline and curse is once).

**How.** `<scratchpad>/game/303/rewrite.py` (`python3 -I rewrite.py <data dir>`, run again moves nothing), from the
data at `ad501a66`: drops the 64 `repeat` fields, writes the `ranks`, the six `is_selection`, the eight caps and the
16 `HasNot` of themselves.

**Tests.** `libs/seeds/tests/game_data/repeats_from_the_header.rs`; `parameterised_feats.rs` (the feats and options
that repeat, New Strain's 4); `mythic.rs` (Extra Feat and Extra Mythic Ability repeat).
