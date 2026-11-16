# Repeat from the header

Type: build (data form and engine)
Status: open
Blocked by: 302
Model: Opus to build: every repeat rule changes source.
Map: [WotR Build Planner](../map.md)

## Problem

Split from [298](298-selections-as-the-game-has-them.md) (user, 2026-11-16, "B: forms now, splits in 293"): one of the staged component plan's five form steps (user, 2026-11-15; [295](295-the-component-vocabulary.md)). The design is in [research/component-vocabulary-2026-11-15.md](../research/component-vocabulary-2026-11-15.md), §3 and §4. The per-class selection splits are [293](293-the-importer.md)'s, not this ticket's.

**What the planner holds today:** `Repeat`, a hand-written rule for how often an option may be taken.

## Change

- **`Repeat` is derived from the blueprint header the data holds:** `Ranks`, whether it is a selection or parametrised, and `PrerequisiteCondition`, as the game's code decides a feature's repeat (cite the method).
- **The field goes.** Where the derived rule and today's differ, the Answer lists each with the game's citation, and a real-data test holds the game's value.

## Done when

- The form proof (`record` on the base in a temporary worktree, `compare` on the tip) shows 0 differences, or each difference is explained in the Answer and comes from the game, cited.
- Every corpus and bench link opens with the same picks (298's link test).
- The self-checks' short runs, the gold builds and every real-data test are unchanged.
- The scoped checks pass, and the game check exits 0.

