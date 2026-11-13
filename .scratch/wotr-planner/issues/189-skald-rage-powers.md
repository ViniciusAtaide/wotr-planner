# The skald can't choose its rage powers

Type: bugfix
Status: open
Blocked by: none
Model: Sonnet to build (data against the guide); Opus to review the archetype table.
Map: [WotR Build Planner](../map.md)

## Problem

Found by a QA session through the MCP server (2026-11-13), still so on `main` at `ce9cc3b`. The skald's table in the guide has a Rage Power cell at 3, 6, 9, 12, 15 and 18, and its text says the skald adds the effects of a Barbarian's rage powers to Inspired Rage from 3rd level. The data grants `skald/rage-powers` as a feature on those rows (`libs/game-data/classes/skald.ron`), with no selection: a skald 20 is offered no rage power anywhere (`options` on `slot.selection.3.0` is `no_such_choice`). The feature's own summary says "chosen Barbarian rage powers".

The guide treats them as picks elsewhere too: the Battle Scion "whenever a Battle Scion can add a Rage Power to her song, … can instead take any teamwork feat", and the archetype summaries count them ("Gains fewer Rage Powers", "No Rage Powers", "gives up two Rage Powers").

## Build

- Read `guide-cache/skald.html` for the skald's and each archetype's Rage Power cells and any limit on which rage powers a skald may take; ticket [179](179-dragonheir-scion-gains-draconic-resistances.md) already read the Battle Singer's (6, 12, 18) and the Demon Dancer's (12, 15, 18).
- Give the skald's rows a slot of `barbarian/rage-power` (or a selection sharing its options, as ticket [25](25-shared-selection-options.md) did) at the guide's levels, levelled by the skald so a rage power's level prerequisite reads skald levels; the `skald/rage-powers` feature stays only if something still needs it.
- Each archetype at its own table's cells: Battle Singer, Court Poet (none), Demon Dancer, Inciter (gives up two), Herald of the Horn (keeps them), Hunt Caller, and the Battle Scion's teamwork-feat alternative (ask the user if the guide's sentence doesn't settle whether it is the same slot).
- Ticket [49](49-extra-feats-extra-picks.md)'s rule: whether Extra Rage Power then reaches the skald.

**Not a bug:** the skald's Rogue Talents at 2, 7, 12 and 17 are the guide's ("The Skald can select from the Rogue Talent list at 2nd, 7th, 12th and 17th levels"; ticket [60](60-borrowed-class-selections.md)).

## Done when

A skald 20 is offered a rage power at each of its table's levels and each archetype at its own; a real-data test pins the levels for the class and every archetype; the snapshot diff shows only the new slots; `scripts/check.sh` passes.
