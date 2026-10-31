# Leptos and toolchain setup (researched 2026-10-31)

Answers [issue 02](../issues/02-leptos-toolchain.md). Every claim cites the source it came from. "Verified" means it was
reproduced in a throwaway workspace that mirrors the layout below
(`/private/tmp/claude-501/-Users-thaynamarcela-code/a3a3a839-3248-4d99-8b63-aebebfc09811/scratchpad/wsprobe`,
temporary; the manifests in §3 are copied from it verbatim).

## 1. Versions (as of 2026-10-31)

| Crate / tool | Use | Latest stable | Notes | Source |
| --- | --- | --- | --- | --- |
| `leptos` | 0.8.23 | 0.8.23 (published 2026-10-31) | `0.9.0-rc1` also published 2026-10-31; MSRV 1.88 | crates.io API `/api/v1/crates/leptos/versions` |
| `leptos_router` | 0.8.17 | 0.8.17 | | crates.io API |
| `leptos_meta` | 0.8.7 | 0.8.7 | | crates.io API |
| `leptos_axum` | 0.8.10 | 0.8.10 (2026-06-25) | | crates.io API |
| `server_fn` | (transitive) 0.8.13 | 0.8.13 | `leptos 0.8.23` requires `^0.8.13` | crates.io API `/crates/leptos/0.8.23/dependencies` |
| `cargo-leptos` | 0.3.11 | 0.3.11 (2026-10-02) | MSRV 1.82 | crates.io API |
| `axum` | 0.8.9 | 0.8.9 | | crates.io API |
| `tokio` | 1.54 | 1.54.1 | | crates.io API |
| `sqlx` / `sqlx-cli` | 0.9.0 | 0.9.0 (2026-05-21 on crates.io; CHANGELOG dated 2026-05-06) | **MSRV 1.94** | crates.io API; [sqlx CHANGELOG](https://github.com/transact-rs/sqlx/blob/main/CHANGELOG.md) |
| `wasm-bindgen` | 0.2.130 | 0.2.130 (2026-10-30) | resolved by `cargo` in the probe | crates.io API; probe `Cargo.lock` |

### Leptos 0.8 vs 0.9

- `0.9.0-rc1` shipped today. Its release notes say 0.9 "should be mostly compatible with most user application code" and
  list the breaking bits: server functions honour the body-size limit (multipart included), lazy-loading moved behind a
  `lazy` feature, `serde_qs` 1.0 for form encoding, `on:toggle` typed as `ToggleEvent`, signal trait-impl changes
  (sometimes `foo=*signal`), and **function-call syntax for signals now works on stable**.
  Source: [GitHub release v0.9.0-rc1](https://github.com/leptos-rs/leptos/releases/tag/v0.9.0-rc1).
- The official templates still target 0.8: `start-axum` describes itself as "Starter template for use with Leptos 0.8 and
  Axum"; both templates pin `0.8.x`. Source: GitHub API `repos/leptos-rs/start-axum`,
  [start-axum `Cargo.toml`](https://github.com/leptos-rs/start-axum/blob/main/Cargo.toml),
  [start-axum-workspace `Cargo.toml`](https://github.com/leptos-rs/start-axum-workspace/blob/main/Cargo.toml).
- **Recommendation:** start on **0.8.23** (stable, matches the templates); move to 0.9.0 once it leaves RC. The upgrade
  should be small per the release notes.

### What the `nightly` feature gives

On `leptos = 0.8.23`, `nightly = ["leptos_macro/nightly", "reactive_graph/nightly", "tachys/nightly"]`
(crates.io API, version features). It is active only when the compiler is nightly too (`all(feature = "nightly",
rustc_nightly)`; `rustc_nightly` is set by each crate's `build.rs`). Concretely:

1. **Call syntax for signals** — `count()` instead of `count.get()`, `set_count(1)` instead of `set_count.set(1)`. It
   uses the unstable `fn_traits` + `unboxed_closures` features. Source: `reactive_graph-0.2.16/src/lib.rs` lines 71–72
   and the `// this can be count() on nightly` doc comments in `src/signal.rs` (crate tarball from static.crates.io).
2. **Static-string view optimisation** — string literals in `view!` (text and attribute values) compile to
   `Static::<"...">` const-generic types via `adt_const_params` / `unsized_const_params`. Source:
   [`tachys/src/lib.rs`](https://github.com/leptos-rs/leptos/blob/main/tachys/src/lib.rs) lines 9–15 and
   [`leptos_macro/src/view/mod.rs`](https://github.com/leptos-rs/leptos/blob/main/leptos_macro/src/view/mod.rs)
   (`text_to_tokens`).

Both templates add `features = ["nightly"]` to `leptos` **and** `leptos_router` when you answer "Yes" to "Use nightly
features?" (template `Cargo.toml` + `cargo-generate.toml`). In 0.9 point 1 becomes stable, so `nightly` will only buy
the static-string optimisation (0.9.0-rc1 release notes; `reactive_graph` on `main` no longer gates anything on
`nightly`).

## 2. Toolchain: pin `nightly-2026-10-30`

```toml
# rust-toolchain.toml
[toolchain]
channel = "nightly-2026-10-30"
components = ["rustfmt", "clippy", "rust-src", "rust-analyzer"]
targets = ["wasm32-unknown-unknown"]
profile = "minimal"
```

- `nightly-2026-10-30` is `rustc 1.101.0-nightly (92b5b8c87 2026-10-29)`, LLVM 23.1.4 (verified: `rustup toolchain
  install nightly-2026-10-30 --profile minimal --target wasm32-unknown-unknown --component rustfmt,clippy,rust-src`).
- Component availability: for `aarch64-apple-darwin` every component (rustc, cargo, rust-std, rustfmt, clippy,
  rust-analyzer, miri, llvm-tools …) is present on every nightly 2026-10-25 → 2026-10-31; `rust-std` for
  `wasm32-unknown-unknown` is present on all those dates too. Source:
  [rustup-components-history aarch64-apple-darwin](https://rust-lang.github.io/rustup-components-history/aarch64-apple-darwin.html),
  [wasm32-unknown-unknown](https://rust-lang.github.io/rustup-components-history/wasm32-unknown-unknown.html).
- Why the 30th, not the 31st: the 30th is the newest nightly that has been out a full day with complete components and
  that I built against. Either works; the pin just has to be a fixed date so builds are reproducible.
- MSRV floor is comfortably met: sqlx 0.9 needs 1.94, Leptos 1.88, cargo-leptos 1.82 (crates.io `rust_version`; sqlx
  CHANGELOG "Breaking" section).
- The templates themselves just say `channel = "nightly"` (unpinned) and only ship `rust-toolchain.toml` when you pick
  nightly features (start-axum `rust-toolchain.toml`, `cargo-generate.toml` `ignore = ["rust-toolchain.toml"]` when
  `nightly == "No"`). Pinning a date is our choice, not the template's.
- wasm target: the Leptos book's setup step is `rustup target add wasm32-unknown-unknown`
  ([book: Getting Started](https://book.leptos.dev/getting_started/index.html)); the `targets` key above does that
  automatically.

### Verification on `nightly-2026-10-30` (aarch64-apple-darwin)

**Verified end to end** (2026-10-31) in a probe workspace with exactly the layout and manifests of §3:

| Step | Result |
| --- | --- |
| `cargo install --locked cargo-leptos@0.3.11` on the pinned nightly | OK, ~8 min (release build) |
| `cargo build -p server` (native, `leptos` + `leptos_router` with `nightly`) | OK, 1m25s cold |
| `cargo build -p frontend --target wasm32-unknown-unknown` | OK, 46s cold; `sqlx` absent from the wasm dependency graph (`cargo tree -i sqlx` finds nothing) |
| `cargo leptos build` | OK, 1m06s; cargo-leptos auto-installed `wasm-bindgen-0.2.130` ("Using wasm-bindgen version 0.2.130 detected in project") |
| `cargo leptos serve` + Postgres 18 in docker | Migrations ran at startup (`_sqlx_migrations` row `success = t`); SSR HTML rendered; the `#[server]` fn returned rows from Postgres through the context pool; in the browser the page hydrated, the nightly `set_count(count() + 1)` button counted, the `Suspense` list resolved, no console errors |

Output seen worth knowing: cargo-leptos 0.3.11 prints `Metadata keys ["env"] from metadata.leptos are not recognized
and will be ignored` for the template's `env = "DEV"` line in `[[workspace.metadata.leptos]]` — drop it. It builds the
wasm lib into a separate target dir (`--target-dir=target/front`), so the server and wasm builds don't fight over
features or the lock.

Not verified: `cargo sqlx prepare` / offline build (sqlx-cli not installed; the probe compiled `query_as!` online against
the live database). The offline rules below come from the sqlx docs and the `sqlx-macros-core` source.

## 3. Templates and workspace layout

- Two official templates, both maintained under `leptos-rs`, neither archived (GitHub API, 2026-10-31):
  [`start-axum`](https://github.com/leptos-rs/start-axum) (single crate, `cdylib + rlib`, `ssr`/`hydrate` features) and
  [`start-axum-workspace`](https://github.com/leptos-rs/start-axum-workspace) (crates `app`, `frontend`, `server`).
  The cargo-leptos README lists both as the Axum starters
  ([cargo-leptos README](https://github.com/leptos-rs/cargo-leptos#readme)). There is no newer successor.
- `start-axum-workspace` shape (its `Cargo.toml` + member manifests):
  - `[[workspace.metadata.leptos]]` with `bin-package = "server"` and `lib-package = "frontend"`.
  - `app` = all components; features `hydrate = ["leptos/hydrate"]`, `ssr = ["leptos/ssr", "leptos_meta/ssr",
    "leptos_router/ssr", "dep:leptos_axum"]`.
  - `frontend` = `crate-type = ["cdylib", "rlib"]`, depends on `app` with `features = ["hydrate"]`, contains only the
    `#[wasm_bindgen] fn hydrate()` entry (`leptos::mount::hydrate_body(App)`).
  - `server` = binary, depends on `app` with `features = ["ssr"]`, builds the Axum router with
    `generate_route_list(App)` + `.leptos_routes(...)` + `file_and_error_handler(shell)`.
  - Because the feature choice lives in `frontend`/`server`, `bin-features`/`lib-features` stay empty.
- Note: the workspace template is older than the single-crate one (last push 2025-12-23 vs 2026-03-06; still
  `edition = "2021"`, `resolver = "2"`, versions `0.8.2`). Bump versions when copying it.

### Recommended layout for wotr-planner

Five crates — the four planned ones plus the template's thin `frontend` wasm entry crate (keeps `app` an ordinary
`rlib` and matches the official template):

```
wotr-planner/
  Cargo.toml            # workspace + [[workspace.metadata.leptos]]
  rust-toolchain.toml
  .env                  # DATABASE_URL=postgres://..., SQLX_OFFLINE optional
  docker-compose.yml
  app/                  # Leptos components + #[server] fns; features ssr / hydrate
  frontend/             # cdylib: #[wasm_bindgen] hydrate()  -> lib-package
  server/               # Axum main()                         -> bin-package
  rules/                # pure Rust + serde; no features; compiled to both targets
  data/                 # sqlx models, queries, migrations/, seed; native only
    migrations/
    .sqlx/              # offline query metadata (checked in)
  style/  public/
```

Dependency edges (verified by building it):

| crate | depends on | wasm? |
| --- | --- | --- |
| `rules` | `serde` | yes (no features) |
| `data` | `rules`, `sqlx` (unconditional) | **never** |
| `app` | `leptos*`, `rules`, `serde`; `data` + `leptos_axum` **optional, only via `ssr`** | yes with `hydrate` |
| `frontend` | `app` (`hydrate`), `leptos` (`hydrate`), `wasm-bindgen`, `console_error_panic_hook` | yes |
| `server` | `app` (`ssr`), `data`, `leptos` (`ssr`), `leptos_axum`, `axum`, `tokio` | no |

Key manifest pieces (exactly what was built):

```toml
# Cargo.toml (workspace)
[workspace]
resolver = "3"
members = ["app", "frontend", "server", "rules", "data"]

[workspace.package]
edition = "2024"
version = "0.1.0"

[workspace.dependencies]
leptos = { version = "0.8.23", features = ["nightly"] }
leptos_meta = "0.8.7"
leptos_router = { version = "0.8.17", features = ["nightly"] }
leptos_axum = "0.8.10"
axum = "0.8.9"
tokio = { version = "1.54", features = ["rt-multi-thread", "macros"] }
serde = { version = "1", features = ["derive"] }
sqlx = { version = "0.9.0", default-features = false, features = ["runtime-tokio", "postgres", "macros", "migrate"] }
wasm-bindgen = "=0.2.130"
console_error_panic_hook = "0.1.7"
rules = { path = "rules" }
data = { path = "data" }
app = { path = "app", default-features = false }

[profile.wasm-release]
inherits = "release"
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"

[profile.dev.package.sqlx-macros]
opt-level = 3

[[workspace.metadata.leptos]]
name = "wotr-planner"
bin-package = "server"
lib-package = "frontend"
site-root = "target/site"
site-pkg-dir = "pkg"
style-file = "style/main.css"
assets-dir = "public"
site-addr = "127.0.0.1:3000"
reload-port = 3001
bin-features = []
bin-default-features = false
lib-features = []
lib-default-features = false
lib-profile-release = "wasm-release"

# app/Cargo.toml
[dependencies]
leptos.workspace = true
leptos_meta.workspace = true
leptos_router.workspace = true
leptos_axum = { workspace = true, optional = true }
rules.workspace = true
data = { workspace = true, optional = true }
serde.workspace = true

[features]
default = []
hydrate = ["leptos/hydrate"]
ssr = ["leptos/ssr", "leptos_meta/ssr", "leptos_router/ssr", "dep:leptos_axum", "dep:data"]
```

Sources for the pieces: `wasm-release` profile and `lib-profile-release` from start-axum `Cargo.toml`;
`[profile.dev.package.sqlx-macros] opt-level = 3` from the [sqlx README](https://github.com/transact-rs/sqlx#readme)
("Compile-time verified queries do quite a bit of work at compile time"); all `[[workspace.metadata.leptos]]` keys from
start-axum-workspace `Cargo.toml` and the cargo-leptos README. `edition = "2024"` / `resolver = "3"` are our choice
(the probe built with them); the templates still use 2021 / 2.

## 4. Wiring sqlx into Leptos server functions

### Pool: Leptos context, provided by `leptos_routes_with_context`

The Leptos book: "Context can often be used instead of `State` to provide shared server data (for example, a database
connection pool)", read back with `use_context::<T>()` in the server function
([book: Extractors](https://book.leptos.dev/server/26_extractors.html)).

`leptos_axum 0.8.10`'s `leptos_routes_with_context` both renders routes **and registers every server function route**
(`for (path, method) in server_fn::axum::server_fn_paths()` → `handle_server_fns_with_context(cx_with_state, req)`), so
one `provide_context` closure covers SSR-time calls and browser-originated calls alike. Source:
`leptos_axum-0.8.10/src/lib.rs` lines ~1797–1830. (The doc comment on `handle_server_fns_with_context` warns that the
same context must be provided to both handlers if you wire them separately — using `leptos_routes_with_context` alone
avoids that.)

```rust
// server/src/main.rs (verified)
let pool = data::connect(&db_url).await.expect("db");   // PgPoolOptions + MIGRATOR.run
let app = Router::new()
    .leptos_routes_with_context(
        &leptos_options,
        routes,
        { let pool = pool.clone(); move || provide_context(pool.clone()) },
        { let opts = leptos_options.clone(); move || shell(opts.clone()) },
    )
    .fallback(leptos_axum::file_and_error_handler(shell))
    .with_state(leptos_options);

// app/src/lib.rs (verified)
#[server]
pub async fn list_feats() -> Result<Vec<rules::Feat>, ServerFnError> {
    let pool = expect_context::<data::PgPool>();
    data::list_feats(&pool).await.map_err(|e| ServerFnError::new(e.to_string()))
}
```

The `#[server]` body may name `data::` without `#[cfg(feature = "ssr")]`: "For the client-side/browser build, the body of
the server function is stubbed out with an HTTP request"
([book: Server Functions](https://book.leptos.dev/server/25_server_functions.html)). Verified: `app` compiled for wasm
with `data` absent from the dependency graph. Any *other* item that touches `data`/`sqlx` (helpers, `use` lines at
module scope, `impl From<Row>`) must be `#[cfg(feature = "ssr")]`.

Alternative: Axum `State` via `extract_with_state` — the book says you then "need to provide the state both via context
and via Axum router state" with `FromRef`. More ceremony for no gain here.

### Types shared with the browser

Put the serde DTOs (`Feat`, `Build`, …) in `rules` (no sqlx). In `data`, read straight into them with
`sqlx::query_as!(rules::Feat, "SELECT id, name FROM feats")`: the macro builds a struct literal, so the target needs
public fields but **no `sqlx::FromRow` derive** (verified compiling against Postgres 18). If a type does need a sqlx
derive, gate it: `#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]` in a crate where `sqlx` is optional — but the
`rules`-owns-types / `data`-owns-queries split avoids needing that.

### Offline query checking

From the [sqlx-cli README](https://github.com/transact-rs/sqlx/blob/main/sqlx-cli/README.md):

- `cargo sqlx prepare` writes query metadata to `.sqlx/` in the current directory; `--workspace` writes a single
  `.sqlx/` at the workspace root. Check it in. `cargo sqlx prepare --check [--workspace]` fails when stale.
- "The presence of a `DATABASE_URL` environment variable will take precedence over the presence of `.sqlx`"; set
  `SQLX_OFFLINE=true` to force offline.
- Queries behind feature flags or in tests are only seen if you pass cargo args: `cargo sqlx prepare -- --all-targets --all-features`.
- Offline mode is always enabled in the library (no feature needed) ([sqlx README](https://github.com/transact-rs/sqlx#readme)).

The macros look for metadata in `SQLX_OFFLINE_DIR`, then **the crate's own `.sqlx`**, then the workspace root `.sqlx`
(`sqlx-macros-core-0.9.0/src/query/mod.rs` lines 97–102). So with all queries in `data`, run
`cd data && cargo sqlx prepare` and commit `data/.sqlx/`. **Do not** use `--all-features` at the workspace level (see
pitfalls).

### Migrations

- Embed and run at startup: `pub static MIGRATOR: Migrator = sqlx::migrate!();` reads `./migrations` relative to the
  crate's `Cargo.toml` at compile time; `MIGRATOR.run(&pool).await?` in `data::connect` (pattern from the Leptos
  example [`todo_app_sqlite_axum/src/main.rs`](https://github.com/leptos-rs/leptos/blob/main/examples/todo_app_sqlite_axum/src/main.rs),
  which calls `sqlx::migrate!().run(&mut conn)` before building the router).
- `sqlx-cli` 0.9.0 for authoring/inspecting: `sqlx migrate add <name> --source data/migrations`, `sqlx migrate run`,
  `sqlx migrate info --source …` (sqlx-cli README). Install:
  `cargo install sqlx-cli --no-default-features --features rustls,postgres` (README lists `native-tls,postgres` and
  `rustls` variants). **`cargo install --locked sqlx-cli` no longer works in 0.9** — the lockfile was removed from the
  repo (sqlx CHANGELOG 0.9.0 "Important Announcements").
- Recommendation: run migrations at server startup (one fewer manual step, Postgres is local) and use `sqlx-cli` only to
  create migration files and to `prepare`. Seeding can be a separate `data` bin or a migration-like step decided in the
  schema ticket.

### sqlx 0.9 feature changes to know

From the sqlx 0.9.0 CHANGELOG / README:

- Combined runtime+TLS features (`runtime-tokio-rustls`, …) were **deleted**. Use `runtime-tokio` (no TLS) plus
  optionally `tls-rustls-ring-webpki`, `tls-native-tls`, etc. Local docker Postgres needs no TLS, so
  `runtime-tokio` alone is fine. (The Leptos `todo_app_sqlite_axum` example still uses the 0.8 name
  `runtime-tokio-rustls` with `sqlx = "0.8.6"` — don't copy that line.)
- Default features are now `any, macros, migrate, json`; we turn defaults off and list `runtime-tokio, postgres, macros,
  migrate` (+ `json`/`uuid`/`time` if needed). Source: crates.io features for `sqlx/0.9.0`.
- `query*()` functions now take `impl SqlSafeStr` (only `&'static str` or `AssertSqlSafe(..)`), so `format!`-built SQL
  must be wrapped. Postgres nullability inference changed ("force generic plan"). `sqlx.toml` config is opt-in via the
  `sqlx-toml` feature.
- Repository moved from `launchbadge/sqlx` to [`transact-rs/sqlx`](https://github.com/transact-rs/sqlx) (GitHub API
  redirects; CHANGELOG announcement).

## 5. Pitfalls

1. **sqlx/tokio leaking into the wasm build.** If `ssr` (and therefore `data` → `sqlx` → `tokio`/`mio`) is enabled on
   the wasm target, the build fails with `error: This wasm target is unsupported by mio. If using Tokio, disable the net
   feature.` (verified with `cargo check -p frontend --target wasm32-unknown-unknown --features app/ssr`). Rules:
   `data`, `sqlx`, `leptos_axum`, `tokio`, `axum` are only reachable via `app/ssr` or from `server`; `frontend` depends
   on `app` with only `hydrate`; never put `ssr` in `app`'s `default` features.
2. **Feature unification.** `cargo check --workspace` (or `cargo sqlx prepare --workspace`) builds `frontend` and
   `server` in one invocation, so `app` gets `ssr` **and** `hydrate` together. On 0.8.23 this still compiles natively
   (verified), but `--all-features` / `--target wasm32 --workspace` will not. cargo-leptos avoids this because it builds
   the lib and bin packages in separate cargo invocations (`lib-package` for wasm, `bin-package` native; cargo-leptos
   README). Run `cargo sqlx prepare` inside `data/`, and run `rules` tests with `cargo test -p rules`.
3. **wasm-bindgen CLI/crate version must match exactly.** cargo-leptos downloads a `wasm-bindgen-cli` matching the
   lockfile's `wasm-bindgen`, *unless* a global `wasm-bindgen-cli` is installed, in which case you manage it yourself
   (cargo-leptos README, "wasm-bindgen version management"). Pin `wasm-bindgen = "=0.2.130"` in the workspace so
   `cargo update` doesn't move it silently, and don't install `wasm-bindgen-cli` globally.
4. **`query!` needs a database or `.sqlx` at compile time** — including when cargo-leptos builds the server. Keep
   `DATABASE_URL` in `.env` (sqlx reads it) with docker compose running, or commit `data/.sqlx` and set
   `SQLX_OFFLINE=true` (sqlx-cli README). The compile-time check also needs migrations already applied, so run the
   server (or `sqlx migrate run`) once before adding a query against a new table.
5. **`nightly` breakage risk** lives in `tachys`'s `adt_const_params`/`unsized_const_params` usage — the source comment
   says support "has now moved back and forth between these two features a couple times"
   (`tachys/src/lib.rs`). A dated pin protects us; when bumping the date, re-run `cargo leptos build`. If a future
   nightly breaks it, dropping `features = ["nightly"]` is a one-line fallback (only signal call syntax is lost on 0.8).
6. **Nightly cargo warns about unused optional dependencies** — the probe got `warning: unused dependency
   'leptos_axum'` on `app/Cargo.toml` when `app` didn't use it. Harmless; drop `leptos_axum` from `app` unless `app` uses
   it (e.g. `leptos_axum::redirect`, `ResponseOptions`).
7. **Port 3000 is already taken on this machine** — during the probe a Vite dev server from another checkout was
   listening on `:3000`. Pick a different `site-addr`/`reload-port` (e.g. `127.0.0.1:3900` / `3901`, which the probe
   used via `LEPTOS_SITE_ADDR` / `LEPTOS_RELOAD_PORT`; env names from the cargo-leptos README).
8. **Root route:** with the `path!` macro write `path!("/")`; the probe's `path!("")` fell through to the router
   fallback ("Not found.") on `/`. The start-axum template uses `StaticSegment("")` instead (start-axum `src/app.rs`).
9. **Leptos 0.9 is imminent** (rc1 today). Expect a small migration (signal-trait changes, `lazy` feature) shortly
   after this project starts (v0.9.0-rc1 release notes).
