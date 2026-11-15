# The game check: the data against its blueprints

Type: build
Status: resolved
Blocked by: 236, 237
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Problem

Decided in [234](234-game-blueprints-as-a-source.md) (user, 2026-11-14): the game wins on rules, and a check against the game replaces both the spot checks of [146](146-checking-the-data-against-the-guide.md) and the regression checks removed by [235](235-remove-the-regression-checks.md).
- It is code of our own in the repo that reads the community's `.binz`. The user took the recommendation, and went further on the regression checks.
- It runs by hand, since worktrees and CI have no dump.

## Change

A tool in `tools/`, a binary such as `cargo run -p game-check`:
- **Input:** it reads the `.binz` (or its extraction) from `blueprints/` ([236](236-the-game-sources-in-the-repo.md)), and resolves GUID references and the strings file for names.
- **What it compares.** For each entry with a `blueprint` ([237](237-a-blueprint-guid-per-entry.md)):
  - **prerequisites:** the `Prerequisite*` components, with `Group` All or Any; stat values, BAB, feats and features, class levels, caster level, `PrerequisiteNoFeature`, parametrized weapon subcategories, alignment, race;
  - **class tables:** hit die, BAB and save progressions, skill points, class skills;
  - **progression rows:** each level's features and selections, for classes, archetypes (added and removed rows) and prestige classes;
  - **selection option lists:** a selection's options against the blueprint's `m_AllFeatures`;
  - **mythic paths:** each rank's grants;
  - **items:** the always-on bonuses, with their type and value.

  For each area it says what agrees and lists each difference, with the entry's key, the blueprint's name and GUID, and the two values.
- **Output.** A plain-text report grouped by area and kind of difference, largest group first. The exit status is non-zero when anything differs, unless an area is listed as known with its ticket.
- **Deterministic.** The same dump and the same data give a byte-identical report:
  - **Order.** Groups by count, descending, then by area and kind. Within a group, by entry key, then blueprint GUID. Never in a hash map's iteration order.
  - **Content.** No timestamps, timings, absolute paths, thread counts or other varying text in the report. A run's timing, if printed, goes to stderr.
  - **Its input.** The check reads only the pinned dump. It verifies the `.binz`'s SHA-256 against the pin of [236](236-the-game-sources-in-the-repo.md) before reading, and refuses a different file.
- **Words.** It prints numbers, keys, names and GUIDs, never the game's description texts.

`CLAUDE.md` says when to run it: after any data ticket and once per queue, by the main session. It also says that each difference is applied without asking unless it touches one of the user's facts.

## Answer

**Built (2026-11-15):** `tools/game-check`, a workspace member, binary and library. `cargo run -p game-check -- [--binz <file>] [--data <dir>]`; the dump defaults to `blueprints/<pinned asset>` of the checkout it was compiled in, so from a worktree pass `--binz /…/wotr-planner/blueprints/blueprints_raw_2.7.0x_0.binz`. About 2 s a run in a dev build.
- **Input** (`src/binz.rs`, `src/pins.rs`, `src/dump.rs`):
  - The pins come from `scripts/game-sources.pins`, embedded with `include_str!`.
  - The dump's size is checked, then its SHA-256, before anything is decoded. A difference is `Error::Refused`, naming both values, and exit 2.
  - The `.binz` is read chunk by chunk (16-byte header; LZ4 blocks through `lz4_flex` 0.14, decode only). Only the blueprint chunks (33/0) are read.
  - Each record is GUID, asset name, type and JSON. The JSON is kept, and read as UTF-8, only for the kinds a comparison reads (classes, archetypes, races, features of every sort, progressions, stat tables, items, enchantments). It is parsed on first use.
  - The strings chunk is never read.
- **The planner's side** is `GameData::blueprints()` (ticket 237) and the data's own helpers (`selections`, `feat_lists`, `effective_row`, `Class::save_bonus`, `FeatureSelection::offered`).
- **Areas** (`src/areas/`), each counting what it compared, what agreed and what it left out, and why:
  - **mapping:** the blueprint is in the dump and its type fits the kind. A prestige class must be one in the game, a mythic path a mythic class, and an archetype among its class's `m_Archetypes`.
  - **prerequisites** of feats, options, mythic abilities and feats, classes and archetypes:
    - Both sides go into one normal form over GUIDs: `All` and `Any` flattened and sorted, and the alignments in one group folded into a mask. This follows `BlueprintFeature.MeetsPrerequisites` and `BlueprintCharacterClass.MeetsPrerequisites`, WrathDecompiled 6ce4a6b.
    - The two forms are compared leaf by leaf: missing in planner, not in game, value differs, grouping differs, or a blueprint the planner doesn't map.
  - **class tables:**
    - Skill points and class skills, both tagged "(the user's facts)".
    - BAB and saves compared by value per level, from `BlueprintStatProgression.Bonuses`.
    - For archetypes: `AddSkillPoints`, `ReplaceClassSkills` and the save overrides.
  - **progression rows**, for classes, prestige classes and archetypes:
    - Archetypes are applied as `ProgressionData.CalculateLevelEntries` does.
    - Unmapped game grants are unfolded: a fact wrapper into its features, a nested progression into its rows, a selection into a pick of its `Group`. A hidden one is dropped.
    - Then the rows are paired: same level, another level, the game's own copy of a selection the planner shares, or a bonus pick of another group.
    - What is left is listed as "in one side only" or "a further rank".
  - **option lists:**
    - A selection's offered options, plus the pools of a `pool_of` selection, against `m_AllFeatures`. A feat list against its own `m_AllFeatures` and `IgnorePrerequisites`.
    - The general feat, mythic ability and mythic feat pools against `BasicFeatSelection`, `MythicAbilitySelection` and `MythicFeatSelection`.
    - "a choice the game doesn't offer": the planner's selection names a blueprint that isn't a selection.
  - **mythic ranks:**
    - A path's class rows by rank. The rank offset comes from the class's own `PrerequisiteClassLevel(MythicStartingClass)`, or from `PrerequisiteMythicLevel` for a later path.
    - The ranks before the offset come from `MythicStartingProgression`, plus the path's `…FirstLevelsProgression` (a fixed table of six, by GUID).
    - The general picks are left out.
  - **items:**
    - The slot, by blueprint type.
    - Flat always-on bonuses through enchantments and wearer facts: `AddStatBonusEquipment`, `AllSavesBonusEquipment`, `AddStatBonus`, simple `AddContextStatBonus`, and the spell DC, caster level, spell penetration and concentration components. Each is compared by target, type and amount.
- **Report** (`src/report.rs`):
  - A header (the asset, the pinned hash, the counts), then each area's counts and what it left out.
  - The differences grouped by (area, kind, known ticket), largest group first, then by area, kind and ticket. Inside a group, sorted by entry key, GUID, game value, planner value.
  - Asset names and GUIDs only. No path, no time (the run's time goes to stderr). BTreeMaps throughout.
- **Known list** (`tools/game-check/known.txt`, `src/known.rs`): `<area>[ / <kind>[ / <entry>[ / <value>]]] -> <ticket>`, `*` for any kind, the most specific line winning.
  - A known finding is grouped as `[known: <ticket>]`.
  - The exit status is 1 when a difference isn't known or a known line matches nothing, 0 otherwise, and 2 on refusal.
- **Workspace:** the member is added; `[profile.dev.package]` sets `opt-level = 3` for `sha2`, `lz4_flex` and `serde_json`.

**Decisions:**
- The main session, 2026-11-15:
  - The report names blueprints by asset name and GUID, never by display name, so the strings chunk is never read. The ticket's "the strings file for names" became asset names, since display names are game text.
  - No machine-readable sharing or "also" list (the user rejected a sharings file). What copies, splits and multi-blueprint entries cause shows up as differences, grouped in `known.txt` under the ticket that removes them: `ranks-and-filtered-pools` and `game-data-by-blueprint`.
  - Feat slots are matched by the game selection's `Group`. `OptionList` is counted, not compared.
  - `Feature{count>1}` is compared as Has(feature) and counted.
  - Manuals and tomes are deferred. Spells and spell lists get the mapping check only.
  - The dev-profile opt-levels.
- The pins file as the check's source, refusing any other dump (ticket 236's decision).
- **Ours, inferences**, each commented where it's applied:
  - `PrerequisiteCasterType` (a class with a spellbook of that type, `PrerequisiteCasterType.CheckInternal`) is the planner's spell level 1.
  - A game ability prerequisite of 5 or less always holds (7 bought, less a racial 2), e.g. `ShatterDefenses`' Intelligence 3.
  - The planner's `Category(Magic)` slot is the game's `WizardFeat` group (`WizardFeatSelection`, `SkaldFeatSelection`, `SorcererBonusFeat`).
  - A penalty is untyped, as the planner's `GearBonus` says.
  - A planner `Any` naming every option a selection offers is the selection itself (`ExtraDiscoverySelection` asks for `DiscoverySelection`).
- **Ours, normalisations:**
  - Prerequisites that hold for any Build or concern the story or pets are dropped and counted (not a pet, `MainCharacter`, `Etude`, `NoClassLevel(AnimalClass)`, a class's forced-true level in itself).
  - The planner's leaves with no game form are dropped and counted (`Story`, `PickedThrough`, `NoOtherOptionOfGroup`, …).

**Tests:** `tools/game-check/tests/fixture/`, a hand-written dump in our own words (`blueprints.json`, invented names, 60 records) written as LZ4 chunks at test time, with its pins and `planner.ron`.
- `areas::` has one test per area, each asserting every planted difference and the entries that agree:
  - `mapping_finds_a_missing_blueprint_a_wrong_type_a_prestige_flag_and_a_stray_archetype`
  - `prerequisites_find_a_value_a_missing_leaf_an_extra_one_a_grouping_and_an_unmapped_blueprint`
  - `class_tables_find_skill_points_a_save_and_class_skills_for_a_class_and_an_archetype`
  - `progression_rows_find_a_grant_at_another_level_in_one_side_only_and_an_archetype_s_removal`
  - `option_lists_find_options_on_one_side_a_list_s_flag_a_choice_the_game_lacks_and_the_pools`
  - `mythic_ranks_find_a_grant_at_another_rank_after_the_starting_ranks_and_a_nested_progression`
  - `items_find_an_amount_a_type_a_slot_and_a_bonus_on_each_side`
- `determinism::`:
  - `a_shuffled_dump_and_data_give_a_byte_identical_report`: records reversed and rotated across three chunks, the planner's lists reversed.
  - `the_report_holds_no_path_or_time`.
- `refusal::`:
  - `a_dump_whose_hash_is_not_the_pin_is_refused_naming_both`
  - `a_truncated_dump_is_refused_on_its_size`
  - `the_pinned_dump_reads_every_record`
- `known::`:
  - `a_known_group_names_its_ticket_and_a_stale_line_fails`
  - `the_check_passes_when_every_difference_is_known`
  - `an_entry_line_wins_over_its_group`
- `pins::`:
  - `the_check_reads_the_dumps_pins_from_scripts_game_sources_pins`
  - `the_known_list_parses`
- With every comparison switched off, the 7 area tests, 3 known tests and 2 determinism tests failed; the 5 others passed. `scripts/check.sh` runs them, since the crate is a workspace member. The real run is not in it: worktrees and CI have no dump.

**First run** (the whole data, run twice: `cmp` identical; exit 1):
- **Areas:**
  - mapping: 4,309 compared, all agree. Left out: 175 entries with no blueprint.
  - prerequisites: 1,393 compared, 821 agree.
  - class tables: 235 compared, 179 agree.
  - progression rows: 235 compared, 33 agree.
  - option lists: 202 compared, 93 agree.
  - mythic ranks: 10 compared, 3 agree.
  - items: 82 compared, 74 agree.
- **Differences:** 4,889 in 33 groups. 1,653 are known, in 8 groups; 3,236 are not, in 25 groups.
- **Known, ranks-and-filtered-pools:**
  - progression rows / a further rank in planner only: 594
  - option lists / in game only (the per-mystery revelation pools and the Magic Deceiver's): 559
  - progression rows / a further rank in game only: 180
  - mythic ranks / a further rank in planner only: 8
- **Known, game-data-by-blueprint:**
  - progression rows / a selection the game keeps separate: 193 (bard, skald and slayer talents, skald rage powers, …)
  - option lists / in game only: 57; in planner only: 56 (domains, blessings, Enforcing Gaze, the kineticist element lists, the pool entries of Advanced Weapon Training, Extra Channel, Extra Hex, Magical Tail and Abundant Smite)
  - mythic ranks / a selection the game keeps separate: 6
- **Not known, prerequisites:**
  - missing in planner: 619
  - missing in planner, a blueprint it doesn't map: 579
  - not in game: 529
  - value differs: 9
  - grouping differs: 3
- **Not known, progression rows:**
  - in game only: 586
  - in planner only: 136
  - a bonus pick of another group: 56
  - at another level: 9
- **Not known, option lists:**
  - in game only: 401
  - in planner only: 193
  - a choice the game doesn't offer: 11
  - ignores prerequisites or not: 7
- **Not known, class tables:**
  - Will differs: 33
  - class skills differ (the user's facts): 13
  - Reflex differs: 11
  - Fortitude differs: 3
  - skill points differ (the user's facts): 2
  - BAB differs: 1
- **Not known, mythic ranks:** in game only: 23; in planner only: 2.
- **Not known, items:**
  - bonus not in game: 3
  - slot differs: 3
  - bonus missing in planner: 2
  - type differs: 2
- **237's three "differences for 238" all show:**
  - `archetype alchemist/incense-synthesizer` / progression rows / in planner only: `incense-fog at level 1`.
  - `selection shifter/dragonblood-shifter/greater-wyrmshifter` / option lists / a choice the game doesn't offer: `ShifterDragonFormFeatureFinal`, with 10 options in the planner.
  - `selection stalwart-defender/defensive-powers` / option lists / in planner only: `clear-mind (ClearMindFeature)`. `immobile` and `mighty-resilience` have no blueprint and are counted as such.

The full report is kept outside the repo, since it holds names of the game's data in bulk. The not-known groups wait on difference tickets, the ones from `research/game-source-check-2026-11-14.md` (missing in the main checkout at 667164e) or new ones (the main session's, per the decisions). Groups of the user's facts (skill points, class skills) go to the user first.

**Proposed for CLAUDE.md** (the main session writes it): "Run the game check after any data ticket and once per queue (main session, main checkout): `cargo run -p game-check`. It prints the differences between the data and the game's blueprints, grouped by area and kind, largest first. A difference is applied by its ticket without asking, unless it touches one of the user's facts (the skill tables, backgrounds, …), which goes to the user first. A difference to be fixed later goes in `tools/game-check/known.txt` under its ticket; the check fails on a difference not known and on a known line that matches none."

**Suggestions for rules** (not this ticket's to edit):
- A public `GameData` way to list the options a `pool_of` selection offers (the check rebuilds `Catalog::pool`).
- A game group per `FeatSlot` category, so a slot pairs like any grant.
- A `Prerequisite::CasterType`, to replace the spell-level-1 inference.

**Snapshot message:** none; ticket 235 removed the snapshot check, and this ticket changes no engine or data. **What the queue's proof should show:** nothing; no engine or data change.

## Done when

- The check runs over the whole data and prints its report.
- A test on a small fixture (hand-written blueprint JSON in our own words: no game text) proves each comparison finds a planted difference.
- Two runs over the whole data give byte-identical reports (`cmp`), and so does a test on the fixture, run with entries in a shuffled order.
- A dump whose hash isn't the pinned one is refused before anything is compared.
- Its first run's differences are each covered by a ticket: those already filed from [research/game-source-check-2026-11-14.md](../research/game-source-check-2026-11-14.md), or new ones.
- `scripts/check.sh` passes.
