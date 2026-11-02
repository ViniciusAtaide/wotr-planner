# Organise the engine tests

Type: task
Status: resolved
Blocked by: 51
Map: [WotR Build Planner](../map.md)

## Question

**User, 2026-11-02:** `rules/tests/engine/` (about 12,400 lines) reads like a library of its own. `support/mod.rs` alone is about 3,000 lines: the test world (an invented game: classes, archetypes, feats, selections, option rows, mythic paths, races, backgrounds, built in Rust) plus the Build helpers (`build`, `times`, `class_slot`, `pick_feat`, …), grown a bit by every ticket since 11. Several test files are named after the ticket that added them (`follow_ups.rs`, `option_follow_ups.rs`, `second_follow_ups.rs`, …), not the rule they test.

1. **Split `support/`** into modules: `support/world/` (one file per part of the world: classes, feats, selections, paths, races, backgrounds, …) and `support/builds.rs` (the helpers), with `support/mod.rs` only declaring and re-exporting them. Move-only: the world's contents don't change.
2. **Name test files by topic:** move the tests from ticket-named files (`follow_ups`, `option_follow_ups`, `second_follow_ups`, and any other named after a ticket) into the topic files they belong to (`parameters`, `option_levels`, `path_rules`, `removable`, …, or a new topic file where none fits), and delete the emptied files. Test names and bodies don't change.

Not in scope: describing the test world in RON files instead of Rust (the user chose the split only).

## Build

AFK; the only agent in the repo; touches only `rules/tests/`. Follow `CLAUDE.md`'s test-crate rules (every file gets its `mod` line in `main.rs`; shared helpers via `mod support;`).

**Done when:** `cargo test -p rules -- --list` lists the same tests before and after, apart from their module paths (save both lists and a mapping in the scratchpad); every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes; no file in `rules/tests/engine/support/` is over about 600 lines; no test file is named after a ticket.

## Progress log

- 2026-11-02: baseline at HEAD `506f6e7`: `cargo test -p rules -- --list` (318 integration + 4 unit) and a copy of `rules/tests/engine/` saved in the scratchpad (`t52/before-raw.txt`, `t52/engine-orig/`, `t52/support-orig.rs`).
- 2026-11-02: `t52/split.py` cut HEAD's `support/mod.rs` into `support/builds.rs` and `support/world/**` by whole-item line ranges, so the moved code is the original text; `t52/wire.py` added the `use` lines and the visibility the move needs. A line-multiset check finds nothing added or lost but module docs, `mod`/`use` lines, the visibility prefixes and the dropped `// ---- the world` divider.
- 2026-11-02: `t52/move.py` moved the 36 tests of `follow_ups.rs`, `option_follow_ups.rs` and `second_follow_ups.rs` (and their helpers) into topic files by line ranges, verbatim, and the three files were deleted. Its check lists the source lines it didn't place: only imports, module docs and helpers the destination already has with the same body.
- 2026-11-02: verification passed (see Answer). Nothing committed; RustRover's staging undone (`git restore --staged .`), `server/src/main.rs` untouched.

## Answer

**Done.** `support/` is 17 files, none over 368 lines, and `support/mod.rs` only declares and re-exports. No test file is named after a ticket. The test list is the same 322 names (318 + 4 unit); only the 36 moved tests changed module path.

### `support/` layout (lines)

| File | Lines | Holds |
| --- | ---: | --- |
| `support/mod.rs` | 12 | the module docs, `#![allow(dead_code)]`, `mod builds; mod world;`, `pub use builds::*; pub use world::world;` |
| `support/builds.rs` | 164 | keys (`feat`, `class`, `skill`, …, `weapon`, `skill_value`), picks (`pick_feat`, `pick_option_for`, …), slots (`general_slot`, `class_slot`, …), Builds (`scores`, `point_buy`, `build`, `times`, `engine`) and impossible Builds (`without`, `why_not`) |
| `support/world/mod.rs` | 139 | `world()`, `progression`, `guide`, `skill_row`, and the builders every part uses: `list`, `plain_feature`, `bonus_feature` |
| `support/world/backgrounds.rs` | 59 | `backgrounds` |
| `support/world/races.rs` | 94 | `races` (with subraces) |
| `support/world/feats.rs` | 304 | `weapons`, `feat_row`, `param_feat_row`, `repeat_feat_row`, `needs_feat`, `needs_score`, `feats` |
| `support/world/selections.rs` | 88 | `selection_option`, and the "options with rows by option level" builders: `option_row`, `with_class_skills`, `levelled_selection`, `levelled_option` |
| `support/world/mythic.rs` | 111 | `mythic_abilities`, `mythic_feats` |
| `support/world/paths.rs` | 368 | `trickster`, `rank_row`, `legend`, `gold_dragon`, `swarm_that_walks`, `aeon`, `devil` |
| `support/world/classes/mod.rs` | 71 | the class files' `mod` lines and re-export; `row`, `archetype_row`, `archetype` |
| `support/world/classes/martial.rs` | 351 | `fighter`, `monk` + `zen_archer`, `warpriest`, `ranger` |
| `support/world/classes/rogues.rs` | 273 | `rogue`, `slayer` + `slayer_style`, `slayer_level`, `slayer_level_below`, `combat_style` |
| `support/world/classes/arcane.rs` | 219 | `wizard`, `witch` + `false_god`, `bard` |
| `support/world/classes/sorcerer.rs` | 303 | `BLOODLINES`, `sorcerer` (with its archetypes) |
| `support/world/classes/divine.rs` | 260 | `druid`, `cleric` + `CLERIC_DOMAINS`, `separatist`, `accursed` |
| `support/world/classes/kineticist.rs` | 122 | `kineticist` |
| `support/world/classes/prestige.rs` | 265 | `mystic_theurge`, `arcane_trickster`, `eldritch_knight`, `winter_witch`, `dragon_disciple` |

Classes take seven files because together they are about 1,800 lines. The groups are the classes that build on each other (rogue and slayer share talents; the cleric's archetypes; the prestige classes).

What isn't a pure move in `support/`:

- **Visibility.** A builder used by a sibling file is `pub(super)` (`selection_option` and the option-row builders, `needs_feat`, each part's top function such as `feats` or `races`). The class functions are `pub(in crate::support::world)`, so `classes/mod.rs` can re-export them to `world()`; `pub(super)` there would have been too narrow to re-export. Builders in `world/mod.rs` and `classes/mod.rs` stay private: the child files see them as descendants.
- **Imports.** Each file has `use rules::*;` as before, plus explicit imports of the helpers it calls (`crate::support::builds::{feat, …}`, `crate::support::world::{guide, …}`).
- **Module docs.** One short `//!` line per new file. The `// ---- the world` divider is gone; the other section dividers moved with their code.

The tests still reach everything through `use crate::support::*;` (the glob re-export) or `crate::support::world` (`canonical.rs`), so no test's `use` line changed for the split. The `testing` feature is untouched: rules' dev-dependency still turns it on.

### Where the tests moved

| From | Test | To |
| --- | --- | --- |
| `follow_ups` | `a_levels_slots_are_checked_class_selections_first_then_class_slots_race_slots_and_the_general_slot`, `a_human_warpriests_race_slot_takes_dazzling_display_on_the_class_slots_weapon_focus`, `a_human_druid_with_a_companion_takes_boon_companion_at_level_1` | `slots` (new section: pick order within a level) |
| `second_follow_ups` | `a_subrace_without_the_races_slot_gets_none`, `a_subraces_own_slot_may_name_the_races_list` | `slots` (new section: a subrace's feat slots) |
| `follow_ups` | `a_half_elfs_race_slot_is_skill_focus_in_a_skill_of_choice` | `feat_lists` |
| `follow_ups` | `spell_focus_mythic_needs_spell_focus_in_the_same_school`, `a_parameterised_mythic_feat_names_its_value_and_repeats_once_per_value`, `the_mythic_feat_picker_offers_a_parameterised_one_once_per_value`, `armor_focus_needs_the_armour_proficiency_of_the_armour_type_it_is_taken_for`, `a_mantis_zealot_holds_its_weapon_feats_in_its_one_weapon`, `a_selection_whose_one_option_takes_a_value_is_still_a_pick`, `the_way_of_the_bow_offers_only_its_listed_weapons`, `the_way_of_the_bow_gives_weapon_focus_at_2_and_weapon_specialization_in_the_same_weapon_at_6`, `an_option_can_take_a_value_for_its_rows_alone` | `parameters` |
| `follow_ups` | `perfect_mind_raises_bab_and_every_skills_ranks_for_prerequisites` | `path_rules` |
| `follow_ups` | `destiny_beyond_birth_removes_the_races_ability_penalty` | `derived` |
| `follow_ups` | `a_slayer_talent_at_2_can_be_a_combat_style_feat_from_the_first_list`, `each_combat_style_feat_comes_in_its_own_window_of_levels`, `a_slayer_takes_three_combat_style_feats_at_most` | **`combat_styles`** (new) |
| `option_follow_ups` | `a_slayer_picks_one_ranger_style_with_his_first_combat_style_feat` | `combat_styles` |
| `second_follow_ups` | `a_ranger_slayers_combat_style_feat_comes_from_the_windows_stage` | `combat_styles` |
| `option_follow_ups` | `a_crossblooded_sorcerer_cant_take_a_second_draconic_bloodline`, `second_bloodline_cant_take_a_second_draconic_bloodline`, `an_option_of_a_group_is_held_in_the_states_groups`, `expanded_element_at_15_cant_take_the_7th_level_element_again`, `an_element_doubled_at_7_isnt_taken_a_third_time` | **`option_limits`** (new) |
| `option_follow_ups` | `crossblooded_second_bloodline_gives_its_arcana_too`, `a_separatists_second_domain_lags_two_levels_behind`, `impossible_domain_levels_its_domain_like_the_domains_held`, `a_maneuver_comes_at_its_revelations_option_level`, `a_pool_selection_also_offers_its_own_options`, `a_pool_selection_is_checked_after_the_selection_it_draws_on` | `option_levels` |
| `second_follow_ups` | `a_living_deitys_impossible_domain_runs_on_the_ways_levels_from_the_pick` | `option_levels` |
| `option_follow_ups` | `a_champions_alignment_blessing_matches_his_alignment`, `a_warpriests_blessing_doesnt_check_his_alignment` | `options` |

`combat_styles.rs` (189 lines): combat style feats through a slayer talent (windows, at most three, the ranger's style picked with the first, the stage's list). `option_limits.rs` (142 lines): how often an option may be held (one option of a group, an element at most twice, Expanded Element not the 7th-level element again). Neither fits an existing file. The combat style tests came from all three ticket files and make a topic of their own; the nearest file, `shared_options`, has an `options` helper with another signature. The limits sit beside nothing: `option_levels` (the other kineticist tests) already has a `kineticist` helper with another signature, and the Draconic group checks aren't about option levels.

Each test moved with its section divider and the helpers it calls, appended at the end of its new file. Test bodies, helper bodies and test names are unchanged:

- **A helper the destination already has with the same body was not copied:** `values_at`, `options` (`parameters`); `with`, `at`, `holds`, `status`, `offered`, `ABYSSAL`, `ARCANE`, `RED` (`option_levels`); and the second `option_feat`, `style_slot` and `WAR_DOMAIN`.
- **A helper two destinations need is in both:** `with` and `status` (`options`, `option_limits`), `crossblooded` (`option_levels`, `option_limits`), the keys-only `options` (`slots`, `combat_styles`).
- **Two same-body pairs under different names stay as they were**, because the test bodies call them by name: `combat_styles` has `offered` and `options`, and `combat_style` and `slayer_talent`; `parameters` has `spell_school` and `school`. Merging them means editing test bodies, so it's a follow-up if wanted.
- **Module docs** of the destination files gained a clause for what they now hold (`slots`, `feat_lists`, `parameters`, `path_rules`, `derived`, `option_levels`, `options`); `path_rules`' was rewrapped. Their `use rules::{…}` lines gained the types the moved tests use.

No file kept most of its content, so nothing was `git mv`'d: the old `support/mod.rs` and the three ticket files were split several ways.

Test file sizes after the move: `option_levels` 1,026 (was 807), `parameters` 984 (715), `path_rules` 892 (860), `options` 425 (355), `slots` 216 (110), `derived` 192 (171), `feat_lists` 108 (84). `rules/tests/engine/` is 12,584 lines (was 12,390; the difference is the new files' imports and docs and the duplicated helpers).

### Test-list comparison

`cargo test -p rules -- --list` before and after (`t52/before-raw.txt`, `t52/after-raw.txt`):

- 322 tests each (318 in `engine`, 4 unit tests).
- The names without module path, sorted (`t52/before-names.txt`, `t52/after-names.txt`), are **identical**, and no name repeats.
- With full paths, 286 are unchanged and 36 moved; `t52/mapping.txt` lists each `old::path -> new::path`.

### Verification (ticket 22's checks)

- `cargo test --workspace`: pass. rules 318 + 4 unit (unchanged), data files 25, game_data 150, postgres 26, extract 91.
- `cargo clippy --workspace --all-targets`: clean (also re-run on `rules` after touching the test crate).
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass.
- `cargo run -p data --bin seed -- --check-only`: 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 38 backgrounds.
- No DB change, so no reset or reseed. Only files under `rules/tests/` changed (and this ticket and the map).
