# Build classes with their picks from the start

Type: task
Status: open
Blocked by: 67
Map: [WotR Build Planner](../map.md)

## Question

**User, 2026-11-03** (in a forked session): "why didn't we started kinetiscits/wild-talents selections from the start in classes? the label servers no purpose on that go", then "yeah lets do this. we should patch the extraction data directly in code and not in runtime".

Today the generic class-table reader from [ticket 10](10-extract-guide.md) turns every level-table cell into a feature. Patch passes appended to `libs/extract/src/build.rs` then find the label, remove it and insert the selection: `wild_talents`, `ki_powers`, `infusions`, `mercies_and_exploits`, `exploits`, `borrowed_selections`, `bonus_feat_choices`, and those of tickets 66 and 67 once they land.

The class-building stage should know which cells are picks and make the slot directly. Each of those modules stops being a patch and becomes a provider that the class stage asks, for a class or archetype and a cell, whether it's a pick, of which selection, and with which options. No label is created and then deleted.

Apply the same idea, where it fits, to the other passes that patch finished data: the extra picks (`extra_picks`, `mythic_extra_picks`, `mythic_ability_picks`, `borrowed_extra_picks`), `archetype_alignments`, `option_levels`, `option_families`, the follow-ups (`follow_ups`, `option_follow_ups`, `second_follow_ups`, `talent_follow_ups`), `class_rules`, `report_gaps` and the rest. Each fact is built where its owner is built.

The few passes whose order is real stay as a short ordered prelude, with a comment giving the reason for each. Examples: resolving prerequisites, dropping dangling grants (`references::drop_dangling`), and sparsifying archetype rows before naming lists (`progressions::sparse_archetype_rows` before `feat_lists::name_lists`). The ticket-history comments in `build.rs` go.

## Build

AFK; the only agent in the repo. A refactor of `libs/extract` only: no change to `rules`, the schema, `libs/game-data/` or the link format. Keep each module's assertions of the guide's sentences (they're the extractor's checks), moved to where the fact is now built. Keep the `settled` report filters working, or replace them with the same effect. Use the RustRover IDE tools for moves and renames where they fit.

**Done when:**
- `libs/game-data/` is byte-identical: `git diff HEAD -- libs/game-data` is empty.
- A fresh extraction into the scratchpad, report included, is byte-identical to one made with HEAD's extractor before the change. Save both and `diff -r` them.
- `cargo test --workspace -- --list` is unchanged apart from moved names. Save the diff, and explain each moved name in the Answer.
- `build.rs` reads as: the stages that build each owner, then the short ordered prelude or postlude with a reason per step. There are no label-then-replace passes left. List any patch pass you kept, and why, in the Answer.
- The lighter per-ticket checks pass: fmt, clippy native and wasm, `cargo test --workspace`, the `rules` wasm build, seed `--check-only`.

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

## Answer
