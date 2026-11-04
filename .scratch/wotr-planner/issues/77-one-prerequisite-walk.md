# One prerequisite tree walk

Type: task
Status: open
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
