# The Wolverine shifter's rage powers at 8

Type: build (data)
Status: open
Blocked by: —
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

ShifterWolverineProgression 4fc5f64d… grants RagePowerSelection twice at level 8 (its LevelEntries; found by the 308 queue's allowlist audit, 2026-11-17). The planner's wolverine-aspect option has no rows, so a Wolverine shifter has no rage power slots and is refused Extra Rage Power, before and after 308c.

## Change

Give the aspect its rows from the game (`game-check show ShifterWolverineProgression`), with a real-data test citing the blueprint.

## Done when

A Wolverine shifter 8 has two rage power slots and may take Extra Rage Power; the scoped checks of CLAUDE.md pass.
