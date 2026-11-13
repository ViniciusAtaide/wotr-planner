# The dev server serves a stale precompressed asset from an earlier release build

Type: bugfix
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Problem

Found in a QA session, 2026-11-13. Opening <http://localhost:3900> on the dev server (`cargo leptos watch`) showed "Loading the game data…" forever, with `RuntimeError: unreachable` in the console and no panic message.

`target/site/pkg/` held `wotr-planner.{js,wasm,css}.br` (and `.gz`) from an earlier `cargo leptos build --release --precompress`, which `scripts/check.sh --full`'s `leptos-release` step runs into the same site root. A later dev build rewrote only the plain `.js` and `.wasm`, and the server's `ServeDir::new(site_root).precompressed_br()` (`apps/backend/src/main.rs`, `app`) went on handing a browser that takes brotli the stale release JS and wasm, which crashed on the newer game data. The comment there assumed "A dev build has no `.br` files", which is not so after a release build in the same site root.

**Repro:** `cargo leptos build --release --precompress`, then `cargo leptos watch` after a change to the frontend or the game data; `curl -sI -H 'Accept-Encoding: br' localhost:3900/pkg/wotr-planner.js` answers `content-encoding: br` with the release build's `last-modified`. Deleting the stale `.br`/`.gz` files in `target/site/pkg/` worked around it.

## Answer

Done 2026-11-13 (branch `qa/dev-precompressed`). The site's `ServeDir` calls `precompressed_br()` only when `cfg!(debug_assertions)` is off, so a dev (debug) server never serves a `.br` file and compresses the wasm, JS and CSS on the fly, as it does for the HTML; a release server is unchanged. The comment above it and the README's release-build paragraph say so.

**`scripts/check.sh --full` left as it is.** Deleting the `.br`/`.gz` files after its `leptos-release` step would cover only that one way of leaving them (a release build by hand leaves them too) and would take them from a release server run afterwards, so the server fix alone is the simpler and safer one. A release build without `--precompress` after one with it still leaves stale `.br` files that a release server serves; the README already says to always pass the flag.

**Test.** `apps/backend/src/main.rs`, `only_a_release_server_serves_a_precompressed_copy`: a site root with `pkg/wotr-planner.js` ("fresh") beside a `.br` copy ("stale"), asked with `Accept-Encoding: br`. `site_pkg(root, precompressed)` holds the choice, so a debug run checks both arms: without `precompressed` the plain file with no `Content-Encoding`, with it the `.br` file. The server itself then answers as its build does (debug: the plain file). It failed before the fix with `(Some("br"), "stale")`.

**Not covered: the site root's other files.** The fonts and the preview image are served by `leptos_axum::file_and_error_handler`, which takes a `.br` or `.gz` copy in any build, and `--precompress` writes those copies too. A changed font or image would be served stale by a dev server after a release build. They rarely change and a stale one breaks nothing, so the fallback is left as it is.

**Behaviour checks.** No change to `rules` or `libs/game-data/`; `scripts/check.sh` passes.
