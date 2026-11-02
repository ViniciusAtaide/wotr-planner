# Folder layout: apps and libs

Type: task
Status: resolved
Blocked by: 52
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

- 2026-11-02: baseline `cargo run -p extract -- --out <scratch>/before` (58 files) at HEAD `69f8a02`.
- Nine `git mv`s as in the table: `frontend` → `apps/frontend`, `server` → `apps/backend`, `style` and `public` → `apps/frontend/{style,public}`, `app` → `libs/ui`, `data` → `libs/seeds` (with `.sqlx/` and `migrations/`), `rules`, `extract`, `game-data` → `libs/…`. `guide-cache/` untouched at the root. The user's uncommitted doc-comment edit in `server/src/main.rs` moved with the file and is still unstaged at `apps/backend/src/main.rs`.
- Path edits: the root `Cargo.toml` (`members`, the three path dependencies, `style-file`, `assets-dir`); the extractor's default guide folder is now two levels above its manifest (`ancestors().nth(2)`), since `libs/extract/..` is `libs/`; `data::game_data_dir()` needed no change (`libs/seeds/..` + `game-data`), only its doc comment; the extract tests' fixture path `../data/tests/fixtures` → `../seeds/tests/fixtures`; the seed's "not part of the game-data layout" message names `libs/seeds/src/files.rs`; doc comments, the migration's header comments, `.env.example`, `CLAUDE.md` (migration path, `--source libs/seeds/migrations`, `cd libs/seeds && cargo sqlx prepare`), `README.md` (paths, and a Folder column in the Layout table), `libs/game-data/README.md`, the map's Notes, and open tickets 48 and 49. `docker-compose.yml`, `.gitignore`, `scripts/` and `GLOSSARY.md` name no moved folder, so they're unchanged; `Cargo.lock` is unchanged. Resolved tickets, research notes and the map's Decisions keep the old paths as history.
- The migration's comments changed, so its checksum did: the real seed refused ("previously applied but has been modified"). Reset `wotr_planner` in `wotr-planner-postgres` (5434) per `CLAUDE.md`, with the new `--source libs/seeds/migrations`, and reseeded.
- Checks, all from the repo root: `cargo test --workspace` pass (rules 318 + 4, extract 91, data files 25, game data 150, postgres 26); `cargo clippy --workspace --all-targets` clean; `cargo fmt --all --check` pass; `cargo build -p rules --target wasm32-unknown-unknown` pass; `SQLX_OFFLINE=true cargo leptos build` pass; `cd libs/seeds && cargo sqlx prepare --check` pass; seed `--check-only` and the real seed both give 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 38 backgrounds; `cargo leptos build --release` pass (CSS from the new `style-file`).
- `cargo run -p extract -- --out <scratch>/after` finds `guide-cache/` by default and `diff -r before after` is empty (58 files, report 23 items).
- `cargo leptos watch` from the root: `/` 200, `/pkg/wotr-planner.css` and `.wasm` 200; in a browser the sheet hydrates with the game data (Build start, Race/Alignment/Background pickers, point buy, "+ Level 1"), styled, live reload connected, and no stale-data warning. Server stopped.
- `git status`: 318 renames (286 pure, 32 with path edits) plus 7 modified root and `.scratch` files. A grep for the old folder names as paths finds only package names (`-p data`, `bin-package = "server"`), the `app/timings` feature, a CSS class and folder-name joins outside `.scratch/`, and in `.scratch/` only the resolved tickets, research notes, the map's Decisions and this ticket's table.

## Answer

The workspace is now `apps/{frontend,backend}` and `libs/{ui,seeds,rules,extract,game-data}`. Only paths changed. Package names are as before, so `cargo -p …`, `bin-package`/`lib-package` and every `use` are unchanged, and `cargo leptos watch` still runs from the root. `guide-cache/` stays at the root, and the extractor still finds it with byte-identical output. Every ticket-22 check, the release build and the watch server pass. One side effect: the migration file's comments changed, so an existing dev database must be reset once (done here). A checkout on another machine will need the same reset, because `Database::connect` refuses a modified migration.

Names that now read oddly next to their folders. These are recommendations only; nothing was renamed:

- **`data` in `libs/seeds/`: rename to `seeds`** (recommended). The crate is the schema, the migration, the loader and the `seed` binary, so `seeds` fits, and `cargo run -p seeds --bin seed` reads naturally. The cost is `data::` → `seeds::` across `server`, `extract` and the tests, plus `-p data` in the docs and `CLAUDE.md`. Keeping `data` costs only the mismatch.
- **`app` in `libs/ui/`: rename to `ui`** (recommended). It's the Leptos UI shared by both apps, and `app` next to an `apps/` folder is easy to confuse. It's only `use app::…` in the two entry crates plus the `app/hydrate`, `app/ssr` and `app/timings` feature references.
- **`server` in `apps/backend/`: optional.** `backend` would match the folder, but `server` is also the binary's name (the macOS firewall prompt, the README's "allow the `server` binary"), and cargo-leptos's `bin-package` changes with it. Low value; keep unless the user wants folder = package everywhere.
- `frontend`, `rules` and `extract` already match their folders.
