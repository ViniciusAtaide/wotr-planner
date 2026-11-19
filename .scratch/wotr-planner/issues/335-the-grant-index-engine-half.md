# The grant index: its Engine half, and an Extra feat's slot under the feat

Type: build (engine, MCP)
Status: open
Blocked by: 333, 334
Model: Opus to build.
Map: [WotR Build Planner](../map.md)
Design: [design-G.md, T4](../research/architecture/design-G.md), in the frame of [overall.md](../research/architecture/overall.md); decisions 5 in [decisions.md](../research/architecture/decisions.md).

## Problem

- `validate` places an Extra feat's slot under its level's class. A refused Extra feat's slot pick is therefore a separate root, labelled "Level 3: slot" (330 item 2).
- `pick_gives` omits a feat's `selection`, an ability's features and option rows' chosen feats.
- `held_holder`, `within_reach` and `is_named_by` each re-scan rows.

## Change

Add design-G §1's Engine half: `grants()`, `pick_gives`, `grantor_in`, `in_reach`, `opened_by`. Make `labels.rs`, MCP `tree.rs`, `wording.rs` and `declare.rs` read it (design-G §2's T4 rows). Delete `GrantedBy`. `grantor_in` and `in_reach` keep today's row-only answers (decision 5).

## Done when

- **330 item 2 is fixed.** Take Instinctual Warrior 3 with `extra-rage-power` refused and a pick at `slot.feat-selection.3.0`.
  - `validate` nests that slot's line under the feat's line, with no root of its own.
  - The label names the selection the feat would open, in `placed_label`'s style, pinned by an exact-text test.
  - 330 item 2 is marked done in ticket 330.
- The MCP `validate` tests pass unchanged.
- The form proof (`FORM_PROOF_OFFERS=1 FORM_PROOF_REASONS=1`) shows statuses and reasons unchanged.

## Files

- `libs/rules/src/engine/{grants.rs (new), labels.rs, catalog.rs}`, `libs/rules/src/engine.rs`
- `apps/mcp/src/validate/{tree.rs, wording.rs, declare.rs}`
- `apps/mcp/tests/server/validate.rs`
- `libs/rules/tests/engine/grants.rs`
- `libs/seeds/tests/game_data/grant_index.rs`
- `.scratch/wotr-planner/issues/330-older-texts-the-308-qa-found.md` (item 2's status)

## Checks

- `cargo test -p rules -p mcp -p seeds -p corpus`
- the rules wasm build
- `cargo fmt --all --check`
- the form proof as above
