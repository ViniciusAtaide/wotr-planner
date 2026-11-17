# Importer S2, 306q: feats holding several blueprints I

Type: build (the importer, stage S2)
Status: resolved
Blocked by: 306o
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [306](306-importer-selections.md), the importer's stage S2, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof, the expected counts and the files it shares are under its heading in [research/306/slices.md](../research/306/slices.md). The decisions are in [research/306/decisions.md](../research/306/decisions.md) and that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built with 306r as one batch (branch `ticket/306q-multi-blueprint-feats-1`, base `e9303e88`). The note is `libs/game-data/notes/306q-multi-blueprint-feats.md`.

**What was built**
- `advanced-weapon-training-2/3/4` (`AdvancedWeaponTraining2` 70a139f0…, `3` ee9ab011…, `4` 0b55d725…): each needs the feat before, fighter 10/15/20 and not itself; `-2` aliases `advanced-weapon-training` (the only heir that holds).
- `magical-tail` takes `MagicalTail` febb8fe9… (`ranks: 8`), a remap, not a split (slices decision 9).
- Mythic: `abundant-smite-smite-chaos` keeps its key and its blueprint `AbundantSmite` 7e5b63fa… (name "Abundant Smite", needs Smite Evil): a mythic split keeps every old key's meaning (decisions §0 item 5; slices.md's line that moved the key contradicted it, review fix). New `abundant-smite-chaos` (`AbundantSmiteChaos` 4cdc155e…, needs the Hellknight's Smite Chaos). Both are in `mythic/extra-mythic-ability`.
- New feats `extra-rage-instinctual-warrior` (dcc33811…), `extra-performance-skald` (15c78493…), `extra-performance-martyr` (639ce2b8…), each aliased to the old key, `ranks: 10`, own words.
- Lists: the writer added the new feats to the pool `feats`; by hand `combat-feats` gained AWT 2 to 4 and Bleeding Critical (the game's `FighterFeatSelection` holds them; the category test requires it) and the Dragon Feats list the 10 the game's list holds (not AWT 2 to 4, which `DragonLevel2FeatSelection` lacks; builder's decision, 306n writes the rest).
- Completed the hand-written lists: `magus/bonus-feats`, `magus/arcane-rider/bonus-feats`, `warpriest/bonus-feats`, `cavalier/bonus-feats`, `cavalier/gendarme/bonus-feats` now hold AWT 2 to 4 and Bleeding Critical (the writer, `--only`); the 306l and 306m tests pin the full member sets (181, 181, 155, 146, 155).
- `known.txt`: 20 lines of 306q/306r removed, plus the `pool general feats` lines (293, 263), the `magical-tail` ranks line, the `AbundantSmiteChaos` pool line, and the stale `feats` wildcard and `abundant-smite-smite-chaos` prerequisite lines; added two lines for `extra-channel-shaman` under 307 (see 306r); Bleeding Critical and Spell Bane carry no `guide` field (the saved guide has no page for them). The game check exits 0 (3,861 differences in 194 groups, 0 not known).
- Tests updated for the counts: repeating feats 37 to 45, feats with `Requires` 635 to 649, the Dragon Feats list.

**Tests** (`libs/seeds/tests/game_data/importer_lists.rs`; written after the data, so I didn't watch them fail first): `advanced_weapon_training_is_four_feats_each_after_the_last` (AWT blueprints above, `Ranks` 1, fighter levels and predecessors from the `PrerequisiteFeature` and `PrerequisiteClassLevel` of each), `magical_tail_is_the_games_one_feat_of_eight_ranks` (`MagicalTail` febb8fe9…, `Ranks` 8), `abundant_smite_is_two_mythic_abilities` (7e5b63fa…, 4cdc155e…). `advanced_weapon_training_needs_the_fighters_weapon_training` stays ignored (the planner's AWT1 prerequisite still names the Disciple of the Pike's and Sohei's trainings, 263).

**Proof (offers, base `e9303e88`, recorded in this lane before the first edit; rerun after the review fix):** Builds 2,158 compared, 1,994 open, before and after; none lost or added; the generators name none of these feats. Links: 1,994 the same, 0 through an heir (no corpus Build repeats Advanced Weapon Training or picks an Extra feat that split), 0 refused, 0 changed; `306-refused.txt` is unchanged. Offers: 18,038,927 recorded, 18,269,058 now (230,131 only now: the new feats' options). Statuses changed: 87 in 4 (pick, kinds) groups, in 17 Builds (`FORM_PROOF_OFFER_BUILDS_OUT`). Each moves toward the game: `extra-channel` ok to Unmet (66) and BreaksLater to Unmet (8), in 10 Builds, the Hospitaler (`t54 hospitaler` empty/filled and legend, `paladin paladin/hospitaler`) and the Empyreal Sorcerer (`full sorcerer sorcerer/empyreal-sorcerer`, legend, plain): `ExtraChannel` cd9f1977… needs `ChannelEnergyFeature` or `ChannelNegativeFeature`, and their channels are `ChannelEnergyHospitalerFeature` and `ChannelEnergyEmpyrealFeature`, which have their own Extra Channel feats; `abundant-smite-smite-chaos` ok to Unmet (2) and BreaksLater to Unmet (11), in 7 Builds, the Hellknight Builds (`cleric hellknight`, `witch hellknight`, `wizard hellknight`, `full targeted hellknight-pentamic` and its `-impossible` copy and the plain ones): `AbundantSmite` 7e5b63fa… needs `SmiteEvilFeature`, which a Hellknight lacks, and `abundant-smite-chaos` is offered. Generated picks changed in 5 Builds (the pools grew): `t60 skald 20 / empty` (and legend), `t60 skald/hunt-caller 20 / filled` (and legend) drew Extra Performance (Skald) and `t66 fighter 5 instinctual warrior 8 purity / filled` drew Extra Rage (Instinctual Warrior); the numbers of the two `t60 skald 20` Builds lose Skill Focus (Mobility). The Divine Guardian and Divine Hunter paladins' four Builds differ in the sheet only by the ability's name. No slot-order refusal; no offer moves away from the game.
