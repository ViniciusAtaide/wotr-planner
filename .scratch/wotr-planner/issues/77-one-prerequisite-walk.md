# One prerequisite tree walk

Type: task
Status: open
Blocked by: [75](75-game-data-master.md)
Map: [WotR Build Planner](../map.md)

## Question

The `All | Any | AtLeast | Not` recursion over `Prerequisite` is hand-written about 19 times (architecture review, 2026-11-03):
- **`rules`:** five times in `libs/rules/src/prerequisite.rs` (lines 135–249).
- **`extract`:** `option_follow_ups.rs:63`, `wild_talents.rs:374`, `borrowed_selections.rs:847`, `borrowed_extra_picks.rs:224`, `shared_options.rs:375, 398`, `parameters.rs:687`, `references.rs:75`, `mercies_and_exploits.rs:215`. Re-check the list after ticket 75; passes it deletes need no change.
- **Copied helpers:** `extract/src/archetype_alignments.rs:128, 143` repeat `Prerequisite::is_alignment` and `names_alignment`, which are `pub(crate)` in `rules`.

Two of the walks skip `Not`: `parameters.rs:687` (`replace_leaf`) and `shared_options.rs:374` (`point_at_originals`) recurse into `All`, `Any` and `AtLeast` only, so a leaf under a `Not` is never replaced or repointed. That may be a latent bug.

## Build

- **Add the walks to `Prerequisite`** in `rules` (for example `leaves()`, `leaves_mut()`, `any_leaf()`; pick the names), with tests, and make `is_alignment` and `names_alignment` `pub`.
- **Use them** at every site above and delete the copies. Where a site does more than walk (`mercies_and_exploits.rs`' `replace_leaf` joins `Any` children), keep that logic and walk with the helper.
- **Keep the `Not` skips as they are** in this ticket: a walk that skips `Not` gets a helper or a filter that skips it too. Then check the data: does any `Not` in `libs/game-data/` hold a leaf those two walks would have touched? Bring the answer back as a question; change nothing for it.

**Done when:**
- No hand-written recursion over `Prerequisite`'s variants remains outside the new helpers (`match`es that do one level are fine).
- `scripts/extraction-diff.sh <out-dir>` against `HEAD` reports identical.
- `scripts/proof.sh rules <out-dir>` against `HEAD` reports no difference.
- `scripts/check.sh` passes.
- The Answer says what the `Not` check found.
