# The domain split, with a slot-aware link migration

Type: build
Status: open
Blocked by: 244
Model: Opus to build: a codec change that keeps every link opening, plus a data split.
Map: [WotR Build Planner](../map.md)

## Problem

[244](244-the-game-data-laid-out-by-blueprint.md) lays the data out by blueprint. Its decision 10 splits merged entries back into the game's objects. One split can't be made safely there; the user, 2026-11-15, deferred it to this ticket ("Defer the domain split").

The game keeps a separate progression for each domain in each slot:
- the cleric's second domain (`SecondDomainsSelection` 43281c3d7fe18cc4d91928395837cd1e, listing `<Domain>DomainProgressionSecondary`);
- the Separatist's second domain (`SecondDomainsSeparatistSelection` 42b781e4375d499383b2602d90661283, listing `…Separatist`);
- the druid's and Blight Druid's domains (`BlightDruidBond` 096fc02f6cc817a43991c4b437e12b8e, listing `…Druid`).

The planner has one option per domain (`cleric/domain/air`), offered in all of these slots.

Splitting gives those slots options with other keys. An old link names `cleric/domain/<x>` at the second-domain slot, so it would no longer open. 244's baseline counted 114 corpus Builds with a second domain, 13 Separatists and about 7 druids. Aliases can't help, because the old key stays live in the first slot.

## Change

- **A slot-aware link migration.** `open_link` reads an old domain key in a second-domain, Separatist or druid slot as that slot's matching new option, by the domain it names.
- **The split.** Each slot's selection holds the game's per-slot options, each with its blueprint and its own prerequisites (the game's "not the same domain as the first" leaves).
- **The game check.** Its `known.txt` lines for the domains, filed under 244 and left there when 244 deferred this, are cleared.

## Done when

- Every corpus and bench link opens with the same picks, with the migration doing the reading, and a test covers each slot kind.
- The domain lines of `known.txt` are gone, and the game check exits 0.
- The scoped checks pass.
