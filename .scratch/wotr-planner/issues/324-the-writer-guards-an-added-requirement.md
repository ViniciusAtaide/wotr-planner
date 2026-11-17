# The prerequisites writer guards an added requirement and a stand-in entry

Type: build (tooling)
Status: open
Blocked by: —
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Problem

The 308 queue's range review (2026-11-17) and 308d and 308f found a path the writer's guards (308a's narrowing allowlist, 308h's carrier guard, 322's carrier list) don't close: **a requirement an entry gains**. The narrowing guard counts only a dropped alternative or an added `Not`; the carrier guard sees a gained fact only when a game component grants it. So these pass and refuse characters the game allows:

- an added `Has` of an entry no planner row or slot grants: the Elemental Witch's acid, electricity and fire rays need WitchElementsPatronProgression 67e85f52b1f020847aaa738d8999d4cd, which the planner holds as a feature, not a pick (308f; ticket 308's "308 left" holds them unwritten);
- an added requirement on a planner entry that stands for two game blueprints in two selections: `kineticist/element/air` is listed in `selections/kineticist/expanded-element.ron` for SecondaryElementAir 659c39542b728c04b83e969c834782a9, while its blueprint is ElementalFocusAir (308f put the elements back by hand);
- the Stalwart Defender's Endurance (308d), now a carrier hit.

Smaller: facts are compared whatever their value (`leaf_at`), an `Any` turning into an `All` isn't seen, and regenerating `carriers-known.txt` lists any new hit `-> 307` with nothing checking it.

## Change

Count an added requirement as a narrowing when its fact's entry is one no planner row, grant or selection gives, or when the rewritten entry is offered in a selection whose game blueprint differs from its own (`shared_options`); either then needs an allowlist line with evidence. Compare a leaf's value and an `Any`/`All` change. Make a carrier-list regeneration print the new hits for the main session to check.

## Done when

A full dry run writes none of the four held fields in ticket 308's "308 left" without an allowlist line; fixture tests for each case; `cargo test -p game-check` passes.
