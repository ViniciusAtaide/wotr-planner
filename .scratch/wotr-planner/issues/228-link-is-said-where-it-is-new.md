# `link` and the echoed `build` are said where they are new

Type: feature
Status: open
Blocked by: none (edits `sheet`'s result type with [227](227-narrowed-sheet-leaves-out-the-start-and-totals.md): one after the other)
Model: Sonnet.
Map: [WotR Build Planner](../map.md)

## Problem

From the second ranking of the MCP server's token costs (2026-11-14, measured on the live server). Every Build result carries `link`, a 187-character URL that holds the whole code again (`https://…/#b=<code>`), and an agent needs it once, for the user, at the end. After [209](209-choose-and-withdraw-return-what-changed-in-todo.md) it is the largest fixed part of the small answers: `choose` of one step on a fighter 8 is 624 characters, 189 of them `link` (30%); a one-step `choose` on a fighter 20 is 300, `link` 121 (40%); `withdraw` 404, `link` 175 (43%). A session of 25 steps pays it 25 times. Two more echoes: `sheet` returns `build`, which is the code the agent just sent (checked on a filled Build: byte-identical), and `compare` returns each Build's `build` and `link` in `builds` although its lists follow the order the agent gave (319 and 268 characters per Build on the filled Trickster).

## Build

- `choose` and `withdraw` keep `build` (it is new) and leave out `link` unless called with `link: true` (the name follows `todo: true` and `options`' `envelope`; the research confirms it). `new_build` and `validate` keep both: they are the calls an agent ends on, so the final link is one call away. `validate`'s description and the `wotr-build` skill say "give the user the link from `validate`, or from `choose` with `link: true`".
- `sheet` and `compare` leave out `build` (the agent holds it) and `link`, unless `link: true`; `compare`'s `builds` rows keep the numbers, in the order given. The research checks that no test or tool reads them back, and whether `describe` or `improve` echo either.
- The instructions text names where the final link comes from; its length test must still pass (under 4,000 characters).
- `responses.rs` already has "choose, fighter 20" (budget 300), "choose, a level added to fighter 8" (690), "withdraw, fighter 8 pick" (445) and "sheet, fighter 8" (9,950): their budgets drop with the new size.

## Measured and expected

`choose` one step, fighter 20: 300 to about 180; a level added to fighter 8: 624 to about 435; `withdraw` 404 to about 230; `sheet` 325 less (more on a Build with a long code); `compare` of two filled Builds about 1,200 less. A session of 25 steps, four sheets and a compare saves about 6,400 characters.

## Done when

The budgets above hold their new sizes; `link: true` gives today's field; `validate` and `new_build` are unchanged; a test pins that a final link comes from `validate`; the skill and the instructions say so; `scripts/check.sh` passes.
