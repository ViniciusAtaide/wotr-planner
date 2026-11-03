# Give GameData its lookups

Type: task
Status: open
Blocked by: [82](82-remove-the-extractor.md), [77](77-one-prerequisite-walk.md)
Map: [WotR Build Planner](../map.md)

## Question

`rules::GameData` has no lookup or traversal helpers, so each crate writes its own (architecture review, 2026-11-03). `.chain(&…prestige_classes)` to walk every class appears 8 times in `libs/rules/src` and 11 times in `libs/ui`, `libs/seeds`, `libs/proof` and `apps`. The review counted more in `libs/extract`; [ticket 82](82-remove-the-extractor.md) deletes them (re-scoped after [ticket 75](75-game-data-master.md)).

## Build

- **Add the lookups to `GameData`** in `rules`, with tests: every class (base and prestige), and any by-key lookup (class, selection, option) that two or more sites write by hand. Pick the names; follow the key newtypes in `rules/src/keys.rs`.
- **Use them** at the `.chain` sites and in place of the hand-written finders.
- **Leave the data's shape alone**: base and prestige classes stay two `Vec`s.

**Done when:**
- No `.chain(&…prestige_classes)` remains outside `GameData`'s own helpers.
- `scripts/proof.sh rules <out-dir>` against `HEAD` reports no difference.
- `scripts/check.sh` passes.
