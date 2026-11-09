# Planning ahead: what a class split reaches and gives up

Type: design
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

In the Demon Greatsword run (2026-11-08, see [112](112-effect-summaries.md)), the agent picked Two-Handed Fighter 5 / Barbarian 15. Only while filling level 19 did it find what that split costs:
- Lethal Accuracy needs Barbarian 16;
- Greater Weapon Focus needs Fighter 8;
- Barbarian 20's Mighty Rage is gone.

The planner already knew: `options` with `only_open: false` says "Needs Barbarian level 16 (have 14)". But the agent only asked once the levels existed, after the split was chosen.

## Options

- **A. Instructions only:** tell agents to branch before committing to a split, for example Barbarian 20 against Fighter 5 / Barbarian 15, then call `compare`. It already lists the features each Build gains that the other doesn't. No code.
- **B. A `reach` tool:** `{build, classes: [{class, levels}], kinds?}` applies the planned levels to a scratch copy. It lists the feats, selection options and class features that are open at the last level, and those that stay closed, with their reasons. The Build itself doesn't change.

## Open questions (with recommendations)

1. **Which option?** *Recommend:* A now, as part of [117](117-mcp-ergonomics.md)'s instructions change. Build B only if agents still pick a split blind.
2. **Should B also list what a split forfeits against a single-class Build?** *Recommend:* no. That is `compare` on two branches, which exists.
