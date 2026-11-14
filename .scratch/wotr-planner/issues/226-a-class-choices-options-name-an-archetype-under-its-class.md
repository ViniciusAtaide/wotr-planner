# A class choice's `options` name each archetype under its class

Type: feature
Status: open
Blocked by: none (runs after [229](229-values-lists-a-parameterised-feat-once.md) or before it, never beside it: both edit `apps/mcp/src/{options,shapes}.rs`)
Model: Sonnet.
Map: [WotR Build Planner](../map.md)

## Problem

From the second ranking of the MCP server's token costs (2026-11-14, after 209–211; measured on the live server). `options` on `level.1.class` of a blank Build is 28,828 characters for 140 rows; on `level.9.class` of a fighter 8, 32,961. 18 rows are base classes (3,586 characters); the other 122 are **archetypes**, each a row of its own with a summary (11,416 characters of summaries) and a value that repeats its class (`{"archetype":"alchemist/chirurgeon","class":"alchemist"}`, 6,718 characters of values): 25,015 characters, 87% of the answer. An agent choosing a class reads 122 archetype summaries of classes it will not take, and does it again at every level it adds a class at. `compact` already cuts the summaries (13,056), but an agent that wants to pick an archetype needs them.

## Build

- On a class choice (`level.N.class`), `options` lists each **class** as one row with its summary, as now, and lists its archetypes on the row as names only, with the archetype keys: `{"name":"Alchemist","summary":…,"value":{"class":"alchemist"},"archetypes":[{"name":"Chirurgeon","key":"alchemist/chirurgeon"},…]}`. The research proposes the exact shape (a list of names and keys, or a map); the value an agent passes to `choose` stays what it is today: `{"class":…,"archetype":…}`.
- A new argument on `options` (the research names it; `within` is the candidate) takes a class key and lists that class's archetype rows in full, with their summaries and `value`s, as the rows are today. A key that isn't a class on this choice is refused with a reason.
- An archetype the Build can't take now (closed) is still found: with `only_open` false the row says its `status` as today; with `only_open` true the class row lists only the open archetypes and `closed` counts the rest, as for any row.
- **How it sits with the Offer queue.** [200](200-offers-come-from-the-engine.md) moves `offers()` into `rules` and gives each Offer a typed value (class with archetype). This ticket changes only the step that turns Offers into `groups` in `apps/mcp/src/options.rs`, as [211](211-options-rows-say-each-fact-once.md)'s grouping does: it folds the archetype Offers into their class row by the value's class, and adds no field to `Offer`. If 200 lands first, the builder reads the new enum value; if this lands first, 200's builder starts from this shape.
- The tool description, the `options` argument docs and the `wotr-build` skill say the new shape, and that `compact` is the same shape without summaries. `search` (kinds `archetype`) is unchanged and is how an agent looks for an archetype by what it does.
- The size budget in `apps/mcp/tests/server/responses.rs` gains "options, a class choice" on a blank Build (`new_build`'s code, `level.1.class`).

## Measured and expected

Rows rebuilt from the same data: base class rows 3,586 plus archetype names and keys about 3,800 (122 × the name and key without summary or repeated class) and the envelope: about 8,000 against 28,828, a saving near 20,000 a listing; `within: "alchemist"` about 1,400 a class. A multiclass Build's later class choices cost the same each, 33,000 on a fighter 8.

## Done when

`options` on a blank Build's `level.1.class` is about 8,000 characters (the test's budget about a tenth above what is built) and has the same classes, archetype keys and class summaries; a test shows every archetype row of today's listing is reachable (by `within` or the class row), with the same summary and the same `choose` value; `choose` with an archetype value from the new shape works; the refusals are pinned; `scripts/check.sh` passes.
