# `sheet` says a level's skill ranks in one row

Type: feature
Status: open
Blocked by: none (edits `apps/mcp/src/{sheet,shapes}.rs` with [231](231-sheet-says-a-class-a-feat-and-a-feature-once.md): one after the other)
Model: Sonnet.
Map: [WotR Build Planner](../map.md)

## Problem

From the second ranking of the MCP server's token costs (2026-11-14; the QA session measured `picks` at 35% of a Legend 40 sheet, mostly per-skill rows). A level's skill points come in `picks` as one row per skill: `{"choice":"level.9.skill.athletics","label":"Athletics ranks","name":"Athletics +3","value":3}`, 95 characters a skill, three or four skills a level, up to 40 levels. The same row's `skill_ranks` (the running ranks, trimmed to what changed) and `skill_points` (`available`, `spent`) already say the same facts in another form, and `totals.skills` says them a third time. The filled Trickster 20 sheet has 6 such rows (567 characters of 20,334); a Build that spends three skills at each of 40 levels has 120 rows, about 11,400 characters of its 33,000.

## Build

- A level's skill picks become one row: `{"choice":"level.9.skill","label":"Skill points","value":{"athletics":2,"mobility":1}}`, the value in the shape `choose` takes for `level.N.skill` (a map of skill key to ranks added at this level); a level with no ranks spent has no row (its open points are `todo` and `skill_points`).
- The research checks whether an address `level.N.skill.<key>` is accepted anywhere (`choose`, `withdraw`, `options`) and whether any test or skill text names it. If it is, the row still carries the whole address in the `value` of its row or the description says how to build one; if not, nothing is lost. It also checks whether `skill_ranks` per row says anything the row's map doesn't (the running total) and keeps it only if so.
- `compare`'s `picks` rows and any other tool that lists a level's skill picks are checked for the same repetition and listed in the report, changed or not.
- The tool description and the `wotr-build` skill say a level's ranks are one row.
- `responses.rs` gains a "sheet" call on a Build with skill ranks spent at each level of a fighter 8 (the test's `fighter()` with a `level.N.skill` pick per level), new in the file.

## Measured and expected

Per-skill row 95 characters; the one-row form is about 75 plus 20 a skill: three skills 285 to about 135. A 40-level Build's 120 rows (about 11,400) become about 5,400, a saving near 6,000 a full sheet; narrowed calls save a level's share.

## Done when

A test rebuilds today's per-skill rows from the new rows and compares; the new fixture's sheet is about the size built (budget a tenth above); the report names every tool checked; `scripts/check.sh` passes.
