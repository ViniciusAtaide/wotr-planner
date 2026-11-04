# Move the picker's rules into the engine

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The map's Notes put the rules in the `rules` crate and keep the UI as views. The architecture review (2026-11-03) found `libs/ui` deciding rules itself, beside the engine's own versions:

| Where (`libs/ui/src/`) | What it decides | The engine's version |
| --- | --- | --- |
| `picker.rs:707–744` (`Advances`) | the candidate spellcasting-advancement sets: each held class alone and every pair, each tried through `engine.apply` and `is_possible` | `rules/src/engine/walk/level.rs` `check_advances` ("at most one arcane and one divine"); the UI's "at most two" is a second copy |
| `picker.rs:539` `first_taken_at`, `:548` `advances_spellcasting`, `:564` `class_edit` | class entry and the default `advances` | `advances_spellcasting` repeats `advances_arcane \|\| advances_divine` from `check_advances` |
| `picker.rs:852` `racial_modifiers`, `:885` `has_chosen_modifier` | the race/subrace modifier choice | `rules/src/engine/walk.rs` (around 410–413) |
| `picker.rs:623–627` and `:680–690` | "is this the current class", written twice | — |
| `picker.rs:824, 830`; `mythic.rs:45` (prose) | "Mythic rank 8", "Mythic rank 3" as literals | `PATH_SWITCH_RANK`, `PATH_CHOICE_RANK`, which `mythic.rs:6` already imports |

`Engine::data()` is public (`rules/src/engine.rs:63`), which is how the UI reaches round the engine: it scans raw `GameData` at `picker.rs:550, 806, 857, 889, 975, 1108` and `start.rs:91, 283`.

## Build

- **Each rule moves into `rules`** as an `Engine` (or `Catalog`/`labels`) method the picker calls, built test-first (`tdd`, as the map says for `rules`). Name them after what the UI asks, for example the possible advancement sets for a class at a level, a class edit, a race's modifier choice. The possible advancement sets come from the engine's rule, so a class that could advance three no longer depends on the UI's pairs.
- **The duplicated "current class" test** becomes one helper.
- **The mythic rank labels** are built from the constants.
- **Then `Engine::data()` becomes `pub(crate)`**, and each UI scan uses an engine or `Catalog` lookup instead (add the lookup in `rules` where none fits). The compiler then keeps the UI from reaching round the engine again.
- **`libs/proof` calls `engine.data()` too** (`links.rs:37, 56`, `fingerprint.rs:189`, `generate.rs:88`, `builds/mod.rs:33`). Have it keep the `GameData` it builds the engine from instead. `scripts/proof.sh rules` compiles the working tree's harness against `HEAD`'s crates as well, so the harness may use only API both sides have.
- Leave `slot_place` (`picker.rs:313`) and the English labels in `labels.rs` as they are.

The UI shows the same thing before and after.

**Done when:**
- `rules` has no `pub fn data`; `libs/ui` builds without it.
- None of the table's UI functions still decides a rule; each calls `rules`.
- The new `rules` methods have tests.
- `scripts/proof.sh rules <out-dir>` against `HEAD` reports no difference.
- `scripts/check.sh` passes.

## Progress log

Scratch dir (outside the repo): `/private/tmp/claude-501/-Users-thaynamarcela-code/20f0bb4f-7437-4fe9-8ef8-ebb8716f6fbe/scratchpad/t76/` (below, `t76/`).

- 2026-11-04: re-found every site at `HEAD` (31341d7). The table's lines still hold (`picker.rs` 539–585, 610–631, 670–703, 705–785, 805–811, 821–833, 852–902; `mythic.rs:45`). `engine.data()` is also called by four rules tests (`options.rs:85`, `path_rules.rs:54`, `codec.rs:219, 516`), which the `pub(crate)` change breaks too; they move to `world()` or the new lookups.
- 2026-11-04: seams (a small design choice, settled here): the new `Engine` methods, tested through `libs/rules/tests/engine/` as the map says for `rules`. The UI's "current class" helper stays in `libs/ui` (the table names no engine version; it reads the Build, it decides no rule).
- 2026-11-04: red → green, one slice at a time: `advances_options` (2 tests in `advances.rs`), then `asks_advances` and `class_edit` (3 more there), then `racial_modifier` and `chooses_racial_ability` (2 in `derived.rs`), then the lookups `races`, `backgrounds`, `skills`, `background_note`, `path_summary` (2 in `labels.rs`). Each test failed to compile before its method existed. 356 engine tests pass.
- 2026-11-04: decision: the possible advancement sets come from one rule, `most_advanced` (one class per spell type the rows advance under `Each`, one in all under `Either`), which `check_advances` now applies too (`serves.len() <= most_advanced(..)` in place of its `Either`/`_` arms; the same answers for every set). `advances_options` tries every set of held classes up to that size, in first-taken order, then none. The UI's sets were singles and pairs; the rule allows a pair only under `Each` with both spell types (Mystic Theurge), and for any other prestige class the UI's pairs were refused anyway, so the offered list is the same.
- 2026-11-04: decision: `Engine::data()` is removed, not made `pub(crate)`: nothing in `rules` called it (the engine reads through `Catalog::data`), so a `pub(crate)` copy only warned as dead code. The four rules tests use `world()` (`.canonical()` where order matters) instead.
- 2026-11-04: decision: the race list, background list and skill list are slice lookups on `Engine` (`races()`, `backgrounds()`, `skills()`, key order); a single background's note and a path's summary are keyed lookups in `labels.rs` beside `background_name` and `path_name`. The subrace-replaces-race choice of modifiers is one `Catalog::racial_modifiers`, and where a chosen bonus goes is one `racial_amounts`; the walk's start uses both, so the UI's copy and the walk's are now one.
- 2026-11-04: the proof keeps its game data: `proof::engine_on(data)` canonicalises it, builds the engine on a clone and returns both; `record`, `fingerprint`, `battery`, `fill_full`, `full_builds`, `builds::all/own` and `links::read/export` take `&GameData`. It uses only `GameData::canonical`, `Engine::new` and `LinkKeys::new`, which `HEAD` has too, and the data is the same canonical order `engine.data()` gave, so its Builds and batteries are unchanged.
- 2026-11-04: left as the ticket says: `slot_place` and `labels.rs`'s English. Also left: the Mythic track's "MR 1 and 2" (the ranks a chosen path grants, not one of the two constants).
- 2026-11-04: `scripts/check.sh` (with `wotr-planner-postgres` up): fmt, clippy, wasm-clippy, test, rules-wasm, seed-check OK, ALL-OK (log `t76/check.log`).
- 2026-11-04: `scripts/proof.sh rules t76/proof` against `HEAD` (31341d7): the working tree's harness built on both sides; 1999 Builds each side, 0 only in A, 0 only in B, 0 differing; `compare.txt` empty (log `t76/proof.log`).

## Answer

Every rule the picker decided now lives in `rules`, and the UI calls it; `Engine::data()` is gone.

- **New `Engine` methods** (all tested in `libs/rules/tests/engine/`):
  - `advances_options(build, level, option) -> Vec<BTreeSet<ClassKey>>`: the possible advancement sets for a prestige class taken at a level. Replaces the `Advances` component's candidate sets (each held class, every pair, then none, each through `apply` and `is_possible`). The set size now comes from the engine's rule, `most_advanced` (one class per spell type under `Each`, one under `Either`), which `check_advances` uses too. Tests: `the_advancement_choices_are_each_held_class_alone_then_each_pair_then_none_that_the_rules_allow`, `a_prestige_class_advancing_either_spell_type_offers_one_class_at_a_time`.
  - `asks_advances(build, level, class) -> bool`: a prestige class whose rows advance spellcasting, first taken at that level. Replaces `ClassPicker`'s `is_prestige && advances_spellcasting && first_taken_at`; `advances_spellcasting`'s copy of `advances_arcane || advances_divine` is `advanced_spellcasting`, shared with `check_advances`. Test: `only_a_prestige_class_first_taken_asks_which_spellcasting_it_advances`.
  - `class_edit(build, level, option, advances) -> Edit`: replaces the UI's `class_edit` and `first_taken_at`; `class_options` shares its default advancement (`held_advances`). Tests: `a_class_edit_sets_the_choice_only_where_the_class_is_first_taken`, `a_class_edit_gives_a_class_first_taken_its_archetype`.
  - `racial_modifier(start, ability) -> i8` and `chooses_racial_ability(start) -> bool`: replace `racial_modifiers` and `has_chosen_modifier` (`picker.rs`, used by `start.rs` and `RacePicker`). They and the walk's Build start share `Catalog::racial_modifiers` (a subrace's list replaces its race's) and `racial_amounts` (where a chosen bonus goes). Tests: `the_racial_modifiers_are_…`, `a_race_or_subrace_with_a_chosen_bonus_lets_the_start_choose_its_ability`.
  - Lookups for the UI's raw scans: `races()`, `backgrounds()`, `skills()` (`RacePicker`, `BackgroundPicker`, `SkillsPanel`), `background_note(key)` (`StartBody`), `path_summary(key)` (`PathPicker`). Tests: `the_start_pickers_list_the_races_backgrounds_and_skills_in_key_order`, `a_background_note_and_a_path_summary_come_from_the_game_data`.
- **UI:** the two "current class" tests are one helper, `holds_class_option` (`picker.rs`); "Mythic rank 3/8" and the Mythic track's prose are built from `PATH_CHOICE_RANK` and `PATH_SWITCH_RANK`. `slot_place` and `labels.rs` are unchanged.
- **`data()` closed:** removed rather than made `pub(crate)`, as nothing in `rules` called it (the engine reads through `Catalog::data`); `libs/ui` builds without it. Four rules tests use `world()` instead. `libs/proof` keeps its own canonical copy through `proof::engine_on(data) -> (Engine, GameData)` and passes `&GameData` to `record`, `fingerprint`, `battery`, `fill_full`, `full_builds`, `builds::all/own` and `links::read/export`; it uses only API `HEAD` has too.
- **Checks:** `scripts/proof.sh rules` against `HEAD`: no difference (1999 Builds, 0 differing, empty `compare.txt`). `scripts/check.sh`: ALL-OK.
