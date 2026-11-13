# `validate` refuses a Mythic rank at its earliest level

Type: bugfix
Status: resolved
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

Found by a QA session through the MCP server (2026-11-13), still so on `main` at `ce9cc3b`. A sorcerer 14 declared with `"mythic.4.level": 11, "mythic.5.level": 14` (each rank's table level) is `invalid`, each with `not_allowed`: "MR 4 comes at level 11 at the earliest (the rank table's level): `mythic.4.level` takes a later level, up to the Build's last. To take the rank at level 11, leave `mythic.4.level` out." (`apps/mcp/src/validate/wording.rs`).

But `options` on `mythic.4.level` lists "Level 11 (the earliest)" with status ok, `choose` takes 11 as a no-op (as it takes any choice's current value), and the tool's description says `validate` takes each value "as `choose` takes them". An agent that copies `options`' values into its map gets a refusal for a value it was offered.

## Build

- `validate` takes `mythic.R.level` equal to the level the rank comes at as a no-op, as `choose` does: the table's level, or the rank below's later level when that holds it (`options` then lists "Level 15 (the earliest)" for MR 4 with MR 3 at 15).
- A level below that stays refused, with the reason it has now.

**Not part of this ticket:** `validate` accepting ranks declared out of order (`{3: 15, 4: 13}`) is ticket [171](171-mythic-picks-after-the-levels.md)'s dormant level, by design.

## Done when

The map above is `valid` and gives the same code as leaving both out; a test pins it beside 171's; `scripts/check.sh` passes.

## Answer

Done 2026-11-13 (branch `ticket/191-validate-rank-earliest-level`, from `main` at `17ddb44`). No rule changes: only what `libs/rules` shows of its own state became public.

### What was built

- **`apps/mcp/src/validate/declare.rs`:** `Declared::rank_levels`, run after the levels are read. In rank order it removes each declared rank level and puts it back unless it equals where the rank comes without it (`Engine::rank_floor`: the rank table's level, or the rank below's when that is later) and the Build reaches the rank (`Engine::rank_reached`). A level equal to where the rank comes is so the current value, a no-op, as in `choose`: the map gives the Build, link and code of the map without it. Ranks in order matter: `{3: 15, 4: 15}` reads as `{3: 15}`, which is what `choose` stores.
- **`libs/rules` (two visibility/helper changes, no rule changed, no link opens differently):** `Engine::rank_floor` (`engine/switch.rs`) is now `pub`; `Engine::rank_reached` (`engine/ranks.rs`) is new: the Build has the rank's level and no Path switch ended the ascensions before it. The walk's refusal (`walk.rs`, `check_unreached`) is untouched.
- **`apps/mcp/src/validate/wording.rs`:** the `not_allowed` text for `mythic.R.level` names the level the rank comes at and why: "MR 4 comes at level 13 at the earliest (MR 3 is taken at level 13)" when the rank below holds it later, "(the rank table's level)" otherwise; it says the level itself is taken.

### Decisions (the main session, 2026-11-13, under the user's "answer the easy ones")

1. Wording when the rank below holds the rank later: name that level and the rank below (option b); the plain table case keeps its phrase.
2. A level equal to the earliest for a rank the Build does not reach stays refused, as `choose` and `options` refuse it: too few levels, and a Legend switch that ended the ascensions before the rank (`rank_reached` tells both).
3. The floor is `Engine::rank_floor`, made public.
4. A rank's own level under a later rank below (171's dormant level, `{3: 13, 4: 12}`) behaves as before, and is pinned.
5. No edit to `validate`'s description or the `wotr-build` skill: left as they are.

### Tests (`apps/mcp/tests/server/rank_levels.rs`)

- Changed: `validate_takes_a_ranks_level_and_says_why_one_is_none` no longer expects the table's level to be refused; level 10 is.
- `validate_takes_the_level_a_rank_comes_at_as_no_change`: `{4: 11, 5: 14}` on 14 levels gives the same `build` and `link` as leaving both out.
- `validate_takes_the_level_the_rank_below_holds_a_rank_at_as_no_change`: `{3: 13, 4: 13}` is `{3: 13}`, as `choose` makes it.
- `validate_still_refuses_a_level_before_where_a_rank_comes`: level 10, and `{3: 13, 4: 11}` with the new reason.
- `validate_keeps_a_level_a_rank_has_of_its_own_under_a_later_rank_below`: the dormant level `{3: 13, 4: 12}` is kept.
- `validate_refuses_the_earliest_level_of_a_rank_the_build_does_not_reach`: too few levels, and MR 9 after a Legend switch.
- `validate_takes_every_level_options_offers_for_a_rank`: `validate` takes each value `options` lists for MR 3 and 4.

### Checked

The tests were seen to fail before the change. `scripts/check.sh`: ALL-OK (fmt, clippy, wasm-clippy, test with the snapshot check showing no difference, rules-wasm, data-check). `cargo fmt --all --check` passes. Not run, by the queue's rule: `scripts/check.sh --full` and `scripts/proof.sh`.
