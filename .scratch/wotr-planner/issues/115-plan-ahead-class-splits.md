# Planning ahead: what a class split reaches and gives up

Type: design
Status: resolved (option A; option B, a `reach` tool, not built)
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

## Settled

The user approved the recommendations (2026-11-08): **option A**, as part of [117](117-mcp-ergonomics.md)'s "Before you plan" paragraph. **Option B, the `reach` tool, is not built**; it waits until agents still pick a split blind with A in place. Question 2 is moot without B.

## Progress log

- 2026-11-08: Built on 117 (e7f1e00), whose paragraph left this bullet for this ticket. Red first: `tests/server/listing.rs` gains a test that the "Before you plan" paragraph names a class split, the Fighter 5 / Barbarian 15 example, branching, "`compare` the branches" and `only_open: false`; it failed. Then added the bullet to `INSTRUCTIONS` (`apps/mcp/src/server.rs`), and it passed. No other code.
- `SQLX_OFFLINE=true scripts/check.sh`: ALL-OK.

## Answer

Done 2026-11-08 with option A: the server's instructions tell agents, before they plan, to branch candidate class splits from one code, add their levels and `compare` the branches, and to read a closed option's needs with `options {only_open: false}`. No `reach` tool (option B). 83 server tests pass (1 new).

The bullet, for the user's review (the whole paragraph is in 117's Answer):

> - A class split decides what the last levels can take: Fighter 5 / Barbarian 15 gives up Barbarian levels 16 to 20 and whatever needs them. Before you commit to a class split, branch the candidates from one code (Barbarian 20, and Fighter 5 / Barbarian 15), add their levels, and `compare` the branches: it lists the feats and features each gains that the other doesn't. `options` with `only_open: false` says what a closed option still needs ("Needs Barbarian level 16 (have 14)").
