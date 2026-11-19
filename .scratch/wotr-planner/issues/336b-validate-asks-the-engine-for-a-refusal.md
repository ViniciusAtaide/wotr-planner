# `validate` asks the engine for a refusal

Type: build (MCP)
Status: open
Blocked by: 336a
Model: Opus to build.
Map: [WotR Build Planner](../map.md)
Design: [design-R.md, T5b](../research/architecture/design-R.md), in the frame of [overall.md](../research/architecture/overall.md); decisions in [decisions.md](../research/architecture/decisions.md).

## Problem

- **330 item 1:** `validate` drops the 171 hint that `choose` gives.
- `validate` re-derives the trim, held-where, the 192 clause and `needs` from the walk on its own. This cost ticket 192 two fix rounds: d0097f67 and 9ce1f809.

## Change

The `validate` adapter (design-R §2.3) words what the walk returns through `Refusals`. `trimmed`, `unmet`, `already_held` and `tree.rs::needs` go. `validate` keeps asking the walk.

## Done when

- 330 item 1 reads "… MR 9 taken at level 20 opens it, which moves MR 10 to level 20: declare `mythic.9.level` 20", tested exactly through `validate`. 330 item 1 is marked done.
- With a second failure beside it, the hint is absent, as in ticket 192.
- The reasons record shows that only the 330.1 lines changed.

## Files

- `apps/mcp/src/validate/{wording.rs, tree.rs, mod.rs}`
- `libs/rules/src/engine/refusal.rs`
- tests
- `.scratch/wotr-planner/issues/330-older-texts-the-308-qa-found.md` (item 1's status)

## Checks

As for 336a.
