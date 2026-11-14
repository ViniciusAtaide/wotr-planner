# `choose` and `withdraw` return what changed in `todo`

Type: feature
Status: resolved
Blocked by: none
Model: Sonnet to build; Opus to review.
Map: [WotR Build Planner](../map.md)

## Problem

From a ranking of the MCP server's token costs (user, 2026-11-14: "lets do 1 2 and 3"). Every Build tool returns the envelope `build`, `link`, `todo`, and `todo` lists every open choice each time. On a fighter 20 with only its classes chosen, `todo` is about 4,000 of the 4,200 characters a one-step `choose` returns, and 1,600 on a fighter 8. A build session makes a dozen `choose` and `withdraw` calls or more, so it reads the same list a dozen times: about 25,000 to 45,000 characters of a session, most of it repeated.

The agent already holds the list: the call before gave it, for the code it passes in.

## Build

- `choose` and `withdraw` return the changes to `todo` against the Build they were given, in place of the whole list: the choices it opened and the choices it closed, each as `todo` lists it, and the count still open. A choice whose label changed (skill points partly spent) counts as opened with its new label.
- A flag on both returns the whole `todo` as now, for an agent that lost track. Name it as `options`' `envelope` flag is named, or say why not.
- `new_build` and `validate` keep the whole `todo`: their caller has no earlier list for the code they return. Say so in the research if a reason argues otherwise.
- A step refused still returns the changes from the steps applied before it.
- The tool descriptions and the `wotr-build` skill (`.claude/skills/wotr-build/SKILL.md`, embedded by the server; a test holds the instructions' length) say what the changes are and when to ask for the whole list. "Each fact once" in `CODING_STANDARDS.md` stays true.
- `apps/mcp/tests/server/responses.rs` gains a fighter 20 `choose` call, so the size budget sees a Build at the level cap, and its budget holds the new size.

## Done when

A one-step `choose` on a fighter 20 returns only what that step opened and closed; the flag returns today's `todo`; `withdraw` does the same; tests pin the changes for an opening step (a feat that opens a later slot or a Mythic pick), a closing step and a refused step; `scripts/check.sh` passes.

## Answer

Done 2026-11-14.

1. **Built** (`apps/mcp/src/{code,shapes,choose,withdraw,server}.rs`): `choose` and `withdraw` return `build`, `link`, their own fields and a flattened `TodoReport`: `opened` [{choice, label}] (new `todo`'s order; a choice open before whose label changed, skill points part spent, counts as opened with its new label), `closed` [address] (old Build's order; closed only when its address is gone) and `open` (count now open). `opened` and `closed` are left out when empty; `open: 0` stays. `code.rs` gains `report` and `todo_before` and makes `todo` `pub(crate)`; `envelope()` is unchanged (`compare`, `sheet`, `options`, `validate` and `new_build` keep the whole `todo`). The "before" list is taken from the Build as opened, before any step or the withdrawal, so a refused step returns the changes of the steps before it.
2. **Decisions** (the main session's): `closed` lists addresses only, not "each as `todo` lists it" as the ticket said, since a label only repeats what the agent holds ("Each fact once"); the flag is `todo: true` on both tools (default false), returning today's whole `todo` and no `opened`, `closed` or `open`; `new_build` and `validate` keep the whole list; `Envelope` is not split.
3. **Docs**: the tool descriptions of `choose`, `withdraw`, `new_build` and `validate`, and `wotr-build/SKILL.md` (step 4 reads `open` 0, Workflow 2 says which tools return the list and which the changes, Batching says the changes are against the code sent); one line in `qa-session/SKILL.md`. The instructions text is unchanged.
4. **Tests** (`apps/mcp/tests/server/`): choose.rs: `a_closing_step_returns_its_address_and_the_count_not_the_list`, `an_opening_step_returns_the_choices_it_opened_with_their_labels`, `real_data_a_new_level_returns_the_slots_it_opens_as_opened`, `a_label_that_changes_counts_as_opened_with_the_new_label_not_closed`, `a_refused_step_returns_the_changes_of_the_steps_before_it`, `the_todo_flag_returns_the_whole_list_and_no_changes`, `the_changes_over_several_steps_turn_the_old_list_into_the_new_one`, `a_link_gives_the_same_changes_as_its_code`; withdraw.rs: `withdrawing_a_leaf_opens_its_slot`, `withdrawing_a_level_closes_its_slots`, `withdraw_with_todo_true_returns_the_whole_list`; responses.rs gains "choose, fighter 20" (266 characters, 4,188 with `todo: true`; budget 300). Existing tests that read `todo` from `choose` use `todo: true` or `closed`. No Mythic pick was pinned: a new level's slots stand for an opening step.
5. **Snapshot message**: none (no rules or data change).
6. **The queue's proof** should show no difference: the engine and data are untouched.
