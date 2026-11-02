# Folder layout: apps and libs

Type: task
Status: open
Blocked by: 51
Map: [WotR Build Planner](../map.md)

## Question

**User, 2026-11-02:** move the folders into `apps/{frontend|backend}` and `libs/{seeds|rules|extract|game-data…}`, for organisation.

| Now | Moves to | Note |
|---|---|---|
| `frontend/` (wasm entry, cargo-leptos `lib-package`) | `apps/frontend/` | |
| `server/` (Axum, cargo-leptos `bin-package`) | `apps/backend/` | |
| `style/`, `public/` | `apps/frontend/style/`, `apps/frontend/public/` | cargo-leptos `style-file`, `assets-dir` |
| `app/` (the Leptos UI, shared by both apps) | `libs/ui/` | not named by the user; chosen by the lead |
| `data/` (schema, loader, seed bin, `.sqlx/`, migrations) | `libs/seeds/` | the user's "seeds" |
| `rules/` | `libs/rules/` | |
| `extract/` | `libs/extract/` | |
| `game-data/` | `libs/game-data/` | data files, not a crate |
| `guide-cache/`, `.scratch/`, `scripts/`, root config files | stay at the root | `guide-cache/` is the user's saved pages (git-ignored) |

Folders move; **package names don't change** (`app`, `frontend`, `server`, `data`, `rules`, `extract`), so `cargo -p …`, `cargo leptos`'s `bin-package`/`lib-package` and the crates' `use` paths stay as they are. Say in the Answer if a name now reads oddly next to its folder (e.g. package `data` in `libs/seeds/`) and recommend whether to rename it; don't rename without the user.

## Build

AFK; the only agent in the repo. Use `git mv` so history follows the files. Update every path that names a folder: the workspace `members` and path dependencies in every `Cargo.toml`, cargo-leptos metadata (`style-file`, `assets-dir`), `data::game_data_dir()` and the extractor's `guide-cache` path (`CARGO_MANIFEST_DIR`-relative), `sqlx::migrate!()` and `.sqlx/` (they move with the crate; `cargo sqlx prepare --check` must still pass), `.gitignore`, `docker-compose.yml`, `scripts/`, `CLAUDE.md` (DB reset steps: `sqlx migrate run --source …`, `cd … && cargo sqlx prepare`), `README.md`, `GLOSSARY.md`, `game-data/README.md`, the map's notes and every ticket under `.scratch/wotr-planner/issues/` whose text gives a path as an instruction (leave historical progress logs as written), and doc comments that name folders. The lead's launch config (`/Users/thaynamarcela/code/.claude/launch.json`) runs `cargo leptos watch` from the repo root, which must keep working.

No code behaviour changes: no file's content changes except paths.

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes from the repo root with the new paths (`cargo test --workspace`, clippy, fmt, the wasm build of `rules`, `SQLX_OFFLINE=true cargo leptos build`, `cargo sqlx prepare --check` in the new data folder, seed `--check-only` and the real seed); `cargo leptos build --release` passes; `cargo run -p extract` into a scratch directory still finds `guide-cache/` and produces the same files as before the move; `cargo leptos watch` serves the sheet; `git status` shows only renames plus the path edits; a grep for the old top-level folder names as paths finds nothing outside historical logs.

## Progress log

## Answer
