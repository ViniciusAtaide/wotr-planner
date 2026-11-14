# `sheet` says what each level changes, and takes the levels it is asked about

Type: feature
Status: open
Blocked by: [209](209-choose-and-withdraw-return-what-changed-in-todo.md) (the envelope, the skill and the size test)
Model: Sonnet to build; Opus to review.
Map: [WotR Build Planner](../map.md)

## Problem

From a ranking of the MCP server's token costs (user, 2026-11-14: "lets do 1 2 and 3"). `sheet` on a fighter 20 with only its classes chosen is about 18,000 characters: 9,500 of levels, 4,000 of `todo`, 1,500 of totals. A level-40 Legend Build doubles the levels. Each level row says again what didn't change since the level before: all six `scores`, `class` with its key and name, the attack line's `weapon`, `critical` and `counts`. And `todo` repeats the list the agent already holds from the envelope of its last call.

This ticket also takes in [206](206-sheet-for-some-levels.md) (user, 2026-11-14: fold it in): a spot check of one fact reads the whole sheet, and the QA session that reads sheets all day used about 664,000 tokens.

## Build

- **Each level says what changed.** A level row leaves out a field whose value is the same as the level before's: the scores (or each score), the class, the attack line's weapon, critical and counts, the saves and BAB if the research finds them worth it. The first level of the answer says everything, so a reader never needs a level it wasn't sent. The research proposes the exact rule, field by field, and what an agent would misread under it; the reviewer attacks it.
- **No `todo` by default.** `sheet` leaves out `todo` unless asked, the way `options`' `envelope` flag works; the envelope's `build` and `link` stay.
- **Levels and ranks** (ticket 206): `sheet` takes `levels` (a list of character levels) and `ranks` (a list of Mythic ranks). Given either, the answer holds only those levels' and ranks' entries, with the Build start and `totals` as now; each level given is said in full under the rule above, since the levels before it aren't sent. A level or rank the Build doesn't have is refused with a reason in the words the other tools use for it (`apps/mcp/src/unreached.rs`).
- Look for the same repetition in Mythic ranks and `totals` (a skill's zero counts: `Ranks 0` beside `ranks: 0`), and cut what says nothing.
- The tool description and the `wotr-build` skill say what a level row leaves out, and to ask for the levels a question is about. Other tools that return level rows (`compare`?) are checked for the same repetition and listed in the report, changed or not.
- `apps/mcp/tests/server/responses.rs` gains a fighter 20 `sheet` call, and its budget holds the new size.

## Done when

A fighter 20's `sheet` is about half today's size with nothing lost (a test rebuilds every level's full row from the answer and compares it with today's); `levels: [9]` returns level 9 alone, in full; the refusal is pinned; `scripts/check.sh` passes. Ticket 206 is closed as taken in here.
