# 306s: the general and group feat lists

## What the data says now

- **`combat-feats`** is the game's `FighterFeatSelection` 41c8486641f7d6d4283ca9dae4147a9f (the list the fighter's bonus feat, the Forester's, the Titan Fighter's and the other combat-feat slots offer). It gains the teamwork feats the game lists (Back to Back, Coordinated Defense and Maneuvers, Outflank, Precise Strike, Seize the Moment, Shield Wall, Tandem Trip, Volley Fire), Acrobatic Spellcaster, Arcane Armor Mastery and Training, Arcane Strike, Natural Spell, Intimidating Prowess and the Diva's three; and loses Diehard, Dragon Roar, Great Fortitude, Iron Will, Lightning Reflexes, their Improved feats, Stat Focus, Throw Anything and Toughness (the game's list holds none of them).
- **`magic-feats`** is the game's `WizardFeatSelection` 8c3102c2ff3b69444b139a98521a4899. It gains Spell Chain, and loses Acrobatic Spellcaster, Arcane Armor Mastery and Training, Arcane Strike, Brew Potions, Destructive Dispel, Dispel Synergy, Favored Enemy Spellcasting, Completely Normal Spell and Scribe Scrolls.
- **`teamwork-feats`** is the game's `TeamworkFeat` d87e2f6a9278ac04caeb0f93eff95fcb: it loses Spell Chain (the game lists it in the wizard's list, not the teamwork one).
- **`feats`** (`BasicFeatSelection` 247a4068296e8be42890143f451b4b45) was already the game's list: the writer finds nothing to change.
- **`mythic/extra-mythic-ability`** loses Emergency Help (`EmergencyHelpFeature` 22ce17c5253183b42845a5bfc54f8afa) and Any Means Necessary (`FreeFromEvil` a0a814897cd33ed4ea6ffca4b0e1197a), which `ExtraMythicAbilityMythicFeat` 8a6a511c55e67d04db328cc49aaad2b8 does not list. The writer refuses the selection as a whole (all or nothing): the game's list holds `SecondMysteryEnlightenedPhilosopher` 4ff6f2905e1f4d1b92930b87f85bf86c, which has no mythic ability until 306v. The two removals are by hand, and `known.txt` keeps `option lists / in game only / selection mythic/extra-mythic-ability -> 306v` for that one member.
- **`selections/mythic/extra-feat.ron`** is new: `ExtraFeatMythicFeat` e10c4f18a6c8b4342afe6954bde0587b is a selection of its own, whose list is `BasicFeatSelection`'s but for five Trickster feats (Stat Focus, Completely Normal Spell, the three Improved Improved Critical tiers: `TricksterStatFocusFeatSelection` 0d1d80bd…, `CompletelyNormalSpellFeat` 094b6278…, `TricksterImprovedImprovedCritical` 56f94bad…, `TricksterImprovedImprovedImprovedCritical` 006a9660…, `TricksterImprovedImprovedImprovedCriticalImproved` 319c882a…). The mythic feat `extra-feat` names it (`selection: "mythic/extra-feat"`) where it named `feats`, so its slot offers the game's list. Its slot's kind is `Feat("mythic/extra-feat")`, not `Feat("feats")`. A link carries no selection key, so no link moves.
- `libs/rules/src/check.rs`: a selection of feats that a mythic feat names counts as named (`UnusedFeatList` no longer flags `mythic/extra-feat`).

## Not here

The Hermit's and the Herbalist's second mysteries stay outside Extra Mythic Ability (the game's list lacks them); the Philosopher's is 306v's.

## Tests

`nested_selections::the_combat_magic_and_teamwork_lists_are_the_games` (the three lists against the game, by GUID), `extra_mythic_ability_and_extra_feat_grant_their_selections` (the two lists and their blueprints), and the two 265 tests un-ignored: `mythic::extra_feat_offers_the_general_slots_feats_but_the_tricksters_five`, `mythic::extra_mythic_ability_leaves_out_emergency_help_and_any_means_necessary`.
