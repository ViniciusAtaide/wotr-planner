# `compare` says a difference that repeats once

Type: feature
Status: open
Blocked by: none (after [228](228-link-is-said-where-it-is-new.md), which edits `builds`)
Model: Sonnet.
Map: [WotR Build Planner](../map.md)

## Problem

From the second ranking of the MCP server's token costs (2026-11-14). The QA session measured about 18,800 of 20,000 characters of a fighter vs wizard `compare` as a difference that is the same on every level. On the live server, two Builds of 20 levels (fighter, wizard; nothing else chosen) give 6,878 characters, of which `levels` is 5,822. What repeats in it: the class pick on every level, `{"choice":"level.N.class","label":"Class","names":["Fighter","Wizard"]}`, 1,471 characters (25% of the levels), the same names at each level; `features` and `feats` lists of one Build against an empty list. The 20,000-character case the QA session saw I could not rebuild with these Builds: it was larger because the Builds were filled, so the builder measures the QA session's Builds (the map's History entry and the QA notes hold them) first and states the repeat's share there.

## Build

- A level's difference that is the same as the level before's, apart from the level in its address, is left out of that row: the class pick when both Builds take the same classes at every level, so a reader says "Fighter against Wizard from level 1" once, in the first row that has it; the research proposes the exact rule, as 210 did for `sheet`, and what an agent would misread under it.
- `compare` also takes `levels` (a list of character levels) as `sheet` does, and each level given is said in full under the rule, since the levels before it aren't sent.
- The summaries (`builds`) and the lists that follow the order given stay as they are.
- The tool description and the skill say the rule and the argument.
- `responses.rs` gains "compare, fighter 20 against wizard 20" (6,878 characters today).

## Measured and expected

Fighter 20 against wizard 20: 6,878 to about 5,300 by the class pick alone; the filled Builds of the QA session by the share the builder measures there (up to the 18,800 characters it reported). Risk: low if the first row says all.

## Done when

A test rebuilds every level's full difference from the trimmed rows and `levels: [9]`; the budget holds the new size; `scripts/check.sh` passes.
