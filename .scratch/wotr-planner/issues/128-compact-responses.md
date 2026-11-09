# Compact MCP responses

Type: task
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The Max DC run (2026-11-09, see [119](119-caster-line.md)) spent much of its context on text it didn't use:
- `search {kinds: ["mythic_ability"]}` returned all 63 abilities with full prerequisites. Witch Wandering Hex's prerequisite alone names about 55 hexes; Mythical Beast's names 17 animal companions, two of them twice ("Animal Companion or Animal Companion").
- `options` on `slot.mythic.1` listed 41 open abilities with summaries, and every `options` call repeats the envelope (`build`, `link`, `todo`), a code and a link of about 300 characters each, twice per call.
- Every `choose` returns the link beside the code; the agent needs the link only at the end.

## Scope

1. **A `compact` argument** on `search` and `options`: each hit or option as key, name, kind or category, and status, with no summary, prerequisite or guide link. The default stays as today.
2. **Long prerequisites shortened** in `search` and `describe`: past a length, a prerequisite of many alternatives reads "one of 55 hexes (Air Barrier, Battle Master, …)" with the full tree under `describe`. Duplicate alternatives in the text are merged (the Mythical Beast case is the text's, or the data's; find which).
3. **The envelope on request:** `options` returns `build` and `todo` only when asked (`envelope: true`), since it changes nothing; `choose`, `withdraw` and `new_build` keep it.

## Open questions (with recommendations)

1. **Drop `link` from the envelope?** *Recommend:* keep it on `choose`; it's what the user gets, and the agent may stop at any step.
2. **Mythical Beast's "Animal Companion or Animal Companion":** *Recommend:* if two features share a name, the text names their owner ("Animal Companion (Hunter)") rather than merging them, as the other alternatives already do.

## Done when

Server tests cover `compact` on both tools, a shortened prerequisite and `options` without the envelope; the run's `search {kinds: ["mythic_ability"]}` is under half its size in compact form (measured, the numbers in this ticket); `scripts/check.sh` passes.
