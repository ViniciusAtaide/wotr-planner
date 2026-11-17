# Second Mystery and Second Bloodrager Bloodline admit a Wind Whisperer and maybe a Dragon Disciple

Type: build (data)
Status: open
Blocked by: —
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Problem

From the 308 queue's allowlist audit (2026-11-17):

- **Wind Whisperer.** Second Mystery (308d) accepts `oracle/wind-whisperer/mystery`, which the planner maps to OracleMysterySelection 5531b975…. But WIndWhispererArchetype 77769e43… removes that selection at level 1 and adds OracleWindMysteryFeature, so the game refuses a Wind Whisperer Second Mystery; the planner admits it.
- **Dragon Disciple (unconfirmed).** Second Bloodrager Bloodline (308d) accepts `bloodrager/bloodline`, which `prestige/dragon-disciple.ron` grants; DragonDiscipleClass only reads BloodragerBloodlineSelection through ContextConditionHasFact, so it probably doesn't grant it in the game.

## Change

Check both against the game; where the planner admits what the game refuses, fix the archetype's or class's rows (the game's), with real-data tests.

## Done when

Each case cited and tested; the scoped checks pass.
