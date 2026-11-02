# Faster edits and pickers

Type: task
Status: open
Blocked by: 50, 51, 52
Map: [WotR Build Planner](../map.md)

## Question

The UI benchmark (2026-11-02, release build, snapshot 174ec14; report and scripts in the scratchpad's `bench/`: `results/summary.md`, `scripts/`, `buildlinks/`, `serve-release.sh`) found what a phone user feels. Fix #1–#4 below; leave #5 (a binary game-data format) out.

1. **`Engine::removable` on every edit** (`libs/ui/src/planner.rs` memo, read per row in `libs/ui/src/grid.rs`; `libs/rules/src/engine.rs` `removable`): walks every choice (~260 on a 40-level Build, each from its level to the end), 130–150 ms of each 150–220 ms edit at 4× CPU throttling. The grid needs only the class rows' answer; the picker header needs one choice's answer when it opens. Ask only for those (e.g. class-level removability for the grid, `is_removable(build, &choice)` for an open picker), and make the recheck walk record failures only (no sheet clones) and stop at the first failure.
2. **Opening a level-1 slot picker in a long Build** (`Engine::options`, `recheck.later` per candidate; `class_options` likewise): 150–280 ms at 4×. Re-walk only candidates a later choice could depend on (held later, named in a later prerequisite or `Not`, or granting slots, features, ability or skill changes), or show `check_pick`'s list at once and fill the "breaks later" locks a frame later. The result must stay exact: an option shown open must never break a later choice.
3. **Build start controls call `allows` repeatedly** (`libs/ui/src/start.rs` point buy, racial ability, skills panel; `libs/ui/src/picker.rs` race and background pickers): 18 calls per edit while the panel is open (+~54 ms at 4×), 47/37 on opening race/background. Share one recheck across a panel's calls (e.g. `allows_many`), fold the Build start once level 1 exists, and fill the race/background lists after the first paint as the slot picker does.
4. **Payload:** no compression, no caching, and the game data downloads only after hydration (15.3 s cold on slow 4G, 3.4 s with brotli; every repeat visit re-downloads 1.5 MB, because the server fn is a POST). Add a compression layer (brotli/gzip), serve the game data as a cacheable GET (content hash, `Cache-Control: immutable`), and start that download as early as the wasm.

## Build

AFK, test-first (`tdd`) for every `rules` change; the only agent in the repo. Every engine change must keep answers identical: a before/after proof over the benchmark's Builds and ticket 42's Build sets (options' statuses, removability, `allows`, sheets byte-identical). Re-run the benchmark (`bench/scripts/run-all.sh` against a release build of the new code built with `--lib-features timings`, since release builds no longer record `__wotrTimings` otherwise; on its own port and target dir) and put a before/after table in the Answer.

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes; the proof shows identical answers; at 4× throttling on the 40-level Build an edit paints in under 60 ms median and the level-1 feat picker opens in under 60 ms median; the release game data is served compressed and cached (a repeat visit doesn't download it).

## Progress log

## Answer
