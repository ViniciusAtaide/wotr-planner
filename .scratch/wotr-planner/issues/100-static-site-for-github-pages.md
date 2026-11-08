# A static build of the app for GitHub Pages

Type: task
Status: resolved
Blocked by: none (runs alongside [99](99-switch-rules-beside-their-data.md), in its own worktree)
Map: [WotR Build Planner](../map.md)

## Question

The user asked for a production build deployed on a fitting platform (2026-11-08), and suggested GitHub Pages over Fly.io. Until now the map ended with the app running locally.

Pages serves static files only. The app needs nothing dynamic at runtime: there are no server functions, the Build lives in the URL fragment (`/#b=…`, which the server never sees), and `/` with no fragment is a blank Build. At runtime the server only renders the first HTML, serves `/pkg` (wasm, JS, CSS), and serves the game data at `/game-data/<hash>.json`, loaded from the SQLite database. So the site can be files.

Settled with the user (2026-11-08): SQLite stays (ticket 85's note), here at build time only. Nothing is pushed, and no repository is created: the user decides the account and visibility. A free Pages site needs a public repo, `gh` here is signed in as a work account, and `libs/game-data/` would be published (`guide-cache/` is git-ignored).

## Build

- **Make the site from the release server:** the recommended approach, needing no new tool and no client-only mode. Build with `cargo leptos build --release --precompress`, seed the database, start the release server, and save:
  - its rendered `/` as `index.html`, which the wasm hydrates as it does today;
  - `/pkg`;
  - the game-data JSON.
  Together these are a site folder. Then the first paint stays server-rendered, and the game data still comes out of the database through the server's own loader. If you find a real reason this can't work (for example hydration needing per-request markup), say so, and fall back to a client-only (`csr`) build, recording why.
- **A sub-path:** a project Pages site is served at `https://<user>.github.io/<repo>/`, so every URL the page and the JS use must work under a prefix (relative paths, or a base set at build time). Pick and record which. The site works the same at `/` and under `/wotr-planner/`.
- **Compression:** Pages gzips files itself. Leave out the `.br`/`.gz` files, unless something needs them.
- **A script**, `scripts/build-site.sh <out-dir>`, does the whole build locally and in CI, with its usage in its header.
- **The workflow:** `.github/workflows/pages.yml` runs on pushes to `main` and by hand. It:
  - installs the pinned nightly (`rust-toolchain.toml`), the `wasm32-unknown-unknown` target and cargo-leptos 0.3.11;
  - sets `SQLX_OFFLINE=true`;
  - runs `scripts/build-site.sh`;
  - uploads with `actions/upload-pages-artifact` and deploys with `actions/deploy-pages` (pin each action to a major version).
  It can't run locally: check it with a YAML parser, and read each step against what the script needs.
- **The steps left for the user:** a short "Deploying to GitHub Pages" section in this ticket's Answer:
  - create the repo (which account, public);
  - `git remote add` and push;
  - Settings › Pages › Source: GitHub Actions;
  - the URL the site will have.
- **The map** gets its Deployment line updated when this merges (the orchestrator does that; don't edit `map.md`).

**Done when:**
- `scripts/build-site.sh` builds the site from a clean checkout.
- The local check:
  - serve the folder with a static file server (`python3 -m http.server`) under a sub-path that mimics Pages;
  - the app loads, hydrates, opens pickers and makes a Build, a `#b=` link reloads to the same Build, and there are no console errors and no 404s (`read_network_requests`).
- The workflow parses, and its steps match the script.
- `scripts/check.sh` passes.

## Choices settled

1. **From the release server, as recommended.** Nothing needed a client-only build: the server's first HTML is the same for every request (the sheet renders "Loading the game data…" until the wasm has the data, and the Build is in the fragment, which the server never sees), so one saved `index.html` hydrates like the served page. The only per-request markup is leptos's nonce, which is inert without a Content-Security-Policy.
2. **Sub-path: relative URLs, not a base set at build time.** The page names its files relative to itself: `<HydrationScripts root=".">` (`./pkg/wotr-planner.js`, `./pkg/wotr-planner.wasm`, and the inline script's `import`), the stylesheet `./pkg/wotr-planner.css`, and the game data `./game-data/<hash>.json` (`GameDataUrl` is now relative; the server still serves it at `/game-data/<hash>.json`, which is where that resolves from `/`). So one folder works at `/` and under any prefix, with no repo name baked in, and the dev and release servers work as before.
3. **The router is gone.** `App` had one route, `/`, and the Build lives in the fragment. Under `/wotr-planner/` that route matched nothing, so the client would have rendered the router's "Not found." over the server's sheet. `App` now renders the sheet directly, whatever the path, and `leptos_router` left the `ui` crate and the workspace. The server still serves the page at `/` only (leptos_axum's default route when the app has no routes); its fallback serves a site-root file or a plain "Not found." 404 rather than rendering the app.
4. **No favicon request.** The head has `<link rel="icon" href="data:,">`: without an icon link a browser asks for `/favicon.ico` at the host's root, which on a project site is outside the site (a 404).
5. **The script** (`scripts/build-site.sh <out-dir>`): seeds a new database in a temporary folder (`DATABASE_URL` there; the dev database and `.env` are never read or touched), builds with `SQLX_OFFLINE=true` against `libs/seeds/.sqlx`, runs `cargo leptos build --release` **without `--precompress`** (its `.br`/`.gz` files would only be thrown away; Pages compresses by itself), starts `target/release/server` on a free 127.0.0.1 port (python3 picks it), saves `/` as `index.html` and the game data the page names, and copies `target/site/pkg` minus `.br`, `.gz` and wasm-bindgen's `.d.ts`. It fails if the page names a file by an absolute path, or names a `./…` file the folder lacks. `<out-dir>` must be missing or empty.
6. **The workflow** (`.github/workflows/pages.yml`): on pushes to `main` and `workflow_dispatch`; `pages: write` and `id-token: write`; one `pages` concurrency group that doesn't cancel a running deployment. The build job: `actions/checkout@v5`; `rustup toolchain install` (reads `rust-toolchain.toml`) and `rustup target add wasm32-unknown-unknown`; `Swatinem/rust-cache@v2` (caches `~/.cargo`, cargo-leptos's binary included, and `target/`); `cargo install --locked cargo-leptos --version 0.3.11`; `scripts/build-site.sh site` with `SQLX_OFFLINE=true`; `actions/upload-pages-artifact@v4` with `path: site`. The deploy job: `actions/deploy-pages@v4` in the `github-pages` environment. `actions/configure-pages` is left out: with relative URLs nothing needs the site's base path.
7. **Tests** (the server's seam, `apps/backend/src/main.rs`; `main` now builds its router with `app(game_data, leptos_options)`): the page names its stylesheet, scripts, wasm and game data relative to itself and nothing by an absolute path; the game data is at the URL the page names; any other path is a plain 404; the page names its icon. The router's removal is a client-side change no server test can see, so the browser check covers it.

## Progress log

- 2026-11-08: started at HEAD 8443e4a, clean tree, in its own worktree (ticket 99 ran alongside). Read `CLAUDE.md`, the map's Notes, this ticket, the server, the frontend, the leptos metadata and `scripts/check.sh`.
- 2026-11-08: test-first in the server. `the_page_names_its_files_relative_to_itself` was red against the absolute `/pkg/…` and `/game-data/…` URLs, green with `root="."`, the relative stylesheet and the relative `GameDataUrl`. Removing the router made `any_other_page_is_not_found` red (the fallback rendered the sheet), green with the plain 404. `the_page_names_its_icon_so_the_browser_asks_for_none` red, then green with the icon link.
- 2026-11-08: `scripts/build-site.sh` into a folder under `$TMPDIR`: 2.9 MB, five files. Rebuilt with `.env` and the dev database moved away: the same site.
- 2026-11-08: local check with `python3 -m http.server`, the site at `/wotr-planner/` (port 3920) and at `/` (port 3921), in the built-in browser: both load and hydrate, pickers open (race, deity, level 1 class), a Build updates the `#b=` fragment and a reload of that link opens the same Build, no console messages, and every request the page made was a 200 (or 304). The only 404s in the server logs were `/favicon.ico`, asked by the browser pane itself (not in the page's network log, and still asked with the icon link in place).
- 2026-11-08: the workflow parses (Ruby's YAML); its steps read against what the script needs (toolchain, wasm target, cargo-leptos, `SQLX_OFFLINE`, curl and python3, both on `ubuntu-latest`). `scripts/check.sh`: ALL-OK.

## Answer

Done 2026-11-08. `scripts/build-site.sh <out-dir>` builds the app as a static site from the release server: `index.html` (the server-rendered page, which hydrates), `pkg/` (wasm, JS, CSS) and `game-data/<hash>.json`, about 2.9 MB (1.6 MB of JSON, 1.4 MB of wasm, before Pages' gzip). The page names every file relative to itself, so the same folder works at `/` and under `/<repo>/`; for that the app lost its one-route router. `.github/workflows/pages.yml` builds and deploys it on every push to `main`, or by hand.

### Deploying to GitHub Pages

1. **Create the repository** on GitHub under the account you choose (your personal account rather than the work account `gh` is signed in as here, if that's the intent). A free Pages site needs it **public**, which publishes the code and `libs/game-data/` (`guide-cache/`, the database and `.env` are git-ignored). Create it empty: no README, licence or `.gitignore`.
2. **Push:** from the main checkout, `git remote add origin https://github.com/<user>/<repo>.git`, then `git push -u origin main`.
3. **Settings › Pages › Build and deployment › Source: GitHub Actions.** The push in step 2 may already have run the workflow and failed at the deploy step before this was set; re-run it from the Actions tab (Pages › Run workflow).
4. **The site** is at `https://<user>.github.io/<repo>/` (for example `https://<user>.github.io/wotr-planner/`). The first run takes longer (installing cargo-leptos and building everything, perhaps 15–20 minutes); later runs use the cache.

Only GitHub can show: that the workflow runs (the `rustup toolchain install` step, the cache, cargo-leptos's install on Linux, the deploy), the time it takes, and that Pages serves the site at the project URL.
