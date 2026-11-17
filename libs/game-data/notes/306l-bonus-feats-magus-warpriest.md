# 306l: the magus's, the Arcane Rider's and the warpriest's bonus feat lists

## What the data says now

- **Three new feat lists**, each with `ignores_prerequisites` left at its default (false), as the game's `IgnorePrerequisites`:
  `magus/bonus-feats` (`MagusFeatSelection` 66befe7b24c42dd458952e3c47c93563, 177 feats), `magus/arcane-rider/bonus-feats`
  (`ArcaneRiderFeatSelection` 8e627812dc034b9db12fa396fdc9ec75, 177) and `warpriest/bonus-feats` (`WarpriestFeatSelection`
  303fd456ddb14437946e344bad9a893b, 151). The magus's and the rider's lists are the same set (the game lists one of the rider's
  feats, `TricksterImprovedImprovedImprovedCriticalImproved` 319c882a…, twice); the warpriest's is the magus's less its spell and
  metamagic feats.
- **Rows.** `classes/magus.ron` 5, 11 and 17 and the rows of armored-battlemage (5, 17), bladebound (17), eldritch-scion (11),
  hexcrafter (11), spell-dancer (17) and sword-saint (11) name `magus/bonus-feats` where they named the general `feats`;
  `classes/warpriest.ron` 3, 6, 9, 12, 15, 18 and champion-of-the-faith 12, cult-leader 6 and 18, feral-champion 9 and 15,
  mantis-zealot 18 and shieldbearer 12 name `warpriest/bonus-feats` where they named `combat-feats`. Each row held one
  selection, so no slot id moves (`FeatSource::Class`).
- **Arcane Rider** had no rows at 5 and 17 (it inherited the magus's, which the game's archetype removes) and used the general list at 11.
  It now has rows 5 and 17 (each keeping the magus's Arcane Weapon grant of that level) and row 11, all naming
  `magus/arcane-rider/bonus-feats` (`ArcaneRiderArchetype` 2dff80a0… `AddFeatures` at 5, 11, 17; `RemoveFeatures` `MagusFeatSelection` at 5, 11, 17).
- The magus's list was the general list (every feat): picks outside the game's list are refused on old links (see `306-refused.txt`).

## Written by the writer, and by hand

The writer wrote nothing: a selection with a member that has no feat entry is left unwritten whole, and each of the three lists
holds four: `AdvancedWeaponTraining2`, `3`, `4` (306q) and `BleedingCriticalFeature` (306r). So the lists were written by hand:
the game's `m_AllFeatures` mapped through each feat's `blueprint`, in key order, the four left out.

## Left in `known.txt`

Twelve lines (the four members, for each list), under 306q's and 306r's file names, which add the feats. Removed: 283's
`progression rows / a bonus pick of another group / archetype magus/hexcrafter`.

## Test

`importer_lists::the_magus_arcane_rider_and_warpriest_bonus_feats_are_their_own`.
