# The engine words a refusal: the picker side

Type: build (engine, MCP, web app)
Status: open
Blocked by: 199 (another session's ticket: it must have landed what design-R §3 lists), 335
Model: Opus to build.
Map: [WotR Build Planner](../map.md)
Design: [design-R.md, T5a](../research/architecture/design-R.md), in the frame of [overall.md](../research/architecture/overall.md); decisions 1 and 2 in [decisions.md](../research/architecture/decisions.md).

## Problem

The reason for a refused option is built three ways:
- the block at `options.rs:984–998` and `choose.rs:231–251` is duplicated;
- MCP's `Breaks` names choices inside the tool crate;
- the MCP entry line in a rank's slot lacks the 171 hint that the web app gives.

## Change

- Add `Refusals`, `Refusal`, `Breaks` and `RankMove` in `rules` (design-R §1), plus `slot_entries_in` on `Source::Picker`.
- The MCP `options`/`choose` adapter becomes `refusal_reason`.
- The `withdraw` adapter calls `breaks`.
- The web app's older pickers read `refusal.text`.
- Rules names no tool and no address (CODING_STANDARDS:9).

## Done when

- The duplicated block is one adapter call.
- `issue_reason`, MCP `Breaks` and `Cause` are gone.
- `Offer` carries `refusal`.
- The entry line in a rank's slot gains the 171 hint (decision 1, item 4), pinned by an exact-text test.
- The reasons record shows no other change, and `FORM_PROOF_OFFERS=1` shows no status moved.

## Files

- `libs/rules/src/engine/{refusal.rs (new), pickers.rs}`, `libs/rules/src/{engine.rs, lib.rs}`
- `apps/mcp/src/{options.rs, choose.rs, withdraw.rs, gear.rs}`
- `libs/ui/src/picker.rs`
- `libs/rules/tests/engine/{refusals.rs (new), main.rs}`
- MCP tests

## Checks

Design-R §3 "Checks (both)":
- `cargo test -p rules -p mcp -p ui -p frontend -p seeds -p corpus`
- the rules wasm build
- the frontend's wasm clippy
- `scripts/css-classes.sh`
- `cargo fmt --all --check`
- `scripts/test-modules.sh`
- the form proof with `FORM_PROOF_OFFERS=1 FORM_PROOF_REASONS=1`
