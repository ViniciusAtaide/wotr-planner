# GameData::check validates references

Type: task
Status: open
Blocked by: none (runs alongside [97](97-slot-options-presented.md), in its own worktree)
Map: [WotR Build Planner](../map.md)

## Question

The architecture review of 2026-11-07 (candidate 4) found referential integrity split between `GameData::check` and SQLite's foreign keys:

- `libs/rules/src/check.rs` says so at its top and in five places (670, 721, 942, 1089, 1142: "an unknown X is the foreign keys' business"), and `check_game_data_files` (`libs/seeds/src/lib.rs` ~101) says only `seed` catches dangling keys.
- So in-memory data is never checked for them: the hand-built small world in `libs/rules/tests/engine/support/` passes `check()` in only a few tests (`removable.rs:467`, `parameters.rs:404`, `alignment.rs:97`), and a misspelled key there silently offers nothing.
- The engine carries tolerances for missing keys (a missing feat list "accepts none", a missing shared option "offers nothing"): `engine/catalog.rs` ~291, `engine/slots.rs` ~159 and ~175, `engine/pickers.rs` ~61, `engine/switch.rs` ~99 (line numbers before tickets 95–96; search for them).

The user chose to build it (2026-11-08) without a design round, keeping their ticket 75 choice: the database still checks foreign keys. Settle small choices yourself, record them, and bring a real open question back.

## Build

- **`check` validates every reference:** each key a data item names (feats, feat lists, classes, archetypes, selections and options, shared options, features, weapons, skills, deities, domains, mythic paths, backgrounds, races, …) resolves to an item of the right kind, reported as a problem naming the item and the missing key. The SQLite foreign keys stay as they are, as a backstop.
- **The comments go:** the "foreign keys' business" notes in `check.rs` and `check_game_data_files`'s doc say what's true now.
- **The small world is guarded:** one test asserts `world().check() == []` (and any other hand-built fixture the tests share), and missing-key cases become in-memory `check` tests in `libs/rules/tests/engine/check.rs`.
- **The engine's tolerances:** where a missing key can no longer reach the engine (data that passed `check`), the tolerance either goes or becomes an `expect` with the reason; record each one and which you chose. Keep any the stale-link path still needs (a link naming a key the current data lacks is decoded elsewhere; don't change that behaviour).
- **The real data passes:** `scripts/check.sh`'s seed-check runs `check` on `libs/game-data/`; if the new checks find real dangling keys, report them rather than editing the data files, unless the fix is obvious and the guide in `guide-cache/` confirms it.
- Test-first in `rules` (`tdd`).

**Done when:**
- The rules tests above pass, the small world passes `check`, and the real data passes seed-check.
- `scripts/proof.sh rules` against the parent commit shows no behaviour difference.
- `scripts/check.sh` passes, and `--full` once at the end.
