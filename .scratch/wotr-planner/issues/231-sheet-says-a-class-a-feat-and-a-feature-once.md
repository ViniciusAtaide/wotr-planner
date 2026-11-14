# `sheet` says a class, a feat and a feature once

Type: feature
Status: open
Blocked by: none (edits `apps/mcp/src/{sheet,shapes}.rs` with [230](230-sheet-says-a-levels-skill-ranks-in-one-row.md): one after the other)
Model: Sonnet to build; reviewed against 210's decisions.
Map: [WotR Build Planner](../map.md)

## Problem

From the second ranking of the MCP server's token costs (2026-11-14, measured on a filled Trickster 20 sheet of 20,334 characters). [210](210-sheet-says-each-level-once.md) trimmed what didn't change between level rows; three repeats of another kind are left, each a fact the sheet says in two places:

1. **The class pick.** Every level's `picks` starts with `{"choice":"level.9.class","label":"Class","name":"Fighter","value":{"class":"fighter"}}`: 1,751 characters over 20 levels (8.6%; 696 of 9,052 on the fighter 8), and the row's `class` says the same. The choice address is `level.<level>.class` for every level.
2. **A feat in `feats` and in `picks`.** A chosen feat is in the row's `feats` (key and name) and in a pick (choice, label, name, value): 970 characters of `feats` repeat what the picks hold (4.8%).
3. **A path's features in a level row and in its rank row.** A Mythic rank's features (`trickster/sneak-attack`, …) are in `ranks[].features` and again in the `features` of the level row the rank is taken at: 15 of 17 distinct rank features appear twice, about 1,100 characters (5.4%). [210's review](210-sheet-says-each-level-once.md) decided a rank row is read on its own (`ranks: [3]`), so the rank row keeps them.

## Build

- The class pick is left out of `picks` when the row says its class (the first row, and each row where `class` changes); a row that doesn't give `class` keeps the class of the row before, as the other trimmed fields do. The research checks whether the pick's `archetype` is said by the row (210: "a row that gives `class` gives its `archetype` or none") and whether `withdraw` and `options` need anything beyond the address `level.N.class` that the skill already names.
- A feat a pick names is left out of `feats`; `feats` holds the feats a row has that no pick of it names (granted feats). The research proposes the rule, checks what an agent reading only `feats` would misread, and names it in the description.
- A level row's `features` leaves out the Mythic path's features that the rank row of the rank taken at that level holds, if the research finds the level rows are not read alone; a `levels: [9]` call without `ranks` would then lose them, so the research proposes either that a narrowed `levels` call also brings the rank rows of those levels or keeps the level copy. Whichever it picks, a test rebuilds today's row.
- The tool description and the `wotr-build` skill say the new rules.
- `responses.rs`: the budgets of "sheet, fighter 8" and "sheet, fighter 20" drop with the new sizes.

## Measured and expected

Filled Trickster 20: 20,334 to about 16,500 (class picks 1,751, feats 970, rank features 1,100); fighter 8: 9,052 to about 8,300 (class picks 696). Risk: medium for 2 and 3 (a reader must know where a fact went); 1 is the safest.

## Done when

A test rebuilds each level's full row (today's `picks`, `feats`, `features`) from the new sheet and compares; the budgets hold the new sizes; the decisions on 2 and 3 are in the ticket's Answer; `scripts/check.sh` passes.
