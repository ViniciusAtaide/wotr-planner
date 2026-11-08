# The Path switch's rules sit beside its data

Type: task
Status: resolved
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

## Choices settled

1. **`MythicPath::row(rank) -> Option<&MythicRankRow>`** in `game_data.rs` is the one rank→row lookup. Its four sites call it: the walk's row (`walk/ascension.rs`), `first_ascension_option` (`switch.rs`, `.row(FIRST_ASCENSION_RANK).is_some()`), `path_rank_features` (`labels.rs`), and `first_ascension_name`'s `row_names`, which now maps `path_rank_features(path, FIRST_ASCENSION_RANK)` to names instead of re-doing it.
2. **`PathSwitch::ends_ascensions_at() -> Option<u8>`**: `Some(PATH_SWITCH_RANK)` without further ascensions, else `None`. `Engine::apply`'s retain drops a rank's picks when `ends_ascensions_at().is_some_and(|end| rank >= end)`, and the walk's `switch_path` ends the ascensions on the same test at its rank, so both read one rule. The mirror stays on purpose (the ticket's reason: `apply` runs in every `Recheck`); the walk is still the authority (`check_unreached`). It reads `crate::PATH_SWITCH_RANK` from `game_data.rs`, as `rank_after_switch` already sits there beside its data.
3. **One cut-off helper: `PathSwitch::keeps(rank, from_path) -> bool`**, `rank <= kept_through_rank` for a general pick and `rank <= path_kept_through_rank` for the old path's grant. The rank sheet's `general_lost`/`path_lost` are `!keeps(rank, false)`/`!keeps(rank, true)`. A `bool` rather than a new enum, matching `GrantSource::from_path`, which every caller already holds.
4. **`PathSwitch::drops(rank, from_path, feature, selection) -> bool`** replaces `switch_drops`: past the cut-off (`!keeps`), or, for the old path's grant only, a `feature` in `dropped_features` or a pick in a `selection` in `dropped_selections`. As before, a general pick's grant goes by the general cut-off alone, whatever it names.
5. **The walker keeps the mechanics.** A private `GrantSource::dropped_by(switch, feature)` in `walk/ascension.rs` passes a source's rank, `from_path` and selection to `drops`; the `rank_grants` partition passes the feature when the grant is one, the `holdings` partition passes none (an option is never a feature), so it no longer builds a throwaway `RankGrant` with cloned keys. Partition, release, option levels and the rank sheet stay in the walker.
6. **Tests in a new file, `libs/rules/tests/engine/switch_rules.rs`**, so the existing switch tests stay byte-for-byte unchanged. They read the small world's Trickster, Legend, Gold Dragon, Swarm That Walks and Devil, and their expected values come from the glossary's Path switch entry (Legend keeps MR 1–2 of both and ends the ascensions at MR 8; Gold Dragon keeps general picks of MR 1–7, path grants of MR 1–2; Devil keeps MR 1–7 but drops the Aeon's gaze by name).

## Progress log

- 2026-11-08: started at HEAD 8443e4a, clean tree, in a worktree (ticket 100 runs alongside in another). Read `CLAUDE.md`, the map's Notes, `GLOSSARY.md` and this ticket; copied the dev database in.
- 2026-11-08: test-first, one slice at a time, in `switch_rules.rs`: `MythicPath::row` (red: no method `row`; green), `ends_ascensions_at` (red: no method; green), `keeps` (red: no method; green), `drops` (red: no method; green). 4 passed.
- 2026-11-08: moved the four lookups and the two cut-offs onto the new methods; removed `switch_drops`. `cargo test -p rules`: 479 + 6 passed, the existing switch tests unchanged.
- 2026-11-08: rules proof against 8443e4a (`scripts/proof.sh rules … 8443e4a`, its own `CARGO_TARGET_DIR` under `$TMPDIR`, deleted after), 2,162 Builds (PathSwitch checked 379 times): `A: 2162 Builds, B: 2162; only in A: 0, only in B: 0, differing: 0`.
- 2026-11-08: `scripts/check.sh`: ALL-OK.

## Answer

Done 2026-11-08, no behaviour change. The switch's rules sit on `PathSwitch` beside its data: `ends_ascensions_at` (read by `Engine::apply` and the walk), `keeps` (the one cut-off comparison) and `drops` (what a switch takes away, formerly the walker's `switch_drops`). `MythicPath::row` is the one rank→row lookup, at the four sites, and `first_ascension_name` reads `path_rank_features`. The walker keeps the partition, the release and the rank sheet. New tests: `libs/rules/tests/engine/switch_rules.rs` (4). Proof: no behaviour difference over 2,162 Builds.
