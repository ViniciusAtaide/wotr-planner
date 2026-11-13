# WotR Planner

A build planner for Pathfinder: Wrath of the Righteous. Leptos (SSR + hydration) on Axum, SQLite via sqlx on the
server (the browser gets the game data as JSON).

## Prerequisites

- [rustup](https://rustup.rs). `rust-toolchain.toml` pins `nightly-2026-10-30` with the `wasm32-unknown-unknown`
  target; rustup installs it on the first `cargo` command in this directory.
- cargo-leptos 0.3.11: `cargo install --locked cargo-leptos@0.3.11` (takes about 8 minutes).
- sqlx-cli 0.9 (for `sqlx migrate run` when creating the database, and `cargo sqlx prepare`):
  `cargo install sqlx-cli@0.9.0 --no-default-features --features rustls,sqlite`.
  (`--locked` does not work for sqlx-cli 0.9.)
- A C compiler (on macOS, the Xcode command line tools): sqlx builds SQLite's C source once. No database server
  is needed.

## Run it

```sh
cp .env.example .env                    # once
sqlx database create && sqlx migrate run --source libs/seeds/migrations   # once: the schema the queries compile against
cargo run -p seeds --bin seed           # load the game data in libs/game-data/ (see below)
cargo leptos watch                      # builds, rebuilds on change
```

Open <http://localhost:3900>.

A release build: `cargo leptos build --release --precompress`, then run `target/release/server` from the repo root.
`--precompress` writes brotli (quality 11) and gzip copies of the wasm, JS and CSS next to them. The server sends
the brotli copy to a browser that takes brotli; without the flag they are compressed on every request, at brotli
quality 4. The gzip copies go unused: the server's own gzip is smaller. cargo-leptos has no `Cargo.toml` key for
the flag, so always pass it. A dev server (`cargo leptos watch`, a debug build) never sends the brotli copies: dev and
release builds share `target/site/`, and a dev build rewrites only the plain files, so a release build's copies left
beside them would be stale.

A static site, for GitHub Pages: `scripts/build-site.sh <out-dir>` saves the release server's page, wasm, JS, CSS
and game data as files (usage in its header). The page names them relative to itself, so the folder works at `/`
and under a project site's `/<repo>/`. `.github/workflows/pages.yml` runs it on every push to `main` and deploys
the folder; the first deployment's steps are in `.scratch/wotr-planner/issues/100-static-site-for-github-pages.md`.

### Game data

The game data lives in RON files under `libs/game-data/` (see `libs/game-data/README.md`). The database is one SQLite
file, `wotr-planner.db` at the repo root (git-ignored; `DATABASE_URL` in `.env` names it, relative to the repo root).
`cargo run -p seeds --bin seed [DIR]` checks the files in `DIR` (default: `$GAME_DATA_DIR`, or `libs/game-data/`;
`rules::GameData::check`), builds a new database from them next to the old one (`wotr-planner.db.new`: the
migration, every table and the files' hash, in one transaction), checks every reference (SQLite's foreign key check,
which names each key the data lacks) and renames it over `wotr-planner.db`. A seed that fails leaves the old database
as it was. The server opens the database read-only and loads the game data once at startup, so restart it after a
reseed. At startup it warns when that hash doesn't match the files in `$GAME_DATA_DIR`
(default `libs/game-data/`); it never reseeds by itself. `libs/seeds/tests/fixtures/game-data` is a small hand-written
example of the layout; to seed it instead, pass its path to the seed (and set `GAME_DATA_DIR` to it to silence the warning).

### From a phone on the same network

The dev server binds to `0.0.0.0:3900` (live reload on port 3901). Find the computer's LAN address
(on macOS: `ipconfig getifaddr en0`) and open `http://<that-address>:3900` on the phone. If it does not load, allow
incoming connections for the `server` binary in the macOS firewall prompt or settings.

## Layout

| Crate | Folder | What it holds |
| --- | --- | --- |
| `ui` | `libs/ui/` | Leptos components and `#[server]` functions; serves the `Arc<GameData>` the server provides and never depends on `seeds` |
| `frontend` | `apps/frontend/` | the wasm entry point (`hydrate` only) |
| `server` | `apps/backend/` | the Axum entry point |
| `rules` | `libs/rules/` | pure game rules, serde only, compiled for both browser and server |
| `seeds` | `libs/seeds/` | sqlx queries and the SQLite migration (`libs/seeds/migrations`) behind `Database` and `seeds::Error`; native only, used by `server` |

## Tests

```sh
cargo test --workspace
```

The `seeds` tests in `libs/seeds/tests/database.rs` need no database server: each one seeds its own SQLite file in a
temporary folder (or, through `#[sqlx::test]`, under `libs/seeds/target/sqlx/test-dbs`), so they never touch the dev data.

## Database queries and offline builds

`sqlx::query!` checks SQL against the database file at compile time, using `DATABASE_URL` from `.env`; without
`DATABASE_URL` it reads the checked-in `libs/seeds/.sqlx` metadata instead. While prototyping the whole schema is one
migration, `libs/seeds/migrations/20261031000000_game_data.sql`: a schema change edits that file and rebuilds the dev
database (steps in `CLAUDE.md`), with no fix-up migrations. The seed runs the migration on every new database it
builds; the server never migrates, and refuses a database built from another migration (reseed it).

After adding or changing a query, refresh the checked-in metadata (from the repo root; `cargo sqlx prepare` runs in
`libs/seeds`, so it gets the database's absolute path):

```sh
(cd libs/seeds && cargo sqlx prepare -D "sqlite://$PWD/../../wotr-planner.db")
```

To build without a database: `SQLX_OFFLINE=true cargo leptos build`.
