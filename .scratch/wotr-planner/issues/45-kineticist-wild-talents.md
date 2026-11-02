# Kineticist wild talents

Type: task
Status: resolved
Blocked by: 44
Map: [WotR Build Planner](../map.md)

## Question

Build [ticket 43](43-extraction-report-gaps.md)'s decision 2 (user, 2026-11-02): wild talents become a selection. Today the even-level "Wild Talents" is a feature, not a pick.

- From `kineticist#wild-talents`: picked at 2nd and every even level, from talents whose level is at most half the kineticist level, each limited to its elements. The guide lists 26 talents with their level and element conditions. The four defensive talents stay each element's 2nd-level grant, as now.
- The bonus-feat wild talent (Universal, level 1) is taken once (the guide doesn't say it repeats) and grants a bonus feat slot, ignoring prerequisites, from the held elements' wild-talent bonus-feat lists (`OptionList(selection: "kineticist/element")`, so Expanded Element offers both elements' lists).
- Kineticist archetypes that change wild talents follow their own sections.

Data from the saved guide pages only. If the guide doesn't settle a talent's condition, write a question in this ticket with a recommendation and go on.

## Build

AFK, test-first (`tdd`); the only agent in the repo; schema change and DB reset allowed per `CLAUDE.md` if the types need one (a new prerequisite kind, for example); `V1_LINK` regenerated with a comment if `Build` changes. Same method as tickets 30–38 and 43: extractor module, one-off rewrite of `game-data/`, real-data tests first, fresh extraction matching the rewrite, before/after proof. The kineticist report item is closed: [ticket 44](44-sorcerer-feat-slayer-deities.md) moved it here (`kineticist#kineticist-bonus-feat`, "a wild talent that grants a bonus feat slot; a selection option can't grant a slot"), so `extract::report_gaps::moved` leaves it out of an extraction's report and `game-data/EXTRACTION_REPORT.md` has 0 items; this ticket settles it instead (`moved` goes, and the rule that builds the talent settles the item).

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: a kineticist 2 picks a wild talent of level ≤ 1 from its element's and Universal talents only; a Fire kineticist isn't offered a Water-only talent; a talent of level 3 opens at kineticist 6; the bonus-feat talent grants one slot offering only the held elements' bonus feats, ignoring prerequisites, and can't be taken twice.

## Progress log

- 2026-11-02: started. Baseline: HEAD 174ec14 (code as 63d04a4; 174ec14 adds ticket 46's plan only; the user's uncommitted edit in `server/src/main.rs` left alone); scratch in `t45/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t45/`), `old-src/` is `git archive HEAD`, `extract-old/` HEAD's extraction (23 report items, the kineticist's left out by `report_gaps::moved`; its `classes/kineticist.ron` and the feats' kineticist lines are byte-identical to `game-data/`'s). Read `kineticist#wild-talents` and the "Wild Talents" section against the decision: 26 talents under "Level 1/2/3/4/6/9 Talents", each with an `Element` row (Air, Earth, Fire, Water, "Fire, Water", Universal, and Elemental Whispers' "All") and some a `Prerequisites` row (Fire blast; Searing Flesh, Enveloping Winds, Flesh of Stone; Kinetic Healer); the four defensive ones are `Type | Defensive`; "The Kineticist's class level must be at least twice the level of the Wild Talent to unlock it."; every archetype's table has "Wild Talents" at each even level (no archetype section changes them; the Kinetic Knight's defensive talent comes at 4). Two talents do more than a note: Skill Focus ("Identical to the feat") and Expanded Defense ("You unlock the Defensive Wild Talent for any element you selected with Expanded Element"), which the three talents needing a defensive talent depend on for an expanded element.
- 2026-11-02: real-data tests first (`game_data.rs`, 9 new): the selection (22 options, each one's prerequisite by the table above, no "Wild Talents" feature left, the class's rows 2 to 20); a Fire kineticist 2 offered exactly its 8 open level-1 talents, Kinetic Healer locked "Needs Elemental Focus - Water", Foxfire "Needs Kineticist level 4 (have 2)"; Celerity (level 3) "Needs Kineticist level 6 (have 4)" and open at 6; Flame Shield "Needs Searing Flesh" for a Kinetic Knight at 2, open at 4, Healing Burst after Kinetic Healer; the bonus-feat talent's one slot (exactly the Fire list, Improved Iron Will open without Iron Will, a feat off the list impossible, "Already taken" at 4, both lists with Expanded Element (Water)); Expanded Defense (an expanded Air's Enveloping Winds from 8, Aerial Evasion open; Water at 15 gets Shroud of Water at 15); the Skill Focus talent; every archetype picks at 2 to 20; the labels. Red run on HEAD's data (`t45/real-data-red.txt`: 131 pass, 9 fail, each at the missing slot or selection). Then `every_feature_a_prerequisite_needs_is_named` caught Extra Wild Talent's `Feature("kineticist/wild-talents")`: a 10th test, `extra_wild_talent_needs_kineticist_level_2`, red on HEAD's data (`t45/real-data-red-extra-wild-talent.txt`).
- 2026-11-02: the engine (test-first, `rules/tests/engine/option_levels.rs`, 3 new on the test world's kineticist, which gains element rows with a defensive feature at 2, a lists-only Expanded Element and wild talents with an Expanded Defense): an expanded element gives no defensive talent; a feature's `option_row` naming the selection a lists-only option is held through gives that row in full, to elements taken before it and after (red: `t45/rules-red.txt`, 309 pass, 1 fail); Bloodline Ascendance still gives a Crossblooded second bloodline nothing. `Walker`: a `Holding` keeps the levels `unlocked`; `advance` gives an unlocked row in full, `grant_option_row` gives a lists-only holding the row only when the row's selection is the one it's held through (replacing the arcana it had of the row). No `Build`, schema or type change.
- 2026-11-02: data and extractor. New `extract::wild_talents` (`apply`, `settled`): it asserts the pick's and the level rule's sentences, every progression table's "Wild Talents" at 2 to 20, the 26 talents (4 defensive, each its element's row 2, at level 1), the elements' lists ignoring prerequisites, the three special talents' sentences, Skill Focus's parameter, Expanded Element's `feat_lists_only`, the rows holding the feature (base 2 to 20, Blood Kineticist 20, Kinetic Knight 4), the feats page's "Prerequisites: Wild Talents" and that only Extra Wild Talent needs the feature. `extract` runs it after `report_gaps::apply`; `report_gaps::moved` is gone and `wild_talents::settled` drops the item instead. `t45/rewrite/` (ticket 44's, calling `wild_talents::apply`) wrote `game-data/` byte-identical to its dry run: 29 changes in 2 files (`classes/kineticist.ron`, `feats.ron`; `t45/game-data-log.json`), 0 check problems. Real-data tests green (141, `t45/real-data-green.txt`). A fresh extraction (`t45/extract-new/`) is byte-identical in its RON files to HEAD's extraction rewritten by `t45/rewrite/` (`t45/dry/extract-old-rewritten/`, the same 29 changes, line for line those in `game-data/`); its report is byte-identical to HEAD's extraction's (23 items: the item `moved` left out is now settled). `extract/tests/extract/wild_talents.rs` 1 test (`report_gaps.rs` loses its `moved` test). `cargo test -p extract` 90 pass.
- 2026-11-02: proof, docs, DB, Verification. Proof: `t45/proof-old/` (HEAD's rules) and `t45/proof-new/` (ticket 45's) build `t45/proof-common.rs` (ticket 44's Builds plus `talent_builds`); `t45/compare.py` (93854 checks) passes. Docs: `game-data/README.md` (ticket 45's paragraph), `game-data/EXTRACTION_REPORT.md` (header; still 0 items), the map (Decisions so far). Reseeded the dev DB (no schema change). Verification below.

## Questions for the user

1. **Does the Skill Focus wild talent repeat?** The guide says only "Identical to the feat"; the feat repeats once per skill. It's built as the rogue talent Skill Focus is: taken once, Skill Focus in one skill. **Recommendation:** keep it once (the guide doesn't say it repeats, as for the bonus-feat talent).
2. **Extra Wild Talent's extra pick.** "You gain one additional wild talent": like every Extra feat in the data (Extra Rogue Talent, Extra Hex, Extra Slayer Talent …), the feat grants no extra slot; only its prerequisite is checked (now kineticist level 2). **Recommendation:** a follow-up ticket that lets an Extra feat grant one more slot of its selection, for all of them at once, if you want them planned.

## Answer

Done 2026-11-02, uncommitted. Ticket 43's decision 2 is built: the wild talents are a selection, and the bonus-feat talent grants its slot; the kineticist's report item is settled (`report_gaps::moved` is gone). One engine capability (an option row unlocked for a lists-only holding); no schema, type or `Build` change, so the v1 link vector stays.

### Data

`extract::wild_talents` asserts each guide sentence and the data's shape before changing anything:

- **The selection** `kineticist/wild-talents` ("Wild Talents", levelled by kineticist) in place of the "Wild Talents" feature on the rows that held it: the base class's 2 to 20, the Kinetic Knight's 4 and the Blood Kineticist's 20 (archetype rows that differ from the base's; the others inherit the base's). No archetype changes the pick.
- **22 talents** (the guide's 26 but the four defensive ones, which stay each element's 2nd-level row): Aerial Adaptation, Cold Adaptation, Elemental Whispers, Heat Adaptation, Fire's Fury, Flame Shield, Kinetic Healer, Kineticist Bonus Feat, Skill Focus, Skilled Kineticist, Slick (level 1); Foxfire (2); Aerial Evasion, Celerity, Jagged Flesh, Kinetic Restoration (3); Enduring Earth, Expanded Defense, Healing Burst (4); Kinetic Revivification, Tremor Sense (6); Tidal Wave (9). Each needs:
  - `ClassLevel(kineticist, 2 × its level)`: the talent's level lives only there (no derived column);
  - one of its elements, `SelectionOption("kineticist/element/…")`, an `Any` for "Fire, Water", none for Universal and All. An option held through any selection counts, so Expanded Element's (and the Blood Kineticist's Water) opens its element's talents;
  - its "Prerequisites": a defensive talent is the element's feature (Flame Shield: Searing Flesh; Aerial Evasion: Enveloping Winds; Jagged Flesh: Flesh of Stone), Kinetic Healer the option (Healing Burst, Kinetic Revivification); "Fire blast" (Fire's Fury, Foxfire) is the Fire element, which the talent asks already.
  A Universal talent's prerequisite is the `ClassLevel` alone.
- **Kineticist Bonus Feat:** taken once; its row 1 is an `OptionList(selection: "kineticist/element")` slot, so it offers the held elements' wild-talent bonus feats (both with Expanded Element), ignoring prerequisites (those lists ignore them).
- **Skill Focus:** `chosen_feat: "skill-focus"`, Skill Focus in a skill of the player's choosing, once (question 1).
- **Expanded Defense:** its row 1 holds the feature `kineticist/wild-talents/expanded-defense/defense`, `option_row: (selection: "kineticist/expanded-element", level: 2)`: the expanded elements' defensive talents, so an expanded Air's Aerial Evasion is open.
- **Extra Wild Talent:** "Prerequisites: Wild Talents" named the feature that went; it needs `ClassLevel(kineticist, 2)`, the first pick's level, as Extra Slayer Talent needs slayer 2 (the same characters qualify: every kineticist row 2 held the feature). Its extra pick isn't modelled (question 2).

`t45/rewrite/` (in the scratchpad `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t45/`) ran `wild_talents::apply` over `game-data/`: 29 changes in 2 files (kineticist, feats), byte-identical to its dry run, 0 check problems. The extractor was never run over `game-data/`.

### Rules

A feature's `option_row` (Bloodline Ascendance's kind) passed over every lists-only holding (Crossblooded's second bloodline, Expanded Element). Now, when the row it names is of the selection the lists-only option is held through, that holding gets the row in full, whether the option was taken before the feature (in place of the arcana it had of the row) or after (as the row's level comes). Only Expanded Defense names such a row; Bloodline Ascendance still gives a Crossblooded second bloodline nothing. Engine tests (test world): 3 new.

### Tests

Real-data tests (`game_data.rs`, 10 new; red on HEAD's data first):

- the selection's 22 options and each one's prerequisite; no "Wild Talents" feature; picks at 2 to 20;
- a Fire kineticist 2 is offered its element's and the Universal level-1 talents only; a Water-only talent reads "Needs Elemental Focus - Water", and a Water kineticist has it;
- a level-3 talent (Celerity) reads "Needs Kineticist level 6 (have 4)" at 4 and opens at 6; at 4 the Build is impossible;
- Flame Shield waits for Searing Flesh (a Kinetic Knight's at 4), Healing Burst for Kinetic Healer;
- the bonus-feat talent's one slot offers only the held elements' lists, ignoring prerequisites, both lists with an expanded element, nothing off them, and the talent can't be taken twice ("Already taken");
- Expanded Defense gives an expanded element's defensive talent, also to an element taken later;
- the Skill Focus talent is Skill Focus in its skill;
- every archetype picks a wild talent at each even level;
- the labels: the slot reads "Wild Talents", the bonus-feat slot "Elemental Focus feat" (the generic rule: the selection whose lists it offers, as a sorcerer's "Bloodline feat"), and every slot of a kineticist 20 has a label;
- Extra Wild Talent needs kineticist level 2 ("Needs Kineticist level 2 (have 1)").

### Extractor

`extract` runs `wild_talents::apply` after `report_gaps::apply` and drops the item `wild_talents::settled` names (`kineticist#kineticist-bonus-feat`); `report_gaps::moved` is gone. A fresh extraction (`t45/extract-new/`) is byte-identical in its RON files to HEAD's extraction rewritten by `t45/rewrite/` (the same 29 changes, line for line those in `game-data/`); its report is byte-identical to HEAD's extraction's (23 items).

### Proof

`t45/proof-old/` (HEAD's rules) and `t45/proof-new/` (ticket 45's) build `t45/proof-common.rs`: ticket 44's Builds plus a Fire kineticist 20 holding the bonus-feat talent (Improved Iron Will in its slot), Foxfire, Flame Shield, Expanded Defense, Kinetic Healer, Healing Burst, Skill Focus (Perception), Aerial Evasion on an expanded Air, Tidal Wave and Kinetic Revivification; a Kinetic Knight (Earth, Jagged Flesh at 6); a Blood Kineticist; three stale links (a Water talent on Fire, Celerity at 4, the bonus-feat talent twice); Extra Wild Talent at kineticist 1 and 3. `t45/compare.py` (93854 checks pass):

- **Engine:** the new rules walk HEAD's data byte for byte as HEAD's rules.
- **Data:** the old data, with the feature replaced by the selection on its 12 rows, Expanded Defense's feature and Extra Wild Talent's `ClassLevel`, is exactly the new data.
- **Walk:** all 6459 lines are identical but for the new Wild Talents slot at 91 levels of 16 Builds (each option open exactly when its prerequisite, evaluated on the line's state, holds and it isn't held) and the "Wild Talents" feature gone from the features; the three Builds holding talents turn possible; the stale links stay impossible (one picker moves: Water at 1 would make the Water talent possible); Extra Wild Talent stays closed at kineticist 1 and open at 3, and opens and closes everywhere as before.

### Docs

- `game-data/README.md`: ticket 45's paragraph.
- `game-data/EXTRACTION_REPORT.md`: the header says ticket 45 fixed the kineticist item; 0 items.
- The map: a Decisions-so-far line.
- `GLOSSARY.md`: unchanged.

### Verification

- `cargo test --workspace`: pass (rules 310 + unit 2, extract 90, data files 25 + game_data 141 + postgres 25).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass (no query file changed).
- No schema change, so no DB reset: `cargo run -p data --bin seed` reseeded `wotr-planner-postgres` on 5434 only; the postgres tests pass on it.
- `--check-only` and the real seed both give unchanged counts: 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths and 38 backgrounds.
- Rows: `feature_selection` 116 and `selection_option` 747 (1 and 22 more); `feat_slot` 283 (was 282): `option_list` 54 (53); `prerequisite_node` 2071 (was 2008: the 22 talents' 63 nodes; Extra Wild Talent's leaf changes kind); `feature` unchanged (the Wild Talents feature out, Expanded Defense's in).
- Ticket 22's acceptance grep finds nothing (also over `extract` and `data/src`).
- The v1 link vector is unchanged (no `Build` change).
- Proof: passes (above).

### Open questions

The two questions above.
