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
- Mythic: `abundant-smite-smite-chaos` now holds `AbundantSmiteChaos` 4cdc155e… (needs Smite Chaos); new `abundant-smite` (`AbundantSmite` 7e5b63fa…, needs Smite Evil); no alias (mythic abilities can't migrate); both in `mythic/extra-mythic-ability`.
- New feats `extra-rage-instinctual-warrior` (dcc33811…), `extra-performance-skald` (15c78493…), `extra-performance-martyr` (639ce2b8…), each aliased to the old key, `ranks: 10`, own words.
- Lists: the writer added the new feats to the pool `feats`; by hand `combat-feats` gained AWT 2 to 4 and Bleeding Critical (the game's `FighterFeatSelection` holds them; the category test requires it) and the Dragon Feats list the 10 the game's list holds (not AWT 2 to 4, which `DragonLevel2FeatSelection` lacks; builder's decision, 306n writes the rest).
- Completed the hand-written lists: `magus/bonus-feats`, `magus/arcane-rider/bonus-feats`, `warpriest/bonus-feats`, `cavalier/bonus-feats`, `cavalier/gendarme/bonus-feats` now hold AWT 2 to 4 and Bleeding Critical (the writer, `--only`); the 306l and 306m tests pin the full member sets (181, 181, 155, 146, 155).
- `known.txt`: 20 lines of 306q/306r removed, plus the `pool general feats` lines (293, 263), the `magical-tail` ranks line, the `AbundantSmiteChaos` pool line, and the stale `feats` wildcard and `abundant-smite-smite-chaos` prerequisite lines; added `prerequisites / * / feat extra-channel-shaman -> 264`. The game check exits 0 (3,860 differences in 192 groups, 0 not known).
- Tests updated for the counts: repeating feats 37 to 45, feats with `Requires` 635 to 649, the Dragon Feats list.

**Tests** (`libs/seeds/tests/game_data/importer_lists.rs`; written after the data, so I didn't watch them fail first): `advanced_weapon_training_is_four_feats_each_after_the_last` (AWT blueprints above, `Ranks` 1, fighter levels and predecessors from the `PrerequisiteFeature` and `PrerequisiteClassLevel` of each), `magical_tail_is_the_games_one_feat_of_eight_ranks` (`MagicalTail` febb8fe9…, `Ranks` 8), `abundant_smite_is_two_mythic_abilities` (7e5b63fa…, 4cdc155e…). `advanced_weapon_training_needs_the_fighters_weapon_training` stays ignored (the planner's AWT1 prerequisite still names the Disciple of the Pike's and Sohei's trainings, 263).

**Proof (offers, base `e9303e88`, recorded in this lane before the first edit):** Builds 2,158 compared, 1,994 open, before and after; no Build lost or added; the generators name none of these feats. Links: 1,990 the same, 0 through an heir (no corpus Build repeats Advanced Weapon Training or picks an Extra feat that split), 4 refused, 0 changed. The 4 are `full t54 divine-guardian / empty`, `full t54 divine-hunter-paladin / empty` and their `/ legend` copies: their mythic rank 3 holds `abundant-smite-smite-chaos` with Smite Evil, which now means the Chaos one (listed in `306-refused.txt`, own block). Offers: 18,038,927 recorded, 18,269,058 now (230,131 only now: the new feats). Statuses changed: 9,024 in 191 (pick, kinds) groups, in 44 Builds (`FORM_PROOF_OFFER_BUILDS_OUT`: the paladin, Hospitaler, Warrior of the Holy Light, Divine Guardian/Hunter and Empyreal Sorcerer mythic Builds): `abundant-smite-smite-chaos` ok to Unmet (139 in 41 Builds, game: needs Smite Chaos, and `abundant-smite` is offered), `extra-channel` ok to Unmet (66 in 10 Builds: the Hospitaler's and Empyreal Sorcerer's channels moved to their own feats), the rest ok to BreaksLater in those Builds (the 4 refused ones for 4-Build groups; the 1- and 2-Build groups I did not trace Build by Build). Nine Builds' generated picks changed with the grown pools (the four refused; `t60 skald 20` and `t60 skald/hunt-caller 20 / filled` and `t66 fighter 5 instinctual warrior 8 purity / filled`, each plain and `/ legend` where listed): they drew Extra Performance (Skald) or Extra Rage (Instinctual Warrior). No slot-order refusal; no offer moves away from the game.
