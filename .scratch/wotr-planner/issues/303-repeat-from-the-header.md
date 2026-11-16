# Repeat from the header

Type: build (data form and engine)
Status: resolved
Blocked by: 302
Model: Opus to build: every repeat rule changes source.
Map: [WotR Build Planner](../map.md)

## Problem

Split from [298](298-selections-as-the-game-has-them.md) (user, 2026-11-16, "B: forms now, splits in 293"): one of the staged component plan's five form steps (user, 2026-11-15; [295](295-the-component-vocabulary.md)). The design is in [research/component-vocabulary-2026-11-15.md](../research/component-vocabulary-2026-11-15.md), §3 and §4. The per-class selection splits are [293](293-the-importer.md)'s, not this ticket's.

**What the planner holds today:** `Repeat`, a hand-written rule for how often an option may be taken.

## Change

- **`Repeat` is derived from the blueprint header the data holds:** `Ranks`, whether it is a selection or parametrised, and `PrerequisiteCondition`, as the game's code decides a feature's repeat (cite the method).
- **The field goes.** Where the derived rule and today's differ, the Answer lists each with the game's citation, and a real-data test holds the game's value.

## Answer

**Built** (2026-11-16) on the main session's decisions (`<scratchpad>/game/303/decisions.md`), with the ones of mine
below. Note: `libs/game-data/notes/303-repeat-from-the-header.md`.

**The rule.** `Repeat` is no longer data: `Repeat::derive` (`libs/rules/src/parameter.rs`), read through `repeat()` on
`Feat`, `SelectionOption`, `MythicAbility` and `MythicFeat`, derives it as `BlueprintFeatureSelection.CanSelect` does
(`Blueprints/Classes/Selection/BlueprintFeatureSelection.cs`, WrathDecompiled 6ce4a6b): a feature that isn't an
`IFeatureSelection` stops at its `Ranks` (`ranks`, default 1, written only above 1); a selection (a feat, mythic feat or
mythic ability that takes a value or grants a `selection`; an option with `is_selection: true`) is taken once per
value when it takes one, else while it has an option left (`NestedFeatureSelectionUtils.AllNestedFeaturesUnavailable`),
once with a `Not` of itself in its prerequisite's `All` (its own `PrerequisiteNoFeature`), up to `n` times with a `Not`
of its own rank `n` (its own `PrerequisiteCondition`, `ContextConditionBuffRank`). `Unlimited` is gone. New:
`Repeat::WhileOptionLeft`, `Issue::NoOptionLeft(selection)` ("No … left to take"; MCP `no_option_left`),
`Catalog::has_option_left`, `Prerequisite::Rank { fact, rank }` (a `FactRank` above 1 of a fact that isn't a feature;
`FactRank` takes a `Fact`), `SelectionOption::is_selection` and `nested_selection`, `MythicAbility::plain_feature`, `CharacterState::mythic_feat_holds`,
`mythic_ability_count` and `mythic_feat_count` (a mythic feat or ability held twice; `Fact::rank` reads them), and the
walk opens a mythic pick's selection whenever its count grows. The check: `DataIssue::NoRanks`,
`SelectionWithoutPick`; `PerValueWithoutParameter`, `PerValueOptionWithoutValue` and `TimesOutOfRange` are gone.

**The data** (`<scratchpad>/game/303/rewrite.py`, from `ad501a66`, run again moves nothing): 80 files; the 64 `repeat`
fields dropped; `ranks` on 34 entries; `is_selection` on 6 options; the eight Extra talent feats' cap
`Not(FactRank(Feat(<itself>), 3))`; `HasNot(<itself>)` on 16 entries; `plain_feature` on Witch Wandering Hex. The seeds and game-check fixtures moved the same
way. `libs/game-data/README.md`: the stale `extra_pick_of`/`extra_pick` lines (302's review) reworded to the nested form,
and the `repeat` lines to the derivation.

**The game check** (exits 0): the own-`NoFeature` skip is narrowed to non-selections, so the 15 selections' own
`HasNot` is compared and agrees; a new mapping comparison, `ranks differ`, finds 6, each known under 293 (Magical Tail,
the Primalist's rage powers, the four elements); Beneficial Curse's planner `HasNot` is known under 293.

**Decisions.**
- Main session: Q1–Q5 as in `decisions.md`.
- Mine: the options take their selection-ness from a header field, `is_selection`, not their rows: no shape of the
  rows separates Combat Trick and the Battle Scion's Teamwork Feat (repeating today) from Maneuver Mastery, the druid's
  Domain or the Kineticist Bonus Feat (once today, Q2's 293), and an option that takes a value (Battle Master, Weapon
  Mastery, Way of the Bow's bow) is no parametrised feature in the game. Written on Combat Trick, the rogue Weapon
  Focus and the two Skill Focus talents (game), Finesse Training and the Battle Scion's Teamwork Feat (planner, each
  with its `why`): the second planner exception beside Q4's.
- Mine: Beneficial Curse stays once by a planner `HasNot` of itself rather than `ranks` (Q3): a selection's `ranks`
  doesn't bound it in the game's rule.
- Main session (mid-build): Witch Wandering Hex stays once. Its `selection` is 302's planner one, while
  `WitchWanderingHex` fb8490d1721bd5a4883c3b6559310351 is a `BlueprintFeature` of Ranks 1 (`CanSelect`); the first
  offer count showed it opening (41 statuses), so the ability says `plain_feature: true` and derives by its `ranks`.
- Mine: "no option left" counts what is held, not the options' prerequisites, and skips a first-had selection
  (inference).
- Mine: the own conditions stay in the compiled prerequisite (so the game check compares them, and a picker can read
  them); the tests that pin a prerequisite tree say them.

**Simplify** (four review angles): `Repeat::derive` takes the entry's own `Fact` (built only for a selection with no
value) in place of two closures per kind; `check_repeat` takes whether the pick's value is held; `part_name`
(`walk/bonuses.rs`) reuses `fact_name`; the game check indexes `ranks` once; `mythic_feat_mut` moved to the rules
tests' support. Left for a later form ticket: an option's `selection` in place of `is_selection` (it would move the
level-1 rows), `Rank` absorbing `Feature { count }` (the data's form), `mythic_feats` as a count map (a public field
read across crates), one `is_feature_selection` in the game check.

**Tests.**
- Rules, hand-made (`libs/rules/tests/engine/repeats.rs`, each Build also judged by `rules::agreement::judge`):
  `the_repeat_is_derived_from_the_header`, `a_selection_is_taken_again_while_it_has_an_option_left`,
  `a_selection_with_no_option_left_is_refused`, `a_selection_not_of_itself_is_taken_once`,
  `a_selection_capped_at_its_rank_stops_there`, `a_feature_stops_at_its_ranks`. Rewritten onto the derived value:
  `parameters.rs` (`a_parameterised_feat_is_taken_once_per_value` replaces `…_taken_once_is_held_for_one_value`: the
  game's parametrised feature is once per value), `check.rs` (`ranks_and_chosen_feats_fit_their_feats`,
  `an_option_is_a_selection_only_over_a_value_or_a_granted_selection`,
  `a_mythic_feat_has_a_rank_and_a_value_for_the_same_value`, `a_mythic_ability_has_a_rank`), and the struct literals.
- Real data (`libs/seeds/tests/game_data/repeats_from_the_header.rs`): `extra_feat_mythic_is_taken_twice` and
  `extra_mythic_ability_is_taken_twice` (`ExtraFeatMythicFeat` e10c4f18a6c8b4342afe6954bde0587b,
  `ExtraMythicAbilityMythicFeat` 8a6a511c55e67d04db328cc49aaad2b8, offered by `MythicFeatSelection`
  9ee0f6745f555484299b0a1563b99d81, Mode Default, no own NoFeature or Condition; `CanSelect`),
  `impossible_domain_is_taken_twice_for_another_domain` (`ExtraDomain` 213a8480d22206b45acbfa0619ca5aaf, Mode OnlyNew,
  offered by `MythicAbilitySelection` ba0e5a900b775be4a99702f1ed08914d), `second_bloodline_is_still_taken_once`
  (`SecondBloodline` 3cf2ab2c320b73347a7c21cf0d0995bd's own PrerequisiteNoFeature), `witch_wandering_hex_is_taken_once`
  (`WitchWanderingHex` fb8490d1…, Ranks 1), `the_mythic_entries_repeat_as_their_headers_say`,
  `the_selections_with_their_own_no_feature_are_taken_once` (the 15 GUIDs). `parameterised_feats.rs`'s options table
  now holds New Strain `Times(4)` (`PlagueHexIncreaseDamageStepFeature` 505648df95f5489fa13ddbd1af9473fd, Ranks 4,
  offered by `PlagueHexSelection` 68db664d2b5046eab2e6fd267f5d017d: the fifth refused; no Build reaches five, its
  slots at 2, 6, 10, 14, 18 and it needs Prophet of Pestilence 10) and the rogue Weapon Focus `PerValue`; the ignored
  `plague_mutation_new_strain_repeats_four_times` (waiting on 278) is folded into it. Moved onto the derived value
  with their values unchanged: the 249/250 tests (`parameterised_feats.rs`, `repeating_talents.rs`,
  `skald_rage_powers.rs`, `pick_values.rs`, `finesse_training.rs`, `follow_ups.rs`, `instinctual_rage_powers.rs`,
  `files.rs` and its fixture), Combat Trick, the Battle Scion's Teamwork Feat and Extra Rage Power from `Unlimited` to
  `WhileOptionLeft`, the Primalist's rage powers to `Times(5)`, `mythic.rs`'s Extra Feat and Extra Mythic Ability to
  `WhileOptionLeft` (the allowed change). The tests that pin a prerequisite tree say the own conditions
  (`and_not`, `own_rank` in `support.rs`): `wild_talents.rs` (2), `report_gaps.rs`, `mythic.rs` (Beneficial Curse),
  `negated_prerequisites.rs` (+feat, mythic ability, rank), `prerequisites_as_components.rs` (608 → 611: the three
  Loremaster secrets). Game check: the fixture's `a_selections_own_no_feature_and_a_feats_ranks_are_compared`.

**Proof.** `record` on `ad501a66` in a temporary worktree (removed), `compare` on the tip: 2,162 Builds (2,001 open),
2,162 old links reopened, every one as before (no reopen difference); 825 differences, all in the corpus's generated
`full …` Builds: 147 draw other picks (decoded picks, picks, slots), 153 another link and sheet (the 147 and six of
their Legend switches, whose skill draws moved), 78 other numbers. `fill_full` draws each slot's pick at random among
its open options, so an option newly open shifts every later draw of the Build; no hand-made, bench or link Build
moved. The offer count (`<scratchpad>/game/303/offers_dump.rs`, a one-off on the base's 2,160 Builds' links, base
engine against tip engine; compared by the main session's `offers-diff`): 18,078,022 options each side, none only on
one side, 685 statuses changed, each an allowed one: Extra Feat (mythic) `AlreadyHeld`→ok 162, `BreaksLater`→ok 178,
`AlreadyHeld`→`BreaksLater` 8 (107 Builds); Extra Mythic Ability 150, 161, 5 (101 Builds); Impossible Domain
`AlreadyHeld`→ok 21 (6 Builds). No corpus Build holds New Strain five times. `cargo test -p corpus --test links`
passes.

## Done when

- The form proof (`record` on the base in a temporary worktree, `compare` on the tip) shows 0 differences, or each difference is explained in the Answer and comes from the game, cited.
- Every corpus and bench link opens with the same picks (298's link test).
- The self-checks' short runs, the gold builds and every real-data test are unchanged.
- The scoped checks pass, and the game check exits 0.

