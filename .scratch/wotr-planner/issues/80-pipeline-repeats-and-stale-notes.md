# Drop the pipeline's repeated work and stale config notes

Type: task
Status: open
Blocked by: [79](79-game-files-without-sqlx.md)
Map: [WotR Build Planner](../map.md)

## Question

Small leftovers from the architecture review (2026-11-03):
- **`GameData::canonical()` runs four times** along the pipeline: `extract/src/build/mod.rs:169`, `seeds/src/files.rs:99`, `seeds/src/load.rs:123`, `rules/src/engine/catalog.rs:38`. Ticket 79 may move the `files.rs` one.
- **`check()` runs twice per extraction:** `extract/src/build/mod.rs:160` reports the problems, then `extract/src/main.rs:79` runs it again only for the count.
- **`libs/ui/Cargo.toml`'s `ssr` comment** says the feature pulls sqlx and tokio into the wasm build, but `ui` depends on neither crate. And the root `Cargo.toml`'s `ui = { …, default-features = false }` does nothing while `ui`'s `default` is empty.

## Build

- **`canonical()`** runs once at each boundary where data enters from outside (reading the files, loading from Postgres, the engine taking data from the network) and nowhere a value is already canonical. Find out which calls are redundant; keep any the code relies on, and say why in the Answer.
- **The extraction** checks once and passes the count on.
- **The `ssr` comment** says what the feature really pulls in (check `cargo tree -p ui -e features --features ssr`); drop `default-features = false` if it still does nothing.

**Done when:**
- `scripts/extraction-diff.sh <out-dir>` against `HEAD` reports identical.
- `scripts/proof.sh rules <out-dir>` against `HEAD` reports no difference.
- `scripts/check.sh` passes.
