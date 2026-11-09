# Spells as game data

Type: design
Status: open (option B decided; stage 1, the foundation, built 2026-11-09; stages 2 and 3 to do)
Blocked by: none (the user took option B, 2026-11-09; the source is saved)
Map: [WotR Build Planner](../map.md)

## Question

The map's Out of scope says "Spells known or prepared: not part of the class/feat/mythic summary", and the MCP instructions make spells the agent's own knowledge. The Max DC run (2026-11-09, see [119](119-caster-line.md)) showed what that costs a caster build:
- Every spell it recommended (Hold Monster, Overwhelming Presence, Euphoric Tranquility, Power Word Stun, Stinking Cloud, Boneshaker) was unchecked: not their school or subschool, not their save, not whether they're on the sorcerer's list, not their level.
- It picked Expanded Arsenal for Conjuration and then Necromancy without being able to ask which spells those schools hold that aren't mind-affecting.
- Its biggest DC sources depend on a spell's subschool (Fey: compulsion, Infernal: charm), which the planner can't tell.

This is a scope decision, as [86](86-deities.md) was for deities.

## What the saved pages hold

The guide's three spell pages were added to `PAGES` in `scripts/save-guide-pages.sh`, and the user saved them on 2026-11-09 (`guide-cache/SAVED_AT`). Read the same day, with the 38 pages already saved:

- **The shape.** Every spell list is a table per spell level (Spell, School). A spell's facts are given once, in a block under the first list that has it: Target, Duration (when it has one), Saving Throw, Spell Resistance, Spell Descriptors and Casting Time, then a description. Other lists only name the spell in their table. For example, the bard's level 0 table lists Daze, whose block is on `wizard-spells`. Some names carry footnote markers ("Acid Splash1"), which an extraction must strip.
- **The subschool is there:** it's in Spell Descriptors (Hold Monster: "Mind-Affecting, Compulsion"), so [119](119-caster-line.md)'s question 3 could take it from the spell.
- **`wizard-spells`:** "the spell selections for Wizard and Sorcerer characters", levels 0–9, 280 spell blocks. Every spell the run recommended is there: Hold Person, Hold Monster, Hideous Laughter, Stinking Cloud, Boneshaker, Euphoric Tranquility, Overwhelming Presence, Power Word Stun.
- **`cleric-spells`:** levels 0–9, 104 blocks, for spells not already on the wizard page (Cure Light Wounds, Bless, Flame Strike, Heal).
- **`druid-spells`:** levels 0–9, 53 blocks (Entangle).
- **Lists in pages already saved**, each with blocks for its own spells:
  - classes: bard, bloodrager, hunter, inquisitor, magus, paladin, ranger, shaman, warpriest and witch (`<class>#<class>-spells`), and the alchemist's extracts (`alchemist#alchemist-extracts`);
  - archetype and option lists: the shaman spirits' bonus spells, the oracle's bonus and additional spells, the Beast Tamer's bonus spells, the Magic Deceiver's spells;
  - the Mythic spellbooks: aeon, angel, azata, demon, gold dragon, lich and trickster (`<path>#<path>-spells`). Angel and lich also have an "unmerged" list.
- **Classes that use another class's list**, as their pages say: the arcanist uses the wizard's ("Uses the Wizard spell list"), the oracle the cleric's, and the skald the bard's.
- **Spells per day:** each casting class's page has a table of "BAB progression, saves and spells per day per level", so question 4 has a source.

So the guide covers every casting class's list and every Mythic spellbook. It doesn't give each spell's level per class in one place; that has to be built by joining the tables.

## Options

- **A. Stay out of scope:** spells are the agent's knowledge, as today.
- **B. A spell catalogue, not spell picks:** each spell's name, school, subschool, descriptors (mind-affecting, fear, death), save, spell resistance, area or targets, and level per class list, with a summary like 112's. `search` and `describe` cover it (`kind: spell`); no Build choice changes. Only reference data.
- **C. B plus spells known:** a sorcerer's or oracle's spells known become picks per level, a wizard's spellbook a list. A Build choice, a link format change and a large UI.

## Open questions (with recommendations)

1. **Which option?** *Recommend:* B. It answers "which compulsion spells can a sorcerer cast at 9th level" without changing a Build. C is a different product.
2. **Source:** *Recommend:* the three spell pages for the spell blocks, and the class, archetype and path sections above for which list holds a spell at which level. A spell named in a list with no block anywhere is a data issue that the extraction reports; there is no other source (the map's "the guide wins" rule). The descriptions become summaries in our own words, as in [112](112-effect-summaries.md), never the guide's text.
3. **Mythic spellbooks:** a path's spells go into the class spellbook it merges with. *Recommend:* catalogue them as lists of their own (`azata`, `lich`, ...) and leave the merging rules for C, since B has no spell picks.
4. **Spells per day:** Abundant Casting's summary gives +4 slots at levels 1–3, but the sheet gives no slots to add to. *Recommend:* a separate ticket. The tables are on the class pages, so it no longer needs a scope decision, only the user's interest.
5. **The map's Out of scope entry:** today it reads "Spells known or prepared: not part of the class/feat/mythic summary." *Recommend:* whichever option is taken, the answer rewrites that entry so the map says what's now covered. With B: the spell catalogue is in scope (reference data, this ticket) and spells known or prepared stay out. With C: the entry leaves Out of scope for Notes. With A: the entry stays and names this ticket as the decision.

## Plan: three stages, the middle one parallel

About 630 spell blocks: 280 on `wizard-spells`, 104 on `cleric-spells`, 53 on `druid-spells`, and 196 on the class and path pages. The work splits by **where a spell's block lives**: its page and the level heading it sits under. A spell's level differs by list (Hold Person is wizard 3, cleric 2, bard 2), so a spell has no one level; its block's heading is what places it.

**Why not one writer per page and level** (considered, 2026-11-09): that's 98 writers, 48 of them with three spells or fewer (17 with one), plus a reviewer each. Each agent's setup (the summary style from 112, the data format, the project rules) would outweigh its work, 98 voices would drift apart, and the finish would merge 98 branches. Seven balanced writers keep most of the speed.

Every agent works in its own worktree with its own `target/` and database, as `CLAUDE.md` says.

1. **Foundation** (one agent):
   - the spell types in `rules` (`GameData`, `GameData::check`);
   - one new migration, `<timestamp>_121-spells.sql`, for the spell and spell-list tables, plus the seed and the load;
   - the facts extracted from the saved pages, with empty summaries, in **one data file per page and level heading** a block lives under (`libs/game-data/spells/wizard/level-3.ron`, `spells/lich/level-5.ron`, ...: 98 files). No two writers then share a file, whatever the grouping, so the groups below can be rebalanced without moving data. The lists, saying which list holds which spell at which level, go in `spells/lists.ron`, which only the foundation writes. Class lists with no blocks of their own (bloodrager, hunter, magus, shaman, warpriest) appear only there;
   - real-data tests for the facts (Hold Monster);
   - merged into `main` before stage 2 starts.
2. **In parallel**, each from that merge, eight agents:
   - **Writer 1:** `spells/wizard/level-0.ron` to `level-3.ron` (107 spells).
   - **Writer 2:** `spells/wizard/level-4.ron` to `level-6.ron` (100).
   - **Writer 3:** `spells/wizard/level-7.ron` to `level-9.ron` (73).
   - **Writer 4:** `spells/cleric/` (104).
   - **Writer 5:** `spells/druid/` (53).
   - **Writer 6:** the Angel, Azata, Lich and Demon spellbooks (32 + 26 + 25 + 20 = 103).
   - **Writer 7:** the Aeon, Trickster and Gold Dragon spellbooks (21 + 17 + 11) and the class pages: bard 11, inquisitor 9, paladin 9, arcanist 7, ranger 4, alchemist 2, witch 2 (93 in all).
   - **Agent D:** the MCP side, a `spell` kind in `search` and `describe`, in `apps/mcp` only, against the foundation's types.

   Writers edit only the `summary` fields of their own files, so they never change the schema or touch each other's files; each may reseed its own database. As in [112](112-effect-summaries.md), each writer's summaries then go to an independent reviewer, who gets the summary and the guide's section, not the writer's reasoning.
3. **Finish** (the main session): merge the eight branches, reseed, run `scripts/check.sh --schema` and `scripts/proof.sh`, change the honesty boundary, and add the map's Decisions line.

Other schema tickets (119, 120) can run alongside this one: each adds its own migration.

## Done when

`search {kinds: ["spell"]}` finds a spell by name, school or subschool, and `describe` gives its facts and the lists and levels it's on, with a real-data test (Hold Monster: enchantment, compulsion, Will negates, on the wizard and sorcerer list); `scripts/check.sh` passes.

Whatever the option, the map is updated with it:
- its Out of scope entry on spells is rewritten as in question 5;
- a Decisions line is added for this ticket;
- if B or C, the MCP instructions' honesty boundary stops listing spells and spell availability as the agent's own knowledge (only what the catalogue doesn't hold stays there), and the Decisions line for [104](104-mcp-server-plan.md) is marked superseded in that part.

## Choices settled

1. **Option B** (user, 2026-11-09): a spell catalogue as reference data, with no spell picks and no change to a Build or its link.
2. **The map's Out of scope entry** (question 5) was rewritten the same day: the catalogue is in scope, and spells known or prepared stay out. The Decisions line and the honesty boundary change wait until B is built.
3. **Questions 2–4 take their recommendations** (user, 2026-11-09):
   - **Source:** the three spell pages for the spell blocks, and the class, archetype and path sections for which list holds a spell at which level. A spell named in a list with no block anywhere is a data issue the extraction reports. Descriptions become summaries in our own words, never the guide's text.
   - **Mythic spellbooks:** catalogued as lists of their own (`azata`, `lich`, ...); the rules for merging them into a class spellbook are left for option C.
   - **Spells per day:** not part of this ticket. It gets a ticket of its own if the user wants it; the tables are on the class pages, so it needs no scope decision.
4. **Stage 1's modelling** (recommended answers, taken while the user slept, 2026-11-09; no question in the ticket covered them, so each is the smallest choice consistent with the map's Notes):
   - **A spell's school** is `Option<SpellSchool>`: `None` is the guide's Universalist (Dismiss Spell, the only one). No Spell Focus value was added for it.
   - **Facts as printed:** `target`, `duration`, `saving_throw` and `components` are the block's text (the saving throws run to 26 phrasings, "Will half (if used to damage)", "?"); `spell_resistance` is a bool; `descriptors` (30 values, the subschool among them) and `casting_time` (5) are enums. A block with no Duration row has `duration: None`.
   - **Lists are keyed by their owner** (`wizard`, `arcanist/magic-deceiver`, `azata`), checked by `GameData::check` and the schema, and say which other classes and archetypes use them (`used_by`), as their pages say: the sorcerer, the arcanist and the Eldritch Scoundrel the wizard's, the oracle the cleric's, the skald the bard's, the Nature Mage the druid's, the Unlettered Arcanist the witch's.
   - **Angel and Lich: both lists kept.** `angel` and `lich` are the path's own spellbook (the "(Unmerged)" tables, levels 1–10); `angel/merged` and `lich/merged` are the spells merging adds to the class spellbook, levels 3–10 (the pages: "If you choose to merge spellbooks, the following spells are added to your list of known spells"). The blocks sit under the merged tables, so `spells/angel/` and `spells/lich/` are by their levels.
   - **Footnotes are list facts:** an entry's `only` names the archetype ("Flame Dancer only"; the druid's Aspect of the Stag, through the Drovier's Communal Aspect) or the Gold Dragon's story branch (`Story(Uncorrupted)`, `Story(VowOfDahak)`).
   - **A block sits on its page's list:** three blocks sit under a "Level <n>" heading whose table omits them, and are added to that list at that level (Rage, wizard 3; Winds of Vengeance, wizard 9; Thirsting Entanglement, druid 4). Where a table's name and a block disagree, a misspelling takes the correct spelling and a renamed spell takes the block it links to; where a table's link names another spell than its text, the text wins. All are listed in `libs/game-data/README.md`.
   - **The browser gets no spells:** `GameData::spells` and `spell_lists` are `#[serde(skip_serializing)]`. Measured on the real data: the game data JSON is 1,963,829 B raw, 161,587 B brotli (quality 11), 225,852 B gzip without them, and 2,299,217 / 184,760 / 258,159 B with them (+17 % raw, +14 % brotli), before any summary. The browser has no spell UI and its rules never read spells (map Notes: the browser loads the game data to run the rules; mobile first), and the MCP server reads the data files, so nothing needs them in the JSON. The MCP server's `site_file` hashes the same serialisation, so it still names the file the site serves.
   - **Not in stage 1:** the shaman spirits' bonus spells, the oracle's bonus and additional spells and the Beast Tamer's bonus spells, which aren't Spell / School tables. A later ticket can add them as lists of the spirit, mystery or archetype.
5. **Stage 2, Agent D: the MCP side** (recommended answers, taken while the user slept, 2026-11-09; where the ticket gave none, the smallest choice consistent with the map's Notes):
   - **Two kinds:** `spell` and `spell_list` in `search` and `describe` (the list kind recommended, and small). Both sort after the existing kinds, so no earlier search order changes.
   - **A spell's `category` is its school** ("Universalist" for Dismiss Spell), and its hit carries its `descriptors`. The `category` filter (ticket 116's design: one name or a list, case and spacing ignored) also matches a spell's descriptors, so `category: "Compulsion"` or `"Mind-Affecting"` lists those spells; the no-match note's near categories include descriptors. `compact` drops the descriptors; a spell with none leaves them out. Summary matching (ticket 123) needed no change.
   - **A spell list's `category` is its owner's kind** ("Class list", "Archetype list", "Mythic spellbook"), its `parent` the owner's key (`lich/merged`: `lich`).
   - **`describe {kind: spell}`:** `facts` (school, descriptors, target, duration, saving throw, spell resistance, casting time, material component, as printed; an empty duration or component left out) and `lists`, one entry per list and level (the bard's Fear twice), each with the list's owner, `used_by` and any `only` footnote as "Flame Dancer only" / "Uncorrupted only". The hit's descriptors are dropped there, since `facts` hold them.
   - **`describe {kind: spell_list}`:** `owner`, `used_by` (each `{kind, key, name}`, `kind` a describable kind) and `spells` by level, each spell with its school, descriptors and footnote but no summary, to keep the wizard's 290 entries small; `describe {kind: spell}` gives the rest.
   - **A class's, archetype's or path's `describe` names its `spell_lists`:** the lists it owns or is in `used_by` of (the sorcerer: the wizard's; the Angel: `angel`, `angel/merged`). An archetype gets only the lists naming it, its class's being on the class; no lists, no field.
   - **Spell hits have no prerequisite** (reference data, nothing to take), so `prerequisite` is left out rather than `null`.
   - **The honesty boundary:** a new bullet makes the catalogue's facts (school, descriptors, target, duration, save, spell resistance, casting time, component, every list and level, `used_by`, footnotes) fact, and the summary bullet says spells carry summaries too. "Spells and spell availability" left the agent's-knowledge list, which now names what the catalogue lacks: spells known or prepared, spells per day, the bonus spells stage 1 left out, merging a Mythic spellbook, what a duration or damage comes to at a caster level. The caster line's "Which spells exist, and their schools and subschools, stay your own knowledge" now points at the catalogue. The workflow's step 1 mentions spells. The other instructions are unchanged.

## Progress log

- 2026-11-09, stage 1 (the foundation), worktree `t121-foundation`:
  - Read the saved pages with a one-off parser kept outside the repo (scratchpad `t121/`: `parse.py`, `collect.py`, `extract.py`). 633 spell blocks ("Name (School)" and a table of facts) and 182 list tables, as the ticket said: wizard-spells 280, cleric-spells 104, druid-spells 53, and 196 on the class and path pages. The table rows link to the blocks (`<a href=…#hold-monster-enchantment>`), which resolved the renamed entries.
  - The brief's "witch's two tables headed Level 5" is, in the saved page, a level 6 table headed by an `<h5>` "Level 6" rather than an `<h4>`; the extractor reads both, and the README records it.
  - Tests first (`tdd`): `libs/rules/tests/engine/spells.rs` (7 tests: duplicate keys, unknown spells and casters, levels to 9 and 10, a spell on no list, a list keyed outside its owner, `only` outside the list) failed to compile, then passed with `libs/rules/src/spells.rs` (`Spell`, `SpellDescriptor`, `CastingTime`, `SpellList`, `SpellCaster`, `SpellListEntry`, `SpellListOnly`), the two keys and the check.
  - Migration `20261109013122_121-spells.sql`: `spell`, `spell_descriptor`, `spell_list`, `spell_list_user`, `spell_list_entry`, STRICT, with foreign keys and the owner and key-under-owner CHECKs. Seed and load in `libs/seeds/src`, ten new `.sqlx` queries; `files.rs` reads `spells/lists.ron` and every `spells/<page>/level-<n>.ron` (anything else under `spells/` is an error). The fixture gained three spells and three lists.
  - Data: 632 spells in 98 files, 24 lists, 2,494 entries; every summary is `""` for stage 2. `seed --check-only` passes. The real data round-trips through a seeded database (a one-off test, not kept).
  - Real-data tests in `libs/seeds/tests/game_data/spells.rs` (10): Hold Monster (Enchantment, Mind-Affecting and Compulsion, Will negates, spell resistance, wizard 5 and used by the sorcerer; bard and inquisitor 4, witch 5, not the cleric's), the Max DC run's other spells, blocks per page, entries and levels per list, each spell's file against its list and level, `used_by`, footnotes, the Universalist spell and every hand-fix. Also `tests/files.rs` (the fixture's spells and lists; misplaced spell files) and `tests/database.rs` (the spell tables' constraints).

  - For stage 2: the writers' files hold the counts the Plan gives (wizard 0–3: 107, 4–6: 100, 7–9: 73; cleric 104; druid 53; Angel, Azata, Lich, Demon 103), except the bard's 10 (Heaven's Wing Strike is one spell, linked to its wizard-page block), so writer 7 has 92. A writer edits only `summary`. The MCP agent finds the types in `libs/rules/src/spells.rs` and the data in `GameData::spells` and `GameData::spell_lists` (the server reads the files, so the browser JSON's lack of them doesn't matter to it).
  - Checks: `scripts/check.sh --schema`: ALL-OK (fmt, clippy, wasm-clippy, the workspace tests, rules-wasm, seed-check, sqlx-migrate, sqlx-prepare). Behaviour proof, `scripts/proof.sh both` (the rules types and the data together; `data` mode reads HEAD's data, which has no `spells/`, so the files reader takes a missing folder as no spells): 2,162 Builds, none only on one side, none differing. The proof ran with `CARGO_TARGET_DIR` in this agent's scratchpad rather than the shared default, to keep it from other worktrees' builds.
  - The dev database was rebuilt with `CLAUDE.md`'s four steps; the main checkout's needs the same after the merge (new migration).
- 2026-11-09, stage 2, Agent D (the MCP side), worktree `t121-mcp-spells`, `apps/mcp` only: the server reads the data files (`seeds::check_game_data_files`), so the spells reach it although the browser JSON skips them. Tests first: `apps/mcp/tests/server/spells.rs` (8 tests: Hold Monster by name and key with its school, descriptors, summary and link; under `category: "Compulsion"`, "Enchantment", a list of categories and "Universalist"; by its summary's words; described with its facts and wizard 5, bard 4, inquisitor 4, witch 5, not the cleric's, the sorcerer among the wizard list's users; Fireball's "Flame Dancer only" on the bard's; the wizard list by level 0–9 and the lich's to 10, found by `search`; `spell_lists` on the sorcerer, bard, Magic Deceiver and Angel, none on the fighter; the instructions). Then `Kind::Spell`/`SpellList`, `Hit::descriptors`, the `Item` fields (`facts`, `lists`, `owner`, `used_by`, `spells`, `spell_lists`) and their shapes, a new `src/spells.rs`, the index and the `category` filter in `search.rs`, the instructions and the `search` and `describe` descriptions. Two older tests followed the data: `search::kinds_filter_the_matches` needs `limit: 100` to reach a feat past the Summon Monster spells, and the end-to-end test now finds Animal Growth by name (only "mass animal growth" still matches no name). `cargo fmt --all --check` and `scripts/check.sh`: ALL-OK (fmt, clippy, wasm-clippy, test, rules-wasm, seed-check). No migration, no query change.
