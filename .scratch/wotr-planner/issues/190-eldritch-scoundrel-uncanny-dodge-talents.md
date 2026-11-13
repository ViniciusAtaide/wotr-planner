# The Eldritch Scoundrel can't take (Improved) Uncanny Dodge as a rogue talent

Type: bugfix
Status: open
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

Found by a QA session through the MCP server (2026-11-13), still so on `main` at `ce9cc3b`. The guide's rogue page, under Rogue Talents: "An Eldritch Scoundrel can select (Improved) Uncanny Dodge as Rogue Talents." The Eldritch Scoundrel loses both features ("No Uncanny Dodge"), and the data's own summary of `rogue/rogue-talents` says it "can also take (Improved) Uncanny Dodge" (`libs/game-data/classes/rogue.ron`). But `options` on an Eldritch Scoundrel's talent slot (`slot.selection.8.0`, `only_open: false`) offers neither.

## Build

- Two options of `rogue/rogue-talents`, Uncanny Dodge and Improved Uncanny Dodge, each granting the rogue's feature (`rogue/uncanny-dodge`, `rogue/improved-uncanny-dodge`), open only to an Eldritch Scoundrel. The selection is shared with the bard and the skald (ticket [60](60-borrowed-class-selections.md)), so the prerequisite names the archetype.
- Whether Improved Uncanny Dodge needs Uncanny Dodge first: the guide's sentence doesn't say. Take the rogue's order (Uncanny Dodge at 4, Improved at 8) as the prerequisite only if the guide supports it; otherwise ask the user.
- The same shape as [173](173-uncanny-dodge-as-a-slayer-talent.md) (Uncanny Dodge as a slayer talent), but here the guide names it, so no decision is needed.

## Done when

An Eldritch Scoundrel is offered both at its talent slots (2, 8, 12, 16, 20) and no other class or archetype is; a real-data test pins it; `scripts/check.sh` passes.
