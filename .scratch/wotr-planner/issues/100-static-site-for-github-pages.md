# A static build of the app for GitHub Pages

Type: task
Status: open
Blocked by: none
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
