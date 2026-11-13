# WotR Planner

A build planner for Pathfinder: Wrath of the Righteous. Leptos (SSR + hydration) on Axum. The server reads the game
data from RON files and gives it to the browser as JSON; there is no database.

## Prerequisites

- [rustup](https://rustup.rs). `rust-toolchain.toml` pins `nightly-2026-10-30` with the `wasm32-unknown-unknown`
  target; rustup installs it on the first `cargo` command in this directory.
- cargo-leptos 0.3.11: `cargo install --locked cargo-leptos@0.3.11` (takes about 8 minutes).

## Run it

```sh
cargo leptos watch                      # builds, rebuilds on change
```

Open <http://localhost:3900>.

A release build: `cargo leptos build --release`, then run `target/release/server` from the repo root.

The server never compresses, in any build: compressing is publishing's job, and GitHub Pages gzips what it serves.
Don't pass `--precompress`: it writes `.br` and `.gz` copies into `target/site/` that a later build doesn't rewrite,
and leptos's file handler would serve them stale. The wasm, JS and CSS are sent with `Cache-Control: no-cache`, so
a browser picks up a rebuild at once.

A static site, for GitHub Pages: `scripts/build-site.sh <out-dir>` saves the release server's page, wasm, JS, CSS
and game data as files (usage in its header). The page names them relative to itself, so the folder works at `/`
and under a project site's `/<repo>/`. `.github/workflows/pages.yml` runs it on every push to `main` and deploys
the folder; the first deployment's steps are in `.scratch/wotr-planner/issues/100-static-site-for-github-pages.md`.

### Game data

The game data lives in RON files under `libs/game-data/` (see `libs/game-data/README.md`). The server reads them once
at startup, from `$GAME_DATA_DIR` (default `libs/game-data/`; `.env` may set it, see `.env.example`), checks them
(`rules::GameData::check`, which names every key the data lacks) and serves them as one JSON document at
`/game-data/<hash>.json`, the hash being its content's. Data with a problem stops the server with the list of
problems. Restart the server after editing the files. `cargo run -p seeds --bin check-data [DIR]` checks a folder on
its own. `libs/seeds/tests/fixtures/game-data` is a small hand-written example of the layout; to serve it instead,
set `GAME_DATA_DIR` to its path.

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
| `seeds` | `libs/seeds/` | reads and checks the game data files (`libs/game-data/`); native only, used by `server`, `mcp` and `proof` |

## Tests

```sh
cargo test --workspace
```

The `seeds` tests read the real files in `libs/game-data/` (`tests/game_data/`, with the snapshot check and the gold
builds) and the fixture in `tests/fixtures/game-data/` (`tests/files.rs`).
