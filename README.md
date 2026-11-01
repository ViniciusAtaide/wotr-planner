# WotR Planner

A build planner for Pathfinder: Wrath of the Righteous. Leptos (SSR + hydration) on Axum, Postgres via sqlx.

## Prerequisites

- [rustup](https://rustup.rs). `rust-toolchain.toml` pins `nightly-2026-10-30` with the `wasm32-unknown-unknown`
  target; rustup installs it on the first `cargo` command in this directory.
- cargo-leptos 0.3.11: `cargo install --locked cargo-leptos@0.3.11` (takes about 8 minutes).
- sqlx-cli 0.9 (for `sqlx migrate run` when resetting the dev database, and `cargo sqlx prepare`):
  `cargo install sqlx-cli@0.9.0 --no-default-features --features rustls,postgres`.
  (`--locked` does not work for sqlx-cli 0.9.)
- Docker, for Postgres.

## Run it

```sh
cp .env.example .env      # once
docker compose up -d      # Postgres 18 on localhost:5434
cargo run -p data --bin seed -- data/tests/fixtures/game-data   # load game data (see below)
cargo leptos watch        # builds, runs migrations at startup, rebuilds on change
```

Open <http://localhost:3900>.

### Game data

The game data lives in RON files under `game-data/` (see `game-data/README.md`). `cargo run -p data --bin seed [DIR]`
replaces every game table with the files in `DIR` (default: `$GAME_DATA_DIR`, or `game-data/`) in one transaction and
stores their hash, after checking the data (`rules::GameData::check`). The server loads the game data once at startup,
so restart it after a reseed. At startup it warns when that hash doesn't match the files in `$GAME_DATA_DIR`
(default `game-data/`); it never reseeds by itself. `game-data/` is empty until the guide is extracted, so for now seed
the hand-written fixture in `data/tests/fixtures/game-data` (and set `GAME_DATA_DIR` to it to silence the warning).

### From a phone on the same network

The dev server binds to `0.0.0.0:3900` (live reload on port 3901). Find the computer's LAN address
(on macOS: `ipconfig getifaddr en0`) and open `http://<that-address>:3900` on the phone. If it does not load, allow
incoming connections for the `server` binary in the macOS firewall prompt or settings.

## Layout

| Crate | What it holds |
| --- | --- |
| `app` | Leptos components and `#[server]` functions; serves the `Arc<GameData>` the server provides and never depends on `data` |
| `frontend` | the wasm entry point (`hydrate` only) |
| `server` | the Axum entry point |
| `rules` | pure game rules, serde only, compiled for both browser and server |
| `data` | sqlx queries and the migration (`data/migrations`) behind `Database` and `data::Error`; native only, used by `server` |

## Tests

```sh
cargo test --workspace
```

The `data` tests in `data/tests/postgres.rs` need Postgres running (`docker compose up -d`): each one creates and drops
its own throwaway database through `DATABASE_URL`, so they never touch the dev data.

## Database queries and offline builds

`sqlx::query!` checks SQL against the database at compile time, using `DATABASE_URL` from `.env`. While prototyping
the whole schema is one migration, `data/migrations/20261031000000_game_data.sql`: a schema change edits that file and
resets the dev database (steps in `CLAUDE.md`), with no fix-up migrations. Migrations run when the server starts; with
sqlx-cli you can also run them by hand: `sqlx migrate run --source data/migrations`.

After adding or changing a query, refresh the checked-in metadata (needs Postgres running and migrated):

```sh
cd data && cargo sqlx prepare
```

To build without a database: `SQLX_OFFLINE=true cargo leptos build`.
