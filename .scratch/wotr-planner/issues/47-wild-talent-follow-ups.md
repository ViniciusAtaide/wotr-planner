# Wild talent follow-ups

Type: task
Status: resolved
Blocked by: 46
Map: [WotR Build Planner](../map.md)

## Question

Build [ticket 45](45-kineticist-wild-talents.md)'s decisions 1 and 3 (user, 2026-11-02):

1. **The Skill Focus wild talent repeats per skill** (a fact from the user, who wins over the guide's silence): it can be picked at several wild talent levels, each time with a skill the character doesn't already have it for. Model it the way the data already models "once per value" (a feat's `repeat: PerValue`, or what options with a `chosen_feat` use), so a second pick for the same skill is refused and one for another skill is open. **The rogue talent Skill Focus follows the same rule** (user, 2026-11-02: "rogue talent follows the same rule as wild talent"): picked again at later rogue talent levels, each time for another skill. The slayer shares the rogue talents' options, so it applies to a slayer's talents too.
2. **Rogue talents Weapon Focus and Combat Trick** (user, 2026-11-02): "for the rogue talent weapon focus can be only taken once. but the combat trick -> weapon focus can be taken repeatable times as long as you select a different weapon".
   - The rogue talent **Weapon Focus** stays taken once (as today).
   - The rogue talent **Combat Trick** can be taken more than once, and each time grants a bonus Combat feat slot (today it's a bare option that grants nothing). Through those slots, Weapon Focus can be taken again for another weapon, as the feat's own once-per-weapon rule already says. Check the saved rogue page for what Combat Trick grants and whether its feat ignores prerequisites; if the page doesn't say, keep prerequisites. The slayer shares the rogue talents, so it applies to its talents too.
3. **The bonus-feat slot's label** reads "Wild talent feat" instead of "Elemental Focus feat". It's named after the selection by a generic rule in `rules/src/engine/labels.rs`; change the slot's label without renaming the Elemental Focus selection, and keep the other `OptionList` labels as they are ("Bloodline feat").

## Build

AFK, test-first (`tdd`); the only agent in the repo. Same method as tickets 43–45 for the data change (extractor rule asserted against the guide page's shape, one-off rewrite of `game-data/`, fresh extraction matching, real-data tests first). If the types can't say "an option once per value" yet, add it test-first in `rules` (and the schema, with a DB reset per `CLAUDE.md`).

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: a kineticist takes Skill Focus (Perception) at 2 and Skill Focus (Stealth) at 4; Skill Focus (Perception) again at 6 is refused; a rogue takes the Skill Focus talent for two different skills and is refused a second one for the same skill; the bonus-feat slot's label is "Wild talent feat"; the sorcerer's level-1 slot still reads "Bloodline feat"; a rogue takes Combat Trick twice, gets a Combat feat slot each time, and takes Weapon Focus (Longsword) in one and Weapon Focus (Dagger) in the other, while Weapon Focus (Longsword) twice is refused; the rogue talent Weapon Focus can't be taken twice.

## Progress log

- 2026-11-02: started. Baseline: HEAD 7a9c14a (the user's uncommitted edit in `server/src/main.rs` left alone); scratch in `t47/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t47/`), `old-src/` is `git archive HEAD`, `extract-old/` HEAD's extraction (23 report items; byte-identical to ticket 45's fresh extraction). Read the guide: `kineticist#skill-focus` ("Identical to the feat"), `rogue#skill-focus` and `rogue#weapon-focus` ("This is the feat of the same name."), `rogue#combat-trick` ("Any combat feat that the character qualifies for.": prerequisites kept). The types couldn't say "an option once per value": `GameData::check`, the engine and the schema refused `PerValue` on an option. Combat Trick's slot needs no new capability: an option's row-1 feat slot through a selection with option levels (the bonus-feat wild talent's kind), given again by each holding. The bonus-feat slot's label comes from the generic `OptionList` rule, and the slayer's combat style slots are `OptionList`s granted by option rows too, so no generic rule tells the kineticist's apart: the slot gets its own label in the data.
- 2026-11-02: user's scope update (mid-ticket): the rogue talent Weapon Focus stays once; Combat Trick repeats with a Combat feat slot each time (item 2). Added to the plan.
- 2026-11-02: real-data tests first (`game_data.rs`, 7 new, 3 changed): red on HEAD's rules and data (`t47/real-data-red.txt`: 143 pass, 7 fail, each where expected: `Once` for `PerValue`/`Unlimited`, a two-skill Build impossible, no Combat Trick slot, "Elemental Focus feat"; the rogue talent Weapon Focus test passes, pinning it). Changed: the wild talent label test expects "Wild talent feat"; `a_rogue_slayer_cannot_take_one_talent_twice` uses Fast Stealth (Combat Trick repeats now); the test listing every repeating option adds the three.
- 2026-11-02: rules, test-first (test world): `check::an_option_repeats_once_per_value_only_of_its_chosen_feat`, `labels::an_option_list_slot_is_named_after_its_selection_unless_it_has_a_label`, `parameters::an_option_once_per_value_repeats_for_another_value_of_its_chosen_feat`, `parameters::an_option_taken_again_grants_its_rows_slot_again` (red: `t47/rules-red-compile.txt`, then with the type stubs `t47/rules-red.txt`, 315 pass, 3 fail; the fourth passes on HEAD's engine, pinning that a second holding gives its row's slot again). Built: `FeatSlot::OptionList.label`, `DataIssue::PerValueWithoutChosenFeat`, the engine's `PerValue` option rule, the label in `slot_label`. Green: 318 (`t47/rules-green.txt`).
- 2026-11-02: schema and DB: `selection_option`'s `CHECK (repeat <> 'per_value')` became `CHECK (repeat <> 'per_value' OR chosen_feat_key IS NOT NULL)`; `feat_slot.label` (non-empty, option list slots only). Seed and load carry the label; `cargo sqlx prepare` replaced 2 query files (the `feat_slot` insert and select). DB reset per `CLAUDE.md` (`wotr-planner-postgres` on 5434 only).
- 2026-11-02: data and extractor. New `extract::talent_follow_ups` (`apply`), run after `wild_talents::apply`; it asserts the guide's sentences and the data's shape (Skill Focus `PerValue` on a skill; both talents `Once` with Skill Focus as chosen feat; Weapon Focus `Once`; Combat Trick bare; exactly four selections offer it, the slayer's levelled by the slayer already; the bonus-feat talent's one unlabelled slot). `t47/rewrite/` (ticket 45's, calling `talent_follow_ups::apply`) wrote `game-data/` byte-identical to its dry run: 7 changes in 2 files (`classes/kineticist.ron`, `classes/rogue.ron`; `t47/game-data-log.json`), 0 check problems. A second run refuses (Skill Focus is `PerValue` already). Real-data tests green (150, `t47/real-data-green.txt`). A fresh extraction (`t47/extract-new/`) is byte-identical to HEAD's extraction rewritten by `t47/rewrite/` (`t47/dry/extract-old-rewritten/`; the same 7 changes, line for line those in `game-data/`); its report is byte-identical to HEAD's extraction's (23 items). The RON writer writes a slot's label on its line; `extract/tests/extract/talent_follow_ups.rs` 1 test.
- 2026-11-02: proof, docs, DB, Verification. Proof: `t47/proof-old/` (HEAD's rules) and `t47/proof-new/` build `t47/proof-common.rs` (ticket 45's Builds plus `follow_builds`, each slot's label printed); `t47/compare.py` (96137 checks) passes (`t47/compare.txt`). Docs: `game-data/README.md` (ticket 47's paragraph), `GLOSSARY.md` (**Repeat**), the map. Reseeded. Verification below.

## Answer

Done 2026-11-02, uncommitted. Ticket 45's decisions 1 and 3 and the user's rule for the rogue talents Weapon Focus and Combat Trick are built. One capability in `rules` (an option once per value of its chosen feat) and one slot field (an option list slot's label), so the one migration changed and the DB was reset; no `Build` change, so the v1 link vector stays.

### Data

`extract::talent_follow_ups` asserts each guide sentence and the data's shape before changing anything:

- **The Skill Focus talents** (`kineticist/wild-talents/skill-focus`, `rogue/rogue-talents/skill-focus`, which the slayer's talents share): `repeat: PerValue`, Skill Focus in another skill each time.
- **The rogue talent Weapon Focus**: unchanged, once.
- **Combat Trick** (`rogue/rogue-talents/combat-trick`): `repeat: Unlimited`, row 1 `feat_slots: [CombatFeat]`: a bonus Combat feat slot each time, prerequisites checked ("that the character qualifies for"). Weapon Focus in a second weapon through a second slot is the feat's own once per weapon.
- **Option levels for Combat Trick's row:** the rogue talents, the Sylvan Trickster's Fey Tricks and the Underground Chemist's Discovery are levelled by the rogue (`option_levels: ["rogue"]`); the slayer's talents were levelled by the slayer already. These four are every selection offering Combat Trick.
- **The bonus-feat wild talent's slot**: `OptionList(selection: "kineticist/element", label: "Wild talent feat")`; the Elemental Focus selection keeps its name, and every other option list slot (sorcerer and bloodrager "Bloodline feat", the slayer's and ranger's combat style slots) keeps its label.

`t47/rewrite/` (in the scratchpad `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t47/`) ran `talent_follow_ups::apply` over `game-data/`: 7 changes in 2 files (kineticist, rogue), byte-identical to its dry run, 0 check problems. The extractor was never run over `game-data/`.

### Rules

- `Repeat::PerValue` on an option: once for each value of its chosen feat. A pick is refused ("Already taken") for a value the chosen feat is held for, whatever gave it. `GameData::check` allows it only with a `chosen_feat` (new `DataIssue::PerValueWithoutChosenFeat`; an option with its own parameter, such as Battle Master, has no values held to count).
- `FeatSlot::OptionList.label`: the slot's own label; `slot_label` reads it before the generic "<selection name> feat".
- An option taken again gives its row's slot again (each holding is its own), unchanged and now pinned by a test.

Engine tests (test world): 4 new.

### Schema

- `selection_option`: `CHECK (repeat <> 'per_value' OR chosen_feat_key IS NOT NULL)` replaces "never `per_value`".
- `feat_slot.label TEXT`: non-empty, on `option_list` slots only.
- Still 41 tables. Postgres tests: the two CHECKs rejected, `per_value` accepted with a chosen feat, and a round trip of a `PerValue` option and a labelled slot (1 new test).

### Tests

Real-data tests (`game_data.rs`, 7 new, red on HEAD first):

- the Skill Focus talents are `PerValue` with Skill Focus as chosen feat, the rogue talent Weapon Focus `Once`;
- a kineticist takes Skill Focus (Perception) at 2 and Skill Focus (Stealth) at 4; Perception again at 6 is "Already taken", Athletics is open;
- a rogue and a slayer take the Skill Focus talent for two skills and are refused the same skill again;
- the rogue talent Weapon Focus can't be taken twice, even for another weapon;
- Combat Trick is `Unlimited` with a row-1 Combat feat slot; a rogue taking it at 2 and 4 has a "Combat feat" slot at each, offering Weapon Focus, not Skill Focus, with Cleave locked "Needs Power Attack";
- a rogue takes Weapon Focus (Longsword) and (Dagger) through two Combat Tricks; Longsword twice is "Already taken"; the link opens as the Build; the rogue talent Weapon Focus at 6 is open for a rapier, not a longsword;
- Combat Trick gives its slot through the slayer's talents, Fey Tricks and Discovery (rogue 10).

Changed: the bonus-feat slot reads "Wild talent feat" (the sorcerer's level-1 slot still reads "Bloodline feat", its own test unchanged); the rogue/slayer "one talent twice" test uses Fast Stealth; the list of repeating options adds the three.

### Extractor

`extract` runs `talent_follow_ups::apply` after `wild_talents::apply`; it settles no report item. A fresh extraction (`t47/extract-new/`) is byte-identical in its RON files to HEAD's extraction rewritten by `t47/rewrite/` (the same 7 changes, line for line those in `game-data/`); its report is byte-identical to HEAD's extraction's (23 items). `extract/tests/extract/talent_follow_ups.rs`: 1 test (the writer's per-value option and labelled slot, read back).

### Proof

`t47/proof-old/` (HEAD's rules) and `t47/proof-new/` (ticket 47's) build `t47/proof-common.rs`: ticket 45's Builds plus 14 `follow ...` Builds (a kineticist, rogue and slayer with the Skill Focus talent for two skills, and for one twice; a kineticist with it and the bonus-feat talent; a rogue with Combat Trick twice, Weapon Focus (Longsword) and (Dagger) in its slots and the Weapon Focus talent (Rapier) at 6; Combat Trick twice with Longsword twice; the Weapon Focus talent twice; Combat Trick once through the rogue's, slayer's, Sylvan Trickster's and Underground Chemist's selections). `t47/compare.py` (96137 checks pass):

- **Engine:** the new rules walk HEAD's data byte for byte as HEAD's rules, but the new field's Debug text (`label: None`).
- **Data:** the old data with the 7 changes is exactly the new data.
- **Walk:** all 6561 lines are identical but for the bonus-feat slot's label (3 lines), a Skill Focus talent or Combat Trick open again in a later picker once held (10) or in an earlier picker before its later pick (6), a rogue talent's option level (9), and the `follow` Builds: the two-skill and two-trick Builds turn possible, the stale ones stay impossible, the single-trick ones stay possible with a "Combat feat" slot at their trick (8 slots in all).

### Docs

- `game-data/README.md`: ticket 47's paragraph.
- `GLOSSARY.md`: **Repeat** (Combat Trick; an option once per value of its chosen feat).
- `game-data/EXTRACTION_REPORT.md`: unchanged (0 items).
- The map: a Decisions-so-far line.

### Verification

- `cargo test --workspace`: pass (rules 318 + unit 4, extract 91, data files 25 + game_data 150 + postgres 26).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass (2 query files replaced: the `feat_slot` insert and select carry `label`).
- Dev DB reset per `CLAUDE.md`, only `wotr-planner-postgres` on 5434: drop/create `wotr_planner`, `sqlx migrate run --source data/migrations`, `cargo run -p data --bin seed`, `cargo sqlx prepare`.
- `--check-only` and the real seed both give unchanged counts: 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths and 38 backgrounds.
- Rows: `feat_slot` 284 (was 283: Combat Trick's `combat_feat`), one with a label; `option_level` 531 (530); `selection_level_class` 66 (63); `selection_option` 747, `feature_selection` 116, `prerequisite_node` 2071 and `feature` 1922 unchanged.
- Ticket 22's acceptance grep finds nothing (also over `extract` and `data/src`).
- The v1 link vector is unchanged (no `Build` change).
- Proof: passes (above).

### Open questions

None.
