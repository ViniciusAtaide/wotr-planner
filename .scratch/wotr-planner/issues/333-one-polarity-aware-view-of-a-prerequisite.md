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

**Decisions** (the main session's, in the brief and `decisions.md` 4): `needs_unheld` stays on the raw tree and game-check's `canon` stays whole. Mine: `Reading` turns only `AtLeast(1)` into an `Any`, not `AtLeast(0)`; `carrier_guard` loses its `AtLeast(0 | 1)` arm. The data check refuses an `AtLeast(0)` with children (`DataIssue::AtLeastOutOfRange`) and the compiled trees use `max(1)` (`component.rs`), so no checked tree holds one; the byte-identical report and dry run agree. Were one there, `carrier_guard` would descend it as it does any `AtLeast(n >= 2)`, widening no alternatives, where the old arm widened. A refused `AtLeast` reads as the failures it needs: fewer than `needed` hold, so at least `len - needed + 1` fail, and when `needed` exceeds the children (it can never hold) the refusal always holds, an empty `All`. `TryFrom`'s error is `&DataAt`, not `()`.

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

### Review fixes

A second commit, "Ticket 333: review fixes", answering the review's findings (all the reviewer's; the code choices below are mine).

- **A refused `AtLeast`** (`Prerequisite::read`): it computed `len.saturating_sub(needed) + 1`, so `Not(AtLeast(3, [a, b]))` read as needing one of `a`, `b` to fail, though it always holds. It now takes `len + 1 - needed` with `checked_sub`, and when that is none (or less) reads as `All([])`, which always holds. The comment says the rule: fewer than `needed` hold, so at least `len - needed + 1` fail.
- **`Repeat::derive`** (`parameter.rs`): the commit's version returned at the first refused part that matched, so `Not(Any([SelectionOption(own), Rank{own, 3}]))` gave `Times(3)` where the base gave `Once`. It now looks for the refused own leaf over every refused part first, then for a refused own rank. The gain of the commit stays: a refused own rank inside a refused `Any` with another condition (`Not(Any([Rank{own, 3}, other]))`) is `Times(3)`, where the base's walk saw no refused rank. One shape now differs from the base: in `All([Not(Rank{own, 3}), Not(own)])` the base took the first refusal (`Times(3)`) and this takes the leaf (`Once`), whatever the order. No data holds it: the form proof below shows no offer changed.
- **No wildcard:** `condition_names` (`check.rs`) and `Prerequisite::fact` name every `Prerequisite` variant, so a new one fails to compile until it is placed. The old comment ("a group, a `Not`, a number or a fact") left out the states of the Build (alignment, deity, caster type, story and the rest) that the wildcard also took.
- **Tests that claim what they check:** the round-trip lists (`fact_leaf_round_trips_for_every_kind`, and `fact_and_data_at_convert_both_ways` through a shared `every_fact`) number each sample through an exhaustive `Fact` match; `each_fact_leaf_names_its_fact` adds `FeatFor`, `FeatIfValue`, a `Rank` over an option, and the deliberate `None`s `ClassSpellLevel` and `PickedThrough`; `a_leaf_says_its_threshold_and_value` adds `Feature.count`, `AbilityScore`, `FullAbilityScore`, `ClassSpellLevel`, `SpellLevel` and `SpontaneousSpellLevel`; `a_reading_of_a_not_inside_an_all_merges_into_it` asserts the leaves' order and polarity.
- **`AtLeast(0)`:** the stronger reason is in Decisions above. The data check refuses one with children (`AtLeastOutOfRange`, `check.rs`) and the compiled trees use `max(1)`, so `Reading` need not read it as an `Any`. A hypothetical one would reach `carrier_guard::walk` as `Reading::AtLeast(0, ..)` and go down the `All` arm: each child required with only the outer alternatives, where the old `AtLeast(0 | 1)` arm let each stand in for the others. It could then report a gap the base would not have.

**New tests:** `prerequisites::a_refused_at_least_reads_as_the_failures_it_needs` (`Not(AtLeast(3, [a, b]))` and `Not(AtLeast(1, []))` are `All([])`; `Not(AtLeast(2, [a, b, c]))` is `AtLeast(2, [¬a, ¬b, ¬c])`; `Not(AtLeast(3, [a, b, c]))` is `Any([¬a, ¬b, ¬c])`), `prerequisites::de_morgan_reads_nested_and_empty_groups` (`Not(All([a, Any([b, c])]))` is `Any([¬a, All([¬b, ¬c])])`, `Not(All([]))` is `Any([])`, `Not(Any([]))` is `All([])`, `Not(Any([Not(a), b]))` is `All([a, ¬b])`), and the unit tests `parameter::tests::a_refused_own_leaf_beside_a_refused_own_rank_is_once` and `parameter::tests::a_refused_own_rank_inside_a_refused_or_is_times`. The expected values are logic over the tree's shape (De Morgan, and the base's `Repeat`); no game fact changed, so no game citation.

**Checks** (after the last source edit)
- `cargo test -p rules -p seeds -p mcp -p game-check`: every binary ok (rules engine 792 + 1 ignored, rules unit 19, game-check 172, mcp 251, seeds game_data 654 + 90 ignored); the four new names print in `cargo test -p rules -- --list`.
- rules wasm build, `check-data`, `cargo fmt --all --check` and `scripts/test-modules.sh`: pass.
- Game check: exit 0, its report `cmp` byte-identical to the base's.
- Form proof `compare` against the base record, with `FORM_PROOF_REASONS=1`: `2157 Builds compared (1992 open), 2157 old links reopened, 0 differences, 0 offer differences, 0 reason problems`; `links: 1992 the same, 0 through an heir, 0 refused, 0 changed`; `offers: 18213559 options recorded, 18213559 now`, `0 statuses changed`; `reasons: 15217937 compared, 0 changed in 0 edits`; `failures: 0 appeared, 0 gone`.
