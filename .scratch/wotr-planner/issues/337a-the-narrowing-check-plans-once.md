# The narrowing check: one plan, one record, a relative carrier guard

Type: build (tooling)
Status: open
Blocked by: 333, 334
Model: Opus to build.
Map: [WotR Build Planner](../map.md)
Design: [design-N.md, T6a](../research/architecture/design-N.md) (interface §1, what goes §2, tests §4, risks §5), in the frame of [overall.md](../research/architecture/overall.md); decisions 7 in [decisions.md](../research/architecture/decisions.md).

## Problem

- The game check plans twice, and `stale` and `audit` each re-plan.
- Narrowings and carrier gaps are strings with guessed owners.
- The carrier guard is absolute, so it holds entries whose old tree already had the gap (308h's 17).

## Change

Design-N §1 and §2, except the four new kinds:
- add `write/qualification.rs` with `diff` (Drops and AddsNot over ticket 333's view), `carrier_gaps`, the relative `Carrier`, and `Plan`/`FullPlan`;
- index the planner in place of its linear scan;
- turn owners off, but for the "no form" gaps;
- `regenerate` writes a new line `-> ?`.

## Done when

- The plain check calls `plan` once.
- The audit's output equals the base's, less ` -> 307` and less the `left whole` lines the relative guard releases. Each released line is named in the Answer, with its held tree's gap.
- A dry run lists those released entries as written, and nothing else moves.

## Files

- `tools/game-check/src/write/{qualification.rs (new), prerequisites.rs, carrier_guard.rs (folded), mod.rs}`
- `tools/game-check/src/{carriers_known.rs, main.rs, planner.rs}`
- `tools/game-check/carriers-known.txt` (regenerated)
- `tools/game-check/tests/fixture/{prerequisites_write.rs, qualification.rs (new), main.rs}`

## Checks

- `cargo test -p game-check`, with the new names in `--list`
- `cargo fmt --all --check`
- `scripts/test-modules.sh`
- the game check: exit 0, no stale line, timed on the base and the tip
- `--carrier-audit` and `--write prerequisites --dry-run`, each diffed against the base's
