# Feat slots and lists as selections

Type: build (data form and engine)
Status: resolved
Blocked by: 298
Model: Opus to build: every feat slot changes shape, and every link must keep opening.
Map: [WotR Build Planner](../map.md)

## Problem

Split from [298](298-selections-as-the-game-has-them.md) (user, 2026-11-16, "B: forms now, splits in 293"): one of the staged component plan's five form steps (user, 2026-11-15; [295](295-the-component-vocabulary.md)). The design is in [research/component-vocabulary-2026-11-15.md](../research/component-vocabulary-2026-11-15.md), §3 and §4. The per-class selection splits are [293](293-the-importer.md)'s, not this ticket's.

**What the planner holds today:** feat slots and named feat lists, where the game lists each option in a selection (`BasicFeatSelection` names 276 feats; the fighter's, the magus's and the other bonus-feat lists likewise).

## Change

- **Feat slots and feat lists become selections:** a selection with its options listed, as the game's are, granted on a progression row like any other selection.
- **A re-runnable rewrite script, kept in the main session's scratchpad (`game/301/`),** moves each from the current data. Where the planner's list and the game's differ, this ticket keeps the planner's list; 293 writes the game's.
- **The engine** offers a feat slot through the selection path. The slot ids a link names keep decoding (through 298's migration if an id changes).
- **The fields go** where nothing still needs them.

## Done when

- The form proof (`record` on the base in a temporary worktree, `compare` on the tip) shows 0 differences, or each difference is explained in the Answer and comes from the game, cited.
- Every corpus and bench link opens with the same picks (298's link test).
- The self-checks' short runs, the gold builds and every real-data test are unchanged.
- The scoped checks pass, and the game check exits 0.


## Answer

**Built** (2026-11-16) as written, on the main session's decisions (`<scratchpad>/game/301/decisions.md`). Note:
`libs/game-data/notes/301-feat-slots-and-lists-as-selections.md`.

**The rewrite** is a Rust crate outside the repo (`<scratchpad>/game/301/rewrite/`, `cargo run --release -- <data dir>
[<kind-map.tsv>]`), run on the data at `49306dff`, not on the game. It reads the old forms with its own copies of the old
types, writes the selections, moves each row's, race's, subrace's and path rank's `feat_slots` into its `selections`
(after the keys it named), deletes `feat-lists/`, and reads everything back with the new types: each row's selections
are its old selections and its slots' keys, its selections of feats in its slots' old order, and each new selection is
the list or slot it stands for. Run again, it moves nothing. Moved: 78 feat lists to `selections/` (the Crusader's
comment kept; Dragon Feats' `every_feat` as today's 261 feats); 313 `feat_slots` fields in 108 files (315 slots: 107
lists, 81 Combat, 48 option-list, 35 Magic, 31 Teamwork, 12 general, 1 Mounted Combat); 12 new selections:

| Key | Was | Game blueprint |
| --- | --- | --- |
| `feats` ("Feat", 261 feats) | `General`, the odd-level slot, Extra Feat's | `BasicFeatSelection` 247a4068296e8be42890143f451b4b45 |
| `combat-feats` (144) | `CombatFeat` | `FighterFeatSelection` 41c8486641f7d6d4283ca9dae4147a9f |
| `magic-feats` (35) | `Category(Magic)` | `WizardFeatSelection` 8c3102c2ff3b69444b139a98521a4899 |
| `teamwork-feats` (13) | `Category(Teamwork)` | `TeamworkFeat` d87e2f6a9278ac04caeb0f93eff95fcb (`InquisitorProgression` grants it) |
| `mounted-combat-feats` (5) | `Category(MountedCombat)` | `CavalierBonusFeatSelection` dd17090d14958ef48ba601688b611970 |
| `sorcerer/bloodline-feats` ("Bloodline feat") | `OptionList(sorcerer/bloodline)` | `SorcererFeatSelection` 3a60f0c0442acfb419b0c03b584e1394 |
| `bloodrager/bloodline-feats`, `ranger/combat-style-feats`, `slayer/combat-style-feats-2`, `-6`, `-10`, `kineticist/wild-talent-feats` | the other option-list slots | none: `planner/selections/`, each with its reason (one game selection per option, 302) |

Also run on the seeds fixture (15 fields in 12 files, 7 lists); game-check's `planner.ron` fixture edited by hand, with
`FighterFeatSelection` added to its dump.

**The form.** `FeatureSelection` gains `feats`, `ignores_prerequisites`, `skips_proficiency` and `option_lists:
Option<OptionLists { selection, at_level }>` (decision 1, until 302); `options` defaults to empty; `offers_feats()` says
which kind of slot it makes. `Race` and `Subrace` name `selections`; `OptionLevelRow::feat_list` names a selection;
`CharacterState::option_feat_lists` maps to selection keys. **Gone:** `FeatSlot`, `FeatList`, `FeatListKey`,
`GameData::feat_lists`, every `feat_slots` (rows, races, subraces, path ranks, `EffectiveRow`), `every_feat`,
`DataAt::FeatList`, `DataIssue::EveryFeatListWithFeats`, `Engine::feat_list`. **Kept for 302–305:** every `SlotId`,
`SlotKind::Feat` (now carrying a `FeatureSelectionKey`), `extra_pick_of`, `MythicExtraPick`, `once_per_option`,
`feat_lists_only`, `Repeat`, `option_levels`, the state maps.

**The engine** makes a Feat slot of a row's selection of feats (`Catalog::split_selections`, the one rule: a row's
selections of feats and its others are numbered apart, each in row order, so every slot id is the one its feat slot had;
an option row's and a path rank's pending slots keep their old order). The general slot and Extra Feat's name
`BASIC_FEAT_SELECTION`. A slot's lists are the selection itself, or the held options' lists for `option_lists`
(`Catalog::slot_lists`); the walk's check, the picker and the Sword Saint's proficiency read them. Selections of feats
aren't held as facts (decision 4) and are never granted without a pick. Labels: the selection's name, "Bonus feat" for
`feats` on a race's or a row's slot (`Engine::slot_label`, `feat_slot_label`). The data check carries the list issues to
selections (`FeatListTooShort`, `UnusedFeatList`, `MisplacedBonusList`) and adds `FeatsAndOptions` and `NotOfFeats`
(a race's selection or an option row's `feat_list` that lists no feats). MCP `describe` keeps its `feat_slots` output
and lists only selections of options as `selections`; the web app's empty-slot message reads `option_lists`.

**Decisions.**
- Main session (decisions 1–5): above.
- Mine: the option-list selections' keys (`<selection>-feats`, `-<level>` when fixed at one; `ranger/combat-style-feats`
  and `kineticist/wild-talent-feats` by name), named as their slots were labelled.
- Mine: the game check's rows area compares a grant of a feat pool (`FEAT_POOLS`: the five category blueprints and
  `SorcererFeatSelection`) as a pick of its group on both sides, as it compared the feat slots, so its row findings are
  unchanged; the options area compares a selection's feats, and skips the seven option-list selections (302).
- Mine: the test world and the seeds fixture each gain a second Teamwork feat (Shield Wall), so their teamwork
  selection is a list of two; the test world lists its feats in its pools whenever an engine is built on it
  (`list_feats_in_pools`), so a test's added feat is offered as before.

**Proof** (decision 5). `record` on `49306dff` in a temporary worktree, `compare` on the tip: 2,162 Builds (2,001
open), 2,162 old links reopened; `builds.tsv`, `sheets.txt` and `numbers.txt` byte-identical; `slots.txt` differs on
20,718 of 49,631 lines, and the classifier over `kind-map.tsv` finds every one differs only in the kind column, as the
map says (51 of its 52 kinds met; `Feat(General)` → `Feat(FeatureSelectionKey("feats"))`, `Feat(OneOf { list:
FeatListKey("x") })` → `Feat(FeatureSelectionKey("x"))`, …), 0 otherwise, and no unchanged line of an old feat kind. Every
corpus and bench link opens with the same picks (`cargo test -p corpus --test links`).

**Game check:** exits 0. Its row and mapping findings are unchanged (mapping: 4,050 compared, all agree; 6 more
selections without a blueprint). `known.txt`'s `feat list …` lines are `selection …`; the planner's lists of the five
category selections and Dragon Feats against the game's selections, 222 new option-list findings, are known under 293
(the game's lists are 293's).

**Tests.** `libs/seeds/tests/game_data/feat_slots_as_selections.rs`: `no_feat_lists_or_feat_slots_remain`,
`the_general_slot_is_basic_feat_selection` (game: `BasicFeatSelection` 247a4068…, granted by `BasicFeatsProgression`
5b72dd2ca2cb73b49903806ee8986325 at 1, 3, 5, … and `HumanRace` 0a5d473ead98b0646b94495af250fdc4),
`the_bloodline_feat_slot_is_sorcerer_feat_selection` (`SorcererProgression` grants `SorcererFeatSelection` 3a60f0c0… at
7, 13 and 19), `monk_bonus_feats_keep_their_list` (`MonkBonusFeatSelectionLevel1` ac3b7f5c11bce4e44aeb332f66b75bab:
its six feats, `IgnorePrerequisites`), `dragon_feats_list_every_feat` (the planner's 261; `DragonLevel2FeatSelection`
a21acdafc0169f5488a9bd3256e2e65b's 268 are 293's). `libs/rules/tests/engine/feat_selections.rs`:
`a_row_numbers_feat_and_option_selections_apart`, `a_one_feat_selection_still_takes_a_pick`,
`a_selection_of_feats_and_options_is_refused`, `the_basic_selection_reads_feat_or_bonus_feat`. The existing tests are
rewritten onto the new form with their values unchanged, but the test world's and fixture's feat counts (one more
feat) and the entries without a blueprint (+6 selections).
