# 306m: the cavalier's, its Tactician's and the Gendarme's feat lists

## What the data says now

- **Three feat lists of their own**, each with `ignores_prerequisites` at its default (false), as the game's `IgnorePrerequisites`:
  `cavalier/bonus-feats` (`CavalierBonusFeatSelection` dd17090d14958ef48ba601688b611970, 142 feats; the old `mounted-combat-feats`,
  renamed: it held the five Mounted Combat feats), `cavalier/tactician-feats` (`CavalierTacticianFeatSelection`
  7bc55b5e381358c45b42153b8b2603a6, 12 teamwork feats) and `cavalier/gendarme/bonus-feats` (`GendarmeFeatSelection`
  ef1cd58e0b7fc7f45baedb09407a1cd1, 151 feats).
- **Rows.** `classes/cavalier.ron` 6, 12, 18 and disciple-of-the-pike 6, 12, fearsome-leader 6, 12, 18 name `cavalier/bonus-feats`
  (they named `combat-feats`); cavalier 1, 9, 17, disciple-of-the-pike 1, 9, 17 and beast-rider, standard-bearer and
  cavalier-of-the-paw 1 name `cavalier/tactician-feats` (they named `teamwork-feats`); the Gendarme names its own list at 1, 3, 9, 15, 20 and now
  has rows 6, 12 and 18 for it (`GendarmeArchetype` `AddFeatures` at 1, 3, 6, 9, 12, 15, 18, 20; `RemoveFeatures` the cavalier's list at 6, 12, 18).
  The cavalier-of-the-paw's row 8 still names `cavalier/bonus-feats` where the game has `OrderOfThePawCanineFerocitySelection`: a row question (309).
- Slot ids do not move: each row held one feat list, and feat lists and option selections number apart.
- The general `teamwork-feats` stays for the inquisitor and the others. The Mounted Combat category pool is gone (`FEAT_POOLS` of the tests).

## Written by the writer, and by hand

The writer wrote `cavalier/tactician-feats` (all 12 members map). It left the other two unwritten: each holds four members with no feat
entry (`AdvancedWeaponTraining2`, `3`, `4`: 306q; `BleedingCriticalFeature`: 306r). Those two were written by hand: the game's
`m_AllFeatures` mapped through each feat's `blueprint`, the four left out. `known.txt`: eight lines under 306q and 306r; removed:
293's `mounted-combat-feats` and two stale cavalier-of-the-paw row lines (273: a bonus pick of another group, at another level).

## Test

`importer_lists::the_cavaliers_three_feat_lists_are_the_games` pins each list's full member set by the game's GUIDs.
