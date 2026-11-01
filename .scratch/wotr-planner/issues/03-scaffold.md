# Create the project skeleton

Type: task
Status: resolved
Blocked by: 02
Map: [WotR Build Planner](../map.md)

## Question

Set up the empty project so design and implementation tickets have somewhere to land. Use the versions and layout from [Leptos and toolchain setup](02-leptos-toolchain.md).

- A cargo-leptos SSR workspace in `~/code/wotr-planner` based on `leptos-rs/start-axum-workspace`, with five crates:
  - `app`: Leptos components; reaches `data` only through its `ssr` feature
  - `frontend`: the wasm entry point; `hydrate` only
  - `server`: the Axum entry point
  - `rules`: pure, serde only, compiled for both targets
  - `data`: sqlx models + seed; native only

  Use the exact versions and wiring in [research/leptos-toolchain.md](../research/leptos-toolchain.md).
- **Port 3900**, not 3000: a lifecycle-assessment Vite server already uses 3000. Bind `site-addr` to `0.0.0.0:3900` so the user's Fold 7 can open the dev server over the local network, and set the cargo-leptos `reload-port` to match.
- Put a mobile viewport meta tag in the HTML shell (`width=device-width, initial-scale=1`).
- `cargo-leptos@0.3.11` isn't in `~/.cargo/bin` yet; installing it takes about 8 minutes, so run it in the background.
- Use sqlx 0.9 feature names (`runtime-tokio`, not `runtime-tokio-rustls`). Use `path!("/")` for the root route.
- Install sqlx-cli and verify `cargo sqlx prepare` and offline builds; the research ticket left these untested.
- `rust-toolchain.toml` pinned to the chosen nightly, with the wasm32 target.
- `docker-compose.yml` running Postgres, a `.env.example` with `DATABASE_URL`, and sqlx migrations wired up with one placeholder migration.
- Done when `cargo leptos watch` serves a hello page that reads one row from Postgres, and `cargo test -p rules` runs.

AFK: driven by the agent.

## Answer

Built and verified 2026-10-31 (uncommitted). The workspace follows [research/leptos-toolchain.md](../research/leptos-toolchain.md) §3 manifest for manifest.

### What exists

| Path | What it is |
| --- | --- |
| `Cargo.toml` | workspace (`resolver = "3"`, edition 2024), pinned versions, `wasm-release` profile, `[[workspace.metadata.leptos]]` |
| `rust-toolchain.toml` | `nightly-2026-10-30` + `wasm32-unknown-unknown`, rustfmt/clippy/rust-src/rust-analyzer |
| `app/` | `shell()` (HTML document with `<meta name="viewport" content="width=device-width, initial-scale=1">`), `App` router with `path!("/")`, `HomePage`, `#[server] get_greeting()`; features `hydrate` / `ssr` (`ssr` adds `dep:data`) |
| `frontend/` | cdylib wasm entry, `hydrate()` → `leptos::mount::hydrate_body(app::App)` |
| `server/` | Axum `main`: `dotenvy` → `data::connect` (pool + migrations) → `leptos_routes_with_context` providing the `PgPool` as context |
| `rules/` | pure, serde only. `Greeting { id: i32, message: String }` (the hello DTO) and `CharacterLevel` (newtype, `new(u8) -> Result<_, InvalidCharacterLevel>`, accepts 1..=40) with two tests |
| `data/` | `PgPool` re-export, `MIGRATOR = sqlx::migrate!()`, `connect(url)`, `greeting(pool)` using `query_as!(rules::Greeting, …)`; `migrations/20261031000000_hello.sql` (table `hello`, seeded row `(1, 'Hello from Postgres')`); `.sqlx/` offline metadata |
| `docker-compose.yml` | project `wotr-planner`, service `postgres`, image `postgres:18`, container `wotr-planner-postgres`, volume `wotr-planner-pgdata` |
| `.env.example` | `DATABASE_URL=postgres://wotr:wotr@localhost:5434/wotr_planner` (copied to the git-ignored `.env`) |
| `style/main.css`, `public/` | mobile-first base CSS (served as `/pkg/wotr-planner.css`); empty assets dir |
| `README.md` | prerequisites, run, phone access, layout, tests, sqlx prepare/offline |

### Ports and names

- Dev server: `site-addr = "0.0.0.0:3900"`, `reload-port = 3901`. cargo-leptos binds the reload websocket on the site-addr IP (`*:3901`, checked with `lsof`), and Leptos' reload script connects to `window.location.hostname:3901`, so live reload works from the phone too.
- Postgres: host port **5434** (5432 belongs to another project's `postgres` container), user/password `wotr`/`wotr`, database `wotr_planner`.
- cargo-leptos site name `wotr-planner` → assets at `/pkg/wotr-planner.{css,js,wasm}`.

### Commands (all verified)

```sh
docker compose up -d                          # "Container wotr-planner-postgres Running"
cargo leptos watch                            # "listening on http://0.0.0.0:3900"
curl -s http://localhost:3900/ | grep -o 'Hello from Postgres'   # found; also via http://192.168.0.14:3900
cargo test -p rules                           # 2 passed
cd data && cargo sqlx prepare --check         # exit 0 (metadata current)
SQLX_OFFLINE=true cargo leptos build          # OK, also with DATABASE_URL unset / pointing at a dead port
```

In a browser the page hydrated, showed "Hello from Postgres", and the console only logged `[HOT RELOADING] Connected to server.`. `cargo clippy -p server -p rules -p data` and `cargo fmt --all --check` are clean; `cargo tree -p frontend --target wasm32-unknown-unknown -i sqlx` finds no sqlx.

Tools installed: `cargo-leptos 0.3.11` (`cargo install --locked cargo-leptos@0.3.11`, about 8 minutes) and `sqlx-cli 0.9.0` (`--no-default-features --features rustls,postgres`, without `--locked`), both in `~/.cargo/bin`. `rustup default` stays `stable`; the pin in `rust-toolchain.toml` takes over inside the repo.

### Deviations from the research

- Port 3900/3901 on `0.0.0.0` instead of the template's `127.0.0.1:3000`/`3001`; set in the manifest, not through `LEPTOS_*` env vars.
- `app` does **not** depend on `leptos_axum` (research pitfall 6); add it back to `app`'s `ssr` feature when `app` needs `redirect`/`ResponseOptions`.
- Added `dotenvy` to `server` so `cargo leptos watch` picks up `DATABASE_URL` from `.env`.
- Dropped the template's `env = "DEV"` metadata key (cargo-leptos 0.3.11 ignores it with a warning).
- The seed row is part of the placeholder migration; real game-data seeding is still for the schema ticket.

### Facts later tickets need

- The hello page uses the default **out-of-order streaming**: the raw HTML first contains `<p>Loading...</p>` and the resolved `<p class="greeting">…</p>` arrives later in a `<template>` chunk of the same response. Use `ssr=SsrMode::Async` (or `InOrder`) on a route if a page must be fully in place without JS.
- Shared DTOs live in `rules` with public fields and no sqlx derive; `data` reads into them with `query_as!`. Anything outside a `#[server]` body that names `data`/`sqlx` in `app` must be `#[cfg(feature = "ssr")]`.
- After adding or changing a query: Postgres running and migrated (start the server once, or `sqlx migrate run --source data/migrations`), then `cd data && cargo sqlx prepare`, and commit `data/.sqlx/`. Don't run `prepare --workspace` or `--all-features`.
- Migrations run on server startup from `data/migrations` (embedded at compile time, so new migration files need a rebuild).
- The `hello` table and `rules::Greeting` are placeholders; delete them (with a migration that drops the table) once a real page replaces the hello page.
- If the phone can't reach `:3900`, the macOS firewall may need to allow the `server` binary.
