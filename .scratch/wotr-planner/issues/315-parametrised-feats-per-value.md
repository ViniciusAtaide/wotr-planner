# Parametrised feats the game keeps one per value

Type: build (model gap)
Status: open
Blocked by: 293 (the importer's stage that finds it)
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Problem

Gap G3 of the importer's design ([research/importer-design-2026-11-16.md](../research/importer-design-2026-11-16.md), §2.8): something the game holds that the planner's model has no form for, so the importer leaves it and the game check keeps listing it under this ticket. Skill Focus as `SkillFocusSelection` c9629ef9eebb88b479b2fbc5e836656a's 11 features, Armor Focus as `ArmorFocusSelection` 76d4885a395976547a13c5d6bf95b482's, and the mythic Armor Focus variants; `FeatIfValue`, `FeatForSameValue`; and a link alias that reads a key and a value, which 298's migration can't do. Supersedes the model part of 255.

## Change

Research the game's form first: the blueprints and the code that read it, cited. Then model it and let the importer write it.

## Done when

This ticket's lines in `tools/game-check/known.txt` are gone, the game check exits 0, and the scoped checks pass.
