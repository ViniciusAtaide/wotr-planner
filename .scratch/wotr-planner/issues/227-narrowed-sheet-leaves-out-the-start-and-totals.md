# A narrowed `sheet` leaves out the start, the totals and the fixed text

Type: feature
Status: open
Blocked by: none
Model: Sonnet.
Map: [WotR Build Planner](../map.md)

## Problem

From the second ranking of the MCP server's token costs (2026-11-14, measured on the live server). [210](210-sheet-says-each-level-once.md) made `sheet` take `levels` and `ranks`, but a narrowed call still sends everything that isn't a level row. On a fighter 8 with `levels: [5]` the answer is 4,076 characters and the level row is 877 of them; the rest is the Build start (790), `totals` (899), `totals_leave_out` (485, fixed text), `class_skills` (179), `link` (187), `build` (138) and a few small fields. On the filled Trickster 20 the same call is 5,477 with the row 928: start 857, totals 2,118. An agent that asks for level 9 to check one fact pays 78 to 83% for what it didn't ask for, and it asks again at each level it checks.

## Build

- With `levels` or `ranks` given, `sheet` answers with the rows asked for, `build`, `notes` and what a row needs to be read (`level_cap`, `caster_level`, `spell_levels` if the research finds a row can't be read without them). The Build start, `totals` with its `totals_leave_out`, `class_skills`, `point_buy_spent` and `point_buy_budget` come only on request: new flags `start` and `totals` (the research may fold them into one `include` list), as `todo: true` works today.
- A full sheet (no `levels`, no `ranks`) is unchanged.
- A row's own lines keep saying what they leave out (`attack_leaves_out`, `caster_leaves_out`): they go with the numbers they qualify. `totals_leave_out` goes with `totals`, so no number is sent without its leave-out text.
- The tool description and the `wotr-build` skill say a narrowed call has no start or totals unless asked.
- [228](228-link-is-said-where-it-is-new.md) cuts `link` and the echoed `build` on the same call; the two tickets edit `sheet`'s result type, so they run one after the other.
- `apps/mcp/tests/server/responses.rs` gains "sheet, fighter 8, levels [5]" (4,076 characters today).

## Measured and expected

Fighter 8, `levels: [5]`: 4,076 to about 1,700. Filled Trickster 20, `levels: [9]`: 5,477 to about 1,800. Three narrowed calls in a session save about 7,000 characters.

## Done when

`levels: [5]` on the test fighter 8 is about 1,700 characters (budget about a tenth above what is built) with the same row; `start: true` and `totals: true` bring back today's fields, byte for byte; a full sheet's test output is unchanged; the schema test passes; `scripts/check.sh` passes.
