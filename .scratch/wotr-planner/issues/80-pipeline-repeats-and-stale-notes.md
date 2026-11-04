# Drop the pipeline's repeated work and stale config notes

Type: task
Status: resolved
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

## Progress log

Scratch dir (outside the repo): `/private/tmp/claude-501/-Users-thaynamarcela-code/20f0bb4f-7437-4fe9-8ef8-ebb8716f6fbe/scratchpad/t80/` (below, `t80/`).

- 2026-11-04: re-found the sites at `HEAD` (63f4cbc, after tickets 76–78). Five places put game data in canonical order:
  1. `seeds/src/files.rs`'s `read_game_data_files` (`data.canonical()`): data entering from the files.
  2. `seeds/src/load.rs`'s `load` (`data.canonical()`): data entering from Postgres.
  3. `seeds/src/prerequisites.rs`'s `Trees::get` (`tree.canonicalize()`): each prerequisite tree as `load` reads it, before step 2 sorts every tree again.
  4. `rules/src/engine/catalog.rs`'s `Catalog::new` (`data.canonical()`): data entering the engine (from the network in the browser; from the files in the proof and the real-data tests; built by hand in the `rules` tests).
  5. `proof/src/lib.rs`'s `engine_on` (`data.canonical()`): both callers (`proof`'s `engine_and_builds`, `round_trip.rs`) hand it `read_game_data_files`'s output, already canonical.
  The `.canonical()` calls in `seeds/tests/postgres.rs` and `rules/tests/engine/codec.rs` are on hand-built data in tests; left alone.
- 2026-11-04: checked that step 2 covers every tree step 3 sorted. `Trees::get` has six callers in `load.rs`: feats, Selection options, archetypes, classes (base and prestige), mythic abilities, mythic feats. `GameData::canonicalize` sorts the prerequisite of every one of them (feats, mythic abilities and mythic feats directly; classes and archetypes in `Class::canonicalize`; options in `canonicalize_selections`, run for every class's, archetype's and path's selections, the only owners `load` gives selections). `Prerequisite::canonicalize` is a recursive sort, so sorting once gives what sorting twice did. Nothing in `load` reads a tree's order between the two.
- 2026-11-04: decisions (small, settled here):
  1. **Dropped `Trees::get`'s `canonicalize`** (step 3): `load` makes the trees canonical with the rest of the data, so each tree is now sorted once on the way in. Module doc updated.
  2. **Dropped `engine_on`'s `canonical()`** (step 5) by having `engine_on` take the data folder and read it with `read_game_data_files`, so the value is canonical by that function's contract rather than by its callers' habit. Both callers simplified. The harness still builds against `HEAD`'s crates for `proof.sh rules` (`read_game_data_files` is unchanged there).
  3. **Kept steps 1, 2 and 4**, one per boundary (see the Answer).
- 2026-11-04: `ssr`. `cargo tree -p ui -e normal --format '{p} [{f}]'` with and without `--features ssr` (`t80/pf-diff.txt`): `ssr` turns on `ssr` in leptos, leptos_meta, leptos_router, leptos_macro, leptos_server, server_fn, server_fn_macro and tachys (plus `hydration` in leptos, leptos_dom and leptos_server, `browser` in hydration_context, `inventory` in server_fn) and adds one package, `inventory`. No sqlx, tokio, axum or hyper. `cargo tree -p ui --features ssr --target all -i tokio` finds tokio only under `wasm-bindgen-futures` for `cfg(all(target_os = "emscripten", wasm_bindgen_unstable_tokio))`, and it is there without `ssr` too; `-i sqlx` matches no package.
- 2026-11-04: `default-features = false` on the root's `ui` dependency: dropped. `ui`'s `default` is empty, so it switched nothing off. `cargo tree -p frontend` and `-p server` (`-e normal,build --target all`, with features), before and after (`t80/before-*.txt`, `t80/after-*.txt`): the only change is `ui` now listing its empty `default` feature (`[hydrate]` → `[default,hydrate]`, `[ssr]` → `[default,ssr]`); every other package and feature is the same. The guard against an `ssr` default is the comment on the feature.
- 2026-11-04: `scripts/check.sh` (with `wotr-planner-postgres` up): fmt, clippy, wasm-clippy, test, rules-wasm, seed-check OK, ALL-OK. `cargo test -p seeds --test postgres`: 28 passed (the files → Postgres → `GameData` round trips compare the loaded data, order included, with the files').
- 2026-11-04: `scripts/proof.sh rules t80/proof` against `HEAD` (63f4cbc): 1999 Builds each side, 0 only in A, 0 only in B, 0 differing; `compare.txt` empty.
- 2026-11-04: served JSON. A scratch binary (`t80/jsonhash-head`, `t80/jsonhash-wt`) loading the dev database with `HEAD`'s `seeds` and the working tree's and serialising it as `GameDataFile::new` does: both 1,543,506 bytes, both `/game-data/b27ae7841d84d211.json`. The seed's stored hash is `game_data_hash` over the files, which this ticket doesn't touch.
- 2026-11-04: `SQLX_OFFLINE=true cargo leptos build` (dev): the frontend (wasm, `--no-default-features`) and the server built.

## Answer

Each boundary sorts the game data once; the two repeats are gone.

- **`canonical()` calls:**
  - **Kept: `read_game_data_files`** (`seeds/src/files.rs`). Data enters from the files, whose order means nothing. The seed, `check_game_data_files`, the proof and the real-data tests all rely on it.
  - **Kept: `load_game_data`** (`seeds/src/load.rs`). Data enters from Postgres, whose tables store no order. The served JSON is this value serialised, so its bytes (and its URL, their hash) depend on it, and the round-trip tests compare it with the files' data.
  - **Kept: `Catalog::new`** (`rules/src/engine/catalog.rs`). Data enters the engine from the network in the browser and is built by hand in the `rules` tests; the engine can't assume either is canonical.
  - **Dropped: `Trees::get`'s `tree.canonicalize()`** (`seeds/src/prerequisites.rs`). Every tree it returned is sorted again by `load`'s `canonical()`, which covers all six owners it serves; a recursive sort done once gives the same tree.
  - **Dropped: `proof::engine_on`'s `canonical()`** (`proof/src/lib.rs`). `engine_on` now takes the data folder and reads it with `read_game_data_files`, which returns canonical data; `Engine::new` still sorts its own copy.
- **`ssr` comment** (`libs/ui/Cargo.toml`), now: "Server-side rendering, for the backend: turns on the `ssr` feature of leptos, leptos_meta and leptos_router (and, through them, of server_fn) and adds one crate, `inventory` (`cargo tree -p ui -e features --features ssr`). Never add it to `default`: the frontend's wasm build takes `hydrate`, and server rendering belongs to the backend alone." Evidence: with `ssr`, `ui`'s tree gains only `inventory` and the leptos family's `ssr` features; sqlx is not in it, and tokio appears only for an emscripten-only cfg, with or without `ssr`.
- **`default-features = false`:** dropped from the root's `ui` dependency. `ui`'s `default` is empty, and the frontend's and server's feature trees are the same before and after but for `ui`'s empty `default`.
- **Checks:** `scripts/check.sh` ALL-OK. `scripts/proof.sh rules` against `HEAD`: 1999 Builds, 0 differing, empty `compare.txt`. Served JSON from the dev database identical at `HEAD` and in the working tree (same bytes, same URL). `SQLX_OFFLINE=true cargo leptos build` (dev) built both packages.
