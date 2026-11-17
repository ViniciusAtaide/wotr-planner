# Two checks the 308 queue showed missing: the Captor's prerequisites, and the MCP tests for a data change

Type: check
Status: open
Blocked by: —
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

1. **The Captor.** In `full t61 urban hunter 12 captor`, Trip and Greater Trip are held through the Captor's slot with Intelligence 10 and no Combat Expertise (308c's review). Whether the game's Captor selection ignores prerequisites wasn't checked.
2. **The MCP tests.** `apps/mcp`'s real-data tests broke in the 308 queue (ticket 323) and no ticket ran them: CLAUDE.md's "Checks" gives a `libs/game-data/` change `seeds`, `corpus` and `check-data`, not `cargo test -p mcp`. `libs/rules/src/engine/labels.rs` ~978 also still shows "one of 49 hexes" as an example of a shortened text, which the server no longer gives (323).

## Change

1. Read the Captor's selection blueprint (`IgnorePrerequisites`, its feats' prerequisites) and fix the data or the Build if the game refuses those picks.
2. Propose to the user adding `cargo test -p mcp` to the data row of CLAUDE.md's "Checks" (its cost measured); update the `labels.rs` example.

## Done when

The Captor's case is cited either way; the user has answered the proposal.
