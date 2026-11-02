# Rename the packages after their folders

Type: task
Status: resolved
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

- **2026-11-02**: saved `cargo test --workspace -- --list` (665 lines, 18 test binaries) to the scratchpad before any edit. The user's uncommitted doc-comment line in `apps/backend/src/main.rs` (`warn_if_game_data_is_stale`) was saved as a diff too, and stayed as it was: only that file's `app::`/`data::` paths and its `-p data` message changed.
- Manifests: `name = "seeds"` / `name = "ui"`, the root workspace dependencies, `server`'s `ui` (`ssr`) and `seeds`, `frontend`'s `ui` (`hydrate`) and `timings = ["ui/timings"]`, `extract`'s `seeds` (moved to keep that list alphabetical). The cargo-leptos metadata names no `app` (`bin-package = "server"`, `lib-package = "frontend"`), so it's unchanged. `Cargo.lock` follows.
- Code: every `data::` → `seeds::` and `app::` → `ui::` in 18 Rust files (the server, the frontend's `hydrate_body(ui::App)`, the extractor and its tests, the `seed` binary, the `seeds` tests), the `-p data` in the server's stale warning, the `seed` binary's doc and the extractor's report text. Prose that names the crate: `codec.rs`'s "`seeds`'s real-data test", the extractor tests' "As `seeds` reads the files" (4) and "The seeds crate's fixture" (2), `game_data_dir`'s `expect` and `libs/seeds/Cargo.toml`'s comment. Variables called `data`, the `.app` CSS class and "the app" in prose are untouched. `cargo fmt` moved `use ui::…` after `use tower_http::…` in `main.rs` and rewrapped nothing else that matters.
- Docs: `README.md` (run, game data, the layout table, tests), `libs/game-data/README.md`'s seeding block and `EXTRACTION_REPORT.md`'s instruction line (which the extractor's `report.rs` writes; a fresh extraction writes the same line). `CLAUDE.md`, `GLOSSARY.md`, `scripts/`, the map's Notes and open tickets 54 and 55 name neither package, so they're unchanged. Historical tickets, the map's Decisions and `research/leptos-toolchain.md` keep the old names as written.
- **Not changed: the migration's header comment** (`data::replace_game_data`, `data::seed`, lines 3–4 of `20261031000000_game_data.sql`). sqlx checksums the whole file, comments included, so editing it makes the dev database's applied migration mismatch and the server's startup `migrate` fail until a reset, and this ticket allows none. See the Answer.
- Verification (all pass): `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace` (extract 92, rules 4 + 331, seeds files 25 + game_data 157 + postgres 27, server 4); `cargo build -p rules --target wasm32-unknown-unknown`; `SQLX_OFFLINE=true cargo leptos build`; `cargo leptos build --release`; `cd libs/seeds && cargo sqlx prepare --check` (no `.sqlx/` file changed); `cargo run -p seeds --bin seed -- --check-only` (12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 38 backgrounds). `cargo leptos watch` served `/` on 3900 with no stale-data warning; in the browser the sheet hydrated (the race picker opens with the real races) and the console had no errors. No database reset or reseed.
- Test list: after the rename it is 665 lines again. Paired by binary (crate from the target path, `data` read as `seeds` and `app` as `ui`), the 18 binaries hold the same tests with the same counts; sorted, the two outputs are byte-identical. Only the headers differ (`seeds`/`ui` for `data`/`app`, and their place in cargo's order).
- Acceptance grep (`\bdata::`, `\bapp::`, `-p data`, `"app/` over tracked files outside `.scratch/`, plus tickets 54–55 and the map's Notes): only the two migration comment lines above.

## Answer

Done: the package `data` (`libs/seeds/`) is `seeds` and `app` (`libs/ui/`) is `ui`, in 34 files (6 manifests, `Cargo.lock`, 24 Rust files, and `README.md`, `libs/game-data/README.md` and `EXTRACTION_REPORT.md`), with no behaviour change, no file moves and no database change. Every Done-when check passes, and the test list is the same apart from the binaries' headers.

One leftover: the migration's header comment still says `data::replace_game_data` and `data::seed`. Changing it changes the migration's checksum, which needs the dev-DB reset this ticket rules out. Recommended: fix those two lines in the next ticket that edits the migration and resets anyway (54 or 55 if they need a schema change), or as a one-line reset whenever the user allows one.

**Lead, 2026-11-02:** the leftover is fixed. The migration's header comment now says `seeds::replace_game_data` and `seeds::seed`, and the dev DB was reset per `CLAUDE.md` (the lead was the only agent; `.sqlx/` unchanged). `libs/seeds/Cargo.toml`'s comment, already wrong before the rename, now says the crate is "used only by `server` and `extract`" (`ui` doesn't depend on it). Tickets 54 and 55 point at this ticket's Done when for their checks, since ticket 22's Verification still names the old folders and packages.
