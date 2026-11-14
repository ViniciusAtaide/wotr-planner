# `options` with `values: true` lists a parameterised feat once

Type: feature
Status: open
Blocked by: none (runs one after [226](226-a-class-choices-options-name-an-archetype-under-its-class.md): both edit `apps/mcp/src/{options,shapes}.rs`)
Model: Sonnet.
Map: [WotR Build Planner](../map.md)

## Problem

From the second ranking of the MCP server's token costs (2026-11-14, measured on the live server). `options` on a general feat slot is 10,143 characters; with `values: true`, which lists every parameter of Weapon Focus, Improved Critical, Skill Focus and the rest as a row, it is 48,657 (247 rows), and on the test fighter 8's slot 7 it is 35,580 against 11,720. Each row repeats the feat's whole summary and the key in its value (`{"key":"armor-focus","parameter":"light"}`): 65 feats make the 247 rows, Weapon Focus and Improved Critical have 73 each, and 18,212 characters are summaries said again. An agent that asks for values wants the parameter names to choose one, not the same sentence 73 times.

## Build

- With `values: true`, a feat that takes a parameter is one row with its summary once and its parameters listed: `{"name":"Weapon Focus","summary":…,"key":"weapon-focus","parameters":[{"name":"Greatsword","parameter":"greatsword"},…]}`; the value passed to `choose` stays `{"key":…,"parameter":…}`. The research proposes the exact shape and how `status`, `reason`, `with_gear` and `leans_on_gear` sit on a parameter that is closed or gear-dependent while its feat is open (a parameter keeps its own, the feat row's say what holds for all); a parameter that differs from its feat only by name and value needs nothing else.
- Rows without parameters, and a listing without `values: true`, are unchanged. `compact` drops the summary as today.
- Check the other places an entry's values are listed (`only_open: false`, gear places, `improve`'s `options`, `search`) for the same repetition and list them in the report, changed or not.
- **How it sits with the Offer queue.** [200](200-offers-come-from-the-engine.md) gives each Offer "the noun for an entry of values" and an edit for the value it stands for (`like`); the shaping stays in `options()`'s grouping step in `apps/mcp/src/options.rs`, over `Offer`'s output, and adds no field to `Offer`.
- The tool description, the `values` argument doc and the `wotr-build` skill say the shape.
- `responses.rs` gains "options, a feat slot with values" on the fighter 8's `slot.feat.7.general.0` (35,580 today).

## Measured and expected

Rows rebuilt from the same data: 47,974 to about 18,200, a saving near 29,700 on the slot above (62%); the fighter 8's slot 7 about 35,600 to 14,000. A session that asks for values once saves about 20,000 to 30,000.

## Done when

The test's call is about 14,000 characters (budget about a tenth above what is built), listing the same feats, parameters, summaries and `choose` values as today (a test rebuilds today's rows from the new shape and compares); a parameter's closed status is pinned; the other tools are listed in the report; `scripts/check.sh` passes.
