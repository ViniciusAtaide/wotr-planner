# Serve the wasm and the game data precompressed at brotli quality 11

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

A size measurement (2026-11-06; scratchpad `wasm-size/`) found the first visit's biggest saving isn't in the code. The server's `CompressionLayer::new()` (`apps/backend/src/main.rs`) compresses every response on the fly at brotli's default quality, 4. The release wasm (1,494,877 B raw) goes out as 542,223 B. Compressed once at quality 11, it would be 427,541 B, 21 % smaller, with no code change. The game data (~1.5 MB of JSON, built once at startup in `apps/backend/src/game_data.rs` and held in memory) goes out at quality 4 too. The user asked for this (2026-11-06).

- **Static assets:** cargo-leptos 0.3.11 has `--precompress` ("Precompress static assets with gzip and brotli. Applies to release builds only"). Find out what it writes (file names, which assets, which quality), and whether leptos_axum 0.8's `file_and_error_handler` serves those files for a request whose `Accept-Encoding` takes them (read its source in `~/.cargo/registry`). If it does, the release build only needs the flag; if not, serve `site-pkg-dir` with tower-http's `ServeDir` and `precompressed_br()`/`precompressed_gzip()` ahead of the fallback.
- **Game data:** compress the JSON once at startup at brotli quality 11 (and gzip at its best level for a browser without brotli), and serve the body that matches `Accept-Encoding`, with `Content-Encoding` and `Vary: Accept-Encoding`; plain JSON otherwise. The URL keeps naming the JSON's hash. Time the startup cost and say it in the Answer.
- `CompressionLayer` stays for everything else (the HTML). It leaves a response that already has `Content-Encoding` alone; confirm that in its source.

## Build

- **Where release builds are made:** `scripts/check.sh --full`'s `leptos-release` step and the README's release instructions, if any, use the flag (or the metadata key, if cargo-leptos has one; prefer that, so no release build forgets it). Dev builds stay as they are.
- **Tests:** the game-data route's existing tests (`apps/backend/src/game_data.rs`'s `mod tests`) gain cases for `Accept-Encoding: br`, `gzip`, both, and none: the right `Content-Encoding`, `Vary`, and a body that decompresses to the same JSON. The cache headers are unchanged.

**Done when:**
- A release build served locally (`cargo leptos serve --release` on its own port and target dir, or `cargo leptos build --release` and the server binary) sends the wasm as brotli with a body of about 428 KB, measured with `curl -H 'Accept-Encoding: br' -o /dev/null -w '%{size_download}'`, and the JS and CSS precompressed too. Give the before → after table for the wasm, the JS, the CSS and the game data (brotli and gzip).
- A browser without brotli gets gzip, and one asking for neither gets the plain bytes.
- The app still loads and works in that release build: open it with the browser pane, check the console has no errors and a pick works.
- `scripts/check.sh` passes, and `scripts/check.sh --full` once at the end.

## Progress log

- 2026-11-06: read the sources. cargo-leptos 0.3.11's `--precompress` is CLI-only (`config/project.rs:182`, `precompress: cli.precompress`; no metadata key, no env var). leptos_axum 0.8.10's `file_and_error_handler` serves `.br`/`.gz` files through `ServeDir`. tower-http 0.6.11's `CompressionLayer` skips a response that has `Content-Encoding`. `ServeDir` never sets `Vary`. Details in the Answer.
- 2026-11-06: baseline. HEAD (`7bb3117`) archived to the scratchpad (`t84/old-src`, `git archive`) and built with `cargo leptos build --release` (`CARGO_TARGET_DIR=t84/target`, `LEPTOS_SITE_ROOT=t84/site-old`). Served on 127.0.0.1:3930 and measured with `t84/measure.sh` (curl, `size_download` per `Accept-Encoding`; `t84/before.tsv`). Server stopped.
- 2026-11-06: game data. `GameDataFile` keeps brotli and gzip copies made at startup, picks one per `Accept-Encoding`, and sets `Content-Encoding` and `Vary`. Six tests added (see Answer). The release build with `--precompress` served the wasm at 427,827 B brotli.
- 2026-11-06: cargo-leptos's `.gz` files (libflate) turned out 11–17 % bigger than the gzip `CompressionLayer` already makes on the fly (wasm 663,778 vs 572,541 B). So the server now serves `site-pkg-dir` with its own `ServeDir` that takes only the `.br` files (`precompressed_br()`), and gzip stays on the fly. A `map_response` layer adds `Vary: Accept-Encoding` to precompressed responses, which `ServeDir` leaves without one.
- 2026-11-06: startup cost. Release: 1.63–1.72 s over 4 starts (JSON 2 ms, brotli 1,614 ms, gzip 15 ms, from a temporary timing build). Debug, quality 11: 16.3 s unoptimised, 8.5 s with the compressors at `opt-level = 3`. Brotli's generic code is monomorphised in `server`, so the dev profile override can't reach it. A debug build now uses quality 5: 131 ms.
- 2026-11-06: after. The release build on 3930 (`t84/after.tsv`, and `t84/final.tsv` from `check.sh --full`'s build, identical apart from the HTML's ±2 B). Browser pane check passed. Every server started was stopped. `target/` and `target/site` were not touched: every build, test and check used `t84/target` and `t84/site-*`. Nothing committed.

## Answer

**Done.** A release build sends the wasm at **427,827 B** brotli (was 544,169 B, −21 %), and the JS and CSS precompressed too. The game data goes out at **85,273 B** brotli (was 112,415 B, −24 %) and 116,569 B gzip (was 118,128 B). A browser without brotli gets gzip, and one that takes neither gets the plain bytes. The release server pays 1.6–1.7 s at startup to compress the game data. No change to `libs/game-data/`, the schema, the link format, the game data's URL or its cache headers (the hash is still `b27ae7841d84d211`).

### What `--precompress` writes, and who serves it

- **Writes:** after the frontend build, `compress::compress_static_files(site.root_dir)` (`cargo-leptos-0.3.11/src/command/build.rs:48–50`) walks the whole site root. Next to every file it writes `<file>.gz` and `<file>.br` (`src/ext/compress.rs:45–56`): the wasm, JS, CSS, `.d.ts` files and even `.gitkeep`.
  - gzip is libflate's default `gzip::Encoder::new` (line 46).
  - brotli is `BrotliEncoderParams::default()` (line 28), which is quality 11, window 22 (`brotli-9.1.1/src/enc/encode.rs:330–332`).
  - Release only: `if proj.release && proj.precompress` (build.rs:48).
  - The flag is CLI-only. `Opts.precompress` (`src/config/cli.rs:21–23`) goes straight to `Project.precompress: cli.precompress` (`src/config/project.rs:182`), and `config/dotenvs.rs` maps no `LEPTOS_*` variable to it. So there is no metadata key to set.
- **Served by leptos_axum:** yes. `file_and_error_handler` → `get_static_file` (`leptos_axum-0.8.10/src/lib.rs:2152–2181`) forwards `Accept-Encoding` and calls `ServeDir::new(root).precompressed_gzip().precompressed_br()`.
- **Not used as it stands:** libflate's gzip is worse than the gzip `CompressionLayer` makes on the fly (flate2 level 6): wasm 663,778 vs 572,541 B (+16 %), JS 7,154 vs 6,434 (+11 %), CSS 3,879 vs 3,305 (+17 %). Through `file_and_error_handler`, a gzip-only browser would have got the bigger file. Such a browser is the usual case for a phone opening `http://<LAN address>`, since browsers typically advertise brotli only on secure origins.
  - So `main.rs` routes `site-pkg-dir` (`site_pkg_dir_service_route_path`) to its own `ServeDir::new(site_root).precompressed_br()` ahead of the fallback.
  - A brotli request gets the `.br` file. Any other request gets the raw file, which `CompressionLayer` gzips on the fly as before. The `.gz` files are written but unused.
- **`CompressionLayer` leaves encoded responses alone:** `tower-http-0.6.11/src/compression/future.rs:42–45`: `let should_compress = !res.headers().contains_key(header::CONTENT_ENCODING) && !…CONTENT_RANGE && predicate`. When it doesn't compress, it adds no `Vary` either (lines 49–59 append `Vary` only when `should_compress`). `ServeDir` never sets `Vary` (no `VARY` anywhere in `src/services/fs/`).

### What changed

- `apps/backend/src/game_data.rs`:
  - `GameDataFile` now holds `json`, `br` and `gzip`, compressed in `from_json`. Brotli is quality 11 with window 22 in a release build (`BROTLI_QUALITY`; 5 in a debug build, see below); gzip is `flate2::Compression::best()` (9).
  - The route picks a body with `Encoding::accepted`. That takes the highest q-value among `br`, `gzip` and `x-gzip`, with brotli on a tie; `q=0` refuses a coding; unknown codings and malformed q-values are ignored, as tower-http does. tower-http's own negotiation (`content_encoding.rs`) is `pub(crate)`, so it can't be reused.
  - Every 200 has `Vary: accept-encoding`, plus `Content-Encoding` when a compressed copy is sent. `Content-Type` and `Cache-Control` are unchanged.
  - A `Display` impl gives the startup log line: `game data at /game-data/b27ae7841d84d211.json: 1543506 B JSON, 85273 B brotli, 116569 B gzip (serialised and compressed in 1680 ms)`.
- `apps/backend/src/main.rs`:
  - The `/pkg/{*path}` route serves `.br` only (above).
  - `map_response(vary_on_accept_encoding)`, inside `CompressionLayer`, adds `Vary: accept-encoding` to a response that has `Content-Encoding` and no such `Vary`.
  - The startup timing is printed.
- `apps/backend/Cargo.toml`:
  - `brotli = "9.1.1"` and `flate2 = "1.1.10"` are direct dependencies. Why: to compress the game data once at its best levels. tower-http offers no way to compress a byte buffer outside a response. Both crates were already built for tower-http's `compression-br`/`compression-gzip` (async-compression → compression-codecs), at these versions with the same default features, so `Cargo.lock` gains only the two edges and no package.
  - tower-http gains the `fs` feature for `ServeDir`. leptos_axum already enables it, so nothing new is built.
- `Cargo.toml`: `[profile.dev.package.*] opt-level = 3` for brotli, alloc-no-stdlib, alloc-stdlib, flate2, miniz_oxide and crc32fast (the sqlx-macros precedent). Unoptimised, startup compression in a debug build took 16.3 s.
- `scripts/check.sh`: the `leptos-release` step is `cargo leptos build --release --precompress`.
- `README.md`: a release-build paragraph under "Run it" (there were no release instructions): the command with `--precompress`, what it does, and that the flag has no `Cargo.toml` key.
- Dev builds are unchanged: `cargo leptos watch`/`build` write no `.br` files, so the static files are compressed on the fly as before.

Decisions taken without asking:
- Serve only the `.br` files, and keep on-the-fly gzip rather than libflate's bigger `.gz` (above).
- A debug build compresses the game data at brotli quality 5. Quality 11 took 8.5 s even with the compressors optimised, because brotli's generic code is compiled in `server` at `opt-level = 0`, and a dev server restarts on every change. Quality 5 takes 131 ms and gives 100,862 B, still smaller than the old on-the-fly 112,415 B. The tests run in debug and so exercise quality 5. They check that the body is the kept copy and that it decodes to the JSON, not the quality.
- A missing file under `/pkg/` now gets `ServeDir`'s bare 404, not the app's 404 page.

### Before → after (bytes downloaded, release build, 127.0.0.1:3930)

Before: HEAD `7bb3117`, `cargo leptos build --release`. After: this tree, `cargo leptos build --release --precompress`. Measured with `curl -H 'Accept-Encoding: …' -o /dev/null -w '%{size_download}'` (`t84/measure.sh`; `t84/before.tsv`, `t84/after.tsv`).

| asset | raw | brotli before (q4, on the fly) | brotli after (q11) | gzip before (on the fly) | gzip after |
|---|---|---|---|---|---|
| wasm | 1,494,877 | 544,169 | **427,827** (−21.4 %) | 572,541 | 572,541 (on the fly, unchanged) |
| JS | 22,041 | 6,402 | **5,507** (−14.0 %) | 6,434 | 6,434 (on the fly, unchanged) |
| CSS | 11,708 | 3,390 | **2,883** (−15.0 %) | 3,305 | 3,305 (on the fly, unchanged) |
| game data | 1,543,506 | 112,415 | **85,273** (−24.1 %) | 118,128 | **116,569** (−1.3 %, level 9 at startup) |
| HTML (`/`) | 1,500 | 726–728 | 728–730 | 773–774 | 774–776 |

- With `Accept-Encoding: gzip, deflate, br, zstd` (Chrome's), every asset came back brotli at the "after" size. With none, all came back as the raw bytes, and `Content-Encoding` was absent.
- Every response carried `Vary: accept-encoding`, including the precompressed `.br` ones (through the new layer).
- The `.br` wasm came back byte-identical to `wotr-planner.wasm.br`, with `Content-Type: application/wasm`.
- The ticket's 542,223 B "before" for the wasm was measured differently; curl's on-the-fly figure here is 544,169 B.

### Startup cost

- Release: **1.63–1.72 s** per server start (4 starts: 1,709, 1,632, 1,717, 1,639 ms, then 1,680 ms for `check.sh --full`'s build). One temporary timing build split it as JSON serialisation 2 ms, brotli q11 1,614 ms, gzip 9 15 ms. The server listens only after that.
- Debug (dev server): 131–135 ms at quality 5 with the optimised compressors.

### Tests added

In `apps/backend/src/game_data.rs`'s `mod tests`, through the routes behind `CompressionLayer` as the server has them. Each asserts the status, `Content-Encoding`, `Vary: accept-encoding`, the unchanged `Content-Type` and `Cache-Control`, that the body is the copy kept at startup (the layer didn't recompress it), and that it decompresses to the JSON (`assert_served`):
- `a_browser_that_takes_brotli_gets_the_brotli_copy`: `br`.
- `a_browser_that_takes_only_gzip_gets_the_gzip_copy`: `gzip`, and `gzip, deflate`.
- `a_browser_that_takes_both_gets_brotli`: `gzip, deflate, br, zstd`, and `br, gzip`.
- `a_browser_that_takes_neither_gets_the_plain_json`: no header, `identity`, and `deflate, zstd`.
- `the_q_values_choose_between_brotli_and_gzip`: `br;q=0, gzip` → gzip; `gzip, br;q=0.5` → gzip; equal q → brotli; both `q=0` → plain.

The old `the_game_data_is_compressed_for_a_browser_that_takes_brotli_or_gzip` (size check only) is replaced by these. The other three tests are kept.

In `apps/backend/src/main.rs`:
- `a_precompressed_file_varies_on_accept_encoding`.
- `vary_is_left_alone_when_it_names_accept_encoding_or_nothing_is_compressed`.

All 10 are listed by `cargo test -p server -- --list`.

### Browser check

Release build on 127.0.0.1:3930, in the browser pane:
- The blank Build loaded. Resource timing showed the brotli bodies: wasm 427,827, JS 5,507, CSS 2,883 and game data 85,273 B encoded.
- Picking Human as the race updated the Build start and the link.
- Adding level 1 and picking Fighter gave "Level 1 · Fighter 1" with its slots.
- The console had no messages at all.

### Checks

All with `CARGO_TARGET_DIR=t84/target` (and `LEPTOS_SITE_ROOT=t84/site-full` for `--full`), so the running dev server's `target/` was untouched.
- `scripts/check.sh`: ALL-OK (fmt, clippy, wasm-clippy, test, rules-wasm, seed-check).
- `scripts/check.sh --full`: ALL-OK, adding leptos-dev (89 s) and leptos-release with `--precompress` (41 s).
- `cargo fmt --all --check`: passes (part of check.sh).
