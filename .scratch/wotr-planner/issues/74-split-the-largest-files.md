# Split the largest files by subject

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The session retro (2026-11-03) found agents reading the largest files whole or in 1,000-line chunks, again and again. Split these four by subject (user, 2026-11-03):

| File | Lines |
| --- | --- |
| `libs/seeds/tests/game_data.rs` (the real-data tests; every ticket adds to it) | 9,731 |
| `libs/extract/src/build.rs` | 2,590 |
| `libs/extract/src/option_families.rs` | 2,432 |
| `libs/extract/src/option_levels.rs` | 2,209 |

## Build

The user is awake: settle small design choices yourself and record them here, and bring a real open question back in the report.

**The work is a pure move.** Code moves between files and gains the `mod`, `use` and visibility lines it needs, nothing else:
- no renamed items or logic;
- no change to `rules`, the schema, `libs/game-data/` or the link format.

**The shape:**
- **Each `src` file becomes a module folder**, for example `build/mod.rs` plus a file per subject. Group by what the code is about, so a reader can find a family, stage or rule by its file name.
  - Keep each file well under about 800 lines.
  - Each new file opens with a doc comment saying what it holds.
- **`build.rs`'s ordered passes stay readable in order in one place.** `mod.rs` keeps the sequence and each pass's reason (ticket 72's list); the work each pass does may move out.
- **`game_data.rs` becomes a folder test crate**, `libs/seeds/tests/game_data/`, with `main.rs` and a file per subject: feats, classes and archetypes, selections and options, mythic, backgrounds, alignment and so on. Follow the repo's `CLAUDE.md` on folder test crates: `mod` lines and `mod support;` for shared helpers.
- **The test list:** `cargo test --workspace -- --list` keeps the same count. Every name change is only a module-path change; record the old-to-new mapping in the scratch dir.

**Done when:**
- `scripts/extraction-diff.sh <out-dir>` against `HEAD` reports the extraction identical.
- `git diff HEAD -- libs/game-data` is empty.
- The test list has the same count, and each changed name differs only in its module path. A script checks this: strip the path, then compare the sorted lists.
- `scripts/check.sh` passes.
- No new file in the four splits is over 800 lines. Name any you couldn't get under, with the reason.
- The Answer lists each new file with a one-line summary, so the next agent can find things without opening them.

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

- 2026-11-03: saved the test list before any edit (740 tests). Split the four files one at a time with a script that slices line ranges into the new files and checks every source line lands exactly once. After each extract split: `scripts/extraction-diff.sh` identical (`xdiff-1` build, `xdiff-2` option families, `xdiff-3` option levels) and `cargo test -p extract` green; after the test split, `cargo test -p seeds` green (224 real-data tests).
- For each split, a token check (old file vs the new files, ignoring `use`/`mod` lines, module docs and the added visibility) differs only by the dropped `// ----` section markers, the added `impl` headers and `pub`/`pub(super)`, and trailing commas rustfmt added where a longer signature wrapped.
- Unused imports trimmed by `cargo fix`; each added `pub(super)` the crate compiles without was dropped again by a script.
- Eight doc links in `build` and `option_levels` lost their target with the move; they got explicit targets (same text), so `cargo doc -p extract --document-private-items` warns exactly as at `HEAD`.
- End: `scripts/check.sh` ALL-OK, a final `scripts/extraction-diff.sh` identical (`xdiff-final`), `git diff HEAD -- libs/game-data` empty, and the test-list check passes (740 = 740; 224 names gain a module path; mapping in the scratch dir's `test-renames.tsv`).

## Answer

**The four files are module folders now; a pure move.** No renamed item or changed logic; extraction byte-identical, `libs/game-data` untouched, the test list the same 740 tests with only module paths added. No new file is over 800 lines (the largest is `game_data/support.rs`, 702).

**`libs/extract/src/build/`** (was `build.rs`). The Builder's methods are `impl<'g> Builder<'g>` blocks in the subject files.
- `mod.rs` (294): `extract`, with ticket 72's ordered passes and each pass's reason in place; `Extraction`, `settled`, and the shared types (`Pending`, `FeatureOwner`, `Heading`, `Scope`, `Granted`, `Builder`, `Builder::new`).
- `skills_and_races.rs` (248): skills and the skills table, races, subraces and ability modifiers.
- `feats.rs` (139): feats by category, the Trickster's bonus feats, queuing `Prerequisites:` lines.
- `base_classes.rs` (360): `base_class` (a class and its archetypes), the class's alignment, an archetype's race and alignment rules, spell type, an archetype's skills row.
- `level_tables.rs` (312): the option family's plan for a table, what a cell names, level tables, BAB and save progressions, level rows.
- `grants.rs` (397): what a cell grants, picks, the providers' additions, `define_feature`, `feat_named`, bonus slots, `feats_in`.
- `selections.rs` (179): feature selections from headings, `add_options`, an option's feat.
- `prestige.rs` (221): prestige classes and their skills.
- `mythic.rs` (295): `MYTHIC_ASCENSIONS`, mythic abilities, mythic feats, mythic paths.
- `resolving.rs` (188): `place_features`, `add_prerequisite`, `resolve_prerequisites`.
- `sections.rs` (151): progression-table detection, `class_tables`, the headings a table can name, `find_heading`, `section_text`.

**`libs/extract/src/option_families/`** (was `option_families.rs`).
- `mod.rs` (177): the module doc, `provide` and `check` (the order), `Skills`, `Family`, and the shared helpers `all_text`, `contains`, `sharing`, `starts`.
- `domains.rs` (493): the 37 domains' powers table, the domains' class skills, `domain_short`, building the domain family.
- `domain_selections.rs` (449): taking the domains: the cleric's selection and every selection sharing it (inquisitor, druid, Blight Druid, Winter Child, Divine Hunter, Separatist and its alignments), and their check.
- `mysteries.rs` (368): the mysteries and revelations, `revelation_level`, Maneuver Mastery, the Dual-Cursed Oracle's revelations.
- `oracle.rs` (265): the oracle's and its archetypes' tables taking the mysteries, and their check.
- `living_deity.rs` (161): the Living Deity's mystery, revelations and domains, `living_deity_rows`, Impossible Domain.
- `blessings.rs` (151): the warpriest's blessings (Feral Champion, Champion of the Faith) and their check.
- `elements.rs` (317): the kineticist's elements, `element_feat`, Expanded Element, Blood Kineticist and Kinetic Knight, and their check.
- `hellknight.rs` (179): the Hellknight's order and Pentamic Faith, the Signifer's order, and their check.

**`libs/extract/src/option_levels/`** (was `option_levels.rs`).
- `mod.rs` (261): the module doc, `provide_families`, `give`, `Family`, `Owner`, `provide`, `settles`, `settled`, `bare`, and the re-exports the other modules use.
- `reading.rs` (209): `FeatNames`, `name_list`, `page`, `section`, an option's table, `own_cells`, `anchor_for`, bloodline feats, two-column tables.
- `rows.rs` (166): `row_names`, `RowBuilder`, and the `empty_row`, `feature`, `option`, `selection`, `list` constructors.
- `sorcerer.rs` (216): the sorcerer's bloodlines, Strength of the Abyss, the school power.
- `bloodrager.rs` (150): the bloodrager's bloodlines and the Primalist's choices.
- `tabled.rs` (162): wizard schools, cavalier orders, Magic Deceiver ways.
- `spirits_and_styles.rs` (154): shaman spirits and ranger combat styles (prose families).
- `class_takes.rs` (247): `Takes` and each class's (sorcerer, bloodrager, wizard, cavalier, Magic Deceiver, shaman, ranger).
- `taking.rs` (448): `Taking`, `Cell`, `Planned`, `TablePlan`: planning a table's cells and taking the family.
- `owner_facts.rs` (185): `provide_others`: Dragon Disciple, Eldritch Scion, Dragonheir Scion, Elemental Specialist, Disruptive.
- `check.rs` (134): `check`, what only the finished data shows.

**`libs/seeds/tests/game_data/`** (was `game_data.rs`), a folder test crate: `main.rs` (27) declares `mod support;` and one `mod` per file; each file does `use crate::support::*;` as `libs/rules/tests/engine` does. A helper used by one file stays in it; one used by several is in `support.rs`.
- `support.rs` (702): `DATA`, `engine`, Builds, slots, picks and lookups shared across files.
- `data.rs` (307): the data check, short links, and general facts (Shield Focus, Scribe Scrolls, feat lists, shared talents and hexes).
- `mythic.rs` (643): mythic path rules (26), Mythic Extra Feat and Extra Mythic Ability (55), Witch Wandering Hex and Beneficial Curse (62).
- `parameterised_feats.rs` (420): parameterised and repeatable feats (27).
- `class_rules.rs` (312): class rules (28).
- `option_levels.rs` (223): options with rows by option level (30).
- `option_families.rs` (605): domains, mysteries, blessings and elements (31), Impossible Domain and the druid (36), the Dual-Cursed Oracle's second curse (67).
- `follow_ups.rs` (471): feat and rule follow-ups (32).
- `option_follow_ups.rs` (687): option follow-ups (33) and second follow-ups (35).
- `backgrounds.rs` (159): backgrounds (34).
- `class_skills.rs` (617): options' class skills (37) and the class-skill cleanup (38).
- `report_gaps.rs` (429): the report's gaps (43), the sorcerer's level-1 feat and Extra Slayer Talent (44).
- `wild_talents.rs` (428): the kineticist's wild talents (45).
- `repeating_talents.rs` (290): talents that repeat (47).
- `extra_picks.rs` (511): Extra feats' extra picks (49), Extra Discovery and Extra Arcana (64).
- `mercies_and_exploits.rs` (589): mercies and the Arcane Enforcer's exploits (54), greater exploits and the Exploiter Wizard (57).
- `ki_powers.rs` (248): monk ki powers (58).
- `infusions.rs` (461): the kineticist's infusions (59).
- `borrowed_selections.rs` (574): talents, hexes and discoveries taken through another class (60).
- `bonus_feat_choices.rs` (494): archetype bonus-feat choices (61).
- `alignment.rs` (279): an archetype's alignment requirement replaces its class's (63).
- `negated_prerequisites.rs` (149): reasons for negated prerequisites (65).
- `instinctual_rage_powers.rs` (302): the Instinctual Warrior's monk rage powers (66).

**Decisions (small, settled here):**
1. **The added lines are visibility and paths only.** A method or helper used from another file got `pub(super)` (in the tests, `pub` in `support`); a few struct fields read across files did too (`Takes`', `RowBuilder`'s). The old public paths stay through re-exports in each `mod.rs` (`build::section_text`, `option_families::domain_short`, `option_levels::page` and the rest), so no caller changed.
2. **The `// ----` section markers** were dropped where a file now holds one section (its doc says it), and kept as separators in the test files that hold several tickets' sections.
3. **Doc links that lost their target** (`Pick::Feats` and three more variants, `Picks` twice, `Builder::base_class`, the `check` fn/module ambiguity) got explicit targets, with the same text.
4. **Tests are grouped by subject, keeping each ticket's section whole**, merging related tickets in one file (for example 54 and 57, the exploits). Each file's doc names its tickets.
