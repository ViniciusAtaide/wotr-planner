# QA: make two full builds from a build guide

Type: task
Status: open
Blocked by: [86](86-deities.md)
Map: [WotR Build Planner](../map.md)

## Question

The user asked for a QA exercise after the deity work (2026-11-06): a QA agent makes complete builds in the real app, as a player would, from the build write-ups the user saved in `.scratch/builds`, and reports whether each could be finished; if not, what stopped it. Problems become bugfix tickets.

The file holds two builds:
- **Jack-in-the-Green:** Halfling, Warrior of the Linnorm Kings background, Sylvan Sorcerer 20, CHA 19 point buy, mythic Hero then Azata.
- **Enforcer:** Half-Elf, Andoren Diplomat background, Arcane Enforcer 15 / Sylvan Trickster 4 / Mutation Warrior 1, Strength bonus, mythic Hero then Azata.

**Out of scope** (the map's "Out of scope", and the user, 2026-11-06):
- spells: the "suggested spells" tables, and spell picks inside level-ups such as "Bloodline Spell - Entangle (+ Mage Armor)";
- the animal companion's own build (the Elk's level-ups). A choice *of* companion, such as "Animal Companion - Elk", is in scope if the app offers it.

## Build

- **In the real app:** a release build served on its own port with its own target and site dirs (not the dev server on 3900), driven with the browser tools as a player would: the Build start, then each level and mythic rank in the write-up's order. Use the app's pickers; don't type link fragments.
- **For every line of each write-up,** record one of:
  - done as written;
  - done differently (the app names it otherwise, or a choice the write-up implies has to be made), and how;
  - not possible: what stopped it (missing option, wrongly locked with its reason, wrong level, missing slot, crash or console error), with the level, the slot and the app's exact message.
  
  Features granted automatically (Woodland Stride, Studied Target +2, sneak dice) only need checking on the sheet.
- **Judging a block:** a write-up can be wrong too. Before calling something a bug, check the saved guide pages in `guide-cache/` (never fetch GameFAQs or fextralife) and the game data. A real bug is the app disagreeing with the guide or with the game's rules as the data records them.
- **Each real problem gets a bugfix ticket,** numbered from 89, in the style of the other tickets: the repro (the Build's link at that point, the slot, the message), what the guide says, and the expected result. Group problems that share one cause into one ticket.
- **When the report is written,** delete `.scratch/builds` (user, 2026-11-06): it's staged in git but was never committed, so also unstage it (`git rm --cached`) before deleting.

**Done when:**
- Both builds were attempted to their last level and mythic rank (or as far as the app allows), and the Answer has, per build: finished or not, the final `#b=` link, and a table of every deviation and blocker.
- Every real problem has a bugfix ticket, linked from the Answer.
- `.scratch/builds` is gone.
