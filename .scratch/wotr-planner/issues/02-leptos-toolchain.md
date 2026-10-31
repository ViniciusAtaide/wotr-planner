# Leptos and toolchain setup

Type: research
Status: resolved
Blocked by:
Map: [WotR Build Planner](../map.md)

## Question

What is the current recommended setup for a cargo-leptos SSR app on Axum + Postgres/sqlx, built on the latest Rust nightly?

- Latest stable Leptos and cargo-leptos versions, and any nightly-only features (for example the `nightly` feature flag) and what they give us.
- Which dated nightly to pin in `rust-toolchain.toml`, and the wasm32 target/components it needs. Check that the pinned nightly builds the starter template.
- The state of the official Axum starter template (`start-axum` or its successor) and how a workspace layout fits it: app, server, a shared rules crate, a data/db crate.
- The recommended way to wire sqlx with Leptos server functions: pool in Axum state or context, offline query checking (`SQLX_OFFLINE`, `.sqlx/`), and running migrations at startup or through `sqlx-cli`.
- Known pitfalls of using sqlx in a crate that's also compiled to wasm (feature-gating `ssr`).

## Answer

Full write-up with sources: [research/leptos-toolchain.md](../research/leptos-toolchain.md).

- **Versions:** `leptos 0.8.23`, `leptos_router 0.8.17`, `leptos_meta 0.8.7`, `leptos_axum 0.8.10`, `cargo-leptos 0.3.11`,
  `axum 0.8.9`, `tokio 1.54`, `sqlx`/`sqlx-cli 0.9.0` (MSRV 1.94), `wasm-bindgen =0.2.130`. Leptos `0.9.0-rc1` came out
  today; stay on 0.8 (what the templates use) and upgrade when 0.9.0 is final. The release notes promise a small migration.
- **`nightly` feature** (on `leptos` and `leptos_router`): signal call syntax (`count()`, `set_count(1)`) via
  `fn_traits`, plus `Static::<"...">` const-generic string literals in `view!`. In 0.9 the call syntax works on stable.
- **Toolchain:** `rust-toolchain.toml` → `channel = "nightly-2026-10-30"` (rustc 1.101.0-nightly 92b5b8c87),
  `targets = ["wasm32-unknown-unknown"]`, `components = ["rustfmt", "clippy", "rust-src", "rust-analyzer"]`,
  `profile = "minimal"`. **Verified:** cargo-leptos installs, `cargo leptos build`/`serve` work, the page hydrates in a
  browser, and a server fn reads Postgres 18. Not verified: `cargo sqlx prepare`.
- **Template:** follow `leptos-rs/start-axum-workspace` (`app` + `frontend` cdylib + `server`, `[[workspace.metadata.leptos]]`
  with `lib-package = "frontend"`, `bin-package = "server"`). Add `rules` (serde only, both targets) and `data` (sqlx,
  native only). So it's five crates: keep the template's thin `frontend` wasm entry crate. Bump its stale versions and
  drop its `env = "DEV"` key, which cargo-leptos 0.3.11 ignores with a warning.
- **sqlx wiring:** `app` has `data` and `leptos_axum` as optional deps, enabled only by `ssr`; `frontend` enables only
  `hydrate`. The server builds a `PgPool` and runs `sqlx::migrate!()` (migrations in `data/migrations`) at startup.
  It passes the pool through `leptos_routes_with_context(..., move || provide_context(pool.clone()), ...)`, which covers
  SSR and browser server-fn calls. `#[server]` fns call `expect_context::<PgPool>()`; their bodies are stubbed out on wasm,
  so no `cfg` is needed there. Shared DTOs live in `rules`, and `data` reads them with `query_as!(rules::Feat, ...)`
  (no `FromRow` needed). Offline: `cd data && cargo sqlx prepare` and commit `data/.sqlx/`, which the macros find
  per crate. Use `SQLX_OFFLINE=true` to ignore `DATABASE_URL`. `sqlx-cli` is only for `migrate add/run/info` and
  `prepare`.
- **Pitfalls:**
  - `ssr` on the wasm target pulls in tokio/mio (`This wasm target is unsupported by mio`).
  - Workspace-wide `--all-features` unifies `ssr`+`hydrate`.
  - sqlx 0.9 removed the `runtime-tokio-rustls`-style features, and `cargo install --locked sqlx-cli` no longer works.
  - wasm-bindgen crate and CLI versions must match exactly, so don't install `wasm-bindgen-cli` globally.
  - `query!` needs a live DB with migrations applied, or `.sqlx`.
  - Port 3000 is already used by a local Vite server, so pick another `site-addr`.
  - Use `path!("/")` for the root route.
  - A future nightly could break `nightly` (`adt_const_params`). Fallback: drop the feature.
