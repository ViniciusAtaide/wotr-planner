# Spells as game data

Type: design
Status: open (option B decided, not built)
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
