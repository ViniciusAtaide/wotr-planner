# `choose` and `withdraw` return what changed in `todo`

Type: feature
Status: open
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
