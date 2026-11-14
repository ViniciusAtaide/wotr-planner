# The leave-out lists are short, or asked for

Type: decision, then feature
Status: open (waits on the user)
Blocked by: none
Model: Opus to research; Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

From the second ranking of the MCP server's token costs (2026-11-14, measured on the live server). Each number the server gives says what it leaves out, in fixed text sent on every call: `totals_leave_out` is 485 characters on every `sheet` (full or narrowed), `attack_leaves_out` 291 on every sheet with an attack line and on `compare`, and `improve` sends its own `leaves_out` (8 lines, about 400 characters). A session of four sheets and an `improve` pays about 3,000 characters for text that never changes. The text is the honesty boundary (map Notes, 2026-11-10): an agent that reports a number must say what it leaves out, and a client once cut the instructions where the boundary stood. So this is the lowest-ranked cut, and it is a decision before it is a build.

## Options for the user

1. **Leave it** (recommended unless the cost matters more than the boundary being in every answer). Nothing changes.
2. **A pointer.** Each leave-out is one line naming `wotr://leaves-out` and the part (`"totals_leave_out": "wotr://leaves-out#totals"`), the full text in the resource and in the skill. About 440 of 485 and 250 of 291 saved a call, and an agent that doesn't read the resource reports numbers without the caveat.
3. **The shortest text that still names every group.** The ten lines of `totals_leave_out` cut to their heads ("buffs and auras", "bonuses against one kind of foe", …): about 200 of 485, the boundary still in each answer.

## Build (after the decision)

The chosen form; the wording stays in one place (`CODING_STANDARDS.md`, "Each fact once"); the `wotr-build` skill and `wotr://leaves-out` say the rest; the honesty-boundary test of the instructions still passes. `responses.rs`: "sheet, fighter 8" and "compare" budgets drop; for option 1 nothing is built.

## Done when

The user has chosen; for 2 or 3 the sizes above hold, a test pins that every answer with numbers still carries its leave-out (text or pointer), and `scripts/check.sh` passes.
