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
cargo run -p seeds --bin seed   # load the game data in libs/game-data/ (see below)
cargo leptos watch        # builds, runs migrations at startup, rebuilds on change
```

Open <http://localhost:3900>.

A release build: `cargo leptos build --release --precompress`, then run `target/release/server` from the repo root.
`--precompress` writes brotli (quality 11) and gzip copies of the wasm, JS and CSS next to them. The server sends
the brotli copy to a browser that takes brotli; without the flag they are compressed on every request, at brotli
quality 4. The gzip copies go unused: the server's own gzip is smaller. cargo-leptos has no `Cargo.toml` key for
the flag, so always pass it.

### Game data

The game data lives in RON files under `libs/game-data/` (see `libs/game-data/README.md`). `cargo run -p seeds --bin seed [DIR]`
replaces every game table with the files in `DIR` (default: `$GAME_DATA_DIR`, or `libs/game-data/`) in one transaction and
stores their hash, after checking the data (`rules::GameData::check`). The server loads the game data once at startup,
so restart it after a reseed. At startup it warns when that hash doesn't match the files in `$GAME_DATA_DIR`
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
| `seeds` | `libs/seeds/` | sqlx queries and the migration (`libs/seeds/migrations`) behind `Database` and `seeds::Error`; native only, used by `server` |

## Tests

```sh
cargo test --workspace
```

The `seeds` tests in `libs/seeds/tests/postgres.rs` need Postgres running (`docker compose up -d`): each one creates and drops
its own throwaway database through `DATABASE_URL`, so they never touch the dev data.

## Database queries and offline builds

`sqlx::query!` checks SQL against the database at compile time, using `DATABASE_URL` from `.env`. While prototyping
the whole schema is one migration, `libs/seeds/migrations/20261031000000_game_data.sql`: a schema change edits that file and
resets the dev database (steps in `CLAUDE.md`), with no fix-up migrations. Migrations run when the server starts; with
sqlx-cli you can also run them by hand: `sqlx migrate run --source libs/seeds/migrations`.

After adding or changing a query, refresh the checked-in metadata (needs Postgres running and migrated):

```sh
cd libs/seeds && cargo sqlx prepare
```

To build without a database: `SQLX_OFFLINE=true cargo leptos build`.
