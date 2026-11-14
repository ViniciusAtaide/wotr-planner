# `options` rows say each fact once

Type: feature
Status: open
Blocked by: [210](210-sheet-says-each-level-once.md) (the skill and the size test)
Model: Sonnet.
Map: [WotR Build Planner](../map.md)

## Problem

From a ranking of the MCP server's token costs (user, 2026-11-14: "lets do 1 2 and 3"). `options` on a fighter 8's general feat slot is about 14,000 characters for 74 rows. With `only_open` (the default) every row says `"status":"ok"`, about 1,000 characters that say nothing. Every row says its `category` too, and 74 rows share a dozen categories: about 1,500 more.

## Build

- With `only_open` true, a row leaves out `status`: every row is open. With `only_open` false, rows keep it.
- Rows come grouped by category, each category named once; the research proposes the shape (a list of `{category, options}` or a map) and checks `search`, `improve` and other tools that list options for the same repetition, and lists them in the report, changed or not.
- Summaries stay: they are what the agent chooses by.
- The tool description and the `wotr-build` skill say the new shape; a gear place's options are checked too.
- The size budget in `apps/mcp/tests/server/responses.rs` holds the new size of its `options` call.

## Done when

The fighter 8 feat slot's `options` drops by about 2,500 characters with the same rows and words; `only_open: false` still marks each row's status; `scripts/check.sh` passes.

This ticket runs before the Offer queue's [200](200-offers-come-from-the-engine.md) (user, 2026-11-14), which moves `offers()` into `rules`: 200's builder starts from this shape.
