# A level the Build lacks: one wording in every tool

Type: bugfix
Status: open
Blocked by: none
Model: Sonnet.
Map: [WotR Build Planner](../map.md)

## Problem

From the review of the merged queue 209–211 (2026-11-14); older than the queue. Ticket 210 made one wording for a level the Build doesn't have, `unreached::no_such_level` ("The Build has 8 levels, so it doesn't reach level 9."), and routed `sheet` and `validate` to it. `options`, `choose` and `withdraw` still word it their own way: `no_such_choice` in `apps/mcp/src/options.rs` (~596) answers `slot.feat.9.general.0` on a level-8 Build with "The Build has no choice `…`: `sheet` and `todo` print the addresses it has." That breaks "One wording per refusal" (`CODING_STANDARDS.md`), and the hint is weaker since 210: `sheet` gives `todo` only with `todo: true`.

## Build

- A slot, increase, skill or other level address past the Build's last level, in `options`, `choose` and `withdraw`, is refused with `unreached::no_such_level`'s words (the error code stays `no_such_choice`).
- The fallback for an address the Build has no choice at, within its levels, points to where the addresses are now: `new_build`'s or `validate`'s `todo`, `sheet` with `todo: true`, or a `sheet` row's `picks`.
- Tests pin the three tools' refusal text for a level past the end.

## Done when

`options`, `choose` and `withdraw` on `slot.feat.9.general.0` of a level-8 Build each say "The Build has 8 levels, so it doesn't reach level 9."; `scripts/check.sh` passes.
