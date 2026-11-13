# The Invulnerable Rager's Damage Reduction carries the barbarian's summary

Type: bugfix
Status: open
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

Found by a QA session through the MCP server (2026-11-13), still so on `main` at `ce9cc3b`. The Invulnerable Rager's rows grant `barbarian/damage-reduction` at 2, 4, 6, …, 20, as its table in the guide does ("Damage Reduction" in every even row; the class overview's "10/- DR"). But the feature is the barbarian's own, so the sheet names it "Damage Reduction (Barbarian)" and its summary says "DR 1/- from 7th level, rising by 1 at 10th, 13th, 16th and 19th" (`libs/game-data/classes/barbarian.ron`), wrong for every grant the archetype gets.

## Build

- A feature of the archetype's own, `barbarian/invulnerable-rager/damage-reduction`, named "Damage Reduction (Invulnerable Rager)", with a summary from the guide's table and overview (DR 1/- at 2nd, +1 at each even level, 10/- at 20th), on the archetype's rows in place of the barbarian's.
- First check what reads `barbarian/damage-reduction` (prerequisites such as Increased Damage Reduction, `bonuses.ron`), and keep each reader working for the Invulnerable Rager as the guide intends.

## Done when

The Invulnerable Rager's sheet shows its own Damage Reduction at the even levels with a true summary; nothing that read the barbarian's feature changes for the archetype unless the guide says so; a real-data test pins it; `scripts/check.sh` passes.
