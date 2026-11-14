# `sheet` says what each level changes, and takes the levels it is asked about

Type: feature
Status: resolved
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

## Answer

Done 2026-11-14.

1. **Built** (`apps/mcp/src/{shapes,sheet,attack,caster,totals,unreached,compare,server}.rs`): `sheet` returns `SheetResult<SheetLevel>`; `sheet_of` and `compare` stay on `SheetResult<LevelLine>` (full lines, and `compare`'s open-choices count from `todo`). A trim step, `level_rows`, runs last in `sheet()` after `naked_numbers`, and builds each `SheetLevel` (with `AttackRow` and `CasterRow`) from the full `LevelLine`: on a full sheet (neither `levels` nor `ranks`) a field is left out when it equals that field on the nearest earlier row that has it. Always given: `level`, `class_level`, `bab`, the saves, `attack.attacks`, `damage`, `power_attack`, the caster line's `caster_level`, `dcs`, `heightened`, `spell_penetration`, `concentration`. Left out when unchanged: `class` with `archetype`, `scores` (all six together), `skill_points`, `skill_ranks`, `attack.weapon`, `critical`, `counts`, and the caster line's `class`, `ability` and three counts. `feats`, `features`, `mythic_ranks`, `picks` (the `level.N.class` pick included) and the rank rows are unchanged. `totals` drops every count part that is 0 and leaves out `counts` when none is left. `todo` only with `todo: true`; `build` and `link` always. `levels` and `ranks` (sorted, once each, independent: levels alone gives no rank rows, ranks alone no level rows, `[]` none of that section) give those rows only, every level row in full; `naked.levels` is filtered the same way.
2. **Decisions** (the main session's, from the research): the trim rule above; separate output shapes; totals without zero parts; `todo` as a flag; the refusals, argument name first and the first bad entry in the order given: a level the Build lacks ("`levels`: The Build has 4 levels, so it doesn't reach level 5.", level 0 adding "Levels start at 1."), a rank through `Unreached::of(..).reason(rank, "`sheet` has no row for it")` (`Unreached::no_row`). Mine (builder): a row that gives `class` gives its `archetype` or none; an empty first-row list (`skill_ranks`, counts) is left out as saying nothing, so absent on the first row is none. Note for review: a full sheet does list a not-reached rank's row, so "`sheet` has no row for it" is not literally true there; `ranks` refuses those ranks all the same, as decided.
3. **Other tools with level rows:** `compare` reads full lines and keeps them (its summary has no level rows, only counts); `options`, `choose`, `withdraw` and `describe` have none of this shape. No change.
4. **Docs**: `sheet`'s description (three groups of fields, `levels`, `ranks`, `todo: true`), `wotr-build/SKILL.md` (which tools return `todo`; a paragraph on the omitted fields and on asking for levels), `qa-session/SKILL.md` (a line suggesting `levels`/`ranks` for spot checks). The instructions text is unchanged.
5. **Sizes** (characters of the whole answer, measured on the real data): fighter 8 (responses.rs) 12,979 to 9,058; fighter 20 with only classes chosen 18,027 to 10,553; the Max DC run's Kitsune Sorcerer 20 26,208 to 23,265, and with the compulsion `caster` line 41,172 to 31,555 (a filled Build's `picks` and `skill_ranks` are most of what is left).
6. **Tests** (`apps/mcp/tests/server/`): sheet.rs: `a_full_sheet_leaves_out_what_did_not_change_since_the_level_before`, `todo_comes_with_the_sheet_only_when_asked`, `real_data_every_level_of_a_fighter_20_rebuilds_from_the_trimmed_sheet`, `real_data_every_level_of_a_caster_20_rebuilds_from_the_trimmed_sheet` (each level's row rebuilt by `support::full_row` equals `levels: [N]`'s), `levels_returns_those_levels_alone_each_in_full_sorted`, `ranks_returns_those_rank_rows_alone`, `a_level_or_rank_the_build_lacks_is_refused_with_the_reason`; totals.rs: `totals_leave_out_the_counts_that_are_zero`; gear.rs: `sheet_shows_both_numbers_and_the_dependent_pick` now checks naked rows stay full; responses.rs: "sheet, fighter 20" (budget 11,600) and "sheet, fighter 8" lowered to 9,950. Existing tests that read `todo` pass `todo: true`, and ones that read a later row's trimmed field use `full_row` or `levels`. A Legend-ended rank is not pinned: no small real-data Build reaches it cheaply.
7. **Snapshot message**: none (no rules or data change).
8. **The queue's proof** should show no difference: the engine and data are untouched.
