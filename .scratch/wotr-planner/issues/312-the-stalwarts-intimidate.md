# The Stalwart's Intimidate: the game's sub-skills in class skills

Type: build (model gap)
Status: open
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

Gap G4 of the importer's design ([research/importer-design-2026-11-16.md](../research/importer-design-2026-11-16.md), §2.8). The game names sub-skills (Intimidate among them) in class skills and bonuses, and the planner's skill list has no form for them. The Stalwart's class-skill finding stays in `tools/game-check/known.txt`, under this ticket, after 293l.

## Change

Model the game's sub-skills where class skills and bonuses name them, and let the importer write them. The research to do first: which skills the game splits, how its code counts them (cite file and method), and which entries name them.

## Done when

The Stalwart's finding is gone from `known.txt`, the game check exits 0, and the scoped checks pass.
