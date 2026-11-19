# One polarity-aware view of a prerequisite, with each leaf's Fact

Type: build (engine)
Status: resolved
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

## Answer

Built test-first (the new API did not exist, so the tests failed to compile before the code) on `main` at 76bb211d. Form-only: nothing a user or the game check sees changed.

**Built**
- `libs/rules/src/prerequisite.rs`: `Leaf { condition, refused }` with `fact`, `threshold`, `value`; `Reading` (`All`/`Any`/`AtLeast`/`Leaf`, De Morgan pushed onto the leaves, a pushed group merged into a parent of its kind, an `AtLeast(1, ..)` read as an `Any`) with `parts` and `leaves`; `Prerequisite::reading`, `polar_leaves`, `fact`; `leaves` is now `polar_leaves().map(..)`. `holds_not` (crate-private) answers "is there a `Not` anywhere" for `unmet_negates`, which must see a `Not` over an empty group that has no leaf.
- `Unmet::leaves` (choice.rs). `From<&Fact> for DataAt` and `TryFrom<&DataAt> for Fact` (component.rs; the error is the entry itself).
- Callers moved: `Repeat::derive` (reads `reading().parts()`), `condition_names` (one `fact()` call, then only the value and the four non-fact arms), `ranks.rs` (`unmet_leaves`/`prerequisite_leaves` deleted; `unmet_negates` shrunk), MCP `Node::needs` (`leaves` deleted), game-check `Named::of`, `carrier_guard::walk` (reads a `Reading`), and both `fact_at`s and `leaf_at` deleted.
- `libs/rules/src/lib.rs`: one line, the export `pub use prerequisite::{Leaf, Prerequisite, Reading, Story};`.

**Decisions** (the main session's, in the brief and `decisions.md` 4): `needs_unheld` stays on the raw tree and game-check's `canon` stays whole. Mine, to keep the form proof at zero: `Reading` turns only `AtLeast(1)` into an `Any`, not `AtLeast(0)`; `carrier_guard` loses its `AtLeast(0 | 1)` arm (no data holds an `AtLeast(0)`, and the byte-identical report and dry run show it). `TryFrom`'s error is `&DataAt`, not `()`.

**Tests** (all in `libs/rules/tests/engine/prerequisites.rs` unless said): `a_leaf_under_one_not_is_refused_and_under_two_is_needed`, `the_reading_pushes_a_not_onto_its_leaves`, `a_reading_of_a_not_inside_an_all_merges_into_it`, `each_fact_leaf_names_its_fact`, `a_leaf_says_its_threshold_and_value`, `the_unmet_leaves_carry_polarity`, `fact_and_data_at_convert_both_ways`, and the unit test `prerequisite::tests::fact_leaf_round_trips_for_every_kind` (`Fact::leaf` is crate-private). Facts' expected values come from the data's own shape: each kind's `Fact` is the one `Fact::leaf` builds, the inverse of `Prerequisite::fact` (the old `leaf_at`, exactly); no game fact changed, so no new game citation.

**Grep** (`grep -rnE "Prerequisite::Not|P::Not" --include='*.rs' libs apps tools`, minus tests, `test_world`, `prerequisite.rs`): what remains decides no polarity.
- `state.rs:704` evaluates `Not`.
- `check.rs:1228` recurses into it for the data check.
- `labels.rs:90, 905, 1032, 1076` word it.
- `component.rs` constructs it.
- `game-check/src/areas/prerequisites.rs:698` is `planner_side` building `Node::Not` (decision 4: `canon` stays).

`leaf_at`, `fact_at`, `unmet_leaves`, `prerequisite_leaves` and MCP `leaves` have no match anywhere.

**Checks** (after the last edit)
- `cargo test -p rules -p seeds -p corpus -p mcp -p game-check`: every binary ok (rules engine 790, game-check 172, seeds game_data 654 + 90 ignored, mcp 251).
- rules wasm build, `check-data`, `cargo fmt --all --check` and `scripts/test-modules.sh`: pass.
- Form proof, base recorded in this lane before the first edit, `compare` with `FORM_PROOF_OFFERS=1 FORM_PROOF_REASONS=1`: `2157 Builds compared (1992 open), 2157 old links reopened, 0 differences, 0 offer differences, 0 reason problems`; `offers: 18213559 options recorded, 18213559 now`, `0 statuses changed`; `reasons: 15217937 compared, 0 changed in 0 edits`; `failures: 0 appeared, 0 gone`.
- Game check's report (exit 0) and `--write prerequisites --dry-run`'s output: `cmp` byte-identical to the base's.

**Known difference, not exercised by the data** (design-P risk 5): `Not(Any([Not(a), b]))` now reads as needing `a` in `ranks.rs`, MCP and `carrier_guard`; no `.ron` holds such a shape.
