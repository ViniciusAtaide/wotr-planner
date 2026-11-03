# Drop the pipeline's repeated work and stale config notes

Type: task
Status: open
Blocked by: [82](82-remove-the-extractor.md)
Map: [WotR Build Planner](../map.md)

## Question

Small leftovers from the architecture review (2026-11-03), re-scoped after [ticket 75](75-game-data-master.md) (the extractor's own repeats go with it in [ticket 82](82-remove-the-extractor.md)):
- **`GameData::canonical()` runs three times** after the extractor is gone: `seeds/src/files.rs:99` (reading the files), `seeds/src/load.rs:123` (loading from Postgres) and `rules/src/engine/catalog.rs:38` (the engine taking the data).
- **`libs/ui/Cargo.toml`'s `ssr` comment** says the feature pulls sqlx and tokio into the wasm build, but `ui` depends on neither crate. And the root `Cargo.toml`'s `ui = { …, default-features = false }` does nothing while `ui`'s `default` is empty.

## Build

- **`canonical()`** runs once at each boundary where data enters from outside, and nowhere a value is already canonical. Find out which calls are redundant; keep any the code relies on (the engine also takes data from the network), and say why in the Answer.
- **The `ssr` comment** says what the feature really pulls in (check `cargo tree -p ui -e features --features ssr`); drop `default-features = false` if it still does nothing.

**Done when:**
- `scripts/proof.sh rules <out-dir>` against `HEAD` reports no difference.
- `scripts/check.sh` passes.
