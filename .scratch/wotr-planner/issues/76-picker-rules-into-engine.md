# Move the picker's rules into the engine

Type: task
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The map's Notes put the rules in the `rules` crate and keep the UI as views. The architecture review (2026-11-03) found `libs/ui` deciding rules itself, beside the engine's own versions:

| Where (`libs/ui/src/`) | What it decides | The engine's version |
| --- | --- | --- |
| `picker.rs:707–744` (`Advances`) | the candidate spellcasting-advancement sets: each held class alone and every pair, each tried through `engine.apply` and `is_possible` | `rules/src/engine/walk/level.rs` `check_advances` ("at most one arcane and one divine"); the UI's "at most two" is a second copy |
| `picker.rs:539` `first_taken_at`, `:548` `advances_spellcasting`, `:564` `class_edit` | class entry and the default `advances` | `advances_spellcasting` repeats `advances_arcane \|\| advances_divine` from `check_advances` |
| `picker.rs:852` `racial_modifiers`, `:885` `has_chosen_modifier` | the race/subrace modifier choice | `rules/src/engine/walk.rs` (around 410–413) |
| `picker.rs:623–627` and `:680–690` | "is this the current class", written twice | — |
| `picker.rs:824, 830`; `mythic.rs:45` (prose) | "Mythic rank 8", "Mythic rank 3" as literals | `PATH_SWITCH_RANK`, `PATH_CHOICE_RANK`, which `mythic.rs:6` already imports |

`Engine::data()` is public (`rules/src/engine.rs:63`), which is how the UI reaches round the engine: it scans raw `GameData` at `picker.rs:550, 806, 857, 889, 975, 1108` and `start.rs:91, 283`.

## Build

- **Each rule moves into `rules`** as an `Engine` (or `Catalog`/`labels`) method the picker calls, built test-first (`tdd`, as the map says for `rules`). Name them after what the UI asks, for example the possible advancement sets for a class at a level, a class edit, a race's modifier choice. The possible advancement sets come from the engine's rule, so a class that could advance three no longer depends on the UI's pairs.
- **The duplicated "current class" test** becomes one helper.
- **The mythic rank labels** are built from the constants.
- **Then `Engine::data()` becomes `pub(crate)`**, and each UI scan uses an engine or `Catalog` lookup instead (add the lookup in `rules` where none fits). The compiler then keeps the UI from reaching round the engine again.
- **`libs/proof` calls `engine.data()` too** (`links.rs:37, 56`, `fingerprint.rs:189`, `generate.rs:88`, `builds/mod.rs:33`). Have it keep the `GameData` it builds the engine from instead. `scripts/proof.sh rules` compiles the working tree's harness against `HEAD`'s crates as well, so the harness may use only API both sides have.
- Leave `slot_place` (`picker.rs:313`) and the English labels in `labels.rs` as they are.

The UI shows the same thing before and after.

**Done when:**
- `rules` has no `pub fn data`; `libs/ui` builds without it.
- None of the table's UI functions still decides a rule; each calls `rules`.
- The new `rules` methods have tests.
- `scripts/proof.sh rules <out-dir>` against `HEAD` reports no difference.
- `scripts/check.sh` passes.
