# Give GameData its lookups

Type: task
Status: resolved
Blocked by: [82](82-remove-the-extractor.md), [77](77-one-prerequisite-walk.md)
Map: [WotR Build Planner](../map.md)

## Question

`rules::GameData` has no lookup or traversal helpers, so each crate writes its own (architecture review, 2026-11-03). `.chain(&…prestige_classes)` to walk every class appears 8 times in `libs/rules/src` and 11 times in `libs/ui`, `libs/seeds`, `libs/proof` and `apps`. The review counted more in `libs/extract`; [ticket 82](82-remove-the-extractor.md) deleted them (re-scoped after [ticket 75](75-game-data-master.md)).

## Build

- **Add the lookups to `GameData`** in `rules`, with tests: every class (base and prestige), and any by-key lookup (class, selection, option) that two or more sites write by hand. Pick the names; follow the key newtypes in `rules/src/keys.rs`.
- **Use them** at the `.chain` sites and in place of the hand-written finders.
- **Leave the data's shape alone**: base and prestige classes stay two `Vec`s.

**Done when:**
- No `.chain(&…prestige_classes)` remains outside `GameData`'s own helpers.
- `scripts/proof.sh rules <out-dir>` against `HEAD` reports no difference.
- `scripts/check.sh` passes.

## Progress log

Scratch dir (outside the repo): `/private/tmp/claude-501/-Users-thaynamarcela-code/20f0bb4f-7437-4fe9-8ef8-ebb8716f6fbe/scratchpad/t78/` (below, `t78/`).

- 2026-11-04: re-counted the `.chain(&…prestige_classes)` sites at `HEAD` (f9e87b1). 21 in all, none left in `libs/ui`, `libs/proof` or `apps` (ticket 76 took the UI's out):
  - `rules/src` (8): `game_data.rs`'s `feat_lists` and `canonicalize` (the `&mut` one); `check.rs`'s `option_levels` and `counts_toward_features`; `engine/pickers.rs`'s `class_options`; `engine/catalog.rs`'s `FeatureFacts::new`; `codec.rs`'s class keys and option keys.
  - `rules/tests/engine` (2): `check.rs`'s `class_mut`, `canonical.rs`'s `reversed` (both `&mut`).
  - `seeds/tests` (11): `files.rs` (two `class` closures); `game_data/support.rs` (`class`, `data_prerequisites` twice, `selection_option`); `class_rules.rs`, `data.rs`, `class_skills.rs` (twice), `parameterised_feats.rs`.
- 2026-11-04: what the sites walk, with each class's own items before its archetypes' and base classes before prestige: every class (5 sites, plus 3 mutable); every class's selections (2) or those then each path's (4); every class's features (2: the catalog then chains the paths'); a class by key (5, counting `check.rs`'s `counts_toward_features`); a Selection option by key (2: `support::selection_option` and `borrowed_selections.rs`'s `option_prerequisite`, which wrote it without `.chain`).
- 2026-11-04: decisions (small, settled here):
  1. **Names.** `all_classes()` and `all_classes_mut()` (a method named `classes` would sit beside the `classes` field and mean something else); `class_selections()`, `selections()` and `class_features()`, after the existing `archetypes()` and `feat_lists()`; `class(&ClassKey)` and `selection_option(&SelectionOptionKey)`, named after their key newtypes.
  2. **No second index.** `Catalog::class`, `selection` and `option` stay the engine's lookups, by index. The new by-key ones are walks of the lists for code that holds a bare `GameData` and no `Engine` (the data check, the codec's key table, the tests); their docs say so. Nothing in the engine switched to them.
  3. **`selection_option` searches every selection, paths included**, like `Catalog::option`. Its two test callers searched only classes' selections; option keys are primary keys in the seed, so each key they look up finds the same option.
  4. **No `selection(&FeatureSelectionKey)`.** Only one site (`borrowed_selections.rs`'s `any_selection`) looked a selection up in every class; it now reads `selections().find(…)`. It searched base classes only, despite its doc's "every selection"; selection keys are primary keys too, so it finds the same selections.
  5. **Left alone:** the loops that need to know whether a class is prestige (`check.rs`'s `check.class(class, is_prestige)`, `seed.rs`'s `insert_class`, the catalog's `index_classes` and `index_selections`, which record positions) and the one-off finders over one list in single tests (`data.classes.iter().find(…)`, a class's own `selections.iter().find(…)`). None is a `.chain` and none walks every class.
  6. **Seam:** the `GameData` methods, tested in a new `libs/rules/tests/engine/lookups.rs` against the test world, with the expected keys written out.
- 2026-11-04: red → green, one method at a time (each test failed to compile first): `every_class_is_the_base_classes_then_the_prestige_classes` (`all_classes`), `every_class_can_be_edited_in_place_base_and_prestige_alike` (`all_classes_mut`), `a_class_is_found_by_key_whether_base_or_prestige` (`class`), `class_selections_are_each_classs_own_then_its_archetypes_with_no_paths`, `every_selection_is_the_class_selections_then_each_paths`, `class_features_are_each_classs_own_then_its_archetypes_with_no_paths`, `a_selection_option_is_found_by_key_in_any_selection`. All seven show in `cargo test -p rules --test engine -- --list`.
- 2026-11-04: used the helpers at all 21 sites and in `borrowed_selections.rs`'s two finders. The only `.chain(&…prestige_classes)` left are inside `all_classes` and `all_classes_mut`.
- 2026-11-04: `scripts/check.sh` (with `wotr-planner-postgres` up): fmt, clippy, wasm-clippy, test, rules-wasm, seed-check OK, ALL-OK.
- 2026-11-04: `scripts/proof.sh rules t78/proof` against `HEAD` (f9e87b1): 1999 Builds each side, 0 only in A, 0 only in B, 0 differing; `compare.txt` empty. `libs/proof` had no `.chain` site and needed no change (its `selections.rs`'s `options_of` finds a base class and then one of its selections, so it isn't one of these lookups, and it must build against `HEAD` anyway).

## Answer

`GameData` has its lookups, and every `.chain(&…prestige_classes)` outside them is gone.

- **The helpers** (`libs/rules/src/game_data.rs`), in the data's order: each class's own items before its archetypes', base classes before prestige, paths last:
  - `all_classes()`, `all_classes_mut()`: every class.
  - `class(&ClassKey)`: a class by key, base or prestige.
  - `class_selections()`: every class's and archetype's Feature selections. `selections()`: those, then each path's.
  - `class_features()`: every class's and archetype's features.
  - `selection_option(&SelectionOptionKey)`: a Selection option by key, in any selection.
  - The by-key ones walk the lists, for code with a bare `GameData`. The engine keeps `Catalog`'s indexed `class`, `selection` and `option`, so there is no second index.
- **Tests:** seven in `libs/rules/tests/engine/lookups.rs`, against the test world, with the expected keys written out.
- **Sites now using them:**
  - `rules`: `GameData::feat_lists` and `canonicalize`; `check.rs`'s `option_levels` (`selections`) and `counts_toward_features` (`all_classes`, `class`); `engine/pickers.rs`'s `class_options`; `engine/catalog.rs`'s `FeatureFacts::new` (`class_features`, then the paths'); `codec.rs`'s class and option keys.
  - `rules` tests: `check.rs`'s `class_mut`, `canonical.rs`'s `reversed` (`all_classes_mut`).
  - `seeds` tests: `files.rs`'s two `class` closures; `support.rs`'s `class`, `data_prerequisites` and `selection_option`; `class_rules.rs`, `data.rs`, `class_skills.rs` (2), `parameterised_feats.rs`; `borrowed_selections.rs`'s `any_selection` and `option_prerequisite`. The last two and `support::selection_option` now search more than they did (prestige classes, paths). Keys are primary keys, so each lookup still finds the same item.
- **`.chain` left:** only inside `all_classes` and `all_classes_mut`. None was in `libs/proof`, `libs/ui` or `apps`. Loops that need the prestige flag or positions (`check.rs`'s class check, `seed.rs`'s `insert_class`, the catalog's indexes) and single-list finders in single tests stay as they were (decision 5).
- **Checks:** `scripts/proof.sh rules` against `HEAD`: no difference (1999 Builds, 0 differing, empty `compare.txt`). `scripts/check.sh`: ALL-OK.
