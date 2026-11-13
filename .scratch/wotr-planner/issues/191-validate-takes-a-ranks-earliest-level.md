# `validate` refuses a Mythic rank at its earliest level

Type: bugfix
Status: open
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

Found by a QA session through the MCP server (2026-11-13), still so on `main` at `ce9cc3b`. A sorcerer 14 declared with `"mythic.4.level": 11, "mythic.5.level": 14` (each rank's table level) is `invalid`, each with `not_allowed`: "MR 4 comes at level 11 at the earliest (the rank table's level): `mythic.4.level` takes a later level, up to the Build's last. To take the rank at level 11, leave `mythic.4.level` out." (`apps/mcp/src/validate/wording.rs`).

But `options` on `mythic.4.level` lists "Level 11 (the earliest)" with status ok, `choose` takes 11 as a no-op (as it takes any choice's current value), and the tool's description says `validate` takes each value "as `choose` takes them". An agent that copies `options`' values into its map gets a refusal for a value it was offered.

## Build

- `validate` takes `mythic.R.level` equal to the level the rank comes at as a no-op, as `choose` does: the table's level, or the rank below's later level when that holds it (`options` then lists "Level 15 (the earliest)" for MR 4 with MR 3 at 15).
- A level below that stays refused, with the reason it has now.

**Not part of this ticket:** `validate` accepting ranks declared out of order (`{3: 15, 4: 13}`) is ticket [171](171-mythic-picks-after-the-levels.md)'s dormant level, by design.

## Done when

The map above is `valid` and gives the same code as leaving both out; a test pins it beside 171's; `scripts/check.sh` passes.
