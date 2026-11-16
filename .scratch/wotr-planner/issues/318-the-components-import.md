# The components import

Type: build (model gap)
Status: open
Blocked by: 293 (the importer's stage that finds it)
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Problem

Gap G7 of the importer's design ([research/importer-design-2026-11-16.md](../research/importer-design-2026-11-16.md), §2.8): something the game holds that the planner's model has no form for, so the importer leaves it and the game check keeps listing it under this ticket. Every reachable entry's components (design §5: 3,458 entries, 5,048 components), the game's descriptors, and the check comparing components to components. The user decided every reachable feature becomes an entry (2026-11-15) and that 293 makes only the carriers first, with the rest here (2026-11-16).

## Change

Research the game's form first: the blueprints and the code that read it, cited. Then model it and let the importer write it.

## Done when

This ticket's lines in `tools/game-check/known.txt` are gone, the game check exits 0, and the scoped checks pass.
