# The test world behind a `test-world` feature

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Slice 105 of the [MCP server plan](104-mcp-server-plan.md): the engine tests' small invented world lived in `libs/rules/tests/engine/support/`, inside `rules`' own test crate, so `apps/mcp`'s tests couldn't build an `Engine` on it. Move it into `rules` behind an off-by-default cargo feature, a move only, and show it works from outside. *Done when:* the same engine tests pass (same `--list`), a new MCP test runs on the world, no normal build compiles the feature, and `scripts/check.sh` passes.

## Choices settled

1. **Where it lives:** `rules::test_world` (`libs/rules/src/test_world.rs`), declared `#[cfg(feature = "test-world")] pub mod test_world;` in `lib.rs`. The files moved with `git mv`, keeping their layout:
   - `support/world/` → `src/test_world/world/` (`world()` and its parts: races, classes, feats, selections, mythic, paths, deities, backgrounds);
   - `support/builds.rs` → `src/test_world/builds.rs`. It moved too, because the world's parts build their keys with its helpers (`feat`, `class`, `skill`, …), and slices 107–109 want `engine()`, `build()` and the slot helpers as well.
   - `test_world.rs` re-exports them as `support/mod.rs` did: `pub use builds::*; pub use world::world;`.
2. **What the move forced:** `use rules::*;` → `use crate::*;`, `crate::support::{world,builds}` → `crate::test_world::{world,builds}` (including the `pub(in …)` visibilities), and one `rules::Alignment::LawfulGood` → `crate::Alignment::LawfulGood`. No helper was renamed and no body changed.
3. **What stays in `support`:** `conditions` (flattening an `Unmet`), which only the engine tests use, plus `pub use rules::test_world::*;`, so the tests' `use crate::support::*;` (and the two `use crate::support::world;`) lines didn't change.
4. **How the tests turn it on: the self dev-dependency.** `libs/rules/Cargo.toml` already had `[dev-dependencies] rules = { path = ".", features = ["testing"] }` for its test helpers; `test-world` joins it there. Chosen over `required-features` on the `engine` test target because the pattern was already in place, and `required-features` silently skips the target when the feature is off (`cargo test -p rules` would have run no engine tests at all unless every caller passed `--features test-world`).
5. **`apps/mcp`:** its `[dev-dependencies]` add `rules = { workspace = true, features = ["test-world"] }`. `Planner::from_data(GameData) -> Result<Planner, Vec<DataProblem>>` builds a planner from data in memory after `GameData::check`, so the server's "checked data only" stays true. The test support gains `Client::start_on(planner)`; `Client::start()` is `start_on` with the real data.
6. **No leak.** With resolver 3, dev-dependency features apply only to builds of test targets. `cargo build --unit-graph` (the pinned nightly's `-Z unstable-options`) shows `rules` compiled with **no features** for `-p server -p mcp --bins`, for `-p frontend --lib --target wasm32-unknown-unknown` and for `--workspace`, and with `test-world` only for `-p mcp --tests`. `cargo tree -e features,normal -i rules` agrees for `server`, `mcp` and `frontend`. The world adds no dependencies anyway (only `rules` and `std`).

## Progress log

- 2026-11-08: Fast-forwarded the worktree to `main` (a2e591e). Saved `cargo test -p rules -- --list` (494 lines, 489 tests).
- Moved the files, rewrote the paths, added the feature and the dev-dependency. The `--list` output after the move is byte-for-byte identical.
- Added `apps/mcp/tests/server/test_world.rs`: it starts the server on `rules::test_world::world()`, searches "seeker" (a class only the test world has) and describes it (20 levels, `seeker/creed` at 1). Passes, alongside the 15 existing server tests.
- Checked the features with `--unit-graph` and `cargo tree` (above).
- `SQLX_OFFLINE=true scripts/check.sh`: ALL-OK.

## Answer

Done 2026-11-08. The test world and its Build helpers are `rules::test_world`, behind the off-by-default `test-world` feature. `rules`' engine tests reach it through the existing self dev-dependency and their unchanged `support` module (same 489 tests), and `apps/mcp`'s tests through their own dev-dependency and `Planner::from_data`. No app build compiles it.
