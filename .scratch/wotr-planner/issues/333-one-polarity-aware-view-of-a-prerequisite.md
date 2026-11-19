# One polarity-aware view of a prerequisite, with each leaf's Fact

Type: build (engine)
Status: open
Blocked by: 332
Model: Sonnet to build, Opus to review.
Map: [WotR Build Planner](../map.md)
Design: [design-P.md, T2](../research/architecture/design-P.md), in the frame of [overall.md](../research/architecture/overall.md); decisions in [decisions.md](../research/architecture/decisions.md).

## Problem

Five walkers each decide polarity and the leaf-to-entry mapping on their own:
- `Named::walk` toggles it;
- `ranks.rs` and `carrier_guard` drop a `Not` subtree;
- MCP returns the `Not` node itself;
- `Repeat::derive` matches `Not(Any)`.

There are four leaf↔entry tables: `leaf_at`, `condition_names` and the two `fact_at`s.

## Change

Add `Leaf`, `Reading`, `polar_leaves`, `reading`, `fact`, `Unmet::leaves`, and the `From<&Fact> for DataAt` / `TryFrom<&DataAt> for Fact` pair (design-P §1). Move the callers listed in design-P §2 "T2". `needs_unheld` and game-check's `canon` stay as they are (decision 4).

## Done when

- `leaf_at`, both `fact_at`s, `unmet_leaves`/`prerequisite_leaves` and MCP `leaves` are gone. No function outside `prerequisite.rs` matches `Prerequisite::Not` to decide polarity (shown by a grep in the Answer).
- The form proof (`FORM_PROOF_OFFERS=1 FORM_PROOF_REASONS=1`) shows 0 differences.
- The game check's report is byte-identical.
- `cargo test -p game-check`'s writer fixtures pass unchanged.

## Files

- `libs/rules/src/{prerequisite.rs, choice.rs, component.rs, check.rs, parameter.rs}`
- `libs/rules/src/engine/ranks.rs`
- `apps/mcp/src/validate/tree.rs`
- `tools/game-check/src/write/{prerequisites.rs, carrier_guard.rs, carriers.rs}`
- `libs/rules/tests/engine/prerequisites.rs`

## Checks

- `cargo test -p rules -p seeds -p corpus -p mcp -p game-check`
- the rules wasm build
- `cargo run -p seeds --bin check-data`
- the game check
- the form proof (form-only)
- `cargo fmt --all --check`
