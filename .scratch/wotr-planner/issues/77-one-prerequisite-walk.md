# One prerequisite tree walk

Type: task
Status: resolved
Blocked by: [82](82-remove-the-extractor.md)
Map: [WotR Build Planner](../map.md)

## Question

The `All | Any | AtLeast | Not` recursion over `Prerequisite` is hand-written several times in `rules` (architecture review, 2026-11-03): five times in `libs/rules/src/prerequisite.rs` (`collect_stories`, `collect_feats`, `is_alignment`, `names_alignment`, `canonicalize`, lines 135–249) and again in `libs/rules/src/check.rs` (around 708). The review found about as many copies in `libs/extract`; [ticket 82](82-remove-the-extractor.md) deleted them with the crate (re-scoped after [ticket 75](75-game-data-master.md)).

## Build

- **Add the walks to `Prerequisite`** (for example `leaves()`, `leaves_mut()`, `any_leaf()`; pick the names), with tests.
- **Use them** wherever a function only walks to the leaves, in `rules` and any other crate; delete the copies. A function that does more at each level (`canonicalize` sorting children, the prerequisite text in `engine/labels.rs`) keeps its own `match` and uses a helper only where it fits.

**Done when:**
- No function outside the new helpers recurses over all of `Prerequisite`'s branches just to reach the leaves.
- `scripts/proof.sh rules <out-dir>` against `HEAD` reports no difference.
- `scripts/check.sh` passes.

## Progress log

Scratch dir (outside the repo): `/private/tmp/claude-501/-Users-thaynamarcela-code/20f0bb4f-7437-4fe9-8ef8-ebb8716f6fbe/scratchpad/t77/` (below, `t77/`).

- 2026-11-04: re-found every recursion over `Prerequisite`'s groups at `HEAD` (86a3cb3). In `rules`: `prerequisite.rs`'s `collect_stories`, `collect_feats`, `is_alignment`, `names_alignment`, `canonicalize`; `check.rs`'s `Checker::prerequisite` (656–725); `engine/labels.rs`'s `prerequisite_text`; `state.rs`'s `check_in` (the evaluator, which the Question doesn't list). In `seeds`: `src/prerequisites.rs` (storing and reading the node rows). In tests: `seeds/tests/game_data/` `alignment.rs` (`names_alignment`, a copy of the crate-private one), `class_skills.rs` (`prerequisite_features`), `parameterised_feats.rs` (`has`), `negated_prerequisites.rs` (`negations`); `rules/tests/engine/canonical.rs` (`reverse_prerequisite`). `apps/` names no `Prerequisite`.
- 2026-11-04: decision (small, settled here): one helper, `Prerequisite::leaves(&self) -> impl Iterator<Item = &Prerequisite>`, public (the seeds tests use it). It goes through every group and every `Not`, as all the leaf walks did, and yields only leaves, left to right. No `leaves_mut` and no `any_leaf`: no site needs a mutable leaf walk (`canonicalize` and `reverse_prerequisite` change the groups, not the leaves), and `any_leaf(f)` is `leaves().any(f)`. Seam: the method, tested in `libs/rules/tests/engine/prerequisites.rs`.
- 2026-11-04: red → green: `a_single_condition_is_its_own_only_leaf` (failed to compile, then `once(self)`), `the_leaves_of_nested_groups_come_left_to_right` (failed, then the stack walk over `All`/`Any`/`AtLeast`), `the_leaves_under_a_negation_are_the_trees_leaves_too` (failed, then `Not`), `an_empty_group_has_no_leaves` (passed at once: it pins the edge that `names_alignment` and `stories` rely on). 360 engine tests pass.
- 2026-11-04: used `leaves` in `stories` (`collect_stories` deleted), `collect_feats` and `names_alignment`, and in the three seeds tests. `parameterised_feats.rs`'s `has` didn't descend into a `Not` and compared groups too; with `leaves` it finds a leaf under a `Not` as well. Every `has` assertion is positive and every value it looks for is a leaf, so each one it passed before it passes now.
- 2026-11-04: `scripts/check.sh` (with `wotr-planner-postgres` up): fmt, clippy, wasm-clippy, test, rules-wasm, seed-check OK, ALL-OK (log `t77/check.log`).
- 2026-11-04: `scripts/proof.sh rules t77/proof` against `HEAD` (86a3cb3): 1999 Builds each side, 0 only in A, 0 only in B, 0 differing; `compare.txt` empty (log `t77/proof.log`). The proof needed no change.

## Answer

One leaf walk, `Prerequisite::leaves()`, and every function that only walked to the leaves now uses it.

- **The helper** (`libs/rules/src/prerequisite.rs`): `leaves(&self) -> impl Iterator<Item = &Prerequisite>`, the single conditions under every group and `Not`, left to right, never a group or a `Not` itself. A stack walk, so no recursion. Tests in `libs/rules/tests/engine/prerequisites.rs`: a single condition, nested `Any`/`All`/`AtLeast` in order, through `Not` (a double one too), an empty group.
- **Now using it:**
  - `Prerequisite::stories` (its `collect_stories` deleted), `collect_feats`, `names_alignment` (`rules`).
  - `seeds` tests: `alignment.rs`'s `names_alignment`, `class_skills.rs`'s `prerequisite_features` (its inner `collect` deleted), `parameterised_feats.rs`'s `has`.
- **Kept their own `match`** (each does more at a level than reach the leaves):
  - `is_alignment`: an empty group anywhere makes it false, which the leaves can't show (commented there).
  - `canonicalize`: sorts each group's children.
  - `check.rs`'s `Checker::prerequisite`: checks each group (empty, `AtLeast` out of range, a story among alternatives) and each `Not`'s child.
  - `labels.rs`'s `prerequisite_text`: the text of each level, brackets and "n of:".
  - `state.rs`'s `check_in`: evaluates each group into its `Unmet`.
  - `seeds/src/prerequisites.rs`: stores and reads every node, groups included.
  - Tests: `negated_prerequisites.rs`'s `negations` collects the `Not` nodes themselves (and stops there); `canonical.rs`'s `reverse_prerequisite` reverses each group's children.
- **Checks:** `scripts/proof.sh rules` against `HEAD`: no difference (1999 Builds, 0 differing, empty `compare.txt`). `scripts/check.sh`: ALL-OK.
