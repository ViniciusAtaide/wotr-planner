# Read and write the data files without sqlx

Type: task
Status: closed (won't do)
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The layout of `libs/game-data/` is declared twice: the reader's in `libs/seeds/src/files.rs` (lines 24–34, 85–99) and the writer's in `libs/extract/src/ron_out.rs` (lines 18–56). And `extract` and `proof` depend on `seeds` only to read the files (`read_game_data_files`, `game_data_dir`: `extract/src/main.rs:21, 58`, `proof/src/main.rs:52`). That pulls sqlx, its macros and tokio into both, which is why `scripts/extraction-diff.sh` and `scripts/proof.sh` export `SQLX_OFFLINE=true` (architecture review, 2026-11-03).

## Build

- **One home for the files:** the layout, reading, writing and hashing of the data files move into one sqlx-free place, either a small crate (for example `libs/game-files`) or a feature-gated module of `rules`. Choose, and record why in the Answer. `rules` must still build for `wasm32-unknown-unknown`.
- **`seeds`, `extract` and `proof` use it.** `extract` and `proof` drop their `seeds` dependency.
- **The scripts** drop `SQLX_OFFLINE=true` where their builds no longer need it (`check.sh`'s cargo-leptos step still does).
- Leave the RON output byte-for-byte the same, and the seed's staleness hash the same value.

**Done when:**
- `libs/extract/Cargo.toml` and `libs/proof/Cargo.toml` don't name `seeds`.
- The layout is declared once.
- `scripts/extraction-diff.sh <out-dir>` against `HEAD` reports identical.
- `game_data_hash` of `libs/game-data/` gives the same value as at `HEAD`, so a seeded database doesn't turn stale. (`scripts/proof.sh` can't compare this change: its harness's own dependencies change.)
- `scripts/check.sh` passes.

## Answer

**Closed, not built** (user, 2026-11-03, taking the recommendation on [ticket 75](75-game-data-master.md)'s Q4). With the extractor removed ([ticket 82](82-remove-the-extractor.md)), `ron_out.rs` goes, so the layout is declared once, in `seeds`. What's left is `proof` depending on `seeds`, which costs build time only.
