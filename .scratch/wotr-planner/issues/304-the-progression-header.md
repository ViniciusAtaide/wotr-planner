# The progression header

Type: build (data form and engine)
Status: open
Blocked by: 303
Model: Opus to build: every class's option levels change source.
Map: [WotR Build Planner](../map.md)

## Problem

Split from [298](298-selections-as-the-game-has-them.md) (user, 2026-11-16, "B: forms now, splits in 293"): one of the staged component plan's five form steps (user, 2026-11-15; [295](295-the-component-vocabulary.md)). The design is in [research/component-vocabulary-2026-11-15.md](../research/component-vocabulary-2026-11-15.md), §3 and §4. The per-class selection splits are [293](293-the-importer.md)'s, not this ticket's.

**What the planner holds today:** `option_levels` and its kin, the planner's way of saying which class levels a selection's options count.

## Change

- **The progression header fields replace them,** as the game's `BlueprintProgression` holds them (its classes, archetypes and level sources; cite the blueprint fields and the code that reads them).
- **A known limit:** `option_levels` can't move onto the options while the domains are shared ([294](294-the-domain-split-with-a-slot-aware-link-migration.md)). The domains keep the field until 294, and the Answer says so.

## Done when

- The form proof (`record` on the base in a temporary worktree, `compare` on the tip) shows 0 differences, or each difference is explained in the Answer and comes from the game, cited.
- Every corpus and bench link opens with the same picks (298's link test).
- The self-checks' short runs, the gold builds and every real-data test are unchanged.
- The scoped checks pass, and the game check exits 0.

