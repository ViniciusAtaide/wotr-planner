# `options` rows say each fact once

Type: feature
Status: resolved
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

## Answer

Done 2026-11-14.

1. **Built** (`apps/mcp/src/{shapes,options,server}.rs`, `.claude/skills/wotr-build/SKILL.md`): `OptionsResult.options` is now `groups: Vec<OptionGroup>`, `OptionGroup { category: Option<String> (left out when None), options: Vec<OptionLine> }`. `OptionLine` has no `category`, and `status` is optional, left out exactly when `only_open` is true and the offer is plain open with no gear mark (`{with_gear}` and `{leans_on_gear}` rows keep theirs; with `only_open: false` every row keeps its status, `"ok"` included). The grouping is done in `options()` after `offers()`, `offers()`, `Offer`, choose.rs and rank.rs are untouched: groups come in the order of their first row, every row of a category merges into its group, rows keep the engine's order within a group, a choice with no categories is one group with no `category` (gear's `None` row is in that group, first), and there is never an empty group. Gear places keep their categories (`gear.body`: None, then Robe, then Armour, the engine's order).
2. **Decisions** (the main session's, from the research): the shape, the order of groups, status left out only for plain open rows with `only_open`, gear categories kept, the docs. None found false. Mine (builder): `gear.body` lists Robe before Armour because that is the engine's order of first rows; the test pins it.
3. **Other tools listing options:** `improve` (ranked `options`/`no_change`, each row carries its own change, no shared category or status), `search` (matches carry `kind` and `category`, compact mode already leaves it out), `describe`, `sheet`, `choose`, `withdraw`: no repetition of this kind, no change. The `compact` shape of `options` is the same grouped one.
4. **Docs**: the `options` tool description (shape `groups`, each with its `category`; status left out for an open row when `only_open`, gear marks always kept), the `only_open` arg doc, `OptionGroup` and `OptionLine.status` docs, SKILL.md's Values paragraph and its `with_gear` line. The instructions text is unchanged.
5. **Sizes** (characters of the whole answer, real data, before rebuilt from the same rows with `category` and `status: "ok"` on each): fighter 8 general feat slot (74 rows) 14,010 to 11,720; compact `start.race` (48 rows) 3,290 to 2,631; `level.1.class` on a new build (140 rows) 34,111 to 28,828; `gear.belt` (29 rows) 5,271 to 4,406.
6. **Tests** (`apps/mcp/tests/server/`): options.rs: `options_group_their_rows_by_category_each_named_once` (no repeated category, no row has `category`, no empty group, the row set equals the open rows of the `only_open: false` listing and `closed` the rest), `groups_keep_the_engines_order_and_start_at_a_categorys_first_row`, `a_choice_without_categories_is_one_group_with_no_category`, `only_open_false_keeps_every_rows_status_ok_included`; gear.rs: `a_gear_place_groups_its_items_by_category_and_none_leads`, `open_rows_leave_out_status_but_gear_marks_stay`. support.rs gains `rows(&result)` (flattens `groups`, putting `category` back on each row); the existing read sites in options, gear, compact (`keys(&bare)` now `["choice","closed","groups"]`), choose, end_to_end, finesse_training, pick_values, rank_levels, walks and summaries use it, and their `status == "ok"` assertions on default calls now say there is no status. responses.rs: "options, a feat slot" budget 15,500 to 12,900.
7. **Snapshot message**: none (no rules or data change).
8. **The queue's proof** should show no difference: the engine and data are untouched.

### Review fixes

- **Group order pinned independently:** `groups_keep_the_engines_order_and_start_at_a_categorys_first_row` compared the groups with `rows()`, which flattens in group order, so it could not fail. It now pins a literal order for the test world's feat slot, whose engine list (by key) interleaves categories: Magic, Weapon, Armour and Shield, Miscellaneous, Skill, Combat Manoeuvres, Unarmed Combat, Class, Teamwork. Checked by breaking the grouping on purpose (groups sorted by category): the test fails; restored.
- **Values and compact:** `a_values_listing_is_grouped_too_with_no_category_on_a_row` checks that a `values: true` listing is grouped with no `category` on a row, and that `compact` with `only_open: false` keeps every row's status.
- **Doc comment** on `options()` says the rows are grouped and an open row leaves out its status when `only_open`.
