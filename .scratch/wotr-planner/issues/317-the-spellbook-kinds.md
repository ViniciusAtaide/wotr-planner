# The spellbook kinds

Type: build (model gap)
Status: open
Blocked by: 293 (the importer's stage that finds it)
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Problem

Gap G6 of the importer's design ([research/importer-design-2026-11-16.md](../research/importer-design-2026-11-16.md), §2.8): something the game holds that the planner's model has no form for, so the importer leaves it and the game check keeps listing it under this ticket. `AddSpellbookLevel`, spell lists, `BlueprintFeatureReplaceSpellbook` options, `PrerequisiteAnySpellsInSpellbook`, Fey Magic's spell, and `spell_levels` from the game (290's area).

## Change

Research the game's form first: the blueprints and the code that read it, cited. Then model it and let the importer write it.

## Done when

This ticket's lines in `tools/game-check/known.txt` are gone, the game check exits 0, and the scoped checks pass.
