# Pull the game-data lookups out of Engine

Type: task
Status: open
Blocked by: 55
Map: [WotR Build Planner](../map.md)

## Question

**User, 2026-11-02:** `Engine` has become a god object. [Ticket 51](51-split-engine-module.md) split it into files (`lookups`, `slots`, `pickers`, `removable`, `switch`, `walk`, `labels`), but every `impl Engine` block still reaches every `pub(super)` method and field. The split is physical, not a separation of responsibilities. Run this after every other open ticket (53–55), so it doesn't collide with their engine changes.

Two steps, in order:

1. **Close the open fields.** Make every field of `Indexes` and `FeatureFacts` private to `lookups.rs`, and give the other modules lookup methods instead. Today, other modules read these fields directly:
   - `indexes.options`: `slots.rs`, `pickers.rs`;
   - `indexes.archetypes`: `labels.rs`;
   - `indexes.subraces`: `walk.rs`;
   - `indexes.skills`: `walk/level.rs`;
   - `features.rows`: `walk/options.rs`, `walk/hold.rs`;
   - `features.bonuses`, `features.floors`: `walk/hold.rs`;
   - `features.shared_names`: `labels.rs`.

   Re-grep, since tickets 54–55 may add more.
2. **A `Catalog` type for pure data access.** It owns the canonical `GameData`, `Indexes` and `FeatureFacts`. It holds the by-key lookups (`class`, `feat`, `option`, `selection`, `archetype_of`, `row`, `casts`, `spontaneous`, `value_fits`, `every_value`, `option_parameter`, `option_values`, `parameter_values`, `feat_list`, the feature facts, …) as its own methods, in a module of its own. `Engine` holds a `Catalog` (and `LinkKeys`). The walk, slots, pickers, removal, switch and labels code reaches data through it (`self.catalog.feat(key)`), and code that needs only data takes `&Catalog`, not `&Engine`. Name the type in `GLOSSARY.md` if the glossary lists engine terms (`Catalog` is a suggestion; pick the name that reads best next to the glossary's terms and say why).

Out of scope: splitting labels or the walk into their own types (step 3 of the discussion). That waits until a concrete need shows up. Say in the Answer whether, after step 2, something already argues for it.

## Build

AFK; the only agent in the repo. A refactor: no behaviour change and no test change beyond imports or helper paths.
- `Engine`'s public API (`pub` items the `ui` crate and the tests use) stays the same. If something has to move, say why in the Answer.
- No schema, data or link-format change.
- Use the RustRover IDE tools (`rename_refactoring`, `analyze_calls`, `search_symbol`) for the moves where they fit.

**Done when:**
- Every check from [ticket 53](53-rename-packages.md)'s Done when passes.
- `cargo test --workspace -- --list` lists the same tests before and after (diff saved in the scratchpad).
- Outside the catalog's module, a grep finds no `indexes.` or `features.` field access, and no `pub(super)` field on `Indexes` or `FeatureFacts`.
- The before/after proof is byte-identical over ticket 48's Builds. Every engine answer for every Build is unchanged (method of [ticket 48](48-ui-performance.md)).
- An edit and the level-1 picker are no slower in ticket 48's benchmark (within noise).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

## Answer
