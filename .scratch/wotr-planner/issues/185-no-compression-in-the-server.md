# No compression in the server

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Ticket [183](183-dev-server-serves-stale-precompressed-assets.md) stopped a dev server serving the `.br` copies a release build leaves in `target/site/`, but left two holes: leptos's file handler still serves a `.br`/`.gz` copy of the fonts and the preview image in any build, and a release server after a release build without `--precompress` serves stale copies too. And while checking 183's fix, a browser kept an old build's CSS after a rebuild: `/pkg` had no `Cache-Control`, the file names never change, so the browser guessed a lifetime from the file's age.

The user (2026-11-13): "cargo leptos watch has to work", and "compressing should be only for publishing".

## What the server's compression bought

Nothing for the published site. `scripts/build-site.sh` saves the release server's output as plain files (curl sends no `Accept-Encoding`) and deleted any `.br`/`.gz` copies, and GitHub Pages compresses each request itself. Checked on the live site the same day, after the deploy of `82fe1fb`: the wasm is stored plain (1,977,214 B with `Accept-Encoding: identity`) and sent gzipped by Pages (756,386 B with `gzip`); the page, JS, CSS and game data likewise. Pages serves gzip only, never brotli, and ignores precompressed files.

So the compression ([ticket 84](84-precompressed-assets.md): `--precompress`, the `.br` route, the game data's brotli and gzip copies, `CompressionLayer`) only ever served a server on this machine, at the cost of today's crash on load, 1.7 s of release startup, two dependencies and six dev-profile overrides.

## Answer

Done 2026-11-13 (user: "build ticket 185 and remove all compression"). Reverses ticket 84 and replaces 183's fix.

- **`apps/backend/src/main.rs`:** no `CompressionLayer`, no `precompressed_br`, no `site_pkg` choice and no `Vary` helper. The `site-pkg-dir` route is a plain `ServeDir` with `Cache-Control: no-cache` (`revalidate`), so a browser asks each time and an unchanged file costs a 304.
- **`apps/backend/src/game_data.rs`:** the game data is the JSON alone at its hashed URL, still `immutable`; no brotli or gzip copies, no `Accept-Encoding` negotiation.
- **Dependencies:** `brotli` and `flate2` are gone from `server`, and tower-http keeps only `fs`. The six `[profile.dev.package.*]` overrides for the compressors are gone from `Cargo.toml`. (leptos_axum still builds tower-http's compression for its own use; nothing here calls it.)
- **`scripts/check.sh`:** `leptos-release` is `cargo leptos build --release`, without `--precompress`. **`scripts/build-site.sh`:** no `.br`/`.gz` to delete.
- **README:** the release build without `--precompress`, and why not to pass it.
- **Once, by hand:** the stale `.br`/`.gz` files in the main checkout's `target/site/` deleted (nothing writes them any more).

**Tests.** `the_site_files_are_served_as_they_are_and_revalidated`: a `pkg/wotr-planner.js` beside a stale `.br` copy, asked with `Accept-Encoding: gzip, deflate, br`, comes back plain, with no `Content-Encoding` and with `no-cache`. `the_game_data_is_served_at_its_url_as_it_is_to_be_kept_for_good`: the JSON as it is, `immutable`, with or without `Accept-Encoding`. The brotli/gzip negotiation and `Vary` tests went with the code.

**Trade-off, accepted:** a phone loading the dev server over Wi-Fi downloads the debug wasm uncompressed (about 25 MB, where on-the-fly brotli sent about 2.6 MB). The published site is unchanged.

**Behaviour checks.** No change to `rules` or `libs/game-data/`.
