# 301: feat slots and feat lists as selections

**What the data says now.** A feat slot is a selection of feats, as the game's are `BlueprintFeatureSelection`s
listing their feats: a row, a race, a subrace or a path rank names it in its `selections`, and the slot it makes is a
Feat slot. `feat-lists/` and every `feat_slots` field are gone.

- The 78 feat lists are selections with the same keys, under `selections/` (same name, blueprint, guide,
  `ignores_prerequisites`, `skips_proficiency`, feats; the Crusader's lists keep their leading comment). Gold Dragon's
  Dragon Feats (`every_feat` before) lists every feat by name: today's 261.
- The general and category slots name five new selections, each listing the planner's feats of its kind (every feat;
  every Combat feat, `FeatCategory::is_combat`; every feat of the category):

  | Key | Name | Game blueprint | Feats |
  | --- | --- | --- | --- |
  | `feats` | Feat | `BasicFeatSelection` 247a4068296e8be42890143f451b4b45 | 261 |
  | `combat-feats` | Combat feat | `FighterFeatSelection` 41c8486641f7d6d4283ca9dae4147a9f | 144 |
  | `magic-feats` | Magic feat | `WizardFeatSelection` 8c3102c2ff3b69444b139a98521a4899 | 35 |
  | `teamwork-feats` | Teamwork feat | `TeamworkFeat` d87e2f6a9278ac04caeb0f93eff95fcb | 13 |
  | `mounted-combat-feats` | Mounted Combat feat | `CavalierBonusFeatSelection` dd17090d14958ef48ba601688b611970 | 5 |

  `feats` is the engine's general slot (`rules::BASIC_FEAT_SELECTION`) and Extra Feat's, and the human's bonus feat
  (`races/human.ron`); on a race's or a row's slot it reads "Bonus feat", as the `General` slot did.
- The seven option-list slots are selections with `option_lists: (selection: …, at_level: …)`, named as the slots
  were labelled: `sorcerer/bloodline-feats` ("Bloodline feat", `SorcererFeatSelection`
  3a60f0c0442acfb419b0c03b584e1394, which `SorcererProgression` grants at 7, 13 and 19), and, under
  `planner/selections/` with their reasons, `bloodrager/bloodline-feats` ("Bloodline feat"),
  `ranger/combat-style-feats` ("Combat Style feat"), `slayer/combat-style-feats-2`, `-6` and `-10` ("Combat Style
  (Slayer) feat", at that option level) and `kineticist/wild-talent-feats` ("Wild talent feat").
- 313 `feat_slots` fields in 108 files became `selections` keys (315 slots: 107 lists, 81 Combat, 48 option-list, 35
  Magic, 31 Teamwork, 12 general, 1 Mounted Combat). An option row's `feat_list` names its list's selection.

**How.** A rewrite outside the repo (`<scratchpad>/game/301/rewrite/`, `cargo run --release -- <data dir>
[<kind-map.tsv>]`), from the data at `49306dff`, not the game: it reads the old forms with its own copies of the old
types, writes the selections, moves each row's slots to its `selections` (after the ones it named), deletes
`feat-lists/`, and reads everything back with the new types: each row's selections are its old selections and its
slots' keys, its selections of feats in the order its slots had, and each new selection is the list or slot it stands
for. Run again, it moves nothing. Also run on the seeds fixture; game-check's `planner.ron` fixture was edited by hand.

**Decisions.**
- The main session (`game/301/decisions.md`): the option-list slots keep the planner's form, `option_lists` on the
  selection, until nested selections (302); the category selections name the game's blueprints (the teamwork one
  through its granting progression, `InquisitorProgression`); lists are explicit (Dragon Feats: today's 261, not the
  game's 268); selections of feats are not held as facts (305); every slot id and pick stays byte-identical.
- Here: the option-list selections' keys (`<selection>-feats`, `-<level>` when fixed at one; `ranger/combat-style-feats`
  and `kineticist/wild-talent-feats` by name).
- Where the planner's list and the game's differ, the planner's stays: the game check reports them under 293.

**What it does not model.** The game's per-class and per-option feat selections (293, 302); a selection of feats held
as a fact (305).

**Tests.** `libs/seeds/tests/game_data/feat_slots_as_selections.rs`: no feat lists or feat slots remain; the general
slot is `BasicFeatSelection`; the bloodline feat slot is `SorcererFeatSelection` at 7, 13 and 19; the monk's 1st-level
bonus feats keep their list (`MonkBonusFeatSelectionLevel1` ac3b7f5c11bce4e44aeb332f66b75bab); Dragon Feats list every
feat.
