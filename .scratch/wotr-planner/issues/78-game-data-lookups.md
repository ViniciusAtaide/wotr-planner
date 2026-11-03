# Give GameData its lookups

Type: task
Status: open
Blocked by: [75](75-game-data-master.md), [77](77-one-prerequisite-walk.md)
Map: [WotR Build Planner](../map.md)

## Question

`rules::GameData` has no lookup or traversal helpers, so each crate writes its own (architecture review, 2026-11-03):
- `.chain(&…prestige_classes)` to walk every class appears 42 times across `rules` and `extract`.
- Local finders repeat one another: `extract/src/borrowed_selections.rs:797–843`, `follow_ups.rs:85`, `shared_options.rs:365`, `talent_follow_ups.rs:157`, and `references.rs:294, 321` (`prerequisites`, `prerequisites_mut`).

Re-count after ticket 75; passes it deletes need no change.

## Build

- **Add the lookups to `GameData`** in `rules`, with tests: every class (base and prestige), a class by key, a selection and an option by key (and `_mut`), every prerequisite (and `_mut`). Pick the names; follow the existing key newtypes in `rules/src/keys.rs`.
- **Use them** at the `.chain` sites and in place of the local finders, and delete the finders.
- **Leave the data's shape alone**: base and prestige classes stay two `Vec`s, and the string-literal key comparisons in `extract` stay. Note in the Answer any that a lookup made easy to drop.

**Done when:**
- No `.chain(&…prestige_classes)` remains outside `GameData`'s own helpers.
- `scripts/extraction-diff.sh <out-dir>` against `HEAD` reports identical.
- `scripts/proof.sh rules <out-dir>` against `HEAD` reports no difference.
- `scripts/check.sh` passes.
