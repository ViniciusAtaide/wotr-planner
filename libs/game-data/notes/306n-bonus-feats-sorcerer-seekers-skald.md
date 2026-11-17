# 306n: the sorcerer's, the Seekers' and the skald's bonus feat lists

## What the data says now

- **Four feat lists of their own**, each with `ignores_prerequisites` at its default (false), as the game's `IgnorePrerequisites`, each named "Magic feat" (the sheet's label for these slots, ticket 92, stays):
  - `sorcerer/bonus-feat` (`SorcererBonusFeat` d6dd06f454b34014ab0903cb1ed2ade3, 21 feats), named at `classes/sorcerer.ron` row 1 and at row 1 of the Crossblooded, Empyreal, Geomancer, Nine-Tailed Heir, Sage, Seeker and Sylvan sorcerers. The Overwhelming Mage loses it (`archetype_losses.ron` names it instead of `magic-feats`).
  - `sorcerer/seeker-sorcerer/bonus-feats` and `oracle/seeker/bonus-feats`, both `SeekerFeatSelection` c6b609279cc3174478624182ac1ad812 (26 feats: the sorcerer's 21 and Ambuscading Spells, Arcane Spirit, Dispel Focus, Greater Dispel Focus, Spell Chain), at rows 5, 10 and 15 of each Seeker. Two entries name one blueprint because the game's one selection is added by two archetypes (`SeekerArchetype` 7229db6f… for the sorcerer, c539d65b… for the oracle); a list is the planner's per class.
  - `skald/bonus-feat` (`SkaldFeatSelection` 0a1999535b4f77b4d89f689a385e5ec9, the same 26), at skald row 1 and the Battle Scion, Court Poet, Herald of the Horn and Inciter rows 1.
- The Seeker sorcerer keeps the sorcerer's level-1 list (`SeekerArchetype` removes only `SorcererBloodlineSelection` at 1), so its row 1 names `sorcerer/bonus-feat`.
- `magic-feats` stays for the wizard's lists; the game's `WizardFeatSelection` is a later slice's.
- Slot ids do not move: each row held one feat list, and feat lists and option selections number apart.

## User fact 281

Ticket 281 (user, 2026-11-15): the game's 21 replace ticket 92's Magic feats for the sorcerer's level-1 bonus feat, and the Seeker's list is its own 26, "applied without asking again". `tools/game-check/user-facts.txt` holds `applied 281` for the three lists. No `held` fact is touched. The skald's list is no fact of the user's: it is the game's.

## Written by the writer

All four lists, by `game-check --write option-lists --only` the four keys: every member of each game list maps to one feat. Each file was created with one seed feat (a selection with no feats is no feat list to the writer), and the writer replaced it.

## Tests

`importer_lists::the_sorcerers_seekers_and_skalds_bonus_feats_are_the_games` pins each list's blueprint and full member set by the game's GUIDs, and the rows. `report_gaps::every_sorcerers_level_1_bonus_feat_is_one_of_the_games_21` and `the_seekers_feats_at_5_10_and_15_are_its_own_26` (ignored since 242) now run.
