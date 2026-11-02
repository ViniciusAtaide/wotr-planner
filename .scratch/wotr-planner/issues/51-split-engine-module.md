# Split engine.rs into submodules

Type: task
Status: open
Blocked by: 47 (and the lead's test-only-helpers change that follows it)
Map: [WotR Build Planner](../map.md)

## Question

**User, 2026-11-02** (asked from a forked session): `rules/src/engine.rs` is about 3,000 lines; organise it the way `rules/src/engine/labels.rs` already is: a child module holding a plain `impl Engine { … }` block. **No traits**: there is one engine type and nothing to swap.

Suggested split, each file its own `impl Engine` block (adjust where the code's real seams differ, and say so in the Answer):

- `engine/lookups.rs`: `Engine::new` and the lookups by key;
- `engine/walk.rs`: the `Walker` and `evaluate`;
- `engine/pickers.rs`: `options`, `class_options`, `path_options` and `switch_options`;
- `engine/removable.rs`: `removable`, `allows` and `Recheck`;
- `engine/switch.rs`: the path-switch rules.

`engine.rs` keeps the `Engine` struct, the public types it returns (`Sheet`, `LevelSheet`, `RankSheet`, `Slot`, `SlotOption`, …) unless they sit better beside their code, the module docs, and the `mod` lines. Private items shared between files become `pub(super)` (or `pub(crate)` where another module of the crate already uses them), never wider than needed.

## Build

AFK; the only agent in the repo. A **move-only refactor**: no behaviour change, no renamed public item, no new public item, the same doc comments moving with their code. Prefer moving code as whole items (`git diff --stat` and a reviewer should see moves, not rewrites). Run before [ticket 48](48-ui-performance.md), which reworks the walker and `Recheck`.

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes; the public API of `rules` is unchanged (compare `cargo doc`'s item list, or `cargo public-api` if available, before and after); no file under `rules/src/engine/` is a trait module; `engine.rs` is a short root module; the test counts are unchanged.

## Progress log

## Answer
