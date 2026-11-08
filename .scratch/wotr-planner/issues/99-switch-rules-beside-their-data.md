# The Path switch's rules sit beside its data

Type: task
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The architecture review of 2026-11-07 rated "a mythic track module" (candidate 5) Speculative. A feasibility check (2026-11-08) found the full module not worth it: the switch's mid-walk release ties the mechanics (`switch_path` partitioning `rank_grants` and `holdings`, `GrantSource` threaded through `hold()`) to the walker, so a separate module would only move code. Three of the "seven files" hold only a lookup or a constant. The switch rules haven't changed since tickets 26 and 40–42 (2026-11-01/02).

What's left is small and worth tidying, with no behaviour change:

- **The rank→row lookup** (`ranks.iter().find(|row| row.rank == …)`) is written at four sites: `engine/walk/ascension.rs` ~120, `engine/switch.rs` ~22–26 (`.any`), `engine/labels.rs` ~342–344 and ~391. In `labels.rs`, `first_ascension_name`'s `row_names` closure re-does `path_rank_features`.
- **"A switch without further ascensions ends MR 8+"** is written twice: `Engine::apply` (`switch.rs` ~73–81) drops picks at `PATH_SWITCH_RANK` and up, and the walk sets `ascensions_over` (`ascension.rs` ~237–242). The mirror is on purpose, because `apply` runs inside every `Recheck` and asking the walk would cost a walk per option. The walk stays the authority (`check_unreached`).
- **What a switch drops:** `switch_drops` (`ascension.rs` ~16–27) and the RankStatus code (~204–205) both compare against `kept_through_rank` and `path_kept_through_rank`.

The user said to build it if it's feasible (2026-11-08); this is the feasible part.

## Build

- **(a)** `MythicPath::row(rank) -> Option<&…>` in `libs/rules/src/game_data.rs`, used at the four sites; `first_ascension_name` calls `path_rank_features`. Add `PathSwitch::ends_ascensions_at() -> Option<u8>` (`PATH_SWITCH_RANK` when `!further_ascensions`), used by both `apply`'s retain and the walk's `switch_path`.
- **(c)** The switch's predicates move onto `PathSwitch`: `switch_drops` becomes a `PathSwitch` method (for example `drops(rank, from_path, feature, selection)`), beside one helper both cut-off comparisons use. The walker keeps the mechanics (partition, release, sheet); the rule sits beside its data and gets unit tests without a walk.
- Test-first (`tdd`) for the new `PathSwitch`/`MythicPath` methods in `libs/rules/tests/engine/` (e.g. `path_rules.rs`); the existing switch tests (`path_rules.rs`, `mythic.rs`, `first_ascension.rs`, and the real-data `libs/seeds/tests/game_data/mythic.rs` and `first_ascension.rs`) pass unchanged.

**Done when:**
- The new tests pass and the existing switch tests pass unchanged; the four row lookups and the two cut-off comparisons each have one home.
- `scripts/proof.sh rules` against the parent commit shows no behaviour difference.
- `scripts/check.sh` passes.
