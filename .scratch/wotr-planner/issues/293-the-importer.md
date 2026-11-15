# The importer: the game data's mechanical fields from the blueprints

Type: design, then build
Status: open
Blocked by: 244 (it writes the per-blueprint files); the engine-rule tickets 247–254 and 256 (the importer writes data in the shapes they settle)
Model: Opus to build: one bug touches thousands of entries, so it is built and reviewed with care.
Map: [WotR Build Planner](../map.md)

## Problem

Tickets 247–291 filed the 3,908 differences the game check found (238) against the data. 31 of them (261–291) hand-edit the data, one class file at a time, about 3,600 findings.
- **The game check already does the hard part.** It reads every prerequisite, progression row, option list, class table and item bonus from the blueprints, and maps each onto the planner's model.
- **Hand-copying them is slow and error-prone.** It's 30 Opus builders with review rounds, retyping values the computer already reads.
- **It would be done twice.** 244 rewrites every data file into one file per blueprint, so a fix made today to Sneak Attack's 13 copies would be redone there.

The user, 2026-11-15, agreed to replace the hand edits with an importer. The order is: the engine rules first, then 244, then this ticket. The modelling gaps it can't express become their own tickets.

234 allows it:
- code of our own that reads the community's `.binz` (item 4);
- facts committed in our own form, with blueprint names and GUIDs as citations;
- never the game's text strings, dump or code.

## Change

- **A writer beside the game check** in `tools/game-check`, or a sibling crate sharing its reader and its mapping. For each entry that names a blueprint, it writes the mechanical fields the check compares:
  - prerequisites;
  - class tables: BAB, saves, skill points, class skills;
  - progression rows and archetype additions and removals;
  - selection option lists;
  - mythic ranks;
  - item bonuses.

  It writes them into the entry's per-blueprint file (244), and reads them from the game as the check does, through the same normalisation.
- **What it never touches:** the planner's keys, names, summaries, guide links, `interactions`, the data notes, and the planner-made entries (`planner/`). It never writes the game's text strings.
- **What it can't express** (a component the planner's model has no form for) is left as it is. It is listed per kind, and the list becomes the modelling-gap tickets. The game check keeps reporting those findings, under their tickets.
- **Idempotent and deterministic.** It is a run by hand, like the check. A second run writes nothing, and its output is the same for the same dump and data.
- **After a run, the game check reports only the gaps.** `known.txt` loses each section of 261–291 whose findings are gone.
- **The tests waiting on 261–291** come off `#[ignore]` where the data now holds the game's value. Each one still waiting names its gap ticket.
- **The user's facts.** The seven decided on 2026-11-15, 262's class skills and 276's Improved Uncanny Dodge are applied by the run like any value. The importer never overwrites another of the user's facts: those differences go to the user.

## To decide (recommendations; the user's "build them all, in order" applies)

1. **Where it lives.** Recommended: a `--write` mode of `game-check`, sharing its reader and mapping, so the two can never disagree on what a field means.
2. **What a run may change.** Recommended: only the fields listed above, and only on entries with a blueprint. Any other change is a bug the run refuses.
3. **Review.** Recommended:
   - a sampled hand check of 100 entries across kinds against the blueprints;
   - the corpus self-checks' long runs;
   - the gold builds, with every change to a gold build's gains shown to the user.

## Done when

- One run brings the data to the game's values in the fields above. The game check's remaining findings are exactly the listed gaps, each under a ticket. Two runs: the second writes nothing.
- The tests that waited on 261–291 pass, or name a gap ticket.
- The self-checks' long runs and the gold builds pass, or their changes are shown to the user.
- The scoped checks and the game check pass.
