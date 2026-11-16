# The planner's choices that are a plain feature in the game

Type: build (model gap)
Status: open
Blocked by: 293 (the importer's stage that finds it)
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Problem

Gap G5 of the importer's design ([research/importer-design-2026-11-16.md](../research/importer-design-2026-11-16.md), §2.8): something the game holds that the planner's model has no form for, so the importer leaves it and the game check keeps listing it under this ticket. The 11 "a choice the game doesn't offer" findings (Empyreal, Sage and Sylvan sorcerer copies among them): each becomes a `planner/` entry with its `why`, or goes.

## Change

Research the game's form first: the blueprints and the code that read it, cited. Then model it and let the importer write it.

## Done when

This ticket's lines in `tools/game-check/known.txt` are gone, the game check exits 0, and the scoped checks pass.
