# Create the project skeleton

Type: task
Status: open
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
