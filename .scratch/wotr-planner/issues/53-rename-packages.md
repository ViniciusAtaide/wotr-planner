# Rename the packages after their folders

Type: task
Status: open
Blocked by: 49
Map: [WotR Build Planner](../map.md)

## Question

**User, 2026-11-02** (taking [ticket 50](50-folder-layout.md)'s recommendations): rename the package `data` (in `libs/seeds/`) to `seeds`, and `app` (in `libs/ui/`) to `ui`. `server`, `frontend`, `rules` and `extract` keep their names.

## Build

AFK; the only agent in the repo. A rename only: no behaviour changes and no file moves.

- The package names in `libs/seeds/Cargo.toml` and `libs/ui/Cargo.toml`, the workspace dependencies in the root `Cargo.toml`, and every dependency on them (`data = …` → `seeds = …`, `app = …` → `ui = …`) including dev-dependencies and features (`app/hydrate`, `app/ssr`, `app/timings` → `ui/…`).
- Every `use data::…`, `data::…`, `use app::…` and `app::…` path in code, tests and doc comments. Don't touch the word "data" or "app" where it isn't the crate (variables, CSS classes, prose, `game-data`).
- Commands that name the packages: `-p data`, `cargo run -p data --bin seed`, in `CLAUDE.md`, `README.md`, `GLOSSARY.md`, `libs/game-data/README.md`, `scripts/`, the map's Notes and the open or instructional parts of tickets (leave historical progress logs as written).
- cargo-leptos metadata if it names `app` (`bin-package` / `lib-package` stay `server` / `frontend`).
- `.sqlx/` query files are keyed by query hash, not crate name; `cd libs/seeds && cargo sqlx prepare --check` must still pass.

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes (`cargo fmt --all --check`, clippy with `-D warnings`, `cargo test --workspace`, the wasm build of `rules`, `SQLX_OFFLINE=true cargo leptos build`, `cd libs/seeds && cargo sqlx prepare --check`, `cargo run -p seeds --bin seed -- --check-only`); `cargo leptos build --release` passes; `cargo test --workspace -- --list` lists the same tests as before (save both lists in the scratchpad and diff them, only the crate names in the headers differ); `cargo leptos watch` serves the sheet; a grep for `\bdata::`, `\bapp::`, `-p data` and `"app/` outside historical logs finds nothing that names the crate.

Leave the user's uncommitted edit in `apps/backend/src/main.rs` (a doc-comment line on `warn_if_game_data_is_stale`) exactly as it is.

## Progress log

## Answer
