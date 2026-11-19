# The narrowing check sees added requirements, raised values, Any to All and stand-ins

Type: build (tooling)
Status: open
Blocked by: 337a
Model: Opus to build.
Map: [WotR Build Planner](../map.md)
Design: [design-N.md, T6b](../research/architecture/design-N.md), in the frame of [overall.md](../research/architecture/overall.md); decision 6 in [decisions.md](../research/architecture/decisions.md). Absorbs ticket 324.

## Problem

Ticket 324's list. These rewrites pass the guards today and refuse characters the game allows:
- an added requirement that no planner site or slot meets;
- a stand-in entry, one planner entry for two game blueprints in two selections;
- a raised value;
- an Any that turns into an All.

## Change

- Add the kinds `RaisesValue`, `Tightens`, `AddsRequirement` (both of design-N's rules) and `StandIn`.
- Add the allowlist grammar for them.
- Add `listed_in` and `stand_ins` in `Qualification::new`.

## Done when

- A full `--write prerequisites --dry-run` writes neither the Elemental Witch's three rays nor `kineticist/element/{air,earth,fire}`. The rays are left whole, as `adds … (Unoffered ElementalWitchPatronSelection 3172b696…)`.
- The elements stay whole even with a scratch allowlist that allows their `adds Not archetype kineticist/blood-kineticist`: `StandIn` holds them.
- Each new kind has a fixture test or a tree test.
- The Answer gives the dry run's count per kind over every reached rewrite, with a scratch allow-all list.
- Ticket 324 is resolved, minus Greater Enduring Spells, which is ticket 208's (decision 6).

## Files

- `tools/game-check/src/write/{qualification.rs, prerequisites.rs}`
- `tools/game-check/prerequisite-narrowing.txt` (header only)
- the two test files
- `.scratch/wotr-planner/issues/324-*.md`

## Checks

As for 337a.
