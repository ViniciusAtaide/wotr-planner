# Pull the game-data lookups out of Engine

Type: task
Status: resolved
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

- 2026-11-02: baseline at HEAD `57d80c0` in the scratchpad (`t56/`): `cargo test --workspace -- --list` (`test-list-before.txt`, 663 tests with the crate headers), a `git archive HEAD` copy (`t56/old-src`), and the benchmark at HEAD (release built with `--lib-features timings` into `t56/target-old`, served on 3920; `t56/bench/results/before`). The user's edit in `apps/backend/src/main.rs` is saved as `t56/main-rs-user-edit.diff` and was not touched.
- 2026-11-02: re-grep after tickets 54–55: the field reads were the ones the Question lists, plus `features.names` and `features.daily_choices` (read through `self.features` in `labels.rs` and `lookups.rs`) and `indexes.options` in `labels.rs`'s `pick_category`. Ticket 55's `extra_pick` and `SlotId::MythicExtra` added none.
- 2026-11-02: steps 1 and 2 together. `lookups.rs` moved to `catalog.rs` (`git mv`) and became the `Catalog`; `Indexes` and `FeatureFacts` are private to it with private fields. The call sites were routed by `t56/route.py` (`self.<lookup>(` → `self.catalog.<lookup>(`, `engine.<lookup>(` → `engine.catalog.<lookup>(`), and the field reads were replaced by hand with new lookups. Walker steps that read only data hold a local `catalog` instead of `engine`.
- 2026-11-02: checks, proof and benchmark passed (see Answer). Nothing committed; no DB change.

## Answer

**Done.** `Engine` now holds a `Catalog` and the `LinkKeys`, nothing else. The `Catalog` owns the canonical `GameData`, `Indexes` and `FeatureFacts`, and every lookup by key is its method. No field of `Indexes` or `FeatureFacts` is visible outside `engine/catalog.rs`. The public API, the test list and every engine answer are unchanged, and the benchmark is within noise.

### Layout (lines)

| File | Lines | Holds |
| --- | ---: | --- |
| `engine.rs` | 353 | as before, but the struct is `Engine { catalog, link_keys }`, with a short `impl Engine` for the public data methods: `new`, `data`, `parameter_values`, `feat_list` and `daily_choices`, each a line over the catalog |
| `engine/catalog.rs` (was `lookups.rs`) | 609 | `Catalog` (fields `data`, `indexes`, `features`, all private): `new`, `data`, the lookups by key, the values, rows and casting, the feature facts; `Indexes`, `FeatureFacts`, `ClassAt`, `SelectionAt` and the `index_*` builders, all private |
| `engine/slots.rs` | 332 | an `impl Catalog` block (what a slot accepts) and `check_pick(&Catalog, …)`; a small `impl Engine` for the public `slot_accepts` and `slot_ignores_prerequisites` |
| `engine/switch.rs` | 75 | as before, through `self.catalog` |
| `engine/{labels,pickers,removable}.rs`, `engine/walk.rs`, `engine/walk/*` | | as before, reaching data through `self.catalog` (the engine) or `self.engine.catalog` / a local `catalog` (the walker) |

### The catalog's methods

- **Moved from `Engine`**, unchanged: `class`, `race`, `background`, `feat`, `weapon`, `parameter_values`, `value_fits`, `every_value`, `option_parameter`, `option_values`, `feat_list`, `selection`, `option`, `mythic_ability`, `mythic_feat`, `path`, `archetype_of`, `row`, `casts`, `spontaneous`. Also `ascension_level`, moved from `switch.rs`, since it reads only `data.progression`.
- **New, replacing a field read outside the module:**
  - `archetype(key)`: was `indexes.archetypes` in `labels.rs` (`archetype_name`, `feature_holder`);
  - `subrace(key)`, giving the subrace with its race: was `indexes.subraces` in `walk.rs` (Build start) and `labels.rs`;
  - `skill(key)`: was `indexes.skills` in `walk/level.rs` and `labels.rs`;
  - `has_option(key)`: was `indexes.options.contains_key` in `slots.rs` and `pickers.rs`;
  - `selection_of(option)`, the selection that defines it: was `indexes.options.get` in `labels.rs`'s `pick_category`;
  - `ability_bonuses`, `option_row`, `floor`: were `features.bonuses`, `features.rows` and `features.floors` in `walk/hold.rs`;
  - `features_granting_row(selection, level)`: was the scan of `features.rows` in `walk/options.rs`'s `unlocks`;
  - `feature_name`, `shares_name`, `is_daily_choice`: were `features.names` and `features.shared_names` in `labels.rs`, and `features.daily_choices` in `daily_choices`.
- **What a slot accepts** (`slots.rs`): `offers`, `pools`, `slot_lists`, `slot_accepts_in`, `slot_ignores_in`, `single_option` and `slot_accepts` are an `impl Catalog` block, and `check_pick` takes `&Catalog`. They read only the game data and the character's state, so code that needs only data takes the catalog.
- **Code that takes `&Catalog`, not `&Engine`:** `check_pick` and the walker's `advancing_levels`. The `Walker` and `Recheck` still hold `&Engine`: the walk asks the engine for the Build's path and switch (`chosen_path`, `chosen_switch`, `reaches`), and `Recheck` for `apply` and `first_level`.

One shape change in `labels.rs`: its private `Holder::Archetype` holds the archetype (`&Archetype`) instead of a class and an index, since nothing read the class.

### Public API

Unchanged. `new`, `data`, `parameter_values`, `feat_list`, `daily_choices`, `slot_accepts` and `slot_ignores_prerequisites` stay on `Engine` with their doc comments, delegating to the catalog. `Catalog` itself is `pub(super)`, not part of the crate's API. Checked with rustdoc JSON (ticket 51's `t51/api.py` and `api-docs.py`, outputs in `t56/api/`): the item lists are identical (2,564 rows; 2,569 with `--features testing`), and so are the docs.

### Naming

`Catalog`. `GameData` already names the raw lists, and "game data" is how the docs and the glossary speak of them, so the indexed whole needs another word. "Catalog" says what it is, the data listed for lookup by key, and reads well at a call site: `self.catalog.feat(key)`. It collides with no glossary term. `GLOSSARY.md` is unchanged: it lists the game's and the planner's terms (Build, Slot, Sheet, …) and no engine internals, not even `Engine`.

### Step 3

Nothing yet calls for it. Two things a future split would start from:

- **Labels read only the catalog.** Every method in `labels.rs` (names, reasons, slot labels, categories) reads only the catalog. They stay `impl Engine` because they're the public API the `ui` crate calls, and no caller needs labels without an engine.
- **The walk's ties to the engine are few.** Besides the catalog, the walker asks the engine only for `chosen_path`, `chosen_switch` and `reaches` (`switch.rs`), which also read only data. If the walk ever needs to run without an `Engine` (on the catalog alone), those three move to the catalog, and `Walker` holds `&Catalog`.

### Done when

- **Ticket 53's checks:**
  - `cargo fmt --all --check`: pass.
  - `cargo clippy --workspace --all-targets -- -D warnings`: clean. Also clean: `-p rules --features testing`, and `-p frontend --target wasm32-unknown-unknown`.
  - `cargo test --workspace`: pass. rules 338 + 4 unit; seeds files 25, game_data 168, postgres 28; extract 96; server 4.
  - `cargo build -p rules --target wasm32-unknown-unknown`: pass.
  - `SQLX_OFFLINE=true cargo leptos build`: pass.
  - `cd libs/seeds && cargo sqlx prepare --check`: pass.
  - `cargo run -p seeds --bin seed -- --check-only`: 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 38 backgrounds.
  - `cargo leptos build --release`: pass.
  - `cargo doc -p rules --document-private-items`: one warning, `options` linking to the private `named_feats`. HEAD has the same warning.
  - Log: `t56/verify.txt`. Its first clippy run caught a needless borrow (`&catalog` passed to `advancing_levels`), since fixed; the reruns are clean.
- **Test list:** `cargo test --workspace -- --list`, with the crate headers, is identical before and after: 663 tests, and `t56/test-list.diff` is empty.
- **Field-access grep** (`t56/field-grep.txt`), over `libs/rules/src` and `libs/ui/src` outside `engine/catalog.rs`:
  - It finds no `indexes.<field>` or `features.<field>` access.
  - The only `.features` reads are `CharacterState`'s and `GameData::canonical`'s own fields.
  - `Indexes` and `FeatureFacts` are named nowhere else. Neither struct, nor any of their fields, is `pub` or `pub(super)`.
- **Proof:** byte-identical. The program is ticket 55's (`t56/proof/`), built against HEAD's rules (`t56/proof-old`, on `t56/old-src`) and against the working tree's (`t56/proof-new`). Both ran on the current `libs/game-data` and the benchmark's links.
  - 1,221 Builds: ticket 47's proof Builds; the benchmark's 6; the full and Path-switched Builds; and tickets 49, 54 and 55's Builds.
  - Compared for each Build: possible or not, the sheet, `removable`, every slot's options with statuses, the class, path and switch pickers, and `allows` over the edit battery. `is_removable`, `removable_classes` and `allows_many` are checked against those answers, and ticket 48's shortcut against `allows` on 47,030 feat options.
  - `proof-old.jsonl` and `proof-new.jsonl` are identical: 6,678,026 B, md5 `c6a1c647…`, the same as ticket 55's proof C.
  - Run time: 48.8 s before, 50.5 s after (8 threads).
- **Benchmark:** within noise.
  - Method: ticket 48's `run-all.sh` on release builds made with `--lib-features timings`. HEAD's build ran on 3920 (`t56/target-old`), before any edit (`results/before`). The working tree's ran on 3921 (`t56/target-new`, `results/after`), then HEAD's again, back to back (`results/before2`). Each run waited for a 1-minute load average under 5, and every server was stopped after its run.
  - Table: `t56/bench/results/compare.md` (`scripts/compare56.py`).
  - Medians at 4× throttling, HEAD / HEAD again → now, in ms:

  | Step (legend40) | fold ×4 | desktop ×4 | engine at fold ×4 |
  |---|---|---|---|
  | Edit: pick at level 1 (paint) | 21 / 21 → **21** | 21 / 21 → **20** | `removable_classes` 5.7 → 6.1, `evaluate` 1.4 → 1.3 |
  | Level-1 feat picker opens | 36 / 38 → **39** | 37 / 37 → **38** | `options` 8.7 → 8.9 |
  | Edit: point-buy step | 25 / 26 → **26** | 26 / 26 → **26** | `allows_many` 9.5 → 9.8 |
  | Remove a level / add one | 18 / 17 → **18**, 22 / 22 → **22** | 18 / 17 → **18**, 22 / 22 → **22** | `removable_classes` 4.4 → 4.5 |

  - The picker's per-run engine times are the same: `options` at 13.2, 10.2, 8.8, 7.8 and 8.9 ms against HEAD's 13.3, 10.1, 8.7, 7.9 and 8.3. The two HEAD runs differ from each other by up to 2–3 ms on a step, as much as the working tree differs from either.
  - Every other step and the cold loads are within ±3 ms.
  - The release wasm is 1,480,666 → 1,481,001 B.
- **The user's edit:** `apps/backend/src/main.rs` is unchanged. Its diff is still the one saved in `t56/main-rs-user-edit.diff`.

## Decisions on the questions

**User, 2026-11-02:** all recommendations ("Yes, run overnight": take every recommendation without asking). Both kept as built: `slots.rs` stays an `impl Catalog` block, and `Walker` keeps `&Engine` until something needs the walk without one.
